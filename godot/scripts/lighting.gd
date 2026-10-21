extends RefCounted
## The game's lighting and camera look, shared by the match and the menus' 3D previews: a warm key light with soft shadows, a cool fill
## from the other side, warm ambient light, a filmic tone curve, a bloom kept for the effects and a touch more saturation (no screen-space
## ambient occlusion: under the cel light it made no visible difference, so contact shade is painted instead; see `StageArt._contact`). With the soft cel shader (`shaders/toon.gdshader`) this gives the painterly, warm-lit look of the art reference.
##
## Each stage theme has a mood (`MOODS`): the colour and angle of its light, its ambient light, the hue its shadows are painted toward and
## the colour of the rim light (taken from its sky), applied with `stage_mood`. The shadow and rim colours are global shader parameters
## (`pf_*`), so every cel surface follows them.

## The look of each theme: key light [colour, energy, pitch, yaw], fill [colour, energy], ambient [colour, energy], shadows (the hue they
## turn toward 0..1, how far at most, how dark, the tint for pale colours), the rim colour, the colour grade and the haze its darks are
## lifted toward (the pastel finish in shaders/post.gdshader).
const VisualSettings := preload("res://scripts/visual_settings.gd")

const MOODS := {
	# Magic hour: a lower, golden key that rakes across the props and fighters, violet fill and shadows, a lavender-peach haze.
	# (Day stages' shade deeper, art review 6 round 21 after the owner's Wind Waker HD frame: lit and shaded faces of the props and
	# scenery split clearly, as the reference's rocks and walls; meadow and grove 0.56, was 0.64; ocean 0.54, was 0.6.)
	# (A warmer key and a cool blue-green shade, the gauntlet's round 11: the green stages' shade read neutral beside the benchmark's
	# coloured light.)
	# (Round 12: that overshot into a yellow-green cast over every material. A cream-peach key; the cool kept to the shade.)
	"meadow": {"key": [Color(1.0, 0.94, 0.88), 1.38, -44.0, -34.0], "fill": [Color(0.58, 0.76, 1.0), 0.4],
		# (Shade a deeper version of each colour, barely turned toward blue-green, the meadow rebuild's step 3: the reference's grass shade
		# is a crisp deep green, ours read as a soft olive.)
		"ambient": [Color(0.88, 0.92, 1.0), 0.48], "shadow_hue": 0.4, "shadow_shift": 0.1, "shadow_value": 0.38,
		"shadow_tint": Color(0.6, 0.78, 0.74), "rim": Color(1.0, 0.98, 0.9),
		"grade": [Color(0.92, 0.97, 1.06), Color(1.04, 1.02, 0.96), 0.5], "haze": Color(0.72, 0.87, 1.0)},
	# (Art review 6 round 9: a weaker key, more pink ambient and a stronger grade than the meadow left the grove under a pale veil, the
	# softest stage. Its colours kept; its contrast brought near the meadow's.)
	# (Its pink grade and ambient eased, round 24: pink over the lawn's green cancelled toward grey, the palest stage left.)
	"grove": {"key": [Color(1.0, 0.92, 0.84), 1.32, -40.0, -28.0], "fill": [Color(0.86, 0.66, 1.0), 0.38],
		"ambient": [Color(1.0, 0.93, 0.95), 0.47], "shadow_hue": 0.78, "shadow_shift": 0.06, "shadow_value": 0.42,
		"shadow_tint": Color(0.72, 0.62, 0.92), "rim": Color(1.0, 0.86, 0.92),
		"grade": [Color(0.98, 0.9, 1.04), Color(1.04, 0.98, 0.96), 0.3], "haze": Color(0.84, 0.82, 1.0)},
	"sunset": {"key": [Color(1.0, 0.72, 0.5), 1.25, -20.0, -42.0], "fill": [Color(0.6, 0.5, 1.0), 0.42],
		"ambient": [Color(0.94, 0.84, 0.88), 0.5], "shadow_hue": 0.79, "shadow_shift": 0.1, "shadow_value": 0.58,
		"shadow_tint": Color(0.72, 0.58, 0.84),
		# (A cool rim from the dusk sky overhead, art review 6 round 4: an orange rim on warm fighters against warm rock added no separation.)
		"rim": Color(0.84, 0.8, 1.0),
		"grade": [Color(0.88, 0.8, 1.05), Color(1.08, 0.96, 0.86), 0.45], "haze": Color(1.0, 0.86, 0.88)},
	"night": {"key": [Color(0.72, 0.8, 1.0), 0.95, -48.0, -24.0], "fill": [Color(0.5, 0.56, 0.95), 0.4],
		"ambient": [Color(0.58, 0.64, 0.95), 0.55], "shadow_hue": 0.66, "shadow_shift": 0.1, "shadow_value": 0.56,
		"shadow_tint": Color(0.62, 0.66, 0.92), "rim": Color(0.62, 0.78, 1.0),
		"grade": [Color(0.8, 0.88, 1.1), Color(0.95, 0.98, 1.06), 0.8], "haze": Color(0.72, 0.74, 0.96)},
	"ocean": {"key": [Color(1.0, 0.96, 0.88), 1.3, -44.0, -30.0], "fill": [Color(0.56, 0.78, 1.0), 0.38],
		"ambient": [Color(0.9, 0.95, 1.0), 0.46], "shadow_hue": 0.6, "shadow_shift": 0.07, "shadow_value": 0.54,
		"shadow_tint": Color(0.72, 0.8, 0.92), "rim": Color(0.86, 0.96, 1.0),
		"grade": [Color(0.9, 0.98, 1.06), Color(1.02, 1.02, 0.98), 0.5], "haze": Color(0.62, 0.84, 1.0)},
	"city": {"key": [Color(1.0, 0.82, 0.7), 1.0, -34.0, -36.0], "fill": [Color(0.82, 0.5, 0.95), 0.48],
		"ambient": [Color(0.84, 0.8, 0.94), 0.55], "shadow_hue": 0.75, "shadow_shift": 0.1, "shadow_value": 0.55,
		"shadow_tint": Color(0.66, 0.6, 0.88), "rim": Color(1.0, 0.62, 0.78),
		"grade": [Color(0.92, 0.82, 1.08), Color(1.06, 0.97, 0.98), 0.4], "haze": Color(0.68, 0.58, 0.9)},
}


