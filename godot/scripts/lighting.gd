extends RefCounted
## The game's lighting and camera look, shared by the match and the menus' 3D previews: a warm key light with soft shadows, a cool fill
## from the other side, warm ambient light, a filmic tone curve, a gentle bloom, soft ambient occlusion and a touch more saturation. With
## the soft cel shader (`shaders/toon.gdshader`) this gives the painterly, warm-lit look of the art reference.


## Sets up `env` (and adds the lights under `root`). `preview` uses lighter settings for the small menu viewports.
static func apply(env: Environment, root: Node, preview := false) -> void:
	env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	env.ambient_light_color = Color(1.0, 0.9, 0.86)
	env.ambient_light_energy = 0.5
	env.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	env.tonemap_exposure = 1.05
	env.tonemap_white = 5.0
	env.glow_enabled = true
	env.glow_intensity = 0.3
	env.glow_strength = 0.9
	env.glow_bloom = 0.04
	env.glow_hdr_threshold = 0.95
	env.glow_blend_mode = Environment.GLOW_BLEND_MODE_SOFTLIGHT
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
