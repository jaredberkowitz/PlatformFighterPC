extends RefCounted
## The things fighters throw, shoot and leave lying in a match, as modelled props (presentation only: the sim's projectiles and items,
## read through the bridge). Until the props pass of 2026-11-04 every shot without a look of its own was the one generic purple bolt
## (the Scamp's popgun shot, the Wanderer's arrows and boomerang, the Settler's flowerpot, bowling ball, pellets and rocket...), and four
## item kinds were not drawn at all (the Scamp's banana peel, the Settler's sapling, tree and wood chip). The models are
## art/blender/make_match_props.py's, in models/match_props/, coloured here by their parts' names (`PALETTE`) as the weapons are.
## main.gd keeps one `Slot` per projectile slot and per item slot; each slot builds a prop the first time it needs it and keeps it.

const FighterView := preload("res://scripts/fighter_view.gd")

const DIR := "res://models/match_props/%s.glb"

## Part colours by the first word of a part's name: [colour, light response ("" for the plain cel look)]. The popgun's toy red and wood
## are the gun's own (fighter_view.gd `_paint_popgun`), so its cork reads as the gun's.
const PALETTE := {
	"cork": [Color(0.88, 0.72, 0.5), ""],
	"band": [Color(0.84, 0.2, 0.16), ""],
	"wood": [Color(0.66, 0.45, 0.28), ""],
	"bark": [Color(0.48, 0.32, 0.22), ""],
	"dark": [Color(0.24, 0.18, 0.18), ""],
	"metal": [Color(0.74, 0.76, 0.84), "metal"],
	"leather": [Color(0.46, 0.28, 0.17), ""],
	"peel": [Color(0.98, 0.8, 0.2), ""],
	"pith": [Color(0.99, 0.94, 0.74), ""],
	"soil": [Color(0.38, 0.26, 0.17), ""],
	"leaf": [Color(0.42, 0.7, 0.32), ""],
	"leafdark": [Color(0.24, 0.5, 0.28), ""],
	"clay": [Color(0.82, 0.45, 0.28), ""],
	"petal": [Color(0.96, 0.46, 0.62), ""],
	"centre": [Color(0.99, 0.84, 0.3), ""],
	"ball": [Color(0.2, 0.25, 0.62), "rubber"],
	"hole": [Color(0.07, 0.06, 0.1), ""],
	"stone": [Color(0.6, 0.55, 0.5), ""],
	"paint": [Color(0.96, 0.91, 0.8), "paint"],
	"glass": [Color(0.55, 0.8, 0.96), "paint"],
	"rim": [Color(0.92, 0.76, 0.3), "gold"],
}

## A prop's own colours where the shared ones do not suit it: [prop][part word] -> colour.
const PROP_COLOURS := {
	"arrow": {"wood": Color(0.78, 0.6, 0.4)},
	"barrels": {"wood": Color(0.72, 0.44, 0.24), "metal": Color(0.92, 0.76, 0.3)},
	"rocket": {"dark": Color(0.3, 0.3, 0.36)},
	"banana_peel": {"dark": Color(0.36, 0.24, 0.14)},
	"boomerang": {"wood": Color(0.74, 0.5, 0.3)},
	# The Settler's tools (held_props.gd).
	"watering_can": {"paint": Color(0.36, 0.64, 0.52), "metal": Color(0.92, 0.76, 0.3)},
	"turnips": {"pith": Color(0.97, 0.96, 0.92), "petal": Color(0.66, 0.32, 0.66)},
	"balloons": {"glass": Color(0.32, 0.56, 0.95)},
	"shovel": {"wood": Color(0.72, 0.5, 0.3)},
	"axe": {"wood": Color(0.72, 0.5, 0.3)},
	"bow": {"wood": Color(0.62, 0.4, 0.24), "pith": Color(0.95, 0.93, 0.86)},
	"water_tank": {"glass": Color(0.3, 0.56, 0.92)},
}

