# Project status and how to resume

Read this first when picking the project up again. The design source of truth is `docs/Platform_Fighter_Project_Plan.docx`;
this file records where the build is against its roadmap (section 8) and what to do next. Last updated after the
Phase 7 work (complete): match rules, HUD, results, rematches, the online screen, replays, spectating, groups of 3 to 4 and Quick match; then music and two movement-tuning passes toward the reference game, `docs/PHASE1_MOVEMENT.md` (sim version 30: the 9-frame and hold input buffer, short-hop aerials, light and heavy landings, shield release, air dodge timings and the reference ledge rules; sim version 33: full ground speed and the committed initial dash; sim version 34: DI at the reference's 10 degrees, launch speed influence, hurtboxes lower in a crouch and on the claws' limbs, and the bruiser on the reference hammer heavyweight's attributes with four air jumps (`docs/FEEL_AUDIT.md`); sim version 35: less slippery (landing walks rather than runs, traction on overspeed and on the dash attack's slide) and the reverse aerial rush out of a dash or a run's skid (`docs/PHASE1_MOVEMENT.md`); sim version 36: running jumps take a reduced, capped share of the ground speed, bodies on the ground do not pass through each other (`sim-core/src/push.rs`), and the maul class is removed for now (two classes: longsword and claws); a basic computer opponent for testing and playing alone (`docs/CPU.md`; C in the character select, F5 in training); sim version 37: running off a platform takes speed into the air like a jump, and the claws' throws are the reference's hit-then-throw (combo throws, one kill throw); sim version 38: the jabs, dash attack, grabs, pummels and throws on the reference's published frame data (`ultimateframedata.com`; the aerials, tilts and smashes already matched); hit feel in `docs/COMBAT.md`, the feel audit in `docs/FEEL_AUDIT.md`, recorded CC0 hit sounds in `docs/AUDIO.md`). Since then: sim versions 39 to 44 (reference launches and fast falls, missed techs,
rolls stopping at edges, the special fall, up specials reversing with up-and-back, the ledge jump peaking the reference's height above
the stage, the get-up attacks on reference numbers); the base body with modelled clothes; menus in the concept art's painted style with
a bundled OFL font (`docs/MENUS.md`); a Mii-style character creator with faces built from placeable parts (`docs/ART_PIPELINE.md`); and
a visual quality pass on the menus (one light, depth, one card language). Then a rendering framework (`docs/VISUAL_STYLE.md`: shading
ramps, stage profiles, the look bench, display settings; an art pass with restrained grading, contact shading under props, deeper haze and
crisp cast shadows); B-reverse (sim 45); a 3-frame stick flick for dash and fast fall, so a controller's flick registers (sim 46); and the
shared animation library (`docs/ANIMATION.md`: one clip set for every moveset, mapped in `godot/scripts/anim_library.gd`, per-hit strikes
for multihit moves, a polished idle, jump, landing, jab, forward air and hit reaction). Then the multi-hit engine and a third class
(sim 49): eight hit groups per move, a rehit that clears chosen groups, the autolink angle 367, set knockback, extra hitstun, per-hitbox
SDI and landing hits; and the Needles rogue, whose body, tilts and aerials follow the reference's published data (`docs/sheik/`,
`docs/MOVESETS.md`). Sim 50: the rest of its normals, grabs and throws on reference data (its specials are still the claws kit's), and
three general rules brought to the reference's: the 361 angle, traction slowing grounded slides, and light hitstun landing on its
feet (no knockdown after a drag-down). Sim 51: its specials (Needle Storm, a pulling line of our own, Vanish, Bouncing Fish), the
pull angle (368) and a stored charge for scripts (`store`, `stored`). Sim 52: Needle Storm charges and stores as the reference, Bouncing
Fish's early kick. Sim 53 (from playtest notes): a dash takes a fuller flick (0.75 instead of 0.6, `STICK_DASH`) so a quick push
stays a walk; a run's stick rolled along the bottom or top turns it (the reference's reverse aerial rush motion), since down and
forward no longer crouches out of a run; and the guard held out in front is a hurtbox (`sim-core/tests/stick_feel.rs`). Sim 54
(content schema 3): every fighter's ground movement is its reference character's, from the parameter file: walk and run
acceleration by the stick, the rogue's 7-frame initial dash, heavy landings (`sim-core/tests/ground_attributes.rs`). Then the graphics fidelity pass (branch `graphics-fidelity`, `docs/GRAPHICS_FIDELITY_PLAN.md`, results and frame
times in `docs/GRAPHICS_FIDELITY_LOG.md`): painted facades and glowing windows on the skyline, per-theme value range and more contrast on
every stage, far scenery softly out of focus (Best), painted surface wear and hit scuffs, leafy tree crowns in the wind, a banded sea
with foam. Every phase measured within about 0.1 ms of GPU time of the baseline (Treetop Isle draws fewer calls than before); the look
bench is unchanged. Phase 7 (authored prop models) waits on the owner's choice of source. Then a visual overhaul (branch `visual-overhaul`,
`docs/VISUAL_OVERHAUL_PLAN.md`, audit and results in `docs/VISUAL_OVERHAUL_AUDIT.md` / `_LOG.md`): SMAA, per-material light responses
and a real blade, built stage props and girders, varied skyline buildings with signs, three hit tiers, HUD refinements (and cards that fit
four players), and a four-computer stress demo (`--demo=cpu4 --players=4`). Then an art-direction polish (branch `art-direction-polish`,
`docs/ART_DIRECTION_POLISH_LOG.md`): the attention hierarchy, palette and material rules written into `ART_DIRECTION.md`, calmer coral
girders, skyline buildings in varied muted hues with clustered windows and broken-up office strips, a quieter roof wall, and "FREE PLAY"
said once on the HUD. Then a new base body modelled by script after the concept sheet (`art/blender/make_base_body.py`,
`docs/ART_PIPELINE.md` twenty-eighth pass): the concept's proportions, bare by default (a new Gear slot), matte skin and a warmer, thinner
outline; it replaces the Tripo model, whose licence was unclear. Then magic-hour painted skies per stage, a golden raking key and
violet shadows on Meadow, straw hay bales and a built fence, and drifting motes and light shafts (`docs/ART_PIPELINE.md` twenty-ninth pass). Then the fighter motion polish (`docs/MOTION_POLISH_PLAN.md`, results in
its "Results" section; branch `motion-polish`): strike stretch, impact squash into an eased launch stretch, smear ribbons on punches and
kicks, a spring set for the chest, head, hat, fist and neckwear, cloth tails on spring chains, new clips (tech, platform drop, run stop,
heavy hitstun, an idle fidget), a fix for the head spinning in a long shield hold, and a short-contact run that no longer skates.

