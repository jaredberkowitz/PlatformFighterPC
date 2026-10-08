# Audio

Sound effects are **synthesised at start-up** (`godot/scripts/sfx.gd`): short bursts, sweeps and blips made from noise and sine waves, so there
are no audio files to license or ship and everything is original. They are cosmetic: triggered from what the simulation reports, never
feeding back into it.

| Effect | When |
| --- | --- |
| `whoosh` | an attack starts |
| `hit_light` / `hit_heavy` | a fighter takes damage (heavy from 11% in one hit, or a tumble) |
| `jump` | a jump squat starts, or a double jump |
| `land` | a fighter touches the ground (not out of hitstun) |
| `shield` | the shield goes up |
| `ko` | a stock is lost |
| `blip` / `confirm` | main menu selection and choice |
| `go`, `clank` | made and ready, not triggered yet |

* `Sfx.of(node)` adds the sound node to the tree once; `watch(player, before, now)` turns two snapshots of a fighter into sounds.
* **F4** mutes in a match. Ten voices play at once, oldest replaced first.
* Tests: `godot/tests/sfx_test.gd` (every effect is made, audible, not clipping, a sensible length; the watcher plays the right sounds).

Not done: music, a volume setting, per-character voices, stage ambience, sound for blocking (`clank`), grabs, ledge grabs and the countdown
(`go`), 3D positioning, and judging how it sounds to a person (none of it has been heard by someone yet, and synthesised effects usually need
tuning by ear).
