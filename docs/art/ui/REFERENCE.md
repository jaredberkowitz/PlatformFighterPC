# UI reference table and audit (Phase 1, 2026-10-19)

The owner's four UI references are in `reference/`, renamed so no game is named in the repository:

| File | What it shows |
| --- | --- |
| `ref_a_select_grid.webp` | The newest indie platform fighter: local versus fighter select (640x360) |
| `ref_a_mode_cards.webp` | The same game: the online modes screen (640x360) |
| `ref_b_main_menu_panels.webp` | The big crossover fighter: a main menu of tall full-bleed panels (1500x843) |
| `ref_b_main_menu_wheel.jpg` | The same game: its released main menu, four quadrants around a round centre (500x281) |

They are a target for quality and clarity, not layouts to copy. Our screens at the same moment are in `baseline/` (1280x720, a fresh
debug bridge, branch `ui/overhaul` from `main` at 8819649). `reference_vs_baseline.png` puts each reference beside our nearest screen
(main menu, fighter select, online); `baseline_sheet.png` shows the rest.

## Measurements

Type is cap height as a share of the screen height (read off the images; ours from the font sizes in code, Barlow Condensed's cap
height is about 0.7 of the size). Colour and busyness come from `measure.gd` (every frame scaled to 640x360; a "bin" is one of 512
colour cubes, counted when it covers 1% of the frame; edge % is the share of pixels on a strong light/dark edge).

### Type and scale

| | Screen title | Main labels | Small text | Hero element |
| --- | --- | --- | --- | --- |
| Ref A select | 3.3% | fighter names 2.2% | 1.7% | the two chosen fighters' art, ~90% of the height each |
| Ref A modes | 7.5% | card names 5.5% | 1.7% | the focused card, 64% of the height, the only one in colour |
| Ref B panels | 3.1% (quiet) | tile names 5.7-6.5%, all four equal | 2.6% (one line for the focused tile) | the focused tile, 65% of the height, tilted out of the row |
| Ref B wheel | none | 10% / 5.3% | 3.6% | the main quadrant |
| Our menu | logo ~9% | Play 6.3%, others 4.7%, utilities 2.5% | 1.4-1.8%, a subtitle on every tile | the Play tile, 50% of the height |
| Our select | 5.3% | **fighter names 1.0%** | 1.4% (key bar) | **the chosen fighters, ~28% of the height** |
| Our online | 5.3% | 2.7% (all rows equal) | 2.0% | the preview card |

### Colour and density

| Image | Bins >= 1% | Mean value | Mean saturation | Edge % |
| --- | --- | --- | --- | --- |
| Ref A modes | 21 | 0.39 | **0.59** | 10.1 |
| Ref A select | 22 | 0.59 | 0.49 | 20.6 |
| Ref B panels | 24 | 0.73 | 0.41 | 11.1 |
| Ref B wheel | 15 | 0.57 | 0.58 | 10.1 |
| Our menu | 17 | 0.52 | 0.50 | 8.4 |
| Our select | 18 | 0.49 | 0.47 | 10.9 |
| Our online | 18 | 0.47 | 0.43 | 11.2 |
| Our creator | 16 | 0.50 | 0.45 | 11.0 |
| Our controls | 13 | 0.35 | 0.47 | 13.4 |
| Our replays | 19 | 0.46 | 0.39 | 10.9 |
| Our results | 25 | 0.53 | **0.34** | 8.0 |
| Our pause | 12 | 0.37 | 0.36 | 4.0 |

What the numbers say:
- **Scale of the hero.** Each reference gives one thing most of the screen: the chosen fighters (ref A select), the focused card or
  tile (64-65% of the height). Our select shows its fighters at 28% of the height, about a third of the reference's.
- **Labels are too small where they matter.** Our roster names are 1.0% of the height (about 10 px type at 720p), half the
  reference's 2.2%. Unreadable on a TV.
- **Colour.** The references are as saturated as our most colourful screens or more (0.41-0.59 against 0.34-0.50). Each has one
  confident identity: ref A a deep violet with an orange accent, ref B a light ground under fully saturated panels. Ours is the same navy
  slate behind every screen with cream slabs on top.
- **Density is not the problem.** Our screens are no busier than the references (edge 8-13% against 10-21%). Ref A's select is the
  busiest image and still reads, because its detail sits inside a few big, clearly bounded shapes. Ours spreads small pieces evenly
  (row upon row of equal buttons).
- **Focus.** Ref A desaturates every card but the focused one. Ref B lifts and tilts the focused tile out of the row in a thick frame.
  Ours marks focus with a gold outline and a small lift. Readable, but it is not the loudest thing on screen.

