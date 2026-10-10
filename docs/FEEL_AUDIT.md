# Combat feel audit: where we differ from the reference game, and why

October 2026. Each difference is placed in one layer (**mechanics** = the simulation's numbers and rules, **animation**, **art** = effects and
visual design, **audio**, **camera**) with the evidence for it, how sure we are, and the fix. Reference numbers are from SmashWiki and
ultimateframedata.com.

## What already matches (checked, so not the cause)

| Area | Status |
| --- | --- |
| Knockback formula, 1.2× damage in 1v1, launch speed (0.03 × KB) and its decay (0.051 a frame) | Same formulas |
| Hitlag `floor(damage × 0.65 + 6) × hitbox multiplier`, shield 0.67×, crouch 0.67×, cap 30, electric 1.5× | Same |
| Hitstun `floor(KB × 0.4) − 1`, tumble at 80 KB, the launch speed-up (balloon) for long hitstun | Same |
| Hitstun cancel (air dodge from frame 40, aerial from 45), techs (11-frame window, 40-frame lockout) | Same |
| SDI: 2 units a pulse, every 4 frames, ×1.15 after five hits; ASDI on electric hits only | Same |
| Shield: 50 HP, 0.15 a frame drain, 0.08 regen, shield stun `0.8 × damage × type + 2` | Same |
| Stale moves (nine-slot queue, the reference reductors, 1.05 fresh bonus) | Same |
| Movement: jump arcs (short/full hop airtimes within a frame), fast fall, the 9-frame buffer, landing lag, air dodges, ledges | Same (see `PHASE1_MOVEMENT.md`) |
| The sword fighter's and the claws fighter's aerials and tilts (frames, damage, angles, BKB/KBG, landing lag, autocancel) | Same. A short hop fits one aerial and a full hop two only for a fast one, as in the reference |

So the numbers players feel most (frame data, hitstun, launch) are not where the gap is. It is mostly in what the player *sees and hears*
around those numbers.

## The differences

### Animation

**A1. The victim keeps its old pose through the hit freeze.** *Done: a clip change during hitlag starts at once (the freeze never
moved the cross-fade on).* *High impact, small fix, certain.* The model only advances its animation when
hitlag is 0, and a clip change cross-fades over 0.1 s, so on the frame a hit lands the victim freezes in whatever it was doing (standing,
mid-swing) and only bends into the hurt pose *after* the freeze. In the reference the victim is already in its damage pose for the whole
freeze, shaking. This is the single biggest reason hits don't land with weight. Fix: snap straight to the damage pose when hitlag starts.

**A2. One damage pose for every hit; none for a flinch.** *Partly done: a hit that does not launch snaps the chest and head back
(a flinch); a tumble whirls faster the faster the launch, about the middle of the body. Still one launch pose.* *High impact, medium
fix, certain.* Every launch uses the same `hurt` pose and a hit
too weak to cause hitstun (a flinch, many multi-hit hits) shows no reaction at all, only a shake. The reference has a set: a light flinch
(the head snapping back, by hit height: high, middle, low), damage in the air, a launch pose with the body thrown along the trajectory, and a
tumbling roll for strong launches. Fix: a reaction set in the view: a short flinch pose, a launch pose aimed along the velocity, tumble spin
speed from the launch speed.

**A3. Generic poses for many moves.** *Medium impact, large fix.* The iconic aerials and the sword tilts have their own poses, but grabs,
throws, most specials, jabs and the bruiser's kit share generic clips. Fix: per-move poses, best done in a directed Blender session.

**A4. One landing pose.** *Done: landing out of an aerial stumbles, deeper and pitched forward, for its landing lag.* *Low impact,
small fix.* Light landings, heavy landings and aerial landing lag all show the same crouch; the
reference's aerial landing lag is a visible stumble. Fix: a heavier landing pose for landing lag after an aerial or a fast fall.

### Art and effects

