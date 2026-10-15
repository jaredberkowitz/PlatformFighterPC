# 1v1 against the reference game: what is left

Executive review of standard one-on-one play (movement, combat, defence, controls, match rules, presentation), 14 October 2026, at
sim 111. Online play and other modes are out of scope. Sources: SmashWiki and ultimateframedata.com (reference version 13.0.1), the code on
`main`, and the earlier audits (`FEEL_AUDIT.md`, `PHASE1_MOVEMENT.md`, `COMBAT.md`, `movesets/SHARED_MECHANICS.md`).

**Verdict.** The numbers players feel most already match the reference: knockback, hitstun, hitlag, launch speed and decay, DI / SDI / LSI,
rage, staleness, the 1v1 multiplier, jump arcs, fast fall, the 9-frame buffer, landing lag and autocancels, ledge rules, techs, shield
health and stun, and the ported fighters' frame data. What is left is a short list of missing mechanics (mostly defensive), the controller
setup, and 1v1 match rules. None of it means rebuilding anything.

## Already matching (checked, leave alone)

- Knockback formula (with sim 97's percent and short-hop rules), launch speed 0.03 x KB with 0.051 decay, the launch speed-up, tumble at 80.
- Hitstun `floor(KB x 0.4) - 1`, hitstun cancel (air dodge 40, aerial 45), hitlag `floor(d x 0.65 + 6)` and its multipliers.
- DI 10 degrees, LSI, SDI and ASDI, crouch cancel, rage, the nine-slot stale queue, the fresh bonus.
- Movement: 3-frame jump squat, short/full hop airtimes, short-hop aerial macro, fast fall, air dodges and wavedash, landing lag, the
  committed initial dash, dash dance and foxtrot (sim 103/106), RAR, B-reverse, wall jumps, platform drops.
- Ledges: grab box, intangibility from airtime and damage, regrab limit, ledge trump, get-up / roll / jump / attack, hang time.
- Techs (floor, wall, ceiling), missed techs and knockdown get-ups, trips, bury, grabs, pummels, throws, grab mashing.
- Shield: 50 HP, drain, regen, stun `0.8 x d x mult + 2`, pushback, break and its stun; out of shield (sim 109); smash charge (60 frames,
  1.4x); clanks.
- Presentation: hit freeze in the damage pose, contact-point effects by kind, layered hit sounds, camera shake, the game-ending KO slow
  motion and close-up, the DI line, the damage number's shake and heat colour.

## Gaps

Impact: how much a player would notice. Effort: S (an evening), M (a few sims), L (a larger system).

### Controls (the largest practical gap: the owner now plays on a controller)

| # | Gap | Reference | Ours | Impact | Effort |
| --- | --- | --- | --- | --- | --- |
| C1 | Tap jump (**done**: an option, off by default) | A quick upward flick jumps (on by default, can be turned off) | Jump only from a button | High | S |
| C2 | Controller layout (**done**: bumpers grab, triggers shield, remapping, right-stick modes) | Grab on a shoulder button, every button remappable, the right stick selectable as smash, tilt, special or off | Grab on the right-stick click, no controller remapping, the right stick is smash/aerial only | High | M |
| C3 | Rumble (**done**) | Controller rumble on hits, shield breaks and KOs (an option) | None | Low-Med | S |
| C4 | Hardware | Tested on real pads | Controllers never tried on hardware (known gap) | High risk | S (a playtest) |

### Defence

| # | Gap | Reference | Ours | Impact | Effort |
| --- | --- | --- | --- | --- | --- |
| D1 | Shield tilting and shield pokes (**done, sim 134**: the bubble the game draws is the sim's; a hit whose box misses it but touches the body goes through; a **light** stick press tilts it (owner's call: a firm press, so every flick, roll and keyboard direction, never tilts); tilt distance 0.5 and speed [E]) | The bubble shrinks with HP and can be tilted with the stick; a hit on an exposed part (feet, a hand) goes through | Every hit on a shielding fighter is blocked | High | L |
| D2 | Dodge staling (**done, sim 112**) | Rolls, spot dodges and air dodges used again within a short time get more lag and less intangibility, recovering over time | Every dodge is the same | Med-High | S-M (numbers to research) |
| D3 | Jab lock and jab reset (**done, sim 132**: a weak hit on a lying fighter keeps it floored, at most twice per knockdown; the bounce is 12 frames [E], `Ruleset::jab_lock_frames`) | A weak hit on a lying fighter forces a short "down damage" reaction, and a second one forces it to stand up (no lock loops) | A lying fighter takes an ordinary hit | Med | S |
| D4 | Shield drop through platforms | Removed | **Kept, as shield + special (sim 112, owner decision)**; down no longer drops through | Decided | S |
| D5 | Spot dodge cancel | Reportedly some actions can cancel a spot dodge's end; needs checking against 13.0.1 | None | Low (verify first) | S |

