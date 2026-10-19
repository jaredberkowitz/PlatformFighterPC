# Character round char19: verdict against the new T-pose references

Shapes are judged from the T-poses (`tpose_0`, the sailor, measured) and the hand sheet; character from `beauty.png`. Head shape is not
judged, since the round-head work is in progress. All measurements are pixel ratios taken off the images, accurate to about 10%.
They use **head height without the hat (HH)** and **head width (HW)** as the units, so they can be measured off any T-pose.

**Scores:** against the references 3 out of 5; ship-ready 3 out of 5.

The new references show one main thing: our upper body is built too big around a head of the right size. The hands, sleeves and arms
are each about twice the reference's size, and that is the "odd shoulders, hands as big as the body" the owner sees.

## 1. What the new references show, measured against ours

The reference figures come from `ref_tpose_trio.jpeg` (the yellow chef, front), checked against `ref_tpose_small.jpg`.

| Measure | Reference | Ours (char19) | Verdict |
|---|---|---|---|
| Head height ÷ figure height (crown to sole, no hat) | 0.34 | 0.35 | matches |
| Head width ÷ torso width (shirt body, not counting the sleeves) | about 1.4 | about 1.15 | our torso is wide for the head |
| Width across both sleeve hems ÷ HW | about 1.2 | about 1.75 | **the shelf**: ours is far too wide |
| Sleeve length ÷ shoulder-to-fingertip | about 0.20 to 0.25 (a short puffed cap) | about 0.45, ending in a flat rolled cuff | too long |
| Sleeve hem ÷ arm thickness | about 1.6 (flared) | about 1.2 (a stiff box) | not puffed |
| Shoulder line | drops from the collar at about 20 degrees into a soft cap; the arm leaves from the middle of the hem | flat and horizontal, out to the cuff | flat shelf |
| Arm length, shoulder to fingertip ÷ HH | about 0.85 to 0.9 | about 1.2 | about 25% too long |
| Arm thickness ÷ HH | about 0.20 | about 0.32 | too thick |
| Arm shape | a smooth tube with about a 10% taper, no elbow or wrist | a smooth tube | keep the shape, slim it |
| Hand length ÷ HH | about 0.26 to 0.30 | about 0.60 | **about twice too big** |
| Hand width ÷ arm thickness | about 1.0 to 1.1 (the hand is barely wider than the wrist) | about 1.5 | too broad |
| Fingers | four short, slightly splayed stubs, about 40% of the hand's length; a small thumb at about 30 degrees; a flat little mitten-hand | long fat sausages; the open pose still shows two fingers out (pointing) | too long, wrong pose |
| Bare leg below the shorts ÷ HH | about 0.8 | about 0.7 | slightly short |
| Leg thickness ÷ HH | about 0.33, tapering slightly into the foot, no ankle | about 0.39 | slightly thick |
| Socks (`ref_tpose_small`) | short ribbed socks, the top at about 40% up the shin | tall, to just under the shorts | this disagrees with the old group shot (see below) |
| Shoes | rounded, a little smaller than ours, about 0.4 HW long | about 0.43 HW long | close |
| Shorts | wide, flared at the hem, ending at about a third of the way down the leg | narrower, straighter | a small difference |
| Shirt (`ref_frown`) | a short boxy sleeve with a trim band; a soft shoulder slope from the collar; pointed collar points | a long sleeve ending in a flat cuff | sleeves to change |

**Where the references disagree, so the owner should choose:**
- **Sleeves:** the T-pose references have short puffed sleeves. `ref_salute` shows one outfit with long sleeves rolled up to the elbow. My advice is short sleeves as the default, and the rolled sleeve as one outfit's piece.
- **Socks:** the new T-pose reference has short ankle socks. The old group shot (`reference_cast.png`) has tall socks. These can be set per outfit.

## 2. Model changes, ranked, with targets you can measure off the T-pose

1. **Hands: about half the size.**
   - Hand length **0.30 HH** (now 0.60), so the hand scale goes from 1.1 to about **0.6**.
   - Hand width **1.0 to 1.1x the arm's thickness**.
   - Fingers **40% of the hand's length**, all four out with an even slight splay (about 10 degrees between fingers) and a gentle curl.
   - The thumb small, set low, at about **30 degrees**.
   - Keep the fist's construction, at the new scale.
2. **Sleeves: short puffed caps.**
   - Sleeve length **0.22 of shoulder-to-fingertip**.
   - The hem flared to **1.6x the arm's thickness**, with a trim band. No rolled shelf.
   - Width across both sleeve hems at most **1.25 HW** (now 1.75).
3. **Shoulders: slope them.**
   - The shirt's top line falls from the collar at **20 degrees** into the sleeve cap.
   - The arm comes out of the middle of the sleeve hem, at the top of the sleeve.
   - Torso width at the armpits **0.72 HW** (now about 0.87), so the head reads **1.4x the torso's width**.
4. **Arms: slimmer and shorter.**
   - Thickness **0.20 to 0.22 HH** (now 0.32).
   - Length from shoulder to fingertip **0.85 to 0.9 HH** (now 1.2).
   - Keep the smooth tube with no elbow break, tapering about 10% toward the wrist.
5. **Legs: a touch longer and slimmer.**
   - The bare leg below the shorts **0.8 HH**; thickness **0.33 HH**, tapering slightly into the foot with no ankle pinch.
   - The shorts' hem flared to about **1.2x the width of their top**.
6. **Socks and shoes, per outfit.**
   - Where the outfit follows the new references, short ribbed socks with the top at **40% up the shin**.
   - Shoe length **0.40 HW** (now 0.43), rounded toe, keep the dark sole.

## 3. Character, from the beauty shot
- **Hands:**
  - The oversized hands make every pose read as a boxer: the sailor's hanging hands reach its knees, the blue's guard is two fists the size of its head, and the ranger's spread arms are mostly hand.
  - At the target sizes the faces and hats become the focus again, as they are in every reference image.
- **Sleeves and shoulders:**
  - The long sleeves add a flat shelf to every silhouette.
  - On the ranger and the blue, the wide sleeves and arms make the upper body read as a wardrobe.
  - The short puffed caps of the references would give back the round-headed, small-bodied charm.
- **Keep:** the faces, hats, sash and badges, the shorts colours, the smooth arm tubes, and the fist construction.
