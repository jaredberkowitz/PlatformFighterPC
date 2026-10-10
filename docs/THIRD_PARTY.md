# Third-party software and licences

An inventory for the licence check in `docs/ALPHA.md`, generated from `Cargo.lock` with `cargo metadata` (so it lists what the Rust side links
to, including build-time crates). It is **not legal advice**: a publisher should have the licence texts shipped and the obligations read by someone
qualified. Regenerate it when `Cargo.lock` changes.

## What ships

| Component | Licence | Notes |
| --- | --- | --- |
| Godot Engine 4.7 | MIT | Ships in the export (the runtime and the export templates). Its own third-party notices (Godot's `COPYRIGHT.txt`, shown in the editor under Help, About) must ship with the game. |
| godot-rust (`godot`, `godot-core`, `godot-ffi` and related crates, the bridge to Godot) | **MPL-2.0** | File-level copyleft: if the game changes those crates' files, those changes must be published; using them unmodified only needs the notice and a way to get their source. |
| The Rust crates below | see the table | All permissive (MIT, Apache-2.0, BSD, Unlicense) apart from the MPL-2.0 ones above. |
| This project's own code and content | the repository's licence (`MIT OR Apache-2.0` in `Cargo.toml`) | Every asset is original (`docs/ART_DIRECTION.md`, `docs/CONTENT_POLICY.md`). |

## What does not ship but was used

* **Blender** (GPL) builds the models and animation (`art/blender/`). The tool's licence does not apply to the `.glb` files it produced.
* **Fonts**: the interface asks the operating system for Bahnschrift, Arial Narrow, Impact or Arial (`godot/ui/ui_draw.gd`) and falls back to Godot's
  built-in font. **No font file is bundled**, because those system fonts cannot be redistributed; on a machine without them the menus use the fallback.
  Choosing and shipping an open-licence font (for example one under the SIL Open Font License) is still to do.
* Music and some sound effects are synthesised in code (`docs/AUDIO.md`).

## Sound effects (ship in the export)

| Pack | Author | Licence | Files | Source |
| --- | --- | --- | --- | --- |
| Impact Sounds | Kenney (kenney.nl) | CC0 1.0 (public domain) | `godot/audio/sfx/impact*.ogg`, `footstep_*.ogg` | https://kenney.nl/assets/impact-sounds |
| RPG Audio | Kenney (kenney.nl) | CC0 1.0 (public domain) | `godot/audio/sfx/knifeSlice*.ogg`, `drawKnife*.ogg`, `cloth*.ogg` | https://kenney.nl/assets/rpg-audio |

CC0 needs no attribution; it is credited here as a courtesy and so the origin of every file is on record.

## Rust crates (81, outside this workspace)

Licence summary: 48 MIT OR Apache-2.0; 10 MIT; 8 MPL-2.0; 5 Unlicense OR MIT; 4 Apache-2.0 OR MIT; 3 Apache-2.0; 2 BSD-2-Clause OR Apache-2.0 OR MIT; 1 (MIT OR Apache-2.0) AND Unicode-3.0.

MPL-2.0 crates: gdextension-api 0.5.1, godot 0.5.5, godot-bindings 0.5.5, godot-cell 0.5.5, godot-codegen 0.5.5, godot-core 0.5.5, godot-ffi 0.5.5, godot-macros 0.5.5.

| Crate | Version | Licence |
| --- | --- | --- |
| aho-corasick | 1.1.5 | Unlicense OR MIT |
| anes | 0.1.6 | MIT OR Apache-2.0 |
| anstyle | 1.0.14 | MIT OR Apache-2.0 |
| autocfg | 1.5.1 | Apache-2.0 OR MIT |
| bumpalo | 3.20.3 | MIT OR Apache-2.0 |
| cast | 0.3.0 | MIT OR Apache-2.0 |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| ciborium | 0.2.2 | Apache-2.0 |
| ciborium-io | 0.2.2 | Apache-2.0 |
| ciborium-ll | 0.2.2 | Apache-2.0 |
| clap | 4.6.7 | MIT OR Apache-2.0 |
| clap_builder | 4.6.7 | MIT OR Apache-2.0 |
| clap_lex | 1.1.1 | MIT OR Apache-2.0 |
| criterion | 0.5.1 | Apache-2.0 OR MIT |
| criterion-plot | 0.5.0 | MIT OR Apache-2.0 |
| crossbeam-deque | 0.8.8 | MIT OR Apache-2.0 |
| crossbeam-epoch | 0.9.21 | MIT OR Apache-2.0 |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 |
| crunchy | 0.2.4 | MIT |
| either | 1.18.0 | MIT OR Apache-2.0 |
| futures-core | 0.3.34 | MIT OR Apache-2.0 |
| futures-task | 0.3.34 | MIT OR Apache-2.0 |
| futures-util | 0.3.34 | MIT OR Apache-2.0 |
| gdextension-api | 0.5.1 | MPL-2.0 |
| glam | 0.32.1 | MIT OR Apache-2.0 |
| godot | 0.5.5 | MPL-2.0 |
| godot-bindings | 0.5.5 | MPL-2.0 |
| godot-cell | 0.5.5 | MPL-2.0 |
| godot-codegen | 0.5.5 | MPL-2.0 |
| godot-core | 0.5.5 | MPL-2.0 |
| godot-ffi | 0.5.5 | MPL-2.0 |
| godot-macros | 0.5.5 | MPL-2.0 |
| half | 2.7.1 | MIT OR Apache-2.0 |
| heck | 0.5.0 | MIT OR Apache-2.0 |
| hermit-abi | 0.5.3 | MIT OR Apache-2.0 |
| is-terminal | 0.4.17 | MIT |
| itertools | 0.10.5 | MIT OR Apache-2.0 |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| js-sys | 0.3.106 | MIT OR Apache-2.0 |
| libc | 0.2.190 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| nanoserde | 0.2.1 | MIT OR Apache-2.0 |
| nanoserde-derive | 0.2.1 | MIT |
| num-traits | 0.2.19 | MIT OR Apache-2.0 |
| once_cell | 1.21.4 | MIT OR Apache-2.0 |
| oorandom | 11.1.5 | MIT |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT |
| plotters | 0.3.7 | MIT |
| plotters-backend | 0.3.7 | MIT |
| plotters-svg | 0.3.7 | MIT |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| rayon | 1.12.0 | MIT OR Apache-2.0 |
| rayon-core | 1.13.0 | MIT OR Apache-2.0 |
| regex | 1.13.1 | MIT OR Apache-2.0 |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 |
| rustversion | 1.0.23 | MIT OR Apache-2.0 |
| same-file | 1.0.6 | Unlicense OR MIT |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.151 | MIT OR Apache-2.0 |
| slab | 0.4.12 | MIT |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| syn | 3.0.6 | MIT OR Apache-2.0 |
| tinytemplate | 1.2.1 | Apache-2.0 OR MIT |
| unicode-ident | 1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| venial | 0.6.1 | MIT |
| walkdir | 2.5.0 | Unlicense OR MIT |
| wasm-bindgen | 0.2.129 | MIT OR Apache-2.0 |
| wasm-bindgen-macro | 0.2.129 | MIT OR Apache-2.0 |
| wasm-bindgen-macro-support | 0.2.129 | MIT OR Apache-2.0 |
| wasm-bindgen-shared | 0.2.129 | MIT OR Apache-2.0 |
| web-sys | 0.3.106 | MIT OR Apache-2.0 |
| winapi-util | 0.1.11 | Unlicense OR MIT |
| windows-link | 0.2.1 | MIT OR Apache-2.0 |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 |
| zerocopy | 0.8.60 | BSD-2-Clause OR Apache-2.0 OR MIT |
| zerocopy-derive | 0.8.60 | BSD-2-Clause OR Apache-2.0 OR MIT |
| zmij | 1.0.23 | MIT |
