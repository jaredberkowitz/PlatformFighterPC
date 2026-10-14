//! The whitelist: everything a script can read and call. Adding an entry here is the only way to give scripts a new
//! ability, and each one must be implemented by the host in the simulation.
//!
//! All numbers are 16.16 fixed point. Distances are world units, angles are degrees, `frame`, `age` and `life` are
//! frame counts. Horizontal speeds and the stick are **relative to the way the fighter faces** (positive is
//! forward); positions are absolute.

/// Which kind of thing a script drives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    /// Runs every frame of a fighter's move.
    Fighter = 0,
    /// Runs every frame of a projectile.
    Projectile = 1,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Fighter => "fighter",
            Kind::Projectile => "projectile",
        }
    }

    /// Persistent variables a script of this kind may declare with `var`.
    pub const fn max_vars(self) -> usize {
        match self {
            Kind::Fighter => 4,
            Kind::Projectile => 2,
        }
    }

    pub fn registers(self) -> &'static [Entry] {
        match self {
            Kind::Fighter => FIGHTER_REGISTERS,
            Kind::Projectile => PROJECTILE_REGISTERS,
        }
    }

    pub fn functions(self) -> &'static [Func] {
        match self {
            Kind::Fighter => FIGHTER_FUNCTIONS,
            Kind::Projectile => PROJECTILE_FUNCTIONS,
        }
    }
}

/// A named read-only value.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    pub name: &'static str,
    pub id: u8,
    pub doc: &'static str,
}

/// A callable function. Native functions (ids below [`FIRST_HOST_FUNC`]) are done by the VM itself.
#[derive(Clone, Copy, Debug)]
pub struct Func {
    pub name: &'static str,
    pub id: u8,
    pub arity: u8,
    pub doc: &'static str,
}

/// Function ids at or above this are performed by the host.
/// The most arguments a function takes (the VM passes them in a buffer this long).
pub const MAX_ARITY: usize = 8;

pub const FIRST_HOST_FUNC: u8 = 16;

// ---- Native math (done inside the VM) ----
pub const F_ABS: u8 = 0;
pub const F_MIN: u8 = 1;
pub const F_MAX: u8 = 2;
pub const F_CLAMP: u8 = 3;
pub const F_SIGN: u8 = 4;
pub const F_FLOOR: u8 = 5;
pub const F_SQRT: u8 = 6;

// ---- Host math ----
pub const F_SIN: u8 = 16;
pub const F_COS: u8 = 17;

// ---- Fighter scripts ----
pub const F_SET_VEL: u8 = 32;
pub const F_ADD_VEL: u8 = 33;
pub const F_SPAWN: u8 = 34;
pub const F_INTANGIBLE: u8 = 35;
pub const F_END: u8 = 36;
pub const F_TURN: u8 = 37;
pub const F_REHIT: u8 = 38;
pub const F_STALL: u8 = 39;
pub const F_GOTO: u8 = 40;
pub const F_STORE: u8 = 41;
pub const F_AIR_FLAG: u8 = 42;
pub const F_SET_AIR_FLAG: u8 = 43;
pub const F_SPAWN_SCALED: u8 = 44;
pub const F_SPAWN_AT: u8 = 45;
pub const F_LOCK: u8 = 46;
pub const F_BOOST_ADD: u8 = 47;
pub const F_SET_CHARGE: u8 = 50;
pub const F_DETONATE: u8 = 51;
pub const F_OWNS: u8 = 52;
pub const F_AIR_MARK: u8 = 53;
pub const F_SET_AIR_MARK: u8 = 54;

// ---- Projectile scripts ----
pub const F_P_SET_VEL: u8 = 48;
pub const F_P_KILL: u8 = 49;

