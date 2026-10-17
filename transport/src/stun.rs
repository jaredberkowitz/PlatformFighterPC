//! The little of STUN (RFC 5389) needed to learn this machine's public address: a Binding Request with no attributes, and
//! the XOR-MAPPED-ADDRESS (or the older MAPPED-ADDRESS) of a Binding Success response. Pure byte work; the sockets that send
//! and receive these live in `punch.rs`.

use std::net::{Ipv4Addr, SocketAddrV4};

/// Every STUN message carries this in bytes 4..8.
pub const MAGIC_COOKIE: u32 = 0x2112_A442;
/// A transaction id: random, it ties a response to its request.
pub type TransactionId = [u8; 12];

const BINDING_REQUEST: u16 = 0x0001;
const BINDING_SUCCESS: u16 = 0x0101;
const ATTR_MAPPED_ADDRESS: u16 = 0x0001;
const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;
const HEADER: usize = 20;
const FAMILY_IPV4: u8 = 0x01;

/// Public STUN servers, tried in turn. Any one answering is enough; two answering also tells whether the router keeps the
/// same public port for every destination (codes need that).
pub const DEFAULT_SERVERS: [&str; 6] = [
    "stun.l.google.com:19302",
    "stun.cloudflare.com:3478",
    "stun1.l.google.com:19302",
    "stun.nextcloud.com:3478",
    "stun2.l.google.com:19302",
    "global.stun.twilio.com:3478",
];

/// A Binding Request with this transaction id (20 bytes: no attributes).
pub fn binding_request(id: &TransactionId) -> [u8; HEADER] {
    let mut out = [0u8; HEADER];
    out[0..2].copy_from_slice(&BINDING_REQUEST.to_be_bytes());
    // Message length 0 (bytes 2..4).
    out[4..8].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
    out[8..20].copy_from_slice(id);
    out
}

/// True when `bytes` look like any STUN message (so the game can drop late or stray ones without reading them further).
pub fn looks_like_stun(bytes: &[u8]) -> bool {
    bytes.len() >= HEADER && bytes[0] & 0xC0 == 0 && bytes[4..8] == MAGIC_COOKIE.to_be_bytes()
}

/// The transaction id of a STUN message, if `bytes` is one.
pub fn transaction_id(bytes: &[u8]) -> Option<TransactionId> {
    if !looks_like_stun(bytes) {
        return None;
    }
    let mut id = [0u8; 12];
    id.copy_from_slice(&bytes[8..20]);
    Some(id)
}

/// The public IPv4 address a Binding Success response reports, if `bytes` is one for transaction `id`. Prefers
/// XOR-MAPPED-ADDRESS; falls back to MAPPED-ADDRESS (old servers). Never panics on garbage.
pub fn parse_binding_response(bytes: &[u8], id: &TransactionId) -> Option<SocketAddrV4> {
    if !looks_like_stun(bytes) || bytes[8..20] != id[..] {
        return None;
    }
    if u16::from_be_bytes([bytes[0], bytes[1]]) != BINDING_SUCCESS {
        return None;
    }
    let length = usize::from(u16::from_be_bytes([bytes[2], bytes[3]]));
    let end = HEADER.checked_add(length)?;
    if end > bytes.len() {
        return None;
    }
    let mut at = HEADER;
    let mut plain = None;
    while at + 4 <= end {
        let kind = u16::from_be_bytes([bytes[at], bytes[at + 1]]);
        let len = usize::from(u16::from_be_bytes([bytes[at + 2], bytes[at + 3]]));
        let value_start = at + 4;
        let value_end = value_start.checked_add(len)?;
        if value_end > end {
            return None;
        }
        let value = &bytes[value_start..value_end];
        match kind {
            ATTR_XOR_MAPPED_ADDRESS => {
                if let Some(a) = decode_address(value, true) {
                    return Some(a);
                }
            }
            ATTR_MAPPED_ADDRESS => plain = plain.or(decode_address(value, false)),
            _ => {}
        }
        // Attributes are padded to a multiple of four bytes.
        at = value_start + len.div_ceil(4) * 4;
    }
    plain
}

/// An address attribute's value: reserved byte, family, port, address. IPv6 answers are skipped (IPv4 first).
fn decode_address(value: &[u8], xor: bool) -> Option<SocketAddrV4> {
    if value.len() < 8 || value[1] != FAMILY_IPV4 {
        return None;
    }
    let mut port = u16::from_be_bytes([value[2], value[3]]);
    let mut ip = u32::from_be_bytes([value[4], value[5], value[6], value[7]]);
    if xor {
        port ^= (MAGIC_COOKIE >> 16) as u16;
        ip ^= MAGIC_COOKIE;
    }
    Some(SocketAddrV4::new(Ipv4Addr::from(ip), port))
}

