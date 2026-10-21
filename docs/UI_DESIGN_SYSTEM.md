# UI design system

One set of rules for every screen (menus, select, creator, controls, online, replays, pause, results, the match and run HUDs), so the
game reads as one product. Written 2026-10-17 as step 1 of the UI pass (docs/SATURDAY_BACKLOG.md, "UI redesign"); the code lives in
`godot/ui/ui_draw.gd` (colours, drawing), `godot/ui/ui_kit.gd` (widgets, motion) and `godot/ui/menu_art.gd` (painted cards).
Original art only: no names, logos or layouts copied from any other game.

## Character

Sleek but cartoony: flat colour, thick ink edges, a slight forward lean (the slant), bold condensed italic type, a little bounce.
Panels are painted cards, not system widgets. **Daylight** (the owner's pick, 2026-10-19, `docs/art/ui/mocks/style_options.png`
option 2): the menus stand on a pale cool ground, and the colour lives in the cards, which are fully saturated. Ink text on the ground
and on light cards; white text only on coloured cards, their shaded label bands and the dark ink panels (HUD plates, key bars, banners).

## Palette (`ui_draw.gd`; use the names, never a raw `Color(...)` in a screen)

| Token | Value | Role |
| --- | --- | --- |
| `INK` | 0.11, 0.12, 0.17 | Text on light cards, every edge and outline, dark panels (at 0.72-0.92 alpha) |
| `CREAM` | 0.95, 0.96, 0.98 | Card and button fill, light text on dark panels |
| `CREAM_DARK` | 0.78, 0.81, 0.88 | Card's lower gradient, disabled fill, secondary text on dark |
| `SKY` | 0.74, 0.88, 0.95 | Calm highlight, info |
| `GOLD` | 1.0, 0.82, 0.22 | Focus and selection (glow, the focused button's accent), the reveal wipe's stripe, "free play" |
| `TEAL` | 0.2, 0.62, 0.66 | Confirm, online, positive state |
| `RED` / `BLUE` | 0.92, 0.28, 0.3 / 0.25, 0.5, 0.95 | Player 1 / player 2 only (and the HUD's player plates) |
| `SLATE_TOP` / `SLATE_BOTTOM` | 0.25, 0.3, 0.36 / 0.12, 0.15, 0.2 | Dark panels over the match (pause, results) |
| `GROUND_TOP` / `GROUND_BOTTOM` | `f4f7fc` / `c9d3e8` | The daylight ground behind every menu (`Backdrop`): a cool white falling to a light steel |
| `GROUND_MARK` | `3c5a9c` at low alpha | The ground's faint slanted stripes and corner halftone |
| `LABEL` | `5a6382` | Secondary text on the ground and on light cards (subtitles, captions, stat names) |

Players 3 and 4 take the HUD's yellow and green (`match_hud.gd` `PLAYER_COLORS`); keep them out of menus' chrome. (The HUD's
player 1 and 2 are a touch off `RED` and `BLUE`: one set for both, step 2.) Gold means "this is
selected" (a focused button's fill, READY, the reveal's stripe), nothing else. Focus on cards is shape and depth, not gold (below).

## Type (`UiKit.font()` text, `ui_draw.gd` `display_font()` titles; Barlow Condensed, SIL OFL)

Six sizes, nothing between them (1280x720 base, `canvas_items` stretch):

| Step | Size | Use |
| --- | --- | --- |
| `CAPTION` | 16 | Key hints, footnotes, small stat labels |
| `BODY` | 22 | Descriptions, list rows, tag labels |
| `LABEL` | 28 | Buttons in lists, card titles |
| `HEADING` | 34 | Main buttons, screen sections |
| `TITLE` | 54 | Screen titles (`ScreenTitle`), percentages in the HUD |
| `DISPLAY` | 84 | The logo line, "GAME!", the results winner |

Constants: `UiKit.TYPE_CAPTION` ... `UiKit.TYPE_DISPLAY`. Upper case for titles and buttons; sentence case for descriptions.

## Shape

- Slant: panels, buttons and tags lean `SKEW` (0.22) forward; tiles lean `TILE_SKEW` (0.08). Nothing leans back.
- Edges: ink, at the widths `ui_draw.gd` and the widgets already draw (to be named as constants in step 2); never a coloured edge
  on a card.
- Depth: a hard ink offset shadow under every card (`Art.SHADOW`, 6 by 8 px), a gloss band under the top edge; no soft drop shadows.
  On the light ground the shadow is what separates a card from it, so every card keeps one.
- Corners: square on slanted panels (the slant is the shape); round only for round things (arrows, stock icons).

## Layout

- Safe margin 40 px left and right, 40 px top, 48 px bottom (the key bar sits in the bottom margin).
- Screen title top-left at (40, 40); the key bar (`KeyBar`) bottom across the width; content between.
- Gutters between cards 16 px; between groups 32 px.
- One focus at a time: exactly one thing is lifted and framed (below).
- A screen's titles and descriptions sit on the ground in `INK`; subtitles and captions in `LABEL`. No white text on the ground.

## Focus (daylight, the owner's pick 2026-10-19)

The focused card is the loudest thing on the screen through shape and depth, readable without colour:

| | Rest | Focused |
| --- | --- | --- |
| Cards and tiles (main menu, mode cards, roster cards, the creator's choices) | its hard shadow, its own deep-shade outline | lifted, on a long hard shadow (`FOCUS_SHADOW`, 14 by 16 px), in a thick frame (ink outside, white inside, `Art.focus_frame`), tilted `FOCUS_TILT` (-2.5 degrees) and scaled up to `FOCUS_SCALE` (1.04-1.08, as the gaps allow), drawn over its neighbours |
| Buttons in lists and rows | cream, hard shadow | gold fill (selected), the same frame and long shadow, lifted; never tilted, so rows stay aligned |

The motion is `MOTION_FAST` with the small overshoot (`pop`); the frame, lift, tilt and scale all follow one value. No glow rings.
Destinations show one description line for the focused one (bottom centre, `TYPE_LABEL`, ink) instead of a subtitle on every card.

## Components (`ui_kit.gd`)

`Backdrop` (the daylight ground behind every screen; `dim` cools it a step for dense screens, `warm` leans it toward peach), `ScreenTitle`, `TopBar` (an ink bar with the screen's name and, at the right, what it is for), `Btn` (lists and main actions), `Tile`
(the main menu's destinations, with a scene or spotlight), `Tag` (labels, values), `Arrow` (left/right choosers), `KeyBar` (the
controls for this screen), `notice` (a passing message), `Stage` (a framed fighter preview). A new screen is built from these; a new
widget goes into `ui_kit.gd` first, with these rules, not into the screen.

## Motion

| Name | Time | Where |
| --- | --- | --- |
| `MOTION_FAST` | 0.12 s | Focus pop and release on buttons and tiles (`pop` at 7-8 a second), press squash |
| `MOTION_BASE` | 0.2 s | Values changing, panels opening, a tag swapping text |
| `MOTION_SCREEN` | 0.38 s | Screen change: the ink wipe with a gold stripe (`Reveal`), then the new screen's cards slide in |
| `MOTION_SHINE` | 0.5 s | The gloss sweep across a newly focused button |

Ease out (fast start, soft land) for things arriving, ease in for things leaving; a small overshoot (bounce) only on focus. Nothing on
a menu moves on its own except the backdrop and a fighter's idle.

## Lessons from the owner's results-screen plan (2026-10-17), for every screen

- One treatment for related pieces: the same border weight, shadow direction and panel construction; the strongest decoration only on
  the one thing that matters most on the screen (the headline, the winner, the focused choice). Not every panel at full weight.
- Supporting text quieter than headings: captions and stat labels in `font()` at 0.72 ink, values in `display_font()`; no redundant
  labels ("P1 PLAYER 1" said the same thing twice).
- State in words or shape as well as colour (the WINNER ribbon, not only gold).
- A deliberate backdrop: a light veil deeper where text sits, so the world behind stays a place, not mud; no placeholder textures
  (rays and hatching) behind a showcase.
- One primary action per screen in the screen's accent, the others quieter.
- Motion restrained and coordinated: things arrive in reading order and the screen settles in about a second; nothing loops but the
  world and the fighters.

## Audit (2026-10-17): where screens leave the system

- Raw colours in screens: `menu_art.gd` 57, `match_hud.gd` 43, `run_hud.gd` 38, `select.gd` 15, `controls.gd` 12, `creator.gd` 11,
  `replays.gd` 9, `menu.gd` 8, `results.gd` 7 (counts of `Color(`; many are the palette's values typed again).
- Type sizes in use: 15 to 34 in thirteen steps, plus 54, 81 and 84; map each to the six steps above.
- Player colours: `match_hud.gd` `PLAYER_COLORS` and the palette's `RED`/`BLUE` differ slightly.
- Motion: the screen wipe, the button and tile focus, and (2026-10-17) the cards arriving in sequence behind the wipe (`Entrance`);
  values still snap, and leaving a screen has no wipe.
- Missing screens or states, from the backlog: the character select's big player slots, the title screen, the results screen
  (`--demo=results` stops in the match first).

## Order of the pass

1. This document, and the type and motion constants in `ui_kit.gd`.
2. Replace raw colours and sizes screen by screen (menu, select, creator, controls, online, replays, pause, results, HUDs), each with a
   before/after capture.
3. Motion: cards slide in after the wipe (`MOTION_SCREEN`), values ease (`MOTION_BASE`).
4. Character select redesign, title screen, results screen.
