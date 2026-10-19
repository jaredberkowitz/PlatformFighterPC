# Netplay (Phase 4)

Rollback netcode for two players. The design follows plan section 6: the sim is the only thing that decides what happens, every peer
runs it, and only *inputs* travel. A wrong guess about the other player's input is corrected by rolling back to that frame and
re-simulating.

## Layers

| Crate | Job | Rules |
| --- | --- | --- |
| `netplay` | wire format, handshake, rollback session, `Peer` glue, simulated lossy link | pure: no sockets, clocks or threads (like the sim crates) |
| `transport` | `UdpLink` (direct), `RelayLink` + `Relay` (fallback), `stun` + `code` + `punch` (connection codes) | the only place with sockets |
| `tools` (`pftool`) | `net-fuzz`, `net-host`, `net-join`, `net-relay`, `net-stun`, `net-code` | |
| `godot-bridge` | `net_host`, `net_join`, `net_host_code`, `net_join_code`, `net_update`, ... on `SimRunner`; `NetCode` | the game window uses these |

`netplay::peer::Link` is the seam: a UDP socket, a relay connection and the test link all implement it, so everything above the socket
behaves identically in tests and in play.

## How a match runs

1. **Handshake** (`handshake.rs`). The joiner sends its sim version and content hash; the host compares and refuses on a mismatch
   (answering with its own numbers so both sides can log it), otherwise sends the match settings: seed, characters, input delay.
   The joiner checks the host's numbers too and says ready. Every message is resent until answered. Cosmetic bytes travel along and
   never reach the sim.
2. **Session** (`session.rs`). Each frame the local input is read, applied `input_delay` frames later (default 2), and sent to the other
   peer together with every input it has not acknowledged, so a lost packet is repaired by the next one. For a remote input that has
   not arrived the session predicts "same as their last input" and keeps going.
3. **Rollback.** When a real remote input differs from the guess used for an already simulated frame, the next `advance` restores the
   snapshot from that frame and re-simulates to the present. Snapshots are plain copies of `GameState` (it is plain old data).
4. **Stall.** The sim never runs more than `max_prediction` (8) frames past the newest confirmed remote input. A longer outage makes
   the game wait ("waiting for the other player") instead of running away; nothing is simulated and no local input is consumed.
5. **Desync detection.** Every 30 frames the checksum of the *confirmed* state is exchanged (and re-sent, in case of loss). A mismatch
   raises `Event::Desync { frame, local, remote }` immediately, with the frame number.
6. **Disconnect.** 600 `advance` calls (10 s) without any packet from the other peer, or a goodbye packet, ends the connection.

## Playing by connection code (no port forwarding)

The default way to play a friend. Each player sees a short code on the Online screen, the two swap codes over Discord or any chat,
and both press Connect. No server of ours is involved and, for most home routers, nobody forwards a port.

1. **STUN** (`transport/src/stun.rs`). When the screen opens, `CodeSocket::bind` opens the UDP socket the match will use (port 47000,
   or any free port if that is taken) and sends RFC 5389 Binding Requests *from that socket* to public STUN servers
   (stun.l.google.com:19302, stun.cloudflare.com:3478 and a fallback list; names are resolved on a background thread). The
   XOR-MAPPED-ADDRESS of the answer is this machine's public IPv4 address and port. Requests are retried every 250 ms for up to 5 s;
   with no answer the code carries only the LAN address. Two servers answering with different public ports means a strict router
   (see below), and the screen warns about it up front. While the players swap codes a request every 15 s keeps the router's mapping
   alive. The LAN address is the system's route to the internet (no packet sent), so two players on one network connect locally.
