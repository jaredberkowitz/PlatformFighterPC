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

### 3. Needles rogue (the Sheik-like role)
- A light, fast fighter (weight 78) that strings hits together: quick kicks, a multi-hit up air that drags an airborne foe down with it,
  a stalling, plunging down air, and (planned) thrown needles, a pulling grenade, a vanish recovery and a flip-kick.
- Built to exercise the multi-hit engine (sim 49): up to eight hit groups, a rehit that clears only chosen groups, the autolink angle
  (367: the hit carries the foe along with the attacker instead of launching), set knockback, extra hitstun, per-hitbox SDI and landing hits.
- **As built (phase 1, sim 49):** the body (`FighterParams::rogue`) and the forward, up and down tilts and all five aerials on the
  reference's published data (`docs/sheik/REFERENCE_DATA.md`, which labels each number verified, derived or estimated).
- **Phase 2 (sim 50):** the jabs with the rapid jab and its finisher (scripts), the earlier game's dash attack (a pop-up), the two-hit
  forward and down smashes and the up smash, grabs, pummel and throws, and the reference's rolls and spot dodge.
- **Phase 3 (sim 51):** the specials: Needle Storm (charge, store with shield, throw up to six needles), our own pulling line in place
  of the reference's grenade (a hit reels the target in to just in front, for a follow-up), Vanish (an explosion, a teleport along
  the stick, a reappearance hit, then helpless) and Bouncing Fish (a flip into a heel kick that bounces back off a hit). The up air's
  drag now lands its target in front of the fighter.

### 4. Embers tumbler (the reference all-rounder's role)
- A short middleweight (98) with fast air speed, high floaty jumps and quick fists and feet: up-air juggles, a spinning multi-hit
  down air, a meteor overhand, angled forward tilt and smash, and (phase 2) a bouncing fireball, a turning cloak that reflects, a rising
  multi-hit punch and a pushing water pack.
- **As built (phase 1, sim 57):** the body (`FighterParams::tumbler`) and every normal on the reference's published data
  (`docs/movesets/mario/`), with three general extensions: early jab continuation (`Move::next_from`), angled moves (`Hitbox::aim`) and
  set weight (`Hitbox::set_weight`).
- **Phase 2 (sims 58 to 64):** the specials on the reference's data: a rising multi-hit punch (up), a bouncing fireball (neutral), a
  cape that turns its victim round and reflects (side), and a charged water pump that pushes without damage (down); and a wall jump.
  The general pieces they needed (per-move recovery rules, projectile arcs and bounces, reversing and push hits, fuller reflectors,
  once-per-airtime flags, scaled shots) are listed in `docs/movesets/SHARED_MECHANICS.md`. The longsword's and the claws' up specials
  were re-fitted to their references with the same rules.

### 5. Gauntlets sprinter (the reference speedster bruiser's role)
- A tall heavyweight (104) with the fastest run on the roster (2.55), a long 15-frame initial dash, a high fall speed and heavy hits:
  a jab flurry, a one-frame flying knee, a meteor stomp, and (phase 2) a wound-up punch that turns round, a dash that searches for its
  target, a leaping command grab that explodes and a diving kick.
- **As built (phase 1, sim 66, content schema 5):** the body (`FighterParams::sprinter`) and every normal on the reference's
  published data (`docs/movesets/captain_falcon/`, code in `sim-core/src/moves_gauntlets.rs`), with general extensions: sixteen `ext`
  slots, per-limb intangibility (`Move::part_intangible`), per-hitbox shieldstun (`Hitbox::shieldstun`) and the `presses` / `releases`
  script registers.
- **As built (phase 2, sims 67 to 71, merged at 72; sim 73):** the four specials on the reference's data: the punch (reversed by the
  stick on frame 15, never clanking, the aerial one slowing and nudged along 35 degrees), the search dash into an armored uppercut or
  a meteor overhand, the leaping command grab that explodes on everyone it reaches, the slide kick (crashing into walls, flying on over
  an edge) and the dive kick with its landing hit. General pieces: armor (`Move::armor`), frame-ranged air physics and per-move fast-fall
  and turn frames, search boxes (`HIT_SEARCH`, `Move::on_search`), command grabs (`Hitbox::command`, `Move::catch_move`), no-clank hits
  (`Hitbox::clanks`), off-edge versions (`Move::off_edge`), the `wall` script register, and the reference's exact shield stun for
  everyone. Travel distances are estimates (not published). Report: `docs/movesets/captain_falcon/IMPLEMENTATION_REPORT.md`.

