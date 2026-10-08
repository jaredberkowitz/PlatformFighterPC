# Content policy and moderation

The game is **original**: no one else's characters, names, art, music or franchise terms (see `CLAUDE.md` and the project plan). Players can
make fighters, stages and packs, and choose names other players will see, so there are rules and some tooling. This document says what is
enforced by code today and, honestly, what is not.

## The rules (what a pack or a name may contain)

1. **No trademarks or copyrighted characters** of other people (game franchises, studios, their character names). Fighters made by the
   point-budget creator, the editors or content packs must be original in name, look and moves.
2. **No hateful, harassing or sexual names or text.** (Enforced only by the blocklist below and by humans.)
3. **No personal information** in names or pack text.
4. **Cosmetics never change the game.** Looks, names and accessories are presentation only and never reach the simulation, so they cannot
   affect fairness.
5. **Packs cannot break the simulation:** every value is range-checked and scripts are sandboxed (integer only, a per-frame instruction
   budget, a whitelist of abilities). Ranked play also needs a fighter within the point budget.

## What the code enforces today

* **Name shape** (`sim-content/src/policy.rs`): 1 to 24 characters, letters, digits, space, `- _ ' .`, no leading, trailing or doubled spaces.
* **A blocklist**: the built-in list of other people's trademarks (a starting list, not exhaustive) plus the project's own
  `content/blocklist.txt` (one term per line, no recompiling). Matching ignores case, spacing, punctuation and look-alike characters
  (`M4r10` is caught as `mario`); terms under four letters only match a whole name so ordinary words are not caught by accident.
* **Where it applies**: names of fighters you make in the creator; fighter, weapon and stage names in content bundles (`sim_content::validate`,
  so the editors and `pftool content-check` refuse them); and every name that **arrives from another player** (online opponents, spectated
  matches, replay lists), which is cleaned before it is shown: odd or control characters dropped, spaces tidied, cut to 24 characters, and
  replaced by the default name if empty or blocked. A hostile client cannot put escape sequences or enormous text on your screen.
* **Online**: only a few bytes of fighter spec and a short profile cross the network, both length-limited and validated; there is no free text
  chat. Ranked rules refuse fighters over the point budget on either side.

## What is not done (and what a public service would need)

* **No reporting, no server-side moderation, no accounts.** There is no central server for content. Packs are shared by hand, so the people
  sharing them are the moderators. A public alpha with shared content needs reporting, review and the ability to remove content and ban
  accounts; this repository does not have any of that.
* **A blocklist is beatable.** It stops accidents and lazy abuse, not a determined person. The profanity side of the list is empty by default
  on purpose: add terms for your community to `content/blocklist.txt`.
* **Images and sounds** are not scanned: cosmetics are drawn from the built-in catalogue (hats, faces, colours), so players cannot upload art
  yet. If uploads are ever allowed they need their own review step.
* **No age or region policy**, no privacy policy text, no terms of service. Those are legal documents for whoever publishes the game.
* Whether the trademark list is long enough is a judgement call for the publisher; it should be reviewed with a lawyer before a public release.

## Tests

`sim-content/src/policy.rs` (shape, disguised terms, project terms, cleaning of remote names, nothing the cleaner returns fails the shape check),
`sim-content/src/lib.rs` (a content name with a trademark is refused), `godot/tests/policy_test.gd` (your fighter names and names arriving over the network).