## The menus' 3D previews: plain warm daylight (the meadow's look before it turned to magic hour), so a stage's mood never reaches them.
const MENU_MOOD := {"shadow_hue": 0.73, "shadow_shift": 0.07, "shadow_value": 0.64, "shadow_tint": Color(0.74, 0.7, 0.88),
	"rim": Color(1.0, 0.94, 0.84)}


## Each stage's shading profile, on top of its mood (anything left out takes `STYLE_DEFAULT`): the cel ramp's bands (`ramp`: stops of
## [where on the way from facing away to facing the light, how lit], drawn into `pf_ramp`), how much darker a cast shadow makes the shadow
## side (`deep`), the pale edge light round the fighters (`edge`, its alpha the strength: strong on dark stages so no fighter sinks into
## the background), and the painted gradient over the scenery (`paint`: the warm colour tops lean to, the cool one undersides lean to,
## and how far; fighters take a third of it).
const STYLE_DEFAULT := {
	"ramp": [[0.0, 0.0], [0.34, 0.0], [0.42, 0.56], [0.55, 0.56], [0.62, 1.0], [1.0, 1.0]],
	"deep": 0.78,
	# (Half, the character judge's round 10: close up in the menus the pale rim read as a sticker edge round every piece.)
	"edge": Color(1.0, 0.97, 0.9, 0.0),
	"paint": [Color(1.06, 1.01, 0.92), Color(0.9, 0.94, 1.06), 0.55],
	# How strongly the effects and lit things (the city's windows, lamps) bloom.
	"glow": 0.75,
	# How much deeper the scenery's shade side is than its painted shadow colour (`pf_scenery_shade` in shaders/toon_light.gdshaderinc).
	"scenery_shade": 0.32,
	# The finishing pass's value range (shaders/post.gdshader): how far colours are eased toward grey (`desat`), how far the darks are
	# lifted toward the haze (`lift`), the vignette, how far the far scenery fades into the haze at most (`haze_max`) and a gentle
	# S-curve on the values (`s_curve`, 0 none). The fighters take half of the pastel and the curve.
	# `bloom`: the soft glow of everything bright (the sky, clouds, sunlit walls), and the luma it starts from (`bloom_from`).
	# Cloud shadows drifting over the stage (`pf_clouds` in shaders/toon_light.gdshaderinc): [how deep, units across, sky covered, drift].
	"clouds": [0.0, 24.0, 0.4, 0.5],
	# (`quiet` 0.25, art review 6 round 19, after the owner's Wind Waker HD reference: at 0.5 every stage was greyed by a fifth, so the
	# scenery read milky beside the reference's clean, saturated colour; the fighters keep their lift over it.)
	"finish": {"desat": 0.0, "lift": 0.03, "vignette": 0.16, "haze_max": 0.22, "s_curve": 0.22, "quiet": 0.25, "bloom": 0.4,
		"bloom_from": 0.6},
}
const STYLES := {
	# Dappled light under the great tree: small patches swaying slowly.
	"grove": {"paint": [Color(1.06, 0.99, 0.94), Color(0.94, 0.9, 1.06), 0.55], "clouds": [0.72, 4.5, 0.45, 0.25],
		"finish": {"desat": 0.0, "lift": 0.02, "haze_max": 0.08, "s_curve": 0.3, "quiet": 0.08}},
	# Crisper light-to-shadow steps (a clear lit side and shaded side on every prop, in the manner of a cel-shaded adventure), warmer tops
	# and cooler undersides, and less of the pastel wash, so the colours carry.
	"meadow": {"ramp": [[0.0, 0.0], [0.38, 0.0], [0.43, 0.52], [0.55, 0.52], [0.6, 1.0], [1.0, 1.0]],
		"paint": [Color(1.1, 1.02, 0.88), Color(0.88, 0.88, 1.1), 0.65], "deep": 0.74, "clouds": [0.65, 14.0, 0.42, 0.7],
		# (Less haze and no greying of the scenery, 2026-10-17: together they washed the mid-distance cottages, trees and mountains into
		# flat pastel shapes, the owner's "flat" feel; depth still reads through the cooler, lighter far colours.)
		"finish": {"desat": 0.0, "lift": 0.01, "haze_max": 0.06, "s_curve": 0.3, "quiet": 0.0}},
	"sunset": {"clouds": [0.35, 30.0, 0.3, 0.4], "edge": Color(1.0, 0.86, 0.74, 0.44), "paint": [Color(1.1, 0.98, 0.86), Color(0.86, 0.86, 1.08), 0.7], "deep": 0.72,
		"finish": {"desat": 0.0, "lift": 0.02, "haze_max": 0.08, "s_curve": 0.3, "quiet": 0.08}},
	"night": {"edge": Color(0.78, 0.86, 1.0, 0.62), "paint": [Color(0.98, 1.0, 1.08), Color(0.86, 0.9, 1.1), 0.5], "deep": 0.74, "glow": 0.95,
		"ramp": [[0.0, 0.0], [0.36, 0.0], [0.44, 0.5], [0.56, 0.5], [0.64, 1.0], [1.0, 1.0]]},
	# (The harbour dock, 2026-10-18, the soft-shading trial from docs/COMPETITOR_GAP.md: a smooth turn from light into shade over the
	# scenery rather than cel bands, and fainter ink on it; the modelled planks, piles and crates carry the detail now.)
	"ocean": {"paint": [Color(1.05, 1.02, 0.94), Color(0.9, 0.97, 1.06), 0.5], "clouds": [0.55, 26.0, 0.36, 0.9],
		"ramp": [[0.0, 0.0], [0.3, 0.1], [0.45, 0.45], [0.6, 0.82], [0.75, 1.0], [1.0, 1.0]],
		"finish": {"desat": 0.0, "lift": 0.02, "haze_max": 0.07, "s_curve": 0.28, "quiet": 0.08, "scenery_lines": 0.18}},
	"city": {"edge": Color(1.0, 0.82, 0.92, 0.55), "paint": [Color(1.06, 0.96, 0.96), Color(0.9, 0.88, 1.1), 0.6], "deep": 0.72, "glow": 1.1,
		"finish": {"lift": 0.02, "s_curve": 0.48, "quiet": 0.12}},
}


