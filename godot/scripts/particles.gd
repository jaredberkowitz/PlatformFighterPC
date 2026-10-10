extends RefCounted
## Particle effects (presentation only, never the simulation): one-shot bursts that free themselves when they are done (dust, sparks,
## knock-out explosions), and the looping flame worn by the brawler's rushing specials. A small particle system of its own: each
## particle is a camera-facing soft disc or a spinning star, drawn into one mesh, flat-coloured to sit with the cel art. The particles
## move every frame but are drawn on twos (30 pictures a second), like hand-drawn effects.

const FxMaterial := preload("res://scripts/fx_material.gd")

static var _soft: GradientTexture2D


## A soft round blob (white, fading out at the edge), the texture of every disc particle.
static func _soft_tex() -> GradientTexture2D:
	if _soft == null:
		var g := Gradient.new()
		g.set_color(0, Color(1, 1, 1, 1))
		g.set_color(1, Color(1, 1, 1, 0))
		g.add_point(0.6, Color(1, 1, 1, 0.95))
		_soft = GradientTexture2D.new()
		_soft.gradient = g
		_soft.fill = GradientTexture2D.FILL_RADIAL
		_soft.fill_from = Vector2(0.5, 0.5)
		_soft.fill_to = Vector2(1.0, 0.5)
		_soft.width = 64
		_soft.height = 64
	return _soft


static func _material(textured: bool, on_top: bool, glow := 1.0) -> ShaderMaterial:
	return FxMaterial.get_material(on_top, 4 if on_top else 1, _soft_tex() if textured else null, glow)


## A set of particles: each moves, slows, falls (or rises) and changes colour and size over its life. `spawn_per_second` above zero keeps
## emitting at the node's `source` (a looping effect); otherwise everything is spawned at once and the node frees itself when done.
class Burst extends MeshInstance3D:
	var parts: Array = []           # [pos, vel, age, life, size, rot, spin]
	var star := false
	var gravity := Vector3.ZERO
	var damping := 0.0
	var colours: Array = [Color.WHITE, Color.WHITE, Color(1, 1, 1, 0)]   # start, middle, end
	var grow := false               # discs that swell as they fade (smoke) rather than shrink (sparks)
	var looping := false
	var emitting := false
	var spawn_per_second := 0.0
	var source: Node3D
	var spawn: Callable             # () -> [pos, vel, life, size]
	var _carry := 0.0
	var _clock := 0.0
	var _drawn_step := -1

	func _ready() -> void:
		mesh = ImmediateMesh.new()
		top_level = true
		cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		global_transform = Transform3D.IDENTITY
		extra_cull_margin = 16384.0

	func add(pos: Vector3, vel: Vector3, life: float, size: float) -> void:
		parts.append([pos, vel, 0.0, life, size, randf() * TAU, randf_range(-8.0, 8.0) if star else 0.0])

	func _process(real_delta: float) -> void:
		var delta := FxMaterial.delta(real_delta)
		if looping and emitting and source != null and is_instance_valid(source):
			_carry += delta * spawn_per_second
			while _carry >= 1.0:
				_carry -= 1.0
				var s: Array = spawn.call(source.global_position)
				add(s[0], s[1], s[2], s[3])
		var keep: Array = []
		for p in parts:
			p[2] += delta
			if p[2] >= p[3]:
				continue
			p[1] = p[1] * maxf(0.0, 1.0 - damping * delta) + gravity * delta
			p[0] += p[1] * delta
			p[5] += p[6] * delta
			keep.append(p)
		parts = keep
		if parts.is_empty() and not looping:
			queue_free()
			return
		# Drawn on twos.
		_clock += delta
		var step := int(_clock * 30.0)
		if step != _drawn_step:
			_drawn_step = step
			_draw_parts()

	func _colour(t: float) -> Color:
		return colours[0].lerp(colours[1], t * 2.0) if t < 0.5 else colours[1].lerp(colours[2], t * 2.0 - 1.0)

	func _draw_parts() -> void:
		var im: ImmediateMesh = mesh
		im.clear_surfaces()
		if parts.is_empty():
			return
		var cam := get_viewport().get_camera_3d()
		var right := Vector3.RIGHT
		var up := Vector3.UP
		if cam != null:
			right = cam.global_transform.basis.x
			up = cam.global_transform.basis.y
		im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
		for p in parts:
			var t: float = p[2] / p[3]
			var size: float = p[4] * ((0.5 + 0.7 * t) if grow else (1.0 - 0.85 * t))
			var c := _colour(t)
			var at: Vector3 = p[0]
			if star:
				for k in 10:
					var a0: float = p[5] + TAU * k / 10.0
					var a1: float = p[5] + TAU * (k + 1) / 10.0
					var r0 := size * (0.5 if k % 2 == 0 else 0.22)
					var r1 := size * (0.22 if k % 2 == 0 else 0.5)
					for v in [at, at + (right * cos(a0) + up * sin(a0)) * r0, at + (right * cos(a1) + up * sin(a1)) * r1]:
						im.surface_set_color(c)
						im.surface_add_vertex(v)
			else:
				var r := right * size * 0.5
				var u := up * size * 0.5
				var corners := [at - r - u, at + r - u, at + r + u, at - r + u]
				var uvs := [Vector2(0, 1), Vector2(1, 1), Vector2(1, 0), Vector2(0, 0)]
				for idx in [0, 1, 2, 0, 2, 3]:
					im.surface_set_color(c)
					im.surface_set_uv(uvs[idx])
					im.surface_add_vertex(corners[idx])
		im.surface_end()


