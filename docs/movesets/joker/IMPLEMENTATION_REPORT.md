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

### Phase 2 plan

1. **The hook** (up special, the three placeholder slots): the grounded grab box on 20-26 that catches a fighter not in hitstun, is
   stopped by shields and reflectors, pulls it in and throws it (11%, 78 degrees, release 6, FAF 30); the aerial hit (5%, 20-25 along a
   line to 38 up); **the tether**: the hook catching a ledge in reach and reeling the fighter to the hang (a general "tether" extension:
   a ledge-catch box on a move frame window, the reel speed, the hang; three tether grabs before landing), the once-per-air-time boost.
2. The partner's arrival and leaving (if the owner wants them modelled with estimated lengths), its longer rolls (if numbers appear).
3. The gun's dodge staling and buffered delay; angle 365 (general) for the jump's shots.

## Questions for the owner

1. **"Filled by damage dealt"**: every source says the gauge fills from damage taken (and time, the guard and team events), not dealt.
   Built as the sources (nothing for damage dealt). Change it?
2. **The partner's arrival and leaving animations** (an intangible summon when standing; a mostly invincible, uncancellable exit that
   stops the fall): no frame counts are published. Model them with estimated lengths, or leave them out (as now)?
3. **The gun's distances**: no published band distances (here 20 / 44 / 72 reference units). Footage would settle them; keep the
   estimates?
4. **The phase-2 tether**: build a general tether (a ledge-catch box that reels the fighter in), as the hook needs, for the hook first?
