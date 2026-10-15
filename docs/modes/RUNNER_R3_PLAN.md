# Run mode R3: shops and items (plan, for Saturday 17 October 2026)

Status: **planned, not started.** Written by the executive producer on 15 October 2026, when the owner asked to commit R3 to a plan for
when usage resets. It builds on `RUNNER_PLAN.md` section 3.5 (shops and items) and section 11 (R2c, sim 131). The **modes-online
agent** builds it, and the executive producer reviews and lands it.

Goal (the phase table): **a run's build changes how it plays.** By the second or third shop, two runs with the same fighter should feel
different.

## 1. Rules from the research (RUNNER_WORLD.md section 1)
- **Upgrades come slowly.** Punch Quest's developer regretted giving out "an upgrade or new ability or power every single match". The
  time between upgrades matters more than the total. We keep shops every 5 to 7 rooms and sell 2 to 4 things a visit, priced so a
  normal run affords about one good item per shop.
- **Risk pays.** The treasure alley's chest, arena bonuses and the high road's coins are how you afford more. No item should be bought
  with anything but coins.
- **Everything resets each run** (Decided, RUNNER_PLAN 3.5). The meta layer comes later (R7).

## 2. The shop
The walk-in shop and the shopkeeper already exist (R2a), along with the rule that a bonked shopkeeper stays closed until a new room
(R2c). R3 adds:
- **Four pedestals, filled.** Draws come from weighted pools by rarity:
  - common (white);
  - rare (blue);
  - prized (gold, from tier 3).
  The draw is seeded, so a replay gets the same shop.
- **Standing by a pedestal** shows a speech-bubble card: the item's name, one line saying what it does, the price, and how many you
  already own.
- **Reroll:** a bell on the counter. Ringing it re-draws the unsold pedestals: once for 10 coins, then the price doubles each ring,
  in that shop. The shopkeeper reacts.
- **Prices** come from a table by rarity, scaled by tier (about +15% a tier). The current heal (30 coins) and extra life (150 coins,
  one per shop) stay, as the owner approved.

## 3. The items (first list: 18, all data in `RunContent`)

### Stat boosts (common; they stack, capped at 3 of each)
These are the four character-creator stats, applied in **Rust only** (CLAUDE.md: never compute physics from stats in GDScript). The
run keeps its own copy of the fighter's parameters and rebuilds them through the recipe code when a stat changes.

| Item | Effect per stack |
| --- | --- |
| Power | damage dealt +8% |
| Speed | walk, dash and run speed +6% |
| Weight | weight +8% (harder to launch) |
| Spring | jump height and air jump +6% |

### Properties (rare unless noted; each one is a sim rule that is active only in a run)

| Item | Effect |
| --- | --- |
| Lifesteal | each grunt KO heals 3% |
| Coin magnet | coins within 4 units fly to you |
| Hot hands | your hits leave a short burn, +1% per second for 3 seconds, on grunts only |
| Iron startup | super armor through the startup of your strong attacks (prized) |
| Extra wing | one more midair jump |
| Sturdy shield | shield +30% health, shrinks 25% slower |
| Second wind | after losing a life, 3 more seconds of invincibility |
| Steady combo | the first hit you take doesn't break the combo meter |
| Long reach | your hitboxes are 10% bigger (prized) |
| Shadow orbs | the Exile's orbs (E59, `Move::orbs`) circle you and strike nearby grunts (prized; the owner's idea) |

### Consumables (used at once, or held in one slot and used with a button; the agent picks whichever fits the controls better)

| Item | Effect |
| --- | --- |
| Heal 30% | exists already |
| Extra life | exists already, one per shop |
| Bubble | absorbs the next hit |
| Bomb | a throwable item, reusing the items code |

Move swaps within an archetype stay in R5 and are not part of R3.

## 4. How it is built
- **Run loadout state** in `GameState::run`, hashed: stat stacks, property levels, the held consumable, and the shop's pedestals,
  rerolls and sold flags. Every new field gets a checksum test entry.
- **Stats:** when a stat changes, the run rebuilds the player's `FighterParams` through `sim-content`'s recipe path, into the run's own
  content copy. Versus content is never touched. A test checks that the rebuilt parameters equal the recipe code's output.
- **Properties:** hooks in the run's step and in the run-only branches of the combat and items code. Versus must not pay for them:
  `pftool fuzz-rollback 300` timing stays within 2% of main, and its output line is identical (the R2c lesson).
- **HUD:** a strip of owned item icons under the coins; the results screen lists the build.
- **Godot:** pedestal cards, rarity glows, the bell, item icons. Placeholders use simple shapes; the art list goes to the art director,
  each item marked reused or new.

## 5. Small tuning from the owner's R2c answers (fold into R3)
- Grunts tech, DI and mash from **tier 3** instead of tier 2. The owner called tier 2 "a bit too early". (**Done early, sim 136**, 15 October.)
- Everything else from R2c is kept as is. The gate pass-through, the shieldbearer's bubble, the KO effects, 12 grunts and the tier
  weights are all "fine for now, refine later".

## 6. Tests and acceptance
- One test per item's effect, including stacking and caps.
- Seeded shop draws: the same seed gives the same shop.
- Reroll prices.
- Purchase refusals: not enough coins, a closed shopkeeper, the life limit.
- Rebuilt stats equal the recipe output.
- Properties have no effect outside a run.
- Determinism and replay, with a scripted run that buys from three shops.
- Versus: identical digest, identical fuzzer lines, and timing within 2%.
- Godot: runner_test covers buying, a card, a reroll and the HUD strip; run_shot photographs a full shop and a build on the results
  screen.
- Acceptance: the owner plays two runs and they feel different by the third shop.

## 7. Process
1. The modes-online agent reads this file, RUNNER_PLAN.md (sections 3.5 and 11) and RUNNER_WORLD.md.
2. It asks the Moveset porting agent for the next free SIM_VERSION (132 or later on 15 October), and asks before any schema change.
   Items live in `RunContent`, which is Rust data, so no schema change is expected.
3. It works on branch `runner-r3`, in its own worktree, with one `target/` folder, committing and pushing in small steps (two phases
   were cut off by usage limits).
4. It reports back. The executive producer reviews, checks versus timing, lands the work on main and movesets, updates the desktop
   game and publishes a GitHub release.

## 8. Questions for the owner (can be answered after playing R3)
- Should a consumable be held and used with a button, or used as soon as it's bought?
- Are 18 items enough to start, and is any of them unwanted?
- Should prized items appear from tier 3, or earlier?
