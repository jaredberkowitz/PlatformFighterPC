extends Node3D
## Placeholder blob fighter: round body and head, drawn-on face, a hat and shades, in the spirit of
## docs/ART_DIRECTION.md. This only DISPLAYS sim state. Animation never drives the simulation.

const COLORS := [
	Color(1.0, 0.85, 0.2),   # yellow
	Color(0.98, 0.58, 0.18), # orange
	Color(0.25, 0.45, 0.92), # blue
	Color(0.95, 0.5, 0.72),  # pink
]
const SASH := Color(0.36, 0.24, 0.3)
const INK := Color(0.1, 0.07, 0.12)

# Per-state pose: scale (squash and stretch), forward lean in degrees, yaw toward facing in degrees.

var player := 0
var model: Node3D
var shield: MeshInstance3D
var meshes: Array[MeshInstance3D] = []
var yaw := 0.0
var lean := 0.0


static var _mat_cache := {}
static var _mesh_cache := {}
static var _outline_mat: ShaderMaterial


const TOON_SHADER := preload("res://shaders/toon.gdshader")
const FACE_SHADER := preload("res://shaders/face.gdshader")
const OUTLINE_SHADER := preload("res://shaders/outline.gdshader")
const SHIELD_SHADER := preload("res://shaders/shield.gdshader")
const LIMB_SHADER := preload("res://shaders/limb.gdshader")
const SvgArt := preload("res://scripts/svg_art.gd")
const Particles := preload("res://scripts/particles.gd")
const FxMaterial := preload("res://scripts/fx_material.gd")


## The soft cel material (`shaders/toon.gdshader`) in colour `c`, with the ink outline unless `outline` is false. Cached per colour.
## `texture` (fabric, straw...) multiplies the colour, tiled `tile` times.
static func toon(c: Color, outline := true, texture: Texture2D = null, tile := Vector2.ONE, world_tile := 0.0) -> ShaderMaterial:
	var key := "%s%s%s%s%s" % [c.to_html(), outline, texture.get_instance_id() if texture != null else 0, tile, world_tile]
	if _mat_cache.has(key):
		return _mat_cache[key]
	var m := ShaderMaterial.new()
	m.shader = TOON_SHADER
	_mat_cache[key] = m
	m.set_shader_parameter("albedo", c)
	if texture != null:
		m.set_shader_parameter("use_tex", true)
		m.set_shader_parameter("albedo_tex", texture)
		m.set_shader_parameter("tex_scale", tile)
		m.set_shader_parameter("world_tile", world_tile)
	if outline:
		if _outline_mat == null:
			# An ink line that keeps about the same thickness on screen at any zoom (shaders/outline.gdshader).
			_outline_mat = ShaderMaterial.new()
			_outline_mat.shader = OUTLINE_SHADER
			_outline_mat.set_shader_parameter("color", INK)
		m.next_pass = _outline_mat
	return m


## Throws the fighter's model away and builds it again with another loadout (for example when the other player's
## arrives over the network, or while editing one).
func rebuild(l: RefCounted) -> void:
	for c in get_children():
		remove_child(c)
		c.free()
	meshes.clear()
	trail = null
	trail_points.clear()
	last_trail_frame = -1
	sweep = null
	sweep_samples.clear()
	dizzy = null
	launch_lines = null
	rig = null
	anim = null
	skeleton = null
	current_clip = ""
	face_mesh = null
	face_mat = null
	last_expression = {}
	shown_face = ""
	last_ghost = 0.0
	build(player, l)


func _part(parent: Node3D, mesh: Mesh, mat: Material, pos: Vector3, scl := Vector3.ONE, rot_deg := Vector3.ZERO) -> MeshInstance3D:
	var mi := MeshInstance3D.new()
	mi.mesh = mesh
	mi.material_override = mat
	mi.position = pos
	mi.scale = scl
	mi.rotation_degrees = rot_deg
	parent.add_child(mi)
	meshes.append(mi)
	return mi


func _sphere(r: float) -> SphereMesh:
	var key := "s%.3f" % r
	if _mesh_cache.has(key):
		return _mesh_cache[key]
	var s := SphereMesh.new()
	s.radius = r
	s.height = r * 2.0
	s.radial_segments = 20
	s.rings = 10
	_mesh_cache[key] = s
	return s


func _cyl(top: float, bottom: float, h: float) -> CylinderMesh:
	var c := CylinderMesh.new()
	c.top_radius = top
	c.bottom_radius = bottom
	c.height = h
	c.radial_segments = 20
	return c


const Loadout := preload("res://scripts/loadout.gd")
var loadout: RefCounted
const RIG_PATH := "res://models/blob_rig.glb"
## The brawler's rig has longer arms and legs. It is built about 0.19 taller at the hips, so it is scaled down to keep the same height.
const RIG_LONG_PATH := "res://models/blob_rig_long.glb"
## The base body: the character from the concept art, generated, cleaned up and rigged by the art pipeline (docs/ART_WORKFLOW.md,
## art/generated/base_body). One skinned body on the same skeleton and clips, with glove fists and the drawn face; tried out with
## `--base-body` for now. Its head, shoulder and wrist (for the hats, glasses, face and the weapon arm) come from base_rig.json, which
## make_rigged_blob.py writes with it.
const RIG_BASE_PATH := "res://models/base_rig.glb"
const BASE_INFO_PATH := "res://models/base_rig.json"
static var use_base_body: bool = OS.get_cmdline_user_args().has("--base-body")
static var _base_info: Dictionary = {}
const LONG_SCALE := 0.92
const LONG_LIFT := 0.19
const LONG_FIT := Transform3D(Basis(Vector3(0.92, 0, 0), Vector3(0, 0.92, 0), Vector3(0, 0, 0.92)), Vector3(0, 0.19 * 0.92, 0))
## Where the head and torso sit in the rig compared with the sphere-built look the face, hats and glasses were designed for.
const HEAD_FIT := Transform3D(Basis(Vector3(0.825, 0, 0), Vector3(0, 0.825, 0), Vector3(0, 0, 0.825)), Vector3(0, 1.56 - 1.42 * 0.825, 0))
const TORSO_FIT := Transform3D()
const LOOPING := ["idle", "walk", "run", "dash", "fall", "victory_a", "victory_b", "victory_c"]
## The rig is read once and copied for every fighter (reading it again renames its bones).
static var _rig_templates: Dictionary = {}

## Frees the models kept for copying (the game does this as it shuts down).
static func release_caches() -> void:
	for template in _rig_templates.values():
		if template != null and is_instance_valid(template):
			template.free()
	_rig_templates.clear()
	_parts.clear()
	_mat_cache.clear()
	_limb_cache.clear()
	_mesh_cache.clear()
	_outline_mat = null
	SvgArt.release()
	Particles.release()
	_shadow_tex = null


## Whether this fighter uses the long-limbed rig (set from the fighter's class; changing it rebuilds the model).
var long_limbs := false
var head_fit := Transform3D()
var torso_fit := Transform3D()
var shoulder := Vector2(0.5, 1.15)
var arm_reach := 0.5

var rig: Node3D
var skeleton: Skeleton3D
var anim: AnimationPlayer
var current_clip := ""
## The dash and run share one stride, set by this fighter's initial dash: a dash is one bounding step (from the push-off at
## `DASH_STEP_FROM` to the other foot landing half a cycle later) that lands as the dash ends, and the run goes on at two dash lengths
## a cycle, its legs turning over with the speed.
const DASH_STEP_FROM := 0.12
var gait_frame := -1
## Parents of the face / hat / glasses (they follow the head bone) and of the neckwear (it follows the torso). Without a rig both are
## simply the model.
var head_rig: Node3D
var torso_rig: Node3D
var head_bone := -1
var spine_bone := -1
var head_rest_inv := Transform3D()
var spine_rest_inv := Transform3D()

const PARTS_PATH := "res://models/blob_parts.glb"
static var _parts: Dictionary = {}
static var _parts_tried := false


## A model from `res://models/`. The `.glb` file is read directly when it is there (a development checkout, so a model that was just re-exported
## from Blender is used without waiting for Godot to import it); a packed game only has the imported copy, which is loaded as a scene.
static func _model_scene(path: String) -> Node:
	if FileAccess.file_exists(path):
		var doc := GLTFDocument.new()
		var state := GLTFState.new()
		if doc.append_from_file(path, state) == OK:
			return doc.generate_scene(state)
	if ResourceLoader.exists(path):
		var packed = load(path)
		if packed is PackedScene:
			return packed.instantiate()
	return null


static func _base_rig_info() -> Dictionary:
	if _base_info.is_empty() and FileAccess.file_exists(BASE_INFO_PATH):
		var data = JSON.parse_string(FileAccess.get_file_as_string(BASE_INFO_PATH))
		if data is Dictionary:
			_base_info = data
	return _base_info


## Builds the rigged blob (arms, legs and animation clips from art/blender/make_rigged_blob.py). Returns false if it is not available, and
## the fighter is then built from parts or spheres.
func _build_rig(skin: Material) -> bool:
	var rig_path := RIG_BASE_PATH if use_base_body else (RIG_LONG_PATH if long_limbs else RIG_PATH)
	if not _rig_templates.has(rig_path):
		_rig_templates[rig_path] = null
		_rig_templates[rig_path] = _model_scene(rig_path)
		if _rig_templates[rig_path] == null:
			push_warning("The character model %s did not load; using the simple body." % rig_path)
	if _rig_templates[rig_path] == null:
		return false
	var scene: Node3D = _rig_templates[rig_path].duplicate()
	var found: Array = scene.find_children("*", "Skeleton3D", true, false)
	var players: Array = scene.find_children("*", "AnimationPlayer", true, false)
	if found.is_empty() or players.is_empty():
		scene.free()
		return false
	skeleton = found[0]
	anim = players[0]
	rig = scene
	model.add_child(rig)
	anim.callback_mode_process = AnimationMixer.ANIMATION_CALLBACK_MODE_PROCESS_MANUAL
	for clip_name in anim.get_animation_list():
		anim.get_animation(clip_name).loop_mode = Animation.LOOP_LINEAR if LOOPING.has(clip_name) else Animation.LOOP_NONE
	var white := toon(Color(1, 1, 1))
	var shoe := toon(Color(0.27, 0.2, 0.3))
	var shirt := _shirt_material()
	# The shorts take the outfit colour (the accent, a shade darker) under a plain shirt, and navy under a coloured one.
	var shorts_colour: Color = loadout.accent_color().darkened(0.2) if loadout.shirt <= 1 else Color(0.22, 0.25, 0.42)
	var shorts := toon(shorts_colour)
	var skin_colour: Color = loadout.body_color()
	# Arms and legs carry their clothes as painted bands (shaders/limb.gdshader): a sleeve from the shoulder when there is a shirt; the
	# shorts' leg and a white sock on each leg.
	var arm := _limb_material(skin_colour, _sleeve_colour(), 0.36, Color.WHITE, 2.0)
	var leg := _limb_material(skin_colour, shorts_colour, 0.3, Color.WHITE, 0.62)
	for mi in rig.find_children("*", "MeshInstance3D", true, false):
		var part := str(mi.name)
		if part.begins_with("Arm"):
			mi.material_override = arm
		elif part.begins_with("Leg"):
			mi.material_override = leg
		elif part.begins_with("Hand") or part.begins_with("Sole") or part.begins_with("Collar") or part.begins_with("Cuff"):
			mi.material_override = white
		elif part.begins_with("Strap"):
			mi.material_override = toon(loadout.accent_color())
		elif part.begins_with("Shorts"):
			mi.material_override = shorts
		elif part.begins_with("Foot"):
			mi.material_override = shoe
		elif part.begins_with("Face"):
			face_mesh = mi
		elif part.begins_with("Body") and shirt != null:
			mi.material_override = shirt
		else:
			mi.material_override = skin
		meshes.append(mi)
	head_bone = skeleton.find_bone("head")
	spine_bone = skeleton.find_bone("spine")
	head_rest_inv = skeleton.get_bone_global_rest(head_bone).affine_inverse()
	spine_rest_inv = skeleton.get_bone_global_rest(spine_bone).affine_inverse()
	head_rig = Node3D.new()
	torso_rig = Node3D.new()
	model.add_child(head_rig)
	model.add_child(torso_rig)
	if use_base_body:
		# The look's pieces were designed round a head of radius 0.8 centred 1.42 up: scaled and lifted onto this body's head. The
		# neckwear goes up to its neck, narrower; the weapon arm turns at its shoulder and reaches to its wrist and a little past.
		var info := _base_rig_info()
		var centre: Array = info.get("head_centre", [0.0, 1.86, 0.0])
		var radii: Array = info.get("head_radii", [0.4, 0.34, 0.4])
		var k := float(radii[0]) / 0.8
		head_fit = Transform3D(Basis.from_scale(Vector3.ONE * k), Vector3(0.0, float(centre[1]) - 1.42 * k, float(centre[2])))
		torso_fit = Transform3D(Basis.from_scale(Vector3(0.7, 1.0, 0.7)), Vector3(0.0, float(info.get("neck", 1.47)) - 1.26, 0.0))
		var sh: Array = info.get("shoulder", [0.41, 1.19, 0.04])
		var wr: Array = info.get("wrist", [0.53, 0.74, 0.15])
		shoulder = Vector2(float(sh[0]), float(sh[1]))
		arm_reach = Vector2(float(wr[0]) - float(sh[0]), float(wr[1]) - float(sh[1])).length() + 0.1
	elif long_limbs:
		rig.scale = Vector3.ONE * LONG_SCALE
		head_fit = LONG_FIT * HEAD_FIT
		torso_fit = LONG_FIT
		shoulder = Vector2(0.5 * LONG_SCALE, (1.15 + LONG_LIFT) * LONG_SCALE)
		arm_reach = ARM_REACH * 1.4 * LONG_SCALE
	else:
		head_fit = HEAD_FIT
		torso_fit = TORSO_FIT
		shoulder = SHOULDER
		arm_reach = ARM_REACH
	head_rig.transform = head_fit
	torso_rig.transform = torso_fit
	anim.play("idle")
	anim.advance(0.0)
	return true


