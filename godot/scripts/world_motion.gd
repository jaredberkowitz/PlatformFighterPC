extends Node3D
## The world-building's life (docs/art/stages/WORLD.md, round 2): everything in a world model that moves is grouped under a pivot named
## `move_<kind>_<n>` (art/blender/make_world.py), and this turns and moves each one about its pivot by its kind, gently and on its own
## clock, as the rest of the scenery moves (the windmill's sails, the clouds, the city's cars). Cosmetic: nothing here reaches the sim.
## `--world-motion=off` holds everything still, `place` keeps the travellers (the train, the bus, the people walking) where they stand.

## The kinds this knows (`kind` is the name's second part; the wheeling and soaring birds add the way they face, `r` or `l`).
const KINDS := ["bob", "perch", "wheel", "soar", "train", "drive", "stroll", "dance", "graze", "sway", "flutter", "smoke", "twinkle",
	"firefly"]
## The ones that travel (held still under `--world-motion=place`).
const TRAVELLERS := ["train", "drive", "stroll"]

## [node, kind, its rest transform, phase, facing (+1 or -1), [child, its rest position, its centre over the pivot] for smoke, the
## bulbs' materials and colours for twinkling]
var movers: Array = []
var t := 0.0
var travel := true


## The kind of a pivot named `move_<kind>_...`, or "" for anything else.
static func kind_of(n: Node) -> String:
	var parts := String(n.name).split("_")
	return parts[1] if parts.size() >= 3 and parts[0] == "move" else ""


func setup(world: Node3D, mode: String) -> void:
	travel = mode != "place"
	var k := 0
	for n: Node in world.find_children("move_*", "Node3D", true, false):
		var kind := kind_of(n)
		if not KINDS.has(kind):
			push_warning("world motion: %s is no kind it knows" % n.name)
			continue
		var node := n as Node3D
		var facing := -1.0 if String(n.name).split("_")[2] == "l" else 1.0
		var parts: Array = []
		for c: Node in node.get_children():
			if c is MeshInstance3D:
				var mi := c as MeshInstance3D
				var mat = mi.material_override
				parts.append([mi, mi.position, mi.transform * mi.get_aabb().get_center(), mat,
					(mat as StandardMaterial3D).albedo_color if mat is StandardMaterial3D else Color.WHITE])
		# (Each its own phase, from its place, so neighbours never move in step.)
		var phase := fposmod(node.position.x * 1.37 + node.position.z * 0.71 + k * 2.3, TAU)
		movers.append([node, kind, node.transform, phase, facing, parts])
		k += 1


func _process(delta: float) -> void:
	t += delta
	for m: Array in movers:
		_move(m)


func _move(m: Array) -> void:
	var n: Node3D = m[0]
	var kind: String = m[1]
	var rest: Transform3D = m[2]
	var ph: float = m[3]
	var facing: float = m[4]
	var off := Vector3.ZERO
	var rot := Vector3.ZERO
	var flip := 1.0
	match kind:
		"bob":
			# On the swell: up and down, a slow roll and pitch.
			off.y = 0.22 * sin(t * 1.1 + ph)
			rot.z = deg_to_rad(3.0) * sin(t * 0.9 + ph + 0.6)
			rot.x = deg_to_rad(1.8) * sin(t * 0.7 + ph)
		"perch":
			# Now and then a little hop, and it turns to look the other way.
			off.y = 0.18 * pow(maxf(0.0, sin(t * 1.3 + ph)), 16.0)
			flip = 1.0 if sin(t * 0.31 + ph) > -0.3 else -1.0
		"wheel", "soar":
			# Slow loops on the wind, banking, turned to face the way it flies (a cut-out turned over at each end).
			var big := kind == "soar"
			var w := (0.22 if big else 0.3)
			var rx := 11.0 if big else 6.0
			var ry := 2.4 if big else 1.4
			var a := t * w + ph
			off = Vector3(rx * (cos(a) - cos(ph)), ry * (sin(2.0 * a) - sin(2.0 * ph)) * 0.5, 0.0)
			var vx := -sin(a)
			flip = 1.0 if vx * facing >= 0.0 else -1.0
			rot.z = deg_to_rad(10.0) * cos(2.0 * a) * signf(vx)
		"train":
			if travel:
				# Along the trestle and round again far past the frame's ends.
				off.x = fposmod(t * 3.2 + 110.0, 220.0) - 110.0
			off.y = 0.03 * sin(t * 9.0)
		"drive":
			if travel:
				off.x = fposmod(-t * 4.5 + 110.0, 220.0) - 110.0
			off.y = 0.03 * sin(t * 11.0)
		"stroll":
			# A paper standee walking to and fro: a waddle and a bounce in the step.
			if travel:
				off.x = 2.5 * (sin(t * 0.14 + ph) - sin(ph))
				flip = 1.0 if cos(t * 0.14 + ph) >= 0.0 else -1.0
				rot.z = deg_to_rad(4.0) * sin(t * 6.0 + ph)
				off.y = 0.06 * absf(sin(t * 6.0 + ph))
		"dance":
			off.y = 0.12 * absf(sin(t * 4.2 + ph))
			rot.z = deg_to_rad(6.0) * sin(t * 4.2 + ph)
		"graze":
			# Head down to the grass for a while, then up to look round (the head is on its +x side).
			var g := clampf(sin(t * 0.42 + ph) * 3.0 - 1.2, 0.0, 1.0)
			rot.z = -deg_to_rad(9.0) * g * g * (3.0 - 2.0 * g)
		"sway":
			rot.z = deg_to_rad(2.5) * sin(t * 0.8 + ph)
		"flutter":
			# Hung by its top: toward and away from the wind, and side to side.
			rot.x = deg_to_rad(14.0) * sin(t * 2.2 + ph)
			rot.z = deg_to_rad(6.0) * sin(t * 1.7 + ph * 1.3)
		"smoke":
			# Each puff drifts with the wind, more the higher it is, the curl rising up the column.
			for p: Array in m[5]:
				var h := maxf(0.0, (p[2] as Vector3).y)
				(p[0] as Node3D).position = (p[1] as Vector3) + Vector3(0.16 * h * sin(t * 0.8 - h * 0.6 + ph), 0.05 * h * sin(t * 0.5 - h * 0.4 + ph), 0.0)
		"twinkle":
			for p: Array in m[5]:
				if p[3] is StandardMaterial3D:
					var k := 0.72 + 0.28 * sin(t * 2.6 + ph * 3.0)
					(p[3] as StandardMaterial3D).albedo_color = (p[4] as Color) * Color(k, k, k)
		"firefly":
			# Wandering, and blinking on and off.
			off = Vector3(1.4 * sin(t * 0.45 + ph), 0.9 * sin(t * 0.63 + ph * 1.7), 0.7 * sin(t * 0.37 + ph * 0.6))
			var b := pow(maxf(0.0, sin(t * 1.2 + ph * 2.0)), 3.0)
			n.transform = Transform3D(Basis.IDENTITY.scaled(Vector3.ONE * (0.15 + 0.85 * b)), rest.origin + off)
			return
	n.transform = Transform3D(Basis.from_euler(rot).scaled(Vector3(flip, 1.0, 1.0)), Vector3.ZERO) * Transform3D(rest.basis, Vector3.ZERO)
	n.position = rest.origin + off