Moveset ports from the reference game (`docs/movesets/ROSTER_STATUS.md`): **sim 57**, the fourth class, Embers on the tumbler body
(modelled on the reference game's Mario), phase 1: the body and every normal on reference data (`docs/movesets/mario/`), with early jab
continuation (`Move::next_from`), angled forward tilts and smashes (`Hitbox::aim`) and set-weight hits (`Hitbox::set_weight`); its
specials are still the claws' until phase 2. **Sims 58 to 64** (phase 2, branch `movesets`): its four specials on reference data
(the rising punch, the bouncing fireball, the cape, the water pump) and the general pieces they needed: per-move recovery rules (ledge
windows by side and frame, the special fall's drift and landing lag per move, height multipliers, stick angling, a move's own air
physics, later or air-only intangibility, carrying on after touching down), set-knockback launches keeping plain launch physics,
projectile arcs, ground bounces and hits by age, reversing hits, push hits, fuller reflectors (vertical reversal, lifetime, damage limit,
projectile detector), once-per-airtime script flags, charged scaled shots, a 16-slot projectile pool, pummels with their own hitlag on
the reference's timing, and wall jumps (content schema 4). The Longsword's and the Claws' up specials were re-fitted to their
references with the new rules. **Sim 65** (owner's follow-up): a full water-pump charge holds its stance until special or shield, and
jump pressed against a wall (stick neutral or toward it) also wall-jumps without spending a midair jump. Details and what is still
estimated: `docs/movesets/mario/IMPLEMENTATION_REPORT.md`.
**Sim 66** (content schema 5): the fifth class, Gauntlets on the sprinter body (built-in fighter Sprinter, modelled on the reference
game's Captain Falcon), phase 1: the body (15-frame initial dash, wall jump 1.4 / 3.1) and every normal on reference data
(`docs/movesets/captain_falcon/`): the jab chain with the rapid jab's entry rule (three presses and two releases since jab 1, presses in
hitlag counted), its loop and finisher, the one-frame knee, the down air meteor, angled forward tilt and smash. General pieces: sixteen
`ext` slots (E17, schema 5), per-limb intangibility (E19, `Move::part_intangible`), per-hitbox shieldstun (E24, `Hitbox::shieldstun`)
and the `presses` / `releases` script registers. Its specials are still the claws' until phase 2 (plan in
`docs/movesets/captain_falcon/IMPLEMENTATION_REPORT.md`).
**Sim 67** (branch `seer-phase1`, no schema change): the sixth class, Staff on the seer body (built-in fighter Seer, modelled on the
reference game's Palutena), phase 1: the body (no wall jump, heavy landing 6 from the parameter file) and every normal on reference data
(`docs/movesets/palutena/`): jab 1 into the rapid jab (two presses and releases, or held on a hit), the multi-hit up tilt, neutral air
and up air with their last hits, the smashes' wind as pushes, the up smash's pillar. General pieces: angle types (E26,
`Hitbox::angle_type`), applied to the older classes too (owner decision 21: each change in `sim-core/tests/angle_types.rs`), and a
signed hitstun modifier (E27). Its specials are still the claws' until phase 2 (plan in
`docs/movesets/palutena/IMPLEMENTATION_REPORT.md`).
**Sims 67 to 71** (branch `falcon-phase2`, merged into `movesets` at **sim 72**): the Gauntlets' four specials on reference data, with
the reference's exact shield stun for every fighter (whole-number stuns were a frame short; the changed moves are listed in the
Gauntlets' report), armor (E1), frame-ranged air physics with fast-fall and turn frames (E11 additions), search boxes (E21), command
grabs (E6), no-clank hits (E3c), off-edge versions (E22) and the `wall` script register. **Sim 73** (branch `cleanup-72`): the
reference's angle type 1 (from the direction of travel) on the Claws' up special loop, every Gauntlets special's angle type, the Staff's
no-clank boxes, the Longsword's, Claws' and Needles' get-up attacks at the reference's angle 48, and a second source for the Sprinter's
42-frame neutral air dodge (owner decisions 18 and 23 to 27). The Gauntlets are now **playable** (travel distances [E]); details in
`docs/movesets/captain_falcon/IMPLEMENTATION_REPORT.md`.
**Sims 74 to 77** (branch `seer-phase2`, from `movesets` at sim 72 with the sim 73 cleanup merged in; no schema change): the Staff's
four specials on reference data (`docs/movesets/palutena/IMPLEMENTATION_REPORT.md`, tests `sim-core/tests/staff_specials.rs`): the
warp (74), the counter and reflect barrier (75, with E10c: melee-only counters with a damage cap, a projectile detector that starts
the reflection move, a standing-capsule reflector with a body field, `air_max_rise`, counters catching through their own
intangibility), the explosive flame (76, with E3a: piercing, per-target rehit and eight phases for projectiles, the `flick` register
and `spawn_at`), and the reticle (77, the `lock(range)` target lock with a line-of-sight test, `collision::segment_blocked`). The
Staff is now **playable**; the open [E] readings are listed in its report.
**Sim 78** (branch `counter-fix`, owner decisions 28 and 29; no schema change): counters never catch a push (a windbox hit), and a
counter catches on its own bubble (`Counter::bubble`, the reference's counter bubble) instead of the body's hurtboxes: the Longsword's
radius 10.5 at 10 up, the Staff's radius 11.5 at 10 up and 1.5 ahead [R]. The Longsword's answer is also capped at the reference's
50%, and a grab or a push no longer reaches through a counter stance's intangibility (only the hits the counter catches do). Tests in
`sim-core/tests/kits.rs` and `staff_specials.rs`; details in `docs/movesets/SHARED_MECHANICS.md` ("Counter").

**Sim 79** (branch `grab-dash`; 78 is the moveset session's counter fix): a dash flicked at any point of a grab, pummel or throw
comes out on the first frame the fighter can act if the stick is still held (`Fighter::dash_buffer`), so throws can be chased; the
reference buffers a dash only in the last 6 or 7 frames (owner's call to be more lenient). Tests `sim-core/tests/grab_dash.rs`.
**Sim 82** (branch `dash-buffer-landing`, on `movesets` at sim 80): landing lag keeps a dash the same way, and the keyboard sends a
direction key pressed during grab, throw or landing lag as a flick (a first press was otherwise a walk, so keyboard players could not
buffer a dash at all); tests in `grab_dash.rs`, `landing_buffer.rs` and `godot/tests/input_e2e.gd`.
**Sim 115** (branch `pockets-phase2`, from `movesets` at cb89aa5, sim 113, with `movesets` (sim 114) merged in; content schema 7 unchanged): the Pockets' phase 2, every special of the settler now built. The older jab reworked (owner decision 70: the jab cancel, `Move::cancel_from`, from jab 1's frame 7 and jab 2's 8; 13.0.1's far-box pull on jab 1); **E42 the pocket** (catch any other owner's projectile or a free item, keep it frozen, take it out at x1.9; `Tool::pocket`, `ItemSpec::pocketable`, the `pocketed` register, the bridge's `fighter_pocketed`); **E43 placed pieces** (the sapling, watered into the tree, chopped into the falling tree and a wood chip; `ItemSpec::placed`, `Tool::water`, `Move::chops`, `combat::fire`, the `owns(kind)` function). Owner decisions 69-71. Tests `pockets_pocket.rs`, `pockets_tree.rs`, `older_rules.rs`; the fuzzers drill the settler's pocket and tree. Report: `docs/movesets/villager/IMPLEMENTATION_REPORT.md`.

**Sim 113** (branch `pockets-phase1`, from `movesets` at 71cbbed, sim 111, with `movesets` 336b974 (sim 112) merged in; content schema 7 unchanged): the second wave begins (owner decisions 65-68). The eleventh class, **Pockets** on the **settler** body (built-in fighter **Settler**, modelled on the reference game's Villager), phase 1: the body and every normal on 13.0.1 data (the flowerpot, bowling ball and slingshot pellets as projectiles, the turnips' random counts, the firework, the shovel's bury), the rocket (side special: let go or ridden) and the balloons (up special). **The jab is the previous reference version's (1.1.x)** on that version's hit rules (decisions 66-67). Engine, all general and defaulted: **E40** `Hitbox::rules` (`rules previous`: no one-on-one multiplier, the old hitlag with a victim-only crouch, the old rage, +1 tumble hitstun, no launch speed-up, the gravity boost at every angle, the victim's own vertical fall; the ruleset's `prev_*`, hashed `Fighter::launch_older`), **E39** `Move::versions` (random versions drawn on a move's first frame), **E41** `Move::balloons` (hashed `Fighter::balloons` / `balloon_hp` / `helium_used`, `sim-core/src/balloons.rs`, the `balloons` register), `ChargedShot::power_to`, `Burst::on_hit`, hashed `Item::angle` with the bridge's `item_angle` (for the art director) and `fighter_balloons`. Nothing changes for the older fighters. Tests `sim-core/tests/pockets.rs` (19), `pockets_specials.rs` (19), `older_rules.rs` (11) and cases in `scripted_moves.rs`, `buckler_specials.rs`, `state.rs`, `trig.rs`, `ground_attributes.rs`, `sim-content` (round trip). The balloons' and the ridden rocket's numbers are unpublished: [E], an owner question. Report: `docs/movesets/villager/IMPLEMENTATION_REPORT.md`.
**Sim 111** (branch `buckler-phase3`, from `movesets` at 85d352a, sim 110; **content schema 7**, migration `weapons_get_items`, which rewrites nothing): the Buckler's phase 3 and **E4 items** (owner decision 64). Every fighter can pick up a lying or stuck item (grab, or an attack that is not a smash input), hold it through specials, jumps, dodges and the shield, throw it forward, back, up or down (smash throws with a smash input, the dash throw out of a run), drop it (grab with the stick in the middle) or let go of it in the air with no lag, on its body's reference release frames (`FighterParams::item_throws`) and the 13.0.1 totals (`Ruleset::item_*`); a hit, a grab, a ledge or a KO lets go of it. Engine: `GameState::items` (hashed `Item`), `Fighter::holding` / `item_order` / `item_spec` / `item_throw`, states `ItemPickup` / `ItemThrow`, `moves::ItemSpec` (`Weapon::items`), `Move::pull` / `spend_item`, `ProjectileArc::lodge_item`, `ChargedShot::times`, `sim-core/src/items.rs`, the `holding` register and `detonate()`. The Buckler: the **bomb** (pulled on frame 17, 1800-frame fuse, detonated 12 frames after the press, self damage, 50 hp, bounces, anyone can pick it up), **lodged arrows as items** and the **double arrow**; the passive shield is off while holding; the single arrow's radius fixed (it was 1.7 world units, the reference's is 1.35 reference units). Changed for older fighters: an attack or grab next to a lying item picks it up (no item lies about unless a wanderer makes one). Read-only bridge accessors `item_count`, `item_pos`, `item_kind`, `item_holder`, `item_rest`, `item_life_left`, `fighter_holding`. Tests `sim-core/tests/items.rs` (23), `scripted_moves.rs`, `move_rules.rs`, `bundle.rs`; the fuzzers now play with items and count them. Report: `docs/movesets/link/IMPLEMENTATION_REPORT.md`.
**Sim 110** (branch `clutch-phase3`, from `movesets` at af0949e and merged with `movesets` at 885a44e, sim 106; content schema 6 unchanged: the new block and effect are optional): the Clutch's phase 3, the **tongue and its egg** (neutral special: a command grab on 19-22 that swallows and lays its catch as an egg, the reference's 84-264 frames, mashing, 0.66x damage, no launch, growth, shift and hop, the break-out) and a simple **rolling shell** (side special, decision 49). Engine: **E8** (`Move::encase`, `EFFECT_ENCASE`, `FighterState::Encased`, hashed `Fighter::encase_timer` / `encase_age` / `encase_char` / `encase_move`, `grab::mash_inputs`, `scripting::World::encase`), general and unused by every other weapon (`base.pfc` re-exported). A read-only bridge accessor `fighter_egg_radius(i)`. Tests `sim-core/tests/clutch_egg.rs` (20), `move_rules.rs`, `bundle.rs`; report `docs/movesets/yoshi/IMPLEMENTATION_REPORT.md`.
**Sim 102** (branch `buckler-phase2`, from `movesets` at 9f2383b and merged with `movesets` at 238868c, sim 101; content schema 6 unchanged: every new field is optional and defaults to today's behaviour): the Buckler's phase 2, the **bow** (neutral special: draw, hold to 180 frames, release; charge-scaled arrows that lodge for 180 frames), the **boomerang** (side special: aimed tilt and smash-input throws, out-and-back flight with homing, hits by age, one at a time with a failed throw, the catch, a clank drop) and the **sword wave** (the forward smash at exactly 0%, scaled by its charge). Engine, all general: **E35** charge-scaled shots (`ProjectileSpawn::charged`, hashed `Projectile::damage`, `ProjectileArc::drag`), **E3b** lodging (`ProjectileArc::lodge` / `lodge_angle`, hashed `Projectile::rest`), **E3d** tool life rules (`ProjectileSpawn::tool`: kind, limit, owner catch and catch move, clank fall; hashed `Projectile::kind`; the `owned` script register; `Move::smash_version`), **E38** lying capsules (`Hitbox::half_width`, `combat::shapes_touch`). Changed for older fighters: nothing (no other weapon uses the fields; `base.pfc` re-exported). A read-only bridge accessor `projectile_rest(i)`. Tests `sim-core/tests/buckler_specials.rs` (20), `scripted_moves.rs`, `move_rules.rs`; report `docs/movesets/link/IMPLEMENTATION_REPORT.md`.
**Sim 100** (branch `platform-special`, from `movesets` at 9f2383b and merged with `movesets` at sim 98; content schema 6 unchanged: the new ruleset field is optional and defaults to the standard): owner decision 63, a down flick on a pass-through platform no longer drops on the spot. The fighter stays on the platform, crouching, for `Ruleset::platform_drop_frames` (2, derived from the reference: Terry's 11-frame drop is the usual one plus 9 extra frames) and falls on the frame after that, so 2 frames later than before. A special on the flick's frame, on either waiting frame or on the frame it would fall (3 frames in all) comes out on the ground instead, with the stick down the grounded down special. Attacks and grabs don't interrupt the drop and come out in the air after it, as before. The carry is unaffected. New hashed state `Fighter::drop_pending`. Every fighter changes (the drop is 2 frames later); `base.pfc` re-exported. Tests `sim-core/tests/platform_drop.rs` (11); fuzz-rollback 500 and net-fuzz 200 at zero.
**Sim 98** (branch `bracers-phase3`, from `movesets` at 81d5f50 and merged with `movesets` at sim 97; content schema 6 unchanged: the new block is optional): the Bracers' phase 3, the **carry (E7)**: the forward throw is the reference's cargo hold, a 15-frame invincible lift into `FighterState::Carrying` (`Move::carry`: walk 110%, clock `hold x 11 / 6 - g` with the grab's time and mashing carried over, one jump, the victim overhead, knockback armor 80 not subtractive [E], four throw slots, no ledge [E]); hashed `Fighter::carry_jumps` and `grab_start`; the four cargo throws in ext10-13 (the down one's stop and boost by script). Changed for older fighters: nothing (no other weapon has a carry; `base.pfc` re-exported). A read-only bridge accessor `fighter_grab_partner(i)` for the view. Tests `sim-core/tests/carry.rs` (13); fuzz-rollback 500 and net-fuzz 200 at zero. Report: `docs/movesets/donkey_kong/IMPLEMENTATION_REPORT.md`.

**Sim 95** (branch `clutch-phase2`, from `movesets` at 457731b, merged with `movesets` at sim 94 (the Bracers' phase 2); content schema 6 unchanged: every
new field is optional with a default): the Clutch's phase 2, the lobbed shell (up special) and the drop (down special), and owner
decisions 58-59. Engine, general and off by default: a jab's continuable window past its FAF (`Move::late_next`, decision 58; given to
every jab whose reference window runs past its FAF: the Embers' jabs 1-2, the Gauntlets' jabs 1-2, the Greatsword's jab 1, the Bracers'
jab 1, the Clutch's jab 1), a body's air-jump drift (`FighterParams::air_jump_drift`, decision 59; the bounder 120), ledges only while
not rising (`Move::ledge_falling_only`), a move's own soft-platform drop (E30, `Move::drop_through`), two script shots on one frame (E3h),
bounce limits (`ProjectileArc::bounce_keep_y`, `bounce_max_fall`) and burst triggers (`Burst::on_ground`, `on_expire`, `keep_hits`).
Tests `sim-core/tests/clutch_specials.rs`, `late_jab.rs`, `move_rules.rs`, `scripted_moves.rs`; report
`docs/movesets/yoshi/IMPLEMENTATION_REPORT.md`.

**Sim 90** (a placeholder: the owner asked for 90 on this branch, but `movesets` is at 93 by its end, so it must be renumbered above 93 at the merge; branch `bracers-phase2`, content schema 6 unchanged): the Bracers' phase 2, every special of the hauler (modelled on the reference game's Donkey Kong): the charged punch, the headbutt and its bury, the spinning clothesline (ground and air), the hand slap (ground and air). Engine, all general: the **base 7% trip** for every fighter (decision 50: a hit over 55 knockback that would not lift trips 7% of the time, a move's own chance gains 7; `Ruleset::trip_base` / `trip_base_kb`), the older kits' trip chances as data (decision 51: the Claws' down tilt can now trip), the `aimed` script register and a read-only `fighter_tripped(i)` bridge accessor (decision 52), **E29** additive charge damage (`Move::charge_add`, script `set_charge`), **E5 the bury** (`EFFECT_BURY`, `FighterState::Buried`, hashed `bury_timer` / `bury_kb` / `bury_angle` / `bury_set`, `Ruleset::bury_*`, the shared mash helper `grab::mash_cut`; bury hits ignore subtractive knockback armor), and push immunity windows (`Move::push_immune_from` / `_to`). Changed for older fighters: the base trip (all), the Claws' down tilt trip. Tests `sim-core/tests/bracers_specials.rs`, `bury.rs`, `trips.rs`; fuzz-rollback 500 and net-fuzz 200 at zero. Report: `docs/movesets/donkey_kong/IMPLEMENTATION_REPORT.md`; phase 3 (the carry, E7) next.

**Sim 89** (branch `clutch-phase1`, on main at 64147b0 (sim 87; 88 is reserved for another session), content schema 6 unchanged): the ninth class, Clutch on the bounder body (built-in fighter Bounder, modelled on the reference game's Yoshi, owner decision 45), phase 1: the body (weight 104, the reference's ground, air, jump and dodge numbers, the 51.56-unit double jump) and every normal on [R] data. Engine, all general and off by default: the E1 **knockback armor kind** (`Armor::subtract`, `Armor::min_kb`, `withstands` sees the knockback) and **E33**, a body's air-jump armor and turnaround (`FighterParams::air_jump_armor`, `air_jump_turns`, hashed `Fighter::air_jump_frame`; optional fighter fields, no schema bump). Tests `sim-core/tests/clutch.rs`, `multihit.rs`; report `docs/movesets/yoshi/IMPLEMENTATION_REPORT.md`.

**Sim 96** (branch `buckler-phase1`, merged with `movesets` at sim 95): owner decision 62, the up special and up smash also cancel a jump squat for every fighter (a press after the jump's own frame; an up smash input for the attack), and decision 58's window past the FAF on the Buckler's jabs (to 35). Decision 61: the aerial spin's 5 of 9 hits on a target in front stays [E].

**Sim 91** (branch `buckler-phase1`, from `movesets` at 88adf03 and merged up to 9385f20 (sim 89; 90 is Donkey Kong phase 2 on its own branch), content schema 6 unchanged): the tenth class, Buckler on the wanderer body (built-in fighter Wanderer, modelled on the reference game's Link, owner decision 53), phase 1: the body (weight 104, the reference's ground, air, jump and dodge numbers) and every sword normal and both up specials (the grounded charged spin and the aerial multi-hit spin) on [R] data. Engine, all general and off by default: **E34** position vectors (`Hitbox::posvec_x` / `posvec_y` / `posvec_frames`, hashed `Fighter::posvec`, `LINK_POSVEC`), **E36** (`Move::keep_charge`), **E11** special-fall and air-block speed and stick acceleration (`Move::fall_speed`, `fall_accel`, `AirBlock::speed`, `accel_stick`), **E14** the passive shield (`FighterParams::passive_shield`, hashed `Fighter::shield_sdi`), the `fast_fall` script register, and **E37** (owner decision 56): every fighter can start its grounded up special (special with the stick up) or its up smash (attack with the stick or the attack stick up) straight out of a shield, with no jump squat. Tests `sim-core/tests/buckler.rs`, `scripted_moves.rs`; report `docs/movesets/link/IMPLEMENTATION_REPORT.md`.

**Sim 86** (branch `bracers-phase1`, on `movesets` at f2b69b8, content schema 6; the number is a placeholder the owner renumbers at the merge): the eighth class, Bracers on the hauler body (built-in fighter Hauler, modelled on the reference game's Donkey Kong, owner decision 37), phase 1: the body (weight 127, the reference's ground, air, jump and dodge numbers, a bigger body [E]) and every normal on the raw hitbox tables' data, with two small general extensions: E20 trips (`Hitbox::trip`, a percent rolled on the game's own hashed generator only for a hit that can trip; the tripped fighter sits with the reference's hard-trip timings: `Fighter::tripped`, `Ruleset::trip_lag` / `trip_max` / `trip_invuln`) and E3g hits that clank without rebounding (`Hitbox::rebounds`). Older fighters changed by the reference data: the Gauntlets' forward air late hit now trips (20%), the Longsword's down tilt carries its 35% (it lifts, so it never trips on a flat stage), and the Claws' up smash and the Staff's forward, up and down tilts and neutral air clank without rebounding. The specials are still the claws' placeholders and the forward throw a placeholder throw. Report and the phase 2 / 3 plans: `docs/movesets/donkey_kong/IMPLEMENTATION_REPORT.md`. Tests `sim-core/tests/bracers.rs`, `trips.rs`.

**Sim 87** (branch `greatsword-phase3`, on `movesets` after f2b69b8, merged to main; content schema 6, no schema change: the new fields
are optional): the Greatsword's phase 3, its four powered versions (the bursting wave on the ground and in the air, the self-running
combo slash, the higher rise with air control, the finisher on the ground and in the air), and every special in the powered mode now
spends the gauge. Engine, each general and off by default: E3e bursting projectiles (`ProjectileSpawn::burst`, hashed
`Projectile::bursting`), E28 the vortex angle 366 (hashed `Fighter::link_kind` and `vortex`; the combo slash's nearest box uses it),
E11 `Move::motion_drift`. The fuzzers (`pftool fuzz-rollback`, `pftool net-fuzz`) now draw fighters from every class in the content,
built-in or made, and print a per-class count (before, only classes 0 and 1 ran): `fuzz-rollback 20000` and `net-fuzz 2000`, 0
mismatches over all seven classes; no desync was found. Tests `sim-core/tests/greatsword_powered.rs` and `netplay/tests/powered_rollback.rs`;
report `docs/movesets/cloud/IMPLEMENTATION_REPORT.md`.

**Sim 85** (branch `greatsword-phase2`, on `movesets` at a88c09f, content schema 6; sim 84 is reserved for the engine session): the Greatsword's phase 2, its four normal specials (the wave, the combo slash, the rising and falling slash, the gauge charge) and the engine they need: E9, a body gauge with a timed powered mode that multiplies the body's movement (the champion's Boost gauge; gain from damage dealt and taken, drain while charging, reset on a KO), E16 `boost` / `boosting` / `grab` / `boost_add`, E11 `start_speed_y` and `ledge_drop`, E22b `on_land`, E3f standing capsule hitboxes. Read-only bridge accessors for the HUD: `fighter_gauge`, `fighter_gauge_max`, `fighter_gauge_full`, `fighter_gauge_frames`. Every other fighter unchanged. Report: `docs/movesets/cloud/IMPLEMENTATION_REPORT.md`; phase 3 (the powered versions, E3e, E28) next.

**Sim 83** (branch `greatsword-phase1`, on `movesets` at sim 82; no schema change): the seventh class, Greatsword on the champion body
(built-in fighter Champion, modelled on the reference game's Cloud, owner decision 30), phase 1: the body (weight 100, wall jump 1.3 /
2.4, heavy landing 3) and every normal on the reference's data read from ssbwiki.com's raw hitbox tables (`docs/movesets/cloud/`): the
jab chain's continuable windows and held repeat, the forward smash's three set-weight hits, the up air as one hit, the forward and down
air meteors, hit-then-throw throws. No engine change. Its specials are still the claws' until phases 2 and 3 (the Boost gauge E9,
E3e, E3f, E11 and E16 additions, E22b, E28: plan in `docs/movesets/cloud/IMPLEMENTATION_REPORT.md`). Tests `sim-core/tests/greatsword.rs`.

## Roadmap progress

| Phase | State | Where to read |
| --- | --- | --- |
| 0 Foundation (fixed point, state, checksum, headless runner) | done | `README.md`, `CLAUDE.md` |
| 1 Movement | done | `docs/PHASE1_MOVEMENT.md` |
| 2 Local rollback | done (1000 randomised runs, 0 mismatches) | `netplay/src/local_rollback.rs` |
| 3 Combat (hits, DI, shields, grabs, techs, training mode) | done | `docs/COMBAT.md` |
| 4 Netplay (rollback, UDP, relay, handshake, desync detection) | built and tested locally; **never tried over the real internet** | `docs/NETPLAY.md` |
| 5 Content format and scripting VM | done | `docs/CONTENT.md` |
| 6 Editors (look, fighter, move, stage, package) | done, **untested by a human creator** | `docs/EDITORS.md` |
| (extra) Both characters' kits completed | done (estimates flagged in `docs/COMBAT.md`) | `docs/COMBAT.md` |
| (extra) Main menu, character creator, character select | done | `docs/MENUS.md` |
| (extra) Online play with made fighters, hitbox size scaling, point budget | done | `docs/MENUS.md`, `docs/NETPLAY.md` |
| 7 Game loop: stocks, winner, HUD, results, rematch (local and online), online screen | done, see `docs/MATCHES.md` | `docs/MATCHES.md` |
| 7 Game loop: replays (record, verify, watch) | done, see `docs/MATCHES.md` | `docs/MATCHES.md` |
| 7 Game loop: key rebinding, controller input and menus | done (controllers untested on hardware) | `docs/MENUS.md` |
| 7 Game loop: local 3 to 4 player free-for-alls | done (needs controllers 3 and 4; untested on hardware) | `docs/MATCHES.md` |
| 7 Game loop: stage select (4 stages, online and replays) | done | `docs/MATCHES.md` |
| 7 Game loop: spectating (direct hosts) | done | `docs/MATCHES.md` |
| 7 Game loop: online 3 to 4 players (hub host and lobby) | done, see `docs/MATCHES.md` | `docs/MATCHES.md` |
| 7 Game loop: matchmaking | **Quick match** done (relay queue pairs two waiting players); a list of games, ratings and regions are not built | `docs/MATCHES.md` |
| 8 Public alpha checklist | see `docs/ALPHA.md` (what is ready, unverified and not built) | `docs/ALPHA.md` |
| 8 Content and polish (real art/animation, audio, balance tooling, moderation) | **started**: rigged art and animation, soft cel shading and shared lighting, drawn SVG faces and cloth prints (`docs/ART_PIPELINE.md`), sound effects and synthesised music (`docs/AUDIO.md`), balance tooling (`docs/BALANCE.md`), content policy and name moderation (`docs/CONTENT_POLICY.md`) | `docs/ART_PIPELINE.md` |

## What "done" is verified by

`cargo test --workspace` (about 480 tests), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all`,
`pftool fuzz-rollback 1000` and `pftool net-fuzz 2000` (both 0 mismatches, fighters drawn from every class since sim 87), and the headless Godot tests
(`godot/tests/*.gd`, run with `Godot --headless --path godot --script res://tests/<name>.gd`):
`input_e2e`, `net_e2e`, `content_e2e`, `loadout_test`, `editor_api_test`, `editor_ui_test`, `creator_flow_test`, `replay_test`, `controls_test`, `sfx_test`, `fighter_view_test`, `policy_test`,
`online_fighters_test`, `match_flow_test`, `online_flow_test`, `cpu_test`. All of them pass at the last commit. Godot is expected at `tools/godot/Godot_v4.7.1-stable_win64_console.exe`
(not in git); Rust stable with the MSVC build tools builds the rest. `play.bat` opens the game, `play_editor.bat` the editors,
`play_host.bat` / `play_join.bat` an online match.

## Known gaps and honest caveats (do not lose these)

* **Real-internet netplay is untested.** Everything is localhost or simulated. A friend playtest is pending (see the memory note
  "netplay playtest pending"). No NAT traversal: port-forward UDP 47000, or run `pftool net-relay`.
* **Nobody has used the menus or editors by hand.** Tests drive them, screenshots look right, but feel is unproven.
* **Frame data**: many hitbox positions, throw knockbacks and landing lags are estimates (listed in `docs/COMBAT.md`). Pivot grabs
  were left for the user to test later.
* **Online has a form, not a game list.** The online screen (menu, Online) picks role, mode, connection, fighter, delay and (host) rules, and group matches have a
  lobby, plus Quick match through a relay, but there is no list of games. Nobody has used it over a real network.
* Controllers (play and menus) are implemented but untested on real hardware. The art is a rigged blob with a cel shader, shared lighting and drawn SVG faces and prints (`docs/ART_PIPELINE.md`); hats and glasses are still built in code.
* **Renderer**: Windows now uses Vulkan (Direct3D 12 stalled for milliseconds whenever a mesh was made during play; see the twenty-first
  pass in `docs/ART_PIPELINE.md`). Measure with `--perf`; the developer's laptop caps even an empty scene near 50 to 65 fps.
* Four stages, one mode (versus; local and online up to 4 players). No sudden death after a tied clock.

## Phase 7: what is left, suggested order

Done: online 3 to 4 players, stage select, stocks, elimination, winner, time limit, HUD, results, local and online rematch, online screen, replays (`docs/MATCHES.md`), key
rebinding and controller input/menus (`docs/MENUS.md`; the controller part is untested on hardware).

1. ~~Spectating~~ done for direct hosts (`docs/MATCHES.md`); through the relay it is still to do.
2. Optional polish: sudden death, a list-of-games lobby, results statistics, replay sharing.

## Where things live

* Sim: `sim-core` (state, step, combat, movement, grabs, scripting hosts), `sim-script` (VM), `sim-content` (text format, bundles,
  validation, recipes), `netplay` (wire format, handshake, rollback session, `Peer`), `transport` (UDP and relay: the only sockets),
  `tools` (`pftool`), `godot-bridge` (`SimRunner`, `ContentEditor`).
* Game: `godot/` (`scripts/` the match, `ui/` menus, `editor/` editors, `tests/` headless tests). Content: `content/base.pfc`
  (must equal the built-in roster in code; a test enforces it; re-export with `pftool content-export`).
* Rules for agents and contributors: `CLAUDE.md` (determinism rules, bump `SIM_VERSION` on any behaviour change, the content/script
  conventions, the editors and menus rules).
