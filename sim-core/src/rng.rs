//! Seeded RNG whose entire state is one `u64` that lives in `GameState` (SplitMix64).

use crate::hash::{StateHash, StateHasher};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rng(pub u64);

impl Rng {
    pub const fn new(seed: u64) -> Self {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform-ish value in `0..n` (multiply-shift, no modulo bias worth caring about here).
    pub fn range(&mut self, n: u32) -> u32 {
        ((u64::from(self.next_u32()) * u64::from(n)) >> 32) as u32
    }
}

impl StateHash for Rng {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u64(self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn splitmix_reference_values() {
        // First outputs of SplitMix64 seeded with 0 (from the reference implementation).
        let mut r = Rng::new(0);
        assert_eq!(r.next_u64(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(r.next_u64(), 0x6e78_9e6a_a1b9_65f4);
    }

    #[test]
    fn range_stays_in_bounds() {
        let mut r = Rng::new(7);
        for _ in 0..10_000 {
            assert!(r.range(10) < 10);
        }
    }
}
