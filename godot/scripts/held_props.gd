extends RefCounted
## Tools and kit a fighter holds or wears for some of its moves (presentation only). The Settler (the Pockets class) swings an umbrella,
## digs and plants with a shovel, fires a slingshot, holds up a firework and turnips, heaves a bowling ball, waters with a can, fells its
## tree with an axe and floats on balloons; the Scamp flies on rocket barrels strapped to its back; Embers pumps water from a tank on its
## back; the Wanderer draws a bow in its free hand. Until the props pass of 2026-11-04 every one of those moves was played with empty
## hands. The tools are art/blender/make_tools.py's (models/tools; a name with a folder, "match_props/barrels", is one of the match
## props), coloured by their parts' names as the match props are (match_props.gd `paint`). Hand tools hang on the hand bone as the
## modelled weapons do (fighter_view.gd `_mount_weapon_models`): +X out from the fist along the forearm, +Y the back of the hand. They
## are built the first time a move needs them, so no other fighter pays for them.

## Where a held tool's origin sits on the hand: this far from the wrist along the forearm (the weapons' HAND_WEAPON_FROM_WRIST).
const FROM_WRIST := 0.17

## The kit by class and move: a list of [tool, where] or [tool, where, until]. Where: "weapon" is the hand that holds weapons (the near
## hand, `_weapon_side`), "free" the other hand, "worn" strapped to the back (on the spine bone, so it turns and leans with the body),
## "back" floating above the back (the balloons rise whatever the body does). `until` hides it from that frame of the move on (the
## bowling ball leaves the hands as the shot).
const TOOLS := {
	3: {
		"down special": [["water_tank", "worn"]],
		"extra move 1": [["water_tank", "worn"]],
		"extra move 2": [["water_tank", "worn"]],
	},
	9: {
		"neutral special": [["bow", "free"]],
		"extra move 4": [["bow", "free"]],
		"extra move 9": [["bow", "free"]],
		"extra move 10": [["bow", "free"]],
	},
	10: {
		"ftilt": [["umbrella", "weapon"]],
		"fsmash": [["match_props/bowling_ball", "weapon", 23]],
		"dsmash": [["shovel", "weapon"]],
		"fair": [["slingshot", "weapon"]],
		"bair": [["slingshot", "weapon"]],
		"usmash": [["firework", "weapon"]],
		"uair": [["turnips", "weapon"]],
		"dair": [["turnips", "weapon"]],
		"extra move 0": [["turnips", "weapon"]],
		"extra move 1": [["turnips", "weapon"]],
		"extra move 2": [["turnips", "weapon"]],
		"extra move 3": [["turnips", "weapon"]],
		"up special": [["balloons", "back"]],
		"extra move 8": [["balloons", "back"]],
		"down special": [["shovel", "weapon"]],
		"extra move 10": [["shovel", "weapon"]],
		"extra move 11": [["watering_can", "weapon"]],
		"extra move 12": [["axe", "weapon"]],
	},
	11: {
		"up special": [["match_props/barrels", "worn"]],
		"extra move 5": [["match_props/barrels", "worn"]],
		"extra move 6": [["match_props/barrels", "worn"]],
	},
}
## Where a held tool sits in the fist beyond the weapons' frame (a ball held in front of the fist, not through it), by tool.
const HAND_OFFSET := {"match_props/bowling_ball": Vector3(0.36, -0.05, 0.0)}
## Where worn kit sits, in the body's rest space from the spine (the body faces +Z at rest): behind the back and a little up.
const WORN_AT := {"match_props/barrels": Vector3(0.0, 0.12, -0.42), "water_tank": Vector3(0.0, 0.1, -0.36)}
## A turn about the body's up for worn kit that is not built facing the body's front (Blender -Y, the glTF +Z, is the body's front at
## rest: the water tank's nozzle already reaches forward over the shoulder, and the barrels are the same from either side).
const WORN_TURN := {}
## The balloons above the back: this far up from the spine and behind the body (away from the way it faces), a little behind in depth.
const BALLOON_UP := 0.8
const BALLOON_BEHIND := 0.32
const BALLOON_DEPTH := -0.2

static var _packed := {}


## One fighter's held kit: the holders (a hand's or the spine's, built on first use) and each tool made once per place.
class Held:
	var holders := {}
	var frames := {}
	var tools := {}
	var shown: Array = []


static func _path(name: String) -> String:
	return "res://models/%s.glb" % name if name.contains("/") else "res://models/tools/%s.glb" % name


static func _load(view: Node, name: String) -> Node3D:
	if not _packed.has(name):
		var path := _path(name)
		_packed[name] = load(path) if ResourceLoader.exists(path) else null
	var n: Node = null
	if _packed[name] is PackedScene:
		n = (_packed[name] as PackedScene).instantiate()
	else:
		n = view._model_scene(_path(name))
	if n == null or not (n is Node3D):
		if n != null:
			n.free()
		return null
	load("res://scripts/match_props.gd").paint(n, name.get_file())
	return n


