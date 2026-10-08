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

## Free-for-alls (3 to 4 players, local)

Character select has a **Players** button (or **N**): 2, 3 or 4. Two players get the big panels; three or four get four small ones in a
square. Players 1 and 2 use the keyboard or controllers 1 and 2 as before; **players 3 and 4 need controllers 3 and 4** (the keyboard has
only two key sets). Every controller moves its own cursor in character select (`pad_action` in `godot/ui/select.gd`, fed by `pad_nav.gd`),
and the match starts once every player has locked in a fighter.

In the match the HUD shows a card per player, the camera follows whoever is still in, the results screen has a card each, and the match
is recorded and replayed like any other (the record stores only the inputs of the players who took part). The sim already handled four
fighters and `GameState::roster`/`winner` count however many started; a match ends when one is left.
`SimRunner.load_match_roster(specs)` builds the match content for two to four fighters. Tests: `godot/tests/match_flow_test.gd`
(a four-player match scene: one out, the match goes on; the last standing wins; four result cards), `godot/tests/replay_test.gd`
(a four-player replay verifies), `netplay/src/replay.rs` (four and three players).

Not done: online matches are still two players (the handshake and session are 1v1); no teams; the select screen has no per-player
handicap or colour choice.

## Group matches (3 or 4 players online)

On the Online screen set **Mode: Group (3-4)** (everyone must pick the same mode; direct connections only for now). The host opens a lobby, guests join
(each sends its fighter, name and look), and the host presses **Enter** to start with whoever has joined. A rematch (the host's Rematch button) starts
another match with the same players. Up to three guests; spectators can watch a group match like any other.

How it works (`netplay/src/group.rs`): the host is a **hub**. Every guest has one link to the host and nothing else; every rollback packet (inputs, a
checksum, a goodbye) travels in a small envelope that names its destination, and the host handles what is for it and forwards the rest. The sessions are
the ordinary rollback `Session` with one packet per remote player (each carries that player's own acknowledgement; `Session::drain_outgoing_to` says who each is
for), so a match of four is exactly as exact as a duel. The handshake: `Hello` (version, roster hash, fighter spec, name) until `Welcome` (slot 1 to 3 and who is in the
lobby); at the start the host builds the match content from everyone's fighters and the stage and sends each guest a `Setup` (seed, rules, who is playing, every
fighter and name, the guest's slot) which the guest checks (it works out the fighter numbers itself and insists on the host's) and answers `Ready`; every envelope carries an
epoch so a finished match never leaks into the next. A guest who leaves is noticed by the others (a goodbye, or ten seconds of silence) and the match goes on without them.

* Sockets: `transport::SpectatorSocket` is also the host's hub socket (`HubLink`). Bridge: `group_host_start`, `group_join`, `group_start_match`, `group_update`,
  `group_restart`, `group_lobby`, `group_slot`, `group_players`. The match scene has a lobby mode (`_group_step`).
* Tests: `netplay/tests/group.rs` (lobby; refusals for another version, an unreadable fighter and a ranked cheat; a four-player match whose confirmed frames equal a
  single machine's over 12% loss, reordering and duplicates, for six seeds; a guest leaving; a rematch; 40 randomised conditions), `tools/tests/group_udp.rs` (four players over
  loopback UDP), `godot/tests/online_flow_test.gd` (three players through the bridge: lobby, match to the clock, the same result and positions everywhere, a replay that verifies,
  a rematch).
* Costs: the guests' packets to each other take two hops through the host, so a guest-to-guest path is as slow as the two links added. Duels stay peer to peer.

Not done: group matches through the relay (a room holds two), choosing teams or colours in the lobby, kicking a player from the lobby, a lobby chat, and dropping a
player in the middle of a match and letting a new one in. Real-internet group play is untested like the rest of the netcode.

## Spectating

On the Online screen choose **Role: Watch** and type the host's address (the port the players use). The host's game also listens on the **next
port** (hosting on 47000 means spectators connect to 47001, so that UDP port must be reachable too); up to 8 people can watch one match.

How it works (`netplay/src/spectate.rs`): the host already keeps a `MatchRecord` of the match (seed, setup, fighters, every *confirmed* frame's
inputs). A `SpectatorServer` streams that record to each spectator in chunks of 30 frames with a window of 180 unacknowledged frames and
resends anything not acknowledged; a `SpectatorClient` rebuilds the match from the record's header (same checks as a replay: sim version and
base roster) and steps the simulation as frames arrive. Because the simulation is deterministic the spectator needs nothing else: no rollback,
no state transfer, and a **late joiner** simply receives every frame from the start and catches up (it plays 1, 3 or 8 frames per tick when it
is 10, 40 or 180 frames behind, and stays about 10 frames behind the live game so the stream never runs dry). A rematch (a new seed) replaces the
match on the spectator's screen. The spectator's HUD, stage, fighters and names are the host's.

* Sockets: `transport::SpectatorSocket` (many addresses on one socket); the bridge's `spectate_start`, `spectate_update`, `spectate_stop`,
  `spectator_count`; the match scene's spectate mode (`_spectate_step`).
