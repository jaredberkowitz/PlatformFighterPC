| Ledges | Grab box, trumping, diminishing invincibility (resets on stable ground), hang cap, regrab cooldown, options: get up / roll / attack / jump / drop | `ledge_*`, `Stage::ledges` |
| Walk, dash, run, turn, crouch | Tilt walks (speed scales with tilt). A *fast* flick dashes (a slow roll to full tilt only walks); reversing mid-dash is a dash dance; a slow reverse turns around first; stick down crouches | `walk_speed`, `dash_*`, `turn_frames` |
| Platform drop | Tapping down on a pass-through platform drops through (shield drop is the other route) | `platform_ignore_frames` |
| Walls and ceilings | Solid blocks (`bottom..y`) stop fighters sideways and from below. The ECB is a diamond: widest at `ecb_side_height`, narrowing to the feet and head, so low corners slip past | `ecb_*`, `Platform::bottom` |
| Helpless fall | `Helpless` state (entered via `fighter::enter_helpless`, which up-specials will call): drift and ledge grabs only, extra landing lag | `helpless_landing_lag` |

# Phase 1: movement (implemented in `sim-core`)

All values are `FighterParams` / `Stage` data (`sim-core/src/content.rs`); frame counts are 60 Hz frames.

| System | Behaviour | Key data |
| --- | --- | --- |
| Input history | 8 frames of input per fighter live in `GameState` and the checksum; `pressed_within(button, n)` detects a press edge within the last n frames | `HISTORY_LEN` |
| Jump squat | Ground jump waits `jump_squat_frames`; jump still held at the end = full hop, otherwise short hop | `jump_squat_frames`, `full_hop_velocity`, `short_hop_velocity` |
| Air movement | Drift, gravity, fast fall (stick down), air jumps | `air_*`, `gravity`, `fast_fall_speed`, `air_jumps` |
| Air dodge | Directional, one per airtime, decays; a shield press during jump squat is buffered to the first airborne frame | `air_dodge_*` |
| Wavedash | Downward air dodge that touches (or starts within `ground_assist_dist` of) a surface becomes a ground slide. Slide speed = `dodge_dir.x * waveland_speed`, so distance scales smoothly with how horizontal the stick is. Too horizontal = plain air dodge. A waveland counts as landing | `ground_assist_dist`, `wavedash_min_down` (cone width), `waveland_*` |
| Shield drop | In shield on a pass-through platform, a down press (buffered) drops through; down wins over jump | `shield_drop_*`, `platform_ignore_frames` |
| Platforms | Solid and pass-through top surfaces; land from above, jump up through | `Stage::platforms` |
| Ledges | Grab box, trumping, diminishing invincibility (resets on stable ground), hang cap, regrab cooldown, options: get up / roll / jump / drop | `ledge_*`, `Stage::ledges` |

Contested ledge grabs on the same frame go to the **lowest player index**, which then trumps any occupant
(`step.rs`). Losers keep falling. Covered by `simultaneous_grabs_resolve_by_player_index`.

## Not done yet (deliberately)

- Ledge attack has movement and timing only; its hitboxes arrive in Phase 3.
- Sloped or moving surfaces, and wall jumps or wall cling (not planned for Phase 1).
- Values are first guesses for playtesting (plan section 12).

## Tests

`sim-core/tests/movement.rs` and `ground_and_collision.rs` script inputs for every item above, including wavedash cone/smoothness/failure, shield drop,
platform landing, and ledge trump/simultaneous/invuln/hang-cap/mirroring. `step.rs` has a random-play invariant test, and
the netplay rollback fuzzer re-verifies determinism of all of it.

## Reference numbers and world scale

Movement now follows the reference game's published attributes. **One world unit = 8 reference units**, so a ~17 unit
tall fighter is 2.2 tall (`FighterParams::su(thousandths)` does the conversion). Jump velocities are *derived* from the
reference jump heights and gravity (`FighterParams::hop_velocity`), and tests assert the measured apex matches.

| | Duelist (sword archetype) | Brawler (blaster archetype) |
| --- | --- | --- |
| Walk / run / dash | 1.575 / 1.964 / 2.255 | 1.208 / 1.54 / 2.09 |
| Air speed | 1.071 | 1.281 |
| Gravity | 0.075 | 0.13 |
| Fall / fast fall | 1.58 / 2.528 | 1.8 / 2.88 |
| Full hop / short hop / double jump | 33.66 / 16.26 / 33.66 | 32.02 / 15.38 / 30.71 |
| Jump squat | 3 frames | 3 frames |

Source: the public attribute tables for the two reference characters (ssbwiki.com). Air friction values were not
available for the duelist and are estimates. Air dodge, wavedash, shield drop and ledge numbers are unchanged.

