# Momentum into jumps (19 October 2026)

A tester feels many of our differences from the reference "come down to subtle ways momentum is carried during movement: from a run
into a jump, a walk into a jump, jumping backwards, forwards". This page holds how our jumps carry momentum today, measured, and what
it can be compared against. Measured by `sim-core/tests/momentum_table.rs` (an ignored measuring test: `cargo test --release -p
sim-core --test momentum_table -- --ignored --nocapture`), full hops on a long flat floor, in the reference's units (1 world unit = 8).

## The rule today (sim 144)

Each body has the reference's three published numbers: `jump_speed_x` (a standing jump's sideways speed at full stick),
`jump_speed_x_mul` (the share of ground speed kept, 70-90%) and `jump_speed_x_max` (the most a ground jump leaves with). How the
reference combines them is **not published** (searched again 19 October). Since sim 144 (`jump_stick_share` 2) they add: kept ground
speed plus the stick's share, within the maximum. The 3-frame jump squat keeps the ground speed whole (no braking): also unconfirmed.
In the air, speed above the air speed bleeds off at the air friction while the stick points the same way, and the stick against it
turns it at the air acceleration (both published).

## Measured (take-off speed, distance from take-off to landing)

Stick held the same way through the jump and the flight (forward unless said); the "walk" holds the stick at about half; "run back"
flicks the stick back with the jump out of a run (in our game that turns the fighter round: a reverse aerial rush) and holds it;
"RAR" is the same jump with the stick let go in the air.

| Fighter (reference) | Standing | Walk | Dash | Run | Run, stick neutral | Run back | RAR | Max |
|---|---|---|---|---|---|---|---|---|
| Longsword (Marth) | 0.94 / 61 | 1.19 / 56 | 1.19 / 63 | 1.19 / 63 | 1.19 / 63 | 0.62 / -42 | 0.62 / 30 | 1.2 |
| Claws (Wolf) | 1.00 / 57 | 1.23 / 32 | 1.29 / 58 | 1.29 / 58 | 1.23 / 52 | 0.23 / -44 | 0.23 / 7 | 1.3 |
| Needles (Sheik) | 0.75 / 56 | 1.28 / 37 | 1.39 / 59 | 1.39 / 59 | 1.39 / 51 | 1.17 / -25 | 1.17 / 40 | 1.4 |
| Embers (Mario) | 0.94 / 70 | 1.17 / 38 | 1.29 / 70 | 1.29 / 70 | 1.29 / 51 | 0.45 / -52 | 0.45 / 7 | 1.3 |
| Gauntlets (Falcon) | 0.89 / 60 | 1.00 / 31 | 2.09 / 96 | 2.09 / 96 | 1.91 / 87 | 1.01 / -26 | 1.01 / 42 | 2.1 |
| Staff (Palutena) | 0.71 / 52 | 1.04 / 30 | 1.09 / 53 | 1.09 / 53 | 1.09 / 44 | 0.73 / -38 | 0.73 / 25 | 1.1 |
| Greatsword (Cloud) | 0.94 / 58 | 1.19 / 37 | 1.19 / 59 | 1.19 / 59 | 1.19 / 50 | 1.00 / -25 | 1.00 / 40 | 1.2 |
| Bracers (DK) | 0.94 / 66 | 1.31 / 43 | 1.59 / 72 | 1.59 / 72 | 1.49 / 60 | 0.54 / -40 | 0.54 / 10 | 1.6 |
| Clutch (Yoshi) | 0.90 / 82 | 1.09 / 41 | 1.50 / 86 | 1.50 / 86 | 1.43 / 80 | 0.52 / -65 | 0.52 / 23 | 1.5 |
| Buckler (Link) | 0.94 / 44 | 1.19 / 52 | 1.19 / 52 | 1.19 / 52 | 1.19 / 52 | 0.27 / -28 | 0.27 / 9 | 1.2 |
| Pockets (Villager) | 0.94 / 57 | 1.13 / 41 | 1.29 / 62 | 1.29 / 62 | 1.11 / 48 | 0.16 / -46 | 0.16 / 2 | 1.3 |
| Popgun (Diddy) | 0.94 / 49 | 1.27 / 56 | 1.29 / 58 | 1.29 / 58 | 1.29 / 58 | 0.65 / -23 | 0.65 / 24 | 1.3 |
| Dagger (Joker) | 0.94 / 51 | 1.11 / 30 | 1.19 / 52 | 1.19 / 52 | 1.19 / 46 | 0.59 / -33 | 0.59 / 17 | 1.2 |
| Nodachi (Sephiroth) | 0.89 / 45 | 1.06 / 33 | 1.19 / 49 | 1.19 / 49 | 1.19 / 48 | 0.77 / -25 | 0.77 / 28 | 1.2 |

## What it shows

1. **Moving jumps are capped.** Out of a dash or a run nearly every body leaves at its published maximum (1.2-1.6; the Gauntlets 2.1),
   so a run jump goes only a little further than a standing one (Longsword 63 against 61) and the stick forward or neutral barely
   matters. That is the published maximum at work; if the reference's run jumps go clearly further, the maximum (or how it applies)
   is what to look at.
2. **The reverse aerial rush lost most of its momentum in sim 144.** With the stick and the run's share added, the stick back
   (which turns the fighter round) subtracts a standing jump's speed from the run: the Claws' RAR leaves at 0.23 and lands 7 units
   ahead, the Longsword's at 0.62. Before sim 144 a RAR kept the run's share whole (decision 100: "a reverse aerial rush keeps its
   dash"), and the reference's RAR is known for carrying the run's momentum while facing back. This is very likely wrong now.
3. **A walk jump with the stick at half tilt lands shorter than a standing jump at full tilt** (it leaves faster, but the half-held
   stick then slows the drift to half the air speed): a consequence of the published air control, plausibly the reference's too.
4. **The jump squat keeps the ground speed whole.** If the reference brakes during it (as the earlier game did, with the ground's
   traction), run jumps would leave slower; small, but a "subtle momentum" difference of exactly the kind the tester describes.

## To confirm against the reference

A training-mode clip (one fighter, Wolf or Marth, camera fixed, 60 a second if possible): standing jumps with the stick forward,
neutral and back; a walk then a jump; a dash then a jump; a run then a jump with the stick forward, neutral and back; a reverse aerial
rush; each twice. The tracker from the footage pass (`docs/FOOTAGE_COMPARISON.md`) measures each case's take-off speed and distance,
which go straight against this table.
