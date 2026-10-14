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
| 2 | **The beach's wall is a grid of identical rounded panels**, the biggest surface on that stage, the last obviously tiled one. | `review3`, beach. | The masonry tile: two regular courses, repeated every 7 units. | **Done**: the beach's own `seawall` pattern, big stones in random bond, every row fitted to the tile so it wraps cleanly, laid every 14 (a tide line was tried and dropped: it repeats with the tile). `review3_seawall.png`. |
| 3 | **Platforms are the same plank on four stages.** | Every stage but the city. | One `platform()` builder for all wood themes. | **Done**: `_platform_dressing`, the same readable plank dressed per stage at its ends and under its top: rope lashings (canyon, beach), moss and hanging vines (grove), hay and a flower (meadow). `review3_platforms.png`. |
| 4 | **No foreground**: nothing frames the play space in front of the stage. | All stages. | Never built. | **Tried and dropped** (a judgement call for the owner): framing pieces were built per stage, but the camera's bottom corners hold the HUD's damage cards (the pieces sat behind them, adding clutter under the numbers) and its sides are where fighters recover to the ledges. The reference fighters keep this space clear for the same reason. The depth comes from the stage's own layers instead. |
| 5 | **Player tags crowd in close combat**: P1 and P2 overlap the fighters' heads during a hit. | The strong hit. | Fixed offset above each fighter. | Smaller tags that fade a little in close combat, or step aside when they overlap. |
| 6 | **A blue player's knock-out blast is pale on a blue sky** (the beach). | The KO capture. | The plume uses the player's colour, lightened. | A deeper body, a stronger ink rim, when the sky behind is near the plume's hue. |

Deferred to the end (the owner's schedule): weapon models (the greatsword in particular dominates the small bodies), animation clips and
poses (items, the Bounder's and Wanderer's moves), modelled items.

## Merged with the outside roadmap (the owner's `game_visual_polish_roadmap`)

The outside roadmap agrees on effects, background depth and the UI, and adds what this review missed: **the ocean** (the beach stage's
largest surface is its water, never refined) and the beach's own assets (lighthouse, pier, palms, islands). Its rendering and material
standards are already written (`docs/VISUAL_STYLE.md`, "Material standards"); its character phase is done apart from animation (deferred).
The combined order, to bridge the style across backgrounds, foreground, UI, fighters and effects:

1. Done: the invincibility tint, the beach's sea wall, themed platforms.
2. Done: **the ocean**: painted wave crests with troughs (fading with distance), crisper streaks, lighter shallows.
3. **The beach's assets**: the islands are done (pass 60); the palm crowns, lighthouse and boat are modelled props, so they wait for the props pass (a foliage material on the fronds was tried: no visible change).
4. ~~Foreground framing~~: tried, dropped (see weak point 4).
5. **Effects consistency**: rings (jump, shockwave, shield hits) now carry the standard's fine ink edges (`Ring.inked`,
   `review3_rings.png`); dust and smoke were already painted puffs. Next: a short effect style guide.
6. **UI**: a written design system (type, spacing, panels, states, player colours), the in-match tags (weak point 5), the HUD hierarchy.
7. **The blue KO blast's contrast** (weak point 6).
8. Regression: performance, aspect ratios, controller navigation, every stage.
