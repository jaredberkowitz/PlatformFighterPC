| Ledges | Grab box, trumping, diminishing invincibility (resets on stable ground), hang cap, regrab cooldown, options: get up / roll / attack / jump / drop | `ledge_*`, `Stage::ledges` |
| Walk, dash, run, turn, crouch | Tilt walks (speed scales with tilt). A *fast* flick dashes (a slow roll to full tilt only walks); reversing mid-dash is a dash dance; a slow reverse turns around first; stick down crouches | `walk_speed`, `dash_*`, `turn_frames` |
| Platform drop | Tapping down on a pass-through platform drops through (shield drop is the other route) | `platform_ignore_frames` |
| Walls and ceilings | Solid blocks (`bottom..y`) stop fighters sideways and from below. The ECB is a diamond: widest at `ecb_side_height`, narrowing to the feet and head, so low corners slip past | `ecb_*`, `Platform::bottom` |
| Helpless fall | `Helpless` state (entered via `fighter::enter_helpless`, which up-specials will call): drift and ledge grabs only, extra landing lag | `helpless_landing_lag` |

# Phase 1: movement (implemented in `sim-core`)

All values are `FighterParams` / `Stage` data (`sim-core/src/content.rs`); frame counts are 60 Hz frames.

| System | Behaviour | Key data |
| --- | --- | --- |
| Input history | 8 frames of input per fighter live in `GameState` and the checksum; `pressed_within(button, n)` detects a press edge within the last n frames | `HISTORY_LEN` |
| Jump squat | Ground jump waits `jump_squat_frames`; jump still held at the end = full hop, otherwise short hop | `jump_squat_frames`, `full_hop_velocity`, `short_hop_velocity` |
| Air movement | Drift, gravity, fast fall (stick down), air jumps | `air_*`, `gravity`, `fast_fall_speed`, `air_jumps` |
| Air dodge | Directional, one per airtime, decays; a shield press during jump squat is buffered to the first airborne frame | `air_dodge_*` |
| Wavedash | Downward air dodge that touches (or starts within `ground_assist_dist` of) a surface becomes a ground slide. Slide speed = `dodge_dir.x * waveland_speed`, so distance scales smoothly with how horizontal the stick is. Too horizontal = plain air dodge. A waveland counts as landing | `ground_assist_dist`, `wavedash_min_down` (cone width), `waveland_*` |
| Shield drop | In shield on a pass-through platform, a down press (buffered) drops through; down wins over jump | `shield_drop_*`, `platform_ignore_frames` |
| Platforms | Solid and pass-through top surfaces; land from above, jump up through | `Stage::platforms` |
| Ledges | Grab box, trumping, diminishing invincibility (resets on stable ground), hang cap, regrab cooldown, options: get up / roll / jump / drop | `ledge_*`, `Stage::ledges` |

Contested ledge grabs on the same frame go to the **lowest player index**, which then trumps any occupant
(`step.rs`). Losers keep falling. Covered by `simultaneous_grabs_resolve_by_player_index`.

## Not done yet (deliberately)

- Ledge attack has movement and timing only; its hitboxes arrive in Phase 3.
- Sloped or moving surfaces, and wall jumps or wall cling (not planned for Phase 1).
- Values are first guesses for playtesting (plan section 12).

## Tests

`sim-core/tests/movement.rs` and `ground_and_collision.rs` script inputs for every item above, including wavedash cone/smoothness/failure, shield drop,
platform landing, and ledge trump/simultaneous/invuln/hang-cap/mirroring. `step.rs` has a random-play invariant test, and
the netplay rollback fuzzer re-verifies determinism of all of it.

## Reference numbers and world scale

Movement now follows the reference game's published attributes. **One world unit = 8 reference units**, so a ~17 unit
tall fighter is 2.2 tall (`FighterParams::su(thousandths)` does the conversion). Jump velocities are *derived* from the
reference jump heights and gravity (`FighterParams::hop_velocity`), and tests assert the measured apex matches.

