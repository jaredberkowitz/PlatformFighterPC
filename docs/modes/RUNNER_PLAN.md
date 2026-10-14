# Single-player mode: the endless run (plan, draft 2; R1 built)

Status: **draft 2, owner's answers in (14 October 2026); R1 (the vertical slice) built on branch `runner-r1`, sim 120: see section 7.** Decisions are marked **Decided**. Owner brief, 14 October 2026: a single-player endless-runner mode inspired by the mobile
game *Punch Quest* (Rocketcat Games and Madgarden, 2012), using this game's fighters, physics and art. The player runs right, beats
enemies for currency, spends it in shops on power-ups, stat boosts and other fighters' moves of the same archetype, fights a generated
boss every so often, and tries to get as far as possible.

## 1. What we take from Punch Quest, and what we change

*Punch Quest* in brief:
- The hero runs right by itself, controlled by two thumbs (an uppercut-jump and a forward punch; both together block).
- Kills keep a combo going and fill a power meter for a supermove.
- Kills pay "punchos", spent on boosts and unlocks.
- Branching paths offer route choices, and rare events and quests add variety.

Its strength is that every second is a small decision: punch now, jump now, which path.

Our game is a full platform fighter: dash, short hop aerials, specials, shield, DI. Simplifying it to two buttons would throw that
away. So the plan keeps the full moveset and borrows the *structure* (forward pressure, constant enemies, combo-fed currency, shops,
bosses, branching paths) rather than the controls. A two-button "casual" control scheme can come later as an option.

## 2. Core loop

1. **Pick a fighter** (any built-in fighter, or a created one).
2. **Run.** The screen scrolls right at a steady pace that rises with distance. Enemies, gaps, platforms and pickups come in hand-made
   *chunks* chained by a seeded generator.
3. **Fight.** Enemies die from damage or from being launched off screen. A **combo meter** counts hits and kills without a break and
   multiplies the coins earned. Getting hit breaks it.
4. **Shop** every few hundred metres: a storefront chunk you walk into. Three to four offers, rerollable for coins.
5. **Boss** every N shops: the scroll stops in an arena chunk until the boss is beaten.
6. **Die** (lose your last life) and the run ends. You see distance, coins, kills and best combo, and can retry with a new seed.

The scroll, enemy density and enemy toughness rise with distance (a difficulty curve per region).

## 3. Key design decisions (recommendations; the owner decides)

### 3.1 Movement: exactly the versus game's, inside a gently scrolling view (**Decided**)
- **Owner: free movement, the same layout and movement as the base game.** The fighter has every versus control and physics rule
  (dash, run, jumps, aerials, specials, shield, wavedash, ledges). Nothing is simplified, and the mode reads the same input and
  physics code.
- To keep it a runner, the **camera only moves forward**: it follows the player right and never scrolls back. It also creeps forward
  slowly by itself, slowly at first and a little faster in later cities, so standing still isn't free. Falling behind the left edge
  pushes the fighter along; being pinned against a wall there costs a life.
- *If the creeping camera feels wrong in play, it is one number (its speed) away from a pure follow camera: the player sets the pace.*

### 3.2 Health: damage percent and lives (**Decided**)
- Our own rules: damage percent builds up, enemies launch you further the higher it is, and you lose a life when launched off
  screen, into a pit, or crushed against the left edge.
- **Lives are scarce:** start with 3. More come only from a shop power-up (pricey, limited per shop) or, rarely, found along the
  run (a hidden 1-up on a risky route, at most about one per city). Heal items lower your percent.

### 3.3 Enemies: a new lightweight entity, not fighters
- The sim holds at most 4 fighters. A runner needs 10 to 20 enemies on screen.
- **Recommended:** a compact `Mob` type in the sim (position, speed, state, hit points, hitstun, kind), up to 16 at once. Mobs use
  the same knockback, hitstun and hitlag as fighters, so juggles, combos and spikes all work on them.
