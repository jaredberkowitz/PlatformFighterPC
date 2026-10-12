# Project audit for the third moveset (reference: the reference game's Sheik)

Audit of the existing game before adding a third moveset modelled mechanically on the reference game's Sheik. In the game the moveset
is called **Needles** (its weapon) on the **rogue** body: the reference character's name is never used in-game (`CLAUDE.md`,
`docs/MOVESETS.md`). Data and plan: `REFERENCE_DATA.md` (next to this file).

## 1. Engine and project layout

- **Godot 4.7.1** (Forward+), view only. The game logic is Rust: `sim-core` (deterministic fixed-point simulation: state, step,
  movement, combat, grabs, scripting hosts), `sim-content` (text content format, validation, recipes), `sim-script` (bounded integer
  VM for move scripts), `netplay` (rollback), `transport` (sockets), `godot-bridge` (GDExtension `SimRunner`), `tools` (`pftool`).
- **Fixed step**: 60 simulation frames a second (`sim-core/src/step.rs`); everything is integer or fixed point (`Fx`, 16.16); no
  floats, clocks or `HashMap` in the sim (lints enforce it). All mutable state is in `GameState`, hashed every frame (`checksum`).
- **Main scene**: `godot/main.tscn` + `godot/scripts/main.gd` builds the `SimRunner`, the stage view and one `FighterView` per player,
  reads input (`scripts/input_reader.gd`), steps the sim, and draws snapshots. Start screen `godot/menu.tscn`; character select
  `godot/ui/select.gd`; creator `godot/ui/creator.gd`.
- **Fighter definition**: there are no per-character scripts. A fighter is data: `FighterParams` (body: movement, physics, ledge
  and dodge timings; `sim-core/src/content.rs`) + a `Weapon` (the moveset: one `Move` per `MoveId` slot; `sim-core/src/moves.rs`),
  shipped as `content/base.pfc` (must equal the Rust builders; a test enforces it). A **class** is a body + weapon pair; recipes
  (`sim-content/src/recipe.rs`, `CLASSES`) scale a class's body by four stats. `godot/scripts/roster.gd` lists the class names.

## 2. Input and movement

- **Input**: 4 bytes a frame (stick x/y in -127..127, 16 button bits including the attack-stick directions `C_*`); a 12-frame history
  per fighter (`Fighter::history`). Buffering: a 9-frame press buffer (`input_buffer`) and a hold buffer (`Fighter::buffered`,
  `take`); flicks (`flick_x`, `hard_down`, 3-frame travel window, `STICK_FLICK_FRAMES`).
- **Command interpretation** (`sim-core/src/fighter.rs`): `ground` (jump, shield, platform drop, grab, special, attack, dash, walk),
  `start_ground_attack` / `start_air_attack` choose the move from the aim (stick or attack stick, `attack_aim`), `start_special`.
- **Movement**: walk, dash (initial dash, dash dance, pivot), run, run turnaround (`run_turn_frames`), jump squat (3), full / short
  hop (derived from reference jump heights, `hop_velocity`), short-hop aerials, air drift (`air_accel` + `air_accel_stick`, `air_speed`),
  gravity, fall and fast fall (`hard_down`, cancelled by starting an aerial as in the reference), air jumps, air dodges (directional),
  wavedash, ledges, techs, rolls, spot dodge. Parameters per body in `FighterParams`. No wall jump.

## 3. Attacks and timing (traced)

- **Grounded attack** (forward tilt): `ground` → `take(ATTACK)` → `start_ground_attack` (aim → `MoveId::FTilt`) → `begin_attack`
  (state `Attack`, `state_frame = 0`, `hit_mask = 0`) → each frame `attack()` advances `state_frame`, applies motion / friction,
  charge, scripts, rehits; `combat::resolve_hits` (after every fighter has stepped) finds the active hitboxes (`active_hitboxes`:
  `start <= state_frame <= end`), checks hurtboxes, clanks, picks the best hitbox per target by priority, applies `apply_hit`; the move
  ends at `total_frames` (back to idle: the first actionable frame).
- **Aerial** (forward air): `airborne` → `start_air_attack` → `begin_attack`; `attack()` runs `air_move` (drift, gravity, fast fall);
  landing mid-move enters `Landing` with the move's `landing_lag`, or the fighter's normal landing lag inside the autocancel windows
  (`autocancel_before` / `autocancel_after`).
- **Frame counting**: the reference numbers a move's first frame 1; the sim counts from 0. `ref_move` / `row` (moves.rs) take reference
  frames and subtract one; `total_frames = FAF - 2` (the frame after the last is the first actionable). So a reference "hitbox frames
  5-8, FAF 25" is `start 4, end 7, total_frames 23`, and the first frame the hitbox can touch is the reference's frame 5 (tests in
  `sim-core/tests/kits.rs` check this against the reference for the existing kits).
- **Hitboxes**: circles (`Hitbox`: frames, position relative to the feet and facing, radius, damage, angle, base knockback, growth,
  priority, `group`, kind (normal / grab / throw / pummel), shield damage, hitlag multiplier, electric effect). Moving hitboxes are
  several rows over consecutive frames. Hurtboxes: circles per body part (`combat::hurtboxes`), including limbs for the claws class.