| | Duelist (sword archetype) | Brawler (blaster archetype) |
| --- | --- | --- |
| Walk / run / dash | 1.575 / 1.964 / 2.255 | 1.208 / 1.54 / 2.09 |
| Air speed | 1.071 | 1.281 |
| Gravity | 0.075 | 0.13 |
| Fall / fast fall | 1.58 / 2.528 | 1.8 / 2.88 |
| Full hop / short hop / double jump | 33.66 / 16.26 / 33.66 | 32.02 / 15.38 / 30.71 |
| Jump squat | 3 frames | 3 frames |

Source: the public attribute tables for the two reference characters (ssbwiki.com). Air friction values were not
available for the duelist and are estimates. Air dodge, wavedash, shield drop and ledge numbers are unchanged.

The placeholder stage is now Final Destination-sized (main block 22 wide, side platforms 3.6 up) so run speeds feel right.

## Momentum rules

- Jump squat applies no friction, so a run or dash jump keeps its speed off the ground.
- In the air, speed above `air_speed` is never cut by the stick; only light `air_friction` drags it down. Air acceleration
  is `air_accel + air_accel_stick * stick_tilt`.
- Walking off a ledge keeps momentum the same way.
- Landing while holding the stick the way you are moving goes straight into a **run** (no walk phase), and speed above
  the target eases off at `run_decel` instead of snapping.

## Ledge grab rule (bug fix)

A fighter can grab a ledge only when it is on the **outside** of the edge and at least `ledge_min_drop` **below** it.
Previously a fighter level with the ledge, or slightly over the stage, could snap onto it (for example walking off the
edge). Walking off now means falling for a moment first.

## Dash cancel and ground speed tuning

- **Releasing the stick cancels a dash** the same frame. The speed carries into a slide on ground friction, so a quick tap
  is a short dash and a held stick is a long one. Jumping out of a dash (or a cancelled dash) keeps its speed as air
  momentum. A flick the other way during a dash is still a pivot (dash dance).
- **`GROUND_SPEED_PERCENT`** (in `sim-core/src/content.rs`, currently 90) scales every ground speed, acceleration and
  friction at once, so starts and stops keep their timing and only the distances shrink. Air speed and jump heights use
  the unscaled reference values. Lower the number to slow the ground game further, or set it to 100 for the raw reference.

## Dash dance and jump-ins

- A cancelled dash brakes on `dash_brake` (stronger than plain ground friction) so stops and spacing are crisp.
- Reversing with a flick starts a new dash in that direction immediately (velocity flips on the same frame). This works
  both flick-to-flick and with a brief neutral between dashes.
- Jumping out of a dash keeps the dash speed in the air; holding back brakes the drift so the landing spot is steerable.
- Keyboard: the most recently pressed direction wins when both are held (`input_reader.gd`), so dash dancing does not
  stall at neutral.
- Demo: `Godot --path godot -- --demo=dashdance --shots=<folder>`.

## Less slippery, and fast fall

- Ground friction and `dash_brake` are doubled. A full run now stops in about 9 frames over roughly half a body width, and a
  cancelled full-speed dash in about 4 frames (tests assert under 10 frames, and under 0.6 of a body width).
- **Fast fall is a hard down press, not holding down.** `Fighter::hard_down`: the stick reaches the full threshold coming
  from near neutral, while falling. Holding down, or rolling slowly down, does not fast fall. On a controller this is a
  hard flick down.
- **Keyboard:** the first down tap is a *soft* press (about 0.55): enough to crouch, drop through a platform or shield drop
  (`STICK_DOWN`), but not enough to fast fall. A **double tap** (second press within 14 frames) is a full-strength press,
  which fast falls. Holding down ramps to full after 4 frames, which is a slow roll and so does not fast fall either.
- Reference clip analysis (the `/watch` skill, 0:30 to 0:50): the brawler's player dash-dances in small bursts near one spot with
  a puff of dust at each stop, firing the blaster between dashes, and covers only a few body widths over about two seconds.

## Air drift model (reference: ssbwiki Air acceleration and Air friction)

Per frame while airborne with horizontal stick tilt `t` (signed, -1 to 1):

