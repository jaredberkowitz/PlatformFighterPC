extends RefCounted
## The game's lighting and camera look, shared by the match and the menus' 3D previews: a warm key light with soft shadows, a cool fill
## from the other side, warm ambient light, a filmic tone curve, a bloom kept for the effects, soft ambient occlusion and a touch more
## saturation. With the soft cel shader (`shaders/toon.gdshader`) this gives the painterly, warm-lit look of the art reference.
##
## Each stage theme has a mood (`MOODS`): the colour and angle of its light, its ambient light, the hue its shadows are painted toward and
## the colour of the rim light (taken from its sky), applied with `stage_mood`. The shadow and rim colours are global shader parameters
## (`pf_*`), so every cel surface follows them.

## The look of each theme: key light [colour, energy, pitch, yaw], fill [colour, energy], ambient [colour, energy], shadows (the hue they
## turn toward 0..1, how far at most, how dark, the tint for pale colours) and the rim colour.
const MOODS := {
	"meadow": {"key": [Color(1.0, 0.93, 0.82), 1.25, -38.0, -32.0], "fill": [Color(0.62, 0.72, 1.0), 0.35],
		"ambient": [Color(1.0, 0.9, 0.86), 0.5], "shadow_hue": 0.73, "shadow_shift": 0.07, "shadow_value": 0.64,
		"shadow_tint": Color(0.74, 0.7, 0.88), "rim": Color(1.0, 0.94, 0.84),
		"grade": [Color(0.92, 0.94, 1.04), Color(1.04, 1.0, 0.94), 0.5]},
	"grove": {"key": [Color(1.0, 0.92, 0.84), 1.2, -40.0, -28.0], "fill": [Color(0.86, 0.66, 1.0), 0.38],
		"ambient": [Color(1.0, 0.88, 0.92), 0.52], "shadow_hue": 0.8, "shadow_shift": 0.08, "shadow_value": 0.64,
		"shadow_tint": Color(0.8, 0.7, 0.88), "rim": Color(1.0, 0.86, 0.92),
		"grade": [Color(0.98, 0.9, 1.04), Color(1.04, 0.98, 0.96), 0.6]},
	"sunset": {"key": [Color(1.0, 0.72, 0.5), 1.25, -20.0, -42.0], "fill": [Color(0.6, 0.5, 1.0), 0.42],
		"ambient": [Color(0.86, 0.66, 0.84), 0.5], "shadow_hue": 0.79, "shadow_shift": 0.1, "shadow_value": 0.58,
		"shadow_tint": Color(0.72, 0.58, 0.84), "rim": Color(1.0, 0.68, 0.42),
		"grade": [Color(0.88, 0.8, 1.05), Color(1.08, 0.96, 0.86), 0.8]},
	"night": {"key": [Color(0.72, 0.8, 1.0), 0.95, -48.0, -24.0], "fill": [Color(0.5, 0.56, 0.95), 0.4],
		"ambient": [Color(0.58, 0.64, 0.95), 0.55], "shadow_hue": 0.66, "shadow_shift": 0.1, "shadow_value": 0.56,
		"shadow_tint": Color(0.62, 0.66, 0.92), "rim": Color(0.62, 0.78, 1.0),
		"grade": [Color(0.8, 0.88, 1.1), Color(0.95, 0.98, 1.06), 0.8]},
	"ocean": {"key": [Color(1.0, 0.96, 0.88), 1.3, -44.0, -30.0], "fill": [Color(0.56, 0.78, 1.0), 0.38],
		"ambient": [Color(0.9, 0.95, 1.0), 0.5], "shadow_hue": 0.6, "shadow_shift": 0.07, "shadow_value": 0.66,
		"shadow_tint": Color(0.72, 0.8, 0.92), "rim": Color(0.86, 0.96, 1.0),
		"grade": [Color(0.9, 0.98, 1.06), Color(1.02, 1.02, 0.98), 0.5]},
	"city": {"key": [Color(1.0, 0.82, 0.7), 1.0, -34.0, -36.0], "fill": [Color(0.82, 0.5, 0.95), 0.48],
		"ambient": [Color(0.7, 0.6, 0.9), 0.55], "shadow_hue": 0.75, "shadow_shift": 0.1, "shadow_value": 0.55,
		"shadow_tint": Color(0.66, 0.6, 0.88), "rim": Color(1.0, 0.62, 0.78),
		"grade": [Color(0.92, 0.82, 1.08), Color(1.06, 0.94, 1.0), 0.8]},
}


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
	env.adjustment_saturation = 1.12
	env.adjustment_contrast = 1.03
	if not preview:
		# Haze that thickens with distance behind the stage, so the backdrop sits back and the fighters stand out.
		env.fog_enabled = true
		env.fog_mode = Environment.FOG_MODE_DEPTH
		env.fog_light_color = Color(0.78, 0.86, 0.95)
		env.fog_light_energy = 1.0
		env.fog_density = 0.55
		env.fog_sky_affect = 0.0
		env.fog_depth_curve = 1.4
		env.fog_depth_begin = 45.0
		env.fog_depth_end = 160.0
		env.ssao_enabled = true
		env.ssao_radius = 1.4
		env.ssao_intensity = 1.2
		env.ssao_power = 1.4
	var key := DirectionalLight3D.new()
	key.name = "KeyLight"
	key.rotation_degrees = Vector3(-38, -32, 0)
	key.light_color = Color(1.0, 0.93, 0.82)
	key.light_energy = 1.25
	key.shadow_enabled = not preview
	key.shadow_blur = 2.0
	key.light_angular_distance = 1.5
	key.directional_shadow_max_distance = 90.0
	root.add_child(key)
	var fill := DirectionalLight3D.new()
	fill.name = "FillLight"
	fill.rotation_degrees = Vector3(-12, 150, 0)
	fill.light_color = Color(0.62, 0.72, 1.0)
	fill.light_energy = 0.35
	root.add_child(fill)
	# The menus (and anything lit before a stage is chosen) use the meadow's mood, with no stage shading.
	_set_globals(MOODS.meadow)
	RenderingServer.global_shader_parameter_set("pf_ao_strength", 0.0)


