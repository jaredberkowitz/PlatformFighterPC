//! Rollback netcode.
//!
//! * Phase 2: [`local_rollback`] proves the sim is rollback-safe on one machine, with no sockets.
//! * Phase 4: [`packet`] (the wire format), [`handshake`] (version and content-hash check before a match),
//!   [`session`] (input delay, prediction, rollback, stalling, desync detection), [`peer`] (the glue over any
//!   [`peer::Link`]) and [`testlink`] (a deterministic lossy network for tests).
//!
//! This crate is pure like the sim crates: no sockets, clocks or threads. The real UDP transport lives in the
//! `transport` crate and implements [`peer::Link`].

pub mod handshake;
pub mod local_rollback;
pub mod packet;
pub mod peer;
pub mod replay;
pub mod session;
pub mod spectate;
pub mod testlink;
