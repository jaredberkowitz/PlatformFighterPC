extends RefCounted
## The face, as parts a player chooses and places (like a caricature kit): eyes, brows, nose, mouth and cheeks, each from a catalog of
## drawings (godot/art/face_parts, tools/art/make_face_parts.py), with a colour where it has one, and moved, spaced, sized, turned and
## squashed in steps. This file holds the catalogs and turns a look (loadout.gd) and an expression into the face shader's settings
## (shaders/face_common.gdshaderinc), for the face on the head and for the creator's pictures. Cosmetic only: nothing here reaches
## the simulation.

const SvgArt := preload("res://scripts/svg_art.gd")
const DIR := "res://art/face_parts/"
## How sharply the part drawings are made (their 256 squares, this many times over).
const DETAIL := 1.5

const EYES := ["round", "tall", "wide", "almond", "droopy", "cat", "dot", "lash"]
const BROWS := ["none", "arch", "thin", "bar", "slant", "round", "bushy", "curl"]
const NOSES := ["none", "dot", "curve", "button", "hook", "nostrils", "wide"]
const MOUTHS := ["smile", "grin", "flat", "oh", "cat", "smirk", "toothy", "frown", "blep"]
const CHEEKS := ["none", "round", "oval", "freckles", "lines", "swirl", "heart"]
## Freckles, their own part over any cheeks (the face pass, 2026-10-19), drawn in FRECKLE_COLOR.
const FRECKLES := ["none", "dots", "sprinkle", "few"]
const FRECKLE_COLOR := Color(0.25, 0.15, 0.12, 0.9)

## (The first a deep navy since the reference pass, 2026-10-20, as the salute reference's pupils; it was plum.)
const EYE_COLORS := [Color("0b0b38"), Color("5b3418"), Color("2f5fb3"), Color("2f8a4a"), Color("a0661c"), Color("4a4f5c"), Color("8a2a62"),
	Color("111014")]
const EYE_COLOR_NAMES := ["navy", "brown", "blue", "green", "amber", "grey", "berry", "black"]
## (The first, "ink", the faces' own near-black since the face finish pass, 2026-10-20: a dark plum read soft and printed on.)
const BROW_COLORS := [Color("0b0b38"), Color("3b2416"), Color("7a4a24"), Color("d9a441"), Color("c0562c"), Color("9a9aa4"), Color("f2efe6"),
	Color("2f5fd0"), Color("e070a8"), Color("3fa860")]
const BROW_COLOR_NAMES := ["ink", "dark brown", "chestnut", "blond", "ginger", "grey", "white", "blue", "pink", "green"]
const CHEEK_COLORS := [Color("f59393"), Color("ff8a5c"), Color("f7b38a"), Color("e0607a"), Color("b48ae6"), Color("e04848")]
const CHEEK_COLOR_NAMES := ["pink", "coral", "peach", "rose", "lavender", "red"]

## The adjustments: how many steps each has (the middle one is the part's own place). What a step does is in `apply`.
const ADJUST := {
	"eye_y": 15, "eye_gap": 13, "eye_size": 13, "eye_squash": 11, "eye_turn": 11,
	"brow_y": 15, "brow_gap": 13, "brow_size": 13, "brow_turn": 11,
	"nose_y": 13, "nose_size": 13,
	"mouth_y": 15, "mouth_size": 13, "mouth_squash": 11, "face_lean": 11,
}

