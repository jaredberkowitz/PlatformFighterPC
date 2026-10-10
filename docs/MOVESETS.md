# Movesets and weapons (design notes for Phase 3 and later)

## Principle

A character's moveset is chosen by its **weapon**, not hand-built per character. A character is
`body type` (physics, ECB, proportions) + `weapon` (the move list) + `cosmetics`. This fits the plan: content is
data (hitbox timelines, with scripting only for unusual specials), and the editors expose the same model.

Weapon families we expect to ship: swords of different lengths, claws, fists, magic, and later polearms, bows, hammers.
Each family defines the full move slot list; a specific weapon tunes numbers and may override individual slots.

## Move slots every weapon fills

Jab (and combo), forward/up/down tilt, forward/up/down smash, neutral/forward/back/up/down aerial, grab and throws,
dash attack, ledge attack, and four specials (neutral, side, up, down). The up special is what ends in `Helpless`
(already supported by the sim).

## First two movesets

The reference video is a competitive match between a longsword duelist and a blaster-and-claw brawler. We take the **gameplay roles** as a target and make
everything else original: names, animations, effects, hitbox shapes, numbers and look.

### 1. Longsword duelist (the reference swordsman's role)
- A long, thin, **disjointed** blade: a fast, safe poke game at the edge of its range.
- **Spacing is the skill:** the blade tip hits harder than the hilt, so positioning decides damage.
- Light and fast on the ground, good aerial mobility, strong edge-guarding, weaker when cornered up close.
- Specials sketch: a dash slash (side), a rising slash (up, then helpless), a counter that reflects an incoming hit
  (down), a charged lunge (neutral).
- **Specials as built (kit completion, sim v20):** neutral is **Shield Breaker** (hold special to charge, let go to thrust: 8-9%
  uncharged, about 24% fully charged), side is **Dancing Blade** (up to four hits, each started by pressing special again; the stick
  picks the rising or low variants), up is the rising slash, and down is **Counter** (a 22-frame window that catches a hit and answers
  with 1.2 times its damage, at least 8%). Shield Breaker and Dancing Blade are scripts (`sim-core/src/scripts/`), Counter is a data
  feature. See `docs/COMBAT.md`, "Kit completion".
- Wants from the sim: hitboxes with a **sweet spot / sour spot** by region of the blade, per-hitbox damage and knockback,
  hitbox priority (disjointed vs hurtbox), a counter state that reacts to hits.

### 2. Blaster-and-claw brawler (the reference brawler's role)
- Heavier hits and a **fast fall**, with a ranged blaster that pressures from a distance.
- Strong close-range kill moves and a good **reflector**; a ground-covering drill-style up special.
- Wants from the sim: **projectiles** (fixed-capacity pool already in the state design), a reflect state that flips a
  projectile's owner, fast-fall tuning, and multi-hit moves.

## What this needs from the engine (Phase 3 list)

1. Hitboxes and hurtboxes (capsules or circles in fixed point), per-frame active windows, priority and clanking.
2. Knockback, hitstun, hitlag, DI and SDI; shield and grabs.
3. A `Weapon` content type (move slots to frame data) plus per-weapon overrides.
4. A projectile pool in `GameState`, hashed like everything else.
5. Counter and reflect states.
6. Training-mode view of hitboxes, frame data and the stick/cone display (the stick display is already done).

## Legal reminders

Game mechanics are not protected, so matching a *role* is fine. Do not copy any character, move name, animation,
sound or visual design, and do not use the reference characters' names anywhere in the game.
