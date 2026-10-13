# Project audit for the fourth moveset (reference: the reference game's Mario)

What the engine needs to reproduce the reference game's Mario 1:1 (version 13.0.1), traced through the code at sim version 56, before
anything was built. In the game the moveset is **Embers** on the **tumbler** body (class 3, built-in fighter "Tumbler"). The data is
in `REFERENCE_DATA.md`; what was built is in `IMPLEMENTATION_REPORT.md`. The engine inventory this is measured against is
`docs/movesets/SHARED_MECHANICS.md`; the method is the Needles port's (`docs/sheik/`).

## How a normal flows (traced)

`fighter::update` → `ground` (a press of attack, buffered) → `start_ground_attack` (the stick or the attack stick picks the move: a
dash or run gives the dash attack, up / down past `STICK_DOWN` the up / down tilt or smash, sideways the forward tilt or smash; a
smash needs the strong button, the attack stick or a flick) → `begin_attack` (state `Attack`, frame 0, hit mask and charge cleared) →
`fighter::attack` every frame (smash charge hold, rehit rhythm, projectile request, script, motion or air physics or friction, the
end of the move with the jab chain `Move::next`) → `combat::resolve_hits` (active hitboxes by frame, best per target by priority,
clanks, then `apply_hit`: damage, knockback from the reference formula, set knockback, hitlag, hitstun, launch). Aerials go through
`airborne` → `start_air_attack`, landing through `land_out_of_move` (autocancel windows, landing lag, the landing move). Grabs:
`HIT_GRAB` boxes → `grab::connect` → `Grabbing` → `grabbing()` picks pummel or a throw by the stick.

## What Mario's normals need, and what exists

| Needed | Exists? | Decision |
| --- | --- | --- |
| Three-hit jab, each hit continuing from an early frame (5 and 6), presses buffered | partly: `Move::next` continues only at the move's **end**; the Needles reach early frames with a script, but a script cannot see presses made during the jab's hitlag (a 2.2% jab with H 1.8 has 13 frames of it) | **extend** (general): `Move::next_from`, the move frame from which a buffered press carries on at once (the reference's continuable window); default 255 keeps today's end-only chain |
| Held attack repeating jab 1 from frame 7 | yes, by script (`attack` held, `goto`) | a three-line script (`embers_jab_1.script`) |
| Angled forward tilt and forward smash (up / straight / down, the smash with different growth per angle) | **no**: the stick chooses tilt / smash and direction, never an angle | **extend** (general, used later by Captain Falcon and Yoshi too): `Hitbox::aim` (any / up / straight / down) and `Fighter::aim` set as a forward tilt or smash starts, from the stick tilted at least `STICK_AIM` up or down; the hitbox filter in `combat::active_hitboxes` |
| Set weight on the down air's loop (and later Super Jump Punch) | **no** | **extend**: `Hitbox::set_weight`, the knockback worked out at weight 100 |
| Multi-hit down air with a final hit and a landing hit | yes: groups, `rehit` with `rehit_groups`, `landing_move` | data |
| Sweet / sour spots by phase and by part | yes (rows, priority) | data |
| Hitlag and SDI multipliers, set knockback | yes | data |
| Shield damage | yes as a percent | the reference's additive `sd` converted per hitbox [D] |
| Meteor forward air (280) | yes (an angle) | data |
| Back throw hitting bystanders while it spins | yes: a normal hitbox in a throw move never touches the held fighter (`resolve_hits`) | data |
| Throws, pummel, grabs with the reference's reach | yes | data; the pummel's frame-1 hit cannot happen on a move's first frame, so it is one frame later [D] |
| Ledge attack and one get-up attack | yes (one get-up slot) | data; the face-down and trip get-up attacks have no slots (a limit) |
| Up smash head intangibility | **no** (whole-body only) | limit, needs E19 |
| Down tilt trip | no trips | the reference's 40% can never trip at 80 degrees, so nothing is lost |
| Body: walk / run accelerations, traction, jumps, air numbers, dodges | yes (`FighterParams`) | `FighterParams::tumbler` |
| Wall jump | **no** | E15, phase 2 or 3 |

Every extension is additive in the content format (a field with a default that means today's behaviour), hashed (the hitbox and move
fields in `Weapon::hash_into`, `Fighter::aim` in `Fighter::hash_into` with its checksum case), in `docs/CONTENT.md`, and tested in
the topic files (`multihit.rs`: aim and set weight; `kits.rs`: `next_from`). No existing move uses them, so no other fighter changes
(`base.pfc` re-exported; every existing kit test unchanged). One sim version bump (57).

## Specials (phase 2): what is missing

| Special | Missing | Extension |
| --- | --- | --- |
| Fireball | projectile gravity as data, ground bounce (count, speed range, speed kept), wall bounce under 60 degrees, hit phases by age instead of a linear falloff | E3b, and age phases (part of E3a) |
| Cape | a hit that turns the victim round; a reflector that also turns vertical speed, scales the projectile's remaining life, and an early projectile detector; aerial physics of its own; the once-per-airtime fall break; ledge windows by side and frame | E10, E10b, E11, E16 |
| Super Jump Punch | the move's own rise (root motion) and stick angling, helpless drift and landing lag per move, ledge grab from frame 14 both sides | E11 (and research: the rise is not published) |
| F.L.U.D.D. | a flinchless push with no damage and no hitstun, per-shot strength and size from the charge, aim by stick | E2, a `spawn` variant carrying the charge's knockback and size (a script ability) |

The phase-2 plan with the exact shape of each extension is in `IMPLEMENTATION_REPORT.md`.
