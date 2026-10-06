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
