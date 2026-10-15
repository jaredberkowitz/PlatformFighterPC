# Run mode review: verticality and depth (15 October 2026)

The executive producer's analysis, written for the owner after playing R2c. The owner's note: Run mode "lacks the verticality and depth
of Punch Quest and other inspiration materials."

It compares the run as built (sim 136) with Punch Quest and the other references in `RUNNER_WORLD.md`:
- Spelunky
- Dead Cells
- Rogue Legacy
- Rayman Jungle Run
- the reference fighting game's single-player Adventure stages

It lists the pain points with the evidence from the data, then the fixes, then a proposed order.

## 1. The run as built, in numbers
| What | Value | Where |
| --- | --- | --- |
| The view | 30 wide, 18 tall | `RunRules::view_width`, `view_height` |
| How far the ground may drift up or down over the whole run | ±8 | `RunRules::drift` |
| The most a room's exit can differ from its entry | ±4 (one room is ±8) | `Room::exit_y` |
| The highest step and widest gap a room may ask for | 4.0 up, 4.5 across (a single jump for the worst jumper) | `max_step`, `max_gap` |
| The high road of the fork | 5.5 to 6 above the street | the `fork` room |
| Room templates | 21: 13 streets, 3 arenas, 2 shops, a rest stop, an alley, the van street | `RunContent::placeholder` |
| Ledges per room | 2 | `ROOM_LEDGES` |

**The whole run's height range (16) is smaller than one screen (18).** The high road is a third of a screen up. Every climb is one jump.
That is the verticality problem in one line: the camera never has to look up, and the fighter never has to climb.

## 2. Pain points

### P1. The world is a corridor (verticality)
- Rooms join only left to right, at ground height. There is no room that goes up or down, so the run reads as a flat strip with
  bumps.
- The drift (±8) keeps the run inside one screen of height for its whole length. The skyline promises tall buildings the player
  never climbs, and a street below the player never reaches.
- **Compare:**
  - Punch Quest's paths branch up and down, and choosing the right one is where power-ups come from.
  - Spelunky and Dead Cells mix horizontal and vertical rooms.
  - The reference game's Adventure mode has a vertical climb and a vertical escape stage.

### P2. The fighter's movement is wasted
A platform fighter has double jumps, recovery specials, wall jumps, ledge grabs, platform drops and fast falls. The passability rules
cap every room at one ordinary jump (4.0 up, 4.5 across) so the weakest jumper can pass. Nothing asks for a double jump, a wall jump
up an alley, a ledge catch, or a recovery across a gap, so the mode's best asset (the versus movement) has nothing to do.

### P3. Combat is flat
- Grunts start and fight mostly on the ground floor of a room. Only leapers and lobbers use high ground.
- Launching a grunt pays the same in any direction, and air hits are worth no more than ground hits.
- Punch Quest's depth is in its two attacks:
  - an uppercut sends enemies up;
  - a slam brings them down, with a shockwave.

  Combos and enemy interactions build on those two, for example knocking one enemy into others. Ours has the full versus kit but no
  reason to juggle, spike or aim launches.

### P4. Choices are thin and unseen
- A fork is one room with two roads that rejoin a room later. The treasure alley is the only branch that leads somewhere different.
- The map strip shows only the next room, so there is nothing to plan. Punch Quest's whole loop is reading the forks.

### P5. Too little variety and surprise
- 21 templates in one city, with one rare event.
- Rooms are about one or two screens of the same kind of street.
- There are no set pieces, such as a crane ride, an elevator, a collapsing roof, a chase, or a fall down a shaft.
- Punch Quest lists seven path types and several very rare events. Rogue Legacy's harder rooms hold the chests.

### P6. The world has no depth behind or in front
- It has one play plane and a parallax skyline.
- The street below and the building interiors are only pictures.
- There are no foreground layers, no going "into" a building (an office floor, a stairwell, a rooftop bar), and no traffic far below.

### P7. Flat pacing
- Rooms come in a near-random order: a street, a street, an arena, a street.
- There is no rhythm of build-up, peak and rest, apart from the arena-then-breather rule.
- Dead Cells plans each biome as a graph with peaks and rests.

### P8. Little to find
- Coins are the only pickup between shops.
- In Punch Quest, the paths you pick hand out power-ups, food and rare events. Ours hand out more coins.
- R3 (shops and items) fixes part of this.

## 3. Fixes

