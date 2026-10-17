//! Connection codes: a player's public address, LAN address and a session nonce, packed into a short code like
//! `K7QD-M2XA-9FPE-0RTV-5BWC` that two players swap over any chat.
//!
//! The alphabet is Crockford's base32 (digits and capitals without I, L, O, U): reading is case-insensitive, `I` and `L` count
//! as `1`, `O` as `0`, and spaces and dashes are ignored, so a code survives being retyped or pasted with extra spaces. A
//! 16-bit checksum (which also covers the code format's version) catches typos.
//!
//! Bit layout (most significant bit first): `has_public` (1), `lan_kind` (2), `lan_port_same` (1), then the public address
//! (32 + 16 bits, if any), the LAN address (by kind: 192.168.x.y is 16 bits, 10.x.y.z 24, anything else 32, loopback 0),
//! the LAN port (16 bits, unless it equals the public port), the nonce (16), the checksum (16), and zero padding up to a
//! whole number of four-character groups. A typical home player's code is 20 characters.

use std::fmt;
use std::net::{Ipv4Addr, SocketAddrV4};

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// Changing the layout changes this, and old codes then fail their checksum (both players need the same game version anyway).
const FORMAT_VERSION: u8 = 1;
const GROUP: usize = 4;

const LAN_192_168: u8 = 0;
const LAN_10: u8 = 1;
const LAN_FULL: u8 = 2;
const LAN_LOOPBACK: u8 = 3;

/// What a code says about one player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConnectionCode {
    /// The address the internet sees (from STUN), or `None` when it could not be found (then only the LAN address works).
    pub public: Option<SocketAddrV4>,
    /// The address on the player's own network, for two players behind the same router.
    pub lan: SocketAddrV4,
    /// A random number for this session: probes carry it, so a stranger's or an old session's packets are ignored.
    pub nonce: u16,
}

/// Why a typed code was not accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeError {
    Empty,
    /// A character that is not in the alphabet.
    BadCharacter(char),
    /// Too short, too long, or the checksum does not match (a typo, or a different game version).
    Invalid,
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeError::Empty => write!(f, "Paste your friend's code first."),
            CodeError::BadCharacter(c) => write!(f, "'{c}' cannot be in a code: check it was copied exactly."),
            CodeError::Invalid => write!(
                f,
                "That code is not right: check it was copied exactly (both players also need the same game version)."
            ),
        }
    }
}

struct Bits {
    bits: Vec<bool>,
}

impl Bits {
    fn push(&mut self, value: u32, width: u32) {
        for i in (0..width).rev() {
            self.bits.push((value >> i) & 1 == 1);
        }
    }
}

struct Reader<'a> {
    bits: &'a [bool],
    at: usize,
}

impl Reader<'_> {
    fn take(&mut self, width: u32) -> Option<u32> {
        let end = self.at + width as usize;
        if end > self.bits.len() {
            return None;
        }
        let v = self.bits[self.at..end]
            .iter()
            .fold(0u32, |acc, b| (acc << 1) | u32::from(*b));
        self.at = end;
        Some(v)
    }
}

/// FNV-1a over the format version and the payload bits (packed into bytes), folded to 16 bits.
fn checksum(payload: &[bool]) -> u16 {
    let mut h: u32 = 0x811c_9dc5;
    let mut feed = |b: u8| {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    };
    feed(FORMAT_VERSION);
    feed(payload.len() as u8);
    for chunk in payload.chunks(8) {
        feed(chunk.iter().fold(0u8, |acc, b| (acc << 1) | u8::from(*b)) << (8 - chunk.len()));
    }
    ((h >> 16) ^ (h & 0xffff)) as u16
}

fn lan_kind(ip: Ipv4Addr) -> u8 {
    let o = ip.octets();
    if ip == Ipv4Addr::LOCALHOST {
        LAN_LOOPBACK
    } else if o[0] == 192 && o[1] == 168 {
        LAN_192_168
    } else if o[0] == 10 {
        LAN_10
    } else {
        LAN_FULL
    }
}

