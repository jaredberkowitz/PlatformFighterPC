# Menus, the character creator and character select

The game now starts at a **main menu** (`godot/menu.tscn`): **Play** (character select), **Character Creator**, **Editors** (the
content editors from `docs/EDITORS.md`) and **Quit**. Up and down (W/S) choose, Enter (J, Space) picks, Esc quits.
Launching with `--demo`, `--host`, `--join`, `--relay`, `--content`, `--chars` or `--shots` skips the menus and starts the match
directly, so `play_host.bat`, `play_join.bat`, demos and test scripts work as before.

Look and feel: sleek but cartoony, in the spirit of the two reference screens (a round-portrait select grid with big player
panels; a creator with arrow selectors, slanted label tabs, a name field and a preview on a little stage). All of it is original:
painted cards and brush strokes, round arrow buttons, gold highlights, a fantasy sky of floating islands drawn in code, bold
condensed type, and a slight lift on hovered buttons (see "Look" below). The pieces live in `godot/ui/ui_kit.gd`, `menu_art.gd` and `ui_draw.gd`.

## Character creator

Choose a **class** (the moveset: Longsword or Claws), a **look** (face, hat, glasses, neckwear, body colour, accent colour: cosmetic
only) and four **stats**, give it a **name**, and press **Finish**. Up/Down pick a row, Left/Right change it, Enter on the name
or Finish saves. The "Fighter" row at the top opens a saved fighter to edit (or "New fighter"); Delete removes it.

The stats are 1 to 9, where 5 is the class exactly as it ships. They change how the fighter plays, through one function in Rust
(`sim-content/src/recipe.rs`), so the same recipe is the same fighter on any machine:

| Stat | What it does |
| --- | --- |
| **Size** | Body size from 68% to 132%: the hurtbox, ledge reach and **attack size** scale; bigger is **heavier, slower, jumps lower and falls faster** but hits bigger and reaches further; smaller is quick and light with tighter attacks |
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

A grid of round portraits (**Players: 2/3/4**, see `docs/MATCHES.md`): the two built-in fighters, every fighter you saved, a **?** (random) and a **+** (make a new one). Each
player has a big panel with the fighter at its real size. **Player 1:** W A S D move, J locks in, K takes it back. **Player 2:** arrow
keys, Enter locks in, Backspace takes it back. **Mouse:** left click picks for player 1, right click for player 2. **E** edits the
saved fighter player 1 is on. When both are locked in, Space (or Enter, or the Start Battle button) starts the match; Esc in a match
returns to select, Esc in select returns to the menu.

## How a match gets its fighters

`Roster.build_content` (`godot/scripts/roster.gd`) starts from `content/base.pfc` and adds each created fighter in the match as a
`fighter` section made from its recipe by the bridge (`ContentEditor.derive_fighter`), then the game loads that bundle. Fighter
names appear above the fighters in the match, and each fighter is drawn at its size.

## Point budget and ranked rules

Each stat is worth its value, so the four stats of a fighter spend at most **20 points** (the neutral fighter, 5 + 5 + 5 + 5,
spends exactly 20). To raise one stat you lower another. The limit is defined once, in Rust (`sim-content/src/recipe.rs`, `BUDGET`,
`Recipe::is_legal`), and the menus ask the bridge for it.

* **Creator:** the **Rules** row chooses **Ranked** (the budget applies: arrows refuse to raise a stat when the points are spent, and
  Finish refuses an over-budget fighter) or **Casual** (no limit; the fighter is saved as casual). A gold "Points 20 / 20" tag
  shows the spend.
* **Select:** the **Rules** button (or **R**) switches ranked rules on. Fighters over the budget carry a CASUAL tag and cannot be
  locked in while ranked rules are on.
* **Online:** `--ranked` on the host makes the handshake refuse any fighter over the budget, whichever side brings it.

## Hitbox size scaling

A fighter's attacks scale with its body: `FighterParams::hitbox_scale` (set from the size stat, from 0.68 to 1.32; exactly 1.0 for
the neutral fighter) multiplies the size and position of every hitbox, the muzzle position and size of projectiles, and the
reflector. A big fighter's attacks are bigger and reach further; a small one's are tighter. Damage and knockback do not change.
The weapon drawn in the match follows the scaled hitboxes. Tests: `sim-content/tests/recipes.rs` (a jab's reach rises with size at
sizes 1, 5 and 9; hitboxes and projectile muzzles scale exactly; scale 1.0 at the neutral size).

## Online play with made fighters

Each player brings a **fighter spec**, a few bytes: `[0, n]` for built-in fighter n, or `[1, class, size, speed, jump, weight]` for
a made one. The specs travel in the network handshake; both sides build the same match from them (the base roster plus both made
fighters, `recipe::match_content`) and play on it, so created fighters work online with nothing to install or share. The joiner
checks the host's fighter numbers against its own working-out and refuses a mismatch; a malformed or unknown spec, or an over-budget one
under ranked rules, is refused by both sides with a message. The fighter's **name and look** travel separately as cosmetics (they never
reach the simulation). Details: `docs/NETPLAY.md`.

To play online with a made fighter, make it in the creator, then either play one local match with it first (the menus remember it as
"last played") or pass `--fighter=<name>` to the launcher: for example
`Godot --path godot -- --host=47000 --fighter=big_bertha` and `Godot --path godot -- --join=IP:47000 --fighter=tiny_tim`.

