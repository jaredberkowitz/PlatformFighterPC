//! The pre-match handshake: both peers must run the same sim version and content before a match starts.
//!
//! The joiner says hello with its sim version and content hash; the host compares and either rejects (with its
//! own numbers, so both sides can log the mismatch) or answers with the match settings (seed, characters, input
//! delay). The joiner checks the host's numbers too, then says ready. Every message is resent until answered, so
//! it works over a lossy link. Cosmetic loadouts travel along as opaque bytes and never reach the sim.
//!
//! Pure like the rest of this crate: call [`Handshake::tick`] once per tick for packets to send.

use crate::packet::{Packet, RejectReason, Setup};

/// Ticks between resends of an unanswered message.
const RESEND_EVERY: u32 = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Pending,
    /// Both sides agree. For the joiner this carries the host's settings; for the host, its own.
    Ready(Setup),
    Rejected {
        reason: RejectReason,
        /// The other side's numbers, for the log.
        their_version: u16,
        their_hash: u64,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Host,
    Join,
}

pub struct Handshake {
    role: Role,
    version: u16,
    hash: u64,
    /// The host's settings (host), or the ones received (joiner).
    setup: Option<Setup>,
    /// The joiner's cosmetics (what it says hello with, or what the host received).
    cosmetics: Vec<u8>,
    their_cosmetics: Vec<u8>,
    outcome: Outcome,
    ticks: u32,
    outbox: Vec<Vec<u8>>,
}

impl Handshake {
    pub fn host(version: u16, content_hash: u64, setup: Setup) -> Handshake {
        Handshake {
            role: Role::Host,
            version,
            hash: content_hash,
            cosmetics: setup.cosmetics.clone(),
            setup: Some(setup),
            their_cosmetics: Vec::new(),
            outcome: Outcome::Pending,
            ticks: 0,
            outbox: Vec::new(),
        }
    }

    pub fn join(version: u16, content_hash: u64, cosmetics: Vec<u8>) -> Handshake {
        Handshake {
            role: Role::Join,
            version,
            hash: content_hash,
            setup: None,
            cosmetics,
            their_cosmetics: Vec::new(),
            outcome: Outcome::Pending,
            ticks: 0,
            outbox: Vec::new(),
        }
    }

    pub fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    /// The other side's cosmetics (opaque; never used by the sim).
    pub fn their_cosmetics(&self) -> &[u8] {
        &self.their_cosmetics
    }

    /// Advances one tick and returns the packets to send now.
    pub fn tick(&mut self) -> Vec<Vec<u8>> {
        self.ticks += 1;
        if self.ticks % RESEND_EVERY == 1 {
            match (self.role, &self.outcome) {
                // The joiner keeps asking until it has the setup, then keeps saying ready (the host's ack is
                // the first input packet, which the caller sees).
                (Role::Join, Outcome::Pending) => self.outbox.push(
                    Packet::Hello {
                        sim_version: self.version,
                        content_hash: self.hash,
                        cosmetics: self.cosmetics.clone(),
                    }
                    .encode(),
                ),
                (Role::Join, Outcome::Ready(_)) => self.outbox.push(Packet::Ready.encode()),
                _ => {}
            }
        }
        std::mem::take(&mut self.outbox)
    }

    pub fn handle_packet(&mut self, bytes: &[u8]) {
        let Some(packet) = Packet::decode(bytes) else {
            return;
        };
        match (self.role, packet) {
            (
                Role::Host,
                Packet::Hello {
                    sim_version,
                    content_hash,
                    cosmetics,
                },
            ) => {
                if matches!(self.outcome, Outcome::Ready(_)) {
                    // Already agreed; a late hello gets the setup again.
                } else if let Some(reason) = self.mismatch(sim_version, content_hash) {
                    self.reject(reason, sim_version, content_hash);
                    return;
                }
                self.their_cosmetics = cosmetics;
                if let Some(setup) = &self.setup {
                    self.outbox.push(
                        Packet::Setup {
                            sim_version: self.version,
                            content_hash: self.hash,
                            setup: setup.clone(),
                        }
                        .encode(),
                    );
                }
            }
            (Role::Host, Packet::Ready) => {
                if let (Outcome::Pending, Some(setup)) = (&self.outcome, &self.setup) {
                    self.outcome = Outcome::Ready(setup.clone());
                }
            }
            (
                Role::Join,
                Packet::Setup {
                    sim_version,
                    content_hash,
                    setup,
                },
            ) => {
                if matches!(self.outcome, Outcome::Pending) {
                    if let Some(reason) = self.mismatch(sim_version, content_hash) {
                        self.reject(reason, sim_version, content_hash);
                        return;
                    }
                    self.their_cosmetics = setup.cosmetics.clone();
                    self.outcome = Outcome::Ready(setup.clone());
                    self.setup = Some(setup);
                }
                self.outbox.push(Packet::Ready.encode());
            }
            (
                _,
                Packet::Reject {
                    reason,
                    sim_version,
                    content_hash,
                },
            ) if !matches!(self.outcome, Outcome::Ready(_)) => {
                self.outcome = Outcome::Rejected {
                    reason,
                    their_version: sim_version,
                    their_hash: content_hash,
                };
            }
            _ => {}
        }
    }

    fn mismatch(&self, version: u16, hash: u64) -> Option<RejectReason> {
        if version != self.version {
            Some(RejectReason::SimVersion)
        } else if hash != self.hash {
            Some(RejectReason::ContentHash)
        } else {
            None
        }
    }

    fn reject(&mut self, reason: RejectReason, their_version: u16, their_hash: u64) {
        self.outbox.push(
            Packet::Reject {
                reason,
                sim_version: self.version,
                content_hash: self.hash,
            }
            .encode(),
        );
        self.outcome = Outcome::Rejected {
            reason,
            their_version,
            their_hash,
        };
    }
}