- **Acceleration** = `air_accel + air_accel_stick * |t|`, applied toward a target speed of `t * air_speed`. It is the same
  whether you speed up, slow down or reverse. Duelist: 0.01 + 0.07; brawler: 0.01 + 0.08 (reference units per frame squared).
- **Air friction** applies only with no horizontal input (it decelerates toward zero), and to momentum above the maximum air
  speed when the stick is held the same way (a run or dash jump). Holding the other way brakes that momentum with full
  air acceleration. Values: duelist 0.00375, brawler 0.01. (The brawler's number comes from the dedicated air-friction table;
  its character page lists 0.004, so treat it as less certain.)
- Easing the stick back from full tilt slows you with acceleration, not friction.

Tests pin each of these to exact per-frame values.

## Fast fall fix

A double tap was ignored if the two presses were more than 14 frames apart, which is faster than many people double-tap.
The keyboard window is now 26 frames (about 0.43 s), and the sim remembers a hard down press for 10 frames, so a press just
before the apex still fast falls once you start falling. The input history is 12 frames.

`godot/tests/input_e2e.gd` sends real key events through Godot's Input, the Rust sim and back, and asserts which taps fast fall:
`Godot --headless --path godot --script res://tests/input_e2e.gd`

## Fast fall, second fix

Fast fall used to only raise the fall-speed limit, so the fighter took about 12 frames to speed up and it was easy to miss.
It now **snaps to fast-fall speed on the frame it starts** (as in the reference game), and the character stretches with
speed lines above its head while it lasts. The overlay shows `fastfall` and the vertical speed.

Verified in the real game window by injecting genuine key presses: `Godot --path godot -- --demo=fastfall --shots=<folder>`
(full hop, then a double tap on S just after the apex: vertical speed goes from -0.005 to -0.316 and the flag turns on).
A controller now only overrides the keyboard once its stick passes 0.25, so stick drift cannot cancel keyboard input.

## Walking first, dashing on purpose (movement refinement, sim v13)

Why: on a keyboard every direction press is a full-strength stick push, so every press was a dash and every air
press a full-speed drift. The sim already supported analog walking; the keyboard never reached it.

**Keyboard shaping** (`godot/scripts/input_reader.gd`; the sim still only sees a stick):

- A first press **walks**: it starts at 0.34 tilt and ramps to full over 14 frames. The start sits above the sim's flick
  start (0.3) so the ramp can never be read as a dash. Holding keeps walking; walk speed stops growing at the dash
  threshold, so you never run by holding alone (as with a stick).
- A press within 18 frames of the previous direction press is full strength at once, which is a flick: it **dashes**.
  That covers double tap and tapping the other way while dash dancing. Press the other way later than that and it is a
  walking turn.
- In the air the same ramp gives graded drift: a brief tap is a small nudge, holding builds to full air speed.
- Ctrl (P1) / backslash (P2) hold a constant 0.45 tilt: a slow walk, and a half-speed drift.
- **Short hop key** (N / apostrophe): holds jump for 2 frames (the jump squat is 3), so short hops no longer depend on
  how briefly you can tap Space. Space is still a full hop when held.
- **Smash key** (I / semicolon): attack plus `buttons::STRONG`. Because a ramped press is a tilt, this is how a
  keyboard makes a smash attack toward the held direction (up, down or forward; neutral is a jab). A double tap
  followed by attack within 4 frames is still a smash too. `STRONG` is ignored in the air.

**Sim changes:**

- **A run is committed.** Flicking the other way during a run starts a skid-turn (`Turn`) instead of an instant dash
  reversal. Dash dancing still works inside the initial dash (12 frames), where it matters for spacing.
- **Full hops open fast.** The reference game speeds up the first frames of a full hop so it reaches its peak sooner
  (its "initial height" is about 0.55 of the hop). A full hop now rises 55% of its height in the first 4 frames at
  constant speed, then follows a normal arc (`hop_burst_*`, `full_hop_velocity`; `Fighter::hop_boost` counts the
  opening). Total height is unchanged (the apex tests still pass); the apex arrives about a fifth sooner. The 55% is
  from the wiki; the 4-frame length is an estimate. Short hops and double jumps have no opening.

