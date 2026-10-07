//! A simulated network for tests and fuzzing: two ends of a link that delay, drop, duplicate and reorder datagrams, all
//! driven by a seeded generator so a failing run can be replayed exactly. Single-threaded and clock-free: time is the
//! number of times the shared [`Clock`] was advanced.

use crate::peer::Link;
use sim_core::Rng;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub struct LinkParams {
    /// Fewest and most ticks a datagram takes to arrive (the spread causes reordering).
    pub min_latency: u32,
    pub max_latency: u32,
    /// Chance in percent that a datagram never arrives.
    pub loss_percent: u32,
    /// Chance in percent that a datagram arrives twice.
    pub duplicate_percent: u32,
}

impl LinkParams {
    pub fn perfect() -> LinkParams {
        LinkParams {
            min_latency: 0,
            max_latency: 0,
            loss_percent: 0,
            duplicate_percent: 0,
        }
    }
}

struct Shared {
    now: u64,
    seq: u64,
    rng: Rng,
    params: LinkParams,
    /// Datagrams heading to side 0 and side 1: (arrival tick, sequence number, bytes).
    queues: [Vec<(u64, u64, Vec<u8>)>; 2],
}

/// Advances the simulated time of a link pair.
#[derive(Clone)]
pub struct Clock(Rc<RefCell<Shared>>);

impl Clock {
    pub fn advance(&self) {
        self.0.borrow_mut().now += 1;
    }

    /// Changes the link conditions mid-run (a lag spike, a dead patch).
    pub fn set_params(&self, params: LinkParams) {
        self.0.borrow_mut().params = params;
    }
}

pub struct TestLink {
    shared: Rc<RefCell<Shared>>,
    side: usize,
}

/// Two connected ends and the clock that drives them.
pub fn pair(params: LinkParams, seed: u64) -> (TestLink, TestLink, Clock) {
    let shared = Rc::new(RefCell::new(Shared {
        now: 0,
        seq: 0,
        rng: Rng::new(seed),
        params,
        queues: [Vec::new(), Vec::new()],
    }));
    (
        TestLink {
            shared: shared.clone(),
            side: 0,
        },
        TestLink {
            shared: shared.clone(),
            side: 1,
        },
        Clock(shared),
    )
}

impl Link for TestLink {
    fn send(&mut self, bytes: &[u8]) {
        let mut s = self.shared.borrow_mut();
        let params = s.params;
        let copies = if s.rng.range(100) < params.duplicate_percent {
            2
        } else {
            1
        };
        for _ in 0..copies {
            if s.rng.range(100) < params.loss_percent {
                continue;
            }
            let spread = params.max_latency.saturating_sub(params.min_latency);
            let latency = params.min_latency + s.rng.range(spread + 1);
            let at = s.now + u64::from(latency);
            s.seq += 1;
            let seq = s.seq;
            let to = 1 - self.side;
            s.queues[to].push((at, seq, bytes.to_vec()));
        }
    }

    fn recv(&mut self) -> Option<Vec<u8>> {
        let mut s = self.shared.borrow_mut();
        let now = s.now;
        let q = &mut s.queues[self.side];
        let best = q
            .iter()
            .enumerate()
            .filter(|(_, (at, _, _))| *at <= now)
            .min_by_key(|(_, (at, seq, _))| (*at, *seq))
            .map(|(i, _)| i)?;
        Some(q.remove(best).2)
    }
}
