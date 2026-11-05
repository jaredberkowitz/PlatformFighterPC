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
## Worn kit drawn larger than its model (the match props' size), by tool. (Round 10, the grip pass: the Scamp's rocket barrels sat
## hidden behind the body when it faced the camera to fly, upright; its reference's are as broad as its body and show at both sides
## of its waist. Half as big again, ours do.)
const WORN_SCALE := {"match_props/barrels": 1.5}
## (Round 10, the grip pass) A held tool's length turned to point along a line for part of a move, about the fist, by class, move and
## tool: [where, from frame, to frame, ease frames]. `where` is a direction in forward space (x ahead, y up, in the stage's plane) or
## "tip", toward the move's tip from the fist (`move_tip`: a projectile move's is where its shot starts). Eased in over `ease` frames
## before `from` and out over them after `to`, by the move's frame (a freeze holds it, a seek redraws it). The Settler holds its turnips
## up through the up air's hits, as its reference holds them overhead (they pointed back from the shoulder at the chest); its slingshot
## points along its shots, at where the pellet starts (the forward air's pointed at the floor, the back air's at the camera).
const POINT_UP := [Vector3(-0.2, 1.0, 0.0), 5, 20, 4]
const POINT := {
	10: {
		"uair": {"turnips": POINT_UP},
		"extra move 0": {"turnips": POINT_UP},
		"extra move 1": {"turnips": POINT_UP},
		"fair": {"slingshot": ["tip", 3, 20, 3]},
		"bair": {"slingshot": ["tip", 4, 23, 3]},
	},
}
## (Round 10, the grip pass) Flat tools whose face turns toward the camera by a roll about their length, by tool: the face's normal in
## the tool's own frame. (The mitten fist does not show the hand's roll.) The Settler's spade lay flat on the floor at the down smash's
## dig, edge-on to the camera. A tool pointing at the camera keeps its roll (`FACE_MIN`: how far across the view its length must lie).
const FACE := {"shovel": Vector3(0.0, 0.0, 1.0)}
const FACE_MIN := Vector2(0.25, 0.5)
## The balloons above the back: this far up from the spine and behind the body (away from the way it faces), a little behind in depth.
const BALLOON_UP := 0.8
const BALLOON_BEHIND := 0.32
const BALLOON_DEPTH := -0.2

static var _packed := {}
## PF_R10_OFF=grip shows the tools as the props pass held them.
static var _r10_off := OS.get_environment("PF_R10_OFF").split(",")


static func _off(what: String) -> bool:
	return _r10_off.has(what) or _r10_off.has("all")


## A hand tool's place on hand `side` before any turn: the weapons' frame and the tool's offset in the fist.
static func _base(h: Held, tool: String, side: String) -> Transform3D:
	return (h.frames[side] as Transform3D) * Transform3D(Basis.IDENTITY, HAND_OFFSET.get(tool, Vector3.ZERO))


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
			var grow := float(WORN_SCALE.get(tool, 1.0)) if not _off("grip") else 1.0
			var place := Transform3D(Basis(Vector3.UP, float(WORN_TURN.get(tool, 0.0))).scaled(Vector3.ONE * grow), at)
			made.transform = rest.affine_inverse() * place
			holder.add_child(made)
		_:
			var holder := _hand_holder(h, skeleton, side)
			if holder == null:
				made.free()
				return null
			made.transform = _base(h, tool, side)
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


## Shows one hand tool, `tool`, in hand `side` ("L"/"R") outside any move, or none with "" (round 10: the Settler's umbrella raised in the
## results screen's victory pose, `fighter_view.gd` `_victory_weapon`); hides whatever else was shown. `skeleton` null shows none.
static func hold(h: Held, view: Node3D, skeleton: Skeleton3D, tool: String, side: String) -> void:
	var now: Array = []
	if tool != "" and skeleton != null:
		var key := "%s|%s" % [tool, side]
		if not h.tools.has(key):
			h.tools[key] = _make(h, view, skeleton, tool, "weapon", side)
		if h.tools[key] != null:
			now.append(h.tools[key])
	for n in h.shown:
		if is_instance_valid(n) and not now.has(n):
			(n as Node3D).visible = false
	for n in now:
		(n as Node3D).visible = true
	h.shown = now


