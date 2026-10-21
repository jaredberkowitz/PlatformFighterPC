extends RefCounted

const FxMaterial := preload("res://scripts/fx_material.gd")
## Smear ribbons for fists and feet (docs/MOTION_POLISH_PLAN.md phase 1 item 4): a strike with no authored swing arc (`FighterView.ARCS`)
## leaves a short drawn ribbon along where the striking hand or foot went over its last few frames, in the move's trail colour, blunt and
## widest at the limb and pointed at the tail. The fighter keeps the points (`FighterView._update_smear`); this draws them into the swing
## arc's mesh with the same colours and plane, so it is one system with the arcs.

## How many recent tip positions the ribbon runs through, how thick it is at the limb (world units), and how many pieces each step between
## two points is smoothed into.
const POINTS := 5
## How many frames before the hit the ribbon starts (from the wind-up, so it shows the whole throw).
const LEAD := 4.0
const WIDTH := 0.32
const SUBDIVIDE := 4


## Appends the ribbon through `points` (fighter-local, oldest first, already in the drawing plane) to `im` as one surface. `q` is how far
## it has faded (0 to 1): fading thins it and eats it from the tail. `to_world` turns a local point into the world (the mesh is top level).
static func draw(im: ImmediateMesh, points: Array, q: float, main: Color, core: Color, to_world: Callable) -> void:
	if points.size() < 2 or q >= 1.0:
		return
	var path := _smooth(points)
	var from := int(floor(q * 0.7 * float(path.size() - 1)))
	path = path.slice(from)
	if path.size() < 2:
		return
	var alpha := 1.0 - q
	var width := WIDTH * (1.0 - 0.5 * q)
	# Painted as the swing arcs and the knock-out blast are: layers of solid paint back to front, an ink silhouette, the body in a deeper
	# version of the trail colour, a lit stripe along it and a white-hot core near the limb; blunt and rounded at the limb, pointed at the
	# tail. Solid, so it reads on any background and over a fighter of the same colour.
	var body := Color.from_hsv(main.h, minf(1.0, main.s * 1.35), main.v * 0.82)
	var lit := body.lerp(Color.WHITE, 0.35)
	var ink := Color(0.1, 0.07, 0.12)
	if FxMaterial.paper:
		# Paper: the pale cut edge where the ink was, the core cream.
		ink = Color(1.0, 0.97, 0.9)
		core = core.lerp(Color(1.0, 0.95, 0.82), 0.5)
	var ink_w := 0.05
	# Bands across the ribbon side by side (not stacked, so the see-through trail does not show its ink through the body), each [from, to]
	# as shares of the half width (+-1 the edges) plus an ink edge beyond them: ink, body, the lit stripe, body, ink. The hot core is laid
	# over the lit stripe near the limb.
	var bands := [[-1.0, -1.0, -ink_w, 0.0, Color(ink, alpha), 0.0], [-1.0, 0.05, 0.0, 0.0, Color(body, alpha), 0.0],
		[0.05, 0.6, 0.0, 0.0, Color(lit, alpha), 0.0], [0.6, 1.0, 0.0, 0.0, Color(body, alpha), 0.0],
		[1.0, 1.0, 0.0, ink_w, Color(ink, alpha), 0.0], [0.2, 0.45, 0.0, 0.0, Color(core, alpha), 0.5]]
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
	var last := Vector3.ZERO
	var last_side := Vector3.ZERO
	var last_half := 0.0
	for band in bands:
		var prev_a := Vector3.ZERO
		var prev_b := Vector3.ZERO
		var started := false
		for i in path.size():
			var u := float(i) / float(path.size() - 1)
			if u < float(band[5]):
				continue
			var p: Vector3 = path[i]
			var ahead: Vector3 = path[mini(i + 1, path.size() - 1)] - path[maxi(i - 1, 0)]
			var side := Vector3(-ahead.y, ahead.x, 0.0).normalized()
			# Pointed at the tail, full at the limb.
			var half := 0.5 * width * pow(u, 0.6)
			var a: Vector3 = to_world.call(p + side * (half * float(band[0]) + float(band[2])))
			var b: Vector3 = to_world.call(p + side * (half * float(band[1]) + float(band[3])))
			if started:
				for v in [prev_a, prev_b, b, prev_a, b, a]:
					im.surface_set_color(band[4])
					im.surface_add_vertex(v)
			prev_a = a
			prev_b = b
			started = true
			last = p
			last_side = side
			last_half = half
	# A rounded head at the limb: the body as a half disc, and its ink as a ring round it.
	if last_half > 0.0:
		var fwd := Vector3(last_side.y, -last_side.x, 0.0)
		var seg := 8
		for k in seg:
			var a0 := PI * float(k) / seg
			var a1 := PI * float(k + 1) / seg
			var d0 := last_side * cos(a0) + fwd * sin(a0)
			var d1 := last_side * cos(a1) + fwd * sin(a1)
			for v in [last, last + d0 * last_half, last + d1 * last_half]:
				im.surface_set_color(Color(body, alpha))
				im.surface_add_vertex(to_world.call(v))
			var o0 := last + d0 * (last_half + ink_w)
			var o1 := last + d1 * (last_half + ink_w)
			var i0 := last + d0 * last_half
			var i1 := last + d1 * last_half
			for v in [i0, o0, o1, i0, o1, i1]:
				im.surface_set_color(Color(ink, alpha))
				im.surface_add_vertex(to_world.call(v))
	im.surface_end()


## The points joined by a Catmull-Rom curve, `SUBDIVIDE` pieces between each two.
static func _smooth(points: Array) -> Array:
	var out: Array = []
	var n := points.size()
	for i in n - 1:
		var p0: Vector3 = points[maxi(i - 1, 0)]
		var p1: Vector3 = points[i]
		var p2: Vector3 = points[i + 1]
		var p3: Vector3 = points[mini(i + 2, n - 1)]
		for k in SUBDIVIDE:
			var t := float(k) / SUBDIVIDE
			var t2 := t * t
			var t3 := t2 * t
			out.append(0.5 * ((2.0 * p1) + (-p0 + p2) * t + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2 + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3))
	out.append(points[n - 1])
	return out
