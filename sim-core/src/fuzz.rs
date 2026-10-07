//! Deterministic random input generation shared by tests, the replay tool and the rollback fuzzer.

use crate::input::Input;
use crate::rng::Rng;
use crate::MAX_FIGHTERS;

/// Random but "human-ish" input: each player holds their input and changes it about 1 frame in 8,
/// so rollback predictions (repeat last input) are right some of the time and wrong some of the time.
pub fn random_inputs(rng: &mut Rng, frames: usize) -> Vec<[Input; MAX_FIGHTERS]> {
    let mut current = [Input::default(); MAX_FIGHTERS];
    let mut out = Vec::with_capacity(frames);
    for _ in 0..frames {
        for input in current.iter_mut() {
            if rng.range(8) == 0 {
                *input = Input {
                    stick_x: (rng.range(255) as i32 - 127) as i8,
                    stick_y: (rng.range(255) as i32 - 127) as i8,
                    buttons: (rng.next_u32() & 0x3f) as u16,
                };
            }
        }
        out.push(current);
    }
    out
}