## The shading profile for theme `name` (its own values over the defaults).
static func style(name: String) -> Dictionary:
	var own: Dictionary = STYLES.get(name, {})
	var out := STYLE_DEFAULT.duplicate()
	out.merge(own, true)
	# (The finish merges value by value: a stage that sets its own haze keeps the default's value hierarchy and vignette. A whole-dictionary
	# merge dropped them, so `quiet` was off on every stage.)
	var finish: Dictionary = (STYLE_DEFAULT.finish as Dictionary).duplicate()
	finish.merge(own.get("finish", {}), true)
	out.finish = finish
	return out


static var _ramps := {}

## The paper look under trial (the papercraft adventure as the benchmark, the owner, 2026-10-18), an A/B against the current look: no ink
## on the scenery (its edges lit instead, a cut card's edge), a soft turn into a lighter, cooler shade on it, and soft occlusion where
## things meet. Fighters unchanged. Retired as the game's look (the owner, 2026-10-19: the current look is better; docs/art/STYLE_SHEET.md);
## `--look=paper` still starts with it on, for comparison captures only.
static var paper := OS.get_cmdline_user_args().has("--look=paper")


## The paper finish on the scenery under trial (the stage refinement's paper pilot, 2026-10-20; not the retired `paper` look above: the
## ink and the light stay): `--paper-finish=A|B|C` for the pilot's captures, [fibre grain, sheet wobble, cut-edge rim]. Off by default.
static func paper_finish() -> Vector4:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--paper-finish="):
			return {"A": Vector4(0.6, 0.5, 0.0, 0.0), "B": Vector4(0.6, 0.5, 0.6, 0.0), "C": Vector4(1.0, 0.8, 0.9, 0.0)}.get(a.substr(15), Vector4.ZERO)
	return Vector4.ZERO