2. **The code** (`transport/src/code.rs`). The public address, the LAN address and a 16-bit session nonce, packed into bits and
   written in Crockford base32 in dash-separated groups of four, for example `K7QD-M2XA-9FPE-0RTV-5BWC` (20 characters for a typical
   home player, up to 28). Reading ignores case, spaces and dashes and reads I/L as 1 and O as 0; a 16-bit checksum (which also
   covers the code format's version) refuses typos. The game version is still checked by the handshake (sim version and content hash).
3. **Who hosts.** The player with the lower nonce (then the lower code) hosts and their rules are used; both work it out the same way,
   so nobody chooses a role. The screen says "You will host" or "Your friend will host" once the friend's code is in.
4. **Hole punching** (`transport/src/punch.rs`). After Connect each side sends a 7-byte probe (`"HP"`, kind, sender's nonce,
   receiver's nonce) to the friend's LAN and public addresses every 100 ms. A probe going out opens the sender's router for answers from
   that address; once both sides are sending, probes get through and are answered with an acknowledgement. The first address to
   acknowledge wins. Probing gives up after 45 s (longer than the ~15 s the probes need, because two people rarely press Connect at
   the same moment). Probes carry both nonces, so a stranger's packet or one from an earlier session is ignored.
5. **The match** then runs the ordinary handshake and session over that same socket (`CodeLink`, a third `NetLink` in the bridge).
   `CodeLink` accepts datagrams only from addresses that proved themselves with the right nonces, answers late probes itself and drops
   them, and drops stray STUN answers, so nothing but netplay packets reaches the session. **The netplay wire format and session rules
   did not change** (no protocol or sim version bump); `pftool net-fuzz` is unchanged.

The online screen owns the socket (a `NetCode` object) from the moment it opens until the match scene takes the punched link
(`SimRunner.net_host_code` / `net_join_code`); the object travels in `Roster.session`. Choosing Direct or Relay closes it, so Direct can
still host on port 47000.

**Limits.** A strict router (symmetric NAT: a different public port for every destination, common on phone hotspots, some ISPs and
carrier-grade NAT) defeats this: the port STUN saw is not the one the friend must reach. If neither side can get through, the screen
says so and lists the fallbacks: one player forwards UDP 47000 and they use Direct; or both install a virtual-LAN app (Tailscale,
ZeroTier, Radmin VPN) and use Direct with the address it shows; or, on one network, Direct with the LAN address. IPv4 only. Windows
Firewall asks for permission the first time the game opens the socket; it must be allowed. Codes are not secret, but they only
matter for the session they were made for (the nonce changes each time the screen opens).

## Fighters in the handshake

The content hash both sides compare is the **base** roster. A player may bring a *fighter spec* (a few bytes from
`sim_content::recipe::FighterSpec`: a built-in fighter's index, or a recipe's class and four stats) in the hello (joiner) or the
setup (host). The host works out each player's fighter number from the two specs and sends it in the setup; the joiner works it
out itself and refuses the match if it differs from the host's. Both sides then build the same **match content** (the base roster
plus every made fighter, appended in player order) and run on it. `Peer::match_content()` exposes it so the game can draw the
fighters. The setup also carries a `ranked` flag: under ranked rules a recipe over the point budget (`recipe::BUDGET`, 20) is refused.
Every failure (undecodable bytes, an unknown built-in fighter or class, over budget, only one side bringing a spec, a host whose
numbers do not follow from the specs) rejects with `RejectReason::BadFighter` on both sides. With no specs the old behaviour is
unchanged: the characters in the settings are used on the base content. The wire format changed (a `fighter` field in `Hello`
and `Setup`, a `ranked` byte in `Setup`, reject reason 4), so builds from before this change cannot play against newer ones; the
sim version (22) refuses that anyway.

## Match rules and rematches

The host's setup also carries the match rules (`sim_core::MatchRules`: stocks and time limit); the joiner plays under them. After a
match both players can ask for a **rematch**: each side then runs a new handshake over the same link with a new seed. Every datagram
ends with a one-byte epoch (matches played on this link), so nothing from a finished match reaches the next one. Details and tests:
`docs/MATCHES.md`, `netplay/tests/rematch.rs`.

## The lobby and keeping one code (19 October 2026, the owner after a playtest)

Duels started from the online screen (by code, Direct or Relay) go through a **lobby** before every match (`Peer::with_lobby`,
`netplay/src/peer.rs`; the screen is `godot/ui/net_lobby.gd`). Each player chooses a fighter and locks it in, then a stage and locks it
in; both see the other's choices as they change. Each side sends its whole choice (`LobbyPick`: fighter spec, cosmetics, stage, two
locks) whenever it changes and every 10 ticks, so a lost datagram only delays it; the lobby datagram also carries the sender's sim
version and content hash, so two different builds find out at once (rejected before any handshake), and a player leaving says so
(reject reason 5, `Left`). When both have locked both, the host draws the stage (their common choice, else one of the two by the
match seed) and the ordinary handshake runs with the chosen fighters; the joiner plays on the host's draw. A **rematch** (both ask, as
before) goes back to the lobby with the last choices selected but not locked. Tests: `netplay/tests/lobby.rs` (six rounds of
rematches over six lossy links, the draw, leaving, a different build) and `godot/tests/online_flow_test.gd`.

**One code per launch.** The code's socket (`Roster.netcode`) is opened once and kept while the game is open, polled from every
screen (`godot/scripts/code_keeper.gd`) so its keepalive holds the router's mapping. A code match borrows it; leaving the match gives it
back (`SimRunner.net_leave_to_code`, `CodeLink::release`) with the same port, public address and nonce, so the code stays the same
and a friend's saved code keeps working until one of you restarts the game. (Choosing Direct or Relay closes it, so Direct can host on
the same port; the next code is then new.)

## What is tested

* `netplay/tests/session.rs`: handshake accept, refusal on a different version or content, survival at 50% packet loss; sessions over a
  link with latency, loss, duplication and reordering match a single-machine run **frame for frame**; a total outage stalls and then
  recovers; a corrupted state is reported as a desync at a checksum frame; a silent peer is dropped; garbage and stale packets are ignored.