## The modelled shots by the move that fires them: Vector2i(class, move id) -> prop. An empty name hides the shot (a hit with no body of
## its own: the tree's growth is the tree item growing). Shots not listed keep main.gd's own looks or, for now, the bolt.
const SHOTS := {
	Vector2i(11, 31): "cork",  # the Scamp's popgun shot (extra move 3)
	Vector2i(11, 36): "barrels",  # the Scamp's rocket barrels, knocked off (extra move 8)
	Vector2i(9, 14): "boomerang",  # the Wanderer's boomerang, out (side special)
	Vector2i(9, 33): "boomerang",  # and its aerial throw (extra move 5)
	Vector2i(9, 32): "arrow",  # the Wanderer's arrows, by charge (extra moves 4, 9 and 10)
	Vector2i(9, 37): "arrow",
	Vector2i(9, 39): "arrow",
	Vector2i(10, 4): "flowerpot",  # the Settler's dash attack
	Vector2i(10, 5): "bowling_ball",  # its forward smash
	Vector2i(10, 9): "pellet",  # its forward and back airs' slingshot
	Vector2i(10, 10): "pellet",
	Vector2i(10, 14): "rocket",  # its side special
	Vector2i(10, 43): "",  # the tree's growth (extra move 14): drawn as the tree item growing
	Vector2i(10, 44): "tree",  # the falling tree (extra move 15)
}

## How each shot moves, beyond following its position: "along" points it the way it flies, "spin" turns it about the camera's axis
## (`SPIN` radians a frame, the way it flies), "roll" rolls it along the floor by its speed, "topple" is the felled tree.
const MOTION := {"cork": "along", "arrow": "along", "rocket": "along", "boomerang": "spin", "barrels": "spin", "flowerpot": "spin",
	"pellet": "spin", "bowling_ball": "roll", "tree": "topple"}
const SPIN := {"boomerang": 0.62, "barrels": 0.22, "flowerpot": 0.12, "pellet": 0.3}
## Spinning shots that sit upright once they come to rest (`projectile_rest` above 0).
const UPRIGHT_AT_REST := {"flowerpot": true}
## A shot at rest on the floor has its middle on the floor line (the sim lands a shot by its point; its hit is centred there), so the
## round ones are lifted by their half height to sit on it, easing up over a few frames (`REST_LIFT_RATE` a frame) as a little settle.
## (An arrow is left as it is: it lodges point first, half in the ground.)
const REST_LIFT := {"bowling_ball": 0.42, "flowerpot": 0.34, "cork": 0.15, "pellet": 0.1}
const REST_LIFT_RATE := 0.14

## The item kinds drawn as props (main.gd draws the bomb itself; the arrow item wears the arrow model). 5 sapling, 6 tree, 8 wood
## chip (the Settler), 9 banana peel (the Scamp).
const ITEMS := {5: "sapling", 6: "tree", 8: "wood_chip", 9: "banana_peel"}

## The tree grows from the sapling over the growth's first frames (its hit reaches full height on 19) with a little overshoot.
const TREE_GROW_FRAMES := 18
## The felled tree topples over the frames before its hit starts (13), pivoting on its stump, and comes to rest on its crown: about 68
## degrees over, where its round crown (about 1.3 across the trunk, 2.6 up it) meets the floor.
const TREE_FALL_FRAMES := 13
const TREE_REST_TILT := 1.19
## Where the falling tree's shot starts above the stump (content: extra move 15's projectile y).
const TREE_FALL_FROM_Y := 2.1875
## A thrown banana peel turns this many radians a frame in the air.
const PEEL_SPIN := 0.35

static var _packed := {}
## PF_PROP_TRACE=1 prints a line per shot drawn as a prop ("PROP <frame> <name> <x> <y>"), for finding a prop's frames in a capture.
static var _trace := -1


static func tracing() -> bool:
	if _trace < 0:
		_trace = 1 if OS.has_environment("PF_PROP_TRACE") else 0
	return _trace == 1