## The concept art's face: where each part sits with every adjustment in the middle (in the face square, 0..1). Measured off the concept
## sheet's front view in head radii (eyes 0.41 out and 0.185 up, 0.25 across; the mouth 0.16 down, 0.36 wide; cheeks 0.51 out and 0.2
## down) and turned into the square: the head shows the square 1.75 radii across, the drawing 1.15 times larger (face.gdshader).
## (Bigger and a little wider apart since the body rebuild, 2026-10-18: on the new wide head the concept's eyes read small at play size;
## the reference cast's eyes span about half the face.)
## (Smaller and further apart since the face finish pass, 2026-10-20, the owner's pick of mocked options: each eye was about 0.3 of the
## head's width and the two nearly touched, crowding the whole front of the head; the salute reference's are about 0.22 with clear skin
## between, the features in the middle of a lot of face.)
## (Closer since the reference pass, 2026-10-20, the owner: too spread apart. The salute reference's eye centres are 0.39 of the head's
## width apart; at 0.237 ours were 0.48.)
const EYE_GAP := 0.195
## (Bigger and lower, mouths wider, the character judge's round 16: the reference's features fill the middle of the face.)
const EYE_Y := 0.44
const EYE_SIZE := 0.351   # (0.45 until the face finish pass)
const BROW_GAP := 0.17
const BROW_Y := 0.288   # (0.258 until the face finish pass: down with the smaller eyes)
const BROW_SIZE := 0.306   # (0.36 until the face finish pass)
const BROW_THICK := 1.45   # (2.1 until the owner, 2026-10-19: far too thick)
const NOSE_Y := 0.5
const NOSE_SIZE := 0.14
const MOUTH_Y := 0.615
## (Larger, its line bolder with it, the character pass, 2026-10-19: the reference's mouths carry the expression.)
const MOUTH_SIZE := 0.306   # (0.36 until the face finish pass, 2026-10-20) # (0.32 until the face pass, 2026-10-19: the reference's mouths are about a third of the face wide)
## (Smaller, the character pass, 2026-10-19: the reference cast's pupils are small dots in a wide white.)
const PUPIL := 0.245   # (0.19 until the face finish pass, 2026-10-20: small dots in big whites read as a vacant side-eye; 0.266 until the
                       # reference pass: the salute's pupil is about 70% of its eye's width, sitting a little low)
## The catch-light's radius as a share of the pupil's (face_common.gdshaderinc `catchlight`), the face pass, 2026-10-19.
const CATCHLIGHT := 0.85   # (a big soft blob since the reference pass, as the salute's)
const LOOK := Vector2(0.02, 0.035)   # (0.051, 0.01 until the reference pass: the salute's pupils sit low and nearly central)
## Cheeks sit out from and below the eyes and grow with them (the concept's: just outside the eyes and well below them, 0.2 across).
const CHEEK_OUT := 0.053
const CHEEK_DOWN := 0.174
const CHEEK_SIZE := 0.156   # (0.2 until the face finish pass, with the eyes)

## Expressions: the player's face with its lids, brows, pupils and mouth changed. `lids` is (top, slope, lower) as in the shader;
## `eyes` swaps the eyes for shut or spiral ones; `mouth` for one of the expression mouths; `brow_turn` (radians, raising the outer
## ends: angry) and `brow_lift` move the brows; `pupil` and `scale` resize the pupils and the eyes; `look` is how far the pupils look ahead.
## (Art review 10, refining how they are applied: `brows` is the brow an expression borrows from BROWS when the face has none, so the
## brows' turn reads on the concept's browless face (the calm faces stay browless); `mouth_scale` draws an action mouth larger, so a yell
## or a wince reads at play size rather than as a speck.)
const EXPRESSIONS := {
	# (A little lidded at rest, the character pass, 2026-10-19: the reference cast's neutral face is unimpressed, not wide-eyed.)
	# (The resting moods since the reference pass, 2026-10-20, the owner's pick of mocked faces: the reference's round open eye as the base,
	# each mood varying it with an arched lid; no borrowed brows at rest, the owner: no brows by default.)
	"deadpan": {"lids": Vector3(0.0, 0.0, 0.0)},
	"sleepy": {"lids": Vector3(0.38, 0.0, 0.0), "brow_lift": -0.01},
	"grumpy": {"lids": Vector3(0.24, 0.24, 0.0), "brow_turn": 0.3, "brow_lift": -0.02},
	# (Looking sideways, the reference ranger's sly look, the character judge's round 5.)
	"smug": {"lids": Vector3(0.2, 0.0, 0.16), "brow_turn": -0.06, "look": 0.075},
	"blink": {"eyes": "blink"},
	"happy": {"eyes": "happy", "mouth": "x_happy", "brow_lift": 0.015, "mouth_scale": 1.3},
	"focus": {"lids": Vector3(0.24, 0.17, 0.0), "mouth": "x_focus", "brow_turn": 0.2, "brow_lift": -0.01, "brows": 3, "mouth_scale": 1.2},
	"attack": {"lids": Vector3(0.3, 0.38, 0.0), "pupil": 0.85, "mouth": "x_attack", "brow_turn": 0.4, "brow_lift": -0.025, "brows": 3, "mouth_scale": 1.7},
	"effort": {"eyes": "effort", "mouth": "x_effort", "brow_turn": 0.35, "brow_lift": -0.03, "brows": 3, "mouth_scale": 1.6},
	"hurt": {"eyes": "hurt", "mouth": "x_hurt", "brow_turn": -0.35, "brow_lift": 0.01, "brows": 3, "mouth_scale": 1.5},
	"shock": {"scale": 1.08, "pupil": 0.55, "look": 0.0, "mouth": "x_shock", "brow_lift": 0.03, "mouth_scale": 1.5},
	"dazed": {"eyes": "dazed", "mouth": "x_dazed", "brow_turn": -0.25, "mouth_scale": 1.2},
}
## Which expressions shut the eyes (1) or put a spiral in them (2).
const SHUT := {"blink": 1, "happy": 1, "effort": 1, "hurt": 1, "dazed": 2}