## Applies `paper` to the globals, the environment and the finishing pass `post` (if any).
static func apply_paper(env: Environment, post: ShaderMaterial, base_lines: float) -> void:
	RenderingServer.global_shader_parameter_set("pf_paper", 1.0 if paper else 0.0)
	RenderingServer.global_shader_parameter_set("pf_paper_finish", paper_finish())
	if env != null:
		# (Soft occlusion where things meet in the current look too, gentler than the paper look's, the gauntlet's round 20: the wall's
		# joints, the boards' gaps, under the lawn's lip and round the props' feet read lit flat. Off on Fast.)
		env.ssao_enabled = VisualSettings.get_index("quality") > 0
		# (A tighter radius and stronger, the owner, 2026-10-19: the soft shade where a fighter's pieces meet (under the hats, at the
		# neck, the sash, between the arms and the body) comes from here; at 1.2 it was too broad for a fighter's small joins.)
		env.ssao_radius = 1.6 if paper else 0.7
		env.ssao_intensity = 3.0 if paper else 2.4
		env.ssao_power = 1.4 if paper else 1.25
		env.ssao_light_affect = 0.6 if paper else 0.35
		env.ssao_ao_channel_affect = 1.0
	if post != null:
		post.set_shader_parameter("scenery_lines", 0.0 if paper else base_lines)
		# (A lighter dose of the cut card's lit edge in the current look too, the gauntlet's round 23: the benchmark's crisp lit edges.)
		# (0.8, round 24: at 0.4 it did not read at match distance.)
		post.set_shader_parameter("paper_edge", 0.9 if paper else 0.8)


## The ramp's lookup texture: 256 steps across, each the ramp's lightness there, eased between the stops so each band's edge is soft.
static func ramp_texture(stops: Array) -> ImageTexture:
	var key := str(stops)
	if _ramps.has(key):
		return _ramps[key]
	var img := Image.create(256, 1, false, Image.FORMAT_L8)
	for i in 256:
		var x := i / 255.0
		var v := 0.0
		for k in stops.size() - 1:
			var a: Array = stops[k]
			var b: Array = stops[k + 1]
			if x >= float(a[0]) and x <= float(b[0]):
				var t := (x - float(a[0])) / maxf(float(b[0]) - float(a[0]), 0.0001)
				v = lerpf(float(a[1]), float(b[1]), t * t * (3.0 - 2.0 * t))
				break
		img.set_pixel(i, 0, Color(v, v, v))
	var tex := ImageTexture.create_from_image(img)
	_ramps[key] = tex
	return tex


## How many times white the cores of the effects are drawn (so they, and only they, bloom).
const FX_GLOW := 4.0


