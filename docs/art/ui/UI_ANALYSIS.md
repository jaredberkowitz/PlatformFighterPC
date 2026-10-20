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
