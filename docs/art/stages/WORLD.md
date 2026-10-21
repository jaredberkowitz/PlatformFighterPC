# World-building: what makes each stage a place (2026-10-21)

The owner's direction after review 3: the stages look like paper now; next they should be believable places. Each needs a reason to exist
and signs of life. This is the story sheet for that pass: per stage, the place, why the fight is there, what is missing, and the changes
that would give it the most belief, ranked. All of it adds elements, so each change is mocked first (from the match camera) and built only
on the owner's pick. Everything stays presentation only, in the paper language (rolled tubes, scored card, cut-outs), and never in front
of the fight.

**Where the match camera sees.** Near the stage, the deck or top hides the water and the floor. The beach shows its water over the dock
only beyond about 60 units back, and the canyon its floor beyond about 80. Anything there must be large to read (twice life size and
more), and it is best set to the sides of the fight or high above it.

## Beach: a fishing harbour

**The place:** a small harbour town on a headland. The dock is the town's fish pier, reaching out past its boardwalk to a fisherman's
hut on stilts. The fight is on the pier at the end of a working day.

**What is missing:** nobody uses the pier. There are no boats at it, no catch, no birds, and no one in the town.

**Changes, ranked:**
1. **A working pier** (mock A): a fishing boat moored to a buoy off the hut, nets over its side and crates of the catch on deck, and nets
   and fish crates on the boardwalk. It says what the pier is for.
2. **Harbour life** (mock B): gulls perched on the lamp, the crates and the life ring's post, more gliding over the water, and a rowboat
   by the sailboat. Paired with motion (gulls wheeling, boats bobbing), this is the cheapest life.
3. **The town at work** (mock C): market awnings, townsfolk and bunting on the quay, and chimney smoke. It is the most story, but the
   farthest, so it is the hardest to read in play.

## Canyon: a desert crossing at dusk

**The place:** a mesa above a desert crossing, the last light on the buttes. The fight is on the mesa top, a lookout over the way west.