## Sets up `env` (and adds the lights under `root`). `preview` uses lighter settings for the small menu viewports.
static func apply(env: Environment, root: Node, preview := false) -> void:
	env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	env.ambient_light_color = Color(1.0, 0.9, 0.86)
	env.ambient_light_energy = 0.5
	env.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	env.tonemap_exposure = 1.05
	env.tonemap_white = 5.0
	# Only the effects glow: their bright cores are pushed to `FX_GLOW` times white (shaders/fx.gdshaderinc), above a threshold that the
	# brightest lit surface (a white shirt in full key light, about 1.9) stays under, so the fighters and the stage stay flat.
	env.glow_enabled = true
	env.glow_intensity = 0.75
	env.glow_strength = 1.0
	env.glow_bloom = 0.0
	env.glow_hdr_threshold = 2.3
	env.glow_hdr_scale = 2.0
	env.glow_blend_mode = Environment.GLOW_BLEND_MODE_SCREEN
	RenderingServer.global_shader_parameter_set("pf_fx_glow", FX_GLOW)
	env.adjustment_enabled = true
	# (Neutral since the muted trial, 2026-10-18, the owner after the reference cast: 1.1 and 1.06 pushed every colour, fighters and stages
	# alike, toward striking rather than calm.)
	env.adjustment_saturation = 1.0
	env.adjustment_contrast = 1.02
	# (The haze behind the stage is the match's own, in its stage's colour (`haze_*` in shaders/post.gdshader), not the engine's fog.)
	env.ssao_enabled = false
	var key := DirectionalLight3D.new()
	key.name = "KeyLight"
	key.rotation_degrees = Vector3(-38, -32, 0)
	# (The menus' previews lit from the side as a match is, `KEY_SIDE`, the character pass, 2026-10-19: from nearly in front the edge
	# between light and shade fell across the flat chest as a pale soft patch.)
	if preview:
		key.rotation_degrees.y += KEY_SIDE
		# (The fighters' soft shade where their pieces meet, the ambient occlusion, in the previews too, as in a match.)
		env.ssao_enabled = true
		# (Stronger than a match's, the character judge's round 11: at a menu's size the shade under the brims did not read.)
		env.ssao_radius = 0.8
		env.ssao_intensity = 3.6
		env.ssao_power = 1.35
		env.ssao_light_affect = 0.35
	key.light_color = Color(1.0, 0.93, 0.82)
	key.light_energy = 1.25
	key.shadow_enabled = not preview
	# Crisp, as an illustration's cast shadows are (a soft wide penumbra read as haze on the grass, not shade), with just enough blur to
	# keep the edge clean; the shadow range only as far
	# as the camera ever is from the stage, so the shadow map's detail goes where the fighting is.
	key.shadow_blur = 0.3
	key.light_angular_distance = 5.0
	key.directional_shadow_max_distance = 70.0
	key.directional_shadow_mode = DirectionalLight3D.SHADOW_PARALLEL_2_SPLITS
	key.directional_shadow_split_1 = 0.35
	root.add_child(key)
	var fill := DirectionalLight3D.new()
	fill.name = "FillLight"
	fill.rotation_degrees = Vector3(-12, 150, 0)
	fill.light_color = Color(0.62, 0.72, 1.0)
	fill.light_energy = 0.35
	root.add_child(fill)
	# The menus (and anything lit before a stage is chosen) use their own neutral daylight mood, with no stage shading.
	_set_globals(MENU_MOOD, style("menu"))
	RenderingServer.global_shader_parameter_set("pf_ao_strength", 0.0)


## Lights the scene for the stage's theme `theme` (see `StageArt.theme`): its mood, with the rim taking a little of the sky's horizon, and
## its colour grade on `post` (the match's finishing material, shaders/post.gdshader) if there is one. `root` holds the lights made by
## `apply`.
## How much further round to the side every stage's key light comes (degrees of yaw on top of the mood's): from nearly behind the camera
## the light left round forms (heads, bodies, barrels) one flat value and cast every shadow straight back, out of sight. From the side and
## a little behind, as the sun in a cel-shaded adventure, every prop has a lit side and a shaded side, cast shadows fall across the
## fighting surface where they can be seen, and the stage's front wall turns from the light into its painted shade, so the lit lawn leads
## and the mass under it settles back. (The sunset's own yaw points its light back from its visible sun.)
const KEY_SIDE := -80.0


## How calm the stage fronts are (0 as built, 1 fully calmed; see `pf_front_calm`).
const FRONT_CALM := 0.75


