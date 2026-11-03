# 1v1 against the reference game: what is left

Executive review of standard one-on-one play (movement, combat, defence, controls, match rules, presentation), 14 October 2026, at
sim 111. Online play and other modes are out of scope. Sources: ssbwiki.com and ultimateframedata.com (reference version 13.0.1), the code on
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
- Techs (floor, wall, ceiling), missed techs and knockdown get-ups, trips, bury, grabs, pummels, throws, grab mashing; the rolls out
  of a tech, a knockdown or a trip on their own frames since sim 149 (D7). Not yet: the tech in place's and the stand-ups' lengths
  (D8).
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
| D4 | Shield drop through platforms | Removed | **Kept, as shield + special (sim 112, owner decision)**; since sim 144 (playtest 2026-10-19) a hard down flick drops through too, as the reference's (a double tap of down on a keyboard); a slow tilt or single press crouches | Decided | S |
| D5 | Spot dodge cancel | Reportedly some actions can cancel a spot dodge's end; needs checking against 13.0.1 | None | Low (verify first) | S |
| D6 | A forward roll ends turned round (**done, sim 147**: the turn lands on the frame the roll ends, a stale roll's later end included, so a grab, attack, jump or special buffered out of it comes out facing the new way; `Fighter::roll_turns`, tests `roll_facing.rs`) | A forward roll out of a shield ends facing the way it came from, so rolling past an opponent leaves you facing them; a back roll keeps its facing. Tech rolls, get-up rolls (a trip's too) and ledge rolls keep their facing whichever way they go | A forward roll kept facing the way it rolled, so rolling past someone left your back to them | High | S |
| D7 | Tech, get-up and trip rolls on their own frames (**done, sim 149**: the ruleset's `tech_roll_*`, `getup_roll_*` and `trip_roll_*`, each body's own get-up roll window (`FighterParams::getup_roll_intangible` / `getup_roll_back_intangible`), hashed `Fighter::roll_kind`; lying face up only, as the sim has one way of lying [E]; the distances are the shield roll's [E]; tests `floor_rolls.rs`) | A tech roll lasts 40 frames (acts on 41), intangible 1-20, either way, on every body we have ported; a get-up roll 35 (acts on 36), intangible from frame 1 to somewhere between 14 and 31 by body, way and lying face up or down; a trip's roll 28 (acts on 29), intangible 1-9 | All three played the body's shield roll (26 to 35 frames, intangible from frame 4 or 5), so a tech roll was open on its first three frames and ended 5 to 14 frames early | Med | S-M (new fields; the distances [E]) |
| D8 | The tech in place and the stand-ups on their own frames | A tech in place lasts 26 frames (acts on 27), intangible 1-20; standing up from lying 29 (acts on 30), intangible 1-22; standing up from a trip 21 (acts on 22), intangible 1-16: each the same on every body that has it (one body stands up a frame sooner) | A tech in place is 4 frames of landing lag with 18 intangible frames, so it can act 22 frames early; standing up lasts 26 frames, the first 15 intangible, from lying and from a trip alike | Med-High (tech chasing) | S (ruleset numbers; found while doing D7, not yet checked against footage) |

**How D6 was checked** (2 November 2026). ssbwiki.com `Roll` gives the rule for rolls out of a shield but says nothing about tech
or get-up rolls, and no other written source found does. Those were checked in a public dump of the reference's own game scripts:
the turn is an explicit command in the forward roll's script on every body that has one (91), never in a back roll's, and in none
of the 94 bodies' tech rolls or get-up rolls, nor in the trip rolls, ledge rolls or the phantom's gun dodges of the bodies we have
ported. An earlier reference generation's decompiled source agrees: its roll turns the fighter, its tech and get-up rolls do not.
Not yet checked against footage. One difference on purpose: the reference turns part-way through the roll (frame 19 or 20 on every
body we have ported, about two thirds of the way through), ours on the roll's last frame, as this item's brief asked. Apart from
where a held item is carried, nothing reads a rolling fighter's facing but the roll itself, and a hit sets it anyway, so play is the
same; the visible turn is the view's to place inside the roll. D7's numbers come from the same dump's animation data; the one
fighter's tech page checked on ssbwiki.com agrees (40 frames, intangible 1-20).

**How D7 was done** (2 November 2026, sim 149). A public dump of the reference's game data gives each motion its intangible frames
and the frame it can act on. On all 93 bodies that have them, a tech roll is intangible 1-20 and acts on 41, a get-up roll acts on
36 and a trip's roll is intangible 1-9 and acts on 29: those are ruleset numbers. A get-up roll's intangibility differs by body, by
way and by lying face up or face down: each body we have ported carries its own two face-up windows, forward and back (from 1-19 to
1-31; a body without its own takes the reference's most common, 1-21 forward and 1-22 back). The sim does not track how a fighter
lies, and only one get-up attack exists per body (the face-up one), so the face-down windows are noted in each body's parameters but
not used [E]. Frame 1 is the frame the roll begins on: the landing for a tech roll, the flick for the others, so each is intangible
on its first frame too. The rolls still move as the body's shield roll does [E: the reference publishes no distances], are never
staled and keep their facing (D6). A ruleset length of 0 plays the shield roll instead, the rule before sim 149. D8's numbers are
from the same dump.

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

- The previous reference generation's perfect shield (timed on the press), not the current reference's release parry.
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