## Which clip suits this state of the simulation: [clip name, speed, progress]. Progress is -1 for a clip that simply plays, or 0..1 for
## one that follows the move (an attack's swing lands on the move's own frames).
func _choose_clip(s: Dictionary) -> Array:
	var state: String = s.state
	var grounded: bool = s.platform >= 0
	var speed := absf(float(s.vel.x))
	match state:
		"Walk":
			return ["walk", clampf(speed / 0.09, 0.5, 2.4), -1.0]
		"Run", "Dash":
			# Cycles a second (two dash lengths each) times the clip's length in seconds.
			var stride := 2.0 * maxf(0.5, float(s.get("dash_length", 2.4)))
			var clip := "dash" if state == "Dash" else "run"
			var length := anim.get_animation(clip).length if anim != null and anim.has_animation(clip) else 0.3
			return [clip, minf(2.5, speed * 60.0 / stride * length), -1.0]
		"Crouch", "JumpSquat", "Landing", "WaveLand":
			return ["crouch", 1.0, -1.0]
		"Roll", "SpotDodge", "AirDodge":
			return ["roll", 1.0, -1.0]
		"Shield", "ShieldDrop":
			return ["shield", 1.0, -1.0]
		"Hitstun", "ShieldBreak", "Grabbed", "Rebound":
			return ["hurt", 1.0, -1.0]
		"WallTech":
			return ["shield", 1.0, -1.0]
		"Knockdown", "GetUp":
			return ["knockdown", 1.0, -1.0]
		"LedgeHang":
			return ["ledge", 1.0, -1.0]
		"Grabbing":
			return ["grab", 1.0, -1.0]
		"Attack":
			return _attack_clip(s)
		"Turn":
			# Turning out of a run skids; a turn from standing just turns.
			return ["skid", 1.0, -1.0] if speed > 0.08 else ["idle", 1.0, -1.0]
		"LedgeGetUp":
			if ledge_rolling:
				return ["roll", 1.0, -1.0]
			return ["ledge_climb", 1.0, ledge_k]
		"LedgeAttack":
			# Up onto the edge, then the sweep (its hit on frames 24-26 of 55 lands on the clip's strike).
			var f: float = s.state_frame
			if f < 14.0:
				return ["ledge_climb", 1.0, clampf(f / 14.0, 0.0, 1.0) * 0.66]
			var p := 0.55 * (f - 14.0) / 11.0 if f < 25.0 else 0.55 + (f - 25.0) / 30.0 * 0.45
			return ["attack_low", 1.0, clampf(p, 0.0, 1.0)]
		"Idle", "Respawn", "ShieldRelease":
			return ["idle", 1.0, -1.0]
	if grounded:
		return ["idle", 1.0, -1.0]
	if flip_left > 0.0:
		return ["air_jump", 1.0, -1.0]
	return ["jump", 1.0, -1.0] if float(s.vel.y) > 0.02 else ["fall", 1.0, -1.0]


## The clip for the move being performed and how far through it we are, mapped onto the clip's timeline: wind-up to 0.35, the strike
## through 0.55 to 0.75, then recovery. The move's own frames set the pace, so the swing lands when the hitbox does.
func _attack_clip(s: Dictionary) -> Array:
	var t: PackedInt32Array = s.move_timing
	var name: String = s.move_name
	var total := maxf(1.0, float(t[0]))
	var start := clampf(float(t[1]), 1.0, total)
	var last := clampf(float(t[2]), start, total)
	var f: float = s.state_frame
	var progress: float
	# The swing lands on the move's first active frame and follows through while it is active; through the end lag the body holds the
	# follow-through and only settles back near the end (as the reference game's moves do), so a move looks as long as it really is.
	if f < start:
		progress = f / start * 0.55
	elif f <= last:
		progress = 0.55 + (f - start) / maxf(1.0, last - start) * 0.08
	else:
		var k := clampf((f - last) / maxf(1.0, total - last), 0.0, 1.0)
		progress = 0.63 + 0.37 * pow(k, 1.8)
	var brawler_body: bool = _cls(s) == 1 and BRAWLER_NO_BLADE.has(name)
	var clip := "attack_swing"
	if name.ends_with("throw"):
		clip = "throw"
	elif name.begins_with("grab") or name.begins_with("dash grab") or name.begins_with("pivot") or name == "pummel":
		clip = "grab"
	elif brawler_body:
		clip = KICK_CLIPS.get(name, "attack_kick")
	elif _cls(s) == 1 and name == "neutral special":
		clip = "blaster"
	elif _cls(s) != 1 and SWORD_CLIPS.has(name):
		clip = SWORD_CLIPS[name]
	elif name == "fair":
		clip = "attack_fair"
	elif name == "bair":
		clip = "attack_bair"
	elif name == "nair":
		clip = "attack_nair"
	elif name == "uair":
		clip = "attack_uair"
	elif name == "dair":
		clip = "attack_dair"
	elif name.ends_with("smash"):
		clip = "attack_smash"
	elif name == "dtilt" or name == "down special":
		clip = "attack_low"
	elif name.begins_with("jab"):
		clip = "attack_jab"
	elif name == "dash attack" or name == "side special":
		clip = "attack_lunge"
	elif name == "utilt" or name == "up special":
		clip = "attack_uair"
	elif name == "neutral special":
		clip = "attack_smash"
	return [clip, 1.0, clampf(progress, 0.0, 1.0)]


func _skeleton_to_model() -> Transform3D:
	return model.global_transform.affine_inverse() * skeleton.global_transform


## The stage frame: a node in the model that undoes its turn, so its axes are the stage's (x along the stage, y up, z toward the
## camera) while it still leans, squashes and moves with the body. Everything aimed at a hit (the weapon, the arm and leg reaching, the
## body following them) works in it.
var stage_frame: Node3D


func _skeleton_to_frame() -> Transform3D:
	return stage_frame.global_transform.affine_inverse() * skeleton.global_transform


## How the fighter is turned (the "cheat" of a 3D fighter on a 2D stage): the hips and legs are turned most of the way toward the way it
## faces, so strides, kicks and swings happen across the screen where they read; the chest and then the head twist back toward the
## camera, so the body and the face stay open to the player.
const BODY_TURN := 60.0
const CHEST_TO_CAMERA := 30.0
## Negative: the head turns back toward the opponent from the chest (to about 40 degrees off straight-on), and the eyes in the face look
## that way too (see `set_gaze`), so the two fighters look at each other while their bodies stay open to the camera.
const HEAD_TO_CAMERA := -10.0


## Plays the right clip for this frame and moves the head and torso followers with their bones.
func _animate(s: Dictionary, delta: float) -> void:
	if anim == null:
		return
	var pick: Array = _choose_clip(s)
	var clip: String = pick[0]
	# Where the stride is: a dash to a run goes on in step, and a new dash (also one the other way) starts on its push-off.
	var gait := -1.0
	if current_clip == "dash" or current_clip == "run":
		gait = fmod(anim.current_animation_position / maxf(anim.current_animation_length, 0.001), 1.0)
	var new_dash: bool = s.state == "Dash" and (current_clip != "dash" or int(s.state_frame) < gait_frame)
	gait_frame = int(s.state_frame) if s.state == "Dash" else -1
	if clip != current_clip:
		anim.play(clip, 0.1 if current_clip != "" and not clip.begins_with("attack") else 0.0)
		current_clip = clip
		if clip == "run" and gait >= 0.0:
			anim.seek(gait * anim.current_animation_length, true)
	if new_dash:
		# Off the foot that is down: the push-off of whichever step comes next.
		var from := DASH_STEP_FROM
		if gait >= 0.0 and fmod(gait - DASH_STEP_FROM + 1.0, 1.0) >= 0.5:
			from += 0.5
		anim.seek(from * anim.current_animation_length, true)
	if float(pick[2]) >= 0.0:
		anim.pause()
		anim.seek(float(pick[2]) * anim.get_animation(clip).length, true)
	elif int(s.hitlag) == 0:
		# (Hitlag freezes the pose along with everything else.)
		anim.speed_scale = float(pick[1])
		anim.advance(delta)
	_body_follow(s, delta)
	var to_model := _skeleton_to_model()
	var head_delta: Transform3D = skeleton.get_bone_global_pose(head_bone) * head_rest_inv
	var spine_delta: Transform3D = skeleton.get_bone_global_pose(spine_bone) * spine_rest_inv
	head_rig.transform = to_model * head_delta * to_model.affine_inverse() * head_fit
	torso_rig.transform = to_model * spine_delta * to_model.affine_inverse() * torso_fit


## The rest of the body answers the limb that is reaching for a hit (the kicking leg, or the weapon arm): the torso bends at the waist
## so the limb can get there (back for a kick high overhead, forward over a kick behind or a low swing), and the head looks at the hit.
## It grows with the move's wind-up and fades in its recovery, and it is added on top of the move's clip, so every move gets it.
var body_bend := 0.0
var head_look := 0.0


func _body_follow(s: Dictionary, delta: float) -> void:
	var target_bend := 0.0
	var target_look := 0.0
	var tip: Vector3 = s.move_tip
	if s.state == "Attack" and tip != Vector3.ZERO and spine_bone >= 0:
		var kicking: bool = _cls(s) == 1 and KICK_CLIPS.has(s.move_name)
		var to_frame := _skeleton_to_frame()
		var hips := skeleton.find_bone("hips")
		var hip_y: float = (to_frame * skeleton.get_bone_global_pose(hips).origin).y if hips >= 0 else 0.6
		var from := Vector2(0.0, hip_y) if kicking else shoulder
		var to := Vector2(tip.x, tip.y) - from
		# The limb's direction: 0 straight down, 90 straight ahead, 180 straight up, negative behind.
		var phi := rad_to_deg(atan2(to.x, -to.y))
		if kicking:
			# A leg swings comfortably from about 40 degrees behind to 100 in front; past that the body has to make room.
			if phi > 100.0:
				target_bend = -minf((phi - 100.0) * 0.85, 50.0)
			elif phi < -40.0:
				# (Less: kicks behind already lean the body forward in their clips.)
				target_bend = minf((-40.0 - phi) * 0.4, 24.0)
		else:
			# An arm reaches almost anywhere: lean into low swings in front, a little back from ones overhead or behind.
			if phi > 0.0 and phi < 80.0:
				target_bend = (80.0 - phi) * 0.35
			elif phi > 140.0:
				target_bend = -(phi - 140.0) * 0.4
			elif phi < -60.0:
				target_bend = (-60.0 - phi) * 0.3
		# Look at the hit: up for a hit overhead, down for a low one, by how high it is (about half of the angle to it).
		var rise := rad_to_deg(atan2(to.y, absf(to.x)))
		target_look = clampf(rise * 0.5, -20.0, 30.0)
		# In proportion to how far into the move the limb is (its blend toward the hitbox).
		var k := leg_k if kicking else arm_k
		target_bend *= k
		target_look *= k
	var follow := clampf(delta * 16.0, 0.0, 1.0)
	body_bend = lerpf(body_bend, target_bend, follow)
	head_look = lerpf(head_look, target_look, follow)
	if absf(body_bend) > 0.05:
		_bend_bone(spine_bone, body_bend, int(s.facing))
	if absf(head_look) > 0.05:
		_bend_bone(head_bone, -head_look, int(s.facing))
	# Open the chest and the face to the camera (see BODY_TURN).
	_turn_bone(spine_bone, -CHEST_TO_CAMERA * float(s.facing))
	_turn_bone(head_bone, -HEAD_TO_CAMERA * float(s.facing))


## Turns a bone about the vertical by `degrees` (positive is counter-clockwise seen from above), on top
## of its pose.
func _turn_bone(bone: int, degrees: float) -> void:
	var parent := skeleton.get_bone_parent(bone)
	if parent < 0:
		return
	var axis: Vector3 = (_skeleton_to_frame().affine_inverse().basis * Vector3(0, 1, 0)).normalized()
	var turn := Basis(axis, deg_to_rad(degrees))
	var g := skeleton.get_bone_global_pose(bone)
	var pg := skeleton.get_bone_global_pose(parent)
	skeleton.set_bone_pose_rotation(bone, (pg.basis.inverse() * (turn * g.basis)).get_rotation_quaternion())


## Bends a bone toward the fighter's front (the way it faces in the world) by `degrees` (negative bends back), about its own joint,
## on top of the pose the clip gave it.
func _bend_bone(bone: int, degrees: float, facing: int) -> void:
	var parent := skeleton.get_bone_parent(bone)
	if parent < 0:
		return
	# The model leans toward its facing by turning about its own front axis (as `apply` does), so the bend uses the same axis.
	var axis: Vector3 = (_skeleton_to_frame().affine_inverse().basis * Vector3(0, 0, 1)).normalized()
	var turn := Basis(axis, deg_to_rad(-degrees * float(facing)))
	var g := skeleton.get_bone_global_pose(bone)
	var pg := skeleton.get_bone_global_pose(parent)
	var local := pg.basis.inverse() * (turn * g.basis)
	skeleton.set_bone_pose_rotation(bone, local.get_rotation_quaternion())


## A victory pose for the results screen: plays one of the looping victory clips (by class: the sword and the maul are raised high, the
## claws fighter pumps a fist; `cheer` makes anyone throw both arms up and hop), with the weapon held up in the raised hand. Call every
## frame with the time since the last.
func play_victory(cls: int, delta: float, cheer := false) -> void:
	if anim == null:
		return
	var clip := "victory_c" if cheer else ("victory_b" if cls == 1 else "victory_a")
	set_expression(Loadout.HAPPY_FACE)
	_blink(delta)
	if current_clip != clip and anim.has_animation(clip):
		anim.play(clip, 0.15)
		current_clip = clip
	anim.speed_scale = 1.0
	anim.advance(delta)
	var to_model := _skeleton_to_model()
	var head_delta: Transform3D = skeleton.get_bone_global_pose(head_bone) * head_rest_inv
	var spine_delta: Transform3D = skeleton.get_bone_global_pose(spine_bone) * spine_rest_inv
	head_rig.transform = to_model * head_delta * to_model.affine_inverse() * head_fit
	torso_rig.transform = to_model * spine_delta * to_model.affine_inverse() * torso_fit
	# The sword or the maul, gripped in the raised hand and pointing up.
	var armed := cls != 1 and not cheer
	blade_pivot.visible = armed
	if armed:
		var hand := skeleton.find_bone("hand.R")
		var at: Vector3 = _skeleton_to_frame() * skeleton.get_bone_global_pose(hand).origin
		blade_pivot.position = at
		blade_pivot.rotation = Vector3(0, 0, PI / 2.0)
		var length := 1.9
		var hammer := cls == 2
		blade_pivot.scale = Vector3(length / MESH_LENGTH, 1.0, 1.0) if not hammer else Vector3.ONE
		for part in blade_parts:
			part.visible = not hammer
		for part in hammer_parts:
			part.visible = hammer
		if hammer:
			hammer_parts[0].scale.x = (length - 0.35) / 2.5
			hammer_parts[0].position.x = (length - 0.35) / 2.0
			hammer_parts[1].position.x = length
			hammer_parts[2].position.x = length


## The meshes of the modelled blob (Body, Head, FootL, FootR, HandL, HandR), read straight from the glTF file so no editor import is
## needed; empty if the file is missing or damaged (the fighter then falls back to spheres).
static func blob_parts() -> Dictionary:
	if _parts_tried:
		return _parts
	_parts_tried = true
	var scene := _model_scene(PARTS_PATH)
	if scene == null:
		return _parts
	var found := {}
	for node in scene.find_children("*", "MeshInstance3D", true, false):
		found[str(node.name)] = (node as MeshInstance3D).mesh
	scene.free()
	for needed in ["Body", "Head", "FootL", "FootR", "HandL", "HandR"]:
		if not found.has(needed):
			return _parts
	_parts = found
	return _parts


