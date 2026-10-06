//! Rollback netcode. Phase 2 (this file's current scope) proves the sim is rollback-safe on one
//! machine, with no sockets involved. Phase 4 adds transport, relay fallback, the content-hash
//! handshake and live desync detection.

pub mod local_rollback;