## Lights the scene for the stage's theme `theme` (see `StageArt.theme`): its mood, with the rim taking a little of the sky's horizon, and
## its colour grade on `post` (the match's finishing material, shaders/post.gdshader) if there is one. `root` holds the lights made by
## `apply`.
static func stage_mood(env: Environment, root: Node, theme: Dictionary, post: ShaderMaterial = null) -> void:
	var mood: Dictionary = MOODS.get(theme.get("name", "meadow"), MOODS.meadow)
	var key := root.get_node_or_null("KeyLight") as DirectionalLight3D
	if key != null:
		key.light_color = mood.key[0]
		key.light_energy = mood.key[1]
		key.rotation_degrees = Vector3(mood.key[2], mood.key[3], 0)
	var fill := root.get_node_or_null("FillLight") as DirectionalLight3D
	if fill != null:
		fill.light_color = mood.fill[0]
		fill.light_energy = mood.fill[1]
	env.ambient_light_color = mood.ambient[0]
	env.ambient_light_energy = mood.ambient[1]
	if post != null:
		post.set_shader_parameter("grade_shadows", mood.grade[0])
		post.set_shader_parameter("grade_lights", mood.grade[1])
		post.set_shader_parameter("grade_amount", mood.grade[2])
	var m := mood.duplicate()
	if theme.has("sky_horizon"):
		m.rim = (mood.rim as Color).lerp(theme.sky_horizon, 0.3)
	_set_globals(m)


static func _set_globals(mood: Dictionary) -> void:
	RenderingServer.global_shader_parameter_set("pf_shadow_hue", mood.shadow_hue)
	RenderingServer.global_shader_parameter_set("pf_shadow_shift", mood.shadow_shift)
	RenderingServer.global_shader_parameter_set("pf_shadow_value", mood.shadow_value)
	RenderingServer.global_shader_parameter_set("pf_shadow_tint", mood.shadow_tint)
	RenderingServer.global_shader_parameter_set("pf_rim_color", mood.rim)