## The shell in front of the head that shows the drawn face, and its material (the expression swaps its texture).
var face_mesh: MeshInstance3D
var face_mat: ShaderMaterial


## Builds the fighter. `l` is its cosmetic loadout (see loadout.gd); without one the player's default look is used.
func build(p: int, l: RefCounted = null) -> void:
	player = p
	loadout = l if l != null else Loadout.default_for(p)
	var col: Color = loadout.body_color()
	var skin := toon(col)
	# (The launch stretch, `_launch_look`, is applied to a frame round the model.)
	launch_frame = Node3D.new()
	add_child(launch_frame)
	model = Node3D.new()
	launch_frame.add_child(model)

	# Body, head, feet, hands. Height matches the 2.2 unit ECB.
	var used_rig := _build_rig(skin)
	var shaped := {} if used_rig else blob_parts()
	if used_rig:
		pass
	elif shaped.is_empty():
		# No modelled parts available: plain spheres.
		_part(model, _sphere(0.62), skin, Vector3(0, 0.82, 0), Vector3(1.0, 0.95, 0.9))
		_part(model, _sphere(0.8), skin, Vector3(0, 1.42, 0))
		for sx in [-1.0, 1.0]:
			_part(model, _sphere(0.24), skin, Vector3(sx * 0.32, 0.2, 0.05), Vector3(1, 0.8, 1.35))
			_part(model, _sphere(0.22), toon(Color(1, 1, 1)), Vector3(sx * 0.78, 0.85, 0.05))
	else:
		# Parts modelled in Blender (art/blender/make_blob_parts.py), at the size the spheres had.
		_part(model, shaped.Body, skin, Vector3(0, 0.82, 0))
		_part(model, shaped.Head, skin, Vector3(0, 1.42, 0))
		for sx in [-1.0, 1.0]:
			var side := "L" if sx < 0.0 else "R"
			_part(model, shaped["Foot" + side], skin, Vector3(sx * 0.32, 0.2, 0.05))
			_part(model, shaped["Hand" + side], toon(Color(1, 1, 1)), Vector3(sx * 0.78, 0.85, 0.05))

	if torso_rig == null or not is_instance_valid(torso_rig) or torso_rig.get_parent() != model:
		# Without the rig, the accessories hang off plain nodes at the origin.
		head_rig = Node3D.new()
		torso_rig = Node3D.new()
		model.add_child(head_rig)
		model.add_child(torso_rig)

	_neck(skin)
	_face()
	_hat()
	_glasses()

	_contact_shadow()
	_on_fighter_layer(model)

	shield = MeshInstance3D.new()
	shield.mesh = _sphere(1.5)
	var sm := ShaderMaterial.new()
	sm.shader = SHIELD_SHADER
	shield.material_override = sm
	shield.position = Vector3(0, 1.1, 0)
	shield.visible = false
	add_child(shield)

	# Speed lines above the fighter, shown while fast falling.
	speed_lines = Node3D.new()
	var line_mat := StandardMaterial3D.new()
	line_mat.albedo_color = Color(1, 1, 1, 0.8)
	line_mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	line_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	for x in [-0.55, 0.0, 0.55]:
		var bar := MeshInstance3D.new()
		var bm := BoxMesh.new()
		bm.size = Vector3(0.07, 1.5, 0.07)
		bar.mesh = bm
		bar.material_override = line_mat
		bar.position = Vector3(x, 3.5 + (0.4 if x == 0.0 else 0.0), 0.2)
		speed_lines.add_child(bar)
	speed_lines.visible = false
	add_child(speed_lines)

	# Damage readout above the head.
	percent_label = Label3D.new()
	percent_label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	percent_label.font_size = 80
	percent_label.pixel_size = 0.0075
	percent_label.outline_size = 20
	percent_label.no_depth_test = true
	percent_label.position = Vector3(0, 4.1, 0)
	percent_label.text = "0%"
	add_child(percent_label)
	# The fighter's name, small, under the damage readout.
	name_label = Label3D.new()
	name_label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	name_label.font_size = 48
	name_label.pixel_size = 0.0075
	name_label.outline_size = 14
	name_label.no_depth_test = true
	name_label.position = Vector3(0, 3.55, 0)
	name_label.text = name_text
	name_label.visible = name_text != ""
	add_child(name_label)

	# Weapon: pivots at the hand and extends along +x. Claws just shrink it (see _pose_blade).
	blade_parts.clear()
	hammer_parts.clear()
	stage_frame = Node3D.new()
	model.add_child(stage_frame)
	blade_pivot = Node3D.new()
	stage_frame.add_child(blade_pivot)
	var blade_mesh := BoxMesh.new()
	blade_mesh.size = Vector3(2.7, 0.16, 0.09)
	blade_parts.append(_part(blade_pivot, blade_mesh, toon(Color(0.86, 0.91, 0.99)), Vector3(1.55, 0, 0)))
	var guard_mesh := BoxMesh.new()
	guard_mesh.size = Vector3(0.12, 0.6, 0.12)
	blade_parts.append(_part(blade_pivot, guard_mesh, toon(Color(0.92, 0.76, 0.3)), Vector3(0.18, 0, 0)))
	var grip_mesh := BoxMesh.new()
	grip_mesh.size = Vector3(0.4, 0.12, 0.12)
	_part(blade_pivot, grip_mesh, toon(SASH), Vector3(-0.02, 0, 0))
	# The maul: a thick shaft and a big two-sided head at the far end (shown instead of the blade for the third class).
	var shaft_mesh := BoxMesh.new()
	shaft_mesh.size = Vector3(2.5, 0.2, 0.2)
	hammer_parts.append(_part(blade_pivot, shaft_mesh, toon(Color(0.55, 0.36, 0.2)), Vector3(1.3, 0, 0)))
	var head_mesh := BoxMesh.new()
	head_mesh.size = Vector3(0.7, 0.7, 0.6)
	hammer_parts.append(_part(blade_pivot, head_mesh, toon(Color(0.36, 0.4, 0.5)), Vector3(2.45, 0, 0)))
	# A small gold stud on the head's top, so it reads as a made thing rather than a grey box.
	var stud_mesh := BoxMesh.new()
	stud_mesh.size = Vector3(0.22, 0.2, 0.22)
	hammer_parts.append(_part(blade_pivot, stud_mesh, toon(Color(0.92, 0.76, 0.3)), Vector3(2.45, 0.42, 0)))
	for part in hammer_parts:
		part.visible = false

	# Impact spark, shown during hitlag.
	spark = MeshInstance3D.new()
	spark.mesh = _star_mesh()
	var spark_mat := StandardMaterial3D.new()
	spark_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	spark_mat.vertex_color_use_as_albedo = true
	spark_mat.albedo_color = Color(1.0, 0.97, 0.7, 0.95)
	spark_mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	spark_mat.cull_mode = BaseMaterial3D.CULL_DISABLED
	spark.material_override = spark_mat
	spark.position = Vector3(0, 1.3, 0.6)
	spark.visible = false
	add_child(spark)

	# Flame around the body for the brawler's rushing specials (a stand-in for a proper effect).
	flame = Particles.flame(self)
	add_child(flame)
	_tag_object(self)


## Gives every cel-shaded part of this fighter its own object id (see `object_id` in shaders/toon.gdshader), so the ink pass draws a line
## where this fighter meets anything else but none between its own parts.
func _tag_object(node: Node) -> void:
	if node is GeometryInstance3D:
		# Its material may be the override or set per surface (the rig's parts).
		var mats: Array = [(node as GeometryInstance3D).material_override]
		if node is MeshInstance3D and (node as MeshInstance3D).mesh != null:
			for i in (node as MeshInstance3D).mesh.get_surface_count():
				mats.append((node as MeshInstance3D).get_active_material(i))
		for m in mats:
			if m is ShaderMaterial and ((m as ShaderMaterial).shader in [TOON_SHADER, LIMB_SHADER, FACE_SHADER]):
				(node as GeometryInstance3D).set_instance_shader_parameter("object_id", 0.06 + 0.07 * float(player % 8))
				break
	for c in node.get_children():
		_tag_object(c)


## The whole-body flash: white for a moment when the fighter is hit (strongest at the start of hitlag), a yellow pulse while a smash
## attack is charging. Cosmetic.
var flash_amount := 0.0
var flash_colour := Color.WHITE
var last_flash := -1.0
var flash_hit_frame := -1


func _flash(s: Dictionary, delta: float) -> void:
	var target := 0.0
	var colour := Color.WHITE
	var struck: bool = int(s.hitlag) > 0 and s.launch_pending
	if not struck:
		flash_hit_frame = -1
	if struck:
		# A bright flash on the first three frames of the freeze, then only a faint one, so the fighter stays readable.
		if flash_hit_frame < 0:
			flash_hit_frame = int(s.frame)
		target = 0.55 if int(s.frame) - flash_hit_frame < 3 else 0.12
	elif int(s.charge) > 0 and s.state == "Attack":
		colour = Color(1.0, 0.85, 0.25)
		target = 0.12 + 0.1 * sin(float(s.frame) * 0.6)
	flash_amount = target if target > flash_amount else move_toward(flash_amount, target, delta * 12.0)
	flash_colour = colour
	var value := Color(flash_colour.r, flash_colour.g, flash_colour.b, flash_amount)
	if absf(flash_amount - last_flash) < 0.001:
		return
	last_flash = flash_amount
	for m in meshes:
		if m.material_override is ShaderMaterial:
			m.set_instance_shader_parameter("flash", value)
	if face_mesh != null:
		face_mesh.set_instance_shader_parameter("flash", value)


## A soft dark oval projected down onto whatever the fighter stands over (as the reference game draws one), so its place on the stage reads
## at a glance, in the air too. A decal reaching down from the feet; it fades with height.
var contact_shadow: Decal
static var _shadow_tex: GradientTexture2D
const SHADOW_DEPTH := 10.0


## Puts every mesh under `node` on render layer 2 (seen and lit as usual), so the contact shadow, which only projects on layer 1, skips them.
func _on_fighter_layer(node: Node) -> void:
	for g in node.find_children("*", "GeometryInstance3D", true, false):
		(g as GeometryInstance3D).layers = 2


func _contact_shadow() -> void:
	if _shadow_tex == null:
		var g := Gradient.new()
		g.set_color(0, Color(0.04, 0.02, 0.1, 0.85))
		g.set_color(1, Color(0.04, 0.02, 0.1, 0.0))
		g.add_point(0.45, Color(0.04, 0.02, 0.1, 0.75))
		g.add_point(0.75, Color(0.04, 0.02, 0.1, 0.35))
		_shadow_tex = GradientTexture2D.new()
		_shadow_tex.gradient = g
		_shadow_tex.fill = GradientTexture2D.FILL_RADIAL
		_shadow_tex.fill_from = Vector2(0.5, 0.5)
		_shadow_tex.fill_to = Vector2(1.0, 0.5)
		_shadow_tex.width = 64
		_shadow_tex.height = 64
	contact_shadow = Decal.new()
	contact_shadow.texture_albedo = _shadow_tex
	contact_shadow.size = Vector3(3.0, SHADOW_DEPTH, 2.6)
	# Only on surfaces facing up (not down the front of the stage).
	contact_shadow.normal_fade = 0.6
	# (Nudged toward the camera so it shows in front of the feet, which would otherwise hide it from the low match camera.)
	contact_shadow.position = Vector3(0, 0.8 - SHADOW_DEPTH / 2.0, 0.45)
	# Only the stage takes it: the fighter's own meshes are on layer 2 (see `build`).
	contact_shadow.cull_mask = 1
	contact_shadow.upper_fade = 0.0  # (no fade toward the feet; the floor is at the top of the box)
	contact_shadow.lower_fade = 0.5
	add_child(contact_shadow)


## The face is a drawing (godot/art/faces/<expression>.svg) on the shell in front of the head, lit like the skin around it. The lids in
## the drawings are a placeholder colour that becomes the fighter's own colour, a shade darker.
const FACE_DIR := "res://art/faces/"
const LID_PLACEHOLDER := "#fe00ff"


func _face() -> void:
	if face_mesh == null:
		return
	face_mat = ShaderMaterial.new()
	face_mat.shader = FACE_SHADER
	face_mat.set_shader_parameter("skin", loadout.body_color())
	face_mesh.material_override = face_mat
	set_expression(Loadout.FACES[loadout.face])


## Which way the eyes look: 1 toward screen right, -1 toward screen left (the face drawings look right; facing left mirrors them).
func set_gaze(side: int) -> void:
	if face_mat != null:
		face_mat.set_shader_parameter("mirror", side < 0)


## Shows an expression: one of Loadout.FACES or Loadout.HURT (its `name` picks the drawing).
func set_expression(e: Dictionary) -> void:
	if e == last_expression:
		return
	last_expression = e
	_show_face()


## The drawing to show now: the expression, or closed eyes for a moment while blinking.
func _show_face() -> void:
	if face_mat == null or last_expression.is_empty():
		return
	var name_now: String = last_expression.name
	# Blinks show on the calm faces only (not mid-yell, mid-strain or with the eyes already shut).
	var calm: bool = name_now == Loadout.FOCUS_FACE.name or Loadout.FACES.any(func(f): return f.name == name_now)
	if blink_left > 0.0 and calm:
		name_now = "blink"
	if name_now == shown_face:
		return
	shown_face = name_now
	var lid: Color = loadout.body_color().darkened(0.1)
	face_mat.set_shader_parameter("face_tex", SvgArt.texture(FACE_DIR + name_now + ".svg", {LID_PLACEHOLDER: lid}))


## Blinks every few seconds (cosmetic, so it uses the frame time rather than the simulation).
func _blink(delta: float) -> void:
	if blink_left > 0.0:
		blink_left -= delta
		if blink_left <= 0.0:
			blink_wait = randf_range(2.2, 5.0)
	else:
		blink_wait -= delta
		if blink_wait <= 0.0:
			blink_left = 0.11
	_show_face()


var shown_face := ""
var blink_wait := 3.0
var blink_left := 0.0


## Shirts (Loadout.SHIRTS): the body in the shirt, plain or with a print from godot/art/cloth/. Null for none (the bare body).
const SHIRT_WHITE := Color(0.97, 0.95, 0.9)


func _shirt_material() -> Material:
	var accent: Color = loadout.accent_color()
	match loadout.shirt:
		1:
			return toon(SHIRT_WHITE)
		2:
			return toon(accent)
		3:
			return toon(Color(1, 1, 1), true, SvgArt.texture("res://art/cloth/stripes.svg", {"#ff00ff": accent}), Vector2(1, 7))
		4:
			return toon(Color(1, 1, 1), true, SvgArt.texture("res://art/cloth/aloha.svg", {"#ff00ff": accent}), Vector2(4, 2))
	return null


## The sleeve band's colour (the shirt's main colour), or none (a negative alpha) without a shirt.
func _sleeve_colour() -> Color:
	match loadout.shirt:
		1:
			return SHIRT_WHITE
		2, 3, 4:
			return loadout.accent_color()
	return Color(0, 0, 0, -1)


