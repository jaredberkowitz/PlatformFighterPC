# Implementation report: the thirteenth moveset

Class 12 **Dagger** on the **phantom** body (built-in fighter **Phantom**), modelled mechanically on a reference character (13.0.1).
Reference data: `REFERENCE_DATA.md`; the engine work and why: `PROJECT_AUDIT.md`.

## Phase 1 (sim 121, content schema 7 unchanged; branch `dagger-phase1` from `movesets` at 10e2df5, sim 119; merged `origin/movesets` at 6a7a3a4, sim 120's run mode)

Sim 120 belongs to another session (the run mode); this phase took 121.

### Built

* **The body** (`FighterParams::phantom`): every 13.0.1 attribute from the parameter file (weight 93, walk 1.12, dash 1.9 for 10 frames,
  run 2.06, traction 0.13, air 1.1 / 0.01 + 0.07 / 0.01, gravity 0.127, fall 1.63 / 3.097, jumps 32.5 / 14.2 / 34 with the air jump's
  x0.9 drift, ledge jump 36.25, heavy landing 3), the dodges, the wall jump (1.3 / 2.7), the common item-throw frames.
* **Every normal** (`sim-core/src/moves_dagger.rs`) with and without the partner: the three-hit jab, the aimed forward tilt, the up
  tilt's toss-and-catch multi-hit, the sliding down tilt, the two-hit dash attack, the three smashes, the five aerials (the down air's
  partner meteor), grabs, pummel, throws, ledge and get-up attacks.
* **The gauge and the partner** (E52, E53): `FighterParams::phantom_gauge` (the rates, the stock and crowd multipliers, 1800 frames, the
  cut by damage, the stun, the KO's units, specials that keep it); every normal's partner boxes (part 1) and awakened boxes (`mode`).
* **The gun** (neutral special, ext3-ext7): the shot and its three bands, presses and holds, the dodges forward and back, the grounded
  jump, the aerial shots below and spiral, the air stall; the partner's three bullets and extra shots (powered boxes in the same moves,
  the plain lengths ended by the scripts).
* **The curse bolt** (side special, ext8 powered): the bolt, its eruption (not on shields), the curse (E54); the partner's faster bolt
  with its multi-hit burst, on fighters and the ground.
* **The guard** (down special, ext9-ext11): super armor 3-19, held to 120, 0.4 of a hit and x1.5 hitlag (E55), the gauge gain, the
  backhand on letting go within 33 frames of a hit; **the partner's stance** (ext12-ext14): the counter x1.6 (12-50%) and the reflector
  x1.6 / x1.9.
* **The partner's wings** (the powered up special, ext15). The plain up special (the hook) is still the claws' placeholder: phase 2.
* **Registration**: `CLASSES` 13, `Content::placeholder` (`phantom`, `dagger`), `godot/scripts/roster.gd` (name, blurb, built-in entry with
  the default look), the Godot test counts (`creator_flow_test.gd`, `online_fighters_test.gd`, and the name lists in `content_e2e.gd` /
  `editor_api_test.gd` as the last port did), the fuzzers (the phantom with its partner out every fourth rollback run, every eighth
  network run).

### Verified against [R] / [D] (by tests that ran)

* `sim-core/tests/dagger.rs` (11): every normal's windows (both modes), FAF, landing lag and autocancels; every attack's first hit on its
  reference frame **with and without the partner**; every attack's FAF in both modes; damages, angles, BK and KS of 34 boxes (the
  partner's among them), angle types, the up-aimed forward tilt, the powered forward tilt's shieldstun and hitlag; the partner's boxes
  are part 1, powered and never clank, on every attack but the pummel, throws, ledge and get-up attacks; **the jab's 2% + 0.7% and the
  down tilt's 6% + 7% landing on one frame**; an attack keeping its boxes when the partner comes or goes; the partner's down-air meteor
  (270 degrees); the jump air times (30 / 45 / 20 / 30, within one frame); the body; a resimulation with two phantoms, one powered.
