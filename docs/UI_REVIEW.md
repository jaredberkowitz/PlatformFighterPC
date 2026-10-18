# UI review (2026-10-18)

The owner: the UI still feels amateur and cheap next to the genre's leaders (the newest indie platform fighter, the big crossover
fighter). Screens reviewed: main menu, fighter select, in-match display, pause, results. Rule for the pass: refine what is there
(style, clarity, consistency, finish); no new elements.

## Why it reads cheap

1. **Two edge languages fighting each other.** Ragged brush-stroke banners (screen titles, the pause title, "PLAYER 1 WINS") sit next to
   crisp slanted panels, rounded key caps and soft spotlights. The leaders pick one: crisp, confident, angular shapes with clean cuts.
2. **Every panel is the same flat slab.** Dark navy or flat white, a rough border, a soft drop shadow. No material: no gradient, no inner
   highlight, no bevel, no accent stripe. Flat fills are what read as a prototype (the same lesson as the stages' flat doors and windows).
3. **No typographic hierarchy.** One bold condensed italic at every size and role: titles, buttons, stats and captions all shout at the
   same weight. Labels (KOs, Falls, Dealt) are small grey text on white, the weakest thing on the results screen.
4. **Two palettes.** `ui_draw.gd` and `menu_art.gd` each define their own ink, cream, red and blue, so screens drift apart in colour.
5. **Buttons look default.** White slabs with dark text; the selected one goes yellow with a blurry glow. No pressed state, no depth.
6. **Weak framing over the game.** The pause menu's buttons float over the live stage behind a light dim; its title bar collides with the
   timer. Results panels sit on a busy, unblurred stage.

## The pass (one design system, applied to every screen)

1. **One palette** (a single set of tokens in `ui_draw.gd`, `menu_art.gd` reading from it): a deep ink, one cream, the four player
   colours, one gold accent.
2. **One panel style** for every panel and tile: crisp slanted cut corners (the existing skew), a vertical gradient (lighter top),
   a thin lit top edge, a thick ink outline and a hard offset shadow instead of the soft blur; an accent stripe in the panel's colour.
   The brush-stroke banners become the same crisp shape.
3. **Type scale with roles:** the extra-bold italic only for titles, names and big numbers; a regular weight for labels and captions;
   labels in caps with letter spacing, in the panel's own darker tone rather than grey.
4. **Buttons with states:** rest (the panel style), focused (gold fill, a bright outline, nudged forward), pressed (sunk).
5. **Framing over the game:** pause and results darken and blur the stage behind them, so the panels own the screen; nothing overlaps.

Order: tokens and the panel and button style in `ui_kit.gd` first (every screen picks them up), then each screen's layout and type:
pause, results, fighter select, main menu, in-match display.