**Not changed, on purpose:** air acceleration, air friction, air speed and ground-to-air carry already follow the reference
tables; with analog input they give the graded control. Tap jump (stick up to jump) is not added: up is the up-tilt,
up-smash and up-special input.

Tests: `sim-core/tests/feel.rs` (walk, committed run, hop shape, graded drift, strong button) and the real-key
`godot/tests/input_e2e.gd` (walk, dash, run, taps, short hop). Demo: `--demo=walk`.
Sources: ssbwiki Jump (jump squat, full hop "initial height" 0.55), Initial dash, Walk and the attribute tables.

## Ledge visibility fix

Stage blocks used to be drawn 8 units deep, centred on the fighters' plane. With the perspective camera, the front
face looked wider than the plane the fighters stand in, so a fighter hanging off the side was partly hidden whenever the
camera sat toward the middle of the stage. Blocks and platforms are now pushed back so their front face sits just in
front of that plane (`FRONT_Z` in `stage_view.gd`). Display only; no sim change. A sweep of about 8,000 approach
trajectories around the ledge (every side, height, speed, stick direction and jump timing) found no sim-side grab fault.

Known differences from the reference game, not changed yet: the reference lets a fighter grab slightly behind it with
40% less reach, does not grab while down is held, and lets up-special moves grab the ledge mid-move; here the grab box is
the same in front and behind, ignores the stick, and only applies while airborne and not rising.

## Ledge grabs from the stage edge (sim v14)

Reported: walking off the edge grabbed the ledge while the fighter still looked like it was standing on the stage. At the grab
the body was 0.57 beyond the edge and 0.2 below it, overlapping the corner, and it then snapped 1.6 units. Now:

- The body must be fully clear of the wall (`ecb_half_width` outside the edge) and at least 0.9 below the ledge
  (`ledge_min_drop`), so the snap into the hang is under 1.3 units. The box is 2.2 wide and 2.6 deep (`ledge_reach_x`,
  `ledge_reach_down`) so recoveries stay forgiving.
- **Holding down declines the grab** (also during an up special), so a fighter can walk off or fast fall past the ledge.

## Tuned toward the reference game (sim v26)

The reference numbers below come from SmashWiki's pages for Ultimate (Short hop, Fast fall, Initial dash, Dash-dancing, Air dodge, Wavedash), read in
October 2026. That wiki marks its air dodge numbers as possibly inaccurate, and none of this has been felt by a person yet, so treat every number as
a first pass for playtesting.

**Already matching, left alone:** jump squat is 3 frames; a short hop is about 48% of a full hop (the wiki's table gives 45 to 48% for most fighters; ours
is 48%); fast fall is 1.6 times the fall speed and snaps to that speed the moment it starts (a tap down while falling).

