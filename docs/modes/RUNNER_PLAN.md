# Single-player mode: the endless run (plan, draft 1)

Status: **plan for discussion, nothing built.** Owner brief, 14 October 2026: a single-player endless-runner mode inspired by the mobile
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

### 3.1 Movement: forced scroll, not auto-run (recommended)
- **Recommended: a pushing screen.** The camera scrolls right on its own. The player moves freely inside the view, with dash, jumps
  and the full kit. Falling behind the left edge pushes you, then costs a life if you're pinned against a wall. This keeps the
  fighting deep and still forces forward momentum, like a forced-scroll platformer level.
- Alternative A: **true auto-run.** The fighter runs right by itself and the stick only aims attacks. This is closest to Punch Quest
  and best for a later mobile or casual version, but it loses dash dancing, spacing and most of the kit.
- Alternative B: **free movement, score by distance.** No scroll; stalling is allowed. It's simplest, but it isn't a runner.

### 3.2 Health: damage percent and lives (recommended)
- **Recommended: our own rules.** Damage percent builds up, enemies launch you further the higher it is, and you lose a life when
  launched off screen, into a pit, or crushed against the left edge. Start with 3 lives. Heal items lower your percent, and some
  shops sell an extra life.
- This keeps the game's identity: a run gets tense as your percent climbs, and recovering back to the stage is part of the skill.
- Alternative: an HP bar, as in Punch Quest. It's clearer for new players, but it isn't how this game works.

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

Everything bought is lost at the end of the run, the classic roguelite rule. A light **meta layer** (permanent unlocks: new starting
fighters, new item pools, cosmetic hats) can follow. Punch Quest leaned on long unlock grinds, which reviewers found slow, so keep
the meta layer generous.

### 3.6 Archetypes (proposal for the owner to adjust)

| Archetype | Fighters (class) | Why together |
| --- | --- | --- |
| Blades | Longsword, Greatsword, Buckler | sword in hand, similar reach and swing shapes |
| Fists | Embers, Gauntlets, Bracers, Claws | punches and kicks, close range |
| Agile | Needles, Scamp | light, fast, tricky movement |
| Odd ones | Staff, Clutch, Settler (Pockets) | unique tools: tongue, pocket, staff magic (the owner's example: the Villager-style and Yoshi-style fighters together) |

A move swap only goes between fighters of one archetype. Created fighters use their class's archetype.

### 3.7 Setting: city rooftops (recommended) and regions later
- **City rooftops** answer the verticality worry well:
  - buildings of different heights give natural ups and downs, and gaps between them are pits;
  - fire escapes, billboards, water towers and awnings are platforms;
  - **two routes**: the rooftops (risky: pits and throwers, more coins) or the street (safe: more enemies, fewer coins). This is
    Punch Quest's branching paths with a reason to choose.
- The camera keeps a wide horizontal view. A chunk's height stays within about one and a half screens, and the camera follows
  height smoothly with a dead zone, so the action never leaves the frame.
- The existing city stage gives the art baseline: palette, building style, lighting. The art director would build a **kit**
  (rooftop modules, gaps, fire escapes, signs and a storefront for the shop) rather than one long level.
- **Regions** come later, as distance markers: downtown, the docks or harbour, then the industrial edge. One region could pass
  through our other stage themes (the grove or the canyon) for a change of scene. Each region has its own palette, enemy mix and
  music, from mostly reused art.

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
| R1 | Vertical slice: forced scroll, 5 hand-made chunks on a loop, 2 enemy kinds, coins and the combo meter, lives, the run HUD, a results screen | a 2-minute run is fun to play with placeholder art |
| R2 | The generator: chunk pools by difficulty, the two routes, the difficulty curve, 6 enemy kinds | runs differ and get harder smoothly |
| R3 | Shops and items: stat boosts, 10 properties, consumables, rerolls | a run's build changes how it plays |
| R4 | Bosses: generated CPU fighters with stamina and modifiers, an arena chunk | a boss every few shops, beatable in 2 to 3 tries at worst |
| R5 | Archetype move swaps | a sword fighter can buy another sword fighter's moves |
| R6 | City kit art, a second region, music, polish | art director sign-off |
| R7 | Meta layer, daily seed, then online leaderboards (re-run to verify) | later, with the online work |

## 6. Questions for the owner

1. Movement: the forced scroll (recommended), true auto-run, or free movement?
2. Health: percent and lives (recommended), or an HP bar?
3. Do upgrades reset every run (recommended, with a light permanent unlock layer), or persist as in Punch Quest?
4. Archetypes: does the table in 3.6 fit, or should some fighters move?
5. Setting: city rooftops with a street route (recommended), and regions later?
6. Can created fighters play the mode from the start, or only built-in ones?
7. Who builds it: a dedicated "modes and online" agent (`.claude/agents/modes-online.md`), with me directing (recommended), or me
   directly?

Sources: [TouchArcade review](https://toucharcade.com/2012/10/26/punch-quest-review/),
[AppSpy review](https://www.appspy.com/punch-quest-review), [Destructoid review](https://www.destructoid.com/reviews/review-punch-quest/),
[App Store listing](https://apps.apple.com/bn/app/id554223561).