static var _limb_cache := {}


## The banded limb material (see shaders/limb.gdshader), with the ink outline. A `top` with negative alpha means no top band.
static func _limb_material(skin_c: Color, top_c: Color, top_end: float, bottom_c: Color, bottom_start: float) -> ShaderMaterial:
	var key := "%s%s%s%s%s" % [skin_c.to_html(), top_c, top_end, bottom_c.to_html(), bottom_start]
	if _limb_cache.has(key):
		return _limb_cache[key]
	var m := ShaderMaterial.new()
	m.shader = LIMB_SHADER
	m.set_shader_parameter("skin", skin_c)
	if top_c.a >= 0.0:
		m.set_shader_parameter("top", Color(top_c.r, top_c.g, top_c.b, 1.0))
		m.set_shader_parameter("top_end", top_end)
	m.set_shader_parameter("bottom", bottom_c)
	m.set_shader_parameter("bottom_start", bottom_start)
	m.next_pass = toon(Color.WHITE).next_pass
	_limb_cache[key] = m
	return m


var last_expression := {}


func _neck(_skin: Material) -> void:
	var accent: Color = loadout.accent_color()
	if rig != null:
		_neck_on_rig(accent)
		return
	match loadout.neck:
		1:
			# Sash across the torso.
			var sash := BoxMesh.new()
			sash.size = Vector3(1.5, 0.2, 0.12)
			_part(torso_rig, sash, toon(SASH), Vector3(0, 0.85, 0.5), Vector3.ONE, Vector3(0, 0, -42))
		2:
			# Neckerchief: a knotted square at the throat.
			var sq := BoxMesh.new()
			sq.size = Vector3(0.7, 0.7, 0.1)
			_part(torso_rig, sq, toon(accent), Vector3(0, 1.0, 0.6), Vector3.ONE, Vector3(0, 0, 45))
			_part(torso_rig, _sphere(0.16), toon(accent), Vector3(0, 1.28, 0.62))
		3:
			# Scarf: a ring round the neck with a tail hanging in front.
			var ring := TorusMesh.new()
			ring.inner_radius = 0.5
			ring.outer_radius = 0.72
			_part(torso_rig, ring, toon(accent), Vector3(0, 1.07, 0), Vector3(1, 0.55, 1))
			var tail := BoxMesh.new()
			tail.size = Vector3(0.28, 0.7, 0.08)
			_part(torso_rig, tail, toon(accent), Vector3(0.28, 0.72, 0.58), Vector3.ONE, Vector3(0, 0, 8))


## Neckwear for the rigged torso (a squat ellipsoid centred at height 0.98: half-width 0.56, half-depth 0.5, half-height 0.42). Rings are
## tori sized to hug it, so nothing pokes through the body whatever the pose.
func _neck_on_rig(accent: Color) -> void:
	var centre := Vector3(0, 0.98, 0)
	match loadout.neck:
		1:
			# Sash: a ring worn diagonally from shoulder to hip.
			var ring := TorusMesh.new()
			ring.inner_radius = 0.5
			ring.outer_radius = 0.64
			ring.rings = 36
			ring.ring_segments = 10
			# A flat band (the ring's tube stretched across the ring's plane), worn diagonally.
			var sash := Node3D.new()
			sash.position = centre
			sash.rotation_degrees = Vector3(0, 0, -42)
			torso_rig.add_child(sash)
			_part(sash, ring, toon(SASH, false), Vector3.ZERO, Vector3(0.88, 1.7, 1.0))
			# Round badges pinned on the front of the band, facing out from it.
			var badge := CylinderMesh.new()
			badge.top_radius = 0.068
			badge.bottom_radius = 0.068
			badge.height = 0.03
			badge.radial_segments = 16
			var colours := [Color(1.0, 0.82, 0.3), accent, Color(0.95, 0.95, 0.92)]
			for k in 3:
				var t := deg_to_rad(78.0 + 15.0 * k)
				var out := Vector3(0.88 * cos(t), 0.0, sin(t)).normalized()
				var b := _part(sash, badge, toon(colours[k]), Vector3(0.88 * 0.655 * cos(t), 0.0, 0.655 * sin(t)))
				b.basis = Basis(Quaternion(Vector3.UP, out))
		2:
			# Neckerchief: a collar ring round the top of the torso with a point hanging in front.
			_collar(accent)
			var bib := BoxMesh.new()
			bib.size = Vector3(0.34, 0.34, 0.06)
			_part(torso_rig, bib, toon(accent), Vector3(0, 0.94, 0.5), Vector3.ONE, Vector3(0, 0, 45))
		3:
			# Scarf: the collar ring and a tail hanging down the front on one side.
			_collar(accent)
			var tail := BoxMesh.new()
			tail.size = Vector3(0.2, 0.5, 0.06)
			_part(torso_rig, tail, toon(accent), Vector3(0.26, 0.82, 0.46), Vector3.ONE, Vector3(0, 0, 6))


func _collar(accent: Color) -> void:
	var ring := TorusMesh.new()
	ring.inner_radius = 0.44
	ring.outer_radius = 0.58
	ring.rings = 36
	ring.ring_segments = 10
	_part(torso_rig, ring, toon(accent), Vector3(0, 1.1, 0), Vector3(1.04, 1.0, 0.94))


## The hat sits on a pivot at the crown of the head that lags behind sudden moves and bounces on landings (see `_hat_spring`).
const HAT_PIVOT := Vector3(0, 1.95, 0)
var hat_pivot: Node3D


func _hat() -> void:
	hat_pivot = Node3D.new()
	hat_pivot.position = HAT_PIVOT
	head_rig.add_child(hat_pivot)
	var accent: Color = loadout.accent_color()
	var white := toon(Color(0.97, 0.97, 1.0))
	match loadout.hat:
		1:
			# Sailor cap: white crown and brim with a band in the accent colour.
			_part(hat_pivot, _cyl(0.5, 0.55, 0.32), white, Vector3(0, 2.18, 0) - HAT_PIVOT)
			_part(hat_pivot, _cyl(0.62, 0.62, 0.08), white, Vector3(0, 2.02, 0) - HAT_PIVOT)
			_part(hat_pivot, _cyl(0.5, 0.5, 0.05), toon(accent), Vector3(0, 2.1, 0) - HAT_PIVOT, Vector3(1.04, 1.0, 1.04))
		2:
			# Aviator cap: a leather skullcap with ear flaps, a band round it and goggles pushed up on the forehead.
			var leather := toon(Color(0.55, 0.34, 0.2))
			_part(hat_pivot, _sphere(0.86), leather, Vector3(0, 1.74, -0.06) - HAT_PIVOT, Vector3(1.0, 0.66, 1.0))
			for sx in [-1.0, 1.0]:
				_part(hat_pivot, _sphere(1.0), leather, Vector3(sx * 0.74, 1.5, -0.02) - HAT_PIVOT, Vector3(0.16, 0.34, 0.3))
			var band := TorusMesh.new()
			band.inner_radius = 0.8
			band.outer_radius = 0.88
			band.rings = 40
			_part(hat_pivot, band, toon(Color(0.3, 0.2, 0.14), false), Vector3(0, 1.86, -0.04) - HAT_PIVOT, Vector3(1.0, 0.8, 1.0), Vector3(-14, 0, 0))
			var ring := TorusMesh.new()
			ring.inner_radius = 0.1
			ring.outer_radius = 0.19
			var glass := toon(Color(0.66, 0.88, 1.0), false)
			for sx in [-1.0, 1.0]:
				_part(hat_pivot, ring, toon(Color(0.78, 0.66, 0.42)), Vector3(sx * 0.25, 1.92, 0.74) - HAT_PIVOT, Vector3(1.15, 1.0, 1.15), Vector3(68, 0, 0))
				_part(hat_pivot, _sphere(1.0), glass, Vector3(sx * 0.25, 1.92, 0.74) - HAT_PIVOT, Vector3(0.14, 0.03, 0.14), Vector3(68, 0, 0))
		3:
			# Straw hat (woven, godot/art/cloth/straw.svg) with a band in the accent colour.
			var straw := toon(Color(1, 1, 1), true, SvgArt.texture("res://art/cloth/straw.svg", {"#ff00ff": Color(0.92, 0.8, 0.48)}), Vector2(8, 2))
			_part(hat_pivot, _cyl(1.15, 1.15, 0.07), straw, Vector3(0, 2.05, 0) - HAT_PIVOT)
			_part(hat_pivot, _cyl(0.55, 0.62, 0.35), straw, Vector3(0, 2.25, 0) - HAT_PIVOT)
			_part(hat_pivot, _cyl(0.63, 0.63, 0.08), toon(accent), Vector3(0, 2.14, 0) - HAT_PIVOT)
		4:
			# Beanie with a pompom.
			_part(hat_pivot, _sphere(0.84), toon(accent), Vector3(0, 1.78, 0) - HAT_PIVOT, Vector3(1.0, 0.7, 1.0))
			_part(hat_pivot, _sphere(0.18), white, Vector3(0, 2.38, 0) - HAT_PIVOT)
		5:
			# Crown: a gold band with five points.
			var gold := toon(Color(0.96, 0.8, 0.25))
			_part(hat_pivot, _cyl(0.55, 0.58, 0.2), gold, Vector3(0, 2.1, 0) - HAT_PIVOT)
			for i in 5:
				var a := TAU * i / 5.0
				_part(hat_pivot, _cyl(0.0, 0.11, 0.32), gold, Vector3(sin(a) * 0.5, 2.36, cos(a) * 0.5) - HAT_PIVOT)


func _glasses() -> void:
	var frame := toon(INK, false)
	match loadout.glasses:
		1:
			# Shades: tinted glass the eyes show through, in thin dark rims, with a bridge.
			var lens := StandardMaterial3D.new()
			lens.albedo_color = Color(1.0, 0.55, 0.2, 0.5)
			lens.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
			lens.roughness = 0.45
			lens.metallic_specular = 0.3
			var rim := TorusMesh.new()
			rim.inner_radius = 0.95
			rim.outer_radius = 1.05
			for sx in [-1.0, 1.0]:
				_part(head_rig, _sphere(1.0), lens, Vector3(sx * 0.32, 1.52, 0.84), Vector3(0.25, 0.18, 0.03))
				_part(head_rig, rim, frame, Vector3(sx * 0.32, 1.52, 0.845), Vector3(0.25, 0.04, 0.18), Vector3(90, 0, 0))
			_part(head_rig, _sphere(1.0), frame, Vector3(0, 1.56, 0.86), Vector3(0.1, 0.025, 0.025))
		2:
			# Goggles: chunky rings with a strap round the head.
			var ring := TorusMesh.new()
			ring.inner_radius = 0.14
			ring.outer_radius = 0.28
			for sx in [-1.0, 1.0]:
				_part(head_rig, ring, toon(Color(0.45, 0.28, 0.12)), Vector3(sx * 0.32, 1.52, 0.8), Vector3.ONE, Vector3(90, 0, 0))
				_part(head_rig, _sphere(1.0), toon(Color(0.7, 0.9, 1.0, 1.0), false), Vector3(sx * 0.32, 1.52, 0.82), Vector3(0.14, 0.14, 0.02))
			var strap := TorusMesh.new()
			strap.inner_radius = 0.78
			strap.outer_radius = 0.84
			_part(head_rig, strap, toon(Color(0.3, 0.2, 0.1)), Vector3(0, 1.52, 0), Vector3(1, 0.6, 1))
		3:
			# Round specs: thin rings and a bridge.
			var thin := TorusMesh.new()
			thin.inner_radius = 0.17
			thin.outer_radius = 0.21
			for sx in [-1.0, 1.0]:
				_part(head_rig, thin, frame, Vector3(sx * 0.32, 1.52, 0.8), Vector3.ONE, Vector3(90, 0, 0))
			_part(head_rig, _sphere(1.0), frame, Vector3(0, 1.54, 0.82), Vector3(0.1, 0.02, 0.02))


## The expression for this moment (see Loadout's action faces).
func _face_for(s: Dictionary) -> Dictionary:
	match String(s.state):
		"Hitstun":
			# A wince while frozen by the hit; sent flying (tumbling), dazed.
			if int(s.hitlag) == 0 and s.tumble:
				return Loadout.DAZED_FACE
			return Loadout.HURT
		"ShieldBreak":
			return Loadout.DAZED_FACE
		"Grabbed":
			return Loadout.SHOCK_FACE
		"Knockdown", "Rebound":
			return Loadout.HURT
		"Attack":
			if int(s.charge) > 0:
				return Loadout.EFFORT_FACE
			var t: PackedInt32Array = s.move_timing
			var f: float = s.state_frame
			# The yell starts as the swing comes round and lasts a little past the hit.
			if t[1] > 0 and f >= float(t[1]) * 0.4 and f <= float(t[2]) + 10.0:
				return Loadout.ATTACK_FACE
			return Loadout.FOCUS_FACE
		"Shield", "ShieldDrop", "LedgeHang", "LedgeGetUp", "Grabbing", "WallTech":
			return Loadout.EFFORT_FACE
		"Run", "Dash", "JumpSquat", "LedgeJump", "AirDodge", "Roll", "SpotDodge", "WaveLand", "Turn":
			return Loadout.FOCUS_FACE
	return Loadout.FACES[loadout.face]


## Squash pose per state as a single number: positive squashes down and out, negative stretches up.
const SQUASH := {
	"JumpSquat": 0.32, "Landing": 0.2, "Crouch": 0.28, "WaveLand": 0.24, "Turn": 0.08,
	"Dash": 0.04, "ShieldDrop": 0.0, "Roll": 0.3, "SpotDodge": 0.34, "ShieldBreak": 0.22, "Grabbed": 0.14, "Knockdown": 0.42, "GetUp": 0.2,
}
## Forward lean in degrees per state.
const LEAN := {"Dash": 18.0, "Run": 14.0, "WaveLand": 28.0, "Walk": 4.0, "LedgeAttack": 20.0, "Roll": 24.0, "ShieldBreak": 32.0, "Grabbing": 8.0, "Grabbed": -10.0, "Knockdown": 78.0, "GetUp": 24.0}

# Damped springs make landings and takeoffs read as soft and elastic instead of linear and stiff.
const SPRING_K := 420.0
const SPRING_DAMP := 22.0  # a bit under critical (2*sqrt(K) = 41), so there is a small rebound

var squash := 0.0
var squash_vel := 0.0
var lean_vel := 0.0
var was_grounded := false
var last_vy := 0.0
## The ledge options, shown as moves rather than the simulation's jump from the hanging place to the stage: the body is drawn from where
## it hung toward where the option puts it (up first, then over the edge), while the climb clip pulls it up; a roll travels the whole
## way rolling. `ledge_k` is how far through the travel it is. The ledge grab starts a swing (`ledge_swing_t`).
var hang_from := Vector3.ZERO
var was_hanging := false
var climb_from := Vector3.ZERO
var climbing := false
var ledge_rolling := false
var ledge_k := 1.0
var ledge_offset := Vector3.ZERO
var ledge_swing_t := -1.0
var last_state := ""