impl ConnectionCode {
    /// The code as text: groups of four characters joined by dashes.
    pub fn encode(&self) -> String {
        let mut b = Bits { bits: Vec::new() };
        let kind = lan_kind(*self.lan.ip());
        let port_same = self.public.is_some_and(|p| p.port() == self.lan.port());
        b.push(u32::from(self.public.is_some()), 1);
        b.push(u32::from(kind), 2);
        b.push(u32::from(port_same), 1);
        if let Some(p) = self.public {
            b.push(u32::from(*p.ip()), 32);
            b.push(u32::from(p.port()), 16);
        }
        let ip = u32::from(*self.lan.ip());
        match kind {
            LAN_192_168 => b.push(ip & 0xffff, 16),
            LAN_10 => b.push(ip & 0xff_ffff, 24),
            LAN_FULL => b.push(ip, 32),
            _ => {}
        }
        if !port_same {
            b.push(u32::from(self.lan.port()), 16);
        }
        b.push(u32::from(self.nonce), 16);
        let sum = checksum(&b.bits);
        b.push(u32::from(sum), 16);
        let group_bits = 5 * GROUP;
        while b.bits.len() % group_bits != 0 {
            b.bits.push(false);
        }
        let chars: Vec<char> = b
            .bits
            .chunks(5)
            .map(|c| {
                let v = c.iter().fold(0usize, |acc, x| (acc << 1) | usize::from(*x));
                char::from(ALPHABET[v])
            })
            .collect();
        chars
            .chunks(GROUP)
            .map(|g| g.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Reads a code typed or pasted by a player. Case, spaces and dashes do not matter; `I`/`L` read as `1` and `O` as `0`.
    pub fn decode(text: &str) -> Result<ConnectionCode, CodeError> {
        let mut bits = Vec::new();
        for c in text.chars() {
            if c.is_whitespace() || c == '-' || c == '.' || c == '_' {
                continue;
            }
            let up = match c.to_ascii_uppercase() {
                'I' | 'L' => '1',
                'O' => '0',
                u => u,
            };
            let v = ALPHABET
                .iter()
                .position(|a| char::from(*a) == up)
                .ok_or(CodeError::BadCharacter(c))?;
            for i in (0..5).rev() {
                bits.push((v >> i) & 1 == 1);
            }
        }
        if bits.is_empty() {
            return Err(CodeError::Empty);
        }
        if bits.len() > 5 * 40 {
            return Err(CodeError::Invalid);
        }
        let mut r = Reader { bits: &bits, at: 0 };
        let inv = CodeError::Invalid;
        let has_public = r.take(1).ok_or(inv)? == 1;
        let kind = r.take(2).ok_or(inv)? as u8;
        let port_same = r.take(1).ok_or(inv)? == 1;
        if port_same && !has_public {
            return Err(inv);
        }
        let public = if has_public {
            let ip = Ipv4Addr::from(r.take(32).ok_or(inv)?);
            let port = r.take(16).ok_or(inv)? as u16;
            Some(SocketAddrV4::new(ip, port))
        } else {
            None
        };
        let lan_ip = match kind {
            LAN_192_168 => Ipv4Addr::from(0xc0a8_0000 | r.take(16).ok_or(inv)?),
            LAN_10 => Ipv4Addr::from(0x0a00_0000 | r.take(24).ok_or(inv)?),
            LAN_FULL => Ipv4Addr::from(r.take(32).ok_or(inv)?),
            _ => Ipv4Addr::LOCALHOST,
        };
        let lan_port = match (port_same, public) {
            (true, Some(p)) => p.port(),
            _ => r.take(16).ok_or(inv)? as u16,
        };
        let nonce = r.take(16).ok_or(inv)? as u16;
        let payload_end = r.at;
        let sum = r.take(16).ok_or(inv)? as u16;
        if sum != checksum(&bits[..payload_end]) {
            return Err(inv);
        }
        // Only the zero padding of the last group may follow.
        let rest = &bits[r.at..];
        if rest.len() >= 5 * GROUP || rest.iter().any(|b| *b) {
            return Err(inv);
        }
        Ok(ConnectionCode {
            public,
            lan: SocketAddrV4::new(lan_ip, lan_port),
            nonce,
        })
    }

    /// Which of two players hosts (and so sets the rules): the one with the smaller nonce, or, on a tie, the smaller code.
    /// Both players work this out the same way, so nobody has to choose.
    pub fn hosts_against(&self, other: &ConnectionCode) -> bool {
        (self.nonce, self.encode()) < (other.nonce, other.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> ConnectionCode {
        ConnectionCode {
            public: Some(SocketAddrV4::new(Ipv4Addr::new(203, 0, 113, 45), 47000)),
            lan: SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 23), 47000),
            nonce: 0xbeef,
        }
    }

    #[test]
    fn a_home_players_code_is_five_groups_and_round_trips() {
        let code = home().encode();
        assert_eq!(code.len(), 5 * 4 + 4, "{code}");
        assert_eq!(code.matches('-').count(), 4);
        assert_eq!(ConnectionCode::decode(&code), Ok(home()));
    }

    #[test]
    fn every_kind_of_address_round_trips() {
        let cases = [
            (Some((198, 51, 100, 7, 50000)), (10, 20, 30, 40, 47000)),
            (Some((198, 51, 100, 7, 50000)), (172, 16, 4, 9, 47001)),
            (Some((1, 2, 3, 4, 1)), (192, 168, 0, 1, 2)),
            (None, (192, 168, 50, 60, 47000)),
            (None, (127, 0, 0, 1, 55555)),
            (Some((255, 255, 255, 255, 65535)), (127, 0, 0, 1, 65535)),
        ];
        for (n, (public, lan)) in cases.into_iter().enumerate() {
            let code = ConnectionCode {
                public: public
                    .map(|(a, b, c, d, p)| SocketAddrV4::new(Ipv4Addr::new(a, b, c, d), p)),
                lan: SocketAddrV4::new(Ipv4Addr::new(lan.0, lan.1, lan.2, lan.3), lan.4),
                nonce: n as u16 * 9001,
            };
            let text = code.encode();
            assert!(text.replace('-', "").len() <= 28, "{text}");
            assert_eq!(ConnectionCode::decode(&text), Ok(code), "{text}");
        }
    }

    #[test]
    fn case_spaces_and_look_alike_letters_do_not_matter() {
        let code = home().encode();
        let messy = format!("  {}  ", code.to_lowercase().replace('-', " - "));
        assert_eq!(ConnectionCode::decode(&messy), Ok(home()));
        let squashed: String = code.chars().filter(|c| *c != '-').collect();
        assert_eq!(ConnectionCode::decode(&squashed), Ok(home()));
        // 0 typed as O and 1 typed as I or L still read correctly.
        let swapped = code.replace('0', "O").replace('1', "l");
        assert_eq!(ConnectionCode::decode(&swapped), Ok(home()));
    }

    #[test]
    fn typos_are_caught_by_the_checksum() {
        let code = home().encode();
        let chars: Vec<char> = code.chars().collect();
        let mut caught = 0;
        let mut tried = 0;
        for i in 0..chars.len() {
            if chars[i] == '-' {
                continue;
            }
            for &a in ALPHABET.iter() {
                let a = char::from(a);
                if a == chars[i] {
                    continue;
                }
                let mut typo = chars.clone();
                typo[i] = a;
                let text: String = typo.into_iter().collect();
                tried += 1;
                if ConnectionCode::decode(&text).is_err() {
                    caught += 1;
                }
            }
        }
        assert_eq!(caught, tried, "every single-character typo is refused");
        // Two neighbours swapped.
        let mut swapped = chars.clone();
        swapped.swap(0, 1);
        if swapped != chars {
            assert!(ConnectionCode::decode(&swapped.into_iter().collect::<String>()).is_err());
        }
        // Missing or extra characters.
        assert!(ConnectionCode::decode(&code[..code.len() - 1]).is_err());
        assert!(ConnectionCode::decode(&format!("{code}7")).is_err());
        assert!(ConnectionCode::decode(&format!("{code}-0000")).is_err());
    }

    #[test]
    fn empty_and_foreign_text_get_clear_errors() {
        assert_eq!(ConnectionCode::decode(" - "), Err(CodeError::Empty));
        assert_eq!(
            ConnectionCode::decode("K7QD-M2XU"),
            Err(CodeError::BadCharacter('U'))
        );
        assert_eq!(
            ConnectionCode::decode("203.0.113.5:47000"),
            Err(CodeError::BadCharacter(':'))
        );
        assert_eq!(ConnectionCode::decode("ABCD"), Err(CodeError::Invalid));
        assert!(!CodeError::Invalid.to_string().is_empty());
    }

    #[test]
    fn exactly_one_of_two_players_hosts() {
        let a = home();
        let mut b = home();
        b.nonce = 0x1234;
        assert!(b.hosts_against(&a) && !a.hosts_against(&b));
        // Same nonce: the codes decide.
        let mut c = home();
        c.lan = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 99), 47000);
        assert_ne!(a.hosts_against(&c), c.hosts_against(&a));
    }
}
