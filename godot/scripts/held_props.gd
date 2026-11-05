extends RefCounted
## Tools a fighter holds for some of its moves (presentation only). The Settler (the Pockets class) swings an umbrella, digs and plants
## with a shovel, fires a slingshot, holds up a firework and turnips, waters with a can, fells its tree with an axe and floats on
## balloons; until the props pass of 2026-11-04 every one of those moves was played with empty hands. The tools are
## art/blender/make_tools.py's (models/tools), coloured by their parts' names as the match props are (match_props.gd `paint`), and hung
## on the hand bone as the modelled weapons are (fighter_view.gd `_mount_weapon_models`): +X out from the fist along the forearm, +Y the
## back of the hand. They are built the first time a move needs them, so no other fighter pays for them.

const DIR := "res://models/tools/%s.glb"
## Where a held tool's origin sits on the hand: this far from the wrist along the forearm (the weapons' HAND_WEAPON_FROM_WRIST).
const FROM_WRIST := 0.17

## The tools by class and move: [tool, where]. "weapon" is the hand that holds weapons (the near hand, `_weapon_side`), "back" hangs
## it above the back (the balloons float up whatever the body does).
const TOOLS := {
	10: {
		"ftilt": ["umbrella", "weapon"],
		"dsmash": ["shovel", "weapon"],
		"fair": ["slingshot", "weapon"],
		"bair": ["slingshot", "weapon"],
		"usmash": ["firework", "weapon"],
		"uair": ["turnips", "weapon"],
		"dair": ["turnips", "weapon"],
		"extra move 0": ["turnips", "weapon"],
		"extra move 1": ["turnips", "weapon"],
		"extra move 2": ["turnips", "weapon"],
		"extra move 3": ["turnips", "weapon"],
		"up special": ["balloons", "back"],
		"extra move 8": ["balloons", "back"],
		"down special": ["shovel", "weapon"],
		"extra move 10": ["shovel", "weapon"],
		"extra move 11": ["watering_can", "weapon"],
		"extra move 12": ["axe", "weapon"],
	},
}
## The balloons above the back: this far up from the spine and behind the body (away from the way it faces), a little behind in depth.
const BALLOON_UP := 0.8
const BALLOON_BEHIND := 0.32
const BALLOON_DEPTH := -0.2

static var _packed := {}


## One fighter's held tools: a holder per hand (built on first use) and each tool made once per hand.
class Held:
	var holders := {}
	var frames := {}
	var tools := {}
	var shown: Node3D = null


static func _load(view: Node, name: String) -> Node3D:
	if not _packed.has(name):
		var path := DIR % name
		_packed[name] = load(path) if ResourceLoader.exists(path) else null
	var n: Node = null
	if _packed[name] is PackedScene:
		n = (_packed[name] as PackedScene).instantiate()
	else:
		n = view._model_scene(DIR % name)
	if n == null or not (n is Node3D):
		if n != null:
			n.free()
		return null
	load("res://scripts/match_props.gd").paint(n, name)
	return n


## The holder on hand `side` ("L"/"R"), and the frame a tool takes in it (as the weapons'), built the first time.
static func _holder(h: Held, skeleton: Skeleton3D, side: String) -> BoneAttachment3D:
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


## Shows the tool the fighter's move holds (or none): `cls` its class, `move` its move's name in an attack, `side` the weapon hand,
## `facing` +1 right. `view` is the fighter's view (for loading models and the outline's object id); `skeleton` its rig.
static func update(h: Held, view: Node3D, skeleton: Skeleton3D, cls: int, move: String, attacking: bool, side: String,
		facing: float) -> void:
	var want: Array = TOOLS.get(cls, {}).get(move, []) if attacking else []
	var n: Node3D = null
	if not want.is_empty() and skeleton != null:
		var tool: String = want[0]
		var where: String = want[1]
		var key := "%s|%s" % [tool, side if where == "weapon" else "back"]
		if not h.tools.has(key):
			var made := _load(view, tool)
			if made != null:
				if where == "back":
					# Floats free of the body's turns: placed in the world each frame, above the spine.
					made.top_level = true
					skeleton.add_child(made)
				else:
					var holder := _holder(h, skeleton, side)
					if holder == null:
						made.free()
						made = null
					else:
						made.transform = h.frames[side]
						holder.add_child(made)
				if made != null:
					made.visible = false
					if view.has_method("_as_tool"):
						view._as_tool(made)
			h.tools[key] = made
		n = h.tools[key]
		if n != null and where == "back":
			var spine := skeleton.find_bone("spine")
			var at: Vector3 = (skeleton.global_transform * skeleton.get_bone_global_pose(spine)).origin if spine >= 0 else skeleton.global_position
			n.global_transform = Transform3D(Basis.IDENTITY, at + Vector3(-facing * BALLOON_BEHIND, BALLOON_UP, BALLOON_DEPTH))
	if n != h.shown:
		if h.shown != null and is_instance_valid(h.shown):
			h.shown.visible = false
		if n != null:
			n.visible = true
		h.shown = n
