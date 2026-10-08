# Editors (Phase 6)

Open them with `play_editor.bat` (or `Godot --path godot res://editor.tscn`). They are built inside Godot on the same
simulation the game runs, so what you see is what plays. The tabs:

| Tab | What it edits |
| --- | --- |
| **Look** | Each player's cosmetic loadout: body colour, face, hat, glasses, neckwear, accent colour. Save, random, reset, share codes. Cosmetic only. |
| **Fighter** | Every physics number of a fighter, grouped (body, ground, jumps, air, dodges, ledges). "New fighter (copy)" makes a fighter that `inherit`s the one on screen and stores only what you change. Choose its moveset. |
| **Moves** | A moveset's move slots: timing and flags, hitboxes (a table), motion, projectile, reflector, counter, and scripts. A timeline shows when each hitbox is active; the side view draws the active hitboxes at their real size and position over the fighter's body. "New moveset (copy)" makes one that inherits every move; "Edit a copy here" overrides one. |
| **Stage** | A side view you can drag: platforms (move, resize), ledges, spawn points; add or delete platforms, solid blocks and ledges; the blast zone; the stage name. |
| **Package** | The bundle's name, author and description; open, save, save packed (hash and sim version), start from the built-in roster, and **Playtest in game**. |

Undo and Redo (top right) cover every edit in every tab.

## How it works

The editors have no rules of their own. A `ContentEditor` (Rust, `godot-bridge/src/editor.rs`, on `sim-content/src/doc.rs`)
holds the bundle as a tree. A tab reads one section as a tree of dictionaries, changes some text values, and puts the section
back; Rust then reads and validates the **whole** document with exactly the code the game uses to load a bundle, and the
problems list shows what it says (zero gravity, a hitbox outside its move, a broken script, a missing name, ...). The game's
view of the content (`SimRunner.load_content_text`) is refreshed after every valid edit. So the editors accept what the game
accepts and nothing else, and a new field or rule added to the format shows up in them without new editor code (parameters
are listed from `sim-content`, moves from `MoveId`).

## Making and playing a custom fighter and stage

1. Open the editors. **Fighter** tab: type a name, press "New fighter (copy)", change some numbers.
2. **Moves** tab: pick a moveset, "New moveset (copy)", select a move, "Edit a copy here", change a hitbox or write a script.
   Back in the Fighter tab choose the new moveset for the new fighter.
3. **Stage** tab: drag platforms, add a ledge, move the spawn points.
4. **Package** tab: name it, **Save** (a loose bundle you can keep editing, by hand too) or **Save packed**.
5. **Playtest in game** starts the game with this content: player 1 is the fighter selected in the Fighter tab (WASD), player 2
   the first other fighter (arrow keys). Or run the game yourself: `Godot --path godot -- --content=<file> --chars=2,0`.

Both players of an online match must have the same bundle (the handshake refuses a mismatch and shows both hashes).

## Looks (loadouts)

A look is seven bytes: a version and six catalog indices (`godot/scripts/loadout.gd`). It is saved per player slot in
`user://loadout_p1.dat` / `_p2.dat`, loaded by the game at start-up, and sent in the network handshake, where the other player's
game rebuilds its view of you. It never reaches the simulation or its checksum. A value the receiving game does not know falls
back to the default for that slot, so a newer or missing part never blocks a match. Budgets are enforced by
`godot/tests/loadout_test.gd` (each catalog item adds 1 to 12 meshes; a fully dressed fighter stays within 30 extra).
Faces: deadpan, sleepy, grumpy, smug, plus a hurt face shown while being hit (a cosmetic event).

## Tests

* `sim-content/src/doc.rs` unit tests: sections read, replaced, added, removed; bad edits reported and fixable; text round trip.
* `godot/tests/editor_api_test.gd`: the whole creator path through the bridge, no UI: change a value, get told about a mistake,
  undo, add a fighter that inherits, add a platform to the stage, save, load in the game, check the new fighter really runs faster.
* `godot/tests/editor_ui_test.gd`: drives the real editor scene through the functions the text boxes and buttons call, including
  a synthetic mouse drag of a platform, copying a moveset and overriding a move, a hand-written script (and a broken one),
  save and reopen, and undoing the whole history.
* `godot/tests/loadout_test.gd`: look format, fallbacks, mesh budgets, the handshake carrying both looks.

## Not done (honest list)

* **No one has used these as a creator yet.** The tests prove the paths work; whether the screens are pleasant and clear is
  unproven until a person tries them.
* The move editor shows a timeline and a static side view for the selected frame. It does not play the move on a model, and
  hitboxes are edited in the table, not dragged in the side view (the stage editor does drag).
* Motion, projectile and reflector values are text boxes without graphical handles; scripts are a plain text box (errors appear in the
  problems list, with the script's own line numbers).
* Hurtboxes are drawn as the fighter's collision box (the ECB); the sim's hitbox-versus-hurtbox test uses body circles derived from it.
* A fighter or moveset cannot be renamed or reordered from the UI, and deleting something that others inherit from is reported as an
  error rather than prevented.
* One stage per bundle; there is no stage select, and fighter choice for a playtest is by launch argument (menus are Phase 7).
* Cosmetics are procedural shapes (hats, glasses, neckwear, faces). Real models, imported meshes and a rig with sockets are a
  later extension; the loadout format is catalog indices, so model-based parts can slot in without changing it or the handshake.
* The editors are keyboard-and-mouse only, and untested on a controller or at other window sizes than the ones used for the screenshots.