* `sim-core/tests/dagger_gauge.rs` (9): the fill by itself (and not on the revival platform); 3% taken gives 3.9 and a real hit 1.3 a
  percent added; nothing for damage dealt; the stock-gap and crowd multipliers (1.625, 1.95, x1.1, x1.333); 76.93% summons it for 1800
  frames and nothing fills meanwhile; 15% cuts 240 frames, 112.5% ends it, 10.2 with four fighters; a broken shield stops the clock (the
  champion's keeps counting); every special changes and none spends it; a KO leaves 20 / 25 / 30 (the champion's empties); checksum and
  resimulation.
* `sim-core/tests/dagger_specials.rs` (12): the gun on 12, its three bands by distance, FAF 37; a press firing again on 37 and a hold
  every 36 [E]; the partner's three bullets on 12 / 18 / 24 and FAF 42; the dodges (intangible from 4, moving), the jump, the shots below
  and the spiral; the air stall; the bolt's 1% then 2% a frame later, then 8 ticks 45 frames apart (11% in all); no eruption and no curse
  on a shield; the partner's bolt (1.5% every 40) and its burst on the ground; the guard taking 0.4 without flinching and filling the
  gauge 1.3 x 6.7368 a percent taken (28.6% fills it); the held guard lasting to 120 then 33 frames of lag, FAF 53 unheld, the backhand;
  the stance countering at x1.6 (at least 12%); the wings intangible from the start, a long rise, the special fall.
