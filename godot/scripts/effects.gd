extends RefCounted
## Short-lived hit and movement effects (presentation only): a shockwave ring and a burst of streaks when a hit lands, and a ring under the
## feet on a jump and a midair jump. Each effect is a node that animates itself for its short life and then frees itself.


static func _material(on_top: bool) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	m.vertex_color_use_as_albedo = true
	m.cull_mode = BaseMaterial3D.CULL_DISABLED
	m.no_depth_test = on_top
	m.render_priority = 6 if on_top else 0
	return m


## A ring that expands and thins out as it fades, drawn in its own XY plane (facing the camera unless the node is turned).
class Ring extends MeshInstance3D:
	var age := 0.0
	var life := 0.3
	var from := 0.3
	var to := 1.6
	var thickness := 0.25
	var colour := Color.WHITE

	func _ready() -> void:
		mesh = ImmediateMesh.new()
		cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF

	func _process(delta: float) -> void:
		age += delta
		var t := age / life
		if t >= 1.0:
			queue_free()
			return
		var eased := 1.0 - pow(1.0 - t, 3.0)
		var r := lerpf(from, to, eased)
		var w := thickness * (1.0 - t)
		var im: ImmediateMesh = mesh
		im.clear_surfaces()
		im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
		var c := Color(colour.r, colour.g, colour.b, colour.a * (1.0 - t * t))
		var n := 40
		for k in n:
			var a0 := TAU * k / n
			var a1 := TAU * (k + 1) / n
			var o0 := Vector3(cos(a0), sin(a0), 0) * r
			var o1 := Vector3(cos(a1), sin(a1), 0) * r
			var i0 := Vector3(cos(a0), sin(a0), 0) * maxf(r - w, 0.0)
			var i1 := Vector3(cos(a1), sin(a1), 0) * maxf(r - w, 0.0)
			for v in [o0, o1, i1, o0, i1, i0]:
				im.surface_set_color(c)
				im.surface_add_vertex(v)
		im.surface_end()


## Thin spikes flying out in every direction, shrinking as they go.
class Streaks extends MeshInstance3D:
	var age := 0.0
	var life := 0.22
	var count := 8
	var reach := 1.6
	var colour := Color.WHITE
	var angles: Array = []

	func _ready() -> void:
		mesh = ImmediateMesh.new()
		cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		for k in count:
			angles.append(TAU * (float(k) + randf_range(-0.3, 0.3)) / count)

	func _process(delta: float) -> void:
		age += delta
		var t := age / life
		if t >= 1.0:
			queue_free()
			return
		var im: ImmediateMesh = mesh
		im.clear_surfaces()
		im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
		var c := Color(colour.r, colour.g, colour.b, colour.a * (1.0 - t))
		for a in angles:
			var d := Vector3(cos(a), sin(a), 0)
			var side := Vector3(-d.y, d.x, 0)
			var near := d * reach * (0.25 + 0.75 * t)
			var far := d * reach * (0.6 + 0.6 * t)
			var w := 0.09 * (1.0 - t)
			for v in [near + side * w, far, near - side * w]:
				im.surface_set_color(c)
				im.surface_add_vertex(v)
		im.surface_end()


## A hit landing at `at`: a white shockwave ring, a ring in `color` inside it, and streaks; bigger for a stronger hit (`strength` 0..1).
static func hit(parent: Node, at: Vector3, color: Color, strength: float) -> void:
	var s := clampf(strength, 0.0, 1.0)
	var ring := Ring.new()
	ring.material_override = _material(true)
	ring.colour = Color(1, 1, 1, 0.95)
	ring.to = 1.1 + 1.6 * s
	ring.thickness = 0.18 + 0.2 * s
	ring.life = 0.22 + 0.12 * s
	ring.position = at
	parent.add_child(ring)
	var inner := Ring.new()
	inner.material_override = _material(true)
	inner.colour = Color(color.r, color.g, color.b, 0.8)
	inner.from = 0.15
	inner.to = 0.8 + 1.1 * s
	inner.thickness = 0.3 + 0.25 * s
	inner.life = 0.3 + 0.12 * s
	inner.position = at + Vector3(0, 0, -0.05)
	parent.add_child(inner)
	var streaks := Streaks.new()
	streaks.material_override = _material(true)
	streaks.colour = Color(1.0, 0.97, 0.8, 1.0)
	streaks.count = 6 + int(6 * s)
	streaks.reach = 1.2 + 1.6 * s
	streaks.life = 0.18 + 0.1 * s
	streaks.position = at
	parent.add_child(streaks)


## A jump: a soft ring spreading over the ground at the feet; `air` makes it the midair jump's ring, a little bigger and brighter.
static func jump_ring(parent: Node, feet: Vector3, air: bool) -> void:
	var ring := Ring.new()
	ring.material_override = _material(false)
	ring.colour = Color(1, 1, 1, 0.85 if air else 0.6)
	ring.from = 0.3
	ring.to = 1.3 if air else 1.1
	ring.thickness = 0.16
	ring.life = 0.3
	ring.position = feet + Vector3(0, 0.05, 0)
	# Laid flat (the ring is drawn in its own XY plane).
	ring.rotation_degrees = Vector3(-90, 0, 0)
	parent.add_child(ring)
