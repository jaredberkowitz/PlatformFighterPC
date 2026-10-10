# Audio

Sound effects (`godot/scripts/sfx.gd`) are of two kinds. Most are **recorded sounds** from two free packs by Kenney, released under
**CC0** (public domain, no attribution required; credited in `docs/THIRD_PARTY.md` anyway), in `godot/audio/sfx/`. The rest are **synthesised at
start-up** from noise and sine waves. All are cosmetic: triggered from what the simulation reports, never feeding back into it.

Recorded groups (`FILE_SOUNDS`; one take is picked at random each time, with a slight random pitch so repeats do not sound mechanical):

| Group | Files (Kenney) | Used for |
| --- | --- | --- |
| `punch`, `punch_heavy` | impactPunch_medium, impactPunch_heavy | a blow landing (light, heavy) |
| `slash` + `metal`, `metal_heavy` | knifeSlice, impactMetal_light / heavy | a blade landing: the slice and a ring |
| `wood_heavy` | impactWood_heavy | the maul landing (with a heavy punch under it) |
| `thud` | impactSoft_heavy | the low body of any heavy hit |
| `glass` | impactGlass_light | a hit on a shield |
| `shing` | drawKnife | a blade's swing |
| `cloth` | cloth | a jump |
| `step` | footstep_grass, footstep_concrete | a landing |

Hits are layered by **kind and size** (`hit(kind, strength)`, called by the match on every hit, flinches and each hit of a multi-hit included):
a blade slices and rings, a blow smacks, the maul thumps, and a heavy hit (strength above 0.5, from the hitlag) adds the `thud` and the
synthesised `hit_heavy` boom. Grabs, throws and pummels of the weapon classes sound like blows. A clank plays `metal_heavy`.

Synthesised: `whoosh` (a swing, also under the blade's `shing`), `jump` (under the cloth), `shield`, `ko`, `hit_heavy`, `blip` / `confirm` (menus),
`go`.

* **The swing is heard as the move swings**: four frames before its first active frame (`move_timing`), not when the move starts.
* `Sfx.of(node)` adds the sound node to the tree once; `watch(player, before, now)` turns two snapshots of a fighter into swing, jump, landing,
  shield and KO sounds (hits and blocks are played by the match, which knows who hit whom).
* **F4** mutes in a match. Eighteen voices play at once, oldest replaced first.
* Tests: `godot/tests/sfx_test.gd` (every synthesised effect is made, audible, not clipping, a sensible length; every recorded take loads; the
  swing lands on its frame; hits are layered by kind and size).

## Music

`godot/scripts/music.gd` composes two looping tracks in code, with no audio files: **menu** (96 bpm, C major, a soft arpeggio and pad over a gentle
bass) and **battle** (146 bpm, A minor, a driving bass, a fast arpeggio and drums). Each is four chords of one bar. It renders on a worker
thread the first time (about 0.4 s) and loops seamlessly. The menus play the menu theme and the match scene the battle theme (the node lives on
the tree's root, so it keeps playing across screen changes and asking for the track already playing does nothing). F4 mutes it with the effects;
`volume_db` is the only volume control. Headless runs skip it. Test: `godot/tests/music_test.gd` (length, looping without a click, level, the player).

It has not been heard by a person: the tracks are checked for loudness and loop points, not for whether they are pleasant. Expect to tune the
notes, mix and tempo by ear, or replace them with composed music later.

Not done: a volume setting menu, per-character voices, crowd reactions, stage ambience, grabs, ledge grabs and the countdown (`go`), 3D
positioning, and mixing by ear (the levels are first guesses).
