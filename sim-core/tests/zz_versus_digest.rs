//! Temporary: a digest of versus play that does not depend on the state's layout (compare across branches).
use sim_core::content::Content;
use sim_core::hash::{StateHash, StateHasher};
use sim_core::rng::Rng;
use sim_core::state::GameState;
use sim_core::{step, MAX_FIGHTERS};

fn digest(s: &GameState, h: &mut StateHasher) {
    for i in 0..MAX_FIGHTERS {
        let f = &s.fighters[i];
        f.pos.hash_into(h);
        f.vel.hash_into(h);
        f.kb_vel.hash_into(h);
        f.percent.hash_into(h);
        f.shield_hp.hash_into(h);
        h.write_bytes(format!("{:?}", f.state).as_bytes());
        h.write_u16(f.state_frame);
        h.write_u8(f.stocks);
        h.write_u8(f.hitlag);
        h.write_u16(f.hitstun);
        h.write_u8(f.move_id);
        h.write_bool(f.active);
        h.write_i8(f.grab_with);
        h.write_u8(f.invuln);
        h.write_u8(f.charge);
        h.write_i8(f.facing);
        h.write_i8(f.platform);
        h.write_i8(f.ledge);
        h.write_u8(f.air_jumps_left);
        h.write_i8(f.holding);
        f.gauge.hash_into(h);
        f.wing_points.hash_into(h);
    }
    for p in &s.projectiles {
        h.write_bool(p.active);
        p.pos.hash_into(h);
        p.vel.hash_into(h);
        h.write_u8(p.owner);
        h.write_u8(p.age);
    }
    for it in &s.items {
        h.write_bool(it.active);
        it.pos.hash_into(h);
        it.vel.hash_into(h);
        it.hp.hash_into(h);
    }
    for o in &s.ledge_owner {
        h.write_i8(*o);
    }
    h.write_i8(s.winner);
    s.rng.hash_into(h);
}

#[test]
fn versus_digest() {
    let content = Content::placeholder();
    let n = content.fighters.len() as u8;
    let mut total = StateHasher::new();
    for seed in 0..60u64 {
        let a = (seed % u64::from(n)) as u8;
        let b = ((seed * 7 + 3) % u64::from(n)) as u8;
        let c = ((seed * 5 + 1) % u64::from(n)) as u8;
        let d = ((seed * 11 + 2) % u64::from(n)) as u8;
        let active = if seed % 3 == 0 { 0b1111 } else { 0b0011 };
        let mut s = GameState::new_with_active(&content, seed, [a, b, c, d], active);
        let inputs = sim_core::fuzz::random_inputs(&mut Rng::new(seed + 100), 1800);
        let mut h = StateHasher::new();
        for i in &inputs {
            step(&mut s, &content, i);
            digest(&s, &mut h);
        }
        let v = h.finish();
        total.write_u64(v);
        if seed < 5 {
            println!("seed {seed}: {v:016x}");
        }
    }
    println!("VERSUS DIGEST {:016x}", total.finish());
}