* Safety: a spectator on another sim version or roster gets nothing; garbage is ignored; only 8 spectators are remembered.
* Tests: `netplay/src/spectate.rs` (same ending as the host, 25% loss, a late joiner, a rematch, made fighters on another stage, garbage),
  `tools/tests/spectate_udp.rs` (real UDP), `godot/tests/online_flow_test.gd` (a spectator joins a hosted match late and ends in the same place
  as the players).

Not done: spectating through the relay (relay rooms hold two players), a spectator list or "watch a friend" lobby, spectators chatting,
choosing which player to follow, and a delay setting for tournaments.

## Stages

Four stages (`sim-content/src/stages.rs`: Meadow, Triple Tier, Flat Island, Skyline; all original geometry). A stage is chosen by index when
a match is set up: character select's **Stage** button (or **G**), or the host on the Online screen. The match content is the base roster with
that stage swapped in (`recipe::match_content_on`), so the simulation still sees one stage and everything agrees through the content hash:

* **Online**: the host's stage is in the netplay `Setup` (wire change); both sides build the same content, the joiner takes the host's.
* **Replays** record the stage (format 2) and play on it; an unknown stage is refused.
* Stage 0 is the base roster's own stage and leaves the content untouched. A bundle's own stage is stage 0 (the text format holds one stage).

Tests: `sim-content/src/stages.rs` (every stage valid and distinct, everyone spawns standing, random play is safe on every stage),
`netplay/src/replay.rs` and `netplay/tests/rematch.rs` (the stage is recorded; the host's stage reaches both sides),
`godot/tests/replay_test.gd` (a match on another stage records and replays). Not done: a stage preview picture in the picker, stage
hazards, per-stage music or backdrops, stages in the text format.

## Replays

Every finished match is saved automatically (the newest 50 are kept) in `user://replays/` as a `.pfr` match record, and the main menu's
**Replays** screen lists and plays them (name versus name, who won, length, rules). While watching: **Space** pause, **Left/Right**
jump 5 seconds, **Up/Down** speed (0.25x to 4x), **R** restart, **,** and **.** step a frame while paused, **Esc** back to the list.
Watching works for local and online matches, with made fighters, on any machine with the same sim version.

A record (`netplay/src/replay.rs`, `MatchRecord`) is the seed, the setup (characters, rules), the two fighters' spec bytes, the
players' name-and-look bytes (opaque), every frame's inputs for the players who took part, and the winner and final checksum. The
simulation is deterministic, so that is the whole match. Rules of the format:

* **Verified**: `MatchRecord::verify` plays the record and demands the recorded winner and final checksum; `pftool replay-verify
  <file.pfr>` does it from the command line; the bridge's `replay_verify()` does it in the game. A changed input, version, base
  roster or fighter is caught.
* **Sealed**: when a match is saved, frames after the match was decided are dropped and the ending is computed from the replay itself,
  so a saved record always verifies.
* **Not recorded** when the match cannot be reproduced: it ran on custom content (not the base roster plus made fighters) or the
  match was edited with the training keys (F6 to F8, step back).
* Online, each side records the *confirmed* inputs of the session; the two files come out byte for byte identical
  (`godot/tests/online_flow_test.gd`).
* A replay from another sim version is listed but cannot be played ("older version").

Tests: `netplay/src/replay.rs` (round trip, made fighters, sealing, tampering, garbage), `godot/tests/replay_test.gd` (recording,
playback to the same result, seeking, tampered files, training edits, the replay mode of the match scene, the list screen),
`godot/tests/online_flow_test.gd`.

Not done: spectating a live match (the confirmed-input stream is what a spectator would consume), sharing replays between players
by file picker (copy the `.pfr` into the replay folder for now), slow-motion kill cams.

## Matches started from the menus

* They build their content with `SimRunner.load_match_fighters` (the same function an online match uses), so a local match and an
  online match with the same fighters are the same match.
* The training readout, hitbox drawings and collision outlines are off (F1, F3, F2 bring them back).

## Quick match

The Online screen's fourth role, **Quick match**, needs only a relay's address (`pftool net-relay <port>`, run by whoever is the "server"). Each player
asks the relay's queue (room 0, which typed rooms cannot use); when two are waiting the relay picks a fresh room (numbers from 2^40 up) and tells both:
the one who waited **hosts** (so its stage, stocks, time and ranked rules apply) and the other **joins**, and each then connects through that room
exactly like a typed relay match. Asking is repeated about twice a second; a client that stops asking leaves the queue after six seconds, and the queue holds at
most 256 waiting clients. Esc cancels. This is the whole of the matchmaking: no skill rating, no regions, no list of games, and the relay does no
authentication. Checks: `transport` unit tests (pairing, expiry, distinct rooms, typed rooms unharmed) and the quick-match part of
`godot/tests/online_flow_test.gd` (needs `target/debug/pftool.exe`, or `PFTOOL=<path>`; it skips otherwise).

## Known gaps

* Nobody has used the HUD, results screen or online screen by hand over a real network.
* No lobby with a list of games and no names for rooms: friends type an address (or a relay and a room number), or use **Quick match** (below).
* Only one stage; no sudden death; the results screen shows no per-fighter stats beyond stocks and damage.