func _ledge_motion(s: Dictionary, pos: Vector3) -> void:
	var state: String = s.state
	ledge_offset = Vector3.ZERO
	if state == "LedgeHang":
		if not was_hanging:
			ledge_swing_t = 0.0
		hang_from = pos
		was_hanging = true
		climbing = false
		return
	ledge_swing_t = -1.0
	var getting_up := state == "LedgeGetUp" or state == "LedgeAttack"
	if was_hanging and getting_up:
		climb_from = hang_from - pos
		climbing = true
	was_hanging = false
	if not getting_up:
		climbing = false
		ledge_rolling = false
		ledge_k = 1.0
		return
	if not climbing:
		return
	var total := float(s.get("lag", 34)) if state == "LedgeGetUp" else 55.0
	var f: float = s.state_frame
	ledge_rolling = state == "LedgeGetUp" and absf(climb_from.x) > 2.8
	var travel := total * (0.7 if ledge_rolling else 0.5)
	if state == "LedgeAttack":
		travel = 14.0
	ledge_k = clampf(f / maxf(1.0, travel), 0.0, 1.0)
	var ease := ledge_k * ledge_k * (3.0 - 2.0 * ledge_k)
	var rise := minf(1.0, ledge_k * (3.0 if ledge_rolling else 1.6))
	rise = rise * rise * (3.0 - 2.0 * rise)
	ledge_offset = Vector3(climb_from.x * (1.0 - ease), climb_from.y * (1.0 - rise), 0.0)


## The midair jump's front flip: seconds left of it, and the offset that keeps it turning about the body's middle.
const FLIP_TIME := 0.38
var flip_left := 0.0
var flip_offset := Vector3.ZERO
var prev_jumps := -1
## The attack lunge: the body is thrown forward through the strike and drawn back in the wind-up (model space, along the facing).
var lunge := 0.0
var lunge_vel := 0.0
## The hat's lag (a roll, radians) and bounce (a drop below its place, units), and the velocity they react to.
var hat_roll := 0.0
var hat_roll_vel := 0.0
var hat_drop := 0.0
var hat_drop_vel := 0.0
var prev_vel := Vector2.ZERO
var prev_frame := -1
## Where the run cycle was last frame, for the footfalls.
var last_step_phase := 0.0


## `s` is a dictionary of sim state (see main.gd `_refresh`).
func apply(pos: Vector3, s: Dictionary, delta: float) -> void:
	# The brawler has longer limbs: swap to its rig the first time we see its class.
	if rig != null and (_cls(s) == 1) != long_limbs:
		long_limbs = _cls(s) == 1
		rebuild(loadout)
	position = pos
	var state: String = s.state
	_ledge_motion(s, pos)
	var facing: int = s.facing
	var vy: float = s.vel.y
	var grounded: bool = s.platform >= 0

	# Touching down: kick the squash spring in proportion to how fast we were falling.
	if grounded and not was_grounded:
		squash_vel += clampf(absf(last_vy), 0.0, 0.35) * 55.0
	was_grounded = grounded
	if not grounded:
		last_vy = vy

	var target_squash: float = SQUASH.get(state, 0.0)
	if state == "Attack" and s.move_timing[1] > 0:
		# Coil down while winding up, stretch tall through the strike: the move reads from far away.
		var first: float = s.move_timing[1]
		var last: float = s.move_timing[2]
		var f: float = s.state_frame
		if f < first:
			target_squash = 0.14 * clampf(f / maxf(1.0, first * 0.6), 0.0, 1.0)
		elif f <= last + 2.0:
			target_squash = -0.2
	if state == "Airborne" or state == "Helpless":
		target_squash = -0.1 if vy > 0.06 else (-0.05 if vy < -0.12 else 0.0)
	var target_lean: float = LEAN.get(state, 0.0)
	# The whole body swings with the hit (in the manner of the reference game's big aerials): it pitches into a hit in front or below,
	# arches back under one overhead and curls forward away from one behind, as far as the limb is into its reach.
	if state == "Attack" and s.move_tip != Vector3.ZERO:
		var tip: Vector3 = s.move_tip
		var phi := rad_to_deg(atan2(tip.x, -(tip.y - 1.1)))   # 0 straight down, 90 ahead, 180 overhead, negative behind
		var swing := clampf((130.0 - phi) * 0.32, -22.0, 30.0) if phi >= 0.0 else clampf(-phi * 0.18, 0.0, 22.0)
		target_lean += swing * _reach_weight(s)
	# The attack lunge: drawn back while winding up, thrown forward through the strike.
	var target_lunge := 0.0
	if state == "Attack" and s.move_timing[1] > 0:
		var first: float = s.move_timing[1]
		var last: float = s.move_timing[2]
		var f: float = s.state_frame
		if f < first and f >= first * 0.5:
			target_lunge = -0.12
		elif f >= first and f <= last + 2.0:
			target_lunge = 0.32
	var fast_falling: bool = s.fast_fall and not grounded and (state == "Airborne" or state == "Helpless" or state == "ShieldDrop")
	if fast_falling:
		target_squash = -0.28

	# Fixed sub-steps keep the spring stable at any frame rate.
	var steps := ceili(delta / 0.008)
	var h := delta / maxi(steps, 1)
	for _i in steps:
		squash_vel += (SPRING_K * (target_squash - squash) - SPRING_DAMP * squash_vel) * h
		squash += squash_vel * h
		lean_vel += (SPRING_K * (target_lean - lean) - SPRING_DAMP * lean_vel) * h
		lean += lean_vel * h
		lunge_vel += (SPRING_K * 1.6 * (target_lunge - lunge) - SPRING_DAMP * 1.2 * lunge_vel) * h
		lunge += lunge_vel * h
	squash = clampf(squash, -0.35, 0.55)

	# Running, every footfall squashes the body a little and every stride between them stretches it, in step with the clip.
	var step_squash := 0.0
	if (state == "Run" or state == "Dash") and anim != null and (current_clip == "run" or current_clip == "dash"):
		var length := anim.get_animation(current_clip).length
		var phase := fmod(anim.current_animation_position / maxf(length, 0.001), 1.0)
		step_squash = 0.07 * cos((phase - 0.12) * 4.0 * PI)
		# A puff of dust behind each foot as it lands (at 0.12 and 0.62 of the cycle).
		for land_at in [0.12, 0.62]:
			var crossed: bool = (last_step_phase < land_at and phase >= land_at) or (last_step_phase > phase and (phase >= land_at or last_step_phase < land_at))
			if crossed and grounded and not dusts.is_empty():
				_puff(-float(facing), 1)
		last_step_phase = phase
	# Volume-preserving: squashing down widens the body.
	var total := squash + step_squash
	model.scale = Vector3(1.0 + 0.5 * total, 1.0 - total, 1.0 + 0.5 * total)
	_hat_spring(s, delta)
	# Skidding out of a run, the fighter still faces the way it was running (the simulation has already turned it), leaning back
	# against the slide; it swings round as the skid ends.
	var look := facing
	if state == "Turn" and absf(float(s.vel.x)) > 0.08 and signf(float(s.vel.x)) == -float(facing):
		look = -facing
	yaw = lerp_angle(yaw, float(look) * deg_to_rad(BODY_TURN), clampf(delta * 14.0, 0.0, 1.0))
	# The eyes switch sides as the body swings past facing the camera.
	set_gaze(1 if yaw >= 0.0 else -1)
	# Leaning is a pitch toward the fighter's front (which, turned, is mostly along the stage).
	# A midair jump is a front flip (the body spins once about its middle, curled up; see the `air_jump` clip).
	var jumps := int(s.get("jumps", 0))
	if state == "Airborne" and not grounded and prev_jumps >= 0 and jumps < prev_jumps:
		flip_left = FLIP_TIME
	# A ledge jump flips too.
	if state == "LedgeJump" and last_state != "LedgeJump":
		flip_left = FLIP_TIME
	prev_jumps = jumps
	if grounded or not (state == "Airborne" or state == "LedgeJump"):
		flip_left = 0.0
	flip_left = maxf(0.0, flip_left - delta)
	var flip := 0.0
	if flip_left > 0.0:
		var k := 1.0 - flip_left / FLIP_TIME
		flip = TAU * k * k * (3.0 - 2.0 * k)
	# The ledge: a swing on the arms when it is caught, a forward roll for a ledge roll (the clip curls the body up).
	var pivot := Vector3(0, 1.1, 0)
	if ledge_swing_t >= 0.0 and state == "LedgeHang":
		ledge_swing_t += delta
		flip += deg_to_rad(22.0) * exp(-ledge_swing_t * 3.2) * sin(ledge_swing_t * 10.0)
		pivot = Vector3(0, 2.0, 0)
	if ledge_rolling and state == "LedgeGetUp":
		flip += TAU * ledge_k
		pivot = Vector3(0, 0.6, 0)
	model.rotation = Vector3(deg_to_rad(lean) + flip, yaw, 0)
	# Spin about the middle of the body (or the hands, or the curled-up roll) rather than the feet.
	flip_offset = Vector3.ZERO
	if flip != 0.0:
		flip_offset = pivot - Basis.from_euler(model.rotation) * pivot
	flip_offset += ledge_offset
	last_state = state
	stage_frame.rotation = Vector3(0, -yaw, 0)

	# Air dodge and ledge invincibility read as ghostly (a long directional dodge only while it is still intangible).
	var ghost := 0.0
	if state == "AirDodge" and s.invuln > 0:
		ghost = 0.55
	elif s.invuln > 0:
		ghost = 0.5 if (s.frame / 4) % 2 == 0 else 0.15
	elif s.ledge_invuln > 0:
		ghost = 0.45 if (s.frame / 3) % 2 == 0 else 0.1
	if ghost != last_ghost:
		last_ghost = ghost
		for m in meshes:
			m.transparency = ghost
	shield.visible = state == "Shield" or state == "ShieldDrop"
	if shield.visible:
		# The bubble shrinks and turns from blue through yellow to red as its health runs out.
		var hp: float = clampf(s.shield, 0.0, 1.0)
		shield.scale = Vector3.ONE * (0.45 + 0.55 * hp)
		var col := Color(0.4, 0.7, 1.0, 0.35).lerp(Color(1.0, 0.35, 0.25, 0.45), 1.0 - hp)
		(shield.material_override as ShaderMaterial).set_shader_parameter("tint", Color(col.r, col.g, col.b, 1.0))
	speed_lines.visible = fast_falling
	_animate(s, delta)
	_apply_combat(s, delta)
	model.position += flip_offset
	_blink(delta)


var last_ghost := 0.0
var speed_lines: Node3D
## Shows every transient effect (ghost fade, shield bubble) so their shaders compile before play,
## avoiding a hitch the first time they appear in a match.
func prewarm(on: bool) -> void:
	for m in meshes:
		m.transparency = 0.5 if on else 0.0
	shield.visible = on


# ---- Combat visuals --------------------------------------------------------------------------------

# The weapon is posed from the move data, so it always points at the hitbox that actually hits:
# the blade runs from the hand to the move's sweet spot, and its length is that distance. Claws are
# short and the sword long because their hitboxes are. At rest it is held up and ready (never
# pointing at the floor), it winds up behind the swing, snaps through by the first active frame, holds
# while the hitbox is live, and recovers.
const HAND := Vector2(0.55, 0.95)         # hand position in the fighter's forward space (x forward, y up)
const REST_ANGLE := 68.0                  # degrees: blade held up and slightly forward
const GROUND_CLEARANCE := 0.06            # the blade tip stays at least this far above the floor
const MESH_LENGTH := 2.7                  # length of the blade mesh before scaling

var percent_label: Label3D
var name_label: Label3D
var name_text := ""


## Shows a name above the fighter (empty hides it). Rebuilding the view keeps it.
func set_name_tag(text: String) -> void:
	name_text = text
	if name_label != null:
		name_label.text = text
		name_label.visible = text != ""
var blade_pivot: Node3D
var blade_parts: Array[MeshInstance3D] = []
var hammer_parts: Array[MeshInstance3D] = []


## The fighter's class (which moveset it has): 0 longsword, 1 claws, 2 maul. Snapshots made before classes existed only have `char`.
func _cls(s: Dictionary) -> int:
	return int(s.get("class", s.char))
var spark: MeshInstance3D
var spark_was_visible := false
var spark_spin := 0.0
var spark_strength := 0


## An eight-pointed star (white in the middle) for hit sparks, flat and facing the camera.
static func _star_mesh() -> ArrayMesh:
	var verts := PackedVector3Array()
	var cols := PackedColorArray()
	var points := 8
	for k in points * 2:
		var a0 := TAU * float(k) / float(points * 2)
		var a1 := TAU * float(k + 1) / float(points * 2)
		var r0 := 1.0 if k % 2 == 0 else 0.38
		var r1 := 1.0 if (k + 1) % 2 == 0 else 0.38
		verts.append(Vector3.ZERO)
		cols.append(Color(1, 1, 1, 1))
		verts.append(Vector3(cos(a0) * r0, sin(a0) * r0, 0))
		cols.append(Color(1, 1, 1, 0.0) if r0 > 0.5 else Color(1, 1, 1, 0.8))
		verts.append(Vector3(cos(a1) * r1, sin(a1) * r1, 0))
		cols.append(Color(1, 1, 1, 0.0) if r1 > 0.5 else Color(1, 1, 1, 0.8))
	var arrays := []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = verts
	arrays[Mesh.ARRAY_COLOR] = cols
	var m := ArrayMesh.new()
	m.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	return m
var flame: Node3D  # Particles.Burst
## One clip per move where there is one (the rest share a clip by move type, below in `_attack_clip`).
const SWORD_CLIPS := {"dair": "sword_dair", "ftilt": "sword_ftilt", "utilt": "sword_utilt", "dtilt": "sword_dtilt", "fsmash": "sword_fsmash",
	"usmash": "sword_usmash", "dsmash": "sword_dsmash"}
const KICK_CLIPS := {"nair": "kick_nair", "bair": "kick_bair", "uair": "kick_uair", "dair": "kick_dair", "utilt": "kick_up",
	"dtilt": "kick_low", "dash attack": "kick_dash", "usmash": "kick_uair"}
## How far the kicking leg is pulled toward the hitbox (eases in and out like the arm).
var leg_k := 0.0

## The brawler fights with feet and body, not a blade: these moves draw no weapon.
## Weapon swings that turn one way through several hitboxes: the wind-up starts this many degrees back round from the first one (positive
## is counter-clockwise when facing right), so the blade comes from the right side. Cuts that come down (jab, forward smash, forward air,
## down air, the back air's rise behind) wind up from above; rising cuts (the backhand, forward tilt, dash attack, neutral air, up air)
## from below.
const SWING_FROM := {"jab": 70.0, "jab 2": -60.0, "ftilt": -70.0, "dash attack": -70.0, "fsmash": 80.0, "fair": 75.0, "dair": 75.0,
	"nair": -60.0, "uair": -60.0, "bair": 60.0}
