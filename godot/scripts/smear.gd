extends RefCounted
## Smear ribbons for fists and feet (docs/MOTION_POLISH_PLAN.md phase 1 item 4): a strike with no authored swing arc (`FighterView.ARCS`)
## leaves a short drawn ribbon along where the striking hand or foot went over its last few frames, in the move's trail colour, blunt and
## widest at the limb and pointed at the tail. The fighter keeps the points (`FighterView._update_smear`); this draws them into the swing
## arc's mesh with the same colours and plane, so it is one system with the arcs.

## How many recent tip positions the ribbon runs through, how thick it is at the limb (world units), and how many pieces each step between
## two points is smoothed into.
const POINTS := 5
## How many frames before the hit the ribbon starts (from the wind-up, so it shows the whole throw).
const LEAD := 4.0
const WIDTH := 0.26
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
	# Across the ribbon from one edge to the other: clear, the trail colour, the bright core along the middle, the trail colour, clear.
	var across := [-1.0, -0.55, 0.0, 0.55, 1.0]
	var cols := [Color(main, 0.0), Color(main, 0.85 * alpha), Color(core, alpha), Color(main, 0.85 * alpha), Color(main, 0.0)]
	var prev: Array = []
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
	for i in path.size():
		var u := float(i) / float(path.size() - 1)
		var p: Vector3 = path[i]
		var ahead: Vector3 = path[mini(i + 1, path.size() - 1)] - path[maxi(i - 1, 0)]
		var side := Vector3(-ahead.y, ahead.x, 0.0).normalized()
		# Pointed at the tail, full at the limb.
		var half := 0.5 * width * pow(u, 0.6)
		var column: Array = []
		for a in across:
			column.append(to_world.call(p + side * half * float(a)))
		if i > 0:
			for b in across.size() - 1:
				var quad := [[prev[b], cols[b]], [prev[b + 1], cols[b + 1]], [column[b + 1], cols[b + 1]], [column[b], cols[b]]]
				for idx in [0, 1, 2, 0, 2, 3]:
					im.surface_set_color(quad[idx][1])
					im.surface_add_vertex(quad[idx][0])
		prev = column
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