### F1. Height bands: a three-deck city (fixes P1, P4)
Give the city three decks, each about one screen tall:
1. **The street.** Traffic, kiosks and shopfronts. Safer, with more grunts.
2. **The rooftops** (today's level). Gaps, fire escapes, awnings.
3. **The skyline.** Cranes, billboards, water towers and catwalks between towers. Coins, risk and long falls.

How the decks work:
- Rooms say which deck they enter on and which they leave on: "stay", "climb" or "drop".
- Raise `drift` to cover all three decks, about 54 units.
- The generator keeps a deck plan, and a fork is a choice of deck that lasts several rooms, not one room.
- Falling from a deck lands you on the one below (a fall, not a death) unless there is a true pit.

This is Punch Quest's up-or-down branching, made spatial.

### F2. Vertical room types (fixes P1, P2)
New kinds of room:
- **The climb:** a building face 2 to 3 screens tall, climbed with fire escapes, balconies, a wall-jump alley and ledges.
- **The drop:** the reverse, falling past grunts on balconies.
- **The shaft:** a light well or elevator shaft between two walls.

Each has its own camera rule: the camera scrolls up or down with the player, and the room's gate is below or above the player
instead of behind.

**Two kinds of route in every vertical room:**
- **The sure route:** passable by the worst jumper. It is longer, or has more grunts.
- **The skilled route:** a double jump, a wall jump or a recovery special. It is shorter and pays coins.

The passability check then proves the sure route per fighter, instead of capping the whole room at one jump.

### F3. Vertical combat rewards (fixes P3)
- **Juggle bonus:** hits on an airborne grunt count double toward the combo, and a combo kept off the ground pays a multiplier.
- **Direction bonus:** each kind of knockout pays something:

  | Knockout | Reward |
  | --- | --- |
  | Launching a grunt off the top | "Skyward" bonus coins |
  | Spiking one into the street below | "Grounded" bonus |
  | Knocking one into another grunt | Hurts the second; Punch Quest's "deadsplosion" |
- **Grunts placed in the height bands:**
  - balcony grunts above you;
  - leapers that dive from catwalks;
  - lobbers on cranes.

  A fight then spans heights, so aerials, up tilts and spikes all matter.

### F4. A real map (fixes P4)
- Show about three rooms ahead as a small node map. Forks show both branches, with icons for what each holds: shop, chest, arena,
  "?", deck.
- Forks every 3 to 5 rooms.
- A branch's risk is visible before the choice, which is Punch Quest's rule and Rogue Legacy's.

### F5. Set pieces and events (fixes P5)
These need **moving platforms**, which are still open from the versus review. Building them for the run first, then using them on a
versus stage later, pays twice.

| Set piece | What happens |
| --- | --- |
| Crane ride | A girder swings you across a gap while grunts ride along |
| Elevator | A cage climbs a shaft through an arena wave |
| Window washer's lift | A platform climbs a tower face |
| Collapsing roof | The floor drops a deck when hit or after a delay |
| Rooftop chase | A thief runs ahead; catch them for the loot |
| Zipline | A line down from the skyline to the street |

**More rare events:** a pigeon glide that carries you up a deck, a blackout, a night market. Keep the pity counter.

### F6. Depth in the picture (fixes P6)
This is the art director's part, reusing the city stage's kit.
- The street visibly below the rooftops: traffic, crowds, the street's lamps.
- A foreground layer of signs and wires passing in front of the camera.
- **Interior rooms:** an office floor, a stairwell, a rooftop bar.

  These are cut-away building interiors you run through, a change of light and walls that breaks up the outdoors. The camera
  stays the same.

### F7. Pacing graph (fixes P7)
The generator builds each stretch of 6 to 8 rooms from a rhythm:
1. a lead-in;
2. a choice;
3. a climb or drop;
4. a peak (an arena or set piece);
5. a rest (a shop, a rest stop or a quiet street);
6. a fork again.

This is Dead Cells' concept graph, small. Tiers step the peaks up.

### F8. Things to find on the paths (fixes P8; with R3)
Branches and skilled routes hand out:
- R3's items;
- food (heals);
- a rare 1-up;
- the rare events.

So taking the high, hard route is worth more than coins.

## 4. Proposed order
| Phase | What | Why first |
| --- | --- | --- |
| **R2d: Verticality** | F1 decks, F2 vertical rooms with sure and skilled routes, the vertical camera and gates, about 15 new rooms, and the passability check per route | It is the owner's main complaint, and everything else sits on the decks |
| **R2e: Vertical combat and the map** | F3 juggle and direction bonuses and grunts placed by deck; F4 the node map with branch icons | It makes the new height matter in fights and choices |
| **R3: Shops and items** | As planned (`RUNNER_R3_PLAN.md`), plus F8 (items found on routes) | It is already planned; finds make branches worth taking |
| **R4a: Moving platforms and set pieces** | The sim's moving platforms, then F5 set pieces; versus stages can use the platforms later | A sim feature with versus value too |
| Art (any time) | F6 depth layers and interiors, by the art director, reusing the city kit | It is independent of the sim work |
| Later | F7 the pacing graph; more rare events | It needs enough rooms to arrange |

Technical notes for R2d:
- Rooms gain a deck in and a deck out.
- The room window keeps its left-to-right order: vertical rooms are tall, not sideways.
- The gate becomes the room's back edge in either axis.
- The per-room ledge limit (2) rises with the decks. The shared ledge table grows only in a run, as R2c's `bodies()` did for
  fighters, so versus speed doesn't change.
- The camera's height rules already exist (`cam_lift`, the dead zone); vertical rooms raise its follow speed.
- Fixed-point range is not a concern vertically.

## 5. Questions for the owner
1. Three decks (street, rooftops, skyline): right, or should the skyline deck be rarer (special branches only)?
2. Should skilled routes ever be required? Proposed: never; there is always a sure route.
3. Interiors (an office floor, a stairwell): in the city's style, or keep the run outdoors?
4. Moving platforms for the run first (R4a), then a versus stage later: agreed?