static func stage_mood(env: Environment, root: Node, theme: Dictionary, post: ShaderMaterial = null) -> void:
	var mood: Dictionary = MOODS.get(theme.get("name", "meadow"), MOODS.meadow)
	var key := root.get_node_or_null("KeyLight") as DirectionalLight3D
	if key != null:
		key.light_color = mood.key[0]
		key.light_energy = mood.key[1]
		key.rotation_degrees = Vector3(mood.key[2], float(mood.key[3]) + KEY_SIDE, 0)
		# Toward the sun (the light shines down its -Z), for the sky's pieces lit by hand: clouds, ridges (shaders/cloud.gdshader,
		# painted_ridge.gdshader).
		RenderingServer.global_shader_parameter_set("pf_sun_dir", key.basis.z.normalized())
	var fill := root.get_node_or_null("FillLight") as DirectionalLight3D
	if fill != null:
		fill.light_color = mood.fill[0]
		fill.light_energy = mood.fill[1]
	env.ambient_light_color = mood.ambient[0]
	env.ambient_light_energy = mood.ambient[1]
	env.glow_intensity = float(style(theme.get("name", "meadow")).glow)
	if post != null:
		post.set_shader_parameter("grade_shadows", mood.grade[0])
		post.set_shader_parameter("grade_lights", mood.grade[1])
		post.set_shader_parameter("grade_amount", mood.grade[2])
		post.set_shader_parameter("pastel_haze", mood.get("haze", Color(0.88, 0.9, 1.0)))
		var finish: Dictionary = STYLE_DEFAULT.finish.duplicate()
		finish.merge(style(theme.get("name", "meadow")).get("finish", {}), true)
		post.set_shader_parameter("pastel_desat", finish.desat)
		post.set_shader_parameter("pastel_lift", finish.lift)
		post.set_shader_parameter("vignette", finish.vignette)
		post.set_shader_parameter("haze_max", finish.haze_max)
		post.set_shader_parameter("stage_quiet", float(finish.get("quiet", 0.0)))
		post.set_shader_parameter("s_curve", finish.s_curve)
		post.set_shader_parameter("bloom", finish.bloom)
		post.set_shader_parameter("bloom_threshold", finish.bloom_from)
		post.set_shader_parameter("scenery_lines", float(finish.get("scenery_lines", 0.4)))
	apply_paper(env, post, float(style(theme.get("name", "meadow")).finish.get("scenery_lines", 0.4)))
	# The stage fronts calmed into the stage's own haze (`pf_front_calm`, shaders/toon_common.gdshaderinc; the gauntlet's round 1).
	var haze: Color = (theme.get("sky_horizon", Color(0.8, 0.85, 0.95)) as Color).lerp(mood.ambient[0], 0.4)
	RenderingServer.global_shader_parameter_set("pf_front_calm", Color(haze.r, haze.g, haze.b, FRONT_CALM))
	var m := mood.duplicate()
	if theme.has("sky_horizon"):
		m.rim = (mood.rim as Color).lerp(theme.sky_horizon, 0.3)
	_set_globals(m, style(theme.get("name", "meadow")))


static func _set_globals(mood: Dictionary, look: Dictionary) -> void:
	RenderingServer.global_shader_parameter_set("pf_ramp", ramp_texture(look.ramp))
	RenderingServer.global_shader_parameter_set("pf_deep_shade", look.deep)
	RenderingServer.global_shader_parameter_set("pf_edge_light", look.edge)
	RenderingServer.global_shader_parameter_set("pf_paint_warm", look.paint[0])
	RenderingServer.global_shader_parameter_set("pf_paint_cool", look.paint[1])
	RenderingServer.global_shader_parameter_set("pf_paint_amount", look.paint[2])
	RenderingServer.global_shader_parameter_set("pf_shadow_hue", mood.shadow_hue)
	RenderingServer.global_shader_parameter_set("pf_shadow_shift", mood.shadow_shift)
	RenderingServer.global_shader_parameter_set("pf_shadow_value", mood.shadow_value)
	RenderingServer.global_shader_parameter_set("pf_shadow_tint", mood.shadow_tint)
	RenderingServer.global_shader_parameter_set("pf_rim_color", mood.rim)
	RenderingServer.global_shader_parameter_set("pf_scenery_shade", float(look.get("scenery_shade", 1.0)))
	var cl: Array = look.clouds
	RenderingServer.global_shader_parameter_set("pf_clouds", Vector4(cl[0], cl[1], cl[2], cl[3]))