- **Hit resolution**: deterministic, all collected before any are applied; ties to the lower player index; one hit per target per
  frame (the best by priority); clanks between grounded attacks; projectiles; counters; reflectors.
- **Hit effects**: knockback (reference formula, `combat::knockback`), the 361 angle, hitlag (`hitlag_frames`, per-hitbox multiplier),
  hitstun (`floor(kb * 0.4) - 1`), DI (`di_degrees`), LSI, SDI (`sdi`, interval and combo growth), ASDI for electric hits, tumble,
  launch speed-up (balloon), stale-move negation, short-hop aerial damage (0.85), rage, crouch cancel, shield stun / pushback.
- **Animation** is presentation only (`docs/ANIMATION.md`): attack clips are posed from the move's own frames; the sim never reads them.
- **Multiplayer**: rollback (`netplay`), lockstep-deterministic; `pftool net-fuzz` must stay at zero mismatches. Anything a moveset does
  is content + generic sim code, so it is rolled back like everything else.
- **Tests**: `cargo test --workspace` (sim-core has per-topic integration suites: `kits.rs` (frame data against the reference),
  `reference_feel.rs`, `reference_movement.rs`, `combat.rs`, ...); Godot headless tests in `godot/tests/`; `pftool net-fuzz`.
- **Debug**: the training overlay (F1/F2 in a match: state, frame, hitboxes, hurtboxes, percent, hitlag, hitstun).

## 4. What the engine supports, and what the third moveset needs

| Needed | Supported? | How it is addressed |
| --- | --- | --- |
| Several hitboxes in one frame | yes | rows in one move |
| Distinct hitbox ids / priority | yes (priority) | the reference's ids map to priority (lower id wins) |
| Separate hits in one move | **only two** groups (`hit_bit` folds group >= 1 into one bit; `hit_mask: u8`) | extend: `hit_mask` grows to 32 bits, eight groups (generic) |
| Multi-hit with a rehit interval | yes, move-wide (`Move::rehit`: every group cleared) | extend: `Move::rehit_groups`, the groups the rhythm re-arms (a looping phase rehits, its final hit does not) |
| Repeated hits on one target | yes (rehit clears the hit history) | as above |
| Loop and final hit with distinct properties | yes (groups, rows) | as above |
| Exact activation frames | yes | `ref_move` / `row` |
| Per-hitbox hitlag | yes (`hitlag` percent) | the reference's H multiplier |
| Per-hitbox SDI multiplier | **no** | extend: `Hitbox::sdi` (percent), carried to the victim for its hitlag |
| Grounded-only / aerial-only hitboxes | **no** | extend: `Hitbox::targets` (both, grounded only, aerial only) |
| Set (fixed) knockback | **no** | extend: `Hitbox::fixed_kb` (the reference's set knockback formula) |
| Hitstun modifiers | **no** | extend: `Hitbox::hitstun_add` |
| Autolink angle (367) | **no** (only 361) | extend: angle 367 in `apply_hit` / `apply_launch`: launched at the attacker's velocity plus a pull toward the hitbox, computed when hitlag ends (so SDI changes it), capped as in the reference |
| Landing hit (a hitbox on landing) | **no** | extend: `Move::landing_move`, a move (an `ext` slot) started on a non-autocancelled landing: its hitboxes come out on the landing frame, its length is the landing lag |
| Aerial landing lag / autocancel | yes | |
| DI / SDI / stale moves | yes | |
| Buffered follow-ups (jab chains) | yes (`Move::next`) | |
| Movement during a move | yes (`Move::motion`, scripts) | down air's dive |
| Projectiles, charge | yes (one projectile per move, smash charge, script stall) | needles (later phase) |
| Teleport-like specials | partly (scripted motion, intangibility) | vanish (later phase) |
| Wall jump | **no** | not needed for the priority moves; noted for later |

Every extension is generic (any moveset can use it), additive in the content format (old content reads the same: new fields default
to the old behaviour), hashed, tested, and a `SIM_VERSION` bump. No second combat system: the new moveset is content on the existing one.

## 5. Plan

1. Engine extensions above, each with a test (sim-core), the content format fields and docs (`docs/CONTENT.md`).
2. The rogue body (`FighterParams::rogue`, the reference's attributes) and the Needles weapon, registered as class 2 (recipe
   `CLASSES = 3`, `Roster.CLASS_NAMES`, built-in fighter "Rogue"), with the presentation mapping (`anim_library.gd`: kicks; no blade).
3. The priority moves from reference data: forward / up / down tilt, neutral / forward / back / up / down air (with the landing hit).
   Tests: frame data against the reference, the up air's loop and final hit, autolink pulling a target along (including downward),
   escape by SDI, no phantom final hit, the down air's landing hit only on a real landing.
4. The rest of the kit from reference data (jab, dash attack, smashes, grabs and throws); specials (needles, grenade, vanish, bouncing
   fish) in a later phase, each documented as reference or estimate.