static func steps(slot: String) -> int:
	return ADJUST.get(slot, 1)


static func neutral(slot: String) -> int:
	return (steps(slot) - 1) / 2


## How many steps off its own place an adjustment is (negative: down, closer, smaller, turned in, squashed).
static func offset(look: RefCounted, slot: String) -> int:
	return int(look.get(slot)) - neutral(slot)


static func tex(path: String) -> Texture2D:
	return SvgArt.texture(DIR + path + ".svg", {}, DETAIL)


## Sets everything on a face material (face.gdshader or face_ui.gdshader) for `look` pulling expression `expression`. The fighter's
## colour (`skin`, and the lids a shade darker) comes from the look's body colour.
static func apply(mat: ShaderMaterial, look: RefCounted, expression := "deadpan", body := Color(-1, 0, 0)) -> void:
	var e: Dictionary = EXPRESSIONS.get(expression, {})
	var skin: Color = body if body.r >= 0.0 else look.body_color()
	mat.set_shader_parameter("lid_color", skin.darkened(0.1))

	# Eyes.
	var eye: String = EYES[look.eyes]
	var size_ratio := 1.0 + offset(look, "eye_size") * 0.06
	var gap := EYE_GAP + offset(look, "eye_gap") * 0.012
	var eye_y := EYE_Y + offset(look, "eye_y") * 0.011
	var squash := offset(look, "eye_squash") * 0.06
	mat.set_shader_parameter("eye_place", Vector4(gap, eye_y, EYE_SIZE * size_ratio * float(e.get("scale", 1.0)), offset(look, "eye_turn") * 0.07))
	mat.set_shader_parameter("eye_squash", Vector2(1.0 - squash * 0.6, 1.0 + squash))
	mat.set_shader_parameter("eye_fill_tex", tex("eyes/%s_fill" % eye))
	mat.set_shader_parameter("eye_line_tex", tex("eyes/%s_line" % eye))
	# (No outline round the eye since the reference pass, but for the lashes eye, whose lashes are in its line.)
	mat.set_shader_parameter("eye_line", 1.0 if eye == "lash" else 0.0)
	var shut: int = SHUT.get(str(e.get("eyes", "")), 0)
	mat.set_shader_parameter("eye_mode", shut)
	mat.set_shader_parameter("eye_extra_tex", tex("eyes/closed_" + str(e.eyes)) if shut > 0 else null)
	mat.set_shader_parameter("lids", e.get("lids", Vector3.ZERO))
	mat.set_shader_parameter("pupil_size", PUPIL * float(e.get("pupil", 1.0)))
	mat.set_shader_parameter("catchlight", CATCHLIGHT)
	mat.set_shader_parameter("pupil_shift", Vector2(float(e.get("look", LOOK.x)), LOOK.y))
	mat.set_shader_parameter("eye_color", EYE_COLORS[look.eye_color])

	# Brows.
	var brow := int(look.brows) if int(look.brows) > 0 else int(e.get("brows", 0))
	mat.set_shader_parameter("brow_tex", tex("brows/" + BROWS[brow]) if brow > 0 else null)
	# (The curl is one mark in the middle of the forehead, the character judge's round 7: drawn per side like brows it showed twice; its two
	# mirrored halves meet at the middle into one curl.)
	var curl: bool = BROWS[brow] == "curl"
	mat.set_shader_parameter("brow_place", Vector4(0.035 if curl else BROW_GAP + offset(look, "brow_gap") * 0.012,
		BROW_Y + offset(look, "brow_y") * 0.011 - float(e.get("brow_lift", 0.0)),
		BROW_SIZE * (1.1 if curl else 1.0 + offset(look, "brow_size") * 0.07), 0.0 if curl else offset(look, "brow_turn") * 0.09 + float(e.get("brow_turn", 0.0))))
	mat.set_shader_parameter("brow_color", BROW_COLORS[look.brow_color])
	# (Drawn thick, 2026-10-18: the reference cast's brows are chunky dark bars; at the drawings' own weight they read as thin lines.)
	mat.set_shader_parameter("brow_squash", Vector2(1.0, 1.0 if curl else BROW_THICK))

	# Nose.
	var nose := int(look.nose)
	mat.set_shader_parameter("nose_tex", tex("noses/" + NOSES[nose]) if nose > 0 else null)
	mat.set_shader_parameter("nose_place", Vector3(0.5, NOSE_Y + offset(look, "nose_y") * 0.01, NOSE_SIZE * (1.0 + offset(look, "nose_size") * 0.08)))

	# Mouth: the player's own, or the expression's.
	var mouth: String = str(e.get("mouth", MOUTHS[look.mouth]))
	var ms := offset(look, "mouth_squash") * 0.07
	mat.set_shader_parameter("mouth_tex", tex("mouths/" + mouth))
	mat.set_shader_parameter("mouth_place", Vector4(0.5, MOUTH_Y + offset(look, "mouth_y") * 0.011,
		MOUTH_SIZE * (1.0 + offset(look, "mouth_size") * 0.07) * float(e.get("mouth_scale", 1.0)), 0.0))
	mat.set_shader_parameter("mouth_squash", Vector2(1.0 + ms, 1.0 - ms * 0.5))
	# The face uneven (the character judge's round 12, after the reference's drawn faces): one lid a little lower and the mouth off to its side.
	var lean := offset(look, "face_lean")
	mat.set_shader_parameter("lid_lean", lean * 0.08)
	var mp: Vector4 = mat.get_shader_parameter("mouth_place")
	mat.set_shader_parameter("mouth_place", Vector4(0.5 + lean * 0.018, mp.y, mp.z, mp.w + lean * 0.05))

	# Cheeks, following the eyes.
	var cheek := int(look.cheeks)
	mat.set_shader_parameter("cheek_tex", tex("cheeks/" + CHEEKS[cheek]) if cheek > 0 else null)
	mat.set_shader_parameter("cheek_place", Vector3(gap + CHEEK_OUT * size_ratio, eye_y + CHEEK_DOWN * size_ratio, CHEEK_SIZE * size_ratio))
	var fr := int(look.get("freckles")) if look.get("freckles") != null else 0
	mat.set_shader_parameter("freckle_tex", tex("freckles/" + FRECKLES[fr]) if fr > 0 else null)
	mat.set_shader_parameter("freckle_color", FRECKLE_COLOR)
	var cc: Color = CHEEK_COLORS[look.cheek_color]
	mat.set_shader_parameter("cheek_color", Color(cc, 0.85))


## Where the eyes are on the head's face shell (shown `zoom` times larger than the face square, about its middle), in the shell's
## 512-square: the row of their middles and how many columns out from the middle each is. Glasses are fitted to these.
static func eye_row(look: RefCounted, zoom := 1.15) -> float:
	var y := EYE_Y + offset(look, "eye_y") * 0.011
	return ((y - 0.5) * zoom + 0.5) * 512.0


static func eye_out(look: RefCounted, zoom := 1.15) -> float:
	return (EYE_GAP + offset(look, "eye_gap") * 0.012) * zoom * 512.0
