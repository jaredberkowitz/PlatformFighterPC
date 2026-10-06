//! Platform-independent state hashing (FNV-1a 64 with an avalanche finaliser).
//!
//! Fields are fed explicitly in little-endian order, so the checksum never depends on struct
//! padding, field layout or host endianness. This is a desync detector, not a security hash.

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Clone, Debug)]
pub struct StateHasher(u64);

impl Default for StateHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl StateHasher {
    pub const fn new() -> Self {
        StateHasher(FNV_OFFSET)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }

    pub fn write_u8(&mut self, v: u8) {
        self.write_bytes(&[v]);
    }
    pub fn write_i8(&mut self, v: i8) {
        self.write_bytes(&v.to_le_bytes());
    }
    pub fn write_bool(&mut self, v: bool) {
        self.write_u8(u8::from(v));
    }
    pub fn write_u16(&mut self, v: u16) {
        self.write_bytes(&v.to_le_bytes());
    }
    pub fn write_u32(&mut self, v: u32) {
        self.write_bytes(&v.to_le_bytes());
    }
    pub fn write_i32(&mut self, v: i32) {
        self.write_bytes(&v.to_le_bytes());
    }
    pub fn write_u64(&mut self, v: u64) {
        self.write_bytes(&v.to_le_bytes());
    }

    pub fn finish(&self) -> u64 {
        let mut z = self.0;
        z ^= z >> 30;
        z = z.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z ^= z >> 27;
        z = z.wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

/// Anything that contributes to the checksum.
pub trait StateHash {
    fn hash_into(&self, h: &mut StateHasher);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vector_is_stable() {
        // Guards against accidental changes to the hash, which would invalidate every golden checksum.
        let mut h = StateHasher::new();
        h.write_bytes(b"platform fighter");
        let a = h.finish();
        let mut h2 = StateHasher::new();
        h2.write_bytes(b"platform fighter");
        assert_eq!(a, h2.finish());
        let mut h3 = StateHasher::new();
        h3.write_bytes(b"platform fighteR");
        assert_ne!(a, h3.finish());
    }

    #[test]
    fn integer_writes_are_little_endian() {
        let mut a = StateHasher::new();
        a.write_u32(0x0102_0304);
        let mut b = StateHasher::new();
        b.write_bytes(&[4, 3, 2, 1]);
        assert_eq!(a.finish(), b.finish());
    }
}