- Small enemies die from **stamina** (a few hits) and burst away when they do. Big ones must be launched off screen.
- Enemy behaviours are scripts in the existing script language (`sim-script`), so each new enemy is data, not engine code. Later,
  the editors could make enemies too.
- First enemy set, shaped by the city:
  - a walker that rushes you;
  - a shielded guard that must be grabbed or attacked from above;
  - a thrower on a rooftop;
  - a diving flyer (a pigeon or a drone);
  - a heavy that needs launching off screen;
  - a bomber whose explosion hurts everyone.

### 3.4 Bosses: real fighters, driven by the CPU
- A boss is a roster fighter controlled by the existing computer opponent (`cpu_input`), with **stamina** (a boss health bar)
  instead of stocks, so the fight has a clear end.
- **Generated** from the run's seed and distance:
  - a fighter and its moveset;
  - one or two modifiers: giant, metal (heavy, no flinch from weak hits), fast, a fire aura, or an extra midair jump;
  - a CPU level;
  - a stamina pool scaled to the player's power so far.
- **Beatable by design:**
  - the CPU level is capped below its best;
  - the boss pauses between combos;
  - a health pickup appears halfway through;
  - the arena has no pits.
- Losing to a boss costs a life and restarts the boss at the stamina it had left.

### 3.5 Shops and items
Currency is coins, from kills, multiplied by the combo meter. Coins are also scattered along risky routes. Shop offers are random from
weighted pools, with rarity rising with distance. There are four kinds:
- **Stat boosts:** the four creator stats (power, speed, weight, jump), the same ones as in the character creator, applied in Rust.
- **Properties:** for example lifesteal on kills, a coin magnet, burning hits, armor on smash attacks, an extra midair jump, a
  stronger shield, longer invincibility after a lost life, a combo meter that doesn't break on the first hit, or bigger hitboxes.
  Each is a rule the sim applies, sold in tiers.
- **Moves from other fighters of the same archetype:** replace one move slot (for example the forward smash or the side special)
  with another fighter's version. Archetypes keep the swaps believable: a sword fighter only gets sword moves, so animation and
  reach still fit.
- **Consumables:** heal 30%, a shield bubble that absorbs one hit, a bomb.

