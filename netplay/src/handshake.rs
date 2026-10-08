//! The pre-match handshake: both peers must run the same sim version and content before a match starts.
//!
//! The joiner says hello with its sim version, content hash and the fighter it brings; the host compares and either
//! rejects (with its own numbers, so both sides can log the mismatch) or answers with the match settings (seed,
//! characters, input delay, its own fighter and whether ranked rules apply). The joiner checks the host's numbers too,
//! then says ready. Every message is resent until answered, so it works over a lossy link.
//!
//! **Fighters.** The content both sides compare is the *base* roster. A player may bring a fighter made from a recipe;
//! its few bytes (see `sim_content::recipe::FighterSpec`) travel in the hello and the setup, and both sides build the
//! same match content from the two specs. The host works out each player's fighter index and sends it in the setup; the
//! joiner works it out too and refuses the match if the numbers differ. Under ranked rules a fighter over the point
//! budget is refused. Cosmetic loadouts travel along as opaque bytes and never reach the sim.
//!
//! Pure like the rest of this crate: call [`Handshake::tick`] once per tick for packets to send.

use crate::packet::{Packet, RejectReason, Setup};
use sim_content::recipe::{resolve, FighterSpec};

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

/// How many fighters and movesets the base content has: what a fighter spec can refer to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaseCounts {
    pub fighters: usize,
    pub weapons: usize,
}

pub struct Handshake {
    role: Role,
    version: u16,
    hash: u64,
    base: BaseCounts,
    /// The host's settings (host), or the ones received (joiner).
    setup: Option<Setup>,
    /// The joiner's cosmetics (what it says hello with, or what the host received).
    cosmetics: Vec<u8>,
    their_cosmetics: Vec<u8>,
    /// The joiner's fighter spec: what it says hello with, or what the host received.
    join_fighter: Vec<u8>,
    outcome: Outcome,
    ticks: u32,
    outbox: Vec<Vec<u8>>,
}

impl Handshake {
    pub fn host(version: u16, content_hash: u64, base: BaseCounts, setup: Setup) -> Handshake {
        Handshake {
            role: Role::Host,
            version,
            hash: content_hash,
            base,
            cosmetics: setup.cosmetics.clone(),
            setup: Some(setup),
            their_cosmetics: Vec::new(),
            join_fighter: Vec::new(),
            outcome: Outcome::Pending,
            ticks: 0,
            outbox: Vec::new(),
        }
    }

    pub fn join(
        version: u16,
        content_hash: u64,
        base: BaseCounts,
        cosmetics: Vec<u8>,
        fighter: Vec<u8>,
    ) -> Handshake {
        Handshake {
            role: Role::Join,
            version,
            hash: content_hash,
            base,
            setup: None,
            cosmetics,
            their_cosmetics: Vec::new(),
            join_fighter: fighter,
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

    /// The two fighter specs of the match as `(host's, joiner's)`, once known, if both players brought a made or
    /// chosen fighter. `None` for a match with no specs (both sides just use the characters in the settings).
    pub fn fighter_specs(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        let setup = self.setup.as_ref()?;
        (!setup.fighter.is_empty() && !self.join_fighter.is_empty())
            .then(|| (setup.fighter.clone(), self.join_fighter.clone()))
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
                        fighter: self.join_fighter.clone(),
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
                    fighter,
                },
            ) => {
                if matches!(self.outcome, Outcome::Ready(_)) {
                    // Already agreed; a late hello gets the setup again.
                } else if let Some(reason) = self.mismatch(sim_version, content_hash) {
                    self.reject(reason, sim_version, content_hash);
                    return;
                } else if let Err(reason) = self.accept_fighter(&fighter) {
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
                    if let Err(reason) = self.check_host_fighters(&setup) {
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

    /// The host, on a hello: checks both fighters, and fixes the settings' character numbers to match.
    fn accept_fighter(&mut self, joiner: &[u8]) -> Result<(), RejectReason> {
        let Some(setup) = self.setup.as_mut() else {
            return Ok(());
        };
        self.join_fighter = joiner.to_vec();
        if setup.fighter.is_empty() && joiner.is_empty() {
            return Ok(());
        }
        let chars = resolve_specs(self.base, &setup.fighter, joiner, setup.ranked)
            .ok_or(RejectReason::BadFighter)?;
        setup.chars[0] = chars[0];
        setup.chars[1] = chars[1];
        Ok(())
    }

    /// The joiner, on the setup: works out the fighter numbers itself and insists they are the host's.
    fn check_host_fighters(&self, setup: &Setup) -> Result<(), RejectReason> {
        if setup.fighter.is_empty() && self.join_fighter.is_empty() {
            return Ok(());
        }
        let chars = resolve_specs(self.base, &setup.fighter, &self.join_fighter, setup.ranked)
            .ok_or(RejectReason::BadFighter)?;
        if chars[0] == setup.chars[0] && chars[1] == setup.chars[1] {
            Ok(())
        } else {
            Err(RejectReason::BadFighter)
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

/// The two players' fighter indices, if both specs decode and are allowed; `None` otherwise (including when only one
/// player brought a spec).
pub fn resolve_specs(
    base: BaseCounts,
    host: &[u8],
    joiner: &[u8],
    ranked: bool,
) -> Option<[u8; 2]> {
    let h = FighterSpec::decode(host)?;
    let j = FighterSpec::decode(joiner)?;
    let chars = resolve(base.fighters, base.weapons, &[h, j], ranked).ok()?;
    Some([chars[0], chars[1]])
}
