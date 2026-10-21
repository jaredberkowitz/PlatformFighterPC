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

## Meadow: a hilltop farm (to mock next)

**The place:** a farm on a hill above a valley village with a windmill. The stage is a stone-walled terrace in the farm's field.

**Missing:** the farm at work. There is no road up to it, no animals and no people. **Changes:**
- a farm track winding down from the terrace to the village, with a cart;
- sheep or cows grazing in the near fields;
- chimney smoke over the cottages;
- the windmill's sails turning (motion).

## Grove: the great tree's festival (to mock next)

**The place:** a floating island round a great tree, strung with lanterns for a festival.

**Missing:** who hung the lanterns. **Changes:**
- a treehouse or shrine platform in the great tree, with a ladder;
- paper streamers and a festival gate at the island's edge;
- fireflies or drifting petals (motion);
- a rope bridge to a smaller floating island.

## City: a rooftop over a busy street (to mock next)

**The place:** a rooftop above a shopping street at night, with searchlights over the skyline.

**Missing:** the city's life. **Changes:**
- traffic below (cars passing with their lights; some exist);
- windows lighting and going dark;
- a rooftop billboard;
- a neighbour's lit rooftop party in the distance;
- a train on an elevated line.

## Method
- **Mocks:** `art/blender/make_world.py --theme ocean|sunset --variant A|B|C` builds each change in world space. `--world-mock=<glb>` on
  the stage demo places it (`StageBake.place_world`, painted by family, `WORLD_COLOURS`).
- **Options:** `mocks/world/` (match camera and wide, with a local `review.html`) and on the owner's page.
- **Picks recommended:**
  - Beach: A and B together. They are the strongest pair: the pier's purpose and its life.
  - Canyon: A, the railroad. It fixes the empty floor most and gives the frame a strong horizontal behind the buttes.
