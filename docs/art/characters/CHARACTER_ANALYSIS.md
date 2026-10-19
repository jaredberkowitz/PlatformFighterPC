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