## The holder on hand `side` ("L"/"R"), and the frame a tool takes in it (as the weapons'), built the first time.
static func _hand_holder(h: Held, skeleton: Skeleton3D, side: String) -> BoneAttachment3D:
	if h.holders.has(side):
		return h.holders[side]
	var ih := skeleton.find_bone("hand." + side)
	var il := skeleton.find_bone("armL." + side)
	if ih < 0 or il < 0:
		h.holders[side] = null
		return null
	var hand_rest := skeleton.get_bone_global_rest(ih)
	var fore := (hand_rest.origin - skeleton.get_bone_global_rest(il).origin).normalized()
	var out := Vector3(-1.0 if side == "L" else 1.0, 0.0, 0.0)
	var inv := hand_rest.basis.orthonormalized().inverse()
	var x := (inv * fore).normalized()
	var y := inv * out
	y = (y - x * y.dot(x)).normalized()
	h.frames[side] = Transform3D(Basis(x, y, x.cross(y)), x * FROM_WRIST)
	var holder := BoneAttachment3D.new()
	holder.bone_name = "hand." + side
	skeleton.add_child(holder)
	h.holders[side] = holder
	return holder


## The holder on the spine, and the bone's rest (worn kit is placed in the body's rest space and carried by the bone from there).
static func _spine_holder(h: Held, skeleton: Skeleton3D) -> BoneAttachment3D:
	if h.holders.has("spine"):
		return h.holders["spine"]
	var ib := skeleton.find_bone("spine")
	if ib < 0:
		h.holders["spine"] = null
		return null
	h.frames["spine"] = skeleton.get_bone_global_rest(ib)
	var holder := BoneAttachment3D.new()
	holder.bone_name = "spine"
	skeleton.add_child(holder)
	h.holders["spine"] = holder
	return holder


## Builds tool `tool` for place `where` (`side` the hand for hand tools), or null.
static func _make(h: Held, view: Node3D, skeleton: Skeleton3D, tool: String, where: String, side: String) -> Node3D:
	var made := _load(view, tool)
	if made == null:
		return null
	match where:
		"back":
			# Floats free of the body's turns: placed in the world each frame, above the spine.
			made.top_level = true
			skeleton.add_child(made)
		"worn":
			var holder := _spine_holder(h, skeleton)
			if holder == null:
				made.free()
				return null
			var rest: Transform3D = h.frames["spine"]
			var at: Vector3 = rest.origin + (WORN_AT.get(tool, Vector3(0.0, 0.1, -0.35)) as Vector3)
			var place := Transform3D(Basis(Vector3.UP, float(WORN_TURN.get(tool, 0.0))), at)
			made.transform = rest.affine_inverse() * place
			holder.add_child(made)
		_:
			var holder := _hand_holder(h, skeleton, side)
			if holder == null:
				made.free()
				return null
			var frame: Transform3D = h.frames[side]
			made.transform = frame * Transform3D(Basis.IDENTITY, HAND_OFFSET.get(tool, Vector3.ZERO))
			holder.add_child(made)
	made.visible = false
	if view.has_method("_as_tool"):
		view._as_tool(made)
	return made


## Shows the kit the fighter's move holds or wears (or none): `cls` its class, `move` its move's name and `frame` the move's frame in
## an attack, `side` the weapon hand, `facing` +1 right. `view` is the fighter's view (for loading models and the outline's object id);
## `skeleton` its rig.
static func update(h: Held, view: Node3D, skeleton: Skeleton3D, cls: int, move: String, attacking: bool, side: String,
		facing: float, frame: int = 0) -> void:
	var want: Array = TOOLS.get(cls, {}).get(move, []) if attacking and skeleton != null else []
	var now: Array = []
	for entry in want:
		if entry.size() > 2 and frame >= int(entry[2]):
			continue
		var tool: String = entry[0]
		var where: String = entry[1]
		var hand := side if where == "weapon" else ("R" if side == "L" else "L")
		var key := "%s|%s" % [tool, hand if where in ["weapon", "free"] else where]
		if not h.tools.has(key):
			h.tools[key] = _make(h, view, skeleton, tool, where, hand)
		var n: Node3D = h.tools[key]
		if n == null:
			continue
		if where == "back":
			var spine := skeleton.find_bone("spine")
			var at: Vector3 = (skeleton.global_transform * skeleton.get_bone_global_pose(spine)).origin if spine >= 0 else skeleton.global_position
			n.global_transform = Transform3D(Basis.IDENTITY, at + Vector3(-facing * BALLOON_BEHIND, BALLOON_UP, BALLOON_DEPTH))
		now.append(n)
	for n in h.shown:
		if is_instance_valid(n) and not now.has(n):
			(n as Node3D).visible = false
	for n in now:
		(n as Node3D).visible = true
	h.shown = now