## `glow` 0 for soft things that should never bloom (dust, smoke); sparks, embers and flames keep it.
static func _burst(parent: Node, star: bool, on_top: bool, glow := 1.0) -> Burst:
	var b := Burst.new()
	b.star = star
	b.material_override = _material(not star, on_top, glow)
	parent.add_child(b)
	return b


## A random direction within `spread` degrees of `dir`, in the stage plane mostly (a little depth for `depth`).
static func _aim(dir: Vector3, spread: float, depth := 0.25) -> Vector3:
	var a := deg_to_rad(randf_range(-spread, spread))
	var d := dir.normalized()
	var flat := Vector3(d.x * cos(a) - d.y * sin(a), d.x * sin(a) + d.y * cos(a), 0)
	return (flat + Vector3(0, 0, randf_range(-depth, depth))).normalized()


## Dust kicked up at the feet: soft cream puffs that spread low along the ground (toward `side`, or both ways for 0) and swell as they
## fade. A bigger `force` for a hard landing.
static func dust(parent: Node, at: Vector3, side: float, amount := 4, force := 1.0) -> void:
	var b := _burst(parent, false, false, 0.0)
	b.grow = true
	b.damping = 5.0
	b.gravity = Vector3(0, 0.8, 0)
	b.colours = [Color(0.98, 0.95, 0.88, 0.9), Color(0.96, 0.93, 0.86, 0.6), Color(0.95, 0.92, 0.85, 0.0)]
	for k in amount:
		var s := side if side != 0.0 else (-1.0 if k % 2 == 0 else 1.0)
		var vel := _aim(Vector3(s, 0.35, 0), 20.0) * randf_range(1.5, 3.2) * force
		b.add(at + Vector3(randf_range(-0.15, 0.15), 0.15, 0.3), vel, randf_range(0.35, 0.5), randf_range(0.5, 0.8) * force)


## Sparks flying out of a hit: small bright discs that go white to `colour`, slow and shrink; more and faster for a stronger hit
## (`strength` 0..1). Drawn over the fighters.
static func sparks(parent: Node, at: Vector3, colour: Color, strength: float) -> void:
	var s := clampf(strength, 0.0, 1.0)
	var b := _burst(parent, false, true)
	b.damping = 9.0
	b.gravity = Vector3(0, -5, 0)
	b.colours = [Color(1, 1, 0.92, 1), Color(1.0, 0.86, 0.32, 1), Color(colour.r, colour.g, colour.b, 0)]
	for k in 10 + int(14 * s):
		var vel := _aim(Vector3(1, 0, 0), 180.0) * randf_range(6.0, 12.0 + 8.0 * s)
		b.add(at, vel, randf_range(0.25, 0.4 + 0.15 * s), randf_range(0.3, 0.5))


