# Balance tooling

`pftool balance` plays many matches between scripted bots and reports how every fighter build does. It is a **smoke detector, not a verdict**:
the bot is simple (run at the other fighter, swing when it is in range, shield and hop now and then, recover when it falls off the stage), so
a result means something *between builds of the same kind*, and a number for a whole class mostly measures how well the bot plays that class.

```
cargo run --release -p tools -- balance [--seeds N] [--frames N] [--legal] [--stats 2,5,8] [--quick]
```

* Builds: every combination of the stat levels (default 2, 5, 8; `--quick` uses 3 and 7) for both classes; `--legal` keeps only builds within the
  point budget of 20. Each build plays a panel (the two built-in fighters and six of the builds themselves), on both sides, for `--seeds` seeds.
* Output: the strongest and weakest builds, **what each stat is worth** (average win rate by level), the class averages, and the legal builds
  the bot wins more than 70% or less than 30% of the time.
* Deterministic: the same command prints the same table (the bot uses the simulation's own seeded generator).
* Speed: the quick run (32 builds, one seed) takes about two seconds in release mode; the default run a few minutes.

## What the first run says (read with the bot's limits in mind)

* **Smaller builds win more** (size 3 averages about 50%, size 7 about 38%): a big body is a big target. That is expected and is the intended
  cost of size; whether it is too steep is a question for human play, since the point budget treats the four stats as equal.
* Speed, jump and weight are within a few points of each other, so none is obviously a free lunch for the bot.
* **The brawler (class 1) scores far below the sword (class 0)** against this bot. Most of that is the bot: it approaches and swings with the
  same pattern for both, which suits the sword's reach. Treat it as "the bot cannot use the brawler yet", not as a balance finding; a better
  bot (or a human) is needed to judge the classes against each other.

## Next steps for balance

* A stronger bot (spacing, combos, edge-guarding, per-class move choices) so class comparisons mean something.
* Feed human playtest results in: the replay files (`docs/MATCHES.md`) are the data, and `pftool replay-verify` already plays them.
* Make the point budget weigh the stats differently once the numbers say which are worth more.
* Tests: `tools/src/balance.rs` (deterministic, finishes, even against itself, the bots land hits).
