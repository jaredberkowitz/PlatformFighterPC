# Single-player mode: the endless run (plan, draft 3; R1 to R2b built)

Status: **draft 3 (14 October 2026): the owner played R1 and replaced the creeping camera with rooms (section 3.1). R1 (the vertical
slice) is built (sim 120, section 7); R1b (rooms) is built (sim 122, section 8); R2a (the world: slopes, room variants, the fork, the
walk-in shop) is built on branch `runner-r2`, sim 124 and content schema 8: see section 9; R2b (fighter-shaped grunts, the difficulty
curve, hazards, the runaway van, branching forks) is built on branch `runner-r2b`, sim 127: see section 10.** Decisions are marked **Decided**. Owner brief, 14 October 2026: a single-player endless-runner mode inspired by the mobile
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
2. **Run.** The world is a chain of hand-made *rooms*, left to right, laid by a seeded generator as you go (section 3.1). You set the
   pace and can go back a little; the rooms behind close off as you go on. Enemies, gaps, platforms and pickups are in the rooms.
3. **Fight.** Enemies die from damage or from being launched off screen. A **combo meter** counts hits and kills without a break and
   multiplies the coins earned. Getting hit breaks it.
4. **Shop** every 5 to 7 rooms: a storefront you enter by pressing up at its door. Three to four offers, rerollable for coins.
5. **Boss** every N shops: an arena room whose gate stays shut until the boss is beaten.
6. **Die** (lose your last life) and the run ends. You see distance, coins, kills and best combo, and can retry with a new seed.

Enemy density, enemy toughness and the rooms' mix rise with distance (a difficulty curve per region).

## 3. Key design decisions (recommendations; the owner decides)

### 3.1 Movement and the world: the versus game's movement, in a chain of rooms (**Decided**, rewritten 14 October 2026)
- **Owner: free movement, the same layout and movement as the base game.** The fighter has every versus control and physics rule
  (dash, run, jumps, aerials, specials, shield, wavedash, ledges). Nothing is simplified, and the mode reads the same input and
  physics code.
- **Owner, after playing R1:** "Regarding screen creep: I think it would be better if it was able to back-track, within reason. As it
  is a runner generating the layout as it goes, you should not be able to backtrack too far. A possible solution would be moving from
  'rooms' left to right, with it storing a few rooms further back, and some 'rooms' having enterable shops, etc. The concept would be
  similar to a rogue-like's map, e.g. Rogue Legacy, but with a lot more simplicity, catered to the endless runner style."