### 6. Staff seer (the reference ranged caster's role)
- A tall middleweight (91) with a slow air speed (1.0) but quick air acceleration, a long-reaching staff and shield: spinning multi-hit
  tilts and aerials that carry a target, wings and a pillar of light for smashes with a wind that pushes, a shield-bash back air, and
  (phase 2) a blast set off where it points, a lock-on volley, a teleport and a counter that also reflects shots.
- **As built (phase 1, sim 67):** the body (`FighterParams::seer`, no wall jump, heavy landing 6) and every normal on the reference's
  published data (`docs/movesets/palutena/`, code in `sim-core/src/moves_staff.rs`), with two general extensions: angle types
  (`Hitbox::angle_type`: by box, forward, backward, by position; the older classes now carry their reference types too) and a signed
  hitstun modifier (`Hitbox::hitstun_add`). Since sim 73 its reference no-clank moves never clank (`Hitbox::clanks`). The specials
  are the claws' until phase 2.

### 7. Greatsword champion (the reference greatsword swordsman's role)
- A middleweight (100) with a huge two-handed sword and kicks: long, single strong hits rather than multi-hits (the forward smash's three
  quick hits are the exception), sword meteors in the forward and down air, a strong-then-lingering up air, and (phases 2 and 3) a slow
  sword wave, a combo slash that only carries on when it lands, a rising slash with a falling follow-up, and a gauge ("Boost" for now)
  that charges from damage and turns the next special into a finisher.
- **As built (phase 1, sim 83):** the body (`FighterParams::champion`, wall jump 1.3 / 2.4, heavy landing 3) and every normal on the
  reference's published data (`docs/movesets/cloud/`, code in `sim-core/src/moves_greatsword.rs`), on existing mechanics only. The
  specials are the claws' until phase 2.

### 8. Bracers hauler (the reference super-heavyweight brawler's role)
- A super-heavyweight (127) with huge bare arms: long-reaching tilts with intangible forearms, a jab hook into an uppercut, two
  meteors (the forward air's late fist and the down air's stomp), a backward kick that always sends backward, smashes that clank without
  rebounding, and trips on the down tilt and the down-aimed forward tilt; (phases 2 and 3) a wind-up punch that charges and stores, a
  headbutt that buries, a spinning clothesline, ground slaps and a throw that carries the catch anywhere.
- **As built (phase 1, sim 86):** the body (`FighterParams::hauler`, no wall jump, heavy landing 3, a bigger body [E]) and every
  normal on the reference's published data (`docs/movesets/donkey_kong/`, code in `sim-core/src/moves_bracers.rs`), with E20 trips and
  E3g no-rebound hits. The specials are the claws' until phase 2; the forward throw is a plain placeholder until the carry (phase 3).

### 9. Clutch bounder (the reference floaty heavyweight's role)
- A floaty heavyweight (104, the fastest air speed) with a towering second jump that shrugs off blows (it turns round with the stick
  held back): quick kicks, a tail, a headbutt forward smash with an intangible head, a bicycle-kick up smash, a flurry of twelve
  down-air kicks, a set-weight back air, and a long, slow tongue grab; (phases 2 and 3) lobbed shells that lift it, a crashing drop
  that scatters stars, a tongue that seals foes in a shell, and a rolling shell.
- **As built (phase 1, sim 89):** the body (`FighterParams::bounder`, no wall jump, heavy landing 6, the double jump's subtractive
  knockback armor on its frames 1-44 and its turnaround: E33 and the E1 knockback kind) and every normal on the reference's published
  data (`docs/movesets/yoshi/`, code in `sim-core/src/moves_clutch.rs`). The specials are the claws' until phase 2.

### 10. Buckler wanderer (the reference sword-and-shield adventurer's role)
- A heavy swordsman (104) with low jumps and a small shield that stops shots while he stands, walks or crouches: a long-reaching sword
  with its own tip, blade, arm and hilt boxes on every move, a two-part forward smash, a pogo down air, a dragging forward air, and the
  spin: a charged single slash on the ground, a rising multi-hit whirl in the air that carries a target round; (phases 2 and 3) a
  bow whose arrows lodge and can be picked up, a boomerang that comes back, a bomb set off from afar, and the sword wave at 0%.
- **As built (phase 1, sim 91):** the body (`FighterParams::wanderer`, the passive shield E14, no wall jump, heavy landing 3), every
  sword normal and both spins on the reference's published data (`docs/movesets/link/`, code in `sim-core/src/moves_buckler.rs`), with
  E34 position vectors, E36 (a follow-up keeps its charge), the E11 special-fall speed and acceleration, and E37 (up special and up
  smash straight out of a shield, for every class). The bow, boomerang and bomb are the claws' placeholders until phases 2 and 3.

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
