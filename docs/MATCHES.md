# Matches: stocks, winning, the HUD, results, rematches and the online screen (sim v23)

## Match rules (in the simulation)

`sim_core::MatchRules` (`sim-core/src/state.rs`) says how a match is won. It lives in `GameState` (so it is hashed) and is chosen per
match, unlike `Ruleset`, which is part of the roster's content:

| Field | Meaning |
| --- | --- |
| `stocks` | Stocks each fighter starts with, 1 to 9. **0 means unlimited**: nobody is ever eliminated (free play, training, demos). |
| `time_limit` | Seconds, up to 3600. 0 means no limit. |

`GameState::new_with_rules(content, seed, chars, active, rules)` builds a match; `new` / `new_with_active` use the default (3 stocks,
no limit). Other new `GameState` fields, all hashed: `roster` (which fighters started the match), `winner` (`PLAYING` = -1, a
fighter's index, or `DRAW` = -2).

* A fighter that leaves the blast zone loses a stock and respawns. When it loses its last, it is eliminated: `active` goes false, so
  it is not updated, hit or targeted any more (the same mechanism as the unused slots of a two-player match).
* After each frame `step` settles the match: if one fighter (or none) is left of at least two who started, the match is over; a
  winner of one, a `DRAW` if the last ones fell together. With a time limit, when the clock runs out the fighter with the most stocks
  wins, then the one with the least damage; level on both is a `DRAW`. A match with a single fighter never ends (practice).
* **A finished match stands still**: `step` only advances the frame counter once `winner` is set.
* Tests: `sim-core/tests/match_rules.rs` (elimination, winner, draw, unlimited, time limit, three players, random matches end and
  agree), the checksum-sees-every-field test in `state.rs`, `pftool net-fuzz` (it varies the rules per run).

Not done: sudden death after a tied clock, team matches, items.

## In the game (Godot)

* **Match HUD** (`godot/ui/match_hud.gd`): a card per fighter (name, damage percent that reddens as it grows, stocks as discs),
  the clock (counting down with a time limit, up otherwise, red in the last 10 seconds; hidden in free play), "GO!", "GAME!" and
  "TIME!" banners, and connection messages (connecting, waiting, refused, desync, the other player leaving). A match started from
  the menus hides the training readout; **F1** brings it back.
* A fighter that is out leaves the stage (and the camera stops following it).
* **Results screen** (`godot/ui/results.gd`) about two seconds after the end: who won, each fighter's stocks left and damage, and
  Rematch / Character Select / Main Menu (online: Rematch / Main Menu). Left/Right and Enter, or the mouse.
* **Character select** has Stocks and Time buttons (also **T** and **Y**), remembered between runs (`user://match_rules.json`).
  Launching with `--stocks=N --time=SECONDS` after `--` sets them for a direct launch. Direct launches without them are free play.

## The online screen (main menu, Online)

`godot/ui/online.gd`. Role (Host or Join), Connect (Direct or Relay), then the fields that apply: Address (join, or the relay's), Port
(hosting directly, default 47000), Room (relay), your Fighter (built-in or made; only a few bytes travel), Delay (0 to 6 frames, default
2) and, for the host, Stocks, Time and Rules (Casual or Ranked). The choices are remembered (`user://online.json`). **Host!/Join!**
starts the match scene, which connects and shows "Connecting..." until the other player arrives. Esc in the match goes back to
this screen. The old launch arguments (`--host`, `--join`, `--relay`, `--fighter`, `--ranked`, `--delay`) still work and skip the menus.

The joiner plays under the host's stocks, time and ranked rule (they travel in the handshake's setup).

## Rematches (online)

When the match is over either player can pick **Rematch**. The result screen says when the other player has asked; once both have,
each side starts a new handshake over the same connection (the host picks a new seed) and a new match begins, with the same fighters
and rules. `Peer::request_rematch`, `rematch_state`; the bridge's `net_request_rematch` and `net_rematch_state`.

Every datagram now ends with a one-byte *epoch* (how many matches this connection has played), so late packets of the finished match
can never reach the next one. Rematch requests are their own datagram type, resent until the other side answers.
Tests: `netplay/tests/rematch.rs` (both asking, one asking, a rough link with loss and reordering, ten seeds),
`godot/tests/online_flow_test.gd` (the whole thing through the bridge over UDP: the clock ends the first match, the joiner's rules are
the host's, a rematch starts, no desync).

## Known gaps

* Nobody has used the HUD, results screen or online screen by hand over a real network.
* No lobby with a list of games, no matchmaking, no names for rooms: friends type an address (or a relay and a room number).
* Only one stage; no sudden death; the results screen shows no per-fighter stats beyond stocks and damage.
