# UI pass log

One entry per pass: the defect, its cause, the fix, the files, what was checked and what is left. The audit and the order are in
`REFERENCE.md`; captures are in `baseline/` and one folder per pass.

## Pass 0: four bugs, no design decisions (2026-10-19)

Before/after: `pass0_bugs.png` (red strip before, green after; rows: results, online, lobby, creator, fighter select).

| Defect | Cause | Fix | Files |
| --- | --- | --- | --- |
| Results: Main Menu ran off the right edge | three 230 px buttons from x=560 with 16 px gaps end at x=1282 | the buttons share the room from 560 to the 40 px safe margin (216 px each for three) | `ui/results.gd` |
| Online: the help text spilled out under its card | four lines in a card sized for three; the `Bubble` widget never fitted its text | `UI.Bubble` steps its text down from `TYPE_BODY` until it fits (measured against the card's room: a wrapping label grows to fit its text, so its own height said nothing). The help now fits at 20 px | `ui/ui_kit.gd` |
| Lobby: the match's timer plate peeked out above the LOBBY banner, the percents showed through the veil | the match HUD stayed visible under the lobby | the HUD hides while the lobby is up and comes back (respecting Hide display) when it closes; the capture script does the same | `scripts/main.gd`, `tests/lobby_shot.gd` |
| Creator: focus was a square white-and-coral box | `Pick._draw` drew its own rectangle in raw colours | the kit's gold ring round the card's cut shape (`Art.card_glow`) | `ui/creator.gd` |
| Fighter select: two players on neighbouring cards stacked their P tabs; the READY band covered the tabs and the panels' tops | the tabs were a fixed 50x28 hanging past the card's edges; the band sat at y 296-392 over the panels (378) | the tabs scale with the card and stay inside it (P1/P2 on the top corners, P3/P4 under them); the band is 72 px high in the gap (304-376), its type on the scale (`TYPE_TITLE`, `TYPE_LABEL`) | `ui/select.gd` |

Checked: before/after in the same setup (1280x720, same keys, same waits); both players on one card (`pass0/select_same_card.png`); the
UI tests (`select_grid_test`, `controls_test`, `pause_test`, `creator_flow_test`, `editor_ui_test`, `match_flow_test`, `online_flow_test`,
`input_e2e`) all exit 0.

Left: the help text's 20 px is between two steps of the six-size scale (the fit steps by 2); the online screen's copy could be shorter
instead. The READY band still covers the Stocks/Time/Stage buttons (keys T, Y and G still work); the fighter select redesign moves the
rules to a top bar. Not checked: 1920x1080 and a real gamepad.

## Mocks for the owner: colour/focus and the fighter select layout (2026-10-19)

Throwaway renders with the real fonts, cards and fighters (scratch scripts, nothing in the game changed). Real size in `mocks/`.

**Colour and focus** (`mocks/style_options.png`, one mini screen of four destination cards, ONLINE focused):

| | Ground | Focus |
| --- | --- | --- |
| 0, today | navy slate backdrop | gold ring round the focused card; every card at full strength |
| 1, violet stage | a deep violet ground, faint slanted stripes | the focused card full colour, bigger, in a cream and gold frame; the others sink into the violet (colour and fighters dimmed) |
| 2, daylight | a pale cool ground, ink titles | the focused card lifted on a long hard shadow, tilted out of the row in a thick ink-and-white frame; the others stay full colour |
| 3, navy, stepped back | today's navy | as 1 but on navy: the others greyed to about 60% |

All three options also replace the subtitle on every card with one description line for the focused card, at the bottom centre.

**Fighter select layout** (`mocks/select_options.png`: today after pass 0, then layouts 1-3; all with the rules moved up to a top bar
and names at 15 px, half again today's):

| | Chosen fighters' height | Roster | 3-4 players |
| --- | --- | --- | --- |
| Today | ~28% of the screen | 2 rows of 9-10 small cards, names 1.0% | 4 panels along the bottom |
| 1, flanked | ~55%, full height each side | a 6-wide grid in the middle, tall cards | needs a second answer (2x2 flanks or panels) |
| 2, grid over tall panels | ~31% | 2 rows of 11 across the top | 4 panels, as today |
| 3, fighters forward | ~42%, facing across a VS | a 2-row strip along the bottom | 4 fighters in a row |

**The owner's picks (2026-10-19): colour/focus option 2, daylight; fighter select layout 3, fighters forward.** Next: the daylight ground
and the lifted, tilted focus in the kit (`UI_DESIGN_SYSTEM.md` first), proved on the main menu, then the fighter select rebuilt as
layout 3 on it.

## Pass 1: the daylight ground and the lifted, tilted focus (2026-10-20)

Before/after: `pass1_daylight.png` (rows: main menu with Play focused, online, replays, creator). The rules are in
`docs/UI_DESIGN_SYSTEM.md` (Character, Palette, Focus).

| Change | Where |
| --- | --- |
| The ground: a cool white falling to a light steel with faint pale-blue stripes and corner dots (was a deep navy); `dim` now cools it a step instead of darkening, `warm` leans it peach | `shaders/menu_backdrop.gdshader`, `UI.Backdrop`, `D.GROUND_*` |
| Focus: a long hard ink shadow and a thick ink-and-white frame round the card's cut shape (`Art.focus_frame`; `card_glow` now calls it), the card's own short shadow left out while focused; tiles also tilt 2.5 degrees, grow 3% and draw over their neighbours; list buttons keep the gold fill and never tilt | `ui/menu_art.gd`, `UI.Tile`, `UI.Btn`, `creator.gd` |
| Main menu: no subtitle on every tile; one ink description line for the focused tile at the bottom left, the key bar moved to the right; the logo stays above a tilting tile | `ui/menu.gd` |
| Spotlit cards keep their colour on the light ground (top highlight, pool and floor a step lighter-handed: they bleached the tiles to pastel) | `UI.Spotlight` |

Checked: every menu screen captured on the new ground (`pass1/`): titles are ink banners, key bars dark, cards keep their colour, the
online help and replays rows read. The eight UI tests pass. Left for their own passes: the controls screen draws its own gold-outline
focus (not the kit's); the title screen was not captured.

## Pass 2: fighter select rebuilt as "fighters forward" (2026-10-20)

Before/after: `pass2_select.png` (choosing; both locked in); `pass2_select_states.png` (four players; the run's one player).

| | Before | After |
| --- | --- | --- |
| Chosen fighters | ~28% of the screen's height, in panels along the bottom | ~37-40%, in tall panels under the top bar, facing across a gold VS with two players; bigger again with four (narrower panels keep the height) |
| Roster | 2 rows of square cards across the top, names 10 px (1.0% of the height) | a strip of wide cards (0.78 as tall as wide) along the bottom, names 17 px; portraits aimed lower and framed wider so the face clears the name band and the hats stay in |
| Match rules | Players and Rules top right; Stocks, Time and Stage floating between the grid and the panels | all five on the top bar at the right, dark setting buttons with the value in gold |
| Title | "VERSUS" and "CHOOSE YOUR FIGHTER" on two banners | the mode on a slim ink bar (the screen says what it is by its layout) |
| READY TO FIGHT | a band over the panels | over the roster strip, which is done with by then |
| News (status) | white text straight on the ground | the kit's notice (gold on an ink band) between the fighters and the strip |

Causes and files: `ui/select.gd` (`GRID_AREA`, `GRID_TOP`, `CARD_H`, `card_height`, `PANEL_TOP`, `PANEL_H`, `Header`, `_layout_panels`,
`_rule_button`; the panel's `lay_out` takes a height and `BASE_Y` moved down so the feet stand on the name band). The grid test now
checks rows against the card's real height (`tests/select_grid_test.gd`). `docs/MENUS.md` describes the new screen.

Checked: choosing, both locked in, four players (badge and tab no longer collide in a narrow panel), run mode; the eight UI tests pass.
Not checked: a long roster (30 and 100 fighters, which scroll), a long made-up fighter name in a panel, a real gamepad, 1920x1080.

## Mocks for the owner: stage select (2026-10-20)

Today there is no stage select: a "Stage: Meadow" button on fighter select cycles the name (key G), and the online lobby has an arrow
row. Three layouts, on the daylight ground with the kit's focus, using real stills of the five stages (rendered with the match scene,
fighters and display hidden: `mocks/stages/`). Sheet: `mocks/stage_options.png` (today's lobby chooser, then layouts 1-3); real size
`mocks/stage_1.png` to `stage_3.png`. All three keep the match on the top bar (the two fighters in their colours) and a Random card.

| | What leads | Stage cards | Words |
| --- | --- | --- | --- |
| 1, preview and row | the focused stage large (60% of the width) | a row of six small cards under it | the name and its line in a column beside the preview |
| 2, card wall | every stage at once, big | three to a row, the focused one lifted and tilted | its line along the bottom |
| 3, full bleed | the focused stage fills the screen | a strip of six on an ink band at the foot | the name huge over the picture, its line under it |

Layout 2 grows a row every three stages; 1 and 3 fit about eight in a row before the cards get small. An aside: the stage names describe
their layouts (Triple Tier, Flat Island) while the pictures read as places (a canyon, a harbour); worth a look when the stages are named
for good.

## Pass 3: stage select, a new screen with both of the owner's favoured views (2026-10-20)

The owner prefers layout 1 (preview) and expects some testers to prefer layout 3 (full); both are built, switchable, while opinions come
in. Captures: `pass3/stage_preview.png`, `pass3/stage_full.png`, `pass3/stage_full_random.png`.

| What | Where |
| --- | --- |
| The screen: stage cards with real stills, Random, the match on the top bar, two views (V, Tab or a controller's Y; remembered) | `ui/stage_select.gd`, `stage.tscn` |
| The stills (1280x720 webp, ~80 KB each) and the tool that makes them (one stage per run: a second match scene in one run keeps the first one's scenery) | `art/ui/stages/`, `tests/stage_stills.gd` |
| Fighter select's Start goes to stage select; Back from it restores every lock (`_reselect`); the Stage button and the G key are gone from fighter select | `ui/select.gd` |
| A controller's Y as an "alt" action that sends nothing unless a screen asks (stage select maps it to V) | `scripts/pad_nav.gd` |
| `UI.TopBar` in the kit (fighter select keeps its own bar with the VS for now) | `ui/ui_kit.gd` |

Found on the way: the key was marked handled after Back had already left the tree (an error on every Esc), fixed by handling it first;
the top bar's anchoring measured a screen not yet sized and drew the match line off screen, fixed by pinning its edges; the focused
card's long shadow dimmed the key bar, fixed by drawing the keys above the cards.

Checked: `tests/stage_select_test.gd` (new) drives Start, both views, Random, Back with the locks kept, the remembered view and the
match starting on the chosen stage, and puts the player's own settings back. Not checked: the mouse, a real controller's Y, the online
lobby (it keeps its arrow row: a follow-up).

## Pass 3b: the online lobby's stage step in the stage select look (2026-10-20)

The lobby chose its stage with an arrow row under the fighter cards. Now, once the fighter is locked in, the lobby turns to stage
select's look: the daylight ground, the stage cards with their stills, in the view the player chose on stage select (the same
`user://stage_view.txt`; V, Tab or a controller's Y switches here too). Before/after: `pass3_lobby.png` (top: before, after in the
preview view; bottom: the fighter step after, the full view). Captures: `pass3/lobby_preview.png`, `pass3/lobby_full.png`,
`pass3/lobby_fighters.png`.

| What | Where |
| --- | --- |
| `StageCard` and the two views moved out of stage select into the kit as `StagePicker` (cards, preview, words, remembered view); stage select keeps its keys and the match start | `ui/ui_kit.gd`, `ui/stage_select.gd` |
| The lobby in two steps: the fighters as before (no stage row), then the stage picker under a top bar with the match (both fighters in their players' colours), each player's line beside a chip in their colour, the friend's stage badged with their name on its card | `ui/net_lobby.gd` |
| The lobby knows the player's slot (colours); a controller's Y switches the view in the lobby | `scripts/main.gd` |
| The capture photographs both steps and both views and puts the player's view back | `tests/lobby_shot.gd` |

The rules are unchanged: lock a fighter, then a stage, Esc takes the last lock back; the same `changed` signal and `net_lobby_set`, no wire
or sim change. No Random card in the lobby: the friend sees the very stage this player would fight on, and a random pick would need a
draw the friend cannot see. With five cards the preview row keeps stage select's card width, centred (wider cards ran into the key bar).
The lines were shortened to fit beside the stages ("Ready on Flat Island!"; "Ready! Esc to change.").

Checked: `tests/online_flow_test.gd` (two players through the bridge: the stage step shows a card per stage, the friend's stage badged,
the focused card is the stage sent, both locks) and the other UI tests (select grid, controls, pause, creator flow, editor UI, match
flow, input e2e, stage select). `tests/runner_test.gd` failed once on "a coin picked up is gone" and passed on a re-run (timing; the run
mode does not use the lobby or the stage cards). Not checked: the mouse on the lobby's cards, a real controller's Y, two real machines.

## Paper pass 5: cardstock, the owner's pick of the second paper round (2026-11-03)

Branch `art/ui-paper2` from `art/anim-core` core4 (889008d9), where the paper passes 1-4 (`art/ui-paper`) and the creator fixes
(`art/ui-polish`) already are. The owner asked to keep refining the UI in its own paper-crafted style at the competitors' fidelity.
Judged on core4 at 1920x1080: the menu, select, stage select and results read as paper, but the dense screens (Online, Controls, the
creator's tools, Replays) were still the daylight kit's slabs in paper colours: every label and value its own bar, pale-on-pale, no
hierarchy. Three ways to build the kit's pieces in paper were mocked on the Online screen ("Paper UI Mocks" canvas, row 2: Cardstock,
Scrapbook, Folded); the owner picked **1, Cardstock**. Rules: `docs/UI_DESIGN_SYSTEM.md`, "Paper construction: cardstock".

Before/after: `cardstock_before_after.png` (Online, Controls keys, Replays); captures in `cardstock/` (also the controller and display
pages and the menu).

| Change | Where |
| --- | --- |
| A card is a flat sheet on a white core that shows as a thin pale cut rim along its right and bottom edges, on three paper shadows (near crisp, far soft); no drawn edge lines | `menu_art.gd` `_paper_card`, `_paper_shadow` |
| A form is one cream sheet (`UI.Sheet`, following its column as rows show and hide; one sheet can cover two columns): Online's rows, both players' keys, the controller and display pages | `ui_kit.gd`, `online.gd`, `controls.gd` |
| Row labels printed on the sheet (letter-spaced small capitals in `LABEL`, ink while focused) and values printed in ink (`Tag.printed`) instead of a slab each | `ui_kit.gd` `Tag`, `Selector`, `online.gd`, `controls.gd` |
| A row's focus: a gold strip of card under the whole row, its value risen onto a white tab (`UI.Row`) | `ui_kit.gd` |
| Choosers' arrows are punched indigo discs with a cream chevron | `ui_kit.gd` `Arrow` |
| Text fields are cut windows in the sheet (`UI.field_style`) | `ui_kit.gd`, `online.gd` |
| Keys printed on the paper (small cream key cards, slate words); over a match (pause, the lobby, the creator's preview) the band stays (`KeyBar.band`) | `ui_kit.gd`, `menu_art.gd` `key_cap`, `pause_menu.gd`, `net_lobby.gd`, `creator.gd` |
| No gloss sweep on cardstock buttons; Connect the screen's one coral primary action, Copy and Paste small blue cards; notices on an indigo card and hidden while empty (the empty dark bar on Controls) | `ui_kit.gd`, `online.gd` |
| New paper tokens `SHEET`, `SHEET_WINDOW`, `PAPER_INDIGO` | `ui_draw.gd` |

The older looks (`--ui-paper=0|B|C`) keep their pieces: every cardstock branch checks `Art.paper_style() == 1`.

Not done: the lobby's stage step sits on paper but keeps the band keys; the creator's tool buttons and category column are still
separate cards (the owner's creator redesign comes later); Replays' rows are already cards and only changed material. Not checked: a
real gamepad, the mouse on the new rows (the rows keep their nodes and signals).

## Paper pass 6: fighter select as "face-off", the owner's pick of the select hero mocks (2026-11-03)

The open item from the paper passes was the select screen's hero layout (the fighters at about a third of the height). Three layouts
were mocked with real renders on the "Paper UI Mocks" canvas, row 3 (Face-off, Split sheet, Break-out); the owner picked **Face-off**.
Before/after: `faceoff_before_after.png`; states in `faceoff/` (choosing, both ready, four players, the run).

| Change | Where |
| --- | --- |
| The players' panels at either side, the screen's height (66-646), the fighter about 85% of it, facing the centre; with three or four players two panels stacked at each side; the run's one panel at the left | `select.gd` `_layout_panels`, `SIDE_W`, `PANEL_H` |
| The fighter held in its class's jump at the top of its spring (`AnimLibrary.family_clip(cls, "jump")` at 0.35), a little wider than its panel and up past its top | `preview.gd` `hold_clip`, `select.gd` `PlayerPanel` |
| The fighter cut out of paper: a cream die-cut rim and a hard paper shadow round its shape | `shaders/die_cut.gdshader` on the panel's preview |
| The roster a grid in the centre column under a gold VS (17 cards at 89 px in four columns; the run's grid spans the right side) | `select.gd` `GRID_AREA`, `GRID_LEFT`, `GRID_AREA_RUN`, `grid_layout(n, area)` |
| Cardstock top layer: the name on a cream plate at the foot with the class above it in the player's colour, the P tab a small card at the top corner, READY! a gold card stamped on the plate's corner (at the top it covered the head) | `select.gd` `_paint_over_cardstock` |
| READY TO FIGHT! a coral card over the centre column, words stacked | `select.gd` `Ribbon` |

The grid test's "nine cards sit in one row" (the old strip) is now "the built-in roster's seventeen cards fit without scrolling at 80 px
or more". Not done: the poses are today's jump frames; striking menu poses of their own (and a weapon that follows a swing in the
menus: `_pose_menu_weapon` holds it at rest) need the animator. Not checked: 30 and 100 saved fighters (the grid scrolls), long names
on the plate (they shrink to fit), a real gamepad.

## Hero fit: long blades clear the top bar (2026-11-04)

The animator's hero poses (art/anim-core 60abf33c, the owner's picks: sword A, staff B, brawler B, quick A) raise the blade. On the face-off
panels the Greatsword's (Champion) and the Nodachi's (Exile) ran past the card's top and under the top bar; the Longsword's (Duelist)
reached the bar and covered the VERSUS title and the P1 tab. The animator tried other hand positions (the blade left the panel's side,
reached the bar upright, or crossed the face), so the fit is on the UI side: `select.gd` `HERO_FIT`, by class, [k, lower]. The fighter
is drawn k times smaller with its feet in place (camera height, aim and distance scaled together), then `lower` world units lower.
Duelist 1.14 / 0; Champion 1.32 / 0.42; Exile 1.4 / 0.72 (the nodachi is the longest and most upright). Blades are never drawn over the
bar, which would cover the stocks and time.

Before/after: `hero_fit_before_after.png` (Champion vs Exile both ways round, Duelist vs Brawler); four players both ways round in
`hero_fit/`. Checked: both sides, two and four players. Tests: select_grid, creator_flow, match_flow, stage_select, runner and orphan pass.

### Redo: the card crops the fighter instead (2026-11-04)

The fit above cost too much size (the art director: the Exile read as a much smaller character beside an unscaled Brawler; face-off art
should hold every fighter at one size). Now each fighter is cut at its own card's top edge (`PlayerPanel.clip`, open at the sides and
the foot, so it still breaks out past the card's sides and stands over its name plate). A long raised blade is cropped by the card's frame
as key art crops at a panel edge (the owner picked Sword A from a mock whose tips were already cut), and it is never drawn over the bar,
the VERSUS title or the tabs. `HERO_FIT` is empty: every fighter at one size. Before/after: `hero_crop_before_after.png` (red the fit,
green the crop: Champion v Exile both ways round, four players); also Exile v Brawler both ways round in `hero_crop/`.
