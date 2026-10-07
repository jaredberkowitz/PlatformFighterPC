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
    const OWN: [Func; 9] = [
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
    ];
    concat::<9, 9, 18>(MATH, OWN)
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
