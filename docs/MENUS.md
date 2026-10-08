# Menus, the character creator and character select

The game now starts at a **main menu** (`godot/menu.tscn`): **Play** (character select), **Character Creator**, **Editors** (the
content editors from `docs/EDITORS.md`) and **Quit**. Up and down (W/S) choose, Enter (J, Space) picks, Esc quits.
Launching with `--demo`, `--host`, `--join`, `--relay`, `--content`, `--chars` or `--shots` skips the menus and starts the match
directly, so `play_host.bat`, `play_join.bat`, demos and test scripts work as before.

Look and feel: sleek but cartoony, in the spirit of the two reference screens (a round-portrait select grid with big player
panels; a creator with arrow selectors, slanted label tabs, a name field and a preview on a little stage). All of it is original:
slanted cream labels, round arrow buttons, gold highlights, a slate backdrop with light beams (a pastel graph-paper one for
select), bold condensed type, and a slight bounce on hovered buttons. The pieces live in `godot/ui/ui_kit.gd` and `ui_draw.gd`.

## Character creator

Choose a **class** (the moveset: Longsword or Claws), a **look** (face, hat, glasses, neckwear, body colour, accent colour: cosmetic
only) and four **stats**, give it a **name**, and press **Finish**. Up/Down pick a row, Left/Right change it, Enter on the name
or Finish saves. The "Fighter" row at the top opens a saved fighter to edit (or "New fighter"); Delete removes it.

The stats are 1 to 9, where 5 is the class exactly as it ships. They change how the fighter plays, through one function in Rust
(`sim-content/src/recipe.rs`), so the same recipe is the same fighter on any machine:

| Stat | What it does |
| --- | --- |
| **Size** | Body size from 68% to 132%: the hurtbox and ledge reach scale; bigger is **heavier, slower, jumps lower and falls faster**; smaller is quick and light |
| **Speed** | Walk, run, dash and air speed and how snappy the accelerations are (a little lighter at the top end) |
| **Jump** | How high jumps go |
| **Weight** | How hard the fighter is to launch (and a little faster to fall) |

The "How it plays" bars show the result (run speed, jump height, weight, fall speed, size) as you change things; the preview
fighter is drawn at the chosen size. Tests prove the promises in the real simulation: bigger runs less far and jumps lower at
every step from size 1 to 9, the speed and jump stats do what they say, the neutral recipe is exactly the archetype, and
**every one of the 13,122 possible recipes is valid content** (`sim-content/tests/recipes.rs`).

Saved fighters are small JSON files in `user://characters/` (name, class, four stats and the look code); a damaged file is ignored.
Names are 1 to 24 letters, digits, spaces, `-` and `_`, cannot be a built-in fighter's name, and cannot repeat.

## Character select

A grid of round portraits: the two built-in fighters, every fighter you saved, a **?** (random) and a **+** (make a new one). Each
player has a big panel with the fighter at its real size. **Player 1:** W A S D move, J locks in, K takes it back. **Player 2:** arrow
keys, Enter locks in, Backspace takes it back. **Mouse:** left click picks for player 1, right click for player 2. **E** edits the
saved fighter player 1 is on. When both are locked in, Space (or Enter, or the Start Battle button) starts the match; Esc in a match
returns to select, Esc in select returns to the menu.

## How a match gets its fighters

`Roster.build_content` (`godot/scripts/roster.gd`) starts from `content/base.pfc` and adds each created fighter in the match as a
`fighter` section made from its recipe by the bridge (`ContentEditor.derive_fighter`), then the game loads that bundle. Fighter
names appear above the fighters in the match, and each fighter is drawn at its size.

## Not done (honest list)

* **Online play uses the built-in fighters only.** Two players' created fighters would make two different contents, and the
  handshake refuses a mismatch. The recipe is a handful of bytes and deterministic, so it can travel in the handshake and both sides
  can build the same content; that is real netplay work and belongs with the Phase 7 online flow.
* Menus are keyboard and mouse. There is no controller navigation yet, and no key rebinding.
* Moves do not scale with size (hitbox sizes belong to the moveset); only the body, the physics and the ledge reach do.
* There is no balance guardrail yet: any mix of stats is allowed (plan 7.4 suggests a point budget for ranked play and free
  editing in casual lobbies; the stat ranges are bounded so nothing breaks, but a 9-speed, 9-jump, 1-size fighter is a strong one).
* The character art is the placeholder blob. Real models come later; the menus only show whatever the fighter view draws.
* One stage and one mode (versus). Stage select, stocks and match rules, results and rematch are Phase 7.
* Not tried by a person yet: whether the screens feel good to use is unproven until someone plays with them.

## Tests

* `sim-content/tests/recipes.rs`: validity of every recipe and the movement promises in the real sim.
* `godot/tests/creator_flow_test.gd`: name rules, saving and reloading, damaged files, assembling a match and checking the
  fighters' speed and jump height, then driving the creator, select and menu screens with key presses.