**Decided (executive producer's pick, owner deferred):** everything bought is lost at the end of the run, the classic roguelite rule. A
light **meta layer** comes later: permanent unlocks such as new item pools, starting perks and cosmetic hats, earned by distance and
milestones, never needed to compete. Punch Quest leaned on long unlock grinds, which reviewers found slow, so keep the unlocks
generous.

### 3.6 Archetypes (**Decided for now**; will change as fighters are added, and the Staff's place is open)

| Archetype | Fighters (class) | Why together |
| --- | --- | --- |
| Blades | Longsword, Greatsword, Buckler | sword in hand, similar reach and swing shapes |
| Fists | Embers, Gauntlets, Bracers, Claws | punches and kicks, close range |
| Agile | Needles, Scamp | light, fast, tricky movement |
| Odd ones | Staff, Clutch, Settler (Pockets) | unique tools: tongue, pocket, staff magic (the owner's example: the Villager-style and Yoshi-style fighters together) |

A move swap only goes between fighters of one archetype. Created fighters use their class's archetype, and **can play the mode from
the start (Decided)**. The table is data (a list in content), so new fighters slot in without code. The moveset porting agent adds
each new fighter's archetype as part of its port.

### 3.7 Setting: a journey from city to city (**Decided**: city rooftops; the loop below answers "how does it repeat")
The run is a **road trip through an endless chain of cities**. Each city is one *leg*; at its end you leave it and arrive somewhere new.
That gives the endless mode a sense of progress ("City 7") instead of one street repeating, and it reuses art.

**One city (a leg), about 3 to 4 minutes:**
1. **Arrival:** the outskirts, low buildings, a sign with the city's name ("Welcome to Port Amber").
2. **Rooftops and street:** the main stretch. The two routes split and rejoin: rooftops are risky with more coins, the street is
   safer with more enemies. One or two **shops** (a storefront on the street, a rooftop vendor on the roofs).
3. **The city boss** at the far edge: a landmark arena (a stadium roof, a train yard, a harbour crane deck).
4. **Departure:** a short, calm *transit* chunk with no enemies. It changes the scene and paces the run, and the art differs each time:
   - **a bridge** out of the city;
   - **a train**: you ride on the roof, and later legs can add enemies there;
   - **a highway overpass**;
   - **a ferry**;
   - **a tunnel**.
   The next city fades in behind it.

**Each new city varies on a few cheap axes, drawn from the run's seed:**
- **Palette and time of day.** The lighting system already supports this. Cities follow day, dusk, night, then dawn as the run goes
  on, so the sky marks how far you've travelled.
- **Weather:** clear, rain, fog or snow. These are light overlays.
- **Architecture set:** a kit variant, such as brick downtown, glass towers, a harbour town, a desert adobe town or a neon district.
  The art director builds a few kits; each one can appear in many palettes.
- **A name** from a generated list, shown on the arrival sign and the results screen ("Reached City 9: Coldwater, 4.2 km").
- **Difficulty tier:** the camera's creep, enemy mix and toughness, item rarity and boss modifiers all step up each city.

From city 2 on, a leg can now and then be a **non-city stretch** that passes through one of our other stage themes (the grove, the
canyon, Flat Island) on the road between cities. This brings back existing stage art without breaking the trip's logic.

**Verticality:** each chunk stays within about one and a half screens of height, the camera follows height smoothly with a dead
zone, and the street route is never more than one screen below the roofs, so both routes stay readable.

## 4. Technical shape

- **Simulation (Rust), deterministic and seeded.** One seed gives the same run every time, which makes replays, a daily-seed mode
  and, later, online leaderboards possible (a server can re-run a submitted run to check a score). Everything follows the sim rules
  in `CLAUDE.md`: no floats, `GameState` is `Copy` and hashed, and tunables live in content.
  - **Streaming stage:** the sim keeps a small ring of active chunks (chunk id and x offset) instead of one fixed stage. Collision
    and ledges look across the active chunks. Chunks are content data (platforms, ledges, spawn markers, pickups), checked by
    `sim-content`, and the stage editor could later author them.
  - **Mobs:** a new fixed-size pool in `GameState` (section 3.3), sharing `combat.rs`.
  - **Run state:** distance, scroll speed, coins, combo, lives, owned items and properties, the next shop and boss. All hashed.
  - **Items and properties:** a data list. Each property is a named rule in the sim, like the existing ruleset.
- **Game (Godot):**
  - a run scene (`runner.tscn`) with a scrolling camera;
  - a HUD (distance, coins, combo meter, percent, lives, the boss bar);
  - the shop screen;
  - the results screen;
  - a "Single player" entry on the main menu.
- **Art and audio:** the city kit and region variants from the art director; enemy designs in the house style (round blob bodies,
  2D faces); music per region; coin and combo sounds.
- **Not in v1:** online, leaderboards, daily seeds, the meta layer, regions beyond the city, casual controls.

## 5. Phases

| Phase | What | Done when |
| --- | --- | --- |
| R0 | This plan, refined with the owner; enemy and item lists agreed | owner sign-off |
| R1 | Vertical slice: the forward-only creeping camera, 5 hand-made chunks on a loop, 2 enemy kinds, coins and the combo meter, lives, the run HUD, a results screen, the menu entry | a 2-minute run is fun to play with placeholder art |
| R2 | The generator: chunk pools by difficulty, the two routes, the difficulty curve, 6 enemy kinds | runs differ and get harder smoothly |
| R3 | Shops and items: stat boosts, 10 properties, consumables, rerolls | a run's build changes how it plays |
| R4 | Bosses: generated CPU fighters with stamina and modifiers, an arena chunk | a boss every few shops, beatable in 2 to 3 tries at worst |
| R5 | Archetype move swaps | a sword fighter can buy another sword fighter's moves |
| R6 | The journey: city legs with arrival, boss and transit, generated names, palettes, time of day and weather; the first architecture kits from the art director; music | three cities in a row feel like a trip |
| R7 | Meta layer, daily seed, then online leaderboards (re-run to verify) | later, with the online work |

## 6. Decisions (14 October 2026) and what is still open

| Question | Decision |
| --- | --- |
| Movement | Exactly the versus game's movement and controls, inside a forward-only, gently creeping camera (3.1) |
| Health | Damage percent and lives; extra lives only from a shop power-up or rare finds (3.2) |
| Upgrades | Reset every run; a light meta layer of permanent unlocks later (3.5) |
| Archetypes | The table in 3.6 for now; it will change as fighters are added, and the Staff's place is open |
| Setting | City rooftops and streets, as a journey through an endless chain of generated cities with transit legs between (3.7) |
| Created fighters | Can play from the start |
| Who builds it | The modes and online agent (`.claude/agents/modes-online.md`) builds it phase by phase; the executive producer directs and reviews |

Still open, for later phases:
- the first six enemy designs (the art director's call, within section 3.3's roles);
- the full item list and prices (in R3);
- the boss modifier list (in R4);
- whether the transit legs ever have enemies (a train-roof fight could be a highlight).

## 7. R1 progress (vertical slice, sim 120, branch `runner-r1`)

### What can be played
- **Main menu, Run** (the tile under Play) opens the fighter choice in its run mode: one player, any built-in or made fighter. J or Enter
  locks in, Enter starts. In the run, the city's rooftops and street stream past. The camera only goes forward: it creeps on and follows
  you right. Walkers rush you and flyers dive at you. Beat them for coins, multiplied by your combo. You have three lives: a launch off
  the screen, a fall into a gap, or being crushed against a wall by the screen's left edge each costs one. The results show distance,
  coins, mobs beaten, the best combo and the seed. **Run again** starts a new seed; **Change fighter** goes back to the choice.
- The controls are the versus game's in every way: same input reader, bindings, controllers and physics. Esc returns to the fighter
  choice, P pauses, F1 shows the debug overlay.
- Launched directly: `Godot --path godot -- --run [--fighter=<name>] [--seed=N]`.
- Headless: `pftool run-bot [seeds] [frames]` plays runs with a simple bot (it runs right, jumps at walls and gaps, and jabs what is in
  front). `pftool run-bench [frames]` times the step with all 16 mob slots full.

### How it is built (and why)
- **The run lives in the sim**, deterministic and hashed: `GameState::run` (`sim-core/src/run/`). In versus it is inactive and nothing
  reads it. `step` is unchanged; a run is `run::step_run`, which is the versus `step` on the streamed stage followed by the run's own
  rules. Versus, netplay (`pftool net-fuzz`: 0 mismatches), replays and the editors behave as before, and every existing test passes
  unchanged.
- **The camera (the scroll) is in the sim**, as the brief asked me to decide. Its left edge pushes the fighter and can crush it, and the
  blast zone and mob spawning follow it, so it must replay exactly and be in the checksum. Godot only draws it: the screen's edges are the
  sim's, and the camera follows height with a dead zone. The creep pauses while the fighter is on its revival platform.
- **The streamed stage** is a ring of 4 chunk slots plus fixed pools of 24 platforms and 8 ledges (the versus ledge table) in the run
  state. A pool entry keeps its index while it lives, so a fighter's `platform` and `ledge` indices stay valid as chunks come and go.
  Ground that continues across a seam at one height is one platform, extended, so dashing, rolling and wavedashing across a seam work
  exactly as on one floor (a test checks this). Each frame, the pools are written into the run's working copy of the content's `Stage`
  (free entries become unreachable padding), with the blast zone and respawn point placed around the view. All of the versus collision,
  landing, ledge and knock-out code then works unchanged.
- **Mobs** are a fixed pool of 16 in the run state. Hits on them use the versus formulas (knockback, hitstun, hitlag, launch angles,
  charge, rage). Their hits on the fighter go through the versus `combat::apply_hit`, with the mob standing in on an unplayed fighter slot
  for that one call, so shields, DI, crouch cancelling, armor and counters work against them. Behaviour is a minimal Rust state machine
  per kind. The walker approaches, stops at edges, hops up steps, winds up and lunges. The flyer hovers beside and above you, winds up and
  dives. **R2 should move this behaviour into scripts** (`sim-script`), so that new enemies are data.
- **Run content** (rules, chunks and mob kinds: `sim-core/src/run/data.rs`, `RunContent::placeholder`) is Rust data, like the stage
  library, checked by `sim_content::check_run`. The `.pfc` format is unchanged (no schema bump). Authoring chunks in the stage editor is
  for R2.
- **Godot**: `runner.tscn` / `scripts/runner.gd` extends the match scene (`main.gd`) and overrides only what a run presents differently
  (camera, stage, mobs, display, results and keys). `scripts/run_stage.gd` builds each chunk once from the stage art's blocks in the city
  look, `scripts/mob_view.gd` draws the placeholder mobs, and `ui/run_hud.gd` and `ui/run_results.gd` follow the menus' painted style.

### Tunables (`RunRules::standard`, world units and frames)
| Rule | Value | Meaning |
| --- | --- | --- |
| `lives` | 3 | lives at the start |
| `view_width` | 30 | the view's width (the versus camera's nearest shows about 25) |
| `follow` | 0.4 | the camera holds the fighter 40% across when following |
| `creep` / `creep_growth` / `creep_max` | 0.02 / +0.0025 per 100 m / 0.0625 | the camera's own forward speed (running is about 0.25); **0 makes it a pure follow camera** |
| `cam_max_speed` | 0.5 | the camera's top speed |
| `push_margin` / `push_speed` | 0.8 / 0.35 | the left edge keeps the fighter 0.8 inside the view, pushing at up to 0.35 a frame (not while launched) |
| `crush_depth` | 1 | a fighter pinned against a wall is crushed once the edge is 1 past it |
| `blast_margin` / `blast_top` / `blast_bottom` | 6 / 22 / -14 | blast zone: 6 beyond the view's sides, and these heights |
| `spawn_ahead` | 2 | mobs appear 2 beyond the view's right edge |
| `respawn_at` | 0.35 | a lost life comes back 35% across the view |
| `combo_step` / `combo_max` | 5 / 5 | coins are multiplied by 1 + combo / 5, at most 5; a hit taken or a life lost resets the combo |
| `credit_frames` | 180 | a mob you hit that leaves the screen within 3 s counts as your kill |
| `burst_frames` | 20 | how long a beaten mob takes to pop |

**Mobs** (`MobKind::walker`, `flyer`):
- Walker: 12 stamina, weight 80, speed 0.12. It lunges at 0.3 after an 18-frame windup; its hit is 6% at 45 degrees (base 30, growth 60).
  It pays 5 coins.
- Flyer: 8 stamina, weight 60. It hovers 5 above you and dives at 0.35 after a 24-frame windup; its hit is 8% at 60 degrees (35 / 70). It
  pays 8 coins.

**Chunks**: street (where every run starts), gap (with a stepping stone), step (a taller building), escapes (stacked fire escapes) and drop
(a wide gap with a ledge under its near edge). Every chunk comes once a lap in a seeded order, never the same one twice running.

### Tests and numbers
- `sim-core/tests/runner.rs` (23 tests):
  - streaming: the chunks always cover the view and never lose a platform; the chunk order is seeded; ground joins across a seam; ledges
    and platforms work in a later chunk;
  - camera and lives: the camera creeps, follows and never goes back; the edge pushes; being crushed against a wall, falling into a pit,
    or leaving the screen each cost a life, and the last life ends the run;
  - mobs: knockback grows with percent; a mob out of stamina bursts and pays; an off-screen mob is a kill only with credit; a walker hits
    and breaks the combo; a shield blocks a mob; a flyer dives; one swing hits a mob once per hit group;
  - economy: the combo multiplier and distance;
  - determinism and the checksum: same seed and inputs give the same run, a restored state replays the same, random play with 16 mobs
    keeps the invariants, the checksum sees every run and mob field, and the run content hashes.
- `sim-content` `run::tests`: the built-in run is valid, and bad chunks are refused.
- Godot: `tests/runner_test.gd` covers the bridge, the scene, the display, the results, Run again, the menu tile and the run-mode fighter
  choice. `tests/run_shot.gd` photographs a run.
- Cost: about 0.002 ms a frame on average with 16 mobs (release build, `pftool run-bench`); the worst frame was under 0.1 ms.
- The bot (`pftool run-bot 8 7200`, two minutes each): 6 of 8 runs survived, covering 590 to 800 m and beating 5 to 16 mobs, with best
  combos of 8 to 30. The other two lost all three lives (to gaps) at about 57 and 78 seconds. (The bot only runs, hops and jabs; a
  player fights far better.)

### Known limits (R1)
- **Distance limit:** positions are 16.16 fixed point, good to about 32,000 units (roughly 40 minutes of non-stop running). R6's transit
  legs are the natural place to shift the world back to the origin.
- Mobs and fighter moves:
  - grabs, throws and pushes (wind boxes) do not affect mobs;
  - a multi-hit's rehit does not hit a mob again within one swing;
  - a hitbox's grounded-only or air-only targeting is ignored on mobs;
  - shots hit mobs in a simple way (no reflecting, clanking or pocketing them);
  - a mob's hit on the fighter counts as a "fresh" move (the 1.05 damage bonus).
- Runs are not recorded as replays yet. They are deterministic, so this is part of the R7 leaderboard work.
- Coins come only from kills (no pickups yet). There are no shops, items, bosses or difficulty curve yet (R2 to R4).
- The look is placeholder: chunk pieces are drawn separately, so a seam shows where ground continues from one chunk to the next; the city
  skyline is fixed to the camera (no parallax); the mobs are plain blobs.

### For the art director (placeholders to replace)
1. **A city kit** for chunks: rooftop and street modules that join seamlessly across chunk seams, gaps, steps (taller buildings), fire
   escapes, awnings, water towers and billboards; a storefront for shops (R3).
2. **Enemies** in the house style (round blob bodies, 2D faces): the walker (a street tough or a rushing critter) and the flyer (a pigeon
   or a drone), each with idle, move, windup (a clear tell), attack, hit and burst. Later: the guard, thrower, heavy and bomber (3.3).
3. **A coin** and a pop for a beaten mob, a **combo** flourish for the HUD, and a city skyline that can scroll in parallax.
4. **Shop** art for R3.

### Decisions wanted
- Does the creeping camera feel right at 0.02 (about a twelfth of running speed)? It is one number, and 0 makes it a pure follow camera.
- At the screen's left edge, a launched fighter can now fly past it and be knocked out; once its hitstun ends, the edge pushes it back in.
  Should the edge stop launches instead?

Sources: [TouchArcade review](https://toucharcade.com/2012/10/26/punch-quest-review/),
[AppSpy review](https://www.appspy.com/punch-quest-review), [Destructoid review](https://www.destructoid.com/reviews/review-punch-quest/),
[App Store listing](https://apps.apple.com/bn/app/id554223561).
