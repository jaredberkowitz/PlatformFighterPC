# Art review 3: the whole game after the polish plans (2026-10-14)

A fresh look at everything, after the visual polish, character and stage fidelity plans landed. Main at sim 111. The captures were all
five stages at play distance with the HUD, a strong hit, a knock-out, the main menu and fighter select. Animation, modelled weapons and
props stay deferred, by the owner's call.

## Where we are

- **Clearly improved**: fighter shading (two flat tones, a creamy rim, one clean line); the value hierarchy (now actually on in matches);
  painted, inked effects in the knock-out blast's style; backdrops at the stage's finish on the meadow, canyon and city; built props.
- **Cohesive**: menus and HUD share one card language; every stage reads as one painted world with its own mood.

## Weak points, ranked by impact

| # | Weak point | Seen in | Cause | Plan |
|---|---|---|---|---|
| 1 | **An invincible fighter turned muddy brown** (after a respawn, a dodge, a ledge grab). | The KO capture: the respawning fighter. | It was made 50% see-through, so the dark outline shell behind the body showed through and mixed with the sky. | **Fixed**: invincibility is a pale cool flicker painted on the fighter (`GHOST_FLASH`). |
| 2 | **The beach's wall is a grid of identical rounded panels**, the biggest surface on that stage, the last obviously tiled one. | `review3`, beach. | The masonry tile: two regular courses, repeated every 7 units. | A sea wall of its own: irregular stones, a wet tide line, weed and barnacles, laid large. |
| 3 | **Platforms are the same plank on four stages.** | Every stage but the city. | One `platform()` builder for all wood themes. | Themed platforms: driftwood on the beach, a stone slab or a branch in the grove, sun-bleached planks with rope in the canyon. |
| 4 | **No foreground**: nothing frames the play space in front of the stage. | All stages. | Never built. | A few code-built framing pieces at the bottom corners per stage (grass and flowers, rocks, a railing), kept out of play. |
| 5 | **Player tags crowd in close combat**: P1 and P2 overlap the fighters' heads during a hit. | The strong hit. | Fixed offset above each fighter. | Smaller tags that fade a little in close combat, or step aside when they overlap. |
| 6 | **A blue player's knock-out blast is pale on a blue sky** (the beach). | The KO capture. | The plume uses the player's colour, lightened. | A deeper body, a stronger ink rim, when the sky behind is near the plume's hue. |

Deferred to the end (the owner's schedule): weapon models (the greatsword in particular dominates the small bodies), animation clips and
poses (items, the Bounder's and Wanderer's moves), modelled items.

## Order

1 (done), 2, 3, 4, then 6 and 5.
