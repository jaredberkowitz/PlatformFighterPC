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

Waiting for the owner's pick; only the chosen ones get built.