## How far a POINT entry holds at frame `f` (0..1).
static func _point_weight(e: Array, f: float) -> float:
	var ease := maxf(1.0, float(e[3]))
	return smoothstep(float(e[1]) - ease, float(e[1]), f) * (1.0 - smoothstep(float(e[2]), float(e[2]) + ease, f))


## Turns the shown hand tools after the arm layers have posed the hands (`POINT`, `FACE`), from their base place in the fist each frame.
## `to_frame` takes the rig's space to the stage frame (the fighter's, unflipped); `s` is the fighter's snapshot. With PF_GRIP_TRACE set,
## prints each shown hand tool's axes in the stage frame (x forward by the facing, y up, z toward the camera):
## "GRIP <class> <move> <frame> <tool> x:.. y:.. z:.. at:..".
static func pose(h: Held, skeleton: Skeleton3D, to_frame: Transform3D, cls: int, s: Dictionary) -> void:
	if skeleton == null:
		return
	var move := String(s.move_name)
	var f := float(s.state_frame)
	var facing := float(s.facing)
	var from_frame := to_frame.affine_inverse()
	for key in h.tools:
		var n: Node3D = h.tools[key]
		var tool := String(key).get_slice("|", 0)
		var side := String(key).get_slice("|", 1)
		if n == null or not n.visible or not (side == "L" or side == "R"):
			continue
		var bone: Transform3D = skeleton.get_bone_global_pose(skeleton.find_bone("hand." + side))
		# The tool in the rig's space: its basis turned about its own origin (the fist) by the entries below.
		var held: Transform3D = bone * _base(h, tool, side)
		var b := held.basis
		if not _off("grip"):
			var e: Array = ((POINT.get(cls, {}) as Dictionary).get(move, {}) as Dictionary).get(tool, [])
			var w := _point_weight(e, f) if not e.is_empty() else 0.0
			if w > 0.0:
				var grip: Vector3 = to_frame * held.origin
				var want := Vector3.ZERO
				if e[0] is String:
					var tip: Vector3 = s.get("move_tip", Vector3.ZERO)
					want = Vector3(tip.x * facing, tip.y, grip.z) - grip
				else:
					var d: Vector3 = e[0]
					want = Vector3(d.x * facing, d.y, d.z)
				var to := (from_frame.basis * want).normalized()
				var along := b.x.normalized()
				if to.length() > 0.5 and along.dot(to) > -0.999:
					b = Basis(Quaternion.IDENTITY.slerp(Quaternion(along, to), w)) * b
			var face: Vector3 = FACE.get(tool, Vector3.ZERO)
			if face != Vector3.ZERO:
				var along := b.x.normalized()
				var cam := (from_frame.basis * Vector3(0.0, 0.0, 1.0)).normalized()
				var want := cam - along * cam.dot(along)
				var k := smoothstep(FACE_MIN.x, FACE_MIN.y, want.length())
				var now := b * face
				now = now - along * now.dot(along)
				if k > 0.0 and now.length() > 0.001:
					want = want.normalized()
					now = now.normalized()
					# (Either side of a flat face does: the shortest roll.)
					if now.dot(want) < 0.0:
						want = -want
					b = Basis(along, now.signed_angle_to(want, along) * k) * b
		n.transform = bone.affine_inverse() * Transform3D(b, held.origin)
		if OS.has_environment("PF_GRIP_TRACE"):
			var g: Transform3D = to_frame * Transform3D(b, held.origin)
			var o := g.basis.orthonormalized()
			print("GRIP %d %s %d %s x:%.2f,%.2f,%.2f y:%.2f,%.2f,%.2f z:%.2f,%.2f,%.2f at:%.2f,%.2f,%.2f" % [cls, move.replace(" ", "_"),
				int(f), tool.get_file(), o.x.x * facing, o.x.y, o.x.z, o.y.x * facing, o.y.y, o.y.z, o.z.x * facing, o.z.y, o.z.z,
				g.origin.x * facing, g.origin.y, g.origin.z])
