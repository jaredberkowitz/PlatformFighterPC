extends RefCounted
## The stage's baked soft light (presentation only). When a stage is built, the sky each point near the stage plane can see is worked out
## once from its blocks and platforms (the stage is flat, so this is done in 2D): for every texel, the part of the upper half of the sky
## hidden behind each block or platform, nearer blockers counting more. Inside a block the map shades from the top down instead, so its
## front is lighter under the grass and darker at the foot. The result is a small greyscale texture handed to every cel shader through
## global shader parameters (shaders/stage_ao.gdshaderinc); nothing happens at runtime but a texture read.

const TEXELS_PER_UNIT := 2.0
const MARGIN := Vector2(8.0, 6.0)
## Blockers further than this (world units) hide nothing.
const REACH := 9.0
## How strong the shading is (0 none, 1 full), and how much of the sky a blocker can hide at most.
const STRENGTH := 0.85
const BLOCKING := 0.65


## Bakes the map for `rects` (each [left, right, top, bottom, pass_through]) and installs it. Returns the texture.
static func bake(rects: Array) -> ImageTexture:
	if rects.is_empty():
		clear()
		return null
	var lo := Vector2(1e9, 1e9)
	var hi := Vector2(-1e9, -1e9)
	for r in rects:
		lo = lo.min(Vector2(r[0], r[3]))
		hi = hi.max(Vector2(r[1], r[2]))
	lo -= MARGIN
	hi += Vector2(MARGIN.x, MARGIN.y + 4.0)
	var size := Vector2i(ceili((hi.x - lo.x) * TEXELS_PER_UNIT), ceili((hi.y - lo.y) * TEXELS_PER_UNIT))
	var img := Image.create_empty(size.x, size.y, false, Image.FORMAT_L8)
	for ty in size.y:
		var y := lo.y + (float(ty) + 0.5) / TEXELS_PER_UNIT
		for tx in size.x:
			var x := lo.x + (float(tx) + 0.5) / TEXELS_PER_UNIT
			var v := _sky(x, y, rects)
			# (The texture's first row is the top of the stage.)
			img.set_pixel(tx, size.y - 1 - ty, Color(v, v, v))
	var tex := ImageTexture.create_from_image(img)
	RenderingServer.global_shader_parameter_set("pf_ao_map", tex)
	RenderingServer.global_shader_parameter_set("pf_ao_rect", Vector4(lo.x, lo.y, hi.x - lo.x, hi.y - lo.y))
	RenderingServer.global_shader_parameter_set("pf_ao_strength", STRENGTH)
	return tex


## No stage: no shading (the menus).
static func clear() -> void:
	RenderingServer.global_shader_parameter_set("pf_ao_strength", 0.0)


## How much of the sky (0..1) a point sees.
static func _sky(x: float, y: float, rects: Array) -> float:
	for r in rects:
		if float(r[4]) < 0.5 and x > r[0] and x < r[1] and y < r[2] and y > r[3]:
			# Inside a solid block (its front): lighter at the top, darker toward the foot.
			var k := clampf((y - float(r[3])) / maxf(0.1, float(r[2]) - float(r[3])), 0.0, 1.0)
			return lerpf(0.62, 1.0, sqrt(k))
	var hidden := 0.0
	for r in rects:
		var left: float = r[0]
		var right: float = r[1]
		var top: float = r[2]
		var bottom: float = r[3]
		if top <= y + 0.01:
			# Wholly below the point (or level with it, as the ground it stands on): it hides nothing of the sky above.
			continue
		# The nearest point of the block, and how much it counts.
		var near := Vector2(clampf(x, left, right), clampf(y, bottom, top))
		var dist := near.distance_to(Vector2(x, y))
		var weight := clampf(1.0 - dist / REACH, 0.0, 1.0)
		if weight <= 0.0:
			continue
		# The angles (0 east, PI west) the block covers seen from the point.
		var a_min := PI
		var a_max := 0.0
		for corner in [Vector2(left, bottom), Vector2(right, bottom), Vector2(left, top), Vector2(right, top)]:
			var a := atan2(corner.y - y, corner.x - x)
			# A corner below the horizon counts as on the horizon, on its own side.
			if a < 0.0:
				a = 0.0 if a > -PI * 0.5 else PI
			a_min = minf(a_min, a)
			a_max = maxf(a_max, a)
		# The sky is weighted toward straight up (as light from it falls): the share between two angles is (cos a - cos b) / 2.
		hidden += (cos(a_min) - cos(a_max)) * 0.5 * weight
	# (Light bounces in under things, so even a covered spot keeps some.)
	return clampf(1.0 - hidden * BLOCKING, 0.0, 1.0)
