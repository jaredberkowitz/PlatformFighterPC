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
