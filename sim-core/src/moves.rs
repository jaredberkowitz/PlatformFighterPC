//! Attack data: moves, hitboxes and weapons. A character's moveset comes from its **weapon**,
//! so a character is a body type (physics) plus a weapon (this file) plus cosmetics.
//!
//! Frame numbers count from 1, the first update after the attack starts. Positions are offsets from the
//! fighter's feet, with +x in the direction the fighter faces. Damage and knockback numbers use the
//! reference unit conventions (percent, base knockback, knockback growth).
//!
//! All numbers here are first-pass placeholders to be tuned in playtesting.

use crate::fixed::Fx;
use crate::hash::{StateHash, StateHasher};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MoveId {
    Jab = 0,
    FTilt,
    UTilt,
    DTilt,
    DashAttack,
    FSmash,
    USmash,
    DSmash,
    NAir,
    FAir,
    BAir,
    UAir,
    DAir,
}

impl MoveId {
    pub const COUNT: usize = 13;

    pub const fn is_aerial(self) -> bool {
        matches!(
            self,
            MoveId::NAir | MoveId::FAir | MoveId::BAir | MoveId::UAir | MoveId::DAir
        )
    }

    pub const fn from_index(i: u8) -> MoveId {
        match i {
            0 => MoveId::Jab,
            1 => MoveId::FTilt,
            2 => MoveId::UTilt,
            3 => MoveId::DTilt,
            4 => MoveId::DashAttack,
            5 => MoveId::FSmash,
            6 => MoveId::USmash,
            7 => MoveId::DSmash,
            8 => MoveId::NAir,
            9 => MoveId::FAir,
            10 => MoveId::BAir,
            11 => MoveId::UAir,
            _ => MoveId::DAir,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            MoveId::Jab => "jab",
            MoveId::FTilt => "ftilt",
            MoveId::UTilt => "utilt",
            MoveId::DTilt => "dtilt",
            MoveId::DashAttack => "dash attack",
            MoveId::FSmash => "fsmash",
            MoveId::USmash => "usmash",
            MoveId::DSmash => "dsmash",
            MoveId::NAir => "nair",
            MoveId::FAir => "fair",
            MoveId::BAir => "bair",
            MoveId::UAir => "uair",
            MoveId::DAir => "dair",
        }
    }
}

/// One circular hitbox, active for frames `start..=end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hitbox {
    pub start: u8,
    pub end: u8,
    pub x: Fx,
    pub y: Fx,
    pub radius: Fx,
    /// Percent of damage dealt.
    pub damage: Fx,
    /// Launch angle in degrees, 0 = forward. 361 is the "Sakurai" angle: horizontal for weak hits on the
    /// ground, 44 degrees otherwise.
    pub angle: i16,
    pub base_knockback: i16,
    pub knockback_growth: i16,
    /// When several hitboxes of one move overlap a target in the same frame, the lowest priority number
    /// wins. A sword's tip is priority 0 and its hilt priority 1, which is what makes spacing matter.
    pub priority: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Move {
    pub total_frames: u8,
    /// Extra recovery when an aerial lands (before `autocancel_*` lets it land cleanly).
    pub landing_lag: u8,
    /// Landing on frame <= this, or >= `autocancel_after`, lands with only the normal landing lag.
    pub autocancel_before: u8,
    pub autocancel_after: u8,
    pub hitboxes: Vec<Hitbox>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Weapon {
    /// Indexed by `MoveId as usize`.
    pub moves: Vec<Move>,
}

impl Weapon {
    pub fn get(&self, id: u8) -> &Move {
        let i = usize::from(id).min(self.moves.len().saturating_sub(1));
        &self.moves[i]
    }
}

impl StateHash for Hitbox {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.start);
        h.write_u8(self.end);
        self.x.hash_into(h);
        self.y.hash_into(h);
        self.radius.hash_into(h);
        self.damage.hash_into(h);
        h.write_u16(self.angle as u16);
        h.write_u16(self.base_knockback as u16);
        h.write_u16(self.knockback_growth as u16);
        h.write_u8(self.priority);
    }
}

impl StateHash for Weapon {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(self.moves.len() as u32);
        for m in &self.moves {
            h.write_u8(m.total_frames);
            h.write_u8(m.landing_lag);
            h.write_u8(m.autocancel_before);
            h.write_u8(m.autocancel_after);
            h.write_u32(m.hitboxes.len() as u32);
            for hb in &m.hitboxes {
                hb.hash_into(h);
            }
        }
    }
}

// ---- Weapons -----------------------------------------------------------------------------------------

/// Compact hitbox row: frames, position (tenths of a unit), radius (tenths), damage in half percents,
/// angle, base knockback, growth, priority.
struct B(u8, u8, i32, i32, i32, i32, i16, i16, i16, u8);

fn mk(total: u8, landing_lag: u8, autocancel: (u8, u8), boxes: &[B]) -> Move {
    let tenths = |n: i32| Fx::from_ratio(n, 10);
    Move {
        total_frames: total,
        landing_lag,
        autocancel_before: autocancel.0,
        autocancel_after: autocancel.1,
        hitboxes: boxes
            .iter()
            .map(|b| Hitbox {
                start: b.0,
                end: b.1,
                x: tenths(b.2),
                y: tenths(b.3),
                radius: tenths(b.4),
                damage: Fx::from_ratio(b.5, 2),
                angle: b.6,
                base_knockback: b.7,
                knockback_growth: b.8,
                priority: b.9,
            })
            .collect(),
    }
}

