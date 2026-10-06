//! Lookup-table trigonometry with no floats, ever.
//!
//! The quarter-wave table is computed at compile time with an integer Taylor series in Q32 (`i128`),
//! so there is no generator script and no platform `sin`. Angles are `u16` units of 1/4096 turn.

use crate::fixed::Fx;

const QUARTER: usize = 1024;
const TURN: u16 = 4096;
/// pi/2 in Q32: 0x1.921FB544 * 2^32.
const HALF_PI_Q32: i128 = 0x1_921F_B544;

const fn sin_q16(k: usize) -> i32 {
    let x = HALF_PI_Q32 * k as i128 / QUARTER as i128;
    let x2 = (x * x) >> 32;
    let mut term = x;
    let mut sum = x;
    let mut n: i128 = 1;
    while n < 12 {
        term = -((term * x2) >> 32) / ((2 * n) * (2 * n + 1));
        sum += term;
        n += 1;
    }
    ((sum + (1 << 15)) >> 16) as i32
}

const fn build_table() -> [i32; QUARTER + 1] {
    let mut t = [0i32; QUARTER + 1];
    let mut i = 0;
    while i <= QUARTER {
        t[i] = sin_q16(i);
        i += 1;
    }
    t
}

static SIN_TABLE: [i32; QUARTER + 1] = build_table();

/// An angle in 1/4096ths of a full turn. Arithmetic wraps, which is exactly what angles should do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Angle(u16);

impl Angle {
    pub const ZERO: Angle = Angle(0);
    pub const QUARTER_TURN: Angle = Angle(1024);
    pub const HALF_TURN: Angle = Angle(2048);

    pub const fn from_raw(raw: u16) -> Angle {
        Angle(raw % TURN)
    }
    pub const fn raw(self) -> u16 {
        self.0
    }
    pub const fn wrapping_add(self, rhs: Angle) -> Angle {
        Angle((self.0 + rhs.0) % TURN)
    }
    pub const fn wrapping_sub(self, rhs: Angle) -> Angle {
        Angle((self.0 + TURN - rhs.0) % TURN)
    }
    /// Whole degrees (0..360), rounded down to the nearest 1/4096 turn.
    pub const fn from_degrees(deg: i32) -> Angle {
        let d = deg.rem_euclid(360) as u32;
        Angle((d * TURN as u32 / 360) as u16)
    }
}

pub fn sin(a: Angle) -> Fx {
    let idx = a.0 as usize;
    let q = idx / QUARTER;
    let i = idx % QUARTER;
    let raw = match q {
        0 => SIN_TABLE[i],
        1 => SIN_TABLE[QUARTER - i],
        2 => -SIN_TABLE[i],
        _ => -SIN_TABLE[QUARTER - i],
    };
    Fx::from_raw(raw)
}

pub fn cos(a: Angle) -> Fx {
    sin(a.wrapping_add(Angle::QUARTER_TURN))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_endpoints() {
        assert_eq!(SIN_TABLE[0], 0);
        assert_eq!(SIN_TABLE[QUARTER], Fx::ONE.raw());
        // sin(pi/4) = 0.70710678 * 65536 = 46340.95
        assert_eq!(SIN_TABLE[QUARTER / 2], 46341);
    }

    #[test]
    fn table_is_monotonic_over_the_quarter() {
        for i in 0..QUARTER {
            assert!(SIN_TABLE[i] <= SIN_TABLE[i + 1]);
        }
    }

    #[test]
    fn symmetries() {
        for raw in 0..TURN {
            let a = Angle::from_raw(raw);
            let opposite = a.wrapping_add(Angle::HALF_TURN);
            assert_eq!(sin(opposite), -sin(a));
            assert_eq!(cos(opposite), -cos(a));
        }
        assert_eq!(sin(Angle::ZERO), Fx::ZERO);
        assert_eq!(cos(Angle::ZERO), Fx::ONE);
        assert_eq!(sin(Angle::QUARTER_TURN), Fx::ONE);
        assert_eq!(cos(Angle::QUARTER_TURN), Fx::ZERO);
    }

    #[test]
    fn pythagorean_identity_holds_within_quantisation() {
        for raw in 0..TURN {
            let a = Angle::from_raw(raw);
            let (s, c) = (sin(a), cos(a));
            let sum = (s * s + c * c).raw();
            assert!((sum - Fx::ONE.raw()).abs() <= 8, "angle {raw}: {sum}");
        }
    }

    #[test]
    fn degrees_helper() {
        assert_eq!(Angle::from_degrees(90), Angle::QUARTER_TURN);
        assert_eq!(Angle::from_degrees(180), Angle::HALF_TURN);
        assert_eq!(Angle::from_degrees(-90).raw(), 3072);
        assert_eq!(Angle::from_degrees(450), Angle::QUARTER_TURN);
    }
}
