//! 16.16 fixed-point number backed by an `i32`, with `i64` intermediates.
//!
//! * Add, sub, mul, div and neg all **saturate**; none of them ever wrap or panic.
//! * `mul` rounds toward negative infinity (arithmetic shift); `div` truncates toward zero.
//!   Both are deterministic, which is what matters.
//! * Division by zero saturates to `MAX`/`MIN` by the sign of the numerator (`0/0 == 0`).

use crate::hash::{StateHash, StateHasher};
use core::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Fx(i32);

const fn saturate(v: i64) -> i32 {
    if v > i32::MAX as i64 {
        i32::MAX
    } else if v < i32::MIN as i64 {
        i32::MIN
    } else {
        v as i32
    }
}

/// `num / den` where `num` is already scaled by 2^16.
const fn div_scaled(num: i64, den: i64) -> Fx {
    if den == 0 {
        return if num > 0 {
            Fx::MAX
        } else if num < 0 {
            Fx::MIN
        } else {
            Fx::ZERO
        };
    }
    Fx(saturate(num / den))
}

impl Fx {
    pub const FRAC_BITS: u32 = 16;
    pub const ZERO: Fx = Fx(0);
    pub const ONE: Fx = Fx(1 << 16);
    pub const HALF: Fx = Fx(1 << 15);
    pub const MAX: Fx = Fx(i32::MAX);
    pub const MIN: Fx = Fx(i32::MIN);

    pub const fn from_raw(raw: i32) -> Fx {
        Fx(raw)
    }
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Integer to fixed point (saturates outside +-32767).
    pub const fn from_int(i: i32) -> Fx {
        Fx(saturate((i as i64) << 16))
    }

    /// Exact rational, e.g. `Fx::from_ratio(1, 3)`.
    pub const fn from_ratio(n: i32, d: i32) -> Fx {
        div_scaled((n as i64) << 16, d as i64)
    }

    /// Floor to an integer.
    pub const fn floor_int(self) -> i32 {
        self.0 >> 16
    }

    pub const fn saturating_add(self, rhs: Fx) -> Fx {
        Fx(self.0.saturating_add(rhs.0))
    }
    pub const fn saturating_sub(self, rhs: Fx) -> Fx {
        Fx(self.0.saturating_sub(rhs.0))
    }
    pub const fn saturating_mul(self, rhs: Fx) -> Fx {
        Fx(saturate((self.0 as i64 * rhs.0 as i64) >> 16))
    }
    pub const fn saturating_div(self, rhs: Fx) -> Fx {
        div_scaled((self.0 as i64) << 16, rhs.0 as i64)
    }
    pub const fn neg(self) -> Fx {
        Fx(self.0.saturating_neg())
    }

    /// Multiply by a plain integer (no scaling).
    pub const fn mul_int(self, k: i32) -> Fx {
        Fx(saturate(self.0 as i64 * k as i64))
    }

    pub const fn abs(self) -> Fx {
        if self.0 < 0 {
            self.neg()
        } else {
            self
        }
    }

    /// -1, 0 or 1 as an integer.
    pub const fn signum_int(self) -> i32 {
        if self.0 > 0 {
            1
        } else if self.0 < 0 {
            -1
        } else {
            0
        }
    }

    /// Square root; negative inputs give zero.
    pub const fn sqrt(self) -> Fx {
        if self.0 <= 0 {
            return Fx::ZERO;
        }
        Fx(isqrt_u64((self.0 as u64) << 16) as i32)
    }
}

/// Floor of the square root, exact for every `u64`.
pub const fn isqrt_u64(n: u64) -> u64 {
    let mut num = n;
    let mut res: u64 = 0;
    let mut bit: u64 = 1 << 62;
    while bit > num {
        bit >>= 2;
    }
    while bit != 0 {
        if num >= res + bit {
            num -= res + bit;
            res = (res >> 1) + bit;
        } else {
            res >>= 1;
        }
        bit >>= 2;
    }
    res
}

impl Add for Fx {
    type Output = Fx;
    fn add(self, rhs: Fx) -> Fx {
        self.saturating_add(rhs)
    }
}
impl Sub for Fx {
    type Output = Fx;
    fn sub(self, rhs: Fx) -> Fx {
        self.saturating_sub(rhs)
    }
}
impl Mul for Fx {
    type Output = Fx;
    fn mul(self, rhs: Fx) -> Fx {
        self.saturating_mul(rhs)
    }
}
impl Div for Fx {
    type Output = Fx;
    fn div(self, rhs: Fx) -> Fx {
        self.saturating_div(rhs)
    }
}
impl Neg for Fx {
    type Output = Fx;
    fn neg(self) -> Fx {
        Fx::neg(self)
    }
}
impl AddAssign for Fx {
    fn add_assign(&mut self, rhs: Fx) {
        *self = *self + rhs;
    }
}
impl SubAssign for Fx {
    fn sub_assign(&mut self, rhs: Fx) {
        *self = *self - rhs;
    }
}