* General: `move_rules.rs` (the new content fields in the hash; a powered / plain box out only in its mode; a curse's ticks),
  `scripted_moves.rs` (`powered`, `withstood`), the checksum cases (`move_powered`, `withstood`, the five `curse_*`), sim-content's
  `the_dagger_fields_round_trip_and_are_checked`.

### Estimated or limited [E]

From `REFERENCE_DATA.md`'s list: the gun's band distances (gap 1) and held rhythm; no arrival or leaving animation for the partner (gaps
2-3); no longer rolls with it (gap 4); the bolt's path and the stall (gap 5); the wings' flight (gap 6); every box on a bone (gap 7); the
two-hits-on-one-frame launch rule (gap 8); the guard's 120 counted from its start (gap 9); the stance's bubbles (gap 10); the gun's air
stall (gap 11); angle 365 (gap 12). The gauge's fill by itself is 10,815 frames to full, not 10,810 (the fixed point's 0.00925).
Limits: the hook (phase 2), the gun's dodge staling and buffered delay, the up smash's arm intangibility with the partner, a powered
special cannot be B-reversed (an `as_special` rule the champion shares), the house limits (one get-up attack, the grabs' aerial boxes, the
throws' bystander hit and invincibility).

### Tests and checks run

`cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings` (clean); `cargo test --workspace` (1499 passed, 0 failed, after
merging `origin/movesets` 6a7a3a4); `pftool fuzz-rollback 300` (0 mismatches; the phantom in 144 of 1200 fighter slots); `pftool
net-fuzz 100` (0 mismatches). The Godot tests were updated but not run in this session.

### Training-mode checks (hitbox view)

1. Fill the gauge (take 77%, or hold the guard against a strong hit): the partner's boxes appear on the next attack begun; jab a dummy
   at 0% and read 2.7% (3.24% with the one-on-one 1.2).
2. With the partner out, start a forward tilt and let a dummy hit the phantom for 113% before frame 19: the partner leaves but the tilt's
   second hit stays out to 19.
3. Gun a dummy at three distances: 6% / 3.6% / 1.2% shown, the last with no flinch.
4. Bolt a dummy: 1.2%, 2.4% next frame, then 1.2% every 45 frames, 8 times.
5. Guard a dummy's forward smash: 0.4 of its damage, no flinch, the gauge jumps; let go at once: the backhand.

### For the art agent

New moves fall back to the shared archetypes. Clips wanted (first active frame / active span; pose):

* Jab 1 / 2 / 3 (4-5, 3-4, 3-4): an inward knife slash, a backhanded stab, an outward slash; the partner's swipes beside.
* Forward tilt (8-9, 13-15 / 19): a double knife thrust, aimed up / straight / down; the partner's hand thrust and beckon.
* Up tilt (8-23): the knife tossed spinning overhead and caught; the partner's electric palm (11-19).
* Down tilt (8-14): a baseball slide; the partner sliding alongside.
* Dash attack (6-7, 15-21): a roundhouse then a spinning side kick; the partner's delayed kick.
* Smashes: forward (charge 5, 16-19) a two-handed downward slash with the partner's X slash; up (charge 4, 10-14) a spinning knife
  uppercut, the partner's overhead swing; down (charge 5, 12-13 / 16-17) a sweep in front then behind, the partner's kicks.
* Aerials: neutral (12-27) two corkscrews with the knife circling (the partner back to back); forward (7-8, 12-14) a hook kick then a
  roundhouse; back (7-8) an upward swipe behind (the partner's chop); up (5-21) a corkscrew kick with a finishing kick (the partner's
  hand overhead); down (13-16) an upside-down knife swing (the partner's stomp 15-16).
* Gun: the shot (12), the dodge forward / back (shot 20 / 22), the jump while firing below, the corkscrew below, the spin; the
  partner's blue muzzle flash for three bullets.
* Curse bolt: a finger snap (16), the bolt, the eruption; the partner's bigger bolt and pillar.
* Guard: the defensive stance (3-19, held), the backhand (8-9); the partner's stance, counterattack (5-7) and reflection.
* Wings: a flight up on the partner's wings (98 frames).
* The partner itself: a figure behind the phantom while the mode runs (the view can read `gauge_mode`).

### Phase 2 plan (done in sim 123, below)

1. **The hook** (up special, the three placeholder slots): the grounded grab box on 20-26 that catches a fighter not in hitstun, is
   stopped by shields and reflectors, pulls it in and throws it (11%, 78 degrees, release 6, FAF 30); the aerial hit (5%, 20-25 along a
   line to 38 up); **the tether**: the hook catching a ledge in reach and reeling the fighter to the hang (a general "tether" extension:
   a ledge-catch box on a move frame window, the reel speed, the hang; three tether grabs before landing), the once-per-air-time boost.
2. The partner's arrival and leaving (if the owner wants them modelled with estimated lengths), its longer rolls (if numbers appear).
3. The gun's dodge staling and buffered delay; angle 365 (general) for the jump's shots.

### Phase 1 questions (answered 2026-10-14: decisions 80-82 in `../ROSTER_STATUS.md`)

1. "Filled by damage dealt": yes, as a house rule (decision 80): built in phase 2.
2. The partner's arrival and leaving: close to instant, not modelled (decision 81).
3. The gun's distances: kept as estimates (decision 82).
4. The tether: built general (E56) in phase 2.

## Phase 2 (sim 123, content schema 7 unchanged; branch `dagger-phase2` from `movesets` at 0d6cfe0, sim 121; merged `origin/movesets` at 858cbdd, sim 122's run-mode rooms)

Sim 122 belongs to another session (the run mode's rooms); this phase took 123.

### Built

* **The hook** (the plain up special; `sim-core/src/moves_dagger.rs` `hook`): on the ground a command grab on 20 (the rope, r 3.0, along
  the reference's hook line, and the head, r 5.5) and 21-26 (the head), FAF 60, that catches in the air or on the ground, misses fighters
  in hitstun and is stopped by a raised shield or a reflector (`Hitbox::blockable`); a catch goes on as the pull (ext1): the caught fighter
  held in front, thrown on frame 6 for 11% at 78 (BK 75, KS 40), FAF 30. In the air (ext0) a 5% hit at 55 (BK 76, KS 56, sd +1, hitlag
  x1.2, clanks without rebounding) on 20 (head and line) and 21-25 (head), FAF 45, no special fall; the first in an air time rises a
  little and marks the air time (`dagger_hook.script`, mark 0); a later one is ext2, the same hook without the rise (a flag version).
* **The tether** (E56, general): `Move::tether` on both air hooks: from frame 12 to 25, a free ledge up to 40 of the reference's units
  above the feet and 16 in front is caught; the fighter hangs 8 frames (losing 0.24 of its speed a frame), is reeled to the ledge in 12
  and grabs it as any grab; three catches before landing or getting up; the stick down lets go; a ledge taken meanwhile throws it off.
  New state `FighterState::Tether`, hashed `Fighter::tether_ledge` / `tether_count` / `tether_from`; `scripting::World::ledges`.
* **Phase 1 limits closed**: the gun's dodges staled like rolls (`Move::dodge_stale`: counted, intangibility later when stale); a dodge or
  jump buffered before frame 26 of a shot comes 4 frames later (the gun's script, `early`); **angle 365** (`ANGLE_HALF_LINK`: half the
  attacker's momentum) on the gun jump's shots; the up smash's arm intangible 10-14 only with the partner (`PartIntangible::mode`).
* **Owner decisions 80-82**: damage dealt fills the gauge 1.3 a percent (the existing `Gauge::per_dealt`; a house rule [E]; the curse's
  ticks no longer count as dealt, `gauge::on_damage_over_time`); the partner's arrival and leaving stay instant (not modelled, a
  decision); the gun's band distances stay estimates.

### Verified against [R] / [D] (by tests that ran)

* `sim-core/tests/dagger_hook.rs` (11): the grounded hook's boxes (grab, command, blockable; 20 / 20-26), FAF 60, the catch of a
  fighter in the air on frame 20 and the throw's 11% on 26 (the pull's frame 6), into hitstun; a shield on the head stops it for the rest
  of the move (and the same fighter not shielding is caught); a fighter in hitstun is not caught; the air hook's 5% on frame 20, FAF 45,
  no special fall, the rise once per air time (the second is ext2 and falls); the tether's numbers, its catch on frame 12, 8 frames
  hanging and the ledge on the 20th frame after the catch, no catch out of reach (too low, too far out, facing away), the ledge not held
  meanwhile; the stick down lets go, three catches and no fourth, landing resets the count; a held ledge is not caught, one taken during
  the reel throws the phantom off with the hop; a hit ends the tether (the count kept); the tether fields in the checksum and a
  resimulation onto the ledge; the up smash's limb window only with the partner; 365 on the gun jump and a 365 launch at half speed (no
  tumble); the gun's dodges stale (intangible from 3 fresh, 3 + 4 fully stale) and a buffered dodge or jump 4 frames later.
* `dagger_gauge.rs` (12, three new): a jab's damage fills 1.3 a percent dealt (with the fill by itself), the same a stock behind,
  nothing with the partner out; a shielded jab and a curse's tick fill nothing for the phantom; the champion's gauge still 0.667 a
  percent dealt and a body without a gauge nothing.
* `move_rules.rs`: every tether number, `dodge_stale`, `blockable` and a window's `mode` in the content hash; a reflector field stops a
  blockable grab. `state.rs`: checksum cases `tether_ledge`, `tether_count`, `tether_from`. sim-content:
  `the_dagger_phase_2_fields_round_trip_and_are_checked` (round trip, old files, bad values).

### Estimated or limited [E]

From `REFERENCE_DATA.md` estimates 12-17: 365's grounded rule and the partner's follow-ups at 365; the tether's reach, catch window end,
hang and reel lengths, the reading of the hang resistance, the hop off a taken ledge, the wait before another tether; the grounded hook's
head held at the line's end, what a shield stop leaves, where the pull holds its catch; the air hook's rise and landing lag; the gun
dodges' length not stretched, the buffered delay only out of a shot. Limits: no swing on the tether and no reel by a button; the stick's
tilt of the hook's rise; the partner's longer rolls (unpublished); the house limits (one get-up attack, the grabs' aerial boxes, the
throws' bystander hit and invincibility).

### Tests and checks run

`cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings` (clean); `cargo test --workspace` (1524 passed, 0 failed, after
merging `origin/movesets` 858cbdd); `pftool fuzz-rollback 300` (0 mismatches; the phantom in 144 of 1200 fighter slots); `pftool net-fuzz
100` (0 mismatches). The Godot tests were not run (no Godot change; no fighter count change).

### Training-mode checks (hitbox view)

1. Hold a dummy in the air above and in front (the hook line's far end, about 1.3 ahead and 4.75 up): the grounded hook catches it on
   frame 20 and throws it for 13.2% (11 x 1.2).
2. Let the dummy shield on the right platform at the hook's head: the hook passes, nothing caught for the rest of the move.
3. Jump off the right edge, fall to about 3 below and 1 outside it, face the stage and use the up special: the tether catches on frame 12,
   hangs 8 frames, reels in 12 and the phantom hangs from the ledge. Hold down while hanging: it lets go.
4. Tether three times (letting go each time): the fourth hook catches nothing until landing.
5. Use the up special twice in one air time: only the first rises.

### For the art agent

* The grounded hook (first active 20, out 20-26): a hook shot up and forward on a line from 3.2 ahead / 22 up to 10.6 ahead / 38 up of the
  reference's units (0.4 / 2.75 to 1.3 / 4.75 world units), the head at its far end; the pull (6 frames to the throw) dragging the caught
  fighter down in front, the throw up at 78 degrees.
* The air hook (20-25): the same line from the air, with a small rise on frame 5 the first time.
* **The tether** (`FighterState::Tether`, a new state the view receives as "Tether"): hanging below and outside the ledge on the hook
  line for 8 frames, then reeled straight to the ledge in 12. The ledge index is the sim's `Fighter::tether_ledge` (not yet exposed by the
  bridge: a read-only accessor would let the view draw the rope to the ledge).

## Phase 3 (sim 126, content schema 8 unchanged; branch `dagger-phase3` from `origin/movesets` at a6b8314, sim 125)

The owner's answers to phase 2's questions (decisions 83-86 in `../ROSTER_STATUS.md`): keep the tether's estimates (83: nothing
changed); build the other three, each a general content field off by default.

### Built

* **A shield ends the grounded hook sooner** (decision 84): `Move::shield_end` (content `shield_end`): when a shield or a reflector stops
  one of the move's blockable grab boxes, the move ends that many frames later, if sooner than its own end (hashed `Fighter::move_end`,
  set in `combat::resolve_hits` where the stop is found, read at `fighter::attack`'s end check, cleared by `begin_attack`). The grounded
  hook: 20, so a stop on frame 20 leaves the phantom standing on 40 (FAF 41) instead of 59 (FAF 60) [E: the reference says only that
  contact "cancels" the move; no frame is published].
* **Stale gun dodges last longer, as rolls do** (decision 85): `Move::stale_stretch` (content `stale_stretch` none / roll / back_roll /
  spot; needs `dodge_stale`): the move's whole timeline plays slower by the ruleset's roll, back roll or spot dodge rate per level of the
  staleness it was made with, exactly as `fighter::roll` plays a stale roll. Each frame, `fighter::attack` holds the move's frame back
  while the fresh timeline (`stale_frame` of the frames played) has not reached it (hashed `Fighter::move_held`), so the boxes, the step
  (further), the script's frames and the end all stretch (the end is exactly the roll's `stale_length`); what happens once on a frame
  does not repeat on a held one; the intangibility keeps to the frames played (start shifted, end unmoved, as a roll's). The forward dodge
  at the forward roll's rate (6% a level: fully stale, 38 frames become 50), the back dodge at the back roll's (10%: 44 become 66) [E:
  which rate].
* **Reel in by a press** (decision 86, a house addition): `Tether::reel_press` (content `reel_press` in the tether block): a press of
  special while hanging starts the reel at once. On for both air hooks: pressed on the second hang frame, the phantom holds the ledge 13
  frames after the catch instead of 20.

### Verified (by tests that ran)

* `sim-core/tests/dagger_phase3.rs` (5): the grounded hook stopped by a shield on 20 is out on 40 (whiffed 59), the shielding fighter
  still shielding, a fighter not shielding still caught; with `shield_end` 0 the stopped hook runs to 59 (the old behaviour); `move_end`
  in the checksum, a resimulation from the stop, a new move clearing it. Fresh gun dodges out on 38 / 44; fully stale out on the roll's
  `stale_length` (50 / 66), one level stale on its own, the move frame shown on 26 the fresh timeline's (20), the step longer, the
  intangibility's last frame unmoved; with the stretch off the fresh length (sim 123's); `move_held` in the checksum, a resimulation, one
  level added per dodge however its first frame is held. The tether's reel pressed on hang frame 2 / 8 / during the reel: on the ledge
  13 / 19 / 20 frames after the catch, and 20 with `reel_press` off.
* `move_rules.rs`: the three fields in the content hash; `state.rs`: checksum cases `move_end`, `move_held`; sim-content
  `the_dagger_phase_3_fields_round_trip_and_are_checked` (round trip, a file without them, bad values refused).
* Unchanged: `dagger_hook.rs` (11) passes as before (the stale intangibility still 3 + 4 frames late); no other content uses the fields
  (`base.pfc` changes only in the dagger weapon and the version line).

### Estimated [E]

The 20 frames after a stopped hook; which roll rate stretches each gun dodge. Unchanged: the tether's reach, hang and reel (decision 83).

### Tests and checks run

`cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings` (clean); `cargo test --workspace` (1588 passed, 0 failed);
`pftool fuzz-rollback 300` (0 mismatches); `pftool net-fuzz 100` (0 mismatches). The Godot tests were not run (no Godot change).

### Training-mode checks (hitbox view)

1. Let a dummy shield on the right platform at the hook's head and use the grounded up special: the hook stops on the shield and the
   phantom can act on frame 41 (counting the press as 1).
2. Roll five times quickly, then dodge with the gun (sideways with special during a shot): the dodge plays slower and travels further;
   wait a few seconds and it is quick again.
3. Tether the right ledge from below and press special right away: the phantom is reeled in at once.

### For the art agent

* A stale gun dodge's `state_frame` repeats frames (held back), so its clip plays slower with no change needed; the grounded hook may now
  end on frame 40 after a shield stop (the clip is cut short there: a quick retract would read better).

## Questions for the owner (phase 2)

Answered 2026-10-14 (decisions 83-86): 1 keep; 2, 3 and 4 built in phase 3 (above). The original questions:


1. **The tether's estimates**: the reach (40 up, 16 in front, in the reference's units), 8 frames hanging and 12 reeling in are not
   published. Footage of a few frame-counted tethers would settle them; keep them meanwhile?
2. **A shield stopping the grounded hook**: the sources say contact with a shield or reflector "cancels" it; here the hook simply catches
   no one and the move runs to its FAF 60. If footage shows it ending sooner, that is a small change.
3. **The gun's dodge staling in length**: staleness delays the dodge's intangibility and counts with rolls, but does not lengthen the dodge
   (a scripted move's timeline cannot be stretched yet). Build that stretch, or leave it?
4. **Reel in by a button**: older games of the series let a button reel in early; here the reel is automatic only. Wanted?