## Kicks that send both legs out (front and back), each aimed by the game.
const SPLIT_KICKS := ["nair"]
const BRAWLER_NO_BLADE := ["utilt", "dtilt", "dash attack", "nair", "bair", "dair", "uair", "usmash", "side special", "up special", "down special", "grab", "dash grab", "pummel", "forward throw", "back throw", "up throw", "down throw"]
## Moves that rush the whole body forward in a flame.
const BRAWLER_FLAME := ["side special", "up special"]
var last_percent := -1
var blade_angle := REST_ANGLE
var blade_length := 2.0


func _rest_length(s: Dictionary) -> float:
	# Swords are long; claws are short.
	return clampf(float(s.reach) - HAND.x - 0.65, 0.7, 3.4) if _cls(s) != 1 else 1.0


## Where the blade should point, in degrees in forward space, and how long it should be, for this frame.
func _blade_target(s: Dictionary) -> Array:
	var rest_len := _rest_length(s)
	var name: String = s.move_name
	if name == "":
		return [REST_ANGLE, rest_len, HAND.x]
	var tip: Vector3 = s.move_tip
	var hand_x := HAND.x if tip.x >= 0.0 else -0.35
	var to_tip := Vector2(tip.x - hand_x, tip.y - HAND.y)
	var swing := rad_to_deg(atan2(to_tip.y, to_tip.x))
	var length := clampf(to_tip.length() + tip.z * 0.7, 0.7, 3.6)
	if _cls(s) == 1:
		length = clampf(length, 0.7, 1.4)
	# Never point into the floor while standing on it.
	if s.platform >= 0:
		var lowest := asin(clampf((GROUND_CLEARANCE - HAND.y) / length, -1.0, 1.0))
		var behind := to_tip.x < 0.0
		if sin(deg_to_rad(swing)) < sin(lowest) and not behind:
			swing = rad_to_deg(lowest)
	# Wind-up comes from the opposite side of the swing (or from where a move says its swing starts: see SWING_FROM).
	var wind := swing + 100.0
	if sin(deg_to_rad(swing)) > 0.55 or cos(deg_to_rad(swing)) < 0.0:
		wind = swing - 100.0
	if SWING_FROM.has(name):
		wind = swing + float(SWING_FROM[name])
	var t: PackedInt32Array = s.move_timing  # total, first active, last active
	var f: float = s.state_frame
	var start := maxf(1.0, float(t[1]))
	var swing_from := start * 0.5
	var angle := swing
	var len := length
	if f < swing_from:
		var k := clampf(f / maxf(1.0, swing_from), 0.0, 1.0)
		angle = lerpf(REST_ANGLE, wind, k)
		len = lerpf(rest_len, length, k)
	elif f < start:
		var k := clampf((f - swing_from) / maxf(1.0, start - swing_from), 0.0, 1.0)
		angle = lerpf(wind, swing, k)
	elif f > t[2]:
		# The blade holds its follow-through and comes back late in the end lag.
		var k := pow(clampf((f - t[2]) / maxf(1.0, t[0] - t[2]), 0.0, 1.0), 1.8)
		angle = lerpf(swing, REST_ANGLE, k)
		len = lerpf(length, rest_len, k)
	return [angle, len, hand_x]


## The weapon arm's shoulder in forward space (x forward, y up), and how far the hand reaches from it in a swing.
const SHOULDER := Vector2(0.5, 1.15)
const ARM_REACH := 0.5
var arm_k := 0.0
var hand_target := Vector3.ZERO


func _pose_blade(s: Dictionary, delta: float) -> void:
	var target: Array = _blade_target(s)
	# A little smoothing so the rest pose and quick direction changes do not pop.
	var follow := clampf(delta * 30.0, 0.0, 1.0)
	if s.move_name != "" and s.state_frame > 0:
		follow = 1.0  # while attacking, follow the animation exactly so it matches the hitbox
	blade_angle = lerp_angle(deg_to_rad(blade_angle), deg_to_rad(float(target[0])), follow)
	blade_angle = rad_to_deg(blade_angle)
	blade_length = lerpf(blade_length, float(target[1]), follow)
	var facing: int = s.facing
	# Where the blade's tip is (forward space). It is fixed by the move's hitboxes and never moves because of the arm.
	var old_hand := Vector2(float(target[2]), HAND.y)
	var tip := old_hand + Vector2.from_angle(deg_to_rad(blade_angle)) * blade_length
	# In a swing the hand leaves its resting spot and sweeps round the shoulder, so the arm throws the blade through its arc; the blade
	# then runs from the hand to the same tip.
	var attacking: bool = s.move_name != "" and s.state_frame > 0
	arm_k = move_toward(arm_k, 1.0 if attacking else 0.0, delta * 9.0)
	# A punch reaches out with the wind-up and comes home in the recovery, like a kick (a blade stays in the hand throughout).
	if attacking and _cls(s) == 1:
		arm_k = _reach_weight(s)
	var hand := old_hand
	if rig != null and arm_k > 0.0:
		var to_tip := tip - shoulder
		var dist := maxf(to_tip.length(), 0.001)
		var reach := minf(arm_reach, maxf(dist - 0.3, 0.1)) if _cls(s) != 1 else arm_reach
		hand = old_hand.lerp(shoulder + to_tip / dist * reach, arm_k)
	var along := tip - hand
	var length := maxf(along.length(), 0.3)
	var swing := rad_to_deg(along.angle())
	# Forward space to model space: facing left mirrors the pose about the vertical axis.
	var angle := swing if facing > 0 else 180.0 - swing
	hand_target = Vector3(hand.x * facing, hand.y, 0.35)
	# Trail: the hitbox's centre (the blade tip is the centre plus most of the hitbox radius along the blade).
	var radius: float = s.move_tip.z if s.move_tip != Vector3.ZERO else 0.4
	if _cls(s) == 1:
		_update_trail(s, tip - Vector2.from_angle(deg_to_rad(blade_angle)) * radius * 0.7, radius)
	else:
		_update_sweep(s, hand, tip)
	blade_pivot.position = hand_target
	blade_pivot.rotation = Vector3(0, 0, deg_to_rad(angle))
	var hammer: bool = _cls(s) == 2
	var thick := 1.0 if _cls(s) == 0 else 1.7
	blade_pivot.scale = Vector3(length / MESH_LENGTH, thick, thick)
	if hammer:
		# The head keeps its size: only the shaft stretches, and the head sits at the move's sweet spot.
		blade_pivot.scale = Vector3.ONE
		var reach := maxf(length, 1.3)
		hammer_parts[0].scale.x = (reach - 0.35) / 2.5
		hammer_parts[0].position.x = (reach - 0.35) / 2.0
		hammer_parts[1].position.x = reach
		hammer_parts[2].position.x = reach
	for part in blade_parts:
		part.visible = not hammer
	for part in hammer_parts:
		part.visible = hammer


## How far into reaching for its hit a limb should be at this point of the move: easing in over the second half of the wind-up, fully
## there while the hit is live, and easing back out over the first half of the recovery.
func _reach_weight(s: Dictionary) -> float:
	var t: PackedInt32Array = s.move_timing
	var f: float = s.state_frame
	var first := maxf(1.0, float(t[1]))
	var last := maxf(first, float(t[2]))
	var total := maxf(last + 1.0, float(t[0]))
	var k := 1.0
	if f < first:
		k = clampf((f - first * 0.35) / maxf(1.0, first * 0.65), 0.0, 1.0)
	elif f > last:
		# (It stays out through most of the end lag, coming home late.)
		k = 1.0 - pow(clampf((f - last) / maxf(1.0, (total - last) * 0.85), 0.0, 1.0), 1.6)
	return k * k * (3.0 - 2.0 * k)


## Puts the weapon in the weapon hand as the clip moves it, swept back and up behind the runner (carried in a run).
const CARRY_ANGLE := 158.0


func _carry_weapon(facing: int) -> void:
	var hand := skeleton.find_bone("hand.R" if facing > 0 else "hand.L")
	if hand < 0:
		return
	blade_pivot.position = _skeleton_to_frame() * skeleton.get_bone_global_pose(hand).origin
	var angle := CARRY_ANGLE if facing > 0 else 180.0 - CARRY_ANGLE
	blade_pivot.rotation = Vector3(0, 0, deg_to_rad(angle))


## The hat's secondary motion: it lags behind a sudden change of speed sideways and dips and springs back on a landing or a jump.
func _hat_spring(s: Dictionary, delta: float) -> void:
	if hat_pivot == null:
		return
	var vel := Vector2(float(s.vel.x), float(s.vel.y))
	var frame: int = s.frame
	if frame != prev_frame:
		if prev_frame >= 0 and frame - prev_frame <= 2:
			var dv := vel - prev_vel
			hat_roll_vel += clampf(dv.x, -0.5, 0.5) * 60.0
			hat_drop_vel += clampf(dv.y, -1.0, 1.0) * 6.0
		prev_frame = frame
		prev_vel = vel
	var steps := ceili(delta / 0.008)
	var h := delta / maxi(steps, 1)
	for _i in steps:
		hat_roll_vel += (-300.0 * hat_roll - 14.0 * hat_roll_vel) * h
		hat_roll += hat_roll_vel * h
		hat_drop_vel += (-500.0 * hat_drop - 18.0 * hat_drop_vel) * h
		hat_drop += hat_drop_vel * h
	hat_roll = clampf(hat_roll, -0.5, 0.5)
	hat_drop = clampf(hat_drop, -0.12, 0.15)
	hat_pivot.rotation = Vector3(0, 0, hat_roll)
	hat_pivot.position = HAT_PIVOT + Vector3(0, -hat_drop, 0)


## Two-bone arm IK: the weapon arm reaches `hand_target` (model space). Returns nothing; with `holding` false the arm goes back to the clip.
func _aim_arm(facing: int, holding: bool) -> void:
	if skeleton == null:
		return
	for side in ["L", "R"]:
		var mine: bool = (side == "R") == (facing > 0)
		var iu := skeleton.find_bone("armU." + side)
		var il := skeleton.find_bone("armL." + side)
		var ih := skeleton.find_bone("hand." + side)
		if not (mine and holding):
			skeleton.set_bone_global_pose_override(iu, Transform3D(), 0.0, false)
			skeleton.set_bone_global_pose_override(il, Transform3D(), 0.0, false)
			skeleton.set_bone_global_pose_override(ih, Transform3D(), 0.0, false)
			continue
		var rest_u := skeleton.get_bone_global_rest(iu)
		var rest_l := skeleton.get_bone_global_rest(il)
		var rest_h := skeleton.get_bone_global_rest(ih)
		var elbow0 := rest_l.origin
		var wrist0 := rest_h.origin
		var upper := (elbow0 - rest_u.origin).length()
		# The shoulder where the animated chest has it now, so the arm stays on the body as it bends and turns.
		var shoulder := rest_u.origin
		var parent := skeleton.get_bone_parent(iu)
		if parent >= 0:
			shoulder = skeleton.get_bone_global_pose(parent) * skeleton.get_bone_rest(iu).origin
		var lower := (wrist0 - elbow0).length() + 0.07
		var to_skel := _skeleton_to_frame().affine_inverse()
		var goal: Vector3 = to_skel * hand_target
		var d := goal - shoulder
		var dist := clampf(d.length(), 0.05, upper + lower - 0.002)
		var dir := d.normalized()
		var cos_a := clampf((upper * upper + dist * dist - lower * lower) / (2.0 * upper * dist), -1.0, 1.0)
		var ang := acos(cos_a)
		# The elbow bends down and out and a little back.
		var pole := Vector3(0.7 if side == "R" else -0.7, -1.0, -0.5)
		var bend := (pole - dir * pole.dot(dir)).normalized()
		var elbow := shoulder + dir * (cos(ang) * upper) + bend * (sin(ang) * upper)
		var fore := (goal - elbow).normalized()
		var q_upper := Quaternion((elbow0 - shoulder).normalized(), (elbow - shoulder).normalized())
		var q_lower := Quaternion((wrist0 - elbow0).normalized(), fore)
		skeleton.set_bone_global_pose_override(iu, Transform3D(Basis(q_upper) * rest_u.basis, shoulder), 1.0, true)
		skeleton.set_bone_global_pose_override(il, Transform3D(Basis(q_lower) * rest_l.basis, elbow), 1.0, true)
		skeleton.set_bone_global_pose_override(ih, Transform3D(Basis(q_lower) * rest_h.basis, elbow + fore * (wrist0 - elbow0).length()), 1.0, true)


## A kick: the leg on the side of the hitbox reaches the live hitbox with two-bone IK (hip, knee, ankle), so the foot is where the hit
## is; the other leg and the body come from the move's clip.
func _aim_leg(s: Dictionary, kicking: bool, delta: float) -> void:
	if skeleton == null:
		return
	var tip: Vector3 = s.move_tip
	var live := kicking and tip != Vector3.ZERO
	# The leg reaches for the hit as the wind-up builds, is on it while it is live, and comes home through the recovery.
	leg_k = _reach_weight(s) if live else move_toward(leg_k, 0.0, delta * 12.0)
	var facing := int(s.facing)
	# The front leg kicks forward and up, the back leg backward.
	# On the turned body the model's left leg is the one nearest the camera when facing right (and the right one facing left): it is the
	# front leg, so a kick in front is in plain view and a kick behind uses the far leg.
	var front := "L" if facing > 0 else "R"
	var back := "R" if facing > 0 else "L"
	var kicker := front if tip.x >= 0.0 else back
	# Where each kicking leg's foot goes (forward space). A split kick (the neutral air) kicks both legs out, one ahead and one behind.
	var goals := {kicker: tip}
	if SPLIT_KICKS.has(String(s.move_name)):
		goals = {front: Vector3(0.95, 0.85, 0.0), back: Vector3(-0.85, 0.85, 0.0)}
	for side in ["L", "R"]:
		var it := skeleton.find_bone("thigh." + side)
		var ish := skeleton.find_bone("shin." + side)
		var ift := skeleton.find_bone("foot." + side)
		if not goals.has(side) or leg_k <= 0.01:
			skeleton.set_bone_global_pose_override(it, Transform3D(), 0.0, false)
			skeleton.set_bone_global_pose_override(ish, Transform3D(), 0.0, false)
			skeleton.set_bone_global_pose_override(ift, Transform3D(), 0.0, false)
			continue
		var rest_t := skeleton.get_bone_global_rest(it)
		var rest_s := skeleton.get_bone_global_rest(ish)
		var rest_f := skeleton.get_bone_global_rest(ift)
		var knee0 := rest_s.origin
		var ankle0 := rest_f.origin
		var upper := (knee0 - rest_t.origin).length()
		var lower := (ankle0 - knee0).length() + 0.08
		# The hip joint where the animated pelvis has it now (not where it rests), so the leg stays on the body as it moves.
		var hips := skeleton.find_bone("hips")
		var hip := rest_t.origin
		if hips >= 0:
			hip = skeleton.get_bone_global_pose(hips) * skeleton.get_bone_rest(it).origin
		var to_skel := _skeleton_to_frame().affine_inverse()
		# The hitbox centre in model space; the foot reaches it (or as near as the leg allows).
		var aim: Vector3 = goals[side]
		# (A little toward the camera, so on the turned body the kicking leg passes in front of the torso instead of through it.)
		var goal: Vector3 = to_skel * Vector3(aim.x * float(facing), aim.y, 0.75)
		var d := goal - hip
		var dist := clampf(d.length(), 0.05, upper + lower - 0.002)
		var dir := d.normalized()
		var cos_a := clampf((upper * upper + dist * dist - lower * lower) / (2.0 * upper * dist), -1.0, 1.0)
		var ang := acos(cos_a)
		# The knee bends the way a knee does: it points ahead of the leg in the plane of the kick (up for a kick in front, down and back
		# for one behind), a little out toward the camera, so it never folds backwards.
		var dir_m: Vector3 = (_skeleton_to_frame().basis * dir).normalized()
		var sin_phi := dir_m.x * float(facing)
		var cos_phi := -dir_m.y
		var pole_m := Vector3(float(facing) * cos_phi, sin_phi, 0.3)
		var pole: Vector3 = (to_skel.basis * pole_m).normalized()
		var bend := (pole - dir * pole.dot(dir)).normalized()
		var knee := hip + dir * (cos(ang) * upper) + bend * (sin(ang) * upper)
		var shin_dir := (goal - knee).normalized()
		var q_upper := Quaternion((knee0 - hip).normalized(), (knee - hip).normalized())
		var q_lower := Quaternion((ankle0 - knee0).normalized(), shin_dir)
		skeleton.set_bone_global_pose_override(it, Transform3D(Basis(q_upper) * rest_t.basis, hip), leg_k, true)
		skeleton.set_bone_global_pose_override(ish, Transform3D(Basis(q_lower) * rest_s.basis, knee), leg_k, true)
		skeleton.set_bone_global_pose_override(ift, Transform3D(Basis(q_lower) * rest_f.basis, knee + shin_dir * (ankle0 - knee0).length()), leg_k, true)