## Ranked weaknesses

Confirmed means seen in a capture and traced to code; a guess is marked as one.

1. **Fighter select is laid out like a settings page, not a showcase.** (Confirmed.) Fighters at 28% of the height, names at 1.0%,
   the match rules (Stocks, Time, Stage) float between the grid and the players' panels, and once both are ready the READY TO FIGHT band
   covers the P1/P2 tabs and the top of both panels (`baseline/select_locked.png`). Two players on adjacent cards stack their P tabs on
   top of each other. Cause: `select.gd` builds the grid in a 236 px band at the top (`GRID_AREA`), the panels in the bottom half, and
   places the rule buttons at y=330 between them (`select.gd:505-522`); the band is at y=296 over both (`select.gd:526`).
   Fix: a redesign (mock 2-3 layouts first): the grid in the centre with bigger cards and names, the chosen fighters large on either
   side, the rules in a top bar. Risk: high; `select_grid_test` and `creator_flow_test` drive this screen.
2. **There is no stage select.** (Confirmed.) The stage is a button that cycles a name (`select.gd:658`, "Stage: Meadow"); the
   online lobby has its own arrow chooser. Fix: a new stage select screen after fighter select: stage cards with real renders of each
   stage, a large preview of the focused one, its name and a line about it. Shared with the online lobby. Risk: medium; needs a render
   of each stage (the stage capture tool can make them) and a change to the select-to-match flow.
3. **The online screen is a form.** (Confirmed.) Seven rows of equal label/arrow/value widgets; the help text overflows its box (four
   lines in a box sized for three: `online.gd:108`, the "link" text at `online.gd:24`, "Relay are the older ways" spills out under it).
   The lobby's banner sits just under the match's TIME PLAYED plate, which peeks out above it (`baseline/lobby.png`). Fix: the
   overflow first (a bug); then mode cards (code, quick match, the older direct and relay) as the first step, with settings behind.
   Risk: medium; `online_flow_test`.
4. **One navy everywhere, and less colour than the references.** (Measured.) Every screen is the same navy slate with cream slabs; the
   results screen is the least saturated of all (0.34). Fix: a colour identity per screen family on the one palette (mock 2-3 options).
   Risk: medium, systemic; it reaches every screen, so it goes first and gets checked everywhere.
5. **Focus is not the loudest thing on screen.** (Confirmed for the creator, a judgement elsewhere.) The creator marks its focused
   choice with a square white-and-coral frame (`creator.gd:143-145`, raw colours, not the slant) where the design system says gold
   means selected (`baseline/creator.png`). Elsewhere a gold edge and
   a small lift. Fix: one focus treatment in the kit (scale, lift and a thick frame; the rest stepped back), used everywhere.
6. **Results buttons run off the screen.** (Confirmed.) Three 230 px buttons from x=560 with 16 px gaps end at x=1282 on a 1280 screen
   (`results.gd:157-162`); Main Menu touches the edge. The winner's sword still breaks its frame (known, kept as a hero shot). Fix: fit
   the row inside the 40 px margin. Risk: low.
7. **The creator's tools are a wall of unlabeled icons.** (Judgement.) Ten grey buttons of small arrow glyphs on the right, a large empty
   area under the choices, and category icons with no names. Fix: group and name the tools, use the empty area. Risk: medium;
   `creator_flow_test`.
8. **Dense screens are flat lists.** (Judgement.) Controls is 24 equal rows; replays is a column of equal bars. Readable, low
   priority.

## Proposed order

0. **Bug fixes, no design decisions:** results row overflow (6), online help overflow and the lobby's banner (3), the creator's red focus
   frame (5), the select's stacked P tabs and the band over the panels (1).
1. **System:** colour identity and one focus treatment in `ui_draw.gd` / `ui_kit.gd` (4, 5). Mock options first.
2. **Fighter select redesign** (1). Mock 2-3 layouts first.
3. **Stage select, new** (2). Mock first; then the online lobby uses it.
4. **Main menu refinement:** focus by lift and tilt, one description line for the focused tile instead of a subtitle on every tile.
5. **Online:** mode cards (3).
6. **Creator** (7).
7. **Controls, replays, pause, HUDs, motion and sound** (8).

Every pass: before/after in the same setup, both window sizes, keyboard and gamepad, the UI tests, a line in `UI_ANALYSIS.md`.

## Not checked

- 1920x1080: the game stretches `canvas_items` at 16:9, so the layout is the same; not yet captured.
- The title screen did not appear in the scripted capture (it only shows when the menu is the launched scene); `baseline/menu.png` is
  the menu after it.
- The run mode's screens and the run HUD: not captured this round.