**What is missing:** the crossing itself. The floor between the stage and the buttes is empty sand (review 3's "flat floors"), and
nothing says anyone passes this way.

**Changes, ranked:**
1. **A railroad** (mock A): a timber trestle across the floor behind the buttes, a little train on it under its smoke, and a water tower
   by the track. It is the boldest, crosses the whole frame and gives the floor depth.
2. **A wagon trail** (mock B): a pale trail winding to the horizon with telegraph poles along it and a covered wagon. It is quieter,
   and the poles carry the eye into the distance.
3. **A ranch** (mock C): a windpump and its tank, a corral with cattle, and an adobe house with chimney smoke. It is the most homely,
   and smaller in frame.

## Meadow: a hilltop farm

**The place:** a farm on a hill above a valley village with a windmill. The stage is a stone-walled terrace in the farm's field.

**What is missing:** the farm at work. There is no road up to it, no animals and no people.

**Changes, ranked:**
1. **Sheep and cows** (mock B): a flock grazing in the near fields on both sides of the terrace and a few cows by the village. It is the
   clearest sign of a working farm, and it reads at the match camera.
2. **Village life** (mock C): chimney smoke over the cottages, a washing line and a scarecrow in the field. The smoke says someone lives
   here.
3. **A farm track** (mock A): a pale track winding down from the terrace's left toward the village, with a hay cart on it. It is small in
   frame.
- Later, with motion: the windmill's sails turning.

## Grove: the great tree's festival

**The place:** a floating island round a great tree, strung with lanterns for a festival.

**What is missing:** who hung the lanterns.

**Changes, ranked:**
1. **A treehouse** (mock A): a shrine platform in the great tree above the top platform, with a rail, a roof and a ladder. Someone lives
   in the tree.
2. **Festival gates** (mock B): red gates at the island's edges and paper streamers hanging from the lantern strings. They say what the
   lanterns are for.
3. **Floating islets** (mock C): small islands in the sky either side, each with grass and a tree. They add scale but no story.
- Later, with motion: fireflies or drifting petals.

## City: a rooftop over a busy street

**The place:** a rooftop above a shopping street at night, with searchlights over the skyline.

**What is missing:** the city's life. The street below is empty.

**Changes, ranked:**
1. **Billboard and rooftop party** (mock B): a billboard top right and a lit rooftop party with string lights on a neighbouring roof.
2. **Street traffic** (mock A): a bus and pedestrians on the street at the rooftop's left and right.
3. **An elevated train** (mock C): a train on an elevated line behind the stage. The rooftop hides most of it, so it reads weakly.
- Later, with motion: windows lighting and going dark, the traffic moving.

## Method
- **Mocks:** `art/blender/make_world.py --theme ocean|sunset|meadow|grove|city --variant A|B|C` (variants combine: `--variant AB`)
  builds each change in world space. `--world-mock=<glb>` on the stage demo places it (`StageBake.place_world`, painted by family, `WORLD_COLOURS`).
- **Options:** `mocks/world/` (match camera and wide, with a local `review.html`) and on the owner's page.
- **Picks recommended:**
  - Beach: A and B together. They are the strongest pair: the pier's purpose and its life.
  - Canyon: A, the railroad. It fixes the empty floor most and gives the frame a strong horizontal behind the buttes.
  - Meadow: B and C together (the flock and the village's smoke).
  - Grove: A and B together (the treehouse and the festival gates).
  - City: B, with A's traffic if the street should be busy too.

## Built: the beach's working pier and harbour life, the canyon's railroad (2026-10-21)

The owner's picks, shipped as `godot/models/stages/beach_world.glb` (`make_world.py --theme ocean --variant AB`) and
`canyon_world.glb` (`--theme sunset --variant A`), for each exact layout (`StageBake.WORLDS`, placed by `place_world`).
- `--no-world` compares.
- `tests/stage_front_test.gd` keeps them behind the fighters' plane over the play space, as it does the mood light.
- Before/after: `world_built_1.jpg` (before left; beach above, canyon below).
- **Tests:** all seven Godot tests pass.

## Built: the meadow's flock and village life, the grove's treehouse and festival, the city's billboard, party and traffic (2026-10-21)

The owner's picks, shipped as `godot/models/stages/meadow_world.glb` (`make_world.py --theme meadow --variant BC`), `grove_world.glb`
(`--theme grove --variant AB`) and `city_world.glb` (`--theme city --variant AB`), in `StageBake.WORLDS` for each exact layout.
Every stage now has its world-building.
- `--no-world` compares.
- Before/after: `world_built_2.jpg` (before left; meadow, grove, city from the top).
- **Tests:** all seven Godot tests pass.

## Round 2: life (2026-10-21)

The owner's go-ahead: carry on with the sheet, adding what is believable, adds to the world and fits what is there. The places now have
their reasons and their signs of life; next they should be alive. The scenery already moves on its own clock, smoothly (the windmill's
sails, the clouds, swaying crowns, the city's cars), so the world pieces move the same way, gently, and nothing moves near the fight.

**What moves, by stage:**
- **Beach:** the fishing boat, its buoy and the rowboat bob and roll on the swell; the gliding gulls wheel in slow loops, turning to
  face their way; the perched gulls hop now and then and turn their heads.
- **Canyon:** the train's smoke curls; two hawks (new) wheel high over the buttes; the train runs along the trestle (a traveller).
- **Meadow:** the sheep and cows lower their heads to graze; the chimney smoke curls; the washing flutters; the scarecrow rocks.
- **Grove:** the streamers flutter; fireflies (new) wander and blink round the treehouse and under the lantern strings.
- **City:** the party dances and its string lights twinkle; the bus drives along the street and the people stroll (travellers).

**Options** (mocked as clips from the match camera, `--world-motion=off|place|all`):
- **A, life in place:** everything above except the travellers.
- **B, life and travellers:** A, and the train, the bus and the people move along. Recommended: the train crossing is the canyon's
  biggest gain, and the bus joins the city's own car traffic.

**Method:** `make_world.py` groups what moves under a pivot empty named `move_<kind>_<n>` (nested where a thing rides on another, the
smoke on the train); `scripts/world_motion.gd` animates each kind; `--variant L` adds each stage's new living things (the hawks, the
fireflies).

**Built (2026-10-21):** every shipped world model rebuilt with its movers (`--variant AL` on the canyon, `ABL` on the grove for
the hawks and fireflies); `scripts/world_motion.gd` moves them; `StageBake.place_world` adds it, the fireflies glow as the lamps do.
- **The owner's pick (2026-10-21): B**, life and travellers (`--world-motion=all`, the default); `place` gives A, `off` today's
  stillness, to compare.
- **Clips:** `mocks/world_life/` (8 s each from the match camera, `wm_<stage>_<T|A|B>.mp4`, with a local `review.html`), made with the
  `stage_clip` demo and Godot's `--write-movie` (the camera settles over the first 2.5 s, which the clips skip).
- `tests/stage_front_test.gd` checks every `move_` pivot is a kind the motion knows.
- **Gotchas:** the match camera looks down 10°, so far things sit higher in frame than their height suggests (hawks at 22 units up were
  out of frame); a lone speck is lost among the grove's lanterns without a glow.
