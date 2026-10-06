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

- Walls, ceilings and a real diamond ECB. Collision is feet-point vs top surfaces. Needed before Phase 3 combat.
- Ledge attack, walk/dash/turn states, crouch, tap-down platform drop, helpless fall after up-special.
- Values are first guesses for playtesting (plan section 12).

## Tests

`sim-core/tests/movement.rs` scripts inputs for every item above, including wavedash cone/smoothness/failure, shield drop,
platform landing, and ledge trump/simultaneous/invuln/hang-cap/mirroring. `step.rs` has a random-play invariant test, and
the netplay rollback fuzzer re-verifies determinism of all of it.