- **Decided (executive producer's spec from that direction):**
  1. **The world is a chain of rooms, left to right.** A room is one to two screens wide (28 to 64 units; the view shows 30) and
     keeps the height rule of 3.7. R1's chunks became rooms, and its chunk ring the room window. A pool entry still keeps its index
     while it lives, and ground continuing across a seam is still one platform, so dash and wavedash cross seams.
  2. **Backtracking is limited.** The window holds the current room (the furthest the player has entered), up to **2 rooms behind**
     and **1 ahead** (laid when you enter the current one); both are `RunRules` tunables (`rooms_behind`, `rooms_ahead`). Entering a
     new room drops the oldest room behind, and the left edge of the oldest kept room is a solid wall: a closed gate in the art and a
     real stage wall in the sim, so fighters and mobs collide with it and can wall-jump off it. The forward-only creep and the
     crush-death are gone; `creep` stays as a tunable defaulting to 0, so it can be A/B tested (when on, the view's left edge creeps
     and pushes the fighter, but never crushes). The camera follows the player both ways inside the window, smoothly, clamped to the
     window's walls. The blast zones follow the camera, except that the left side is the gate.
  3. **Room types (data, seeded):**
     - **Street:** plain running with a few mobs; the most common type.
     - **Arena:** the exit closes (a gate on its right edge) as you enter, until every mob in the room is beaten; then it opens with a
       clear signal ("CLEAR!" and the bars sliding up) and pays a coin bonus.
     - **Shop:** a storefront in the room. Standing at its door and pressing up **enters** it. R1b sells two placeholder wares, heal
       30% and an extra life (pricey, one per shop); the list is data (`ShopItem`), R2/R3's foundation.
     - **Rest:** rare, no mobs, a small heal as you enter.
     Generation (seeded): no two arenas in a row, a shop every 5 to 7 rooms, and the first room is a safe street.
  4. **The map:** a Rogue Legacy-style strip at the top of the HUD: the rooms behind (dimmed, the dropped ones shown shut), the
     current room, and the next room ahead with its type. Linear, no branching yet (see the fork note below).
  5. **Pressure without the creep:** the combo meter's decay, the arenas, and distance as the score are enough for now. *Later
     option, if runs prove too passive:* a slow "danger" rising from behind (smoke, a flood, a wrecking crew) that makes lingering in
     the rooms behind costly, without a hard scroll.
  6. **The owner's R1 question 2** ("knocked past the left edge?") is answered by the gate: the left boundary is a wall. A launch
     into it is a wall hit (bounce, tech or wall jump), never a knock-out.
- **A fork (built in R2a as one room with two roads, section 9; this two-exit version was not built):** a room could have two exits, a rooftop exit high on its right wall and a street exit at ground
  level, each leading to its own next room. The window would then lay two rooms ahead (one per exit, both drawn on the map one above
  the other), and the exit the player takes decides which becomes the current room; the other is dropped at once, its gate shut.
  This keeps the window small (behind + 1 + 2 ahead), keeps the map a strip with one short split at its right end, and gives the
  rooftop/street route choice of 3.7 (rooftops risky with more coins, street safer with more mobs) without any backtracking across
  branches.

### 3.2 Health: damage percent and lives (**Decided**)
- Our own rules: damage percent builds up, enemies launch you further the higher it is, and you lose a life when launched off
  screen, into a pit, or crushed against the left edge.
- **Lives are scarce:** start with 3. More come only from a shop power-up (pricey, limited per shop) or, rarely, found along the
  run (a hidden 1-up on a risky route, at most about one per city). Heal items lower your percent.

### 3.3 Enemies: a new lightweight entity, not fighters (superseded: R2b's fighter-shaped grunts, `RUNNER_WORLD.md` section 4 and section 10 here)
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
  - **Streaming stage:** the sim keeps a small window of rooms in play (room id, number and x offset) instead of one fixed stage,
    with a wall at its left end and an arena's gate when one is shut. Collision and ledges look across the rooms in play. Rooms are
    content data (type, platforms, ledges, spawn markers, a shop's door), checked by `sim-content`, and the stage editor could later
    author them.
  - **Mobs:** a new fixed-size pool in `GameState` (section 3.3), sharing `combat.rs`.
  - **Run state:** distance, the camera, the room window and its walls, coins, combo, lives, the shop, owned items and properties,
    the next shop and boss. All hashed.
  - **Items and properties:** a data list. Each property is a named rule in the sim, like the existing ruleset.
- **Game (Godot):**
  - a run scene (`runner.tscn`) with a following camera;
  - a HUD (distance, coins, combo meter, percent, lives, the map, the boss bar);
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
| R1 | Vertical slice: the forward-only creeping camera, 5 hand-made chunks on a loop, 2 enemy kinds, coins and the combo meter, lives, the run HUD, a results screen, the menu entry | a 2-minute run is fun to play with placeholder art (built, sim 120) |
| R1b | Rooms (owner's change after R1): the room window with a wall behind, the two-way camera, streets, arenas, a placeholder shop (heal, extra life), rest stops, seeded room types, the map strip | backtracking feels fair and the rooms read clearly (built, sim 122) |
| R2a | The world (owner, 14 Oct; see `RUNNER_WORLD.md`): slopes, room variants with optional parts, mirroring and obstacle pieces, the high/low fork, elevation lines, footing objects with `look` tags, the walk-in shop with a shopkeeper | rooms feel varied and believable; you can tell footing from decoration (built, sim 124) |
| R2b | Enemies and the curve: fighter-shaped grunts (six kinds, base-game bodies with borrowed moves and tells), budgets and placement rules, hazards that hurt enemies, room pools by difficulty, the difficulty curve, the first rare event; the owner's R2a answers (coins 1, branching forks, the sulking shopkeeper) | runs differ and get harder smoothly (built, sim 127) |
| R3 | Shops and items: stat boosts, 10 properties, consumables, rerolls | a run's build changes how it plays |
| R4 | Bosses: generated CPU fighters with stamina and modifiers, an arena chunk | a boss every few shops, beatable in 2 to 3 tries at worst |
| R5 | Archetype move swaps | a sword fighter can buy another sword fighter's moves |
| R6 | The journey: city legs with arrival, boss and transit, generated names, palettes, time of day and weather; the first architecture kits from the art director; music | three cities in a row feel like a trip |
| R7 | Meta layer, daily seed, then online leaderboards (re-run to verify) | later, with the online work |

## 6. Decisions (14 October 2026) and what is still open

| Question | Decision |
| --- | --- |
| Movement | Exactly the versus game's movement and controls (3.1) |
| The world (owner, after R1) | A chain of rooms with limited backtracking: 2 rooms behind, 1 ahead, a wall behind; no creep (an A/B tunable, 0); streets, arenas, shops, rest stops; a map strip (3.1) |
| Knocked past the left edge (R1 question 2) | The left boundary is a wall (the gate), never a knock-out (3.1) |
| Health | Damage percent and lives; extra lives only from a shop power-up or rare finds (3.2) |
| Upgrades | Reset every run; a light meta layer of permanent unlocks later (3.5) |
| Archetypes | The table in 3.6 for now; it will change as fighters are added, and the Staff's place is open |
| Setting | City rooftops and streets, as a journey through an endless chain of generated cities with transit legs between (3.7) |
| Created fighters | Can play from the start |
| Who builds it | The modes and online agent (`.claude/agents/modes-online.md`) builds it phase by phase; the executive producer directs and reviews |

After R2a (owner, 14 October 2026): a coin is worth 1; forks may lead to different rooms (built in R2b: the branching fork and its
treasure alley); slopes stay as they are; hitting the shopkeeper makes it refuse to sell for a while (built: 3 seconds). After R2a's
play the owner also asked for enemies based on the base game's characters (`RUNNER_WORLD.md` section 4, built in R2b).

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

### Decisions (answered by the owner after playing R1)
- The creeping camera: replaced by rooms (3.1, section 8). The creep is kept as an A/B tunable at 0.
- The left edge: it is a wall now (3.1).

## 8. R1b progress (rooms, sim 122, branch `runner-r1b`)

### What can be played
- **Main menu, Run** as before. The city now comes in **rooms**: you run right at your own pace and can go back up to two rooms; behind
  that a barred gate (a real wall: you can wall-jump off it) closes the way. The **map** at the top shows the rooms behind (dimmed, the
  ones closed off shown barred), the room you are in (ringed) and the next one with its type: a building for a street, crossed swords
  for an arena, a coin for a shop, a green cross for a rest stop.
- **Streets** have a few walkers and flyers. An **arena** shuts a red gate behind its far side as you walk in ("ARENA!") and wakes
  every mob in it; beat them all and the bars slide up ("CLEAR! +25"). A **shop** comes every 5 to 7 rooms: stand at the storefront's
  door ("UP: ENTER THE SHOP") and press up. Inside, the world stands still; up and down pick, attack or jump buys, shield or special
  leaves. It sells **Heal 30%** (30 coins) and an **Extra life** (150 coins, one per shop). A **rest stop** (rare) heals 20% as you
  enter. Results now show rooms reached and arenas cleared.
- Headless: `pftool run-bot 8 7200` (2 minutes each): the bot reached rooms 6 to 16 and cleared 0 to 4 arenas; one of eight lost all
  its lives. (It only runs, hops and jabs, so arenas with flyers hold it up.)

### How it is built (and why)
- **The room window** (`sim-core/src/run/mod.rs`): `Run::slots` is the window (each slot a room id, its number and its edges);
  `Run::room` is the current room. Entering the next room (the player's feet `enter_depth` past its left edge) drops the rooms more than
  `rooms_behind` back, lays rooms up to `rooms_ahead` ahead, and does what the room's type does. Dropping frees that room's platforms
  and ledges, removes its mobs, and trims ground joined across the seam to the new wall, so the pool index a fighter stands on stays
  valid (a test checks this).
- **The walls are stage blocks.** The stage the versus code collides with is the pool, then two fixed entries: the left wall
  (`WALL_INDEX`, at the oldest room's left edge) and an arena's gate (`GATE_INDEX`, padding while open). Each reaches from below the
  blast zone to above its top, so nothing gets over, under or onto it; fighters stop at it, wall-jump and wall-tech off it through the
  versus code unchanged. Mobs collide with it too, and every mob (flyers included) is kept inside the window and on its own side of a
  shut gate.
- **The camera** follows the player both ways, easing (`cam_ease` of the distance a frame, at most `cam_max_speed`), clamped between
  the wall and the window's right end (or a shut gate). The left blast line is behind the wall, out of reach; the right one follows
  the camera, so launches off the right side still knock out. Mobs leave the world only above or below the blast zone or out of the
  window, or when launched by the player off the side of the screen (a kill with credit).
- **The shop is a state of the run, not a separate room.** Of the two options in the spec (a sim-side "inside" state with a menu
  overlay, or a small interior room) I chose the state: while `Run::shop_open` is set, `step_run` runs only the shop (the fighter,
  mobs and camera stand still, the frame count goes on), and the player's own inputs (stick presses and button presses, edge-detected
  against the previous frame's, which the run keeps and hashes) move the cursor and buy. It is deterministic and replays exactly
  (a test replays a scripted shopping trip checksum by checksum), needs no second stage, camera cut or mob suspension, and the menu
  is just a drawing of the state. A shop remembers what it sold while its room is in play, so the extra life stays one per shop even
  if you leave and come back. The wares are data (`RunContent::items`: kind, price, amount, limit per shop).
- **Arenas:** entering one shuts the gate at its right edge, spawns all its markers at once and wakes its mobs. Each mob records the
  room it came from (`Mob::room`); the gate opens when all of the room's markers have spawned and none of its mobs is left (a mob in its
  burst counts as beaten), paying `arena_bonus`. The camera stays inside the gate meanwhile.
- **Generation** (`next_kind`, `next_room`): the first room is the start street (no mobs, never seen again); a shop when one is due
  (every `shop_min` to `shop_max` rooms, drawn from the run's generator); otherwise streets, arenas and rest stops by weight, never two
  arenas or two rest stops in a row. Within a type, a room is drawn at random, not the last one again while another will do. The last
  16 rooms' types are kept for the map (`Run::kinds_log`).
- **Determinism:** every new field is in `Run::hash_into` and in the checksum test's list; the room content (types, doors, wares and
  the new rules) is in `RunContent::hash`. Versus is untouched (`GameState::run` stays inactive; every versus test passes unchanged).
- **Godot:** `scripts/run_stage.gd` draws the left gate (a brick pillar with bars), the arena's red gate (its bars slide up when it
  opens), a storefront with a lit window and a SHOP sign at each shop's door, and a REST sign; `ui/run_hud.gd` draws the map, the door
  prompt, the arena hint and the shop panel; `ui/run_results.gd` shows rooms and arenas. Bridge: `run_rooms`, `run_map`, `run_shop`,
  `run_shop_names`; `run_info` and `run_view` grew (see their doc comments).

### Tunables added or changed (`RunRules::standard`)
| Rule | Value | Meaning |
| --- | --- | --- |
| `rooms_behind` / `rooms_ahead` | 2 / 1 | the window; `rooms_behind + 1 + rooms_ahead` is at most 4 (the ring) |
| `enter_depth` | 2 | you have entered a room once your feet are 2 past its left edge |
| `gate_thickness` | 1 | the walls' thickness |
| `cam_ease` / `cam_max_speed` | 1/8 / 0.5 | the camera closes an eighth of the distance a frame, at most 0.5 |
| `follow` | 0.4 | the camera holds the player 40% across (both ways now) |
| `creep` / `creep_growth` / `creep_max` | 0 / 0 / 0 | the optional creep, off; `push_margin` / `push_speed` (0.8 / 0.35) only matter when it is on. `crush_depth` is gone |
| `shop_min` / `shop_max` | 5 / 7 | a shop every 5 to 7 rooms |
| `street_weight` / `arena_weight` / `rest_weight` | 70 / 22 / 8 | the other rooms' odds |
| `arena_bonus` | 25 | coins for clearing an arena |
| `rest_heal` | 20 | percent healed at a rest stop |
| `door_reach` | 1.5 | how near the door you must stand |

**Rooms** (`RunContent::placeholder`): start (the safe first street), street, gap, step, escapes, drop, avenue (48 wide, about two
screens), two arenas (yard: three walkers and a flyer; roofs: two of each around a raised roof), shop, and a rest garden. **Wares:**
Heal 30% (30 coins, no limit), Extra life (150 coins, one per shop).

### Tests and numbers
- `sim-core/tests/runner.rs` (30 tests). New for R1b:
  - the window: it holds the current room, two behind and one ahead, joined, with the wall at the oldest room's edge, and nothing of a
    dropped room left; dropping trims the ground (same pool index) and removes the room's mobs; the window sizes are rules;
  - the wall: it is a solid stage block that stops a running fighter and can be wall-jumped off; nothing gets behind it (running,
    jumping, launched, or a mob), and the camera never shows past it;
  - the camera follows both ways inside the window; the optional creep pushes but never crushes;
  - an arena locks on entry (gate wall, all its mobs out and awake, the camera inside), stays shut until the last mob is beaten, then
    opens, pays and lets you through;
  - the shop: entering only at the door, the world frozen inside, refusal without coins, a heal and an extra life bought (coins
    deducted), the second life sold out (also after leaving and coming back), nothing paid for a heal not needed, and leaving;
  - a rest stop heals on entry;
  - the generator over 12 seeds of 80 rooms: the start room first and never again, no two arenas (or rest stops) in a row, every shop 5
    to 7 rooms after the last, every type appears, streets most and rests least, the same seed gives the same rooms;
  - determinism: same seed and inputs, a scripted shopping trip replayed checksum by checksum, restoring a state, random play with 16
    mobs (nobody behind the wall); the checksum sees every run, room and mob field (71 edits).
- `sim-content` (`run.rs`): the built-in run is valid; bad rooms and bad window, generator and shop rules are refused (the first room
  must be a street with no mobs; a shop room is required; shops and rest stops have no mobs; arenas have some; a shop's door stands on
  solid ground; no ledge on a room's edge).
- Godot: `tests/runner_test.gd` (the bridge's rooms, map and shop; entering a room; the camera coming back and never past the wall; the
  walls, map, door and shop in the scene and HUD; the shop panel; the results' rooms and arenas), and `tests/run_shot.gd` (now also
  photographs running back to the wall and the shop's panel).
- `cargo test --workspace` (1469 tests), clippy with `-D warnings`, `pftool net-fuzz` (200 runs, 0 mismatches), and the headless
  Godot tests (runner, match flow, input e2e, controls, replay, online flow, net e2e, creator flow) all pass. `pftool run-bench`: about
  0.002 ms a frame on average with 16 mobs, worst under 0.05 ms.

### Known limits (R1b)
- The shop is a menu over the frozen street, not an interior (see the choice above); an interior is an art question for R3.
- Only the player enters rooms; a mob left in a room as it is dropped is removed without paying.
- A mob launched hard to the right while the player is in an arena can be removed at the window's end without credit if it was not
  hit lately (rare).
- The left gate is drawn as a pillar at the very edge of the view (the camera stops at the wall), so only its face shows.
- R1's other limits (section 7) still hold.

### For the art director (added to section 7's list)
1. **The left gate:** a closed gate across the street and roofs (bars, a shutter, a fence with a padlock), about one unit thick, tall
   enough to fill the view's height; shown shut as the rooms behind close off.
2. **The arena gate:** a distinct exit gate (red, heavier) on an arena's far edge, with an opening animation (bars or shutter going up)
   and a clear "cleared" moment.
3. **The storefront:** a shop front at street level with a door the player stands at, a sign, lit windows, and an awning (the room's
   awning platform sits over the door); later, an interior or a shopkeeper for the shop's panel.
4. **Map icons:** street, arena, shop and rest stop icons for the HUD's map strip, plus a "closed" state for dropped rooms and a "you are
   here" mark; the strip itself in the HUD's painted style.
5. **A rest stop:** a rooftop garden or bench corner with a sign.

### Decisions wanted from the owner
- Does going back two rooms feel right, or should it be one or three (`rooms_behind`; three needs a bigger ring)?
- Shop prices (heal 30, extra life 150) and the arena bonus (25) are first guesses: worth a look after a few runs.
- Should the shop be its own small room you walk into (art and a camera cut) rather than the panel over the street? The panel is
  simpler and replays exactly; an interior can come with R3's shop art.

## 9. R2a progress (the world, sim 124, content schema 8, branch `runner-r2`)

Spec: `RUNNER_WORLD.md` sections 2, 3 and 6. Enemies and rare events (its sections 4 and 5) are R2b.

### What can be played
- **Slopes.** Ramps up to a bridge, stairs down to the street, tin roofs dipping into a valley, fire-escape flights. The fighter walks,
  dashes and runs along them at its ground speed with its feet on the surface, lands on them, jumps off them, wavedashes and techs
  along them; walking across the joint between a ramp and the roof beyond never trips it into the air. Ledges hang only at flat
  corners. A low hit into a rising slope can trip: the Gauntlets' 25-degree down tilt trips on the steepest slope, as the reference
  notes ("trips only on a slope"); the 30-degree Longsword down tilt still lifts.
- **Rooms that vary.** Every room can come mirrored (a rising room then falls), with its optional parts on or off (a second awning, a
  crate stack, a chimney, a balcony, a bus shelter, a raised planter, "the gap or no gap"), and an obstacle piece (or none) in each of
  its slots: a dumpster with a crate on its lid, a parked van, an AC unit with its vent stack, a water tower, a scaffold, a billboard's
  catwalk, a fire escape's two landings. Each room has an elevation line (flat, rising, falling, a valley), and the run's ground drifts
  up and down within 8 units of where it began.
- **The fork.** A wide street room with two roads: a coin trail climbs a fire escape from the street to a high road of catwalks and
  scaffolds (gaps, coins, one flyer); the low road is the street (more walkers). Falling from the high road lands on the street, never
  in a pit, and both roads meet at the room's end. One fork variant holds the shop on its low road. The map strip marks a fork with a
  split over its icon.
- **Coins along the way** (2 each), tracing the good line: up stairs, over gaps, on top of pieces. Picking one up does not count
  toward the combo.
- **Mobs come out of doors**: a mob appears in a dark doorway and grows out of it for 24 frames (it can neither hit nor be hit
  meanwhile), then wakes.
- **The walk-in shop.** Press up at a storefront's door and you are inside a small shop of its own: the door you came in by, four
  pedestals with the wares and their prices floating over them, shelves on the back wall, a counter, and the shopkeeper behind it. It
  waves when you come in, looks at the ware you stand by, cheers when you buy, grumbles cheerfully when you can't, and gets dizzy
  ("Hey!") if you hit it, never hurt. Walk up to a ware and press up to buy it ("UP: BUY HEAL 30% (30)" at the bottom of the screen);
  up at the door takes you back out to the street, at the storefront. The world outside stands still meanwhile. R1b's menu panel is
  gone. Wares and prices are as before (heal 30, extra life 150, one per shop).
- **The look**, rebuilt from the city stage's own art on the owner's direction: its blocks with their facade fronts and roof caps, its
  girder platforms, its striped awnings and shopfront, its crates, its car as the parked van, its street lamps, its rooftop AC unit and
  water tower recipes, building fronts behind each room painted by its facade shader, and its three skyline layers scrolling in
  parallax. Footing has a lighter top edge; decoration (lamp posts, billboards' boards, the water tower's legs and tank) has none.
- Debug for tests and photographs: `SimRunner.run_debug_next_room()` and `run_debug_enter_shop()`.

### How it is built (and why)
- **Slopes are a ground feature, not a run feature** (`sim_core::content::Platform::rise`, the top's height at the right end minus at
  the left). Every query reads the top where it is asked (`Platform::top_at`), which on a flat platform is its `y`, so **every flat
  stage behaves exactly as before** and every versus test passes unchanged. A grounded fighter's sideways move is followed by
  `fighter::keep_on_ground` (feet on the surface, horizontal speed kept, the reference's handling), and once more after the bodies
  push apart (`step`). Where a slope meets other ground at the same height, walking past the end carries on onto the next surface
  (`collision::continuation`) and the block's face there is no wall (`collision::face_is_joint`); both apply only where a slope is
  involved. Landing is judged from where the fighter was before its sideways move too, so drifting into a rising slope lands on it.
  The trip rule reads the slope (`combat::lifts_off`): a launch trips only if it does not rise above the ground the way it sends.
- **Schema 8**: a platform may carry `rise` (written only when not 0); the migration from 7 rewrites nothing (flat reads as slope 0).
  `sim_content` refuses slopes steeper than 1 in 2 and ledges at a slope's corner. `content/base.pfc` is re-exported (sim 124).
- **Variants are drawn as a room is laid** and kept in its `RoomSlot` (`mirror`, `parts`, `pieces`, and its ground's heights `y` and
  `y_end`), all from the run's own generator, so a seed is still a run. The layout functions (`run::for_each_footing`,
  `for_each_coin`, `for_each_ledge`, `door_of`) turn a slot back into its world geometry, coins and doors; the pools, the mobs, the
  bridge and the passability proof all read the same functions. A room's ground leaves at `exit_y` (mirrored, at minus that); a room
  that would take the ground beyond `drift` is mirrored the other way. Solid ground reaching a room's bottom is laid down to
  `run::DEEP` (-80), and the blast lines follow the ground (`Run::blast_y`: below the lowest ground of the rooms in play, above the
  highest).
- **The fork is one room with two roads** rather than R1b's sketched two-exit split: the high road is pass-through catwalks and
  stairs over the street, so the room window, the gate and the map need nothing new (a test enters an arena right after a fork). The
  two-exit version stays possible later if the roads should lead to different rooms.
- **Passability is proven, not hoped for** (`sim_content::run::check_passable`, part of `check_run`): every room in every variant
  (mirrored or not, every combination of parts, every piece or none in every slot: 4 to 128 variants a room) is a graph of standing
  surfaces (cut where a solid object stands on them, so a van on the street must be jumped), joined where a jump of at most
  `max_step` up and `max_gap` across (more when falling) has a clear path with a fighter's height of room. The exit must be reached
  from the entry, and the exit must be reachable from every surface the entry reaches (no traps). A test checks the rules' numbers
  against the worst jumper in the cast, every built-in fighter and every class made with the lowest jump, biggest body and most
  weight: the worst reaches 4.98 up and 5.30 across; the rules ask 4.0 and 4.5. Another test proves every room the generator really
  laid over 40 seeds.
- **The shop's interior is its own little stage** (`RunContent::interior`), placed at `INTERIOR_X` (-20000) while you are inside: the
  run swaps `build_stage` to it and runs only the fighter's step and the shop (mobs, camera, rooms and coins outside stand still). The
  shopkeeper is state in the run (`keeper_mood`, `keeper_timer`, `keeper_look`, `keeper_bonks`), all hashed. A hit on it is detected
  like a hit on a mob (the player's live hitboxes or a shot touching its body) and only sets it dizzy: no hitlag, no damage, nothing
  paid.
- **Mob emergence** is a mode (`MODE_EMERGE`, `emerge_frames`); in an arena that is shut on it, a mob wakes straight into its chase.
- **Pools**: a room may have 14 platforms with its pieces (was 6), so the pool is 56; ledges stay 2 a room (the versus ledge table is
  8). **GameState is 5,536 bytes** (was about 4.4 KB); the run's part is 2,768. `pftool run-bench`: 0.0024 ms a frame on average with
  16 mobs, worst 0.07 ms.

### Tunables added (`RunRules::standard`)
| Rule | Value | Meaning |
| --- | --- | --- |
| `fork_chance` | 25 | percent of streets that are a fork |
| `piece_chance` | 70 | percent of slots that get a piece |
| `drift` | 8 | how far the ground may drift up or down from where the run began |
| `coin_value` / `coin_reach` | 2 / 0.9 | a coin's worth, and how near the body's middle picks it up |
| `emerge_frames` | 24 | frames a mob takes to come out of its door (0: it pops in) |
| `max_step` / `max_gap` | 4.0 / 4.5 | the passability guarantee (see above) |

The shop interior's own numbers (`Interior`): 24 wide; the shopkeeper waves 90 frames, cheers 70, grumbles 60, stays dizzy 100.

**Rooms** (`RunContent::placeholder`): start, street, gap, step, escapes, drop, avenue, ramp (rising), tin_roofs (valley), stairs
(falling), fork, the arenas yard and roofs, shop, fork_shop (the shop on a fork's low road), garden. **Pieces:** dumpster, ac_vent,
water_tower, billboard, van, scaffold, fire_escape.

### Tests and numbers
- `sim-core/tests/slopes.rs` (10): reading a slope; running up a ramp onto the roof and back down, grounded and on the surface every
  frame, the ground speed horizontal; walking and dashing on it; landing on a ramp, drifting into one, landing on a flight and dropping
  through it; jumping off; wavedashing along it both ways; teching and lying on it; trips on a slope (and not on flat ground, nor at 30
  degrees); the joint is not a wall; flat platforms never join.
- `sim-core/tests/runner.rs` (41, 11 new): the walk-in shop (going in, the world outside standing still), buying by walking up (the
  shopkeeper's looks, cheers and grumbles; sold out remembered; nothing paid for a heal not needed; leaving by the door), hitting the
  shopkeeper (dizzy, nothing harmful); rooms laid as variants over 24 seeds (mirroring, parts, every piece, joined heights, the drift,
  the pools); a mirrored room is its reflection; parts and pieces lay what they say; the fork's two roads; a fork with the window and
  the gate; coins picked up once; mobs coming out of doors; walkers on slopes; GameState's size; the checksum sees every new field; the
  run content's hash sees pieces, parts, exits and the interior.
- `sim-content`: the passability proof and its refusals (an unclimbable wall, a too-wide gap, a trap), slopes, looks, parts and pieces
  checked, every variant enumerated, the worst jumper in the cast; `tests/bundle.rs`: a schema 7 file loads flat, a slope round-trips
  and is checked; `tests/run_world.rs`: rooms laid over 40 seeds are passable.
- `cargo test --workspace` (1,552 tests), clippy with `-D warnings`, `cargo fmt`, `pftool net-fuzz` (200 runs, 0 mismatches). Godot,
  headless: runner_test (now also the looks, the coins, the interior, walking into the shop and out, the shopkeeper's wave, the
  parallax layers), match_flow_test, input_e2e, controls_test, replay_test, online_flow_test, net_e2e, creator_flow_test, content_e2e,
  editor_api_test: all pass. `tests/run_shot.gd` photographs a slope, a fork, the shop's interior and a ware.
- The bot (`pftool run-bot 8 7200`): rooms 2 to 20 in two minutes, none lost all its lives. (It only runs, hops and jabs; one seed got
  stuck early, likely at a climb it does not know to jump.)

### Known limits (R2a)
- Mob behaviour on slopes is simple: walkers follow a slope and look a little ahead for ground; flyers ignore the ground as before.
- Items lying on a slope slide along it but do not roll down it; a projectile that rests on a slope rests at the surface.
- A shot or a thrown item in flight when you walk into the shop flies on in the frozen street and leaves the blast zone (it is lost).
- Only flat ground joins across room seams into one platform; a slope at a seam joins through `continuation` instead (the same feel).
- The shop's four pedestals show two wares; R3 fills them.
- Pigeons that scatter, swaying laundry and a cat on a ledge (the cheap life of `RUNNER_WORLD.md` section 3) are not built: the lit
  windows of the building fronts and the skyline are. They are on the art list.
- R1's and R1b's other limits still hold.

### For the art director (R2a; **reused** = built from the city stage's art now, **new** = needs a design)
1. **The shopkeeper** (new): a cute original character in the house style (round blob body, 2D face, an apron or a cap), with a wave,
   a look toward a ware, a cheer, a cheerful grumble and a dizzy spin with stars and a "hey!". Placeholder: a peach blob with an
   apron and two eyes.
2. **The shop interior** (new): about one screen; the door (with an EXIT mark), four pedestals for the wares with the price over each,
   a counter, shelves of goods on the back wall, a window, warm light. Placeholder: plain boxes and girders in the city palette.
3. **Ware icons** (new): heal and extra life (R3 adds more). Placeholder: a green sphere with a cross, a red sphere.
4. **Footing by look tag** (the sim's names in brackets):
   - reused as they are: street and roof blocks (`street`, `roof`: the city block with its facade front and cap), the girder platform
     for `balcony`, `fire_escape`, `scaffold`, `billboard` (the catwalk), `water_tower` (the deck), `bus_shelter`, `pedestal`,
     `shelf`; the striped `awning` (the city shops' stripes); `crate` (the city crate); `van` (the city car, stretched); `ac_unit` and
     `vent_pipe` (the city rooftop's recipe); the water tower's tank and legs (the city's recipe); `planter` (the city's potted plant);
   - new: `dumpster` (a green box with a lid), `chimney`, `roof_shed`, `parapet`, `counter` (boxes in the city palette);
   - each needs the light top edge on what holds you, and none on what doesn't.
5. **Slopes** (new shapes, reused materials): `ramp` (a loading ramp), `tin_roof` (corrugated), `stairs` (solid steps) are drawn now as
   prisms in the city block's material with treads; fire-escape flights in the girders' steel with treads and a railing.
6. **The fork's two roads** (reused): the high road is the girder catwalks, scaffolds and fire-escape flights over the street; a sign
   or a lit fire escape that says "up here" would help the telegraph beyond the coin trail.
7. **Background** (reused): the city stage's three skyline layers, now in parallax, and its facade shader for a building front behind
   each room. New if wanted: a different front per building (brick, concrete, glass) beyond the shade changes they have now.
8. **Life** (new, cheap): pigeons that scatter, laundry on lines, a cat on a ledge; a doorway or stairwell for mobs to come out of
   (placeholder: a dark opening).
9. **Coins** (new): a coin with a spin and a pickup sparkle (placeholder: a gold disc).
10. **The storefront** (reused): the city shopfront with a SHOP sign.

### Decisions wanted from the owner
- **Coins along the way** pay 2 each (kills pay 5 to 8 times the combo). Enough of a reward for the high road?
- **The fork** is one room with two roads that meet at its end. Should a fork sometimes lead to different rooms (the two-exit version
  in 3.1), say a treasure alley only from the high road?
- **Slopes** top out at about 26.6 degrees (1 in 2). The steepest ones are where the Gauntlets' down tilt trips; gentler rooms can come
  if the steep ones feel slippery.
- **Hitting the shopkeeper** only makes it dizzy. Fine as a joke, or should it also refuse to sell for a moment?

## 10. R2b progress (enemies and the curve, sim 127, content schema 8, branch `runner-r2b`)

Spec: `RUNNER_WORLD.md` section 4 (rewritten, **Decided**) and section 5, and the owner's answers to R2a (coins worth 1, forks leading
to different rooms, slopes as they are, the shopkeeper refusing to sell after a bonk).

### What can be played
- **Fighter-shaped grunts.** Every enemy is a base-game body in a grunt outfit with a red marker over its head, as big to hit as a
  fighter, fighting with its body's real moves. Before each attack a big "!" bobs over it (18 frames at first) and it makes the move's
  own wind-up. Six kinds:
  - **brawler** (the sprinter body, orange with a beanie): rushes you, runs when far, a jab up close or a forward tilt;
  - **leaper** (the rogue, plum with a hood and goggles): waits on high ground, crouches, leaps at you, and its down air (from above)
    or forward air drifts onto you;
  - **shieldbearer** (the wanderer, slate with a plumed helm): walks in behind a versus shield bubble (kept up through the tell of its
    poke too) and pokes out with its forward tilt or a jab. Hits from the front are blocked (its guard wears down and you freeze on
    it); a grab throws it, guard and all; a hit from above lands; crossing it up lands until it turns round (18 frames); a guard worn
    to nothing leaves it dazed (the broken-shield daze);
  - **spearman** (the duelist, cobalt with a cape): a late, long forward tilt, or a sliding dash attack from 6 away: jump it, punish
    it;
  - **lobber** (the bounder, green with a straw hat and round specs): stays on high ground and lobs its shell (the clutch's lobbed
    shell, a real projectile) in an arc that comes down on you. A reflector sends it back into the grunts (the tumbler's cape does);
  - **bruiser** (the hauler, midnight with shades): walks in slowly and swings its forward smash, with super armor through the tell
    and the smash's startup (it takes the damage, does not flinch), then a big launch.
- **Rooms fill up by budget.** Each room's spots are filled as it is laid, by its tier's budget and the placement rules (lobbers and
  leapers on high ground, bruisers on flat room, never two shieldbearers on one platform). Grunts still come out of doors.
- **The difficulty curve.** A new tier every 6 rooms (5 tiers): more and harder kinds, bigger budgets, shorter tells (18 to 10
  frames), tougher grunts (stamina 100% to 150%, damage 100% to 130%). Rooms come from their tier's pool (the drop, the avenue and the
  new plaza arena from tier 2). The map strip shows "TIER n" with pips; "TIER n" flashes as it steps up.
- **Hazards and a chest.** Gas canisters (red, yellow band) burn for a moment when you hit them, then burst, hurting grunts (your
  kills) and you. Loose signs hang over the street; hit one and it falls on whoever is under it and breaks. The treasure alley has a
  chest that pays 30 when hit open.
- **The runaway van** (the first rare event): about once a city (24 rooms), never in the first 6, a "?" on the map ("RUNAWAY VAN?" as
  you enter). The van waits at the street's start; land on its roof ("HOLD ON!") and it speeds off down the street carrying you,
  untouchable, sending every grunt in its way flying (your kills, your combo), until it parks at the street's end.
- **Forks that branch** (owner's R2a answer): the new branching fork climbs to a catwalk that leaves its right edge 8 up into a
  **treasure alley** (catwalks with coins, a gas canister and the chest, then a fire escape down to the street); the street goes on
  into a street room of its own. Walk out on one road and that room is yours; the other is gone at once. The map draws the alley over
  its sibling, the branch line climbing to it, and the road not taken crossed out.
- **The shopkeeper sulks**: hit it and it is dizzy as before, then crosses its arms ("Hmph!") and refuses to sell for 3 seconds
  ("HMPH! IT WON'T SELL TO YOU").
- **Coins are worth 1.**
- **Fixed** (owner's R2a bugs): coins you pick up vanish at once; light and shadows no longer travel with the backdrop (see below).
- Debug for tests and photographs: `SimRunner.run_debug_grunt(kind, dx, telling)` and `run_debug_force_event()`.

### How it is built (and why)
- **A grunt is a light entity drawn as a fighter.** `run::Mob` stays a fixed pool of 16 (the four fighter slots stay free; a full CPU
  fighter per grunt would cost a fighter slot each and the whole fighter state machine, which a run needs 10 to 16 of). A grunt
  kind (`MobKind`) names a **body** (a roster fighter) and a kit of that body's **moves** (`MoveId`s). The grunt is read as a fighter
  of its body whenever it matters (`mobs::as_fighter`): its hurtbox is `combat::hurtboxes` of that fighter, its attacks are
  `combat::active_hitboxes` of the move at its frame, and the game draws it from the same fighter snapshot
  (`run::grunt_fighter`; the bridge answers fighter `100 + n` with grunt `n`, so `runner.gd` builds its snapshot with the same calls
  `main.gd` uses and `fighter_view.gd` draws it unchanged). Weight, fall speed, gravity, walk and run speed, air speed and jump are its
  body's numbers (`FighterParams`). Behaviour is a small state machine per role (`run/mobs.rs`): idle, move or guard, tell
  (`MODE_WINDUP`), the move (`MODE_ATTACK`, its own frames), leap, cooldown, dazed.
- **Hits both ways are the versus ones.** The player's hits on a grunt use the versus formulas as before, now against its body's
  hurtbox; a grab (`HIT_GRAB`) throws it with the fighter's own forward-throw hit. A grunt's hit on the player goes through
  `combat::apply_hit` from an unplayed fighter slot standing for its body (shields, DI, crouch cancelling, armor and counters all
  work), once per hit group, at its tier's damage.
- **The guard**: a versus shield bubble (`Mob::shield`, regrowing slowly). A hit is blocked only from in front and not from above
  (`mobs::guard_blocks`); a blocked hit wears it, pushes the grunt back and freezes the attacker on the shield's hitlag.
- **Armor**: the bruiser's super armor through its tell and its smash's startup (`mobs::armored`): the hit's damage and hitlag, no
  launch. (The new `Move::armor_mode` from the Exile's port is for fighters' own move armor; a grunt's armor belongs to its kind, not
  to the borrowed move, so it stays a grunt rule.)
- **The lobber's shot is a real projectile.** It is the clutch's lobbed-shell spawn, put in the versus projectile pool with its owner
  and origin set to a fighter slot no one plays in a run (`run::SHOT_SLOT`, whose `char_id` is the lobbers' body for the whole run), so
  the versus code flies it, bursts it and hits the player with it, and a reflector turns it over to the player like any shot; the run's
  `mobs::shots` then lets it hit grunts. Its velocity is solved to come down on the player after `shot_time` frames.
- **Budget and placement** (`run::assign_kinds`) run when a room is laid, from the run's own generator, and the chosen kind per spot
  is kept in the room's slot (`RoomSlot::kinds`, hashed), so a seed is still a run.
- **Props** (`run/props.rs`) are a pool of 12 in the run state, laid with their room (`Room::props`) and dropped with it. The
  player's attack or shot sets one off (`mobs::player_touches`, shared with the shopkeeper's bonk); a hazard's hit lands once on every
  grunt it touches (the player's kills) and once on the player (the versus hit).
- **Events**: room type 4 (`ROOM_EVENT`, a "?" on the map) is drawn by `event_due` before the ordinary room odds: at most one per
  "city" (`city_rooms`), never before `event_from`, at `event_base` per mille plus `event_step` per mille for every eligible room
  without one (`Run::event_pity`). The van (`Run::van`) is a pass-through pool platform (its roof) that the sim moves; a fighter
  standing on it is carried and kept intangible while it drives (the view does not ghost it); grunts it touches take `van_hit` once.
- **Branching forks**: a fork names its two roads (`Room::branch_high`, `branch_low`, `lift`). When the fork becomes the current room,
  one room fewer is kept behind and both roads are laid at its end, the alley `lift` higher; nothing is laid beyond them. Walking out
  past the end, the player's height decides the road (`enter_rooms`); the other is dropped at once with everything it laid in the pools
  (pool entries now remember their room, `PoolPlatform::seq`, `PoolLedge::seq`), its grunts and props, and marked on the map
  (`Run::skipped_log`). The alley and its sibling are as wide and share the street under them, so taking either leaves solid ground; the
  alley has no other solid block (it lies over the other road until one is taken; `sim_content` checks both rules, and the
  passability proof enters an alley on its catwalk). Rooms behind are now kept by count, not by number (a road not taken leaves a gap).
- **The shopkeeper's sulk** is `Run::keeper_sulk` (`Interior::sulk_frames`, 180): while it lasts `buy` refuses (`NOTE_REFUSED`) and
  the mood is `KEEPER_HMPH`.
- **The two R2a bugs.** *Coins:* `run_stage.gd` redrew the coins only when the stage's layout changed, and picking a coin up changes
  no layout, so a collected coin stayed until the next room came; the coins are now synced every frame (a runner_test check). *Light
  and shadows:* the run reuses the city stage's backdrop, which travels with the camera (unlike a versus stage's). It carries the
  hidden city stage's four practical lights (its string lights' warm pools and the mast's red beacon) and 274 shadow-casting props, so
  warm pools slid across the run's buildings and props' shadows crawled over the rooms as the camera moved. `run_stage.gd` now turns
  the backdrop's lights off and its shadow casting off (a runner_test check); its emissive bulbs and lit windows keep its look.
  `tests/run_flicker_probe.gd` measures frame-to-frame changes while running, for the art director to compare against.
- **GameState is 6,352 bytes** (R2a: 5,536; the run's part 3,568, was 2,768; the rest grew 16 bytes with the moveset ports merged).
  `pftool run-bench`: 0.0031 ms a frame on average with all 16 slots full of the six kinds, worst 0.19 ms.

### Tunables added or changed (`RunRules::standard`, `MobKind::standard`, `Tier::standard`, `PropKind::standard`)
| Rule | Value | Meaning |
| --- | --- | --- |
| `coin_value` | 1 | was 2 (owner) |
| `tier_rooms` | 6 | a new tier every 6 rooms |
| `high_ground` / `flat_room` | 2 / 6 | placement: high ground is 2 above the room's lowest solid ground; flat room is a flat platform 6 wide |
| `arena_budget` | 150 | an arena's budget, percent of a street's of its width |
| `city_rooms` / `event_from` | 24 / 6 | at most one event in 24 rooms, none before room 6 |
| `event_base` / `event_step` | 20 / 12 | per mille a room, plus the pity per room without one |
| `van_speed` / `van_accel` / `van_frames` | 0.4 / 1/80 / 240 | the van's top speed, its pick-up, its longest drive |
| `van_hit` | 10%, 50 degrees, 80 / 70 | what the van does to a grunt |
| `guard_turn` / `daze_frames` / `shield_regen` | 18 / 100 / 0.05 | a shieldbearer turns 18 frames after a cross-up; a broken guard dazes 100 frames; the guard regrows 3 a second |
| `grab_throw` | 8%, 45 degrees | a grab on a grunt when the fighter has no forward throw hit |
| `Interior::sulk_frames` | 180 | the shopkeeper refuses to sell for 3 seconds after a bonk |

| Tier | Kinds | Budget (per 16 units) | Tell | Stamina | Damage |
| --- | --- | --- | --- | --- | --- |
| 1 (rooms 0 to 5) | brawler, leaper | 3 | 18 | 100% | 100% |
| 2 (6 to 11) | + spearman, shieldbearer | 4 | 16 | 110% | 105% |
| 3 (12 to 17) | + lobber | 5 | 14 | 120% | 110% |
| 4 (18 to 23) | + bruiser | 6 | 12 | 135% | 120% |
| 5 (24 on) | all | 7 | 10 | 150% | 130% |

| Kind | Body | Moves | Stamina | Cost | From tier | Coins | Reach | Placement |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Brawler | sprinter | jab, forward tilt | 14 | 2 | 1 | 5 | 1.6 / 2.6 | any |
| Leaper | rogue | down air, forward air | 10 | 3 | 1 | 7 | leaps from 9 | high ground |
| Shieldbearer | wanderer | forward tilt, jab | 16 (guard 30) | 3 | 2 | 8 | 2.8 / 1.6 | alone on its platform |
| Spearman | duelist | forward tilt, dash attack | 14 | 3 | 2 | 7 | 3.0 / 6 | any |
| Lobber | bounder | lobbed shell, jab | 10 | 4 | 3 | 8 | throws within 15 | high ground |
| Bruiser | hauler | forward smash (armored), jab | 40 | 5 | 4 | 14 | 3.0 | flat room |

Props: the gas canister (burns 14 frames, bursts for 4: 14%, 60 degrees, 50 / 80, radius 2.6), the loose sign (falls; 10%, 70
degrees), the chest (pays 30).

**Rooms** (new in R2b): plaza (an arena with two catwalks, from tier 2), fork_up (the branching fork), alley (the treasure alley) and
under_alley (its low road), van_street (the event). Every room's spawn markers are now spots the budget fills, more of them, and some
rooms have props.

### Tests and numbers
- `sim-core/tests/runner.rs` (56; 15 new for R2b): a grunt has a fighter's hurtbox (its body's, fighter-tall); each ground kind tells
  for 18 frames then strikes with its body's move and hits; tiers shorten tells and toughen grunts; the tier steps up and rooms come
  from their tier's pool (grunts get their room's tier and stamina); the shieldbearer blocks from the front only (cross-up, from
  above, grab, guard break); the bruiser's armor through its startup only; the lobber's arc onto the player; a reflected shot (the
  tumbler's cape) hits the grunts; placement rules and budgets over 30 seeds of 30 rooms (every kind comes); hazards hurt grunts (a gas
  canister's burst is the player's kill and catches the player; a falling sign; a chest pays once); rare events (never before room 6,
  at most one a city, at least 70% of cities get theirs over 24 seeds of 96 rooms; the pity counter brings it on the second eligible
  room with no base chance; with neither, never); riding the runaway van end to end; a branching fork's two rooms (both ways: the
  other dropped with everything it laid, marked on the map, the run going on from the room taken); a coin is worth 1; a run with
  grunts, props, events and forks replays checksum by checksum; the checksum sees every new field (32 new edits); random play with all
  six kinds keeps the invariants. The R2a tests were adjusted: the window (two roads ahead at a branching fork), the generator (the
  paired roads are not the generator's), the variants (an alley joins its fork's high road), the shopkeeper (dizzy, then sulking and
  refusing, then selling again).
- `sim-content`: grunt kinds (bodies and moves in the roster, a lobber's first move fires a shot, one lobber body), tiers, props, event
  rooms, alleys (paired, no solid block but the street) and branching forks (their roads, widths and lift) are checked; the
  passability proof enters an alley on its catwalk; `tests/run_world.rs` walks both roads of every branching fork over 40 seeds.
- `cargo test --workspace` (all pass), clippy with `-D warnings`, `cargo fmt`. `pftool net-fuzz 200`: 0 mismatches, and the same line
  as on `main` before R2b (8,375 rollbacks, 28,253 stalls).
- Godot, headless: runner_test (now also the grunt kinds through the bridge, a grunt drawn with the fighter rig in its outfit and
  telling, a picked-up coin vanishing, no light or shadow travelling with the backdrop, the props and van data), match_flow_test,
  input_e2e, controls_test, replay_test, online_flow_test, net_e2e, creator_flow_test, content_e2e, editor_api_test: all pass.
  `tests/run_shot.gd` now photographs each grunt kind (the brawler telling), three together, and the van waiting and ridden.
- The bot (`pftool run-bot 8 7200`): rooms 1 to 12 in two minutes; the grunts now stop it more often (it only runs, hops and jabs).

### Known limits (R2b)
- A grab on a grunt throws it at once with the fighter's forward-throw hit; the fighter's own grab animation plays out as a whiff (no
  hold, pummel or choice of throw).
- Grunt moves are their hitboxes and frames; a move's own motion and script (a dash attack's slide, a gauntlets jab's rapid entry, the
  lobbed shell's aiming script) are not run: the grunt kind gives the slide (`lunge`) and the shot's arc.
- The leaper does not grab ledges or recover; a grunt knocked off high ground falls.
- Grunts' borrowed moves do not stale and their hits count as fresh (the 1.05 bonus), as R1's mobs did.
- The van is drawn as the city car stretched (the parked van piece's look); it only drives in its own street and parks at its end.
- The alley and its sibling overlap until one is taken, so for a moment both are drawn (the alley's catwalks over the street). On the
  camera's side this reads as a high road over the street; the other road vanishes when you commit.
- Directional-shadow shimmer while the camera pans is not measured as solved: see the art list (it is the lighting's, not the run's).

### For the art director (R2b; **reused** = built now from existing art, **new** = needs a design)
1. **Grunt outfits** (reused, placeholder): the six roles in existing catalog items (`godot/scripts/grunt_view.gd` `OUTFITS`): brawler
   tangerine with a beanie, neckerchief, striped shirt and gloves; leaper plum with a hood, goggles, scarf; shieldbearer slate with a
   plumed helm, sash, full kit; spearman cobalt with a sailor cap and cape; lobber meadow with a straw hat, round specs, flower shirt;
   bruiser midnight with swept spikes and shades. **New if wanted:** a grunt "uniform" (a team armband, a hoodie or a cap with a
   crest) that says "enemy" more strongly than colours, still in the house style.
2. **The enemy marker** (reused): the match tag's pointer in red. **New if wanted:** red eyes or an angry brow for grunts instead.
3. **The tell** (placeholder): a big yellow "!" that flashes red as the move nears, and a short blip. **New:** a proper tell graphic
   and a sound per kind.
4. **The doorway** (placeholder, R2a): a dark rectangle the grunt grows out of. **New:** a stairwell door, a roof hatch.
5. **Hazards** (new): a gas canister (placeholder: a red cylinder with a yellow band and a valve; burning: it shakes; burst: an orange
   ball), a loose sign on two wires (placeholder: a yellow board with a dark stripe; it swings as it falls and lies tilted). Both need a
   burst or break effect and a sound.
6. **The chest** (new): placeholder a wooden box with a gold lid that swings open; needs an open effect with coins.
7. **The runaway van** (reused): the city car stretched into a van, bouncing as it drives. **New if wanted:** a real delivery van with
   a ladder up the back, exhaust, and a "runaway" look (an open door flapping, a driver ducking).
8. **Map icons** (placeholder, new): the event's gold "?", the alley's little chest, the branch line climbing to an alley, the road not
   taken crossed out, "TIER n" with pips.
9. **The treasure alley** (reused): catwalks are the girder platforms (look `catwalk`), the stairs the fire-escape flights. **New if
   wanted:** clothes lines, rooftop gardens, a "this way" sign at the fork's high exit.
10. **The sulking shopkeeper** (placeholder): the blob leans away with "Hmph!". **New:** arms crossed, a pout.
11. **Lighting** (for the art director's `lighting.gd`): the run's camera pans all the time, unlike versus. If the owner still sees
    shadow edges crawl or shimmer after this fix, the key light's directional shadow is the next suspect: it is refitted to the moving
    view every frame (`directional_shadow_max_distance` 70, two splits). Worth trying for the run only: a shorter shadow distance (the
    run's camera is about 32 from the stage), `SHADOW_ORTHOGONAL` (one map, no split seam to slide), or a higher shadow atlas size.
    `tests/run_flicker_probe.gd` gives numbers to compare. I did not change `lighting.gd`.
12. **`fighter_view.gd`** (for the art director): `rebuild()` frees its children but keeps the dust puffs in `dusts`, so a rebuilt view
    errors on its next dust ("previously freed"). Versus rebuilds rarely; the run's grunt views now avoid rebuilding (one view per role),
    but the fix is to clear `dusts`, `dust_age` and `dust_vel` in `rebuild()`.

### Decisions wanted from the owner
- **Difficulty:** tier 1 gives two or three grunts in a one-screen room (budget 3 points per 16 units, a brawler costs 2); the curve
  reaches all six kinds by room 18 and the hardest tier at room 24. Too fast, too slow?
- **The shieldbearer's guard** stays up through the tell of its poke (only the poke and its cooldown are open). Is that fair, or should
  the tell drop the guard too?
- **Grabs on grunts** throw at once (no hold, no pummel). Enough, or should a grab hold a grunt for a throw of your choice (a bigger
  piece of work)?
- **The van** comes about once a city and lasts about 3 seconds of ride. Longer rides (into the next room) would need it to drive
  through the next room's pieces; worth it?
- **Hazards hurt you too.** The gas canister's burst catches you if you stand next to it when it goes off. Keep, or make hazards hurt
  only grunts?
- **The sulk** lasts 3 seconds (a timer, not until you leave). Fine?

Sources: [TouchArcade review](https://toucharcade.com/2012/10/26/punch-quest-review/),
[AppSpy review](https://www.appspy.com/punch-quest-review), [Destructoid review](https://www.destructoid.com/reviews/review-punch-quest/),
[App Store listing](https://apps.apple.com/bn/app/id554223561).