/// A Binding Success response reporting `addr` (what a STUN server sends; tests use it as a fake server).
pub fn binding_response(id: &TransactionId, addr: SocketAddrV4) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER + 12);
    out.extend_from_slice(&BINDING_SUCCESS.to_be_bytes());
    out.extend_from_slice(&12u16.to_be_bytes());
    out.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
    out.extend_from_slice(id);
    out.extend_from_slice(&ATTR_XOR_MAPPED_ADDRESS.to_be_bytes());
    out.extend_from_slice(&8u16.to_be_bytes());
    out.push(0);
    out.push(FAMILY_IPV4);
    out.extend_from_slice(&(addr.port() ^ (MAGIC_COOKIE >> 16) as u16).to_be_bytes());
    out.extend_from_slice(&(u32::from(*addr.ip()) ^ MAGIC_COOKIE).to_be_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: TransactionId = [
        0xb7, 0xe7, 0xa7, 0x01, 0xbc, 0x34, 0xd6, 0x86, 0xfa, 0x87, 0xdf, 0xae,
    ];

    #[test]
    fn a_binding_request_is_twenty_bytes_with_the_cookie_and_id() {
        let req = binding_request(&ID);
        assert_eq!(&req[0..4], &[0x00, 0x01, 0x00, 0x00]);
        assert_eq!(&req[4..8], &[0x21, 0x12, 0xA4, 0x42]);
        assert_eq!(&req[8..], &ID);
        assert!(looks_like_stun(&req));
        assert_eq!(transaction_id(&req), Some(ID));
    }

    /// The IPv4 response from RFC 5769 section 2.2 (the standard's own test vector): it reports 192.0.2.1:32853. It carries a
    /// SOFTWARE attribute before the address and MESSAGE-INTEGRITY and FINGERPRINT after it, which the parser must step over.
    #[test]
    fn the_rfc_5769_sample_response_decodes() {
        let sample: [u8; 80] = [
            0x01, 0x01, 0x00, 0x3c, 0x21, 0x12, 0xa4, 0x42, 0xb7, 0xe7, 0xa7, 0x01, 0xbc, 0x34,
            0xd6, 0x86, 0xfa, 0x87, 0xdf, 0xae, 0x80, 0x22, 0x00, 0x0b, 0x74, 0x65, 0x73, 0x74,
            0x20, 0x76, 0x65, 0x63, 0x74, 0x6f, 0x72, 0x20, 0x00, 0x20, 0x00, 0x08, 0x00, 0x01,
            0xa1, 0x47, 0xe1, 0x12, 0xa6, 0x43, 0x00, 0x08, 0x00, 0x14, 0x2b, 0x91, 0xf5, 0x99,
            0xfd, 0x9e, 0x90, 0xc3, 0x8c, 0x74, 0x89, 0xf9, 0x2a, 0xf9, 0xba, 0x53, 0xf0, 0x6b,
            0xe7, 0xd7, 0x80, 0x28, 0x00, 0x04, 0xc0, 0x7d, 0x4c, 0x96,
        ];
        assert_eq!(
            parse_binding_response(&sample, &ID),
            Some(SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 32853))
        );
        // Another transaction's response is not ours.
        let mut other = ID;
        other[0] ^= 1;
        assert_eq!(parse_binding_response(&sample, &other), None);
    }

    #[test]
    fn xor_mapped_address_round_trips_and_mapped_address_is_a_fallback() {
        let addr = SocketAddrV4::new(Ipv4Addr::new(203, 0, 113, 77), 61234);
        let resp = binding_response(&ID, addr);
        assert_eq!(parse_binding_response(&resp, &ID), Some(addr));
        // The XOR really is applied on the wire: the raw bytes are not the address.
        assert_ne!(&resp[26..28], &addr.port().to_be_bytes());
        // An old server's plain MAPPED-ADDRESS.
        let mut old = resp.clone();
        old[20..22].copy_from_slice(&ATTR_MAPPED_ADDRESS.to_be_bytes());
        old[26..28].copy_from_slice(&addr.port().to_be_bytes());
        old[28..32].copy_from_slice(&addr.ip().octets());
        assert_eq!(parse_binding_response(&old, &ID), Some(addr));
    }

    #[test]
    fn garbage_and_truncated_responses_are_refused_without_panicking() {
        let addr = SocketAddrV4::new(Ipv4Addr::new(198, 51, 100, 2), 4000);
        let resp = binding_response(&ID, addr);
        for n in 0..resp.len() {
            assert_eq!(
                parse_binding_response(&resp[..n], &ID),
                None,
                "truncated to {n}"
            );
        }
        // An error response type, or a length that runs past the end.
        let mut err = resp.clone();
        err[0..2].copy_from_slice(&0x0111u16.to_be_bytes());
        assert_eq!(parse_binding_response(&err, &ID), None);
        let mut long = resp.clone();
        long[2..4].copy_from_slice(&200u16.to_be_bytes());
        assert_eq!(parse_binding_response(&long, &ID), None);
        // Pseudo-random bytes.
        let mut x: u32 = 12345;
        for _ in 0..5000 {
            let mut buf = vec![0u8; (x % 64) as usize];
            for b in buf.iter_mut() {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                *b = x as u8;
            }
            let _ = parse_binding_response(&buf, &ID);
        }
        assert!(!looks_like_stun(b"PF not stun at all here"));
    }
}