## A coloured copy of prop `name`, or null when its model is missing. (The imported scene is kept as a resource and instanced: a copy
## of a node kept outside the tree would be left over at quit, which crashes the release build.)
static func make(name: String) -> Node3D:
	var n: Node = null
	if not _packed.has(name):
		var path := DIR % name
		_packed[name] = load(path) if ResourceLoader.exists(path) else null
	var packed = _packed[name]
	if packed is PackedScene:
		n = packed.instantiate()
	else:
		n = FighterView._model_scene(DIR % name)
	if n == null or not (n is Node3D):
		if n != null:
			n.free()
		return null
	paint(n, name)
	return n


## Colours a prop's parts by the first word of their names.
static func paint(root: Node, name: String) -> void:
	var own: Dictionary = PROP_COLOURS.get(name, {})
	for m in root.find_children("*", "MeshInstance3D", true, false):
		var mi := m as MeshInstance3D
		var word := String(mi.name).get_slice("_", 0)
		var entry: Array = PALETTE.get(word, PALETTE["wood"])
		var colour: Color = own.get(word, entry[0])
		var mat := FighterView.toon(colour)
		if String(entry[1]) != "":
			mat = FighterView.respond(mat, String(entry[1]))
		mi.material_override = mat
		mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_ON


## The prop a shot from `from` ([class, move]) is drawn as: a name, "" (drawn as nothing), or null when it has no prop.
static func shot(from: Array) -> Variant:
	if from.size() < 2:
		return null
	var key := Vector2i(int(from[0]), int(from[1]))
	return SHOTS[key] if SHOTS.has(key) else null


## One projectile or item slot's props: built on first use under `root` and kept, only the one in use shown.
class Slot:
	var root: Node3D
	var props := {}
	var showing := ""
	## What is shown and since when (the sim frame), where it started and which way it flew (the felled tree's stump), and its turn.
	var key := ""
	var since := 0
	var start_at := Vector3.ZERO
	var start_dir := 1.0
	var lie_at := Vector3.ZERO
	var angle := 0.0
	var lift := 0.0
	var last_frame := -1
	## Called with each prop as it is built (main.gd marks it as a thing in play for the outline).
	var on_new: Callable

	func _init(parent: Node3D, made: Callable = Callable()) -> void:
		root = Node3D.new()
		root.name = "props"
		parent.add_child(root)
		on_new = made


## Shows prop `name` in `slot` (building it the first time), hiding the others; "" shows nothing. Returns the prop shown, if any.
static func use(slot: Slot, name: String) -> Node3D:
	if name != slot.showing:
		for p in slot.props:
			if slot.props[p] != null:
				(slot.props[p] as Node3D).visible = false
		slot.showing = name
	if name == "":
		return null
	if not slot.props.has(name):
		var made := make(name)
		if made != null:
			slot.root.add_child(made)
			if slot.on_new.is_valid():
				slot.on_new.call(made)
		slot.props[name] = made
	var n: Node3D = slot.props[name]
	if n != null:
		n.visible = true
	return n


static func hide_slot(slot: Slot) -> void:
	use(slot, "")
	slot.key = ""
	slot.last_frame = -1


