# What data the game handles

This is the factual input for a privacy policy and terms of service (the Phase 8 "legal" items in `docs/ALPHA.md`). It describes what the code does
today; it is **not** a privacy policy and not legal advice. Re-check it whenever networking, saving or content sharing changes.

## Summary

The game has **no accounts, no analytics, no telemetry, no advertising and no crash reporting**, and contacts no server of its own. The only network
traffic is the match itself (to the other players) and, if a player chooses, to a relay or spectators they or a friend run.

## Stored on the player's computer (Godot's `user://` folder)

| What | Where in the code | Contents |
| --- | --- | --- |
| Created fighters | `roster.gd` | name, class, four stats, cosmetic look |
| Controls and key bindings | `bindings.gd` | key and button choices |
| Match rules chosen on the Online screen, last fighter played | `roster.gd` | stocks, time, stage, ranked flag, a slug |
| Replays | `replays.gd` | a match: seed, rules, stage, the fighters' specs, names and looks, every player's inputs |
| Content bundles saved from the editors | `godot/editor/` | the game data the player made |

Nothing is uploaded. Deleting the `user://` folder removes it all.

## Sent to other players

In an online match each player's game sends to the others: the fighter's recipe (class and stats) or built-in index, the **display name** (up to 24
characters), the cosmetic look, the game version, and the controller inputs every frame. Names from other players are cleaned before they are shown
(`docs/CONTENT_POLICY.md`).

* **Direct play** exposes each player's **IP address** to the other (UDP). Port forwarding opens a port on the host's router.
* A **relay** (`pftool net-relay`, run by whoever publishes one) sees every player's IP address and port, the room number and the traffic of the room
  (it forwards without reading it, but it could). The relay keeps no logs of its own beyond a counter, and does no authentication.
* **Quick match** (the relay's queue) additionally holds a waiting player's IP address and port in memory for six seconds after their last request.
* **Spectators** a host allows receive the match's inputs, names and looks. Anyone who knows the host's address (port + 1) can ask to watch.
* A **group match** routes every guest's traffic through the host, so the host sees all guests' IP addresses.
* **Replays** shared by a player contain the other player's name and look as they were in the match.

## Not collected

Real names, email addresses, ages, locations, device identifiers, payment details, voice or text chat (there is none), or anything from outside the game.

## Things a publisher still has to decide

* Where a relay runs, who runs it, how long (if at all) it logs, and what it tells players.
* A privacy policy and terms of service (what players may name fighters, what happens on a report), an age rating, and a contact for reports.
* Reporting and blocking for players and shared content: there is none, because it needs an account or server to act on. The name filter
  (`docs/CONTENT_POLICY.md`) is the only moderation.
* Children's privacy rules apply if players under 13 (US) or 16 (parts of the EU) are expected; the game collects nothing, but IP addresses are
  personal data under some laws.