// ---- Registers: fighter ----
pub const R_FRAME: u8 = 0;
pub const R_FACING: u8 = 1;
pub const R_X: u8 = 2;
pub const R_Y: u8 = 3;
pub const R_VX: u8 = 4;
pub const R_VY: u8 = 5;
pub const R_PERCENT: u8 = 6;
pub const R_STICK_X: u8 = 7;
pub const R_STICK_Y: u8 = 8;
pub const R_GROUNDED: u8 = 9;
pub const R_ATTACK: u8 = 10;
pub const R_ATTACK_TAP: u8 = 11;
pub const R_SPECIAL: u8 = 12;
pub const R_SPECIAL_TAP: u8 = 13;
pub const R_SHIELD: u8 = 14;
pub const R_CHARGE: u8 = 15;
pub const R_HIT: u8 = 16;
pub const R_STORED: u8 = 17;
pub const R_JUMP: u8 = 18;
pub const R_PRESSES: u8 = 19;
pub const R_RELEASES: u8 = 20;
pub const R_WALL: u8 = 21;
pub const R_FLICK: u8 = 22;
pub const R_LOCKED: u8 = 23;
pub const R_LOCK_X: u8 = 24;
pub const R_LOCK_Y: u8 = 25;
pub const R_BOOST: u8 = 26;
pub const R_BOOSTING: u8 = 27;
pub const R_GRAB: u8 = 28;
pub const R_AIM: u8 = 29;
pub const R_FAST_FALL: u8 = 30;
pub const R_OWNED: u8 = 31;
pub const R_HOLDING: u8 = 32;
pub const R_BALLOONS: u8 = 33;
pub const R_POCKETED: u8 = 34;
pub const R_PULL_WAIT: u8 = 35;
pub const R_JUMP_TAP: u8 = 36;

// ---- Registers: projectile ----
pub const R_PX: u8 = 0;
pub const R_PY: u8 = 1;
pub const R_PVX: u8 = 2;
pub const R_PVY: u8 = 3;
pub const R_AGE: u8 = 4;
pub const R_LIFE: u8 = 5;
pub const R_OWNER_X: u8 = 6;
pub const R_OWNER_Y: u8 = 7;
pub const R_TARGET_X: u8 = 8;
pub const R_TARGET_Y: u8 = 9;
pub const R_HAS_TARGET: u8 = 10;
pub const R_OWNER_STICK_X: u8 = 11;
pub const R_OWNER_STICK_Y: u8 = 12;

const fn reg(name: &'static str, id: u8, doc: &'static str) -> Entry {
    Entry { name, id, doc }
}

const fn func(name: &'static str, id: u8, arity: u8, doc: &'static str) -> Func {
    Func {
        name,
        id,
        arity,
        doc,
    }
}

pub const FIGHTER_REGISTERS: &[Entry] = &[
    reg("frame", R_FRAME, "frames since the move started"),
    reg("facing", R_FACING, "1 facing right, -1 facing left"),
    reg("x", R_X, "position (feet), absolute"),
    reg("y", R_Y, "position (feet), absolute"),
    reg("vx", R_VX, "horizontal speed, positive forward"),
    reg("vy", R_VY, "vertical speed, positive up"),
    reg("percent", R_PERCENT, "damage taken"),
    reg("stick_x", R_STICK_X, "stick, -1..1, positive forward"),
    reg("stick_y", R_STICK_Y, "stick, -1..1, positive up"),
    reg("grounded", R_GROUNDED, "1 on the ground, else 0"),
    reg("attack", R_ATTACK, "attack button held"),
    reg(
        "attack_tap",
        R_ATTACK_TAP,
        "attack button pressed this frame",
    ),
    reg("special", R_SPECIAL, "special button held"),
    reg(
        "special_tap",
        R_SPECIAL_TAP,
        "special button pressed this frame",
    ),
    reg("shield", R_SHIELD, "shield button held"),
    reg("charge", R_CHARGE, "frames stalled so far"),
    reg("hit", R_HIT, "1 once this move has hit someone"),
    reg(
        "stored",
        R_STORED,
        "a charge kept from an earlier use (see store), 0 to 255",
    ),
    reg("jump", R_JUMP, "jump button held"),
    reg(
        "presses",
        R_PRESSES,
        "attack presses since the attack began (its own press is 1), hitlag included",
    ),
    reg(
        "releases",
        R_RELEASES,
        "attack releases since the attack began, hitlag included",
    ),
    reg(
        "wall",
        R_WALL,
        "1 if the fighter's last movement step was stopped by a wall, else 0",
    ),
    reg(
        "flick",
        R_FLICK,
        "1 if the current special was started with a flick of the stick (a smash-style input) or the strong button",
    ),
    reg(
        "locked",
        R_LOCKED,
        "1 once lock found a target in this attack, else 0",
    ),
    reg(
        "lock_x",
        R_LOCK_X,
        "where the lock was made: the target's chest then, absolute (0 without a lock)",
    ),
    reg(
        "lock_y",
        R_LOCK_Y,
        "where the lock was made: the target's chest then, absolute (0 without a lock)",
    ),
    reg(
        "boost",
        R_BOOST,
        "the fighter's gauge (the body's Boost gauge), 0 to its max (0 for a body without one)",
    ),
    reg(
        "boosting",
        R_BOOSTING,
        "1 while the gauge's powered mode runs, else 0",
    ),
    reg("grab", R_GRAB, "grab button held"),
    reg(
        "aimed",
        R_AIM,
        "how the attack was aimed as it started: 1 up, -1 down, 0 straight (an angled forward tilt or smash)",
    ),
    reg(
        "fast_fall",
        R_FAST_FALL,
        "1 while the fighter is fast falling (since its last jump, landing or hit), else 0",
    ),
    reg(
        "owned",
        R_OWNED,
        "how many of this move's projectiles (or tools of its kind) the fighter has out, and its items of that kind, as the frame began",
    ),
    reg(
        "holding",
        R_HOLDING,
        "the kind of the item in the fighter's hand (its item's kind number), 0 for none, as the frame began",
    ),
    reg(
        "balloons",
        R_BALLOONS,
        "how many balloons the fighter carries in its balloon move (a move with balloons), 0 out of one",
    ),
    reg(
        "pocketed",
        R_POCKETED,
        "what the fighter has put away in its pocket: 0 nothing, 1 a projectile, 2 an item",
    ),
    reg(
        "pull_wait",
        R_PULL_WAIT,
        "frames before a move's toss lets the fighter make another tossed item (a move's toss wait), 0 when it can",
    ),
    reg(
        "jump_tap",
        R_JUMP_TAP,
        "1 only on the frame the jump button went down",
    ),
];

