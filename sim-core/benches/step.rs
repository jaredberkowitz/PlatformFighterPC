use criterion::{criterion_group, criterion_main, Criterion};
use sim_core::fuzz::random_inputs;
use sim_core::{step, Content, GameState, Rng};
use std::hint::black_box;

/// Target from the plan: a 4-player step well under 1 ms, so an 8-frame rollback fits in a 16 ms frame.
fn bench_step(c: &mut Criterion) {
    let content = Content::placeholder();
    let inputs = random_inputs(&mut Rng::new(1), 256);
    let mut state = GameState::new(&content, 1, [0, 1, 0, 1]);
    let mut i = 0usize;

    c.bench_function("step_4_players", |b| {
        b.iter(|| {
            step(&mut state, &content, black_box(&inputs[i & 255]));
            i += 1;
        })
    });

    c.bench_function("snapshot_and_checksum", |b| {
        b.iter(|| {
            let saved = black_box(state);
            black_box(saved.checksum())
        })
    });
}

criterion_group!(benches, bench_step);
criterion_main!(benches);