## Embers off a fire hit: small hot discs thrown up and out that rise a little and go from yellow to red as they die.
static func embers(parent: Node, at: Vector3, strength: float) -> void:
	var b := _burst(parent, false, true)
	b.damping = 3.0
	b.gravity = Vector3(0, 3.0, 0)
	b.colours = [Color(1.0, 0.92, 0.5, 1), Color(1.0, 0.45, 0.1, 1), Color(0.7, 0.08, 0.05, 0)]
	for k in 10 + int(10 * strength):
		b.add(at, _aim(Vector3(0, 1, 0), 80.0) * randf_range(3.0, 8.0), randf_range(0.35, 0.6), randf_range(0.25, 0.45))


## A knock-out: sparks and confetti stars in the player's colour thrown along `direction` (back toward the stage), and a cloud of smoke.
static func knock_out(parent: Node, at: Vector3, colour: Color, direction: Vector2) -> void:
	var dir := Vector3(direction.x, direction.y, 0)
	var sp := _burst(parent, false, true)
	sp.damping = 4.0
	sp.colours = [Color(1, 1, 1, 1), Color(colour.r, colour.g, colour.b, 1), Color(colour.r, colour.g, colour.b, 0)]
	for k in 40:
		sp.add(at, _aim(dir, 40.0) * randf_range(10.0, 26.0), randf_range(0.5, 0.9), randf_range(0.7, 1.3))
	var palette := [colour, colour.lightened(0.35), Color(1.0, 0.85, 0.3), Color(1, 1, 1)]
	for k in 4:
		var stars := _burst(parent, true, true)
		stars.damping = 2.5
		stars.gravity = Vector3(0, -6, 0)
		var c: Color = palette[k]
		stars.colours = [c, c, Color(c.r, c.g, c.b, 0)]
		for n in 6:
			stars.add(at, _aim(dir, 55.0) * randf_range(7.0, 16.0), randf_range(1.0, 1.6), randf_range(0.9, 1.4))
	var smoke := _burst(parent, false, false, 0.0)
	smoke.grow = true
	smoke.damping = 3.0
	smoke.colours = [Color(1, 1, 1, 0.85), Color(0.95, 0.93, 0.98, 0.5), Color(0.95, 0.93, 0.98, 0)]
	for k in 14:
		smoke.add(at, _aim(dir, 70.0) * randf_range(2.0, 7.0), randf_range(0.8, 1.1), randf_range(2.4, 3.6))


## The looping flame for the brawler's rushing specials: licks rising off the body, yellow at the core to red at the tips. Add it under
## the fighter (it follows `source`) and switch `emitting` on and off.
static func flame(source: Node3D) -> Burst:
	var b := Burst.new()
	b.material_override = _material(true, false)
	b.looping = true
	b.source = source
	b.spawn_per_second = 90.0
	b.gravity = Vector3(0, 5, 0)
	b.damping = 2.0
	b.colours = [Color(1.0, 0.72, 0.15, 1.0), Color(0.95, 0.3, 0.05, 0.95), Color(0.75, 0.08, 0.05, 0.0)]
	b.spawn = func(at: Vector3) -> Array:
		var off := Vector3(randf_range(-0.7, 0.7), randf_range(0.2, 1.8), 0.35)
		return [at + off, Vector3(randf_range(-0.6, 0.6), randf_range(1.0, 3.0), 0), randf_range(0.3, 0.5), randf_range(1.0, 1.6)]
	return b


static func release() -> void:
	_soft = null
	FxMaterial.release()
