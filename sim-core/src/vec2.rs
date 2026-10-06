//! Fixed-point 2D vector. +x is right, +y is up.

use crate::fixed::{isqrt_u64, Fx};
use crate::hash::{StateHash, StateHasher};
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vec2 {
    pub x: Fx,
    pub y: Fx,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 {
        x: Fx::ZERO,
        y: Fx::ZERO,
    };

    pub const fn new(x: Fx, y: Fx) -> Vec2 {
        Vec2 { x, y }
    }

    pub fn dot(self, rhs: Vec2) -> Fx {
        self.x * rhs.x + self.y * rhs.y
    }

    /// Squared length. Prefer this for distance comparisons so hot paths avoid square roots.
    pub fn length_sq(self) -> Fx {
        self.dot(self)
    }

    /// Exact floor of the true length, with no intermediate overflow.
    ///
    /// sqrt(x_raw^2 + y_raw^2) is already in 16.16 units, so no rescaling is needed.
    pub fn length(self) -> Fx {
        let ax = u64::from(self.x.raw().unsigned_abs());
        let ay = u64::from(self.y.raw().unsigned_abs());
        let r = isqrt_u64(ax * ax + ay * ay);
        Fx::from_raw(r.min(i32::MAX as u64) as i32)
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}
impl Mul<Fx> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: Fx) -> Vec2 {
        Vec2::new(self.x * k, self.y * k)
    }
}
impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        *self = *self + rhs;
    }
}
impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = *self - rhs;
    }
}

impl StateHash for Vec2 {
    fn hash_into(&self, h: &mut StateHasher) {
        self.x.hash_into(h);
        self.y.hash_into(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: i32, y: i32) -> Vec2 {
        Vec2::new(Fx::from_int(x), Fx::from_int(y))
    }

    #[test]
    fn three_four_five() {
        assert_eq!(v(3, 4).length(), Fx::from_int(5));
        assert_eq!(v(-3, 4).length(), Fx::from_int(5));
        assert_eq!(v(3, 4).length_sq(), Fx::from_int(25));
    }

    #[test]
    fn length_handles_extremes_without_overflow() {
        let m = Vec2::new(Fx::MIN, Fx::MIN);
        assert_eq!(m.length(), Fx::MAX);
        assert_eq!(Vec2::ZERO.length(), Fx::ZERO);
    }

    #[test]
    fn ops() {
        assert_eq!(v(1, 2) + v(3, 4), v(4, 6));
        assert_eq!(v(1, 2) - v(3, 4), v(-2, -2));
        assert_eq!(v(1, 2) * Fx::from_int(3), v(3, 6));
        assert_eq!(v(1, 2).dot(v(3, 4)), Fx::from_int(11));
        assert_eq!(-v(1, -2), v(-1, 2));
    }
}
