# Run mode: building a believable, generated world

Research and direction for the run's level design, enemies and world (14 October 2026, executive producer). It builds on
`RUNNER_PLAN.md` (R1 and R1b are done: rooms, two rooms of backtracking, arenas, shops, rest stops, the map strip).

The owner asked for four things:
- level geometry with variety (slopes, alternating pathways), kept simple but entertaining;
- a believable side-scrolling world, where an object that looks like it can be stood on works as a platform;
- the shop as a cute room you walk into, with a shopkeeper: interactable, not a big gameplay feature;
- research from Punch Quest and similar games.

## 1. What the research says

### Punch Quest (RocketCat, 2012), the main inspiration
- **Layered like a classic platformer.** A critic described its levels as built in layers: hazards, platforms, treasure, branching
  paths, power-ups, simple enemies that combine well, simple bosses and bonus rooms. No single layer is complex; the depth comes
  from how they stack.
- **Branching paths are the reward system.** The run is a dungeon full of forks. Picking the right branch is where most power-ups
  come from. A fan wiki lists seven path types, for example:
  - a dangerous path with falling rocks and exploding zombies that usually pays out a power-up or food;
  - a fork that leads to more forks;
  - a treasure path whose sub-paths are full of traps;
  - a rare egg path.
  Treasure goes with harder traps. **Risk is chosen, and it is shown before you choose.**
- **Rare events make stories.** Breaking an egg lets you ride a laser-firing raptor, invulnerable, through its own volcanic scene.
  The game weights the egg path to come sooner if you haven't seen one in a while: it is rare, but never too rare. There are also
  very rare scenes (a night with UFOs firing lasers, an eclipse). Players remember these and tell others about them.
- **Enemies teach one rule each, then combine:**
  - orcs carry shields you uppercut away or slam from above;
  - skeletons thrust spears you jump over and punch under;
  - wraiths are tricky;
  - bees and jellyfish sting and zap;
  - torches explode when hit, and can be used against tough enemies.
  Each has a distinct sound that announces its attack.
- **Speed is the score.** You can play carefully, but the combo multiplier rewards punching forward fast. Being hit, even while
  blocking, breaks the combo.
- **The developer's own lesson** was that progression was too generous ("an upgrade or new ability or power every single match").
  The time between upgrades matters more than the total length of progression.

### Spelunky: rooms from templates with dice inside them
- The level is a grid of rooms. A guaranteed path from entrance to exit is laid first, then each room picks a **template** that
  fits its exits (8 to 16 per room type).
- Inside a template, some tiles are **probabilistic** (a 50% block, a 33% spike), and some areas are **obstacle blocks**:
  small hand-made pieces, swapped at random, that sit on the ground or float.
- Rooms are randomly **mirrored**.
- The result is "half generated, half designed". A small number of hand-made pieces gives a very large number of different-feeling
  rooms, and every room is still guaranteed to be passable.

### Dead Cells: a hybrid with constraints
- Rooms are hand-made, each with a purpose (combat, treasure, merchant) and a number of exits, and **belong to one biome**. A sewer
  room never appears in the prison.
- A per-biome **graph** fixes length, branching, and where the special rooms go. Special rooms are placed first, then filler.
- **Enemies are placed on a budget** proportional to the room's combat length. Each enemy type has rules: some count double, some
  can't share a platform, some need open space.
- Loot comes in a separate last pass.
- The team found that pure randomness made levels "illogical, chaotic". The constraints are what make it feel designed.
- Pacing: dramatic peaks followed by rest.

### Rogue Legacy, Canabalt, Rayman Jungle Run, Alto's Adventure
- **Rogue Legacy:**
  - hundreds of hand-made rooms shuffled into fixed regions;
  - harder rooms hold the chests;
  - optional chest challenges ("reach it without taking damage");
  - even the outdoor forest is cut into rooms.
- **Canabalt:** rooftops built from modular "Lego" pieces snapped into random heights and widths. Its look is greyscale with the
  runner in black, so the player always stands out. The rhythm of the obstacles matters as much as the speed.
- **Rayman Jungle Run:**
  - the depth is in the placement, not the controls;
  - collectibles trace the intended line through a level (a reward and a hint at once);
  - where the level allows, you can double back, which reviewers liked as a change of pace.
- **Alto's Adventure:** endless generated slopes and chasms that read as real terrain because they follow how real slopes behave.
  The hand-made part is the goals, not the ground.

### Lessons for our run
1. **Hand-make the pieces, generate the arrangement.** Every room is authored. The dice choose which room comes, its variant, its
   mirror, its optional pieces and its enemies, never raw geometry.
2. **Show a fork's risk before the choice.** A fork's two exits should look different: high road or low road, lit or dark,
   coins visible or not.
3. **Rare things should be weighted to stay rare but never vanish.** Use a pity counter, as with Punch Quest's egg.
4. **Enemies: one clear rule each, and an announcement before each attack** (a wind-up pose and a sound). Then combine them.
5. **Budget enemies by room length**, with per-kind placement rules.
6. **Rhythm:** effort, then rest. A peak (an arena) is followed by a breather (a street with coins, a rest stop, a shop).
7. **Coins trace the line.** Coin trails show the jump and reward the good route.
8. **Upgrades should come slowly.** Fewer, more meaningful shop visits, not something new every room. This matches our current
   5 to 7 room spacing.