* `netplay/src/packet.rs`: every packet round-trips; truncated or extended packets and 20,000 random byte strings never panic the parser.
* `transport`: UDP exchange, stranger ignored once the peer is known, relay forwarding and room isolation, garbage and a third client ignored.
  Codes: the STUN request bytes, the RFC 5769 sample response (the standard's own test vector) and hand-built XOR and plain mapped
  addresses, truncated and random responses; codes round-trip for every address kind, ignore case, spaces and look-alike letters, and
  every single-character typo is refused; hole punching over loopback with a fake STUN server (address found from the match socket,
  keepalives, a strict router detected from two servers, no answer leaves a LAN-only code, two sockets punch through, strangers, forged
  and stale probes and STUN never reach the session, a friend who never answers fails in time).
* `tools/tests/net_udp.rs`: two peers **connect by code** (fake STUN, codes passed as text, hole punched on loopback) while a stranger
  sprays junk and made-up probes at both, and agree on every confirmed frame.
* `pftool net-stun` asks the real public STUN servers from this machine and prints the public address and the code. Run on
  2026-10-17 from the development PC: it found the public IPv4 address (24.189.x.x) with the local port preserved, and two servers
  agreed on the port (not a strict router). `pftool net-code` plays a headless match by code across two machines.
* `godot/tests/online_flow_test.gd`: the Code link on the online screen (first and default, the player's code, refusing an empty, own
  or mistyped friend's code, who hosts), a whole match by code through the bridge, waiting, cancelling and failing, and the real
  online screen connecting by code and moving on to the match scene, which plays over the punched socket.
* `tools/tests/net_udp.rs`: two peers over **real UDP sockets** on localhost, directly and through the relay, agree on every confirmed frame.
* `pftool net-fuzz 2000`: randomised input delay, latency, loss, duplication and a mid-match total outage; every confirmed frame of
  both peers must equal a single-machine run of the same inputs. Current result: 2000 runs, 0 mismatches, about 105,000 rollbacks.
  The fighters are drawn from every class in the content (from the run's seed, on their own generator so the network schedule of a
  seed does not depend on them): even runs two random built-in fighters, odd runs a random built-in or made fighter each. The last
  line prints how many fighters of each class ran. `pftool fuzz-rollback` does the same for its four fighters (every run, each a
  built-in fighter a quarter of the time, otherwise made from random stats of a random class) and takes `--only <seed>` too.
* `godot/tests/net_e2e.gd`: two `SimRunner` nodes play each other over UDP through the bridge.
* `netplay/tests/fighters.rs`: made vs made, built-in vs made, over lossy links, **frame for frame equal to a single-machine run on the
  match content**; ranked refusals from either side; seven kinds of bad spec; a host that lies about fighter numbers.
  `tools/tests/net_udp.rs` repeats it over real UDP; `pftool net-fuzz` gives every other run random made fighters;
  `godot/tests/online_fighters_test.gd` does it through the bridge.

The fuzzer found one real bug in the session: the input ring (128 frames) was narrower than the window of frames accepted from
the network, so a delayed duplicate packet from long ago could reuse a slot that already held newer inputs. Frames are now only
accepted within half a ring either side of the present.

## Unused fighter slots

The sim has four fighter slots, a two-player match uses two. The other two are marked inactive
(`GameState::new_with_active`, the handshake's `active` mask): they are not updated, cannot be hit, grabbed or
targeted, and so cannot absorb projectiles or take damage unseen. Before this was added they stood invisibly on the stage at
their spawn points and could be hit; the mask is part of the checksum.

## What is not done (honest list)

* **Never tested over the real internet.** Everything above is localhost or simulated (only the STUN step has run against the real
  servers). Latency spikes, NAT behaviour and ISP quirks are exactly what Phase 4's exit criterion ("stable matches across real internet
  connections") needs a real test for. Hole punching between two different real networks has not been tried yet.
* **Codes fail behind strict routers** (symmetric NAT), and there is no relay of ours to fall back on automatically: the player is told
  to use Direct with a forwarded port or a virtual-LAN app. Quick match and Relay need someone to run `pftool net-relay` on a public
  server; there is no room list.
* Group matches and watching still need Direct (codes are for duels).
* **Two players only.** The session supports more (`active` mask), but only the 1v1 handshake and UI exist, as the plan says to validate 1v1 first.
* **No frame-rate sync.** Each peer runs at its own 60 Hz; the stall rule keeps them within a few frames, but a peer whose clock runs
  slower will make the other stall repeatedly rather than slowing both gently. Plan section 6's input-delay setting is a launch argument
  (`--delay=`), not a menu.
* **No spectators, replays of networked matches, or reconnects.**
* The relay does no authentication (anyone who knows a room number can join it) and is not rate limited beyond a room cap.
* A peer can lie about *its own* inputs freely (that is inherent), and the checksum protocol assumes both sides are honest.
* Packet size and frequency are fine on a LAN; on a bad link the redundancy (every unacknowledged input in each packet, up to 40) is
  the only loss protection.
