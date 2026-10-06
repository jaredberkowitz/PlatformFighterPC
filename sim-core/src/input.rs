//! One frame of controller input for one player. Plain-old-data: 4 bytes, no padding.

use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};

pub mod buttons {
    pub const JUMP: u16 = 1 << 0;
    pub const ATTACK: u16 = 1 << 1;
    pub const SPECIAL: u16 = 1 << 2;
    pub const SHIELD: u16 = 1 << 3;
    pub const GRAB: u16 = 1 << 4;
}

/// Stick magnitude (of 127) that counts as a deliberate flick or tap, about 0.6.
pub const STICK_THRESHOLD: i8 = 76;
/// Stick magnitude below which an axis is treated as neutral, about 0.17.
pub const STICK_DEADZONE: i8 = 22;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Input {
    pub stick_x: i8,
    pub stick_y: i8,
    pub buttons: u16,
}

impl Input {
    pub const BYTES: usize = 4;

    pub const fn pressed(self, mask: u16) -> bool {
        self.buttons & mask != 0
    }

    /// Stick axis normalised to [-1, 1].
    pub fn axis(v: i8) -> Fx {
        Fx::from_ratio(i32::from(v), 127).clamp(-Fx::ONE, Fx::ONE)
    }

    pub fn stick_x_fx(self) -> Fx {
        Self::axis(self.stick_x)
    }
    pub fn stick_y_fx(self) -> Fx {
        Self::axis(self.stick_y)
    }

    pub fn to_bytes(self) -> [u8; Self::BYTES] {
        let b = self.buttons.to_le_bytes();
        [self.stick_x as u8, self.stick_y as u8, b[0], b[1]]
    }

    pub fn from_bytes(b: [u8; Self::BYTES]) -> Input {
        Input {
            stick_x: b[0] as i8,
            stick_y: b[1] as i8,
            buttons: u16::from_le_bytes([b[2], b[3]]),
        }
    }
}

impl StateHash for Input {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_i8(self.stick_x);
        h.write_i8(self.stick_y);
        h.write_u16(self.buttons);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_round_trip() {
        let i = Input {
            stick_x: -128,
            stick_y: 127,
            buttons: buttons::JUMP | buttons::SHIELD,
        };
        assert_eq!(Input::from_bytes(i.to_bytes()), i);
    }

    #[test]
    fn axis_is_clamped() {
        assert_eq!(Input::axis(127), Fx::ONE);
        assert_eq!(Input::axis(-128), -Fx::ONE);
        assert_eq!(Input::axis(0), Fx::ZERO);
    }
}