**Dash dance** (`dash_frames`, `dash_reverse_frames`, `dash_turn_delay`, `Fighter::dash_age` and `dash_wait`):
* The initial dash is 10 frames (was 12; the reference has 10 for the sword and blaster characters).
* A flick the other way **up to 15 frames after the dash began** is a new dash, even once the run has started (the reference's "interrupt frame 15
  for all characters"). After that a flick is a skid-turn, as before.
* A reversed dash **stands for 2 frames** before the first step (the reference's turnaround takes 3 frames to start accelerating, against 1 for a plain
  dash), so a dash dance has the reference game's short stutter instead of an instant flip. The fighter still faces the new way on the flick frame.

**Air dodge and wavedash** (`air_dodge_windup`, `air_dodge_sling`, `air_dodge_landing_lag`, `air_dodge_frames`, `waveland_lag`):
* A directional air dodge begins with a **5-frame slingshot**: a slow drift opposite the chosen direction (and upward for a downward dodge), then the full
  dodge. This is why a wavedash cannot touch down on the first frame after a jump and why it takes a few frames longer than before. A neutral dodge has
  none.
* A dodge that lands without sliding (too horizontal, or a neutral one) has **10 frames** of landing lag (was 3); a waveland has **14** (was 10; the wiki
  gives 11 to 19 depending on when the dodge lands).
* The dodge lasts **48 frames** (was 30; the wiki gives about 49 for a neutral dodge). Intangibility is unchanged (frames 4 to 28).
* **A wavedash cannot slide off the edge of a platform**: it stops at the edge.

**Not done:** the dodge getting weaker when repeated (stale dodges: a third less distance), distances in the reference's units (the slide's total
is about 3 world units, close to the wiki's "about 20" training-room squares only by eye), different values per character, and a pivot dash with its own 2
extra frames.

Tests: `sim-core/tests/feel.rs` (dash-dance window and turnaround), `ground_and_collision.rs` (every reversal), `movement.rs` (slingshot, no instant
waveland, edge stop, landing lag).

## Second pass toward the reference game (sim v30)

Sources, read October 2026: SmashWiki (Buffer, Jump, Short hop, Fast fall, Landing lag, Air dodge, Edge, Run, Dash, and two characters' edge
pages) and ultimateframedata.com (the reference duelist's and brawler's pages: airtimes, dodges, shield drop). Measured first: our short hop / full hop / short hop
fast fall / full hop fast fall airtimes were already within one frame of the published ones (duelist 42/56/29/39 against 41/55/28/38; brawler
31/44/22/32 against 30/43/21/31; the one frame is how the frames are counted), so the jump arcs were left alone.

**Input buffer** (`input_buffer`, `Fighter::buffer_used`): a button press made on the frame an action becomes possible or up to **9 frames**
before it still starts it (it was 2 or 3). Holding a button counts too, however early it was pressed (the **hold buffer**), for attack,
special, jump and grab, not shield. Each press starts **one** action: an action uses the press up, and only a new press of that button can
start another. So a held jump that made a full hop does not also double jump, a buffered attack gives one jab, and holding shield into a jump
out of shield does not air dodge (shielding uses the press). Stick inputs (dashes, rolls, smash flicks) keep their short windows.

**Short hops**
- An attack pressed with the jump or during the jump squat makes the jump a **short hop**, even with jump held, and the aerial comes out on
  the first airborne frame (the reference game's short-hop aerial; on a keyboard, Space and J together).
- An aerial made during a short hop deals **0.85x** damage (`Ruleset::short_hop_damage`, `Fighter::short_hop`; cleared by landing, a midair
  jump, a ledge or a hit).

**Midair jumps steer**: a double jump takes its sideways speed from the stick (full tilt = air speed), so it can reverse the drift at once,
and with the stick neutral it goes straight up.

**Landings**: a **light** landing is 2 frames (`landing_lag`); landing while fast falling or at the maximum fall speed is a **heavy** one,
4 frames (`heavy_landing_lag`; the reference game's heavy landings are 2 to 6 by character). Aerials that autocancel use the same rule.

**Shield release**: letting go of the shield takes **11 frames** (`shield_release_frames`, new state `ShieldRelease`) before anything else;
jumping, grabbing, rolling and dodging straight out of the shield skip it. The perfect shield is unchanged (a hit in the first frames of
the shield, Smash 4 style), on purpose.

**Rolls and spot dodge**: forward roll 29 frames, backward roll 34 (`roll_back_frames`), intangible 4-15; spot dodge 25, intangible 3-17.

**Air dodges** (per body type; duelist figures, brawler in brackets):
- A **neutral** dodge keeps the fighter's momentum (it falls and drifts as usual) and lasts 52 [44] frames, intangible 3-29 [2-26].
  Landing: 10 frames.
- A **directional** dodge keeps the slingshot, carries the fighter, then from frame 20 lets it fall without control until it ends: 69 [61]
  frames aimed down, 85 [73] sideways, 116 [93] up, in between by angle. Intangible 3-21 [2-20]. Ledges can be grabbed from frame 24.
- A directional dodge (a wavedash included) lands with **19 frames** of lag just after the slingshot, one less every 4 frames later, down to
  11. The reference game says only "11 to 19, more the earlier it lands"; the 4-frame step is our estimate.
- Being hit gives the air dodge back (as does grabbing a ledge).

**Ledges**
- **The grab takes 19 frames** (`ledge_grab_frames`): no option before frame 20 (a press during it is buffered).
- **Intangibility on the first grab** since landing or being hit: 19 + max(4, 60 x airtime/300 + 44 x (1 - percent/120)) frames, airtime in
  frames (capped at 300), percent capped at 120: 63 at 0% after a short fall, up to 123 after a long time in the air, 23 at 120%. A
  **regrab** without landing or being hit gets **none**.
- **Options**: get up 34 frames (intangible 1-33), roll 45 (1-26), ledge attack 55 (its move's intangibility), ledge jump (new state
  `LedgeJump`, can act on frame 15, intangible 1-12). On the second grab without landing their intangibility is **80%**, on the third
  **50%**, from the fourth **none**. Jump wins over attack when both are pressed.
- **Six grabs** between landings (or hits); a seventh does not catch the ledge (`ledge_grab_limit`).
- Letting go by pressing down or away **ends the ledge intangibility** at once; trumping takes it away too.
- A ledge **behind** the fighter (back to the stage) has a 40% shorter reach (`ledge_reach_back_x`).
- Hanging lets go after **6.5 seconds** (390 frames).

**Not changed**: jump arcs, fall speeds, dash-dance window and air drift (already matched); ground speeds stayed at `GROUND_SPEED_PERCENT` 90 (100 since sim v33,
below).
Not done: dodge staling, the softhop, initial-dash shielding rules, and per-character ledge and dodge numbers beyond the two archetypes.

Tests: `sim-core/tests/reference_movement.rs` (buffer, hold buffer, one press per action, short-hop aerial and its damage, midair jump steering,
light and heavy landings, rolls) and `movement.rs` (ledge intangibility by grab, option decay, grab limit, back reach, letting go, hang time,
air dodge lengths and landing lag, shield release).

## The committed initial dash and full ground speed (sim v33)

Sources, read October 2026: SmashWiki (Dash, Initial dash), smashpro.tips and player write-ups on the initial dash; the shield rule is the
user's description of the reference game's hidden mechanic.

**Ground speed is now 100%** of the reference (`GROUND_SPEED_PERCENT`, was 90 since the keyboard felt slippery; walking is analog now).

**The initial dash is committed** (`dash_frames`, `dash_shield_frame`, `FighterParams::initial_dash_distance`):
* A tap covers the **whole initial dash**: letting go of the stick no longer stops it. It runs its `dash_frames` (10) at dash speed, then becomes
  a run if the stick is held that way, or brakes (`dash_brake`) to a stand. A second tap the same way changes nothing.
* It can be cut short only by: a **dash the other way** (the dash dance, unchanged), a **jump**, a **grab** (the dash grab), a **special**, the
  **dash attack**, or a **smash attack** (a flick up or down, or the strong button). **Tilts cannot** come out of it (any other attack is the dash
  attack), and it cannot crouch or drop through a platform.
* **The shield cuts it to about half**: from frame `dash_shield_frame` (5) the shield comes up, so holding forward and pressing shield stops the
  dash about halfway. A shield held earlier comes out on that frame.
* After the initial dash the run behaves as before (a dash attack, dash grab, jump, shield or skid-turn out of it).

**Not the reference's**: the reference game puts each fighter's initial dash at its own length; ours share 10 frames, with the distance set by
each fighter's dash speeds. Pivots (a flick back released at once, for pivot tilts) are not in.

**The animation keeps step with it** (`godot/scripts/fighter_view.gd`): the initial dash is one bounding step, from the push-off to the other
foot landing exactly as the dash ends, and the run goes on in step from there at two dash lengths a cycle, so the legs turn over with the
speed. The fighter's dash length comes from the simulation (`fighter_dash_length` on the bridge), and a test checks the tapped dash covers
exactly that distance.

Tests: `sim-core/tests/feel.rs` (a tap covers the whole dash and matches `initial_dash_distance`, a second tap changes nothing, dash dance out of
a tapped dash, no crouch or tilt, smash attacks out of it, the shield at half) and `ground_and_collision.rs` (letting go no longer cancels it;
any tap shorter than the dash goes as far).