The placeholder stage is now Final Destination-sized (main block 22 wide, side platforms 3.6 up) so run speeds feel right.

## Momentum rules

- Jump squat applies no friction, so a run or dash jump keeps its speed off the ground.
- In the air, speed above `air_speed` is never cut by the stick; only light `air_friction` drags it down. Air acceleration
  is `air_accel + air_accel_stick * stick_tilt`.
- Walking off a ledge keeps momentum the same way.
- Landing while holding the stick the way you are moving goes straight into a **run** (no walk phase), and speed above
  the target eases off at `run_decel` instead of snapping.

## Ledge grab rule (bug fix)

A fighter can grab a ledge only when it is on the **outside** of the edge and at least `ledge_min_drop` **below** it.
Previously a fighter level with the ledge, or slightly over the stage, could snap onto it (for example walking off the
edge). Walking off now means falling for a moment first.

## Dash cancel and ground speed tuning

- **Releasing the stick cancels a dash** the same frame. The speed carries into a slide on ground friction, so a quick tap
  is a short dash and a held stick is a long one. Jumping out of a dash (or a cancelled dash) keeps its speed as air
  momentum. A flick the other way during a dash is still a pivot (dash dance).
- **`GROUND_SPEED_PERCENT`** (in `sim-core/src/content.rs`, currently 90) scales every ground speed, acceleration and
  friction at once, so starts and stops keep their timing and only the distances shrink. Air speed and jump heights use
  the unscaled reference values. Lower the number to slow the ground game further, or set it to 100 for the raw reference.

## Dash dance and jump-ins

- A cancelled dash brakes on `dash_brake` (stronger than plain ground friction) so stops and spacing are crisp.
- Reversing with a flick starts a new dash in that direction immediately (velocity flips on the same frame). This works
  both flick-to-flick and with a brief neutral between dashes.
- Jumping out of a dash keeps the dash speed in the air; holding back brakes the drift so the landing spot is steerable.
- Keyboard: the most recently pressed direction wins when both are held (`input_reader.gd`), so dash dancing does not
  stall at neutral.
- Demo: `Godot --path godot -- --demo=dashdance --shots=<folder>`.

## Less slippery, and fast fall

- Ground friction and `dash_brake` are doubled. A full run now stops in about 9 frames over roughly half a body width, and a
  cancelled full-speed dash in about 4 frames (tests assert under 10 frames, and under 0.6 of a body width).
- **Fast fall is a hard down press, not holding down.** `Fighter::hard_down`: the stick reaches the full threshold coming
  from near neutral, while falling. Holding down, or rolling slowly down, does not fast fall. On a controller this is a
  hard flick down.
- **Keyboard:** the first down tap is a *soft* press (about 0.55): enough to crouch, drop through a platform or shield drop
  (`STICK_DOWN`), but not enough to fast fall. A **double tap** (second press within 14 frames) is a full-strength press,
  which fast falls. Holding down ramps to full after 4 frames, which is a slow roll and so does not fast fall either.
- Reference clip analysis (the `/watch` skill, 0:30 to 0:50): the Wolf player dash-dances in small bursts near one spot with
  a puff of dust at each stop, firing the blaster between dashes, and covers only a few body widths over about two seconds.

## Air drift model (reference: ssbwiki Air acceleration and Air friction)

Per frame while airborne with horizontal stick tilt `t` (signed, -1 to 1):

- **Acceleration** = `air_accel + air_accel_stick * |t|`, applied toward a target speed of `t * air_speed`. It is the same
  whether you speed up, slow down or reverse. Marth-style: 0.01 + 0.07; Wolf-style: 0.01 + 0.08 (reference units per frame squared).
- **Air friction** applies only with no horizontal input (it decelerates toward zero), and to momentum above the maximum air
  speed when the stick is held the same way (a run or dash jump). Holding the other way brakes that momentum with full
  air acceleration. Values: duelist 0.00375, brawler 0.01. (The brawler's number comes from the dedicated air-friction table;
  its character page lists 0.004, so treat it as less certain.)
- Easing the stick back from full tilt slows you with acceleration, not friction.

Tests pin each of these to exact per-frame values.

## Fast fall fix

A double tap was ignored if the two presses were more than 14 frames apart, which is faster than many people double-tap.
The keyboard window is now 26 frames (about 0.43 s), and the sim remembers a hard down press for 10 frames, so a press just
before the apex still fast falls once you start falling. The input history is 12 frames.

`godot/tests/input_e2e.gd` sends real key events through Godot's Input, the Rust sim and back, and asserts which taps fast fall:
`Godot --headless --path godot --script res://tests/input_e2e.gd`