## 2. Level geometry

### 2.1 Room recipes: templates, variants and obstacle pieces
- A room stays hand-made. It gains **optional parts**, each with a chance:
  - a crate stack;
  - a raised planter;
  - a second awning;
  - a gap or no gap.
- A room may also be **mirrored**. Rooms connect at the ground line, so mirroring keeps the seams valid.
- **Obstacle pieces:** small hand-made clusters, Spelunky-style, fitted into marked slots in a room. Each slot is a ground slot or
  an air slot. Example pieces:
  - a dumpster with a crate on it;
  - an AC unit and a vent pipe;
  - a water tower on legs;
  - a billboard frame;
  - a parked delivery van;
  - a construction scaffold.
- The **room budget** of platforms and ledges grows to fit this. It is currently 6 platforms and 2 ledges, set by fixed pool sizes.
- `sim_content::check_run` keeps verifying that every combination is passable: the gap widths and step heights a fighter with
  the worst jump in the cast can clear.

### 2.2 Slopes
- **Gentle ramps** (up to about 25 degrees): rooftop ramps, a street rising to a bridge, stairs drawn as a ramp, a sloped tin roof.
- The fighter walks, runs and dashes along them, and lands on them. Horizontal speed stays as on flat ground, with the feet
  following the surface (the reference's handling). Jumping off and landing on a slope works.
- Rules:
  - **Versus stages stay flat and unchanged.** Slopes are a ground feature any stage *could* use, but only run rooms use them now.
  - Ledges stay at flat corners only.
  - Wavedashing and teching on a slope follow the surface.
  - Knockback and trip moves on slopes: the move notes already say "trips only on a slope", so once slopes exist, those trips
    should work as the reference intends. Test them.
- Changing the ground format is a content schema change. Bump `SCHEMA_VERSION` with a migration where flat ground reads as
  slope 0.

### 2.3 Alternating pathways (forks)
A **fork room** splits into a **high road** (rooftops) and a **low road** (the street), and they rejoin in the next room. Each road:
- **High road:** gaps and jumps, fewer enemies, more coins, and a fall drops you to the low road. It never costs a life by
  itself unless there is a true pit.
- **Low road:** more enemies, safer footing, sometimes a shop door or a hidden alley.
- **Telegraphed:** a coin trail up the fire escape for the high road; a lit storefront or a crowd of enemies for the low road.

The map strip shows the fork as a split icon. Later, a fork can lead to a **special room**:
- a treasure alley behind a breakable fence;
- a challenge room ("reach the chest without being hit", after Rogue Legacy);
- the rare "event" room (section 4).

### 2.4 Vertical variety inside a room
Each room gets an *elevation line*: flat, rising, falling, or a valley (down then up). The next room starts at the height the last
one ended, and the height drifts over a run, within about one screen of the ground line. Mixing these gives the hills-and-valleys
rhythm without any slope being steep.

## 3. A believable world: if it looks solid, it is
- **Everything you can stand on is a recognisable object**, and every recognisable flat object you would expect to stand on is
  solid or a pass-through platform. Examples:

  | Object | What it is in play |
  | --- | --- |
  | Awnings, balconies, fire-escape landings, scaffolding planks, billboard catwalks, bus-shelter roofs, AC units, water-tower decks | pass-through platforms |
  | Dumpsters, crates, parked vans and buses, kiosks, chimneys, rooftop sheds, stair blocks | solid blocks (you can stand on them and they block you) |
  | Building edges, rooftop parapets | ledges you can hang from |
  | Lamp posts, signs on poles, clotheslines | **decoration only**, kept thin and dark so they never look like footing |
  | Smoke from vents, steam | readable, harmless background |

- **Rule for the art director:** footing has a lighter top edge (the same rule as the versus stages' platforms), and decoration
  doesn't. A player should be able to tell, without trying, what will hold them.
- **Why the geometry exists:**
  - gaps are alleys between buildings (with the street far below if it's a true pit, or a dumpster and a fire escape if you can
    climb back);
  - height changes are building heights;
  - ramps are tin roofs, loading ramps, stairs and the approach to a bridge;
  - walls are the next building's side, with windows.
- **Layers:**
  - play-plane objects are fully lit;
  - a mid layer of the same street, slightly darker and slower, with people in windows and traffic far below;
  - the skyline as the far layer, with the time of day.
  - Parallax between layers gives depth; the R1 report noted the skyline currently has none.
- **Life:**
  - pigeons that scatter as you run past (purely cosmetic);
  - laundry that sways;
  - window lights that switch on at dusk;
  - a cat on a ledge.
  - These cost little and do the most for a "lived-in" city. They're presentation only (Godot); the sim never sees them.
- **Enemies belong somewhere.** Walkers come out of rooftop doors and stairwells; flyers come from pigeon coops and antenna masts.
  Enemies appearing from nothing breaks believability; a short emergence animation fixes it.
- **Transitions:** a room's back wall changes when you change building (brick to concrete to glass), so each room reads as a new
  building on the same street.

Ownership: the **modes agent** owns the sim side:
- which objects are footing;
- the obstacle pieces;
- slopes;
- the fork;
- the placement rules.

The **art director** owns the look:
- the city kit (objects as described above);
- the background layers and life;
- the shopkeeper.

Placeholders stay acceptable until the kit arrives. Every footing object in the run data gets a `look` tag (dumpster, awning,
crate, and so on) so the art can be swapped in without touching the sim.

## 4. Enemies (direction for R2 and R3)
Each enemy has one rule and a clear wind-up. Six to start, in line with the plan's section 3.3 roles:

| Placeholder name | Rule | Counter | Teaches |
| --- | --- | --- | --- |
| Walker (exists) | rushes and swipes | anything | basics |
| Diver (exists) | hovers, dives at you | up air, shield, sidestep | anti-air |
| Shieldbearer | blocks from the front | attack from above, grab, or cross up | approach angles (Punch Quest's orc) |
| Spearman | a long, delayed poke along the ground | jump over it, punish the recovery | timing |
| Lobber | throws arcing bottles from a roof | reflect them, close in, or dodge | projectiles; your fighter's reflect matters |
| Bruiser | slow, armored, big launch | hit and back off, or a strong move breaks the armor | respect and spacing |

- **Hazards as allies (Punch Quest's torches):** gas canisters and loose signs that explode or fall when hit, hurting enemies
  too.
- **Placement:** the enemy budget scales with room length and with the city tier. Lobbers need high ground, bruisers need flat
  room, and there are never two shieldbearers on one platform.
- **Announcements:** every attack has at least an 18-frame wind-up early in the run, shortening a little with each city tier,
  with a sound per kind.

## 5. Rare events (the egg)
At most one per city, weighted with a pity counter, shown as a "?" on the map strip. Ideas in our setting:
- **Rooftop chase:** a runaway delivery van on the street below; ride on its roof for a short invulnerable dash, scattering
  enemies.
- **Pigeon storm:** a flock lifts you for one big glide over a stretch of rooftops.
- **Night market:** a pop-up stall street with discount stalls and a crowd.
- **Blackout:** the lights go out; only lit windows and enemy eyes show; coins are doubled.

## 6. The shop: walk in, meet the shopkeeper
- The storefront is an actual **door into a small interior room**, about one screen, with no enemies.
- Inside are a counter, shelves, and the **shopkeeper**: a cute original character (the art director designs it in the house
  style). It reacts:
  - waves when you enter;
  - looks at the item you stand next to;
  - cheers when you buy;
  - grumbles cheerfully when you can't afford something;
  - gets dizzy if you hit it, never hurt, with a little "hey!".
- Wares sit on pedestals or the counter. You **walk up to an item and press up to buy** it; its price floats above it. No menu.
- Leaving through the door returns you to the street where you entered. The world outside is paused while you're inside, as now.
- It stays small: a cozy moment between rooms, not a feature. The R1b ware list (heal, life) stays as data, and R3 fills it.
- **Sim note:** the interior is a separate small stage the run swaps to while inside, deterministic and hashed. It replaces R1b's
  menu panel. The menu-panel code can go.

## Sources
- [Critical-Gaming Network blog review of Punch Quest](https://critical-gaming.com/blog?currentPage=28)
- [RocketCat developer tips (Pocket Gamer)](https://www.pocketgamer.com/punch-quest/how-to-master-punch-quest-expert-tips-from-developer-rocketcat-games/)
- [The making of Punch Quest (PocketGamer.biz)](https://www.pocketgamer.biz/punching-below-its-weight-the-making-of-punch-quest/)
- [Punch Quest fan wiki: dungeon segments](https://punch-quest.fandom.com/wiki/Dungeon_Segements) and [backgrounds](https://punch-quest.fandom.com/wiki/Backgrounds)
- [Punch Quest guide (Gamelytic)](https://gamelytic.com/punch-quest-game-play-tips-and-guides/)
- [Punch Quest (Wikipedia)](https://en.wikipedia.org/wiki/Punch_Quest)
- [Spelunky Generator Lessons (Darius Kazemi)](https://tinysubversions.com/spelunkyGen2/)
- [The level design of Dead Cells: a hybrid approach (Game Developer)](https://www.gamedeveloper.com/design/building-the-level-design-of-a-procedurally-generated-metroidvania-a-hybrid-approach-)
- [Canabalt GDC postmortem (Destructoid)](https://www.destructoid.com/gdc-10-falling-to-your-death-the-canabalt-postmortem/)
- [Rogue Legacy (StrategyWiki)](https://strategywiki.org/wiki/Rogue_Legacy)
- [Rayman Jungle Run review (Engadget)](https://www.engadget.com/2012-10-02-portabliss-rayman-jungle-run-ios-android.html)
- [Alto's Adventure (IGF)](https://igf.com/altos-adventure)
- [Procedural generation for endless games (SBGames 2013)](https://www.sbgames.org/sbgames2013/proceedings/comp/23-full-paper.pdf)
