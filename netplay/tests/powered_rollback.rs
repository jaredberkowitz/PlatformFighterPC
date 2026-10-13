//! Rollback with the Greatsword's powered mode running all the time (sim 87): four champions whose gauge fills on any damage, so
//! random play keeps throwing the powered versions (the bursting wave, the vortex, the finisher's hover, the rise with air control).
//! Every rollback must resimulate to the same checksums as a straight run.

use netplay::local_rollback::{random_delays, verify};
use sim_core::fuzz::random_inputs;
use sim_core::{Content, Fx, GameState, Rng};

const CHAMPION: u8 = 6;

#[test]
fn powered_champions_roll_back_exactly() {
    let mut content = Content::placeholder();
    // Any percent dealt or taken fills the gauge at once: the mode is on most of the match.
    let g = &mut content.fighters[usize::from(CHAMPION)].gauge;
    g.per_dealt = Fx::from_int(100);
    g.per_taken = Fx::from_int(100);
    let mut powered_frames = 0u32;
    let mut bursts = 0u32;
    for seed in 0..40u64 {
        let mut initial = GameState::new(&content, seed, [CHAMPION; 4]);
        for f in &mut initial.fighters {
            f.gauge = Fx::from_int(100);
            f.gauge_mode = 900;
        }
        let inputs = random_inputs(&mut Rng::new(seed), 600);
        let delays = random_delays(
            &mut Rng::new(seed ^ 0x5eed),
            inputs.len(),
            1 + (seed % 8) as u32,
        );
        verify(&content, &initial, &inputs, &delays).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        // (How much of it was powered play, so the test is known to cover it.)
        let mut s = initial;
        for i in &inputs {
            sim_core::step(&mut s, &content, i);
            powered_frames += s.fighters.iter().filter(|f| f.gauge_mode > 0).count() as u32;
            bursts += s
                .projectiles
                .iter()
                .filter(|p| p.active && p.bursting)
                .count() as u32;
        }
    }
    assert!(powered_frames > 10_000, "{powered_frames}");
    assert!(bursts > 0, "a burst happened");
}