/// A long, thin blade: the tip (priority 0) hits harder than the body, so spacing is the skill.
pub fn longsword() -> Weapon {
    let none = (0, 255);
    Weapon {
        moves: vec![
            // Jab
            mk(
                18,
                0,
                none,
                &[
                    B(3, 5, 26, 11, 6, 8, 361, 10, 20, 0),
                    B(3, 5, 15, 11, 7, 6, 361, 8, 18, 1),
                ],
            ),
            // Forward tilt
            mk(
                28,
                0,
                none,
                &[
                    B(7, 9, 27, 11, 6, 16, 361, 12, 90, 0),
                    B(7, 9, 16, 11, 7, 12, 361, 10, 85, 1),
                ],
            ),
            // Up tilt
            mk(
                26,
                0,
                none,
                &[
                    B(7, 10, 10, 26, 6, 14, 100, 20, 100, 0),
                    B(7, 10, 6, 20, 8, 10, 100, 16, 95, 1),
                ],
            ),
            // Down tilt
            mk(
                24,
                0,
                none,
                &[
                    B(6, 8, 25, 4, 5, 12, 80, 10, 80, 0),
                    B(6, 8, 14, 4, 6, 8, 80, 8, 75, 1),
                ],
            ),
            // Dash attack
            mk(
                36,
                0,
                none,
                &[
                    B(8, 13, 26, 10, 6, 16, 361, 20, 80, 0),
                    B(8, 13, 15, 10, 8, 12, 361, 16, 75, 1),
                ],
            ),
            // Forward smash
            mk(
                46,
                0,
                none,
                &[
                    B(14, 16, 29, 12, 7, 32, 361, 25, 100, 0),
                    B(14, 16, 16, 11, 8, 24, 361, 22, 95, 1),
                ],
            ),
            // Up smash
            mk(
                52,
                0,
                none,
                &[
                    B(14, 18, 8, 29, 7, 26, 88, 30, 100, 0),
                    B(14, 18, 4, 20, 9, 20, 88, 26, 95, 1),
                ],
            ),
            // Down smash (hits both sides)
            mk(
                50,
                0,
                none,
                &[
                    B(12, 14, 27, 4, 6, 24, 30, 25, 85, 0),
                    B(12, 14, -20, 4, 6, 22, 30, 22, 80, 0),
                ],
            ),
            // Neutral air
            mk(32, 12, (4, 28), &[B(5, 18, 12, 11, 12, 12, 361, 8, 70, 0)]),
            // Forward air
            mk(
                38,
                14,
                (5, 32),
                &[
                    B(7, 10, 28, 13, 6, 18, 361, 20, 90, 0),
                    B(7, 10, 17, 12, 6, 14, 361, 16, 85, 1),
                ],
            ),
            // Back air
            mk(
                36,
                12,
                (5, 30),
                &[
                    B(7, 10, -28, 12, 6, 22, 135, 22, 95, 0),
                    B(7, 10, -17, 12, 6, 16, 135, 18, 90, 1),
                ],
            ),
            // Up air
            mk(
                34,
                11,
                (5, 28),
                &[
                    B(6, 10, 4, 30, 7, 16, 85, 20, 90, 0),
                    B(6, 10, 4, 21, 8, 12, 85, 16, 85, 1),
                ],
            ),
            // Down air
            mk(
                48,
                18,
                (9, 40),
                &[
                    B(11, 14, 6, 1, 6, 18, 270, 20, 80, 0),
                    B(11, 14, 6, 6, 7, 12, 270, 16, 75, 1),
                ],
            ),
        ],
    }
}

/// Short-reach, quick and heavy hitting: the longsword's moves pulled in close and sped up.
pub fn claws() -> Weapon {
    let mut w = longsword();
    for m in &mut w.moves {
        let quick = |f: u8| ((u16::from(f) * 85 + 50) / 100) as u8;
        m.total_frames = quick(m.total_frames).max(6);
        m.landing_lag = quick(m.landing_lag);
        m.autocancel_before = quick(m.autocancel_before);
        if m.autocancel_after != 255 {
            m.autocancel_after = quick(m.autocancel_after);
        }
        for hb in &mut m.hitboxes {
            hb.start = quick(hb.start).max(1);
            hb.end = quick(hb.end).max(hb.start);
            hb.x = hb.x * Fx::from_ratio(3, 5);
            hb.radius = hb.radius * Fx::from_ratio(11, 10);
            hb.damage = hb.damage * Fx::from_ratio(11, 10);
            hb.knockback_growth += 5;
            hb.priority = 0; // no tip and hilt: claws hit the same everywhere
        }
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_weapon_has_every_move() {
        for w in [longsword(), claws()] {
            assert_eq!(w.moves.len(), MoveId::COUNT);
            for m in &w.moves {
                assert!(!m.hitboxes.is_empty());
                for hb in &m.hitboxes {
                    assert!(hb.start >= 1 && hb.start <= hb.end && hb.end <= m.total_frames);
                    assert!(hb.damage > Fx::ZERO && hb.radius > Fx::ZERO);
                }
            }
        }
    }

    #[test]
    fn the_tip_hits_harder_than_the_body_for_the_sword() {
        let w = longsword();
        for id in [MoveId::FTilt, MoveId::FSmash, MoveId::FAir] {
            let m = &w.moves[id as usize];
            let tip = m.hitboxes.iter().find(|h| h.priority == 0).unwrap();
            let body = m.hitboxes.iter().find(|h| h.priority == 1).unwrap();
            assert!(tip.damage > body.damage, "{}", id.name());
            assert!(tip.x > body.x, "the tip is further out than the body");
        }
    }

    #[test]
    fn move_ids_round_trip() {
        for i in 0..MoveId::COUNT as u8 {
            assert_eq!(MoveId::from_index(i) as u8, i);
        }
    }

    #[test]
    fn aerials_are_the_last_five() {
        for i in 0..MoveId::COUNT as u8 {
            assert_eq!(MoveId::from_index(i).is_aerial(), i >= 8);
        }
    }
}
