# Pose review (2026-10-15)

A still-by-still look at every class's poses: not motion, just whether each held shape reads as a person fighting. Playtesters judge
the model by these shapes, so a bad pose reads as a bad body. Input to the animation overhaul (Saturday 2026-10-17) and the body
remodel (branch `body-cage`).

**How it was captured:** `--demo=poses --gallery-class=N` (scripts/demo.gd): every normal and special of one class, each photographed
by the game itself at three moments taken from the move's real frame data: **wind-up** (4 frames before the first hit), **strike**
(first active frame) and **follow-through** (4 frames after the last hit). This is the in-game view, so the code layers (arm hinge, body
follow, weapon aim) are included.

**Sheets** (`docs/screenshots/poses/`): top-left cell is the idle stance, then three cells per move in this order: jab, ftilt, utilt,
dtilt, fsmash, usmash, dsmash, nair, fair, bair, uair, dair, neutral/side/up/down special. A black gap means that move has no hit
frame to time a photo on (counters, projectile-only specials). `state_longsword_*.jpg`: run, jump, shield, hurt and launch over time.

| Class | Sheet | Notes |
|---|---|---|
| 0 Longsword | c0_longsword.jpg | Best set. Up smash/up special arch back (see 3); jab/nair/specials end on a locked straight arm (1). |
| 1 Claws | c1_claws.jpg | Forward tilt dives head below hips; up smash is an upside-down handstand; fair flings both arms wide. |
| 2 Needles | c2_needles.jpg | Shares the unarmed set (4). |
| 3 Embers | c3_embers.jpg | Unarmed set; idle reaches one straight arm forward (1). |
| 4 Gauntlets | c4_gauntlets.jpg | Unarmed set; same flips and spread-arm fair as Claws. |
| 5 Staff | c5_staff.jpg | Staff held one-handed at the end of a straight arm in most strikes; reads as poking, not swinging a staff. |
| 6 Greatsword | c6_greatsword.jpg | Good overhead idle; body stays upright while the huge blade moves; it should lean and load into it. |
| 7 Bracers | c7_bracers.jpg | Unarmed set. |
| 8 Clutch | c8_clutch.jpg | Unarmed set. |
| 9 Buckler | c9_buckler.jpg | Shield and blade read well; up tilt/up smash arch back (3). |
| 10 Pockets | c10_pockets.jpg | Unarmed set, several specials untimed. |
| 11 Popgun | c11_popgun.jpg | Unarmed set. |
| 12 Dagger | c12_dagger.jpg | **No dagger shown**; pure unarmed placeholder set (queued art). |
| 13 Nodachi | c13_nodachi.jpg | **No nodachi shown**; unarmed placeholder set, most specials untimed (queued art). |

## What works
- The run: forward lean, pumping free arm, blade trailing.
- Low lunges (sword forward smash follow-through, down tilts): planted feet, body behind the strike.
- Horizontal back airs and flip kicks read as intended in motion.
- The greatsword's overhead idle and the buckler's shield-forward stance give those classes clear silhouettes.

## Problems, most visible first
1. **Locked straight arms ("reaching for a hug").** Strikes and follow-throughs end with the striking arm dead straight at shoulder
   height and the elbow locked: sword jab, nair and specials; staff strikes; the unarmed idles (Embers, Gauntlets) hold an arm straight
   out. Real strikes keep a slight elbow bend and pull the arm back once the hit is past. *Fix:* elbow floor ~15-20 degrees in every
   key; follow-through retracts toward the chest.
2. **Free arm flung wide (T-pose).** Unarmed fair/fsmash strikes spread both arms out to the sides, which reads as falling, not
   punching. *Fix:* the free arm guards the chin or pulls back to the hip (the chamber of a punch), never straight out sideways.
3. **Arching back on up moves ("inhaling").** Up smash, up special and up tilt bend the spine backward with the belly pushed forward
   and the head thrown back. With a round body this is the single strongest "wrong body" read. *Fix:* reach up from the shoulders and
   a lifted chest; hips stay over the feet; no backward bend below the chest.
4. **One pose set for nine classes.** Claws, Needles, Embers, Gauntlets, Bracers, Clutch, Pockets, Popgun, Dagger and Nodachi share
   nearly identical normals (the shared archetype poses; Needles, Bracers, Clutch, Pockets and Popgun were compared at thumbnail scale
   only). Dagger and Nodachi show no weapon at all. *Fix:* the overhaul's per-family
   sets (brawler, quick, heavy, tool) plus the queued weapon art.
5. **Head below the hips at the strike.** Claws/Gauntlets forward tilt and up smash, several up airs: the key pose is upside down. In
   motion a flip can work; as a held strike it reads as tripping. *Fix:* the strike frame is upright or sideways with the head leading;
   flips happen before or after it.
6. **Hurt pose.** Over the whole hitstun the sword is held up across the face with the free arm straight forward (`state_longsword_hurt`).
   It reads as a zombie or a dab, not a recoil, and never changes. *Fix:* head and chest snap back away from the hit, arms trail
   loosely, knees give, then a short recovery.
7. **Launch pose.** The body stretches into a long tube with the legs merged, and the sword points forward like a thrust
   (`state_longsword_launch`). It reads as melting. *Fix:* far less stretch; body curled or spread, limbs and weapon trailing behind the
   direction of flight.
8. **Jump.** One pose for the whole jump: one leg trailing straight, arms static. *Fix:* a tuck at the apex, legs opening on the way
   down, as the reference fighters do.
9. **Shield.** The weapon arm crosses the chest from the far side: the "behind and in front at once" read. *Fix:* the near arm braces
   in front, the weapon held at the side.
10. **Hands and legs.** Weapon hands shrink into a thin rod round the grip, and wrists bend sideways in follow-throughs. Several idles
    cross the knees, which reads as tiptoeing. *Fix:* fist size kept on the grip, straight wrists, feet shoulder-width apart in stances.

## Order
The overhaul plan already covers this; the review adds priorities. Fix items 3, 1 and 2 first: they appear in the most moves across
every class and are what playtesters read as a bad body. Then 6 and 7 (seen in every match), then 4 (per-family sets), then 5 and 8-10.
Check each fix with this same atlas (`--demo=poses`) before and after, and pose against the remodelled body once it lands.