## Draws projectile slot `slot` as prop `name` at `at` flying `vel` (`dir` +1 right), at the sim's `frame`. `scale_pct` is the sim's
## size for charged shots (100 = as built); `rest` the bridge's `projectile_rest` (a lodged shot stops turning).
static func update_shot(slot: Slot, name: String, from: Array, at: Vector3, vel: Vector2, dir: float, frame: int, scale_pct: float,
		rest: int) -> void:
	var n := use(slot, name)
	if n == null:
		return
	var k := "%s|%d|%d" % [name, int(from[0]), int(from[1])]
	if k != slot.key or frame < slot.last_frame:
		slot.key = k
		slot.since = frame
		slot.start_at = at
		slot.start_dir = 1.0 if dir >= 0.0 else -1.0
		slot.lie_at = Vector3.INF
		slot.angle = 0.0 if dir >= 0.0 else PI
		slot.lift = 0.0
		slot.last_frame = frame
	var age := frame - slot.since
	var step := maxi(frame - slot.last_frame, 0)
	slot.last_frame = frame
	var s := clampf(scale_pct / 100.0, 0.5, 1.6) if scale_pct > 0.0 else 1.0
	if tracing():
		print("PROP %d %s %.2f %.2f age %d" % [frame, name, at.x, at.y, age])
	n.scale = Vector3.ONE * s
	var lift_to: float = float(REST_LIFT.get(name, 0.0)) * s if rest != 0 else 0.0
	slot.lift = move_toward(slot.lift, lift_to, REST_LIFT_RATE * float(step)) if step > 0 else slot.lift
	n.position = at + Vector3(0.0, slot.lift, 0.0)
	n.rotation = Vector3.ZERO
	match String(MOTION.get(name, "")):
		"along":
			if rest == 0 and vel.length() > 0.001:
				slot.angle = vel.angle()
			n.rotation.z = slot.angle
		"spin":
			if rest == 0:
				slot.angle -= float(SPIN.get(name, 0.3)) * slot.start_dir * float(step)
			elif UPRIGHT_AT_REST.has(name):
				# (Landed, it sits upright: the flowerpot on its base.)
				slot.angle = 0.0
			n.rotation.z = slot.angle
		"roll":
			# Rolling along the floor: the turn its travel needs on its radius (0.42 as built).
			slot.angle -= vel.x / (0.42 * s) * float(step)
			n.rotation.z = slot.angle
		"topple":
			# The felled tree pivots on its stump (the shot starts TREE_FALL_FROM_Y above it), falling the way the shot flies, faster
			# as it goes, until it rests on its crown (TREE_REST_TILT); then it slides on with the shot, its stump end where it fell.
			var t := clampf(float(age) / TREE_FALL_FRAMES, 0.0, 1.0)
			var stump := slot.start_at - Vector3(0.0, TREE_FALL_FROM_Y, 0.0)
			if age >= TREE_FALL_FRAMES:
				if slot.lie_at == Vector3.INF:
					slot.lie_at = at
				stump.x += at.x - slot.lie_at.x
				stump.y = minf(stump.y, stump.y + at.y - slot.lie_at.y)
			n.position = stump
			n.rotation.z = -t * t * TREE_REST_TILT * slot.start_dir
		_:
			pass


## Draws item slot `slot` (kind `kind`) on its item node: the tree grows when a sapling becomes one; a thrown peel turns in the air.
static func update_item(slot: Slot, kind: int, frame: int, flight: int, way: float, held: bool) -> void:
	var name: String = ITEMS.get(kind, "")
	var n := use(slot, name)
	if n == null:
		slot.key = ""
		return
	if name != slot.key:
		slot.key = name
		slot.since = frame
	var age := frame - slot.since
	if tracing():
		var holder := slot.root.get_parent() as Node3D
		var at: Vector3 = holder.global_position if holder != null else Vector3.ZERO
		print("ITEM %d %s %.2f %.2f age %d" % [frame, name, at.x, at.y, age])
	n.rotation = Vector3.ZERO
	n.scale = Vector3.ONE
	if name == "tree":
		var t := clampf(float(age) / TREE_GROW_FRAMES, 0.0, 1.0)
		# Up from a sapling's size with a little overshoot past full, settling back.
		var grow := lerpf(0.12, 1.0, 1.0 - pow(1.0 - t, 3.0)) + 0.06 * sin(t * PI) * t
		n.scale = Vector3(lerpf(0.6, 1.0, t) * grow, grow, grow)
	elif name == "banana_peel" and not held and flight > 0:
		n.rotation.z = -PEEL_SPIN * float(flight) * way
