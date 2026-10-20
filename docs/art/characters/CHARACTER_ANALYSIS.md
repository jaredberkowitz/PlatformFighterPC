# Character model analysis (2026-10-19)

The model only, not the animations. Compared against the owner's character reference (`docs/art/reference_character_style.webp`: four
round-headed scouts in a vehicle), with the lessons from the stage pass (`docs/art/STYLE_SHEET.md` section 9) applied to characters.
Captures in this folder: `default_turnaround.png` and `default_lineup.png` (the default looks), `dressed_turnaround.png` (a full outfit),
`in_match_closeup.png` (the meadow at a close camera).

## Where we stand
The base is sound: round heads at the reference's share of the height (`docs/BODY_PROPORTIONS.md`), clean outlines, a drawn face, and a
modelled wardrobe (hats, glasses, neckwear, shirts, shorts, gloves, socks, shoes) that, worn, already reads close to the reference.

## Ranked gaps

| # | Gap | What we have | What the reference has | Stage lesson it maps to |
|---|---|---|---|---|
| 1 | **Default fighters are bare** | Every default look wears no shirt, shorts or gloves: in a match the cast are naked coloured blobs (`Loadout.default_for`: shirt 0, gear 0) | Every character fully dressed: shirt with sleeves and trim, sash with badges, shorts, tall socks, gloves | Detail where the eye is (the fighters are the focus of every frame) |
| 2 | **Glossy plastic surface** | A soft gradient across each form, a white specular streak across the head and torso, shiny shoes: a vinyl toy | Matte: two soft tones per form, a clear shadow shape, colour kept in the shade, no specular | Rules 1 and 5: form and light, not shine; shade a deeper version of each colour; one material language with the matte faceted world |
| 3 | **Head on a stalk** | The head narrows into a neck, so the silhouette is a lightbulb or peanut | The head sits straight on the shoulders, a wide squat oval, no visible neck | Silhouette first: big confident shapes |
| 4 | **Same face for everyone** | All defaults wear face 0: the same wide round eyes and a small smile | Every face its own: half-lidded, grumpy, sleepy, deadpan, drawn in thin confident lines; most of each character's personality | Each stage its own personality inside one style; here, each fighter |
| 5 | **Hands** | Grooved thumbs-up mittens, bare in the body's colour | Chunky four-fingered white cartoon gloves, the classic readable hand | Modelled over code-built; readable shapes at play size |
| 6 | **Even tube limbs** | Long thin legs and arms of one width, little form | Short sturdy limbs, wider at the top, socks and shoes giving them structure | Form over texture |
| 7 | **Clothing finish (when worn)** | Shirts flat on the body; sash badges small; shoes glossy | Sleeves standing off the arm with trim stripes, collars, bold badges | Detail in the right places |

## Recommended course of action (a pilot first, as the meadow was)
1. **Dress the defaults and give each its own face** (small, presentation only). Four default outfits in the reference's spirit (sailor,
   aviator scout, holiday shirt, ranger) and four different faces. Biggest change for the least work.
2. **Matte cel material pass** (medium): take the specular streak and gloss off skin, clothes and shoes; a soft but definite shadow
   shape; the shade a deeper version of each colour (as `pf_painted_shadow` does for the stages); check it under all five stages'
   lights and in greyscale against each stage's ground.
3. **Silhouette: head on the shoulders** (larger): shorten and widen the neck so the head sits on the shoulders; slightly shorter,
   sturdier limbs. The body is built by `art/blender/make_cage_body.py` and refitted for the rig, so every clip must be re-checked.
4. **Gloves**: a modelled four-fingered glove as the default hand (the mitten stays as an option).
5. **Clothing finish**: sleeves standing off the arm with trim, bigger sash badges, matte shoes.

## Tools to set up (as the stages got)
- A character reference table (`docs/art/characters/REFERENCE.md`): measured proportions and colours from the reference.
- One command for a character review round: turnaround (full colour, neutral, greyscale), the lineup, and an in-match close-up on two
  stages, with a before/after.
- A character brief for the judge (`docs/gauntlet/JUDGE.md`), scoring against the reference and the shared style rules.

## Done (2026-10-19, the body before the clothes, the owner's order)
- **Matte shading** (`shaders/toon_light.gdshaderinc`: `PF_FIGHTER_STEP` 0.74, `PF_FIGHTER_SHADE` 0.7, `PF_FIGHTER_EDGE` 0.06): a definite
  shadow side on every form; nearly all-lit, the fighters read as one flat shiny tone.
- **Head on the shoulders** (`art/blender/make_cage_body.py`): the upper torso fuller, the neck widened into the jaw, the head lowered.
- **Sturdier limbs**: fuller upper arms and thighs, tapering.
- **Hands**: four chunky fingers split clearly on the open hand and the fist (`make_rigged_blob.py` `fist`, `mitten`).
- Refitted with `fit_body.py --shoulder-lift 0.11` (the default 0.07 dropped the shoulder joint 0.1 and would move every arm), every
  clip rebuilt. Before/after: `body_before_after.png` (in a match), `turnaround_before_after.png`.