pub const PROJECTILE_REGISTERS: &[Entry] = &[
    reg("px", R_PX, "position, absolute"),
    reg("py", R_PY, "position, absolute"),
    reg("pvx", R_PVX, "velocity, absolute (positive is right)"),
    reg("pvy", R_PVY, "velocity, absolute (positive is up)"),
    reg("age", R_AGE, "frames since it appeared"),
    reg("life", R_LIFE, "frames it lasts"),
    reg(
        "owner_x",
        R_OWNER_X,
        "position of the fighter it belongs to",
    ),
    reg(
        "owner_y",
        R_OWNER_Y,
        "position of the fighter it belongs to",
    ),
    reg(
        "target_x",
        R_TARGET_X,
        "position of the nearest fighter that is not its owner",
    ),
    reg(
        "target_y",
        R_TARGET_Y,
        "position of the nearest fighter that is not its owner",
    ),
    reg("has_target", R_HAS_TARGET, "1 if there is such a fighter"),
    reg(
        "owner_stick_x",
        R_OWNER_STICK_X,
        "the stick of the fighter it belongs to, -1..1, positive is right (absolute)",
    ),
    reg(
        "owner_stick_y",
        R_OWNER_STICK_Y,
        "the stick of the fighter it belongs to, -1..1, positive is up",
    ),
];

const MATH: [Func; 9] = [
    func("abs", F_ABS, 1, "absolute value"),
    func("min", F_MIN, 2, "smaller of two"),
    func("max", F_MAX, 2, "larger of two"),
    func("clamp", F_CLAMP, 3, "clamp(value, low, high)"),
    func("sign", F_SIGN, 1, "-1, 0 or 1"),
    func("floor", F_FLOOR, 1, "round down to a whole number"),
    func("sqrt", F_SQRT, 1, "square root (0 for negatives)"),
    func("sin", F_SIN, 1, "sine of an angle in degrees"),
    func("cos", F_COS, 1, "cosine of an angle in degrees"),
];

const fn concat<const A: usize, const B: usize, const N: usize>(
    a: [Func; A],
    b: [Func; B],
) -> [Func; N] {
    let mut out = [a[0]; N];
    let mut i = 0;
    while i < A {
        out[i] = a[i];
        i += 1;
    }
    let mut j = 0;
    while j < B {
        out[A + j] = b[j];
        j += 1;
    }
    out
}