### Offence and movement

| # | Gap | Reference | Ours | Impact | Effort |
| --- | --- | --- | --- | --- | --- |
| O1 | Perfect pivot, pivot tilts and pivot smashes (**done, sim 133**, an addition by owner's call: the current reference removed pivots, the previous generation had them; a one-frame flick back out of a dash slides into a standing turnaround, `Ruleset::perfect_pivot_frames`, 0 turns it off) | A flick back then forward slides into a standing turnaround that cancels into tilts and smashes | Pivot grab only | Med (competitive spacing) | S-M |
| O2 | Footstool (**done, sim 135**: jump with the feet on a head bounces off it, no damage, no midair jump spent, lower each time before landing; ground victim frozen 21 frames (labbed reference), air victim pushed down tumbling and can tech; 4 intangible frames; not within 16 frames of an attack; bands, decay, push and air stun [E]) | Jump on an opponent's head: a jump for you, a short stun for them on the ground, a forced fall in the air | Missing | Med (edgeguards, combo escapes, combos) | M |
| O3 | B-reverse on the down special (**researched 15 Oct**: in the reference only *some* characters' down specials reverse, with no animation for it, and the window is 4 frames after the special input; so it is a per-move flag for the moveset ports to set from each fighter's data, not a universal rule) | B-reverse works on neutral and down specials | Neutral and side specials only (verify which specials the reference allows) | Low | S |

### Match rules and stages (1v1)

| # | Gap | Reference | Ours | Impact | Effort |
| --- | --- | --- | --- | --- | --- |
| R1 | Standard ruleset (**done**: 7 minutes by default, sudden death, sim 114) | 3 stocks, 7 minutes; a tie on stocks and damage goes to sudden death | 3 stocks, no time limit by default; no sudden death | Med | S (timer default), M (sudden death) |
| R2 | Moving platforms | The usual 1v1 stage list includes stages with moving and transforming platforms | Four stages, every platform fixed; the sim has no moving platforms | Med (stage variety) | M-L |

### Owner decisions where we differ on purpose (keep)

- Smash 4-style perfect shield, not the reference's release parry.
- Late dash turns are instant (sim 106); the reference's held turn is `dash_reverse_frames = 5`, kept for an A/B test.
- A dash flicked during a throw, landing lag or a falling jump is kept longer than the reference's buffer.
- Wall jumps need the stick held away from the wall plus jump (sim 101).

### Presentation (not art)

- No controller rumble (C3).
- No announcer voice: the "GAME!" banner is silent text. Low impact; an audio task.
- Results screen statistics (known gap in `STATUS.md`).

### Notes for the art director

These are things a player of the reference would notice. They are listed for the art director to schedule, not to be done here.

- Per-move animation is the biggest visible gap: many specials, grabs, throws and jabs share generic poses (`FEEL_AUDIT.md` A3). This is
  waiting for the full animation pass, which the owner has deferred to near the end.
- There is no teeter (the wobble when standing at a ledge's edge).
- Upward KOs could use a star KO or screen KO as well as the blast.
- Entrance and victory poses: check that every fighter has them before release.

## Recommended order

1. **Controller pass** (C1, C2, C3, then a hardware playtest). This is input layer only: the sim does not change, and it is the owner's
   daily experience.
2. **Dodge staling** (D2). Research the reference's numbers first. It is a sim change and makes defence less spammable.
3. **1v1 rules** (R1): a 7-minute default for versus, and sudden death.
4. **Jab reset** (D3) and **perfect pivot** (O1): small sim changes.
5. **Shield tilt and shield pokes** (D1): the largest sim job. The shield needs a shape the hit test can use.
6. **Footstool** (O2).
7. **Moving platforms** (R2), with one stage that uses them. This needs the art director for the stage.

Owner decisions (14 October 2026): tap jump off by default (when the controller pass adds it); keep a shield drop through platforms,
as shield + special, with down no longer dropping (done, sim 112); adopt dodge staling (done, sim 112).