## A smear behind the hitbox, like the one a fast punch or kick leaves. Each simulation frame the centre of the hitbox (the fist or the
## foot, which the arm and leg reach for) is added to a path; the path is drawn as a smooth tapering smear, thickest at the hitbox: a
## bright core in the trail colour with a thin deeper rim, so it reads as a drawn stroke. The points are kept in world space, so the smear
## stays where the swing was while the fighter moves on. It is cosmetic: it only reads the move.
const TRAIL_FRAMES := 10
const TRAIL_WIDTH := 0.62
const TRAIL_SMOOTH := 4
var trail: MeshInstance3D
var last_trail_centre := Vector2(INF, INF)
var trail_points: Array = []     # [{pos: Vector3, frame: int}]
var last_trail_frame := -1


func _make_trail() -> void:
	trail = MeshInstance3D.new()
	trail.mesh = ImmediateMesh.new()
	trail.top_level = true
	trail.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	trail.extra_cull_margin = 16384.0
	trail.material_override = FxMaterial.get_material(true, 5)
	add_child(trail)


# ---- Weapon swing trail ------------------------------------------------------------------------------------------------------------
# A sword (or maul) leaves the trail the reference game draws: the whole crescent the blade sweeps through, a bright core along the path
# of the tip with a crisp deeper rim, the trail colour inside it fading toward the hilt. As it ages the crescent is eaten away from the
# hilt side and thins to a point behind the swing. Each simulation frame records where the hand and the tip are; the crescent between
# two records is drawn in thin slices that turn round the hand, so it is a smooth arc rather than a polygon. Time stands still in hitlag,
# like the swing itself.

const SWEEP_LIFE := 7     # frames a slice of the swing stays visible
const SWEEP_STEPS := 8    # slices drawn between two simulation frames
var sweep: MeshInstance3D
var sweep_samples: Array = []   # [{hand: Vector2, angle: float, length: float, z: float, at: int}], world space
var sweep_clock := 0
var sweep_seen_frame := -1


func _update_sweep(s: Dictionary, hand: Vector2, tip: Vector2) -> void:
	if sweep == null:
		sweep = MeshInstance3D.new()
		sweep.mesh = ImmediateMesh.new()
		sweep.top_level = true
		sweep.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		sweep.extra_cull_margin = 16384.0
		sweep.material_override = FxMaterial.get_material(false, 5)
		add_child(sweep)
	var frame: int = s.frame
	if frame < sweep_seen_frame:
		# Rewound (a replay seek): start clean.
		sweep_samples.clear()
	if frame != sweep_seen_frame:
		sweep_seen_frame = frame
		if int(s.hitlag) == 0:
			sweep_clock += 1
			var t: PackedInt32Array = s.move_timing
			var f: float = s.state_frame
			# From the snap through, a few frames before the first hit, so the arc is already drawn when the hit lands and the freeze
			# holds it.
			var swinging: bool = s.state == "Attack" and s.move_name != "" and t[1] > 0 and f >= float(t[1]) - 3.0 \
					and f <= float(t[2]) + 1.0 and blade_pivot.visible
			if swinging:
				var facing := float(s.facing)
				var g := stage_frame.global_transform
				var hw: Vector3 = g * Vector3(hand.x * facing, hand.y, 0.35)
				var tw: Vector3 = g * Vector3(tip.x * facing, tip.y, 0.35)
				var d := Vector2(tw.x - hw.x, tw.y - hw.y)
				sweep_samples.append({"hand": Vector2(hw.x, hw.y), "angle": d.angle(), "length": d.length(), "z": hw.z + 0.2,
					"at": sweep_clock})
	while sweep_samples.size() > 0 and sweep_clock - int(sweep_samples[0].at) > SWEEP_LIFE:
		sweep_samples.pop_front()
	var im: ImmediateMesh = sweep.mesh
	im.clear_surfaces()
	# The crescent always reaches the blade where it is now, so a swing frozen by a hit shows its arc right up to the blade.
	var drawn := sweep_samples.duplicate()
	var t2: PackedInt32Array = s.move_timing
	if not drawn.is_empty() and s.state == "Attack" and t2[1] > 0 and float(s.state_frame) <= float(t2[2]) + 1.0 and blade_pivot.visible:
		var g := stage_frame.global_transform
		var facing := float(s.facing)
		var hw: Vector3 = g * Vector3(hand.x * facing, hand.y, 0.35)
		var tw: Vector3 = g * Vector3(tip.x * facing, tip.y, 0.35)
		var d := Vector2(tw.x - hw.x, tw.y - hw.y)
		drawn.append({"hand": Vector2(hw.x, hw.y), "angle": d.angle(), "length": d.length(), "z": hw.z + 0.2, "at": sweep_clock})
	if drawn.size() < 2:
		return
	var colours := _trail_colours(s)
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
	for i in drawn.size() - 1:
		for k in SWEEP_STEPS:
			_sweep_slice(im, drawn[i], drawn[i + 1], float(k) / SWEEP_STEPS, float(k + 1) / SWEEP_STEPS, colours)
	im.surface_end()


## One slice of the crescent, between `u0` and `u1` of the way from record `a` to record `b`. Across the blade, from the hilt out: clear,
## the trail colour, the bright core along the tip's path, and a thin deeper rim. An older slice starts further out (the crescent is eaten
## from the hilt side) and fades.
func _sweep_slice(im: ImmediateMesh, a: Dictionary, b: Dictionary, u0: float, u1: float, colours: Array) -> void:
	var main: Color = colours[0]
	var core: Color = colours[1]
	var rim: Color = main.darkened(0.45)
	var edge := []
	for u in [u0, u1]:
		var hand: Vector2 = (a.hand as Vector2).lerp(b.hand, u)
		var ang := lerp_angle(float(a.angle), float(b.angle), u)
		var length := lerpf(float(a.length), float(b.length), u)
		var at := lerpf(float(a.at), float(b.at), u)
		var age := clampf(float(sweep_clock - at) / float(SWEEP_LIFE), 0.0, 1.0)
		var live := 1.0 - age
		var bright := sqrt(live)
		var inner := lerpf(0.3, 0.9, pow(age, 0.8))
		var rings := [inner, lerpf(inner, 0.9, 0.5), 0.9, 0.985, 1.0, 1.045]
		var cols := [Color(main, 0.0), Color(main, 0.85 * live), Color(core, 0.95 * bright), Color(core, bright), Color(rim, 0.95 * bright),
			Color(rim, 0.9 * bright)]
		var z := lerpf(float(a.z), float(b.z), u)
		var dir := Vector2.from_angle(ang)
		var column := []
		for r in rings.size():
			var p: Vector2 = hand + dir * length * float(rings[r])
			column.append([Vector3(p.x, p.y, z), cols[r]])
		edge.append(column)
	for r in (edge[0] as Array).size() - 1:
		var q := [edge[0][r], edge[0][r + 1], edge[1][r + 1], edge[1][r]]
		for idx in [0, 1, 2, 0, 2, 3]:
			im.surface_set_color(q[idx][1])
			im.surface_add_vertex(q[idx][0])


# ---- Big moments: dizzy stars, the launch stretch and speed lines --------------------------------------------------------------------

## Three cartoon stars circling the head while the shield is broken.
var dizzy: Node3D
## A strong launch: the body stretched along the way it flies, and streaks trailing behind it.
var launch_frame: Node3D
var launch_lines: Node3D
const LAUNCH_STRETCH_FROM := 0.35   # world units a frame: slower launches are not stretched
const LAUNCH_LINES_FROM := 0.5


## A five-pointed star of radius `r`: an ink outline behind a yellow star with a pale middle, in one mesh (vertex colours).
static func _cartoon_star(r: float) -> ArrayMesh:
	var verts := PackedVector3Array()
	var cols := PackedColorArray()
	for layer in [[1.28, Color(0.12, 0.07, 0.12), -0.01], [1.0, Color(1.0, 0.82, 0.22), 0.0], [0.45, Color(1.0, 0.97, 0.8), 0.01]]:
		var k: float = layer[0]
		for i in 10:
			var a0 := PI * 0.5 + TAU * float(i) / 10.0
			var a1 := PI * 0.5 + TAU * float(i + 1) / 10.0
			var r0 := r * k * (1.0 if i % 2 == 0 else 0.45)
			var r1 := r * k * (1.0 if (i + 1) % 2 == 0 else 0.45)
			for v in [Vector3(0, 0, layer[2]), Vector3(cos(a0) * r0, sin(a0) * r0, layer[2]), Vector3(cos(a1) * r1, sin(a1) * r1, layer[2])]:
				verts.append(v)
				cols.append(layer[1])
	var arrays := []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = verts
	arrays[Mesh.ARRAY_COLOR] = cols
	var m := ArrayMesh.new()
	m.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	return m


func _dizzy(s: Dictionary) -> void:
	var on: bool = String(s.state) == "ShieldBreak"
	if not on:
		if dizzy != null:
			dizzy.visible = false
		return
	if dizzy == null:
		dizzy = Node3D.new()
		var star := _cartoon_star(0.17)
		for k in 3:
			var mi := MeshInstance3D.new()
			mi.mesh = star
			mi.material_override = FxMaterial.get_material(false, 3, null, 0.0)
			mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
			dizzy.add_child(mi)
		add_child(dizzy)
	dizzy.visible = true
	dizzy.position = Vector3(0, 2.5, 0)
	# Round the head (in front of it on the near side, behind it on the far side), bobbing, each spinning.
	var t := float(s.frame) / 60.0 * 4.5
	for k in 3:
		var a := t + TAU * float(k) / 3.0
		var star_node := dizzy.get_child(k) as Node3D
		star_node.position = Vector3(cos(a) * 0.65, sin(a * 2.0) * 0.06, sin(a) * 0.5)
		star_node.rotation.z = t * 2.0 + float(k)


## A strong launch reads in the body and the air round it: the body is stretched along its flight (up to 45% longer, as thin as it is
## long, about its middle) and streaks trail behind it, on twos. Back to normal the moment the flight slows.
func _launch_look(s: Dictionary) -> void:
	var vel := Vector2(float(s.vel.x), float(s.vel.y))
	var speed := vel.length()
	var flying: bool = String(s.state) == "Hitstun" and int(s.hitlag) == 0
	var k := clampf((speed - LAUNCH_STRETCH_FROM) / 0.8, 0.0, 0.45) if flying else 0.0
	if k <= 0.0:
		launch_frame.transform = Transform3D.IDENTITY
	else:
		var along := Vector3(vel.x, vel.y, 0).normalized()
		var across := Vector3(-along.y, along.x, 0)
		var stretch := Basis(along, across, Vector3(0, 0, 1)) * Basis.from_scale(Vector3(1.0 + k, 1.0 / sqrt(1.0 + k), 1.0)) 				* Basis(along, across, Vector3(0, 0, 1)).inverse()
		var middle := Vector3(0, 1.1, 0)
		launch_frame.transform = Transform3D(stretch, middle - stretch * middle)
	var lines_on := flying and speed > LAUNCH_LINES_FROM
	if launch_lines == null and lines_on:
		launch_lines = Node3D.new()
		var mat := StandardMaterial3D.new()
		mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		mat.albedo_color = Color(1.0, 1.0, 0.97, 0.75)
		mat.disable_fog = true
		for i in 5:
			var mi := MeshInstance3D.new()
			var q := QuadMesh.new()
			q.size = Vector2(1.0, 0.06)
			mi.mesh = q
			mi.material_override = mat
			mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
			launch_lines.add_child(mi)
		add_child(launch_lines)
	if launch_lines == null:
		return
	launch_lines.visible = lines_on
	if not lines_on:
		return
	# Streaks behind the body along the flight, a little spread, their lengths changing on twos.
	var dir := vel.normalized()
	launch_lines.position = Vector3(0, 1.1, -0.3)
	launch_lines.rotation = Vector3(0, 0, dir.angle())
	var seed_step := int(s.frame) / 2
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_step * 7919 + player
	for i in 5:
		var line := launch_lines.get_child(i) as MeshInstance3D
		var length := rng.randf_range(1.2, 2.6) * clampf(speed / 1.0, 0.6, 1.4)
		line.scale = Vector3(length, 1.0, 1.0)
		line.position = Vector3(-0.9 - length * 0.5 - rng.randf_range(0.0, 0.6), (float(i) - 2.0) * 0.32, 0.0)


# ---- Revival platform ------------------------------------------------------------------------------------------------------------
# After a knock-out the fighter waits on a glowing platform above the stage (see `Respawn` in the sim).

var revival: MeshInstance3D