## Controls and controllers

* **Main menu, Controls** (`godot/ui/controls.gd`) lists both players' keyboard keys and rebinds them: Up/Down choose an action,
  Left/Right switch player, **Enter** then the new key rebinds, **Delete** resets that player's keys, Esc goes back. A key already used
  by the same player's other action swaps with it, so no key is ever on two actions; Esc and the F-keys the game uses stay reserved.
  Saved in `user://controls.json` (`godot/scripts/bindings.gd`); a damaged or hand-edited file is repaired. The sim never sees keys.
* **Controllers** play with: left stick or D-pad to move, A or Y jump, X attack, B special, bumpers and triggers shield, right-stick
  click grab, and the **right stick smashes** (a strong attack in the direction pushed). Controller 1 is player 1, controller 2 player 2,
  alongside the keyboards. (Not tested with real hardware: no controller was available.)
* **Controllers in the menus** (`godot/scripts/pad_nav.gd`): D-pad or left stick moves (with key repeat), A confirms, B goes back,
  Start starts, Back/Select leaves; it works by pressing the keys the screens already understand. In character select controller 1 plays
  the player 1 keys and controller 2 the player 2 keys. During a match the controller plays, so it only navigates the results screen and
  replays. Text fields (names, addresses) still need a keyboard.

## Not done (honest list)

* **The online screen is a form, not a lobby.** Online play has its own screen now (main menu, Online: see `docs/MATCHES.md`), but there
  is no list of games: friends type an address (or a relay and a room number), or choose Quick match with a relay's address.
* Controller menu navigation and the new bindings have not been tried with a real controller. Text entry needs a keyboard.
* The point budget is a plain sum of the four stats; it does not weigh them differently, and nothing yet checks whether a legal
  build is balanced (a tiny, fast, high-jumping, featherweight fighter is legal and strong in some ways). Real balance tooling is Phase 8.
* The character art is the placeholder blob. Real models come later; the menus show whatever the fighter view draws.
* One stage and one mode (versus). Stage select is still to do (stocks, results and rematch are done: `docs/MATCHES.md`).
* Not tried by a person yet: whether the screens feel good to use is unproven until someone plays with them.

## Tests

* `sim-content/tests/recipes.rs`: validity of every recipe and the movement promises in the real sim.
* `godot/tests/online_fighters_test.gd`: two game instances over real UDP bring made fighters (and built-in ones), build the same
  match, see the same sizes, names and hashes; ranked rules refuse over-budget fighters from either side; the budget in the creator and select screens.
* `netplay/tests/fighters.rs` and the `net-fuzz` command: the handshake and match are frame-for-frame exact with made fighters.
* `godot/tests/creator_flow_test.gd`: name rules, saving and reloading, damaged files, assembling a match and checking the
  fighters' speed and jump height, then driving the creator, select and menu screens with key presses.


## Look (third pass: the painted style)

The menus follow the concept art: a sunny fantasy world drawn entirely in code (`godot/ui/menu_art.gd`), with no images. Pieces:

- **The sky** (`Art.SkyScene`): a blue gradient with the sun's glow and a halo of rings, floating islands with grass, waterfalls and
  castles (pale ones far off), banks of cream clouds, birds, motes and sparkles, leaves in the bottom corners. A `mood` warms it
  (player 1's coral and gold) or cools it (player 2's blue). Each piece is drawn once and only moved after (islands bob, clouds drift);
  only the birds and motes are redrawn, which keeps the menus cheap (repainting the whole sky each frame cost about 15 ms).
- **Painted cards** (`Art.card`): cut corners, an offset ink shadow, a fill lighter at the top, a broad translucent plane and an ink
  outline. Buttons, labels, menu tiles and the roster cards are all cards; buttons and tiles repaint only when they change.
- **Brush strokes** (`Art.brush_band`): ragged-ended bands with streaks, for titles, names, player tabs, READY badges and the ribbon.
- **Titles** (`Art.title`): live text with an ink outline and a soft shadow. **Key glyphs** (`UI.KeyBar`): the controls along the bottom.

The screens:

- **Main menu**: the name on two brush strokes; a mosaic of tiles (Play, Online, Replays, Character Creator, Controls, Editors, Quit),
  each a card in its own colour with a line-drawn icon; Play is the biggest, a painted scene with the last fighter played standing on a
  ledge in it. Up and down step through the tiles in order, left and right move across. Every screen opens with a slanted wipe
  (`UI.reveal`).
- **Character select**: "VERSUS" and "CHOOSE YOUR FIGHTER" on brush strokes; roster cards in a fixed run of colours with each fighter's
  own 3D portrait (rendered once, then kept as a still), the name on an ink stroke, outlines and P tabs in the colours of the players on
  them (the card lifts); the players' panels along the bottom, each a painted scene (warm for player 1, cool for player 2) with the
  fighter standing on a ledge, the name on a stroke in the player's colour and a gold READY! badge once locked in; a coral "READY TO
  FIGHT!" ribbon across the screen once everyone is in; the keys along the bottom.
- **Results**: the winner standing large on their colour, a card for each fighter in finishing order with knock-outs, falls, damage dealt
  and taken and stocks left (tracked by the match scene from hits and knock-outs; with more than two fighters the attacker is the one frozen
  in an attack on the same frame), and the choices.
