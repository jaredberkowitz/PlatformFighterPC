# Netplay (Phase 4)

Rollback netcode for two players. The design follows plan section 6: the sim is the only thing that decides what happens, every peer
runs it, and only *inputs* travel. A wrong guess about the other player's input is corrected by rolling back to that frame and
re-simulating.

## Layers

| Crate | Job | Rules |
| --- | --- | --- |
| `netplay` | wire format, handshake, rollback session, `Peer` glue, simulated lossy link | pure: no sockets, clocks or threads (like the sim crates) |
| `transport` | `UdpLink` (direct), `RelayLink` + `Relay` (fallback) | the only place with sockets |
| `tools` (`pftool`) | `net-fuzz`, `net-host`, `net-join`, `net-relay` | |
| `godot-bridge` | `net_host`, `net_join`, `net_update`, ... on `SimRunner` | the game window uses these |

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

## What is tested

* `netplay/tests/session.rs`: handshake accept, refusal on a different version or content, survival at 50% packet loss; sessions over a
  link with latency, loss, duplication and reordering match a single-machine run **frame for frame**; a total outage stalls and then
  recovers; a corrupted state is reported as a desync at a checksum frame; a silent peer is dropped; garbage and stale packets are ignored.
* `netplay/src/packet.rs`: every packet round-trips; truncated or extended packets and 20,000 random byte strings never panic the parser.
* `transport`: UDP exchange, stranger ignored once the peer is known, relay forwarding and room isolation, garbage and a third client ignored.
* `tools/tests/net_udp.rs`: two peers over **real UDP sockets** on localhost, directly and through the relay, agree on every confirmed frame.
* `pftool net-fuzz 2000`: randomised input delay, latency, loss, duplication and a mid-match total outage; every confirmed frame of
  both peers must equal a single-machine run of the same inputs. Current result: 2000 runs, 0 mismatches, about 105,000 rollbacks.
* `godot/tests/net_e2e.gd`: two `SimRunner` nodes play each other over UDP through the bridge.

The fuzzer found one real bug in the session: the input ring (128 frames) was narrower than the window of frames accepted from
the network, so a delayed duplicate packet from long ago could reuse a slot that already held newer inputs. Frames are now only
accepted within half a ring either side of the present.

## Unused fighter slots

The sim has four fighter slots, a two-player match uses two. The other two are marked inactive
(`GameState::new_with_active`, the handshake's `active` mask): they are not updated, cannot be hit, grabbed or
targeted, and so cannot absorb projectiles or take damage unseen. Before this was added they stood invisibly on the stage at
their spawn points and could be hit; the mask is part of the checksum.

## What is not done (honest list)

* **Never tested over the real internet.** Everything above is localhost or simulated. Latency spikes, NAT behaviour and ISP quirks
  are exactly what Phase 4's exit criterion ("stable matches across real internet connections") needs a real test for.
* **No NAT traversal.** Direct play needs a forwarded UDP port (or LAN / VPN); the relay is the fallback. There is no matchmaking or
  room list; the room number is agreed out of band.
* **Two players only.** The session supports more (`active` mask), but only the 1v1 handshake and UI exist, as the plan says to validate 1v1 first.
* **No frame-rate sync.** Each peer runs at its own 60 Hz; the stall rule keeps them within a few frames, but a peer whose clock runs
  slower will make the other stall repeatedly rather than slowing both gently. Plan section 6's input-delay setting is a launch argument
  (`--delay=`), not a menu.
* **No spectators, replays of networked matches, or reconnects.**
* The relay does no authentication (anyone who knows a room number can join it) and is not rate limited beyond a room cap.
* A peer can lie about *its own* inputs freely (that is inherent), and the checksum protocol assumes both sides are honest.
* Packet size and frequency are fine on a LAN; on a bad link the redundancy (every unacknowledged input in each packet, up to 40) is
  the only loss protection.