func _revival_platform(s: Dictionary) -> void:
	var on: bool = s.state == "Respawn"
	if revival == null:
		if not on:
			return
		revival = MeshInstance3D.new()
		var disc := CylinderMesh.new()
		disc.top_radius = 1.0
		disc.bottom_radius = 0.75
		disc.height = 0.16
		revival.mesh = disc
		var mat := StandardMaterial3D.new()
		mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		mat.albedo_color = Color(1.0, 0.93, 0.55, 0.85)
		revival.material_override = mat
		revival.position = Vector3(0, -0.08, 0)
		add_child(revival)
	revival.visible = on
	if on:
		# A gentle pulse, so it reads as temporary.
		var pulse := 0.85 + 0.15 * sin(float(s.frame) * 0.25)
		revival.scale = Vector3(pulse, 1.0, pulse)


# ---- Dust ------------------------------------------------------------------------------------------------------------------------
# Little puffs at the feet when a fighter starts a dash, turns, jumps or lands, as the reference game does: they make movement read.

const DUSTS := 10
var dusts: Array[MeshInstance3D] = []
var dust_age: Array[float] = []
var dust_vel: Array[Vector3] = []
var next_dust := 0
var dust_state := ""
var dust_grounded := true
var dust_frame := -1


func _dust(s: Dictionary) -> void:
	if dusts.is_empty():
		var mat := StandardMaterial3D.new()
		mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		mat.albedo_color = Color(0.96, 0.93, 0.85, 0.7)
		for i in DUSTS:
			var p := MeshInstance3D.new()
			p.mesh = _sphere(0.22)
			p.material_override = mat.duplicate()
			p.top_level = true
			p.visible = false
			add_child(p)
			dusts.append(p)
			dust_age.append(99.0)
			dust_vel.append(Vector3.ZERO)
	var frame: int = s.frame
	if frame == dust_frame:
		return
	var dt := 1.0 / 60.0 * float(clampi(frame - dust_frame, 1, 4)) if dust_frame >= 0 else 1.0 / 60.0
	dust_frame = frame
	for i in DUSTS:
		dust_age[i] += dt
		var life := dust_age[i] / 0.4
		dusts[i].visible = life < 1.0
		if dusts[i].visible:
			dusts[i].global_position += dust_vel[i] * dt
			dusts[i].scale = Vector3.ONE * (0.7 + life * 1.1)
			(dusts[i].material_override as StandardMaterial3D).albedo_color.a = 0.7 * (1.0 - life)
	var state: String = s.state
	var grounded: bool = s.platform >= 0
	var facing := float(s.facing)
	if state != dust_state:
		match state:
			"Dash":
				_puff(-facing, 1)
			"Turn":
				_puff(facing, 1)
			"JumpSquat", "WaveLand":
				_puff(-1.0, 1)
				_puff(1.0, 1)
	# A skid kicks up a stream of dust from the planted heel, ahead of the slide.
	if state == "Turn" and absf(float(s.vel.x)) > 0.08 and frame % 3 == 0:
		_puff(signf(float(s.vel.x)), 1)
	if grounded and not dust_grounded and state != "Knockdown":
		_puff(-1.0, 1)
		_puff(1.0, 1)
	# Rage: from 100% a hurt fighter lets off steam, more of it the higher the damage.
	var pct: float = s.get("percent", 0.0)
	if pct >= 100.0 and state != "Respawn":
		var every := maxi(4, 16 - int((pct - 100.0) / 8.0))
		if frame % every == 0:
			var i := next_dust
			next_dust = (next_dust + 1) % DUSTS
			dusts[i].global_position = global_position + Vector3(randf_range(-0.3, 0.3), 2.1, 0.3)
			dust_vel[i] = Vector3(randf_range(-0.3, 0.3), 1.6, 0.0)
			dust_age[i] = 0.0
			dusts[i].visible = true
	dust_state = state
	dust_grounded = grounded


## A puff of dust at the feet, blowing toward `side` (particles.gd).
func _puff(side: float, count: int) -> void:
	Particles.dust(self, global_position + Vector3(side * 0.3, 0, 0), side, 2 + 2 * count)


# ---- Launch smoke ------------------------------------------------------------------------------------------------------------
# A strong launch leaves a trail of puffs behind the tumbling fighter (as the reference game does), so the eye can follow a big hit.

const PUFFS := 14
var puffs: Array[MeshInstance3D] = []
var puff_age: Array[float] = []
var next_puff := 0
var last_puff_at := Vector3.INF
var last_puff_frame := -1


func _launch_smoke(s: Dictionary) -> void:
	if puffs.is_empty():
		var mat := StandardMaterial3D.new()
		mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		mat.albedo_color = Color(0.92, 0.92, 0.95, 0.55)
		for i in PUFFS:
			var p := MeshInstance3D.new()
			p.mesh = _sphere(0.35)
			p.material_override = mat.duplicate()
			p.top_level = true
			p.visible = false
			add_child(p)
			puffs.append(p)
			puff_age.append(99.0)
	var frame: int = s.frame
	if frame != last_puff_frame:
		var dt := 1.0 / 60.0 * float(maxi(1, frame - last_puff_frame)) if last_puff_frame >= 0 else 1.0 / 60.0
		last_puff_frame = frame
		for i in PUFFS:
			puff_age[i] += dt
			var life := puff_age[i] / 0.55
			puffs[i].visible = life < 1.0
			if puffs[i].visible:
				puffs[i].scale = Vector3.ONE * (0.6 + life * 0.9)
				(puffs[i].material_override as StandardMaterial3D).albedo_color.a = 0.55 * (1.0 - life)
		var here := global_position + Vector3(0, 1.0, 0)
		if s.state == "Hitstun" and int(s.hitlag) == 0 and s.tumble and (last_puff_at == Vector3.INF or here.distance_to(last_puff_at) > 0.45):
			var p := puffs[next_puff]
			next_puff = (next_puff + 1) % PUFFS
			p.global_position = here
			puff_age[puffs.find(p)] = 0.0
			p.visible = true
			last_puff_at = here
		elif s.state != "Hitstun":
			last_puff_at = Vector3.INF


## Edge and core colours: violet and white-pink for the brawler, gold and white for the sword.
func _trail_colours(s: Dictionary) -> Array:
	return trail_colours_of(_cls(s))


## A class's trail colours: [the trail, its bright core] (claws violet, maul ember red, sword blue).
static func trail_colours_of(cls: int) -> Array:
	if cls == 1:
		return [Color(0.78, 0.3, 1.0), Color(1.0, 0.9, 1.0)]
	if cls == 2:
		return [Color(0.95, 0.28, 0.12), Color(1.0, 0.92, 0.8)]
	return [Color(0.3, 0.66, 1.0), Color(0.92, 0.98, 1.0)]


## Catmull-Rom smoothing so a path of a few points reads as a curve.
func _smooth(points: Array) -> Array:
	if points.size() < 3:
		return points
	var out: Array = []
	for i in points.size() - 1:
		var p0: Vector3 = points[maxi(i - 1, 0)]
		var p1: Vector3 = points[i]
		var p2: Vector3 = points[i + 1]
		var p3: Vector3 = points[mini(i + 2, points.size() - 1)]
		for k in TRAIL_SMOOTH:
			var t := float(k) / TRAIL_SMOOTH
			out.append(0.5 * ((2.0 * p1) + (-p0 + p2) * t + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t * t * t))
	out.append(points[points.size() - 1])
	return out


## `centre` is where the hitbox is now (forward space, relative to the fighter) and `radius` its size.
func _update_trail(s: Dictionary, centre: Vector2, _radius: float) -> void:
	if trail == null:
		_make_trail()
	var t: PackedInt32Array = s.move_timing
	var f: float = s.state_frame
	var swinging: bool = s.state == "Attack" and t[1] > 0 and f >= 1.0 and f <= t[2] + 5.0 and s.move_tip != Vector3.ZERO
	var frame: int = s.frame
	var facing: float = float(s.facing)
	var base := Vector3(position.x, position.y, 0.0)
	var here := base + Vector3(centre.x * facing, centre.y, 0.7)
	# Only a hit that travels across the body leaves a trail: one held in place (a lingering kick) would only draw the fighter's own
	# fall or drift as a streak.
	var moved := last_trail_centre.distance_to(centre) > 0.08
	if swinging and int(s.hitlag) == 0 and frame != last_trail_frame and (moved or trail_points.is_empty()):
		last_trail_frame = frame
		last_trail_centre = centre
		trail_points.append({"pos": here, "frame": frame})
	while trail_points.size() > 0 and frame - int(trail_points[0].frame) > TRAIL_FRAMES:
		trail_points.pop_front()
	if frame < last_trail_frame:
		trail_points.clear()
		last_trail_frame = -1
	var im: ImmediateMesh = trail.mesh
	im.clear_surfaces()
	var colours := _trail_colours(s)
	if trail_points.size() >= 2:
		var path: Array = []
		var ages: Array = []
		for p in trail_points:
			path.append(p.pos)
			ages.append(float(frame - int(p.frame)) / float(TRAIL_FRAMES))
		path = _smooth(path)
		# Ages for the smoothed points: interpolate along the path.
		var smooth_ages: Array = []
		for i in path.size():
			var u := float(i) / maxf(1.0, path.size() - 1.0) * (ages.size() - 1.0)
			var lo := int(floor(u))
			var hi := mini(lo + 1, ages.size() - 1)
			smooth_ages.append(lerpf(ages[lo], ages[hi], u - lo))
		_ribbon(im, path, smooth_ages, TRAIL_WIDTH * 1.12, (colours[0] as Color).darkened(0.45), 0.9)
		_ribbon(im, path, smooth_ages, TRAIL_WIDTH, colours[0], 0.9)
		_ribbon(im, path, smooth_ages, TRAIL_WIDTH * 0.45, colours[1], 1.0)


## A strip along `path`, `width` across at the head (the newest end), tapering to a point at the old end and fading only at the very end,
## so the smear keeps a crisp shape.
func _ribbon(im: ImmediateMesh, path: Array, ages: Array, width: float, colour: Color, alpha: float) -> void:
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLE_STRIP)
	for i in path.size():
		var along: Vector3 = (path[mini(i + 1, path.size() - 1)] - path[maxi(i - 1, 0)])
		along.z = 0.0
		var side := Vector3(-along.y, along.x, 0.0).normalized() if along.length() > 0.0001 else Vector3.UP
		var age: float = ages[i]
		# Thin at the old end of the path, full at the head, and thinning with age.
		var along_path := float(i) / maxf(1.0, path.size() - 1.0)
		var strength := pow(clampf(1.0 - age, 0.0, 1.0), 0.7) * sqrt(along_path)
		var half := width * 0.5 * strength
		var c := Color(colour.r, colour.g, colour.b, alpha * clampf(strength * 2.5, 0.0, 1.0))
		im.surface_set_color(c)
		im.surface_add_vertex(path[i] + side * half)
		im.surface_set_color(c)
		im.surface_add_vertex(path[i] - side * half)
	im.surface_end()


func _apply_combat(s: Dictionary, delta: float) -> void:
	var state: String = s.state
	var pct := int(s.percent)
	if pct != last_percent:
		last_percent = pct
		percent_label.text = "%d%%" % pct
		var heat := clampf(pct / 150.0, 0.0, 1.0)
		percent_label.modulate = Color(1.0, 1.0 - 0.75 * heat, 1.0 - 0.95 * heat)
	_pose_blade(s, delta)
	# The brawler's kicks and rushes use the body, not a blade; its rushes burn and Fire Wolf spins.
	var brawler: bool = _cls(s) == 1
	# Only the brawler's body moves leave the arms alone; the sword fighter always has its blade.
	var swinging_arm: bool = not (brawler and BRAWLER_NO_BLADE.has(s.move_name))
	blade_pivot.visible = not brawler and swinging_arm
	# Running with nothing to swing, the weapon arm pumps with the run and the weapon trails behind.
	var carrying: bool = not brawler and s.move_name == "" and (state == "Run" or state == "Dash")
	if rig != null:
		var holding: bool = swinging_arm if not brawler else (swinging_arm and state == "Attack")
		_aim_arm(int(s.facing), holding and not carrying)
		if carrying:
			_carry_weapon(int(s.facing))
		_aim_leg(s, brawler and state == "Attack" and KICK_CLIPS.has(s.move_name), delta)
	var rushing: bool = brawler and state == "Attack" and BRAWLER_FLAME.has(s.move_name) and s.state_frame >= 12
	flame.emitting = rushing
	if brawler and state == "Attack" and s.move_name == "up special" and s.state_frame >= 17:
		model.rotation.x = float(s.frame) * 0.6

	var hitlag: int = s.hitlag
	# The face follows what the fighter is doing (cosmetic): hurt when hit, a yell in an attack, and so on, then its own face again.
	set_expression(_face_for(s))
	# (A hit's burst is drawn where it lands by the match, scripts/effects.gd; the star here only flashes on a rebound.)
	spark.visible = hitlag > 0 and state == "Rebound"
	if spark.visible:
		# A star burst on the side the hit came from, bigger and hotter for a stronger hit, turned a new way for each hit.
		if not spark_was_visible:
			spark_spin = randf() * TAU
			spark_strength = hitlag
		var heat := clampf((spark_strength - 8) / 16.0, 0.0, 1.0)
		var col := Color(1.0, 0.97, 0.75).lerp(Color(1.0, 0.45, 0.15), heat)
		(spark.material_override as StandardMaterial3D).albedo_color = Color(col.r, col.g, col.b, 0.95)
		var grow := 0.55 + 0.07 * spark_strength + 0.12 * sin(float(s.frame) * 1.9)
		spark.scale = Vector3(grow, grow, 1.0)
		spark.rotation.z = spark_spin + float(s.frame) * 0.05
		spark.position = Vector3(0.45 * float(s.facing), 1.25, 0.7)
	spark_was_visible = spark.visible
	# The one who was hit shakes while frozen in hitlag (harder for a stronger hit, settling as it ends); the attacker holds still.
	var shaking: bool = hitlag > 0 and (s.launch_pending or state == "Shield" or state == "Attack")
	if shaking:
		var amp := (0.05 + 0.012 * hitlag) * (0.4 if state == "Shield" else (0.3 if state == "Attack" else 1.0))
		var t := float(s.frame)
		model.position = Vector3(sin(t * 9.0) * amp, cos(t * 7.3) * amp * 0.5, 0)
	else:
		model.position = Vector3(lunge * float(s.facing), 0, 0)
	_flash(s, delta)
	_launch_look(s)
	_dizzy(s)
	_launch_smoke(s)
	_dust(s)
	_revival_platform(s)
	# Charging a smash attack: the glow grows and the body trembles harder the longer it is held.
	var charge: int = s.charge
	if charge > 0 and state == "Attack":
		var amount := clampf(charge / 60.0, 0.0, 1.0)
		spark.visible = (s.frame / 3) % 2 == 0
		spark.scale = Vector3.ONE * (0.4 + 0.9 * amount)
		model.position = Vector3(sin(float(s.frame) * 2.3) * 0.04 * (1.0 + 2.0 * amount) + lunge * float(s.facing), 0, 0)
	if state == "Hitstun" and hitlag == 0:
		if s.tumble:
			model.rotation.x = -float(s.frame) * 0.4
		else:
			model.rotation.x = -deg_to_rad(26.0)