## Exact comparison (2026-10-19, after dressing the defaults): `reference_vs_ours.png`
Our aviator scout against the reference's, same outfit idea. The silhouette, colour and face now match; the gap is detail density and
material, not bone structure (the reference's bodies are as simple as ours: a soft ball head on a block body, no anatomy):

1. **Hair and hat volume.** The reference's head carries big soft shapes: a fleece aviator hat with ear flaps, long ginger hair spilling
   out under it. Ours is a thin skull cap, so the head reads bare. Biggest single difference.
2. **Material texture.** Every reference surface has a soft painted texture: fleece, cotton weave, knit socks, enamel badges. Ours are
   flat fills; the shirt and shorts read as plastic.
3. **Clothes stand off the body with real thickness.** Reference sleeves are wide cuffs with a coloured trim stripe, the collar a raised
   green band, socks thick ribbed tubes; ours lie on the body (the shirt's collar is a flat painted V).
4. **Badges.** The reference's sash carries six small square enamel badges with pictures; ours has three large plain coloured dots.
5. **Hands.** The reference's gloves are slim, long four-fingered cartoon gloves with clear separate fingers; ours are puffy mittens.
6. **Glasses.** The reference's are thin wire frames drawn wider than the eyes; ours are thick black plastic and sit on the eyes.
7. **Neck.** The reference's head sinks into the collar, no neck at all; ours still shows a short skin band between head and shirt.
8. **Lighting.** The reference has a warm bounce and soft occlusion where clothes meet the body (under the head, the sash edges); ours
   has none on the fighters.

## Cloth clean-up: sleeves that tore (2026-10-19, after round 19)
**Defect.** The shirt's sleeves tore into white sheets and spikes whenever an arm moved (attacks, a raised arm), and the sleeve hems
were ragged even at rest. Rounds 18 and 19 saw it as the sleeve "gathering at the shoulder in a bend".

**Cause** (`make_rigged_blob.py`, `build_garments`). A cloth point was told to be sleeve or body by `on_arm` on its *pushed-out*
position. A puffed sleeve stands about 0.26 from the arm's line, beyond `ARM_RADIUS` (0.15 since round 19, 0.19 before), so most of the
sleeve counted as torso: it took the trunk's bones and stayed behind when the arm moved, and its hem points missed the sleeve hem's
smoothing. The shirt's weight smoothing and `release_trunk` (both judging by the pushed-out cloth too) then handed the sleeve's underside
to the spine.

**Fix.** Each cloth point is classified by where it lay on the skin before it was smoothed and pushed out (`src`, `sleeve_pts`), and
takes the weights of that skin point; the sleeves keep those weights after the shirt's smoothing. No proportion, colour or clip changed.

**Checked.** Close-ups at rest, arms forward and arms overhead, and the forward-thrust attack sheet, before and after in the same
camera and pose (`cloth_sleeve_fix.png`: red strip before, green after); run and hurt sheets unchanged; fighter view, art pipeline,
loadout and animation library tests pass. Rebuilding the unchanged script reproduces the committed rig (only float noise in the face's
normals), so the before/after differences are this change alone. The torso's cloth under a raised arm still stretches in a soft web.

**Found, then fixed (the owner chose shorter shorts, below).** The shorts' cuff (0.54 to 0.60 high) and the socks' turned-down tops (0.49 to 0.55) both
lie over the knee joint (0.51). In the stance's slight knee bend the skin folds out through them: the band of bare leg under the
shorts and the white teeth along the cuff. Matching their weights to the skin did not help (tried and reverted); the fix is to move
one of them off the knee: shorter shorts (the cuff above about 0.58) or lower socks, leaving some bare knee as the references do.

## Cloth clean-up: shorts above the knee (2026-10-19, the owner's choice)
**Fix.** The shorts now end a little above the knee's bend (`shorts_end` = the knee joint + `KNEE_BLEND` + `SHORTS_KNEE_CLEAR` 0.02,
about 0.59 high; 0.54 before), so the cuff lies wholly on the thigh. The socks are unchanged; a strip of bare knee shows between them
and the cuff, as on the reference cast. Higher up the thighs are closer together and the two leg openings flared into each other
between the legs, so a leg opening no longer flares on its inner side (facing the other leg).

**Checked.** Leg close-ups at rest, arms forward and overhead, and the down tilt's deep kneel, before and after in the same camera
(`shorts_above_knee.png`: red strip before, green after): the white teeth and the band of leg pushing through the cuff are gone, the
crotch's arch stays clean. Run and jump-squat sheets bend cleanly; fighter view, art pipeline, loadout and animation library tests pass.

## Cloth clean-up: hands' budget, and what is left (2026-10-19)
**Hands.** The fist and open hand (`fist`, `mitten` in `make_rigged_blob.py`) are built at about two thirds of their old resolution:
each hand was 4,700 to 5,300 triangles, nearly the whole body's 5,000. Now 2,900 to 3,600; the rig went from 56,600 to 44,200
triangles. The hand sheet (`tests/hand_sheet.gd`) at full size shows no new facets.

**In a match** (`--demo=moves`, the meadow, round 19 against now: `cloth_cleanup_in_match.png`): the sword arm's sleeve wraps the arm
instead of fraying under it, and the shorts end above the knee. At this distance the change is small; it shows most in close-ups,
the character select and the results screen.

**Left as is.**
- With an arm swung fully forward, the puffed sleeve cap folds over itself on the inside of the bend (it stands about 0.13 off the
  arm, the owner's puffed sleeve, round 19). Spreading the arm's weights over the shirt round the armhole was tried and changed
  nothing; the fold comes from the puff itself. A smaller `SLEEVE_FLARE` would ease it, at the cost of the puff.
- The torso cloth under a raised arm still stretches in a soft web.

## Proportion refit to the owner's T-pose lineup (2026-10-19)
Targets measured in `REFERENCE.md` (the owner chose the three-character lineup). In `art/blender/make_cage_body.py` (`_reshape`,
`_width`, `_rise`): the torso narrows from the hips up (`WAIST_NARROW` 0.64, `CHEST_NARROW` 0.78, `SHOULDER_NARROW` 0.86), the arms
moving in with the shoulders as a whole (a hanging forearm given its own height's narrowing was pulled into the waist); the shins
(`SHIN_GROW`) and thighs (`THIGH_DROP` -0.02) longer and the torso a little taller (`TORSO_GROW`); the feet broader and set further out;
then the whole body scaled back to `BODY_HEIGHT` 2.24 so it still matches the content's hurtboxes. Refitted with `fit_body.py
--shoulder-lift 0.07` (0.11 was for the old arm running into the trunk; the narrower torso shows the armpit higher) and rigged.

**A bug found on the way.** `_reshape` shifted points outside the torso's core with `copysign(TORSO_CORE * (k - 1), x)`, which drops the
shift's own sign: every narrowing there widened instead (the hips stood about 0.02 wider than `HIP_NARROW` asked since round 7). Fixed;
with large narrowings it had swollen the waist into a bulge that hid the armpits from `fit_body.py`.

**Checked** (before red, after green): `refit_turnaround.png` (T-pose and turnaround), `refit_lineup_in_match.png` (the default cast and
an up tilt on the meadow). Every hat, pair of glasses, hood and neckwear (`lineup_shot.gd` gear sheets) still sits; run, jab, up smash,
down tilt and hit sheets and the ledge hang look as before; the menu and character select previews frame the new body. Fighter view,
art pipeline, loadout, animation library and match flow tests pass.

**Not this change:** `tests/look_bench.gd` fails (the palest look on the night stage's bright haze, and four looks against the city's
dark backdrop, edge 0.60 to 0.85 against the bench's 0.85) at the round 19 checkpoint as well, with the same numbers; left for a
lighting or outline pass.

## Matte material pass (2026-10-19)
Fighters only; the stages and props are unchanged (every change is keyed on the `fighter` instance flag or on the fighters' own
materials).
- **No highlight on fighters** (`toon_light.gdshaderinc`, `PF_FIGHTER_SHEEN` 0): the small soft highlight every material carried
  (`sheen`, 0.04 on skin and cloth, 0.16 on the "rubber" gloves, soles and buckle) is gone on a fighter, so shorts, boots and gloves
  stop reading as vinyl. Weapons keep their metal bands.
- **Painted texture**: the skin's grain (`SKIN_GRAIN` 0.15 to 0.3) shows as the reference's soft speckle up close and disappears at
  play distance; the cloth's woven strokes a little stronger (`CLOTH_WEAVE` 0.2 to 0.32).
- Left as they were: the two-tone shading (`PF_FIGHTER_STEP`, `EDGE`, `SHADE`, which rounds 8 and 19 settled), the edge light, and the
  fighters' reduced bloom (`post.gdshader` `bloom_fighter`).

**Checked**: close-ups in the same camera (`matte_pass.png`, left before, right after), in a match on the meadow, and the look bench:
the same numbers as before or a little better (the palest look on the night stage's bright haze from 0.84 to 0.86, no longer low); the
city's dark backdrop still fails as it did at round 19. Fighter view, art pipeline, loadout and animation library tests pass.

## Where we stand after the clean-up (2026-10-19): biggest differences left
`analysis_vs_reference.png`, top to bottom: the owner's T-pose lineup, our four default looks in T-pose, the owner's style reference
(the four scouts in the vehicle), our lineup close-up. Proportions, head shape, faces and colours now hold up; the gaps are in the
clothes and the lower body.

1. **Cloth edges read as torn paper.** Hems are thin single-sided bands with a deliberately wobbly edge (round 14), and the sleeves
   stand off the arm as flat flaps (`SLEEVE_FLARE`, no thickness: `CLOTH_THICKNESS` 0). The reference's hems are thick and clean, the
   sleeves solid tubes with a trim stripe. Visible in every capture, close and in a match. (Model script.)
2. **Too many bands across the body.** Top to bottom ours has about ten breaks (hat band, collar, sash with badges, belt and buckle,
   pocket, shorts cuff, sock top, sock, boot cuff, boot toe, sole); the T-pose lineup has three or four (shirt, strap, shorts, legs).
   The figure reads as stacked stripes and the small badges turn to noise at play size. (Loadout and art direction: the owner's call.)
3. **Heavy dark boots, hidden legs.** The reference's legs are bare, thick tubes in the body's colour ending in soft round feet (the
   style reference adds white socks, no boots). Ours are covered by socks and dark brown boots, the darkest mass on the fighter, which
   pulls the eye to the feet and, with the belt and the sash, makes three competing browns.
4. **Noisy outline.** Our brown ink hull wavers and doubles round the boots, cuffs and sleeve flaps; the reference has a thin steady
   line or none. Part of it is #1 (each flap and band draws its own hull).
5. **Hands.** Puffy mittens and fists against the reference's flat hands with clearly separate fingers. Smaller than the rest at play
   size.

Strengths to keep: the round head at the lineup's size against the body, the drawn faces, the matte colours, the proportions.

## Hem pass: clean thick hems, solid sleeves (2026-10-19, gap #1 above)
**Cause of the torn-paper edges.** The shorts' cuffs and the sleeves' trim were cut from the skin and lifted their own distance off it,
while the garment's flared hem stood at another, so the two crossed: most of a cuff sat inside the shorts and showed only as a ragged
fringe under the hem. The hems also carried a deliberate waver (round 14), and the sleeves stood off the arm as flat wings twice its
width (`SLEEVE_FLARE` 0.05 and the shirt's whole lift).

**Fix** (`art/blender/make_rigged_blob.py`):
- `band_on_hems`: the cuffs (`CUFF_BAND`) and the sleeves' trim (`TRIM_BAND`) are built on the finished garment's own hem, a solid band
  with a rounded section laid on a frame from the whole opening (its middle and axis, smoothed round the loop), each section taking the
  hem point's bone weights. The band's object copies the skin's mesh so it keeps the bone groups' names (a new empty mesh dropped them).
- `hem_rolls`: a round roll along both edges of the socks' turn-down and the boots' collar (`HEM_ROLL`), sections every `BAND_STEP`.
- Hems level again (no waver).
- Sleeves sit closer (`SLEEVE_SIT`, `SLEEVE_FLARE` 0.02): a tube about 1.4 times the arm's width, as the references' short sleeves.
- The rig goes from 43.5k to 48.6k triangles.

**Checked** (`hem_pass.png`, red before, green after: two looks at rest, arms forward and overhead, and the lineup): the cuffs read as
solid rolled bands, the sleeve ends as clean tubes, and the arms-forward sleeve no longer folds into flaps. Run and down tilt (deep kneel)
sheets, the brawler's jab, the gear sheets and the in-match moves demo look right. Fighter view, art pipeline, loadout, animation
library and match flow tests pass.

**Left:** the shirt's shoulder still makes a slight horizontal shelf in a straight front view; the socks' turn-down edges are softer
but still uneven where the sock follows the calf.

## Outfit pass: bare legs on the default looks (2026-10-19, gaps #2 and #3, the owner's choice of option B)
Four options were mocked first without changing the game (current; socks with canvas shoes; socks with low dark shoes; bare legs;
trimmed lighter boots), close up and at play size. The owner chose bare legs, after the T-pose lineup.

**Change**: the four default looks (`Loadout.default_for`) wear `gear` 2 (shorts, and gloves where `hands` asks: the aviator) instead of
the full kit, and no belt: a new loadout slot `belt` (`BELTS`: belt and pockets, or none; `has_belt()`) shows the belt, buckle and the
shorts' pocket flaps. Legs and feet are the body's colour. Bands down the body go from about ten to five (hat band, collar, sash, shorts
cuff, sleeve trim).

**Kept available**: the full kit and the belt stay in the loadout (the editor's look tab lists the new slot; the creator's simplified
categories do not show it, as they never showed `hands`). Looks saved before the belt slot keep their belt (`from_bytes`), and the
built-in roster keeps its belts (`Roster.builtins`): `roster_shot.gd` renders the same before and after.

**Checked**: `outfit_bare_legs.png` (red before, green after: the default cast close up, and the sailor and the aviator on the meadow).
Loadout (with new checks: the default look is bare-legged without a belt, an old look keeps its belt), fighter view, art pipeline,
animation library, match flow, editor UI and select grid tests pass. The creator flow test fails before and after this change with the
same errors (`unknown field ledge_exposed_frames in the ruleset`: the built bridge is behind the content), not from this pass.

## Hand pass: separate fingers on the open hand (2026-10-19, gap #5)
The open hand (`mitten` in `make_rigged_blob.py`, also the glove's shape) was a mitten: fat finger tubes overlapping one another and
mostly buried in the palm. Now the palm is shorter, the four fingers slimmer, spread so a gap shows between each and longer out of the
palm, and the thumb thicker, low on the heel and turned well out; the hand's length is about as before. The fist is unchanged.
`hand_pass.png` (left before, right after, the sailor's relaxed hands in the lineup). Fighter view, art pipeline, loadout and
animation library tests pass.

**Outline (gap #4), reviewed and left:** since the hem pass the doubled, wavering line round the cuffs and sleeve flaps is gone; what
remains is a steady line, which the references' style allows. No change made.

**Tool note:** `tests/hand_sheet.gd` frames the hands from above since the proportion refit (its camera was set for the old arms);
the lineup's relaxed pose was used instead.

## Wrist pass: arms flowing into the hands (2026-10-19, the owner: the hands and arms looked disconnected)
**Cause.** `fit_body.py` puts the wrist joint a fixed share of the height (0.075) above the fingertips. Since the proportion refit the
arm is shorter, so the joint sat about 6 cm up the forearm where it is still thick; the skin is cut at the joint, so the forearm ended
in a wide flat rim and the narrower hand sat on it like a ledge.

**Fix.** A `--wrist-lift` option for `fit_body.py` (default 0.075, as before); the base body is refitted with 0.045, which lands the
joint on the cage's modelled wrist where the forearm has tapered. The body mesh is unchanged. The refit command for the base body is now:
`fit_body.py -- --in art/models/base_body/base_body.glb --out art/models/base_body/joints.json --shoulder-lift 0.07 --wrist-lift 0.045`.
The hands are measured from the thinner wrist and come out about 7% smaller (`grow` 1.126 to 1.05).

**Checked**: `wrist_pass.png` (left before, right after, the sailor's relaxed hands): the forearm tapers into the hand with no step.
Forward smash (sword in the fist through the swing), the brawler's jab, the run and the in-match moves demo look right. Fighter view,
art pipeline, loadout, animation library and match flow tests pass.

## Sash pass: a clean band (2026-10-19)
The most visible clutter left was the chest: six badges packed into the middle of the sash every 13.5 degrees and greyed toward their
brass rims read as a cluster of rings; the puffed shirt broke through the band in white wedges; and the shirt's three buttons poked
through it as pale dots. (`fighter_view.gd`)
- Four badges (`SASH_BADGES`), spread from the shoulder to the hip (`SASH_BADGE_FROM` 44, `SASH_BADGE_STEP` 31 degrees), a fifth
  bigger (`SASH_BADGE_SIZE`), the enamel less muted (`SASH_BADGE_MUTE` 0.25, from 0.45).
- The band sits further out over a shirt (`SASH_OVER_SHIRT` 1.36, from 1.2): solid, with no gap at the sides.
- The shirt's buttons are hidden under a sash (neckwear 1 and 5); the placket stays.

**Checked**: `sash_pass.png` (left before, right after, the lineup), chest close-ups at rest and arms forward and overhead, the roster
sheet and the in-match moves demo. Fighter view, loadout, art pipeline, animation library and match flow tests pass.

## Aviator cap pass: fleece ear flaps (2026-10-19)
Side-on, each ear flap of the aviator's cap read as a flat brown disc pasted on the cheek, and the fleece edge down its front (five
separate balls) as a string of beads. In the owner's style reference the flaps are the same fleece as the rim. `make_gear.py`
`aviator_cap`: each flap is now one soft lumpy fleece pad (`fleece_flap`), its top tucked up under the fleece roll so it hangs from the
rim, wider at the top and tapering past the jaw; the leather tongues and the bead edges are gone (two fewer meshes per side).
`aviator_cap_pass.png` (top before, bottom after: front, three-quarter, side). Loadout, fighter view and art pipeline tests pass.

## Back-view pass (2026-10-19)
From behind, three things read as broken:
- **The aviator cap's goggle strap stood 0.12 off the dome as a brim** round the back (`make_gear.py`: a fixed radius 0.93). It now
  follows the shell (`shell_r`), just clear of its dents.
- **The cap's leather rim showed under the fleece roll** at the back as a ragged dark band: the roll sits lower at the back (`back_k`).
- **The shirt showed through the sash on the back** in white patches: the band now stands out as far in depth as in width over a shirt
  (`fighter_view.gd`, the sash's depth scale 0.8 of the widening, from 0.45).

`back_pass.png` (top before, bottom after: the four default looks from behind). Fighter view, loadout, art pipeline and animation
library tests pass.

## Weave pass: no ripples across the cloth (2026-10-19)
From behind and up close the shorts carried dark wavy vertical bands, read as ripples or creases in the cloth. The neutral grey render
was smooth, so they were the painted weave (`brush.gdshaderinc` through `CLOTH_WEAVE`), not the mesh: at strength 0.32 (the matte pass
had raised it) and scale 0.12 its strokes ran about 18 cm apart. A finer scale alone read as wood grain. Now a faint fine cotton weave
(`CLOTH_WEAVE` 0.12, `CLOTH_WEAVE_SCALE` 1.2). A smoothing pass on the shorts' mesh was tried first and changed nothing (reverted).

Also kept from the back-view pass: the sash's depth over a shirt (0.8 of the widening) checked from the side, where it does not stand
out behind the back as a pack.

`weave_pass.png` (top: the seat before and after; bottom: the lineup before and after). Fighter view, loadout, art pipeline, animation
library and match flow tests pass.

## Expressions and accessories against the references (2026-10-19, after the clean-up)
`faces_gear_vs_reference.png`, top to bottom:
- the owner's references: the salute, the frown, and the T-pose lineup's faces;
- our four default faces straight on;
- every hat, pair of glasses and neckwear on the four bodies (the lineup's two gear sheets).

The bodies and the head shapes now match closely. Most of the gap left is in two places: how the faces are drawn, and some of the
accessories.

### Expressions (`godot/art/face_parts/`, `face.gdshader`, `FaceLook`)
1. **No catch-light in the eyes.** The salute reference's eyes have a big white shine, which makes them read as alive at any size.
   Ours are flat dark pupils on a soft grey-beige white. They read as sleepy or dull at play distance, even on the non-sleepy faces.
2. **Thin, small mouths and lines.** The reference's mouths are bold black strokes, about a third of the face wide (the frown, the
   toothy grin). Ours are thin purple lines about a sixth of the face wide; the cat mouth almost vanishes in a match.
3. **The far eye is lost on a turned head.** The face is projected straight on. At three-quarters, as the menus mostly show it, the
   far eye squashes into the head's edge and is cut off (`faces_shot.gd`, every expression). The reference's faces stay readable
   at three-quarters.
4. **Faint freckles and cheeks.** The salute's freckles are bold dots in small round patches. The aviator's are a faint orange
   dusting that disappears.

### Accessories (`art/blender/make_gear.py`, `fighter_view.gd`)
5. **Round specs** (glasses 3): thick brown donut rims standing off the face, with arms that stick out past the head's sides. They
   read as goggles, or as plastic toy glasses. The reference draws glasses as bold frames lying flat on the face (the frown).
6. **Plumed helm**: a grey dish over the head, like a pan lid. Its cheek plates are flat grey slabs that cut through the glasses.
7. **Beanie**: from the front a flat yellow bowl; it does not read as a knit hat.
8. **Crown**: floats tilted above the head, with a gap under it.
9. **Hood and scarf**: dark, heavy slabs; the scarf is a flat black bar across the neck.
10. **Shirts without neckwear**: the placket's three buttons are big pale balls, the biggest detail on the chest. The outfit shirt's
    collar reads as a turtleneck band.

### Suggested order
- First the eye shine and bolder mouths (1, 2): every fighter, every screen, cheap (face parts and shader).
- Then the round specs and the plumed helm (5, 6): the two accessories that clip or read wrong.
- Then the three-quarter face (3), which needs a change to how the face is laid on the head.
- The rest are smaller.

## Face pass: eye catch-lights and bolder mouths (2026-10-19, items 1 and 2 above, the owner's go-ahead)
- **Catch-lights** (`face_common.gdshaderinc` `catchlight`, set by `FaceLook.CATCHLIGHT` 0.46 of the pupil's radius): one flat cream
  dot up and to the left in each pupil, under the lids so a lidded eye keeps it only where the eye is open. (Round 19 had taken the
  shine out as glossy; the owner wants the salute reference's lively eyes. Flat, not a domed doll's eye.)
- **Bolder mouths**: the mouth strokes 30 wide in the drawings (`tools/art/make_face_parts.py` `MOUTH_LINE`, from 21; every mouth and
  expression mouth regenerated) and the mouth a little wider on the face (`FaceLook.MOUTH_SIZE` 0.36, from 0.32).

Both go through `FaceLook.apply`, so the creator's face pictures (`face_ui.gdshader`) match the fighters.

`face_pass.png`: the four default faces straight on (top before, below after), then every expression (rows 1 and 3 before, 2 and 4
after). Checked in a match on the meadow. Fighter view, loadout, art pipeline, editor UI and select grid tests pass.

## Accessory pass: round specs and plumed helm (2026-10-19, items 5 and 6 above)
- **Round specs** (`make_gear.py` `glasses`, `metal_specs_rim`): fine wire, 0.022 wide and 0.018 deep (0.05 and 0.035 before),
  seated closer to the face (`fighter_view.gd`, lift 0.03 from 0.05). They read as drawn glasses on the face, no longer as plastic
  donuts, and the far rim side-on is a thin line rather than a bar past the head.
- **Plumed helm** (`plumed_helm`): a tall dome built on the aviator cap's shell (`HELM_C` 1.52, `HELM_H` 1.12, `HELM_R` 0.9), coming
  down over the head with the front cut up over the brows (`HELM_FRONT` 46), instead of a wide dish sitting on top (a pan lid). The
  flat cheek guards, which crossed the eyes and the glasses at three-quarters, are gone.

`specs_helm_pass.png`: front, three-quarter, side and back; pairs of rows, before above after: the specs, the helm with the specs,
the helm alone. Fighter view, loadout and art pipeline tests pass.

## Three-quarter face (2026-10-19, item 3 above)
The face is drawn on a shell whose texture map is a straight-on projection, so on a head turned three-quarters (the menus' usual view,
and a fighter in a match) the far eye squashed into the head's edge and was cut off.

**Fix.** The drawing slides round the head toward the camera by part of the camera's angle (`face.gdshader` `face_slide`, set each frame
in `fighter_view.gd` `_slide_face`: `FACE_SLIDE` 0.45 of the angle, at most `FACE_SLIDE_MAX` 0.5 rad). The slide is done on the ball (u
back to an angle round the head, turned, projected again), so the parts keep their shapes. The glasses turn round the head with it, so
the eyes stay behind their lenses. Straight on nothing changes; side-on the near eye stays readable.

`three_quarter_face.png`: front, three-quarter and side for the sailor and the aviator (pairs of rows, before above after), then the
expressions sheet at three-quarters before and after. Checked in a match. Fighter view, loadout, art pipeline, editor UI and select
grid tests pass.

## Freckles as their own creator option (2026-10-19, item 4 above, the owner's request)
Freckles were one of the cheek styles, so they replaced the blush and were drawn as faint dots in the blush colour. Now they are their
own part:
- **A new loadout slot** `freckles` (`FaceLook.FRECKLES`: none, dots, sprinkle, few), drawn in a dark brown (`FRECKLE_COLOR`) over
  whatever cheeks the look has (`face_common.gdshaderinc` `freckle_tex`, placed as the cheeks are). The drawings come from
  `tools/art/make_face_parts.py` (`FRECKLES`, written to `godot/art/face_parts/freckles/`).
- **A Freckles category in the creator**, after Cheeks, with its own icon. Its pictures are framed close on one cheek.
- **Defaults and old looks.** The aviator now wears a round blush with dot freckles, as in the reference. Looks saved before the slot
  get none (`from_bytes`, with a test). The old cheek style "freckles" stays, so saved looks keep it.

`freckles_pass.png` (the aviator before and after; the creator's new category). Loadout, fighter view, art pipeline, editor UI and
select grid tests pass. The creator flow test fails as on main (`ledge_exposed_frames`, the built bridge).

## Beanie, crown and shirt buttons (2026-10-20, items 7, 8 and 10 above)
- **Beanie** (`make_gear.py` `beanie`): pulled down over the head (`BEANIE_C` 1.52, `BEANIE_H` 0.98; it sat on top as a flat bowl), with
  deeper ribs (0.028 from 0.012), so it reads as a knit hat with its cuff at the brows.
- **Crown** (`crown`): lower and sized to the head at its band (`fit`, from `head_r`), so it sits on the head instead of floating with a
  gap round it.
- **Shirt buttons** (`make_rigged_blob.py` `shirt_front`): smaller and flatter (radius 0.019 from 0.028); the three pale balls were the
  biggest detail on a shirt without a sash.

`hats_buttons_pass.png`: the two gear sheets, before above after. Fighter view, loadout, art pipeline and animation library tests pass.
Still open from the list: the hood and the scarf (dark, heavy slabs).

## Hood and scarf (2026-10-20, item 9 above)
- **Scarf** (`fighter_view.gd`, neckwear 3): it did not show at all. Its collar ring sat at the plain collar's height, below the puffed
  shirt's neckline, so only a thin stripe poked through the cloth, and its tail hung inside the shirt. Now the collar is lifted to the
  neck over the shirt and thicker (`SCARF_LIFT` 0.1, `SCARF_THICK` 1.6, `_collar`'s new `thick`), and the tail hangs out in front of the
  shirt. It swings through the run (`scarf_run`, checked).
- **Hood** (hat 6): in the outfit's own colour a shade deeper (it was near-black charcoal, the heaviest mass on the fighter, as a dark
  slab round the face), and the cowl round the shoulders shorter and barely flared (a wide stiff cone read as a flat bar).

`hood_scarf_pass.png`: front, three-quarter, side and back; the hood before and after, then the scarf before and after. Fighter view,
loadout and art pipeline tests pass. That closes the accessory list from the comparison above.


## Weak points before beta (2026-10-20, after the UI merge)
A fresh look against the references (the lineup, roster, faces, close-ups, animation sheets and the moves demo, all re-captured). The
owner: the model "still reads as a bit sloppy or amateurish compared to the references". Ranked by how much each costs the finish:
1. **The eye whites cut off at three-quarters**, as the menus show the fighters: a bug from the three-quarter face pass (below, fixed).
2. **A fuzzy, unfinished surface**: skin grain, cloth weave, a wobbly brown ink line and pale fringes along the hems together read as
   felt or a soft render; the references are smooth and clean with crisp edges. (Which of these costs most is still to be measured.)
3. **The sash**: wide, the same maroon band with the same four badges on every default look, standing off the body like a backpack
   from the side with its badges floating; the reference's is a thin leather strap lying on the shirt.
4. **Shapeless shirts**: no collars, puffed blobs, sleeves that just end; the reference's crisp collars, kerchief and sailor collar
   give each its identity. The printed shirt has a flat white slab across the shoulders.
5. **Everyone the same**: one sash on every default look, one belt with a gold buckle across the built-in roster, half-lidded smug
   eyes on most faces.
6. **The built-in roster's boots and socks**: blocky brown boots with a white sock band, crude beside the bare legs.
7. **Cluttered accessories**: cap, goggles and glasses on one head; brows showing under the beanie's brim.
8. **Smaller**: idle open hands read as claws; the armpit fold; dash and pivot grabs reuse the standing grab; no crouch before a jump
   or squash on landing (the playtest notes).

The owner's order: 1, then 2, 4 and 8, then the expressions.

## Eye whites at three-quarters (2026-10-20, item 1 above)
- **Cause**: the face shell was the head's front cap, cut 72 degrees round from the front, its rim sunk under the skin (the drawing
  showed to about 65). The three-quarter pass slides the drawing toward the camera by up to 0.5 rad, which carried the near eye's outer
  edge to about 78 degrees: past the shell, so its white was cut off square, with a notch. The side view had it too.
- **Fix**: the shell wraps a little past the head's side (`make_rigged_blob.py` `FACE_WRAP`). Wider, it reached the head's upper sides
  and jaw, which bulge past the ball the slide was worked out on, so the slide's angle clamped there and a stretched sliver of the
  drawing showed on the side of the head. The slide is no longer done on a ball: each shell point keeps its own depth in its vertex
  colour (`FACE_DEPTH_CODE`), and `face.gdshader` turns the point about the head's upright axis and projects it straight on again
  (the same result as before on a round head, so the front view is unchanged).
- **From behind**, the slide pulled the face round into view at the head's edge; it now fades out past the head's side
  (`fighter_view.gd` `FACE_SLIDE_BACK`).

`eye_wrap_pass.png`, before left of each pair: three-quarter close-up and side view, the six expressions at three-quarters, the back
three-quarter views. Rig rebuilt (mesh only; `base_rig.json` unchanged). Fighter view, art pipeline, loadout, animation library, match
flow, select grid and editor UI tests pass; `creator_flow_test` fails as before in this checkout (its built bridge is older than the
content files).

## Surface finish (2026-10-20, item 2 above)
- **Measured by switching each piece off** in turn (a throwaway mock: current, no skin grain, no grain and no cloth weave, a plain ink
  line, no outline). The skin grain (`SKIN_GRAIN` 0.3) and the cloth weave (`CLOTH_WEAVE` 0.12) made most of the felt look: blotches on
  the skin, fine stripes on the shirts and shorts. Both are now 0, so surfaces are clean colour and light, as on the references.
- **The outline is the owner's choice in the game:** they could not choose between the bold outline and none, so it is a new Display
  setting, **Fighter outline: Bold** (the default, as before) **or Fine** (`visual_settings.gd` `outline`, `fighter_outline()`). Fine
  sets the fighters' `outline_weight` to 0 and the outline shader draws nothing; in a match the ink pass (`post.gdshader`) still draws
  its thin line round each fighter, while the menus show the figures with no line at all.
- **Readability:** the look bench is unchanged on Bold (it fails only on the midnight body against the city's dark backdrop, as it
  already did). On Fine it loses about a third of the silhouettes against the bright hazes, but it measures the menus' previews, which
  have no ink pass; in a match (meadow and city, `--demo=moves`) Fine fighters keep a crisp thin edge and read clearly.

`finish_pass.png`: close-up and group shot, then the meadow match: before, Bold, Fine; then the city match: Bold, Fine.
`finish_pass_setting.png`: the Display page with the new row. Controls, camera, pause, fighter view, art pipeline, loadout, animation
library, match flow, select grid and editor UI tests pass.

## Shirt collar and buttons (2026-10-20, item 4 above)
- **Mocked first** (scratch rig builds): bigger collar points, a more fitted shirt, and both. The owner could barely tell them apart
  under the sash, preferred the puffy shoulders, and with the sash off chose the current collar's size, but "it just reads as
  triangles on the shirt": two flat patches laid on the chest, starting 0.1 below the neckline, with nothing joining them round the neck.
- **The collar** (`make_rigged_blob.py` `shirt_front`, `COLLAR_SHAPE` and the other `COLLAR_` constants) is now one band round the neck,
  laid on the shirt by walking down its surface from the neckline at each angle round the neck (so it lies on the chest in front and
  on the shoulders at the sides alike): narrow at the back, widening into the same two points at the front, its fold standing off and
  rounding over along the top. Built as one sheet (the cloth is drawn from both sides): thickened, the tiny faces at its points threw
  a few corners metres off. A column the walk cannot place is blended from its neighbours, and the band is smoothed along its length.
- **The flower shirt's collar** is the same band in the shirt's own print (it was two squared cream lapels, a white slab).
- **Buttons** are drawn without ink rings: three ringed buttons stacked read as the figure 8.
- The shirt's shape is unchanged (the owner's choice).

`collar_pass.png`, before above after: front and back with no neckwear, then the default looks. The collar follows the torso through
the run, forward smash, jump, grab and hit animations without poking through. Rig rebuilt (mesh only; `base_rig.json` unchanged).
Fighter view, art pipeline, loadout, animation library, match flow, select grid and editor UI tests pass.

## Jump crouch and landing squash (2026-10-20, item 8 above, the playtest's "no crouch before a jump, no landing squash")
- **Measured at real speed.** The demos save a picture every frame, which slows the game and stretches each frame's time to 0.02 to
  0.15 seconds, so the squash springs ran ahead and the captures understated both. A new demo, `--demo=hop` (a short hop and its
  landing, a picture every frame), run with Godot's `--fixed-fps 60` shows what a player sees.
- **The crouch was there, but late.** The jump's crouch lasts three sim frames; the squash spring eased in from standing (0.02, then
  0.07, 0.12...) and only reached its depth as the feet left the ground, and its clip blended in over 0.1 seconds, longer than the
  crouch. Now the crouch starts partway down (`JUMP_SQUAT_START` 0.2) and its clip blends in over 0.03 seconds (`JUMP_SQUAT_BLEND`),
  so it reads from its first frame. (A kick to the spring instead overshot into a flat pancake, 0.43; reverted.)
- **The landing needed nothing.** At real speed it squashes to 0.32 over four frames with dust and eases back up with a small rebound.
  A held landing squash was tried and made no difference; reverted.

`jump_crouch_pass.png`: the take-off at real speed, before above after, then the landing (after). Fighter view, animation library,
match flow and replay tests pass.

## Dash and pivot grabs (2026-10-20, item 8 above, the playtest's "dash grabs and pivot grabs need their own animations")
- Both played the standing grab (`anim_library.gd`), so neither read, and the pivot grab showed no turn: the simulation turns the
  fighter round at once as it starts.
- **Dash grab**: its own clip, `body_dash_grab` (`make_rigged_blob.py`): a lunge out of the run, hips dropped, the body pitched forward
  over a long front step, the back leg stretched out behind, both arms thrown out low.
- **Pivot grab**: its own clip, `body_pivot_grab`, winding the torso against the turn and whipping it round into the reach, and the
  body spins round from the way it was running to its new facing by the grab's first active frame (`fighter_view.gd` `PIVOT_SPIN`).
- Both are body clips (no weapon swing), with the standing grab as their stand-in on a rig without them. Every class maps its grabs
  through the shared table, so all 14 fighters get them.

`grab_pass.png`: each grab's sheet, before above after (dash grab, then pivot grab). Rig rebuilt (two new clips). Fighter view,
animation library, art pipeline, match flow, replay and loadout tests pass.

## Idle hands (2026-10-20, item 8 above)
- Mocked three open hands from scratch rig builds (current, relaxed, a soft mitten); the owner chose the relaxed one.
- `make_rigged_blob.py` `mitten` (bare hands and gloves alike): the fingers lie close together instead of fanned apart, a little fuller
  and shorter, curled further in toward the palm; the thumb sits closer in, turned toward the palm. Spread, slim and curled at the
  tips, the open hand read as a claw at rest.
- In a match the change shows little: armed fighters grip their weapons and unarmed ones hold their guard in fists. It shows in the
  menus, idle poses and open-handed moves.

`idle_hand_pass.png`: close-ups (at rest, a raised hand, the arm forward), before above after, then a match frame before and after.
Rig rebuilt (mesh only). Fighter view, art pipeline, loadout, animation library and match flow tests pass.

## Armpit fold (2026-10-20, item 8 above)
- **Found by elimination.** With the arm swung forward, a black arc ran across the side under the arm. Hiding the shirt removed it;
  turning off the fighters' bold outline (the new Fine setting) removed it too, leaving only a soft fold in the cloth. So it was the
  shirt's own ink outline, drawn along the cloth folding under the arm.
- Two narrower fixes made no visible difference and were taken back out: stretching the armpit's ink-free patch down the side, and
  leaving the ink off the sleeve's body-facing side.
- **Fix**: the shirt's ink-free patch round each armpit is wider (`ARMPIT_NO_INK` 0.07 to 0.2, `make_rigged_blob.py`). The fold now
  reads as a soft shade; the match's ink pass still draws the shirt's silhouette there and a light line for the fold.

`armpit_pass.png`: the close-ups, before above after (the arm forward from the side is the fourth panel), then a match frame before
and after. Rig rebuilt (mesh only). Fighter view, art pipeline, loadout, animation library and match flow tests pass.

## Expressions against the references, and the face finish (2026-10-20)
Compared every expression and the four default faces with the salute, the frown and the T-pose lineup. The owner: ours look "a bit
more clumsy, maybe too large, and too drawn on rather than a part of the character".
- **Too large, crowded**: each eye was about 0.3 of the head's width and the two nearly touched (`EYE_SIZE` 0.45, centres 0.34 of
  the head apart); the salute's are about 0.22 with clear skin between, the features in the middle of a lot of face.
- **Drawn on**: a dark plum ink with a hand-drawn wobble, and flat greyish-cream whites with soft edges, laid on the skin like a
  sticker; the references draw in even near-black strokes on clean whites.
- **Small pupils** in big whites read as a vacant side-eye.
- **One mood**: all four defaults are half-lidded and unimpressed (deadpan, smug, grumpy, smug); the reference cast each have their
  own eyes. (Mocked as option D, not chosen; still open.)

Mocked six options (scratch script laying settings over the face material); the owner chose F:
- `face_look.gd`: the eyes about a fifth smaller and a little further apart (`EYE_SIZE` 0.351, `EYE_GAP` 0.237), the brows down with
  them and smaller (`BROW_Y` 0.288, `BROW_SIZE` 0.306), the mouth smaller (`MOUTH_SIZE` 0.306), cheeks following the eyes; bigger
  pupils (`PUPIL` 0.266) with a bigger shine (`CATCHLIGHT` 0.55); the "ink" brow colour near-black.
- `face_common.gdshaderinc`: near-black ink, no line wobble, clean whites, and each white set into the head with a soft warm shade
  inside its edge (from a blurred copy of its own shape).
- `tools/art/make_face_parts.py`: the mouths drawn in the same near-black ink (regenerated).
- The face on the head and the creator's pictures share the shader, so both follow.

`face_finish_pass.png`, before above after: the four default faces; every expression; a match frame; the creator's mood page. At
match distance the change is small; it shows most in the menus and close-ups. Fighter view, art pipeline, loadout, editor UI, select
grid and match flow tests pass; `creator_flow_test` fails as on `main` (the locally built bridge).
