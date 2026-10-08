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

## Music

`godot/scripts/music.gd` composes two looping tracks in code, with no audio files: **menu** (96 bpm, C major, a soft arpeggio and pad over a gentle
bass) and **battle** (146 bpm, A minor, a driving bass, a fast arpeggio and drums). Each is four chords of one bar. It renders on a worker
thread the first time (about 0.4 s) and loops seamlessly. The menus play the menu theme and the match scene the battle theme (the node lives on
the tree's root, so it keeps playing across screen changes and asking for the track already playing does nothing). F4 mutes it with the effects;
`volume_db` is the only volume control. Headless runs skip it. Test: `godot/tests/music_test.gd` (length, looping without a click, level, the player).

It has not been heard by a person: the tracks are checked for loudness and loop points, not for whether they are pleasant. Expect to tune the
notes, mix and tempo by ear, or replace them with composed music later.

Not done: a volume setting menu, per-character voices, stage ambience, sound for blocking (`clank`), grabs, ledge grabs and the countdown
(`go`), 3D positioning, and judging how it sounds to a person (none of it has been heard by someone yet, and synthesised effects usually need
tuning by ear).