impl StateHash for Fx {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_i32(self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    fn fx_int(i: i32) -> Fx {
        Fx::from_int(i)
    }

    #[test]
    fn basic_arithmetic() {
        assert_eq!(Fx::ONE * Fx::ONE, Fx::ONE);
        assert_eq!(fx_int(3) * fx_int(4), fx_int(12));
        assert_eq!(Fx::from_ratio(1, 2), Fx::HALF);
        assert_eq!(fx_int(1) / fx_int(4), Fx::from_raw(16384));
        assert_eq!(Fx::HALF * fx_int(3), Fx::from_raw(98304));
        assert_eq!((-Fx::HALF) * fx_int(3), Fx::from_raw(-98304));
        assert_eq!(fx_int(7).floor_int(), 7);
        assert_eq!(Fx::from_raw(-1).floor_int(), -1);
    }

    #[test]
    fn saturation_never_wraps() {
        assert_eq!(Fx::MAX + Fx::ONE, Fx::MAX);
        assert_eq!(Fx::MIN - Fx::ONE, Fx::MIN);
        assert_eq!(Fx::MAX * fx_int(2), Fx::MAX);
        assert_eq!(Fx::MIN * fx_int(2), Fx::MIN);
        assert_eq!(-Fx::MIN, Fx::MAX);
        assert_eq!(Fx::MIN.abs(), Fx::MAX);
        assert_eq!(fx_int(40000), Fx::MAX);
        assert_eq!(Fx::MAX.mul_int(3), Fx::MAX);
    }

    #[test]
    fn divide_by_zero_saturates() {
        assert_eq!(Fx::ONE / Fx::ZERO, Fx::MAX);
        assert_eq!(-Fx::ONE / Fx::ZERO, Fx::MIN);
        assert_eq!(Fx::ZERO / Fx::ZERO, Fx::ZERO);
    }

    #[test]
    fn mul_rounds_toward_negative_infinity() {
        assert_eq!(Fx::from_raw(1) * Fx::from_raw(1), Fx::ZERO);
        assert_eq!(Fx::from_raw(-1) * Fx::from_raw(1), Fx::from_raw(-1));
    }

    #[test]
    fn algebraic_properties_on_random_inputs() {
        let mut rng = Rng::new(1234);
        for _ in 0..20_000 {
            let a = Fx::from_raw(rng.next_u32() as i32);
            let b = Fx::from_raw(rng.next_u32() as i32);
            assert_eq!(a * b, b * a, "mul commutes");
            assert_eq!(a + b, b + a, "add commutes");
            assert_eq!(a * Fx::ONE, a, "ONE is the multiplicative identity");
            assert_eq!(a + Fx::ZERO, a);

            // No saturation in this range, so subtraction undoes addition exactly.
            let c = Fx::from_raw((rng.next_u32() as i32) >> 2);
            let d = Fx::from_raw((rng.next_u32() as i32) >> 2);
            assert_eq!((c + d) - d, c);
        }
    }

    #[test]
    fn isqrt_is_exact() {
        let mut rng = Rng::new(99);
        for n in [
            0u64,
            1,
            2,
            3,
            4,
            15,
            16,
            17,
            u64::MAX,
            u64::MAX - 1,
            1 << 62,
        ] {
            check_isqrt(n);
        }
        for _ in 0..20_000 {
            check_isqrt(rng.next_u64());
            check_isqrt(u64::from(rng.next_u32()));
        }
    }

    fn check_isqrt(n: u64) {
        let r = u128::from(isqrt_u64(n));
        let n = u128::from(n);
        assert!(r * r <= n && n < (r + 1) * (r + 1), "isqrt({n}) = {r}");
    }

    #[test]
    fn fx_sqrt() {
        assert_eq!(fx_int(4).sqrt(), fx_int(2));
        assert_eq!(fx_int(9).sqrt(), fx_int(3));
        assert_eq!(Fx::ZERO.sqrt(), Fx::ZERO);
        assert_eq!((-Fx::ONE).sqrt(), Fx::ZERO);
        // sqrt(2) = 1.41421356 -> 92681.9 in 16.16, floored.
        assert_eq!(fx_int(2).sqrt(), Fx::from_raw(92681));
    }
}