pub const FIGHTER_FUNCTIONS: &[Func] = &{
    const OWN: [Func; 21] = [
        func(
            "set_vel",
            F_SET_VEL,
            2,
            "set_vel(forward, up): move at this speed this frame, like a scripted motion",
        ),
        func(
            "add_vel",
            F_ADD_VEL,
            2,
            "add_vel(forward, up): change the current speed",
        ),
        func(
            "spawn",
            F_SPAWN,
            4,
            "spawn(forward, up, speed_forward, speed_up): fire the move's projectile",
        ),
        func(
            "intangible",
            F_INTANGIBLE,
            1,
            "intangible(frames): cannot be hit for this many frames",
        ),
        func("end", F_END, 0, "end the move now"),
        func("turn", F_TURN, 0, "face the other way"),
        func(
            "rehit",
            F_REHIT,
            0,
            "let the move's hitboxes hit the same fighters again",
        ),
        func(
            "stall",
            F_STALL,
            0,
            "hold the move on this frame (at most the charge limit in total)",
        ),
        func(
            "goto",
            F_GOTO,
            1,
            "goto(move): switch to another move slot (0 = jab, see the move list)",
        ),
        func(
            "store",
            F_STORE,
            1,
            "store(n): keep a charge of n (0 to 255) for the next use, read back as stored; lost on a KO",
        ),
        func(
            "air_flag",
            F_AIR_FLAG,
            1,
            "air_flag(n): 1 if once-per-airtime flag n (0 to 7) is set, else 0",
        ),
        func(
            "set_air_flag",
            F_SET_AIR_FLAG,
            1,
            "set_air_flag(n): set once-per-airtime flag n (0 to 7); landing, a ledge or being hit clears them all",
        ),
        func(
            "spawn_scaled",
            F_SPAWN_SCALED,
            6,
            "spawn_scaled(forward, up, speed_forward, speed_up, knockback, size): spawn with extra base knockback (0 to 255) and a size in percent (10 to 400)",
        ),
        func(
            "spawn_at",
            F_SPAWN_AT,
            4,
            "spawn_at(x, y, speed_forward, speed_up): fire the move's projectile at this absolute point, however the fighter moves this frame",
        ),
        func(
            "lock",
            F_LOCK,
            1,
            "lock(range): 1 if this attack has a lock; else looks for the nearest opponent in front that can be hit, chest within range of the chest, nothing solid between, and locks onto its chest there (it stays put)",
        ),
        func(
            "boost_add",
            F_BOOST_ADD,
            1,
            "boost_add(n): add n units to the fighter's gauge (negative drains), between 0 and its max; filling it starts the powered mode; nothing while the mode runs",
        ),
        func(
            "set_charge",
            F_SET_CHARGE,
            1,
            "set_charge(n): set this move's charge to n frames (0 to 255), read back as charge; a move with charge_add deals more with it",
        ),
        func(
            "detonate",
            F_DETONATE,
            0,
            "detonate(): every item the fighter made that can burst, and that no other fighter holds, bursts after its blast's delay",
        ),
        func(
            "owns",
            F_OWNS,
            1,
            "owns(kind): how many live items and projectiles of this kind the fighter made, as the frame began (pocketed ones too)",
        ),
        func(
            "air_mark",
            F_AIR_MARK,
            1,
            "air_mark(n): 1 if once-per-airtime mark n (0 to 7) is set, else 0",
        ),
        func(
            "set_air_mark",
            F_SET_AIR_MARK,
            1,
            "set_air_mark(n): set once-per-airtime mark n (0 to 7); landing or a ledge clears them all, a hit does not",
        ),
    ];
    concat::<9, 21, 30>(MATH, OWN)
};

pub const PROJECTILE_FUNCTIONS: &[Func] = &{
    const OWN: [Func; 2] = [
        func(
            "set_vel",
            F_P_SET_VEL,
            2,
            "set_vel(x, y): new velocity, absolute",
        ),
        func("kill", F_P_KILL, 0, "remove the projectile"),
    ];
    concat::<9, 2, 11>(MATH, OWN)
};

// Every function's arguments fit the VM's buffer.
const _: () = {
    let mut i = 0;
    while i < FIGHTER_FUNCTIONS.len() {
        assert!(FIGHTER_FUNCTIONS[i].arity as usize <= MAX_ARITY);
        i += 1;
    }
    let mut j = 0;
    while j < PROJECTILE_FUNCTIONS.len() {
        assert!(PROJECTILE_FUNCTIONS[j].arity as usize <= MAX_ARITY);
        j += 1;
    }
};