**E1. Hit effects appear at the victim's middle, not where the hit connected.** *Done (seventeenth art pass).* *Medium-high impact, small fix, certain.* We know where the
live hitbox is (the attacker's move tip), but the spark, rings and spike are placed at the victim's body centre, so a sword tip and a
point-blank kick spark in the same place. Fix: place them where the hitbox meets the victim.

**E2. Effects are soft and pastel.** *Done: drawn bursts and slashes by kind (seventeenth art pass, `docs/ART_PIPELINE.md`).* *High impact, medium fix.* Ours are thin rings and additive discs at partial opacity in the attacker's
colour. The reference's hit effects are large, opaque, high-contrast (a white core, a yellow-orange starburst, often outlined), last 6 to 10
frames, scale with damage, and come in kinds: slashes for blades, stars for blows, sparks for electric, flames for fire. Fix: drawn,
opaque effect sprites (SVG like the faces) by kind and size.

**E3. No DI indicator.** *Done: a blue streak along the launch line as a strong launch begins (`Effects.launch_line`).* *Low
impact, small fix.* The reference flashes a blue streak along the final launch angle. Fix: draw it at the end of
hitlag.

### Audio

**S1. Two synthesised hit sounds for everything.** *Done: layered CC0 sounds by kind and size (`docs/AUDIO.md`).* *Very high impact, medium fix, certain.* `sfx.gd` makes every hit either a 0.16 s or a
0.34 s sine sweep with noise. The reference layers a sharp transient (crack), a body (thud), a type layer (slash, punch, kick, electric) and
a size tier, plus crowd reactions on strong hits; sound carries much of "weight". Fix: layered sounds by kind and size, either better
synthesis or CC0 sound files (would need a download; I would ask first).

**S2. The swing sound plays when the move starts, not when it swings.** *Done.* *Medium impact, small fix.* `whoosh` plays on the first frame of the
attack; for a slow move it is heard long before the blade moves. Fix: play it a few frames before the first active frame.

### Mechanics

**M1. DI is about twice as strong.** *Done: 10 degrees (sim v34).* *Medium impact, small fix, certain.* We allow 18° (the older game's value); the reference uses 0.17 rad,
about 9.7°. Launches are too steerable, so combos and kills are less reliable. Fix: 10°.

**M2. No launch speed influence.** *Done (sim v34): `lsi_up`, `lsi_down`, `lsi_vertical` in the ruleset.* *Low-medium impact,
small fix, certain.* The reference scales launch speed ×1.095 when holding up and ×0.92
holding down (not for near-vertical or near-horizontal launches, 65°-115° and 245°-295°). Fix: add it.

**M3. Hurtboxes don't follow the body.** *Done (sim v34): lower in a crouch (`crouch_height`), and a limb hurtbox reaching toward the
hit for a `limbs` moveset (the claws).* *Medium-high impact, medium-large fix, certain.* A fighter is three fixed circles up its middle in
every state. In the reference hurtboxes follow the body: crouching ducks under high attacks, and an extended arm or leg can be hit (blades
excepted). With the chunkier model, hits that visibly touch a limb can miss and a crouch dodges nothing. The animation can't drive the
hurtboxes (the simulation must stay deterministic), so the fix is per-state hurtbox shapes in content data: lower for crouch, and a limb
circle along the hit for punches and kicks.

**M4. Hitbox positions were matched by eye.** *Medium impact, medium fix.* Positions and sizes were placed against our older, thinner model;
the model has changed since. Fix: re-check every move with the hitbox overlay (F1) against the current model.

**M5. Ground speed is 90% of the reference.** *Done: 100% since sim v33, with the committed initial dash (`PHASE1_MOVEMENT.md`).* `GROUND_SPEED_PERCENT` was set to 90 early on to
tame a slippery keyboard; dashes and runs are slower than the reference's. Fix: try 100 now that walking is analog.

### Camera

**C1. Close framing.** *Low impact.* The camera comes in to 19 units when fighters are close; small hits read as large. A matter of taste;
revisit after the effects work.

## Suggested order

1. **A1** snap to the damage pose for the hit freeze (small, the biggest single gain).
2. **E1 + E2** effects at the contact point, then opaque effects by kind and size.
3. **S1 + S2** hit sounds by kind and size; the swing sound on the swing.
4. **M1 + M2** DI to 10°, add launch speed influence (simulation change; replays reset).
5. **A2** the damage-reaction set.
6. **M3 + M4** state hurtboxes and limb hurtboxes; re-align hitboxes to the model.
7. **M5** decide on ground speed.
8. **A3** per-move poses (directed Blender session).
