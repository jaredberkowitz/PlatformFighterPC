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
const TOON_CLOTH_SHADER := preload("res://shaders/toon_cloth.gdshader")
const FACE_SHADER := preload("res://shaders/face.gdshader")
const OUTLINE_SHADER := preload("res://shaders/outline.gdshader")
const SHIELD_SHADER := preload("res://shaders/shield.gdshader")
const LIMB_SHADER := preload("res://shaders/limb.gdshader")
const SKIN_SHADER := preload("res://shaders/skin.gdshader")
const UIDraw := preload("res://ui/ui_draw.gd")
const SvgArt := preload("res://scripts/svg_art.gd")
const FaceLook := preload("res://scripts/face_look.gd")
const Particles := preload("res://scripts/particles.gd")
const FxMaterial := preload("res://scripts/fx_material.gd")
const AnimLibrary := preload("res://scripts/anim_library.gd")
const MotionDeform := preload("res://scripts/motion_deform.gd")
const Smear := preload("res://scripts/smear.gd")
const MotionSprings := preload("res://scripts/motion_springs.gd")
const TailRig := preload("res://scripts/tail_rig.gd")


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
	arc = null
	arc_shown = {}
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
	unkeyed = {}
	neck_pivot = null
	tails = []
	reset_motion()
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
## make_rigged_blob.py writes with it. It is the fighters' body; `--blob` brings back the first, built body.
const RIG_BASE_PATH := "res://models/base_rig.glb"
const BASE_INFO_PATH := "res://models/base_rig.json"
static var use_base_body: bool = not OS.get_cmdline_user_args().has("--blob")
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
	_cloth_cache.clear()
	_respond_cache.clear()
	_blade_cache = null
	_mesh_cache.clear()
	_outline_mat = null
	_cloth_outline = null
	SvgArt.release()
	Particles.release()
	_shadow_tex = null


## Whether this fighter uses the long-limbed rig (set from the fighter's class; changing it rebuilds the model).
var long_limbs := false
var head_fit := Transform3D()
## Where the glasses go (the same design space as `head_fit`, onto the face drawing's eyes; see `_glasses_fit`).
var glasses_fit := Transform3D()
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
const DASH_STEP_FROM := 0.06
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
	# Each kind of stuff takes the light its own way (`respond`): rubbery gloves and soles, the shoes' sheen; cloth stays matte.
	var glove := respond(white, "rubber")
	var shoe := respond(toon(Color(0.27, 0.2, 0.3)), "rubber")
	var shirt := _shirt_material()
	# The shorts take the outfit colour (the accent, a shade darker) under a plain shirt, and navy under a coloured one.
	var shorts_colour: Color = loadout.accent_color().darkened(0.2) if loadout.shirt <= 1 else Color(0.22, 0.25, 0.42)
	var shorts := toon(shorts_colour)
	var skin_colour: Color = loadout.body_color()
	# Arms and legs carry their clothes as painted bands (shaders/limb.gdshader): a sleeve from the shoulder when there is a shirt; the
	# shorts' leg and a white sock on each leg.
	var arm := _limb_material(skin_colour, _sleeve_colour(), 0.36, Color.WHITE, 2.0)
	var leg := _limb_material(skin_colour, shorts_colour, 0.3, Color.WHITE, 0.62)
	# The base body wears modelled clothes (art/blender/make_rigged_blob.py's `build_garments`): a shirt and shorts, coloured here.
	var garments := rig.find_child("Shirt", true, false) != null
	# Bare hands are modelled mittens (`Mitt`, art/blender/make_rigged_blob.py's `mitten`); a rig made before them has none, and its glove
	# fists stand in, in the body's colour.
	var bare: bool = use_base_body and not loadout.has_gloves()
	var mittens := rig.find_child("Mitt*", true, false) != null
	for mi in rig.find_children("*", "MeshInstance3D", true, false):
		var part := str(mi.name)
		# Full detail always: Godot simplifies meshes that look small on screen (its automatic LOD), which chewed thin cloth's hems into
		# a torn edge in the menus' small previews.
		mi.lod_bias = 128.0
		if part.begins_with("Arm"):
			mi.material_override = arm
		elif part.begins_with("Leg"):
			mi.material_override = leg
		elif part.begins_with("Mitt"):
			mi.material_override = respond(toon(skin_colour), "skin")
			mi.visible = bare
		elif part.begins_with("Hand") and bare:
			# Bare hands (the concept's mittens): the body's own colour and skin.
			mi.material_override = respond(toon(skin_colour), "skin")
			mi.visible = not mittens
		elif part.begins_with("Hand") or part.begins_with("Sole"):
			mi.material_override = glove
		elif part.begins_with("Collar") or part.begins_with("Cuff"):
			mi.material_override = white
			# (The glove's cuff: none on a bare hand.)
			mi.visible = not (use_base_body and part.begins_with("Cuff") and not loadout.has_gloves())
		elif part.begins_with("Strap"):
			mi.material_override = toon(loadout.accent_color())
		elif part.begins_with("Shorts"):
			mi.material_override = _cloth(shorts)
			mi.visible = loadout.has_shorts()
		elif part.begins_with("Shirt"):
			var cloth := _garment_shirt_material()
			mi.visible = cloth != null
			if cloth != null:
				var shirt_mat: ShaderMaterial = (_cloth(cloth) as ShaderMaterial).duplicate()
				# An ink line along the bottom hem, so where the shirt meets the shorts reads as one clean edge at any size.
				shirt_mat.set_shader_parameter("hem_low", float(_base_rig_info().get("clothes", {}).get("waist", 0.89)) - SHIRT_HEM_BELOW_WAIST)
				shirt_mat.set_shader_parameter("hem_ink", SHIRT_HEM_INK)
				shirt_mat.set_shader_parameter("hem_color", INK)
				mi.material_override = shirt_mat
		elif part.begins_with("Foot"):
			mi.material_override = shoe
		elif part.begins_with("Face"):
			face_mesh = mi
		elif part.begins_with("Skin"):
			mi.material_override = _skin_material(skin_colour, shorts_colour, not garments)
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
		# The look's pieces were designed round a head of radius 0.8 centred 1.42 up, and the neckwear round the rigged torso's ellipsoid
		# (see `_neck_on_rig`): each is stretched onto this body's own (its head is wider than it is tall, so a hat sits on top rather than
		# above it). The weapon arm turns at its shoulder and reaches to its wrist and a little past.
		var info := _base_rig_info()
		var centre: Array = info.get("head_centre", [0.0, 1.79, 0.0])
		var radii: Array = info.get("head_radii", [0.4, 0.34, 0.4])
		var hs := Vector3(float(radii[0]), float(radii[1]), float(radii[2])) / 0.8
		head_fit = Transform3D(Basis.from_scale(hs), Vector3(0.0, float(centre[1]) - 1.42 * hs.y, float(centre[2])))
		var face: Array = info.get("face", [float(centre[1]) + 0.04 * float(radii[1]), 1.75 * float(radii[0])])
		glasses_fit = _glasses_fit(float(face[0]), float(face[1]), hs.z, float(centre[2]))
		var torso: Dictionary = info.get("torso", {"centre": [0.0, 1.13, 0.0], "radii": [0.3, 0.32, 0.22]})
		var tc: Array = torso.centre
		var tr: Array = torso.radii
		var ts := Vector3(float(tr[0]) / TORSO_RADII.x, float(tr[1]) / TORSO_RADII.y, float(tr[2]) / TORSO_RADII.z)
		torso_fit = Transform3D(Basis.from_scale(ts), Vector3(0.0, float(tc[1]) - TORSO_CENTRE_Y * ts.y, float(tc[2])))
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
	if not use_base_body:
		# The blob's face drawing (make_rigged_blob.py's FACE_CENTRE and FACE_SIZE), moved with the long-limbed body's head.
		glasses_fit = head_fit * HEAD_FIT.affine_inverse() * _glasses_fit(1.585, 1.155, 0.825, 0.0)
	head_rig.transform = head_fit
	torso_rig.transform = torso_fit
	anim.play("idle")
	anim.advance(0.0)
	return true


## When the polish clips play (motion plan phase 3): a stop out of a run while still sliding faster than this (units a frame), over this
## many frames; a fidget after this many frames standing still, for its length, every so often; a platform drop's hop over its first
## frames; the heavy hitstun pose for knockback faster than this.
const RUN_STOP_SPEED := 0.04
## The walk clip plays once a cycle at this ground speed (units a frame), faster or slower with the speed: quick enough that the planted
## foot slides only about a third of the ground speed on the short legs (motion plan phase 4), not so quick that the walk scurries.
const WALK_TEMPO := 0.055
## The run and dash take this many more steps than one stride a dash length: the cute body's short legs cover less ground a step, so
## they scurry (motion plan phase 4, measured by tests/motion_probe.gd).
const STEP_CADENCE := 1.4
const RUN_STOP_FRAMES := 20.0
const FIDGET_AFTER := 300
const FIDGET_EVERY := 420
const FIDGET_FRAMES := 72.0
const DROP_FRAMES := 20.0
const HEAVY_HITSTUN_SPEED := 0.15
const TECH_INVULN_AT_MOST := 24


## Which clip suits this state of the simulation: [clip name, speed, progress]. Progress is -1 for a clip that simply plays, or 0..1 for
## one that follows the move (an attack's swing lands on the move's own frames).
func _choose_clip(s: Dictionary) -> Array:
	var state: String = s.state
	var grounded: bool = s.platform >= 0
	var speed := absf(float(s.vel.x))
	match state:
		"Walk":
			return ["walk", clampf(speed / WALK_TEMPO, 0.5, 3.6), -1.0]
		"Run", "Dash":
			# Cycles a second (two dash lengths each) times the clip's length in seconds.
			var stride := 2.0 * maxf(0.5, float(s.get("dash_length", 2.4)))
			var clip := "dash" if state == "Dash" else "run"
			var length := anim.get_animation(clip).length if anim != null and anim.has_animation(clip) else 0.3
			return [clip, minf(3.2, speed * 60.0 / stride * length * STEP_CADENCE), -1.0]
		"Attack":
			return _attack_clip(s)
		"Turn":
			# Turning out of a run skids; a turn from standing just turns.
			return ["skid", 1.0, -1.0] if speed > 0.08 else ["idle", 1.0, -1.0]
		"LedgeGetUp":
			if ledge_rolling:
				return ["roll", 1.0, ledge_k]
			return ["ledge_climb", 1.0, ledge_k]
		"Landing":
			# A tech in place is a landing with a tech's intangibility (a plain landing has none; a fighter still invincible from its
			# respawn has far more): a slap on the floor and a pop up, over its lag.
			if int(s.get("invuln", 0)) > 0 and int(s.get("invuln", 0)) <= TECH_INVULN_AT_MOST:
				return [AnimLibrary.or_stand_in("tech", anim), 1.0, clampf(float(s.state_frame) / maxf(1.0, float(s.get("lag", 4))), 0.0, 1.0)]
		"Idle":
			# Still sliding as it stops out of a run: braced against the slide, then settled.
			if grounded and speed > RUN_STOP_SPEED and int(s.state_frame) < RUN_STOP_FRAMES:
				return [AnimLibrary.or_stand_in("run_stop", anim), 1.0, float(s.state_frame) / RUN_STOP_FRAMES]
			# Standing still for a long time, a fidget now and then.
			var still := int(s.state_frame) - FIDGET_AFTER
			if grounded and still >= 0 and still % FIDGET_EVERY < FIDGET_FRAMES:
				return [AnimLibrary.or_stand_in("idle_look", anim), 1.0, float(still % FIDGET_EVERY) / FIDGET_FRAMES]
		"Airborne":
			# Just dropped through a platform (the sim ignores platforms for a few frames after it): a hop down out of a crouch.
			if int(s.get("ignore", 0)) > 0 and int(s.state_frame) < DROP_FRAMES and flip_left <= 0.0:
				return [AnimLibrary.or_stand_in("platform_drop", anim), 1.0, float(s.state_frame) / DROP_FRAMES]
		"Hitstun":
			# Sent flying by a hit that does not tumble: folded round the blow, or arched up off it when sent upward.
			var kb: Vector2 = s.get("kb_vel", Vector2.ZERO)
			if not s.tumble and kb.length() > HEAVY_HITSTUN_SPEED:
				return [AnimLibrary.or_stand_in("hurt_up" if kb.y > absf(kb.x) else "hurt_heavy", anim), 1.0, -1.0]
		"LedgeAttack":
			# Up onto the edge, then the sweep (its hit on frames 24-26 of 55 lands on the clip's strike).
			var f: float = s.state_frame
			if f < 14.0:
				return ["ledge_climb", 1.0, clampf(f / 14.0, 0.0, 1.0) * 0.66]
			var p := 0.55 * (f - 14.0) / 11.0 if f < 25.0 else 0.55 + (f - 25.0) / 30.0 * 0.45
			return ["attack_low", 1.0, clampf(p, 0.0, 1.0)]
	# The shared clips (scripts/anim_library.gd): the same for every moveset. The dodges follow their own frames.
	var shared := AnimLibrary.state_clip(state, anim)
	if shared != "":
		var k := dodge_progress(s)
		if AnimLibrary.BY_PROGRESS.has(state) and k >= 0.0:
			return [shared, 1.0, k]
		return [shared, 1.0, -1.0]
	if grounded:
		return ["idle", 1.0, -1.0]
	if flip_left > 0.0:
		return ["air_jump", 1.0, -1.0]
	return ["jump", 1.0, -1.0] if float(s.vel.y) > 0.02 else ["fall", 1.0, -1.0]


## How far through its dodge (roll, spot dodge, air dodge) the fighter is, 0..1, or -1 when not dodging (`dodge` from the bridge's
## `fighter_dodge`: [length, way]).
static func dodge_progress(s: Dictionary) -> float:
	var d: PackedInt32Array = s.get("dodge", PackedInt32Array())
	if d.size() < 2 or d[0] <= 0:
		return -1.0
	return clampf(float(s.state_frame) / float(d[0]), 0.0, 1.0)


## The clip for the move being performed and how far through it we are, mapped onto the clip's timeline: wind-up to 0.35, the strike
## through 0.55 to 0.75, then recovery. The move's own frames set the pace, so the swing lands when the hitbox does.
func _attack_clip(s: Dictionary) -> Array:
	var t: PackedInt32Array = s.move_timing
	var total := maxf(1.0, float(t[0]))
	var start := clampf(float(t[1]), 1.0, total)
	var last := clampf(float(t[2]), start, total)
	var progress := strike_progress(float(s.state_frame), total, start, last, s.get("move_windows", PackedInt32Array()))
	# Which clip: the moveset's own for this move, or the shared archetype (scripts/anim_library.gd).
	return [AnimLibrary.attack_clip(_cls(s), s.move_name, anim), 1.0, clampf(progress, 0.0, 1.0)]


## Where an attack clip is (0..1, on the shared move timeline: wind-up to 0.35, strike at 0.55, follow-through to 0.63, settled at 1) at
## move frame `f`. The swing lands on the move's first active frame and follows through while it is active; through the end lag the body
## holds the follow-through and only settles back near the end (as the reference game's moves do), so a move looks as long as it really
## is. A move with several separate hits (`windows`: [first, last] frame pairs, from the bridge's `fighter_move_windows`) strikes again
## for each: between two hits the body draws partly back toward the wind-up, then snaps forward onto the next hit's first frame, so a
## sequence of hits reads as a rhythm of strikes rather than one long held pose; the last hit gets the full follow-through. Only the
## move's frames drive it, so the animation never sets the timing.
static func strike_progress(f: float, total: float, start: float, last: float, windows: PackedInt32Array) -> float:
	if f < start:
		return f / start * 0.55
	if windows.size() >= 4:
		for k in range(0, windows.size() - 2, 2):
			var a := float(windows[k])
			var b := float(windows[k + 1])
			var next := float(windows[k + 2])
			if next - b - 1.0 >= 2.0:
				if f <= b:
					return 0.55 + clampf((f - a) / maxf(1.0, b - a), 0.0, 1.0) * 0.08
				if f < next:
					# Draw back to RESTRIKE_FROM over the first half of the gap, then strike again.
					var u := (f - b) / maxf(1.0, next - b)
					if u < 0.5:
						return lerpf(0.63, RESTRIKE_FROM, smoothstep(0.0, 1.0, u * 2.0))
					return lerpf(RESTRIKE_FROM, 0.55, (u - 0.5) * 2.0)
			else:
				# Hits back to back (a flurry, or a multihit's re-hits): the draw-back takes the one frame before the next hit, so each
				# hit's own first frame (where its hitlag freezes the pose) still shows the strike.
				var back_at := next - 1.0
				if f < back_at or back_at <= a:
					if f <= b:
						return 0.55 + clampf((f - a) / maxf(1.0, back_at - a), 0.0, 1.0) * 0.08
				elif f < next:
					return RESTRIKE_FROM
		start = float(windows[windows.size() - 2])
	if f <= last:
		return 0.55 + (f - start) / maxf(1.0, last - start) * 0.08
	var k := clampf((f - last) / maxf(1.0, total - last), 0.0, 1.0)
	return 0.63 + 0.37 * pow(k, 1.8)


## Between two hits of a multihit move the clip draws back to here (0.35 is the full wind-up, 0.55 the strike): enough to read as a new
## blow, not a whole new wind-up.
const RESTRIKE_FROM := 0.43


func _skeleton_to_model() -> Transform3D:
	return model.global_transform.affine_inverse() * skeleton.global_transform


## The stage frame: a node in the model that undoes its turn, so its axes are the stage's (x along the stage, y up, z toward the
## camera) while it still leans, squashes and moves with the body. Everything aimed at a hit (the weapon, the arm and leg reaching, the
## body following them) works in it.
var stage_frame: Node3D


func _skeleton_to_frame() -> Transform3D:
	return stage_frame.global_transform.affine_inverse() * skeleton.global_transform


## How the fighter is turned (the "cheat" of a 3D fighter on a 2D stage), as the reference game's fighters stand: the hips and legs turned
## well toward the way it faces (70 degrees off straight-on), so strides, kicks and swings happen across the screen where they read; the
## chest twisted back so it mostly faces the camera (35 degrees off); and the head turned again a little toward the way it faces (45
## degrees off: a three-quarter face), so the two fighters look at each other while the drawn face keeps both eyes in view.
const BODY_TURN := 70.0
const CHEST_TO_CAMERA := 35.0
## How far the head turns back from the chest toward the way the fighter faces (toward its opponent). The eyes in the face look that way
## too (see `set_gaze`).
const HEAD_TOWARD_FOE := 10.0
## In a move: how far the chest still opens to the camera, how much further the hips turn toward the foe, and how far the head may turn
## from the camera (a three-quarter face instead of the near front view of the rest of the time).
const CHEST_IN_ATTACK := 6.0
const BODY_TURN_IN_ATTACK := 14.0
const FACE_AWAY_IN_ATTACK := 58.0
## How far into a move the body is turned toward the foe (0 none, 1 fully), eased in and out.
var attack_turn := 0.0


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
		# A clip that follows the move (or the ledge climb) starts at once: it is posed by seeking while paused, and seeking never moves
		# a cross-fade on, so it would stay showing the clip before.
		var follows: bool = float(pick[2]) >= 0.0
		# Likewise a hit's pose: a fighter frozen by hitlag is already in it for the whole freeze (the animation does not move on then).
		var frozen: bool = int(s.hitlag) > 0
		anim.play(clip, 0.1 if current_clip != "" and not follows and not frozen and not clip.begins_with("attack") else 0.0)
		if frozen and not follows:
			anim.advance(0.0)
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
	_rest_unkeyed(clip)
	_body_follow(s, delta)
	_lag_bones(s)
	_face_camera(int(s.facing))
	_follow_bones()


## The chest and head get turns and bends added on top of the clip every frame (`_body_follow`, `_lag_bones`), which only works if the
## clip sets them every frame. A clip that leaves one at rest has no track for it (the exporter drops it), so nothing would put it back
## and the additions would pile up (the head spun round in a long shield hold): such a bone is put back to rest first.
var unkeyed := {}


func _rest_unkeyed(clip: String) -> void:
	if not unkeyed.has(clip):
		var missing: Array[int] = []
		var a := anim.get_animation(clip) if anim.has_animation(clip) else null
		for bone in [spine_bone, head_bone]:
			if bone < 0 or a == null:
				continue
			var name := skeleton.get_bone_name(bone)
			var found := false
			for t in a.get_track_count():
				if String(a.track_get_path(t)).get_slice(":", 1) == name:
					found = true
					break
			if not found:
				missing.append(bone)
		unkeyed[clip] = missing
	for bone in unkeyed[clip]:
		skeleton.reset_bone_pose(bone)


## Moves the head's and the torso's pieces (hats, glasses, neckwear) with the head and chest bones as they are posed now.
func _follow_bones() -> void:
	if skeleton == null or head_rig == null or not skeleton.is_inside_tree() or not model.is_inside_tree():
		return
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
const HEAD_COUNTER_ARCH := 0.75


func _body_follow(s: Dictionary, delta: float) -> void:
	var target_bend := 0.0
	var target_look := 0.0
	var tip: Vector3 = s.move_tip
	if s.state == "Attack" and tip != Vector3.ZERO and spine_bone >= 0 and _spin(s) == 0.0:
		var kicking: bool = AnimLibrary.is_kick(AnimLibrary.attack_clip(_cls(s), s.move_name, anim))
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
				# (Not too far: the base body's big head arched back past this shows its chin, not its face.)
				target_bend = -minf((phi - 100.0) * 0.85, 26.0 if use_base_body else 50.0)
			elif phi < -40.0:
				# (Less: kicks behind already lean the body forward in their clips.)
				target_bend = minf((-40.0 - phi) * 0.4, 10.0 if use_base_body else 24.0)
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
		# (Less far up for a kick: the body already leans back under a high one, and the face must stay toward the camera.)
		target_look = clampf(rise * 0.5, -20.0, 12.0 if kicking else 30.0)
		# In proportion to how far into the move the limb is (its blend toward the hitbox). The base body's clips already lean and look
		# into its swings and kicks, and its legs reach high on their own, so it takes half of this.
		var k := (leg_k if kicking else arm_k) * (0.5 if use_base_body else 1.0)
		target_bend *= k
		target_look *= k
	var follow := clampf(delta * 16.0, 0.0, 1.0)
	body_bend = lerpf(body_bend, target_bend, follow)
	head_look = lerpf(head_look, target_look, follow)
	if absf(body_bend) > 0.05:
		_bend_bone(spine_bone, body_bend, int(s.facing))
	# (When the body arches back, the head tips forward against most of it, so the face stays toward the camera: the big head tipped back
	# showed only its chin.)
	var head_bend := -head_look - minf(body_bend, 0.0) * HEAD_COUNTER_ARCH
	if absf(head_bend) > 0.05:
		_bend_bone(head_bone, head_bend, int(s.facing))
	# A flinch: the chest and head snap back from the hit and come forward again.
	if flinch_left > 0.0:
		flinch_left = maxf(0.0, flinch_left - delta)
		var k := flinch_left / FLINCH_TIME
		_bend_bone(spine_bone, -16.0 * k, int(s.facing))
		_bend_bone(head_bone, -12.0 * k, int(s.facing))
	# Open the chest to the camera and turn the head back toward the opponent (see BODY_TURN). In a move the chest turns into it instead,
	# toward the opponent (`attack_turn`), so a strike is thrown at the foe and not at the camera.
	_turn_bone(spine_bone, -lerpf(CHEST_TO_CAMERA, CHEST_IN_ATTACK, attack_turn) * float(s.facing))
	_turn_bone(head_bone, HEAD_TOWARD_FOE * float(s.facing))


## The chest and head lagging the hips (the `spine`, `head` and `head_turn` springs), on top of the clip and the body follow; and behind a
## shield, a slow breath (by the sim frame, so a freeze holds it) that moves the chest and head but never the bubble.
const SHIELD_BREATH := 2.5
const SHIELD_BREATH_FRAMES := 80.0


func _lag_bones(s: Dictionary) -> void:
	if skeleton == null or spine_bone < 0:
		return
	var facing := int(s.facing)
	if String(s.state) == "Shield":
		var b := sin(float(s.frame) * TAU / SHIELD_BREATH_FRAMES)
		_bend_bone(spine_bone, SHIELD_BREATH * b, facing)
		_bend_bone(head_bone, -SHIELD_BREATH * 0.6 * b, facing)
	if absf(springs.value("spine")) > 0.05:
		_bend_bone(spine_bone, springs.value("spine"), facing)
	if absf(springs.value("head")) > 0.05:
		_bend_bone(head_bone, springs.value("head"), facing)
	if absf(springs.value("head_turn")) > 0.05:
		_turn_bone(head_bone, springs.value("head_turn"))


## The base body's big head carries the drawn face, half of every read: however a clip and the layers on it twist the chest (a strike
## turns it into the swing), the head never turns more than this far from the camera; past it, it is turned back.
const FACE_AWAY_MAX := 32.0
## Nor does it look down further than this (a pitched-down head shows its crown, and the hat).
const FACE_DOWN_MAX := 18.0


func _face_camera(facing: int) -> void:
	if not use_base_body or skeleton == null or head_bone < 0 or not skeleton.is_inside_tree():
		return
	var head_delta: Transform3D = skeleton.get_bone_global_pose(head_bone) * head_rest_inv
	var ahead: Vector3 = (_skeleton_to_frame().basis * head_delta.basis * Vector3(0, 0, 1)).normalized()
	var away := rad_to_deg(atan2(ahead.x, ahead.z))
	var limit := lerpf(FACE_AWAY_MAX, FACE_AWAY_IN_ATTACK, attack_turn)
	if absf(away) > limit:
		_turn_bone(head_bone, -signf(away) * (absf(away) - limit))
	var down := -rad_to_deg(asin(clampf(ahead.y, -1.0, 1.0)))
	if down > FACE_DOWN_MAX:
		_bend_bone(head_bone, -(down - FACE_DOWN_MAX), facing)


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


## A victory pose for the results screen: plays one of the looping victory clips (by class: the sword is raised high, the claws fighter
## pumps a fist; `cheer` makes anyone throw both arms up and hop), with the weapon held up in the raised hand. Call every
## frame with the time since the last.
func play_victory(cls: int, delta: float, cheer := false) -> void:
	if anim == null:
		return
	var clip := "victory_c" if cheer else ("victory_b" if cls != 0 else "victory_a")
	set_expression(Loadout.HAPPY_FACE)
	_blink(delta)
	if current_clip != clip and anim.has_animation(clip):
		anim.play(clip, 0.15)
		current_clip = clip
	anim.speed_scale = 1.0
	anim.advance(delta)
	_follow_bones()
	# The sword, gripped in the raised hand and pointing up.
	var armed := cls == 0 and not cheer
	blade_pivot.visible = armed
	if armed:
		var hand := skeleton.find_bone("hand.R")
		var at: Vector3 = _skeleton_to_frame() * skeleton.get_bone_global_pose(hand).origin
		blade_pivot.position = at
		blade_pivot.rotation = Vector3(0, 0, PI / 2.0)
		blade_pivot.scale = Vector3(1.9 / MESH_LENGTH, 1.0, 1.0)


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
	percent_label.font = UIDraw.font()
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
	# The match display's typeface, so the player tags over the fighters match it.
	name_label.font = UIDraw.font()
	name_label.position = Vector3(0, 3.55, 0)
	name_label.text = name_text
	name_label.visible = name_text != ""
	add_child(name_label)

	# Weapon: pivots at the hand and extends along +x. Claws just shrink it (see _pose_blade).
	blade_parts.clear()
	stage_frame = Node3D.new()
	model.add_child(stage_frame)
	blade_pivot = Node3D.new()
	stage_frame.add_child(blade_pivot)
	var blade_mesh := _blade_shape()
	blade_parts.append(_part(blade_pivot, blade_mesh, respond(toon(Color(0.82, 0.87, 0.96)), "metal"), Vector3(1.55, 0, 0)))
	var guard_mesh := BoxMesh.new()
	guard_mesh.size = Vector3(0.12, 0.6, 0.12)
	blade_parts.append(_part(blade_pivot, guard_mesh, respond(toon(Color(0.92, 0.76, 0.3)), "gold"), Vector3(0.18, 0, 0)))
	var grip_mesh := BoxMesh.new()
	grip_mesh.size = Vector3(0.4, 0.12, 0.12)
	blade_parts.append(_part(blade_pivot, grip_mesh, toon(SASH), Vector3(-0.02, 0, 0)))
	# The Seer's staff (class STAFF_CLASS), on the same pivot so it is posed as the blade is: a pale wooden shaft held a little from its
	# foot, a gold collar and a glowing round head.
	staff_parts.clear()
	var shaft := CylinderMesh.new()
	shaft.top_radius = 0.08
	shaft.bottom_radius = 0.095
	shaft.height = MESH_LENGTH
	shaft.radial_segments = 10
	staff_parts.append(_part(blade_pivot, shaft, toon(Color(0.86, 0.72, 0.52)), Vector3(MESH_LENGTH * 0.5 - 0.35, 0, 0), Vector3.ONE, Vector3(0, 0, 90)))
	var collar := CylinderMesh.new()
	collar.top_radius = 0.14
	collar.bottom_radius = 0.14
	collar.height = 0.14
	collar.radial_segments = 12
	staff_parts.append(_part(blade_pivot, collar, respond(toon(Color(0.95, 0.78, 0.32)), "gold"), Vector3(MESH_LENGTH - 0.42, 0, 0), Vector3.ONE, Vector3(0, 0, 90)))
	var head := SphereMesh.new()
	head.radius = 0.26
	head.height = 0.52
	staff_parts.append(_part(blade_pivot, head, toon(Color(0.78, 0.96, 1.0)), Vector3(MESH_LENGTH - 0.22, 0, 0)))

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
	# The pieces on the head and chest start where the idle pose has them (a fighter shown without `apply`, as in the menus, keeps it):
	# now, or as soon as the fighter is in the scene (its transforms are only known there).
	if is_inside_tree():
		_follow_bones()
	elif not tree_entered.is_connected(_follow_bones):
		tree_entered.connect(_follow_bones, CONNECT_ONE_SHOT)


const FIGHTER_RIM_BOOST := 1.4


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
			if m is ShaderMaterial and ((m as ShaderMaterial).shader in [TOON_SHADER, TOON_CLOTH_SHADER, LIMB_SHADER, FACE_SHADER, SKIN_SHADER]):
				(node as GeometryInstance3D).set_instance_shader_parameter("object_id", 0.06 + 0.07 * float(player % 8))
				# The fighters are the clearest things on screen: the full ink outline (the scenery's is thinner and softer) and a
				# stronger rim.
				(node as GeometryInstance3D).set_instance_shader_parameter("outline_weight", FIGHTER_OUTLINE_WEIGHT)
				# (Clothes keep the plain rim: a strong one picks out every fold as a bright streak.)
				var cloth := str(node.name).begins_with("Shirt") or str(node.name).begins_with("Shorts")
				(node as GeometryInstance3D).set_instance_shader_parameter("rim_boost", 1.0 if cloth else FIGHTER_RIM_BOOST)
				(node as GeometryInstance3D).set_instance_shader_parameter("fighter", 1.0)
				# The line art in the fighter's own colour, much darkened (see `outline_tint` in shaders/outline.gdshader).
				# (A pale body gets a darker line, so its outline still parts it from a pale sky.)
				var body_c: Color = loadout.body_color() if loadout != null else INK
				var darken := lerpf(OUTLINE_TINT_DARKEN, OUTLINE_TINT_DARKEN_PALE, smoothstep(0.55, 0.9, body_c.get_luminance()))
				var line_c: Color = body_c.darkened(darken) if loadout != null else INK
				(node as GeometryInstance3D).set_instance_shader_parameter("outline_tint", Color(line_c, OUTLINE_TINT))
				break
	for c in node.get_children():
		_tag_object(c)


## The whole-body flash: white for a moment when the fighter is hit (strongest at the start of hitlag), a yellow pulse while a smash
## attack is charging. Cosmetic.
var flash_amount := 0.0
## How strong the hit being flashed for is (0 light to 1 strong), from its hitlag.
var flash_strength := 0.0
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
		# A flash on the first frames of the freeze, then only a faint one, so the fighter stays readable: soft and short for a light hit
		# (by its hitlag), bright for a strong one.
		if flash_hit_frame < 0:
			flash_hit_frame = int(s.frame)
			flash_strength = clampf((float(s.hitlag) - 4.0) / 14.0, 0.0, 1.0)
		var bright_frames := 2 if flash_strength < 0.4 else 3
		target = lerpf(0.3, 0.6, flash_strength) if int(s.frame) - flash_hit_frame < bright_frames else 0.1
	elif int(s.charge) > 0 and s.state == "Attack":
		colour = Color(1.0, 0.85, 0.25)
		target = 0.12 + 0.1 * sin(float(s.frame) * 0.6)
	flash_amount = target if target > flash_amount else move_toward(flash_amount, target, delta * 12.0)
	flash_colour = colour
	var value := Color(flash_colour.r, flash_colour.g, flash_colour.b, flash_amount)
	if absf(flash_amount - last_flash) < 0.001:
		return
	last_flash = flash_amount
	# The rim light flares with the flash too (a material reaction, not only a colour wash), settling with it.
	for m in meshes:
		if m.material_override is ShaderMaterial:
			m.set_instance_shader_parameter("flash", value)
			var cloth := str(m.name).begins_with("Shirt") or str(m.name).begins_with("Shorts")
			m.set_instance_shader_parameter("rim_boost", (1.0 if cloth else FIGHTER_RIM_BOOST) * (1.0 + HIT_RIM_FLARE * flash_amount))
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


## The face is drawn on the shell in front of the head, lit like the skin around it: the look's own eyes, brows, nose, mouth and cheeks
## (FaceLook), with each expression changing the lids, brows and mouth.


func _face() -> void:
	if face_mesh == null:
		return
	face_mat = ShaderMaterial.new()
	face_mat.shader = FACE_SHADER
	face_mat.set_shader_parameter("skin", loadout.body_color())
	face_mesh.material_override = face_mat
	# (A new material: show the face even if the expression is the one already set.)
	shown_face = ""
	last_expression = Loadout.FACES[loadout.face]
	_show_face()


## Which way the eyes look: 1 toward screen right, -1 toward screen left (the face drawings look right; facing left mirrors them).
func set_gaze(side: int) -> void:
	if face_mat != null:
		face_mat.set_shader_parameter("mirror", side < 0)


## Shows an expression: one of Loadout.FACES or Loadout.HURT, or an action face (its `name` is one of FaceLook.EXPRESSIONS).
func set_expression(e: Dictionary) -> void:
	if e == last_expression:
		return
	last_expression = e
	_show_face()


## The face to show now: the expression, or closed eyes for a moment while blinking.
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
	FaceLook.apply(face_mat, loadout, name_now)


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


## The modelled shirt's material for this look (or null: no shirt): the cel material in the shirt's colour, its print laid by the
## garment's rest-pose texture map (x and height, as the skin's), at the scales the painted shirt used.
func _garment_shirt_material() -> Material:
	var accent: Color = loadout.accent_color()
	match loadout.shirt:
		1:
			return toon(SHIRT_WHITE)
		2:
			return toon(accent)
		3:
			return toon(Color(1, 1, 1), true, SvgArt.texture("res://art/cloth/stripes.svg", {"#ff00ff": accent}), Vector2(1.5, 12.0))
		4:
			return toon(Color(1, 1, 1), true, SvgArt.texture("res://art/cloth/aloha.svg", {"#ff00ff": accent}), Vector2(4.0, 4.0))
	return null


## A modelled garment's material: its cel material with an ink outline pushed further back than the body's. Cloth folds inward where
## the body bends (the shirt's sides when the fists come up), and the ordinary outline, sitting just behind the surface, pokes through a
## fold as dark specks; pushed back it still draws the garment's silhouette but stays behind its folds.
static var _cloth_outline: ShaderMaterial
static var _cloth_cache := {}


static func _cloth(m: Material) -> Material:
	if not (m is ShaderMaterial) or (m as ShaderMaterial).next_pass == null:
		return m
	if _cloth_cache.has(m):
		return _cloth_cache[m]
	if _cloth_outline == null:
		_cloth_outline = ((m as ShaderMaterial).next_pass as ShaderMaterial).duplicate()
		_cloth_outline.set_shader_parameter("push_back", CLOTH_OUTLINE_PUSH_BACK)
		# A little thinner than the body's line too: the garments' edges are inner lines, lighter than the silhouette.
		for w in ["width", "min_width", "max_width"]:
			var now: Variant = _cloth_outline.get_shader_parameter(w)
			if now == null:
				now = RenderingServer.shader_get_parameter_default(_cloth_outline.shader.get_rid(), w)
			if now == null:
				# (Without a renderer, as in the headless tests: outline.gdshader's own defaults.)
				now = {"width": 0.0022, "min_width": 0.02, "max_width": 0.11}[w]
			_cloth_outline.set_shader_parameter(w, float(now) * CLOTH_OUTLINE_THIN)
	var c: ShaderMaterial = (m as ShaderMaterial).duplicate()
	c.shader = TOON_CLOTH_SHADER
	# A softer step from light to shadow than the body's: on cloth the hard one picked out every small bump as a row of teeth.
	c.set_shader_parameter("ramp_softness", CLOTH_RAMP_SOFTNESS)
	c.next_pass = _cloth_outline
	_cloth_cache[m] = c
	return c


const CLOTH_OUTLINE_PUSH_BACK := 2.2
const CLOTH_OUTLINE_THIN := 0.65
## How much the rim light flares with a hit's flash (times its usual strength at a full flash).
const HIT_RIM_FLARE := 2.5
## How much of its own colour a fighter's outline takes, and how dark that colour is made first.
const OUTLINE_TINT := 0.75
## The fighters' line weight against the scenery's 0.6 (outline.gdshader): still the strongest line on screen, but a thinner, warmer line
## like the concept art's, not a heavy black one.
const FIGHTER_OUTLINE_WEIGHT := 0.82
const OUTLINE_TINT_DARKEN := 0.64
const OUTLINE_TINT_DARKEN_PALE := 0.8
const CLOTH_RAMP_SOFTNESS := 0.32
## The shirt's bottom hem: how far below the waist it is cut (art/blender/make_rigged_blob.py) and how wide its ink line is (rest units).
const SHIRT_HEM_BELOW_WAIST := 0.012
const SHIRT_HEM_INK := 0.01


## The sleeve band's colour (the shirt's main colour), or none (a negative alpha) without a shirt.
func _sleeve_colour() -> Color:
	match loadout.shirt:
		1:
			return SHIRT_WHITE
		2, 3, 4:
			return loadout.accent_color()
	return Color(0, 0, 0, -1)


static var _limb_cache := {}


## The base body's skin with this look's clothes painted on (shaders/skin.gdshader): the shirt and its sleeves, the shorts, white socks,
## shoes with the outfit-coloured strap, where base_rig.json puts them.
func _skin_material(skin_c: Color, shorts_c: Color, painted_clothes := true) -> ShaderMaterial:
	var key := "skin%s%s%d%s%s%d" % [skin_c.to_html(), shorts_c.to_html(), loadout.shirt, loadout.accent_color().to_html(), painted_clothes,
		loadout.gear]
	if _limb_cache.has(key):
		return _limb_cache[key]
	var m := ShaderMaterial.new()
	m.shader = SKIN_SHADER
	m.set_shader_parameter("skin", skin_c)
	var accent: Color = loadout.accent_color()
	match loadout.shirt:
		1:
			m.set_shader_parameter("shirt", SHIRT_WHITE)
		2:
			m.set_shader_parameter("shirt", accent)
		3:
			m.set_shader_parameter("shirt", Color.WHITE)
			m.set_shader_parameter("shirt_tex", SvgArt.texture("res://art/cloth/stripes.svg", {"#ff00ff": accent}))
			m.set_shader_parameter("use_shirt_tex", true)
			m.set_shader_parameter("shirt_tex_scale", Vector2(1.5, 12.0))
		4:
			m.set_shader_parameter("shirt", Color.WHITE)
			m.set_shader_parameter("shirt_tex", SvgArt.texture("res://art/cloth/aloha.svg", {"#ff00ff": accent}))
			m.set_shader_parameter("use_shirt_tex", true)
			m.set_shader_parameter("shirt_tex_scale", Vector2(4.0, 4.0))
		_:
			m.set_shader_parameter("shirt", Color(0, 0, 0, 0))
	m.set_shader_parameter("shorts", shorts_c)
	m.set_shader_parameter("painted_clothes", painted_clothes)
	m.set_shader_parameter("shorts_on", loadout.has_shorts())
	m.set_shader_parameter("shod", loadout.has_shoes())
	for p in RESPONSES.skin:
		m.set_shader_parameter(p, RESPONSES.skin[p])
	m.set_shader_parameter("shoe", Color(0.27, 0.2, 0.3))
	m.set_shader_parameter("strap", accent)
	var regions: Dictionary = _base_rig_info().get("clothes", {})
	for name in ["neck", "waist", "shorts_end", "sock_top", "shoe_top", "sole_top"]:
		if regions.has(name):
			m.set_shader_parameter(name + "_z", float(regions[name]))
	if regions.has("torso_half_width"):
		m.set_shader_parameter("torso_half_width", float(regions.torso_half_width))
	for name in ["shoulder_l", "elbow_l", "shoulder_r", "elbow_r"]:
		if regions.has(name):
			m.set_shader_parameter(name, Vector2(float(regions[name][0]), float(regions[name][1])))
	m.next_pass = toon(Color.WHITE).next_pass
	if not painted_clothes:
		# Under the modelled clothes the skin's outline is left out too (it would poke through the cloth where it sits close).
		var line: ShaderMaterial = (m.next_pass as ShaderMaterial).duplicate()
		line.set_shader_parameter("hide_under_shirt", loadout.shirt != 0)
		line.set_shader_parameter("hide_under_shorts", loadout.has_shorts())
		m.next_pass = line
	_limb_cache[key] = m
	return m


## The banded limb material (see shaders/limb.gdshader), with the ink outline. A `top` with negative alpha means no top band.
static var _blade_cache: ArrayMesh = null


## The sword's blade: 2.7 long (x, centred), 0.16 deep and 0.09 thick like the box it replaced (so the code that stretches it to the
## move's reach is unchanged), but with a diamond cross-section (a ridge down the middle, two bevels each side, flat-shaded) and a point.
## The upper bevels catch the key light and the lower ones fall into shade, the classic two-tone blade, and as the blade swings its bevels
## tilt and the metal highlight (`RESPONSES.metal`) slides along them.
static func _blade_shape() -> ArrayMesh:
	if _blade_cache != null:
		return _blade_cache
	var half := 1.35
	var tip_from := 0.95
	var h := 0.08
	var t := 0.045
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	st.set_smooth_group(-1)
	# The cross-section at x: top, front, bottom, back, shrinking to nothing at the point.
	var ring := func(x: float) -> Array:
		var k := 1.0 if x <= tip_from else clampf((half - x) / (half - tip_from), 0.0, 1.0)
		return [Vector3(x, h * k, 0), Vector3(x, 0, t * k), Vector3(x, -h * k, 0), Vector3(x, 0, -t * k)]
	var xs := [-half, tip_from, half]
	for i in xs.size() - 1:
		var a: Array = ring.call(xs[i])
		var b: Array = ring.call(xs[i + 1])
		for k in 4:
			var k2 := (k + 1) % 4
			st.add_vertex(a[k]); st.add_vertex(b[k]); st.add_vertex(b[k2])
			st.add_vertex(a[k]); st.add_vertex(b[k2]); st.add_vertex(a[k2])
	# The flat end at the hilt.
	var base: Array = ring.call(-half)
	st.add_vertex(base[0]); st.add_vertex(base[2]); st.add_vertex(base[1])
	st.add_vertex(base[0]); st.add_vertex(base[3]); st.add_vertex(base[2])
	st.generate_normals()
	_blade_cache = st.commit()
	return _blade_cache


## How each kind of stuff on a fighter takes the light (shaders/toon_light.gdshaderinc), so skin, cloth, rubber and metal read apart while
## sharing one cel look: [parameter: value].
const RESPONSES := {
	# (Skin is matte and soft-stepped, as the concept art's bodies: flat colour that turns gently into shade, no glossy bands.)
	"skin": {"ramp_softness": 0.16, "sheen": 0.0},
	"rubber": {"ramp_softness": 0.12, "sheen": 0.16},
	"metal": {"ramp_softness": 0.08, "sheen": 0.0, "metal": 1.0, "rim_strength": 0.55},
	"gold": {"ramp_softness": 0.1, "sheen": 0.0, "metal": 0.75, "rim_strength": 0.5},
}
static var _respond_cache := {}


## A copy of material `m` with the light response `kind` (`RESPONSES`). Copies are shared per material and kind; `m` itself (often shared
## with props through `toon`'s cache) is never changed.
static func respond(m: ShaderMaterial, kind: String) -> ShaderMaterial:
	var key := "%d|%s" % [m.get_instance_id(), kind]
	if _respond_cache.has(key):
		return _respond_cache[key]
	var c := m.duplicate() as ShaderMaterial
	for p in RESPONSES[kind]:
		c.set_shader_parameter(p, RESPONSES[kind][p])
	_respond_cache[key] = c
	return c


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
	for p in RESPONSES.skin:
		m.set_shader_parameter(p, RESPONSES.skin[p])
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


## The rigged torso the neckwear is made round: a squat ellipsoid centred this high, with these half-width, half-height and half-depth.
const TORSO_CENTRE_Y := 0.98
const TORSO_RADII := Vector3(0.56, 0.42, 0.5)


## The glasses' fit: the design space the glasses are made in (the look's head of radius 0.8 centred 1.42 up, eyes at height 1.52 and
## 0.32 either side) laid onto the face, a square of `face_size` centred `face_centre` up. The face is shown 1.15 times larger than the
## shell's 512 square, so for the standard face its eyes are on row 207, 110 columns out (FaceLook.eye_row and eye_out follow the look's
## own eye placement). The lenses spread with the eyes and grow with them.
func _glasses_fit(face_centre: float, face_size: float, depth_scale: float, depth: float) -> Transform3D:
	# Design units to face units: the design eyes 0.32 out are the face's eyes, `out` columns (of 512) out.
	var out := FaceLook.eye_out(loadout)
	var kx := face_size * (out / 512.0) / 0.32
	# (The lenses' height follows the eyes' size: the design lens fits the old 0.38 eye.)
	var ky := face_size * (110.0 / 512.0) / 0.32 * (1.0 + FaceLook.offset(loadout, "eye_size") * 0.06) * FaceLook.EYE_SIZE / 0.38
	var eye_y := face_centre + (0.5 - FaceLook.eye_row(loadout) / 512.0) * face_size
	return Transform3D(Basis.from_scale(Vector3(kx, ky, depth_scale)), Vector3(0.0, eye_y - 1.52 * ky, depth))


## Neckwear for the rigged torso (TORSO_CENTRE_Y, TORSO_RADII). Rings are tori sized to hug it, so nothing pokes through the body whatever
## the pose.
func _neck_on_rig(accent: Color) -> void:
	var centre := Vector3(0, TORSO_CENTRE_Y, 0)
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
			# The sash's knotted end hanging at the hip, swinging with the body (`TailRig`).
			_tail(centre + Vector3(0.3, -0.36, 0.4), toon(SASH, false), 0.36, 0.13, 0.05, 3)
		2:
			# Neckerchief: a collar ring round the top of the torso with a point hanging in front.
			var at := _collar(accent)
			var bib := BoxMesh.new()
			bib.size = Vector3(0.34, 0.34, 0.06)
			# (On a pivot at the knot, so it swings with the body: the `neck` spring.)
			neck_pivot = Node3D.new()
			neck_pivot.position = Vector3(0, at - 0.02, 0.5)
			torso_rig.add_child(neck_pivot)
			_part(neck_pivot, bib, toon(accent), Vector3(0, -0.14, 0), Vector3.ONE, Vector3(0, 0, 45))
		3:
			# Scarf: the collar ring and a tail hanging down the front on one side.
			var at := _collar(accent)
			# (A chain of pieces on springs, so it swings and whips with the body: `TailRig`.)
			_tail(Vector3(0.26, at - 0.03, 0.46), toon(accent), 0.5, 0.2, 0.06, 4)


## The pivot the neckerchief's point hangs from (none for the other neckwear), and the cloth tails on spring chains (the scarf's end, the
## sash's end).
var neck_pivot: Node3D
var tails: Array = []


func _tail(at: Vector3, mat: Material, length: float, width: float, thickness: float, segments: int) -> void:
	var t := TailRig.new()
	t.position = at
	torso_rig.add_child(t)
	t.build(mat, length, width, thickness, segments)
	meshes.append_array(t.meshes())
	tails.append(t)


## The collar ring of the neckerchief and the scarf, just under the chin (where the head meets the shoulders, from base_rig.json's `neck`).
## Returns its height in the torso's design space, for what hangs from it.
func _collar(accent: Color) -> float:
	var ring := TorusMesh.new()
	ring.inner_radius = 0.44
	ring.outer_radius = 0.58
	ring.rings = 36
	ring.ring_segments = 10
	var at := 1.1
	var info := _base_rig_info()
	if info.has("neck") and torso_fit.basis.y.y > 0.001:
		at = (float(info.neck) - COLLAR_BELOW_NECK - torso_fit.origin.y) / torso_fit.basis.y.y
	_part(torso_rig, ring, toon(accent), Vector3(0, at, 0), Vector3(1.04, 1.0, 0.94))
	return at


## How far below the neck (where the head meets the shoulders) the collar ring sits: its top just shows under the chin.
const COLLAR_BELOW_NECK := 0.1


## The hat sits on a pivot at the crown of the head that lags behind sudden moves and bounces on landings (the `hat_roll` and `hat_drop` springs, see `_secondary`).
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
	# The glasses are placed in design space through their own fit (see `_glasses_fit`): a node under the head whose transform turns
	# the head's fit into it.
	var glasses_rig := Node3D.new()
	glasses_rig.transform = head_fit.affine_inverse() * glasses_fit
	head_rig.add_child(glasses_rig)
	var to_head: Transform3D = glasses_rig.transform
	# The glasses' space is stretched across by the eyes' spacing and up by their size: a lens is made round again by this.
	lens_aspect = glasses_fit.basis.get_scale().y / maxf(0.001, glasses_fit.basis.get_scale().x)
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
				_on_face(_part(glasses_rig, _sphere(1.0), lens, Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.04, Vector3(0.25, 0.18, 0.03))
				_on_face(_part(glasses_rig, rim, frame, Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.045, Vector3(0.25, 0.04, 0.18), true)
			_on_face(_part(glasses_rig, _sphere(1.0), frame, Vector3.ZERO), to_head, 0.0, 1.56, 0.06, Vector3(0.1, 0.025, 0.025))
		2:
			# Goggles: chunky rings with a strap round the head.
			var ring := TorusMesh.new()
			ring.inner_radius = 0.14
			ring.outer_radius = 0.28
			for sx in [-1.0, 1.0]:
				_on_face(_part(glasses_rig, ring, toon(Color(0.45, 0.28, 0.12)), Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.0, Vector3.ONE, true)
				_on_face(_part(glasses_rig, _sphere(1.0), toon(Color(0.7, 0.9, 1.0, 1.0), false), Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.02,
					Vector3(0.14, 0.14, 0.02))
			var strap := TorusMesh.new()
			strap.inner_radius = 0.78
			strap.outer_radius = 0.84
			_part(glasses_rig, strap, toon(Color(0.3, 0.2, 0.1)), Vector3(0, 1.52, 0), Vector3(1, 0.6, 1))
		3:
			# Round specs: thin rings and a bridge.
			var thin := TorusMesh.new()
			thin.inner_radius = 0.17
			thin.outer_radius = 0.21
			for sx in [-1.0, 1.0]:
				_on_face(_part(glasses_rig, thin, frame, Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.0, Vector3.ONE, true)
			_on_face(_part(glasses_rig, _sphere(1.0), frame, Vector3.ZERO), to_head, 0.0, 1.54, 0.02, Vector3(0.1, 0.02, 0.02))


## Seats a piece of the glasses on the face: over the drawn eye at (`x`, `y`) in the glasses' design space, on the head's surface
## (`lift` out from it, in the head's design units) and turned to face out from it, so on a fighter turned three-quarters to the camera the
## far lens stays on the face instead of floating past the head's edge. `to_head` takes the glasses' space to the head's design space (a
## sphere of radius 0.8 about 1.42 up, see `head_fit`). `ring` turns a torus to lie flat on the face.
var lens_aspect := 1.0


func _on_face(mi: MeshInstance3D, to_head: Transform3D, x: float, y: float, lift: float, scl: Vector3, ring := false) -> void:
	var d := to_head * Vector3(x, y, 0.0)
	var up := d.y - 1.42
	var z := sqrt(maxf(0.0, 0.64 - d.x * d.x - up * up))
	var normal := Vector3(d.x, up, z).normalized()
	var on := Vector3(d.x, d.y, 0.0) + Vector3(0.0, 0.0, z) + normal * lift
	var back := to_head.affine_inverse()
	mi.position = back * on
	# The surface's outward direction in the glasses' space (their space is only scaled and moved from the head's).
	var out := (back.basis * normal).normalized()
	var turn := Basis(Quaternion(Vector3(0, 0, 1), out))
	var round_scl := scl * Vector3(lens_aspect if x != 0.0 else 1.0, 1.0, 1.0)
	mi.basis = turn * (Basis.from_euler(Vector3(PI / 2.0, 0, 0)) if ring else Basis.IDENTITY) * Basis.from_scale(round_scl)


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


## How far round a tumbling launch has turned the body (radians).
var tumble_angle := 0.0
## Flying faster than this (world units a frame), a launched fighter is thrown head first along its path rather than tumbling.
const LAUNCH_POSE_SPEED := 0.32
## A hit too weak to launch: the body snaps back from it and recovers over this long (seconds left of FLINCH_TIME).
var flinch_left := 0.0
const FLINCH_TIME := 0.22
## Landing out of an aerial (its landing lag) or a fast fall: a heavier, deeper landing until it ends.
var heavy_landing := false


## Squash pose per state as a single number: positive squashes down and out, negative stretches up.
const SQUASH := {
	"JumpSquat": 0.32, "Landing": 0.2, "Crouch": 0.28, "WaveLand": 0.24, "Turn": 0.08,
	"Dash": 0.04, "ShieldDrop": 0.0, "Roll": 0.3, "SpotDodge": 0.34, "ShieldBreak": 0.22, "Grabbed": 0.14, "Knockdown": 0.12, "GetUp": 0.1,
}
## Forward lean in degrees per state.
const LEAN := {"Dash": 18.0, "Run": 14.0, "WaveLand": 28.0, "Walk": 4.0, "LedgeAttack": 20.0, "ShieldBreak": 32.0, "Grabbing": 8.0, "Grabbed": -10.0, "Knockdown": 78.0, "GetUp": 24.0}

# Damped springs make landings and takeoffs read as soft and elastic instead of linear and stiff.
const SPRING_K := 420.0
const SPRING_DAMP := 22.0  # a bit under critical (2*sqrt(K) = 41), so there is a small rebound
## How hard the take-off throws the squash spring into a stretch (its velocity, per second).
const TAKE_OFF_KICK := 9.0

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


## A roll's somersault turns through the middle of the roll (as shares of it), and a spot dodge steps this far back into the stage.
const ROLL_SPIN_FROM := 0.12
const ROLL_SPIN_TO := 0.78
const SPOT_DODGE_STEP := 0.7
## How far a fighter lying on the floor is raised (see `apply`).
const KNOCKDOWN_LIFT := 0.32
## The midair jump's front flip: seconds left of it, and the offset that keeps it turning about the body's middle.
const FLIP_TIME := 0.38
var flip_left := 0.0
var flip_offset := Vector3.ZERO
var prev_jumps := -1
## A wall jump flips the other way (backward, off the wall); the count of wall jumps seen last frame.
var flip_back := false
var prev_wall_jumps := -1
## The attack lunge: the body is thrown forward through the strike and drawn back in the wind-up (model space, along the facing).
var lunge := 0.0
var lunge_vel := 0.0
## The hat's lag (a roll, radians) and bounce (a drop below its place, units), and the velocity they react to.
## Secondary motion (`MotionSprings`, all stepped with the sim: frozen in hitlag and while paused, at rest after a rollback or seek):
## `hat_roll` the hat's lag (radians) and `hat_drop` its bounce (units below its place); `spine` and `head` the chest and head lagging the
## hips (degrees, forward positive) when the body speeds up, stops or lands; `head_turn` the head looking back the old way for a moment on
## a turn (degrees); `hand` a fist's follow-through past its strike (units); `neck` the neckwear swinging (radians).
var springs := _make_springs()
var prev_vel := Vector2.ZERO
var prev_facing := 0
var prev_state_frame := -1
var prev_hitlag := 0


static func _make_springs() -> RefCounted:
	var m := MotionSprings.new()
	m.add("hat_roll", 300.0, 14.0, -0.5, 0.5)
	m.add("hat_drop", 500.0, 18.0, -0.12, 0.15)
	m.add("spine", 240.0, 15.0, -8.0, 8.0)
	m.add("head", 170.0, 11.0, -12.0, 12.0)
	m.add("head_turn", 160.0, 13.0, -16.0, 16.0)
	m.add("hand", 260.0, 12.0, -0.06, 0.16)
	m.add("neck", 110.0, 6.5, -0.6, 0.6)
	return m


## Puts every piece of secondary motion back to rest: on a rebuild, and for whoever drives the view across a discontinuity it cannot see
## in the frame numbers (the frame jumps already rest the springs and smears on their own).
func reset_motion() -> void:
	springs.reset()
	prev_facing = 0
	prev_state_frame = -1
	body_k = 0.0
	body_frame = -1
	smear_pts.clear()
	smear_frame = -1
	flinch_left = 0.0
	for t in tails:
		t.reset()
## Where the run cycle was last frame, for the footfalls.
var last_step_phase := 0.0


## `s` is a dictionary of sim state (see main.gd `_refresh`).
func apply(pos: Vector3, s: Dictionary, delta: float) -> void:
	# The old built body's brawler has longer limbs: swap to its rig the first time we see its class. (The base body is one rig for every
	# class, so it never rebuilds here: a rebuild on the first frame of a match also left that fighter's per-mesh shader values, the ink
	# pass's object id among them, stale on part of the body, which drew a line round its face.)
	if rig != null and not use_base_body and (_cls(s) == 1) != long_limbs:
		long_limbs = _cls(s) == 1
		rebuild(loadout)
	position = pos
	var state: String = s.state
	# Vanish: nothing to see while it teleports.
	if model != null:
		model.visible = not (_cls(s) == 2 and state == "Attack" and s.move_name == "up special"
			and int(s.state_frame) >= VANISH_GONE[0] and int(s.state_frame) <= VANISH_GONE[1])
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
	# Leaving the jump squat: a kick into a tall stretch, so the take-off springs up out of the crouch rather than only rebounding.
	if last_state == "JumpSquat" and state != "JumpSquat" and not grounded:
		squash_vel -= TAKE_OFF_KICK

	var target_squash: float = SQUASH.get(state, 0.0)
	# Landing out of an aerial or a fast fall stumbles: deeper, and pitched forward, for its whole landing lag.
	if state == "Landing" and last_state != "Landing":
		heavy_landing = last_state == "Attack" or s.fast_fall
	if state != "Landing":
		heavy_landing = false
	if heavy_landing:
		target_squash = 0.28
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
	# Standing up from the floor: the body rises from lying (the knockdown's lean) as the `getup` clip sits it up and gets the feet
	# under it.
	if state == "GetUp" and dodge_progress(s) >= 0.0:
		target_lean = LEAN.Knockdown * (1.0 - smoothstep(0.0, 0.55, dodge_progress(s)))
	if heavy_landing:
		target_lean = 8.0
	# The whole body swings with the hit (in the manner of the reference game's big aerials): it pitches into a hit in front or below,
	# arches back under one overhead and curls forward away from one behind, as far as the limb is into its reach.
	if state == "Attack" and s.move_tip != Vector3.ZERO:
		var tip: Vector3 = s.move_tip
		var phi := rad_to_deg(atan2(tip.x, -(tip.y - 1.1)))   # 0 straight down, 90 ahead, 180 overhead, negative behind
		var swing := clampf((130.0 - phi) * 0.32, -22.0, 30.0) if phi >= 0.0 else clampf(-phi * 0.18, 0.0, 22.0)
		# (A body with legs leans with its spine and hips, from its clips and the body-follow; tipping the whole model would pivot it on
		# its feet, so the base body keeps only a little of this.)
		target_lean += swing * _reach_weight(s) * (0.35 if use_base_body else 1.0)
	# The attack lunge: drawn back while winding up, thrown forward through the strike.
	var target_lunge := 0.0
	if state == "Attack" and s.move_timing[1] > 0:
		var first: float = s.move_timing[1]
		var last: float = s.move_timing[2]
		var f: float = s.state_frame
		# (Less on the base body, whose legs step into the strike in its clips.)
		var lunge_k := 0.45 if use_base_body else 1.0
		if f < first and f >= first * 0.5:
			target_lunge = -0.12 * lunge_k
		elif f >= first and f <= last + 2.0:
			target_lunge = 0.32 * lunge_k
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
		step_squash = 0.07 * cos((phase - 0.06) * 4.0 * PI)
		# A puff of dust behind each foot as it lands (at 0.06 and 0.56 of the cycle).
		for land_at in [0.06, 0.56]:
			var crossed: bool = (last_step_phase < land_at and phase >= land_at) or (last_step_phase > phase and (phase >= land_at or last_step_phase < land_at))
			if crossed and grounded and not dusts.is_empty():
				_puff(-float(facing), 1)
		last_step_phase = phase
	# Volume-preserving: squashing down widens the body.
	var total := squash + step_squash
	model.scale = Vector3(1.0 + 0.5 * total, 1.0 - total, 1.0 + 0.5 * total)
	_secondary(s, delta)
	# Skidding out of a run, the fighter still faces the way it was running (the simulation has already turned it), leaning back
	# against the slide; it swings round as the skid ends.
	var look := facing
	if state == "Turn" and absf(float(s.vel.x)) > 0.08 and signf(float(s.vel.x)) == -float(facing):
		look = -facing
	var into := 1.0 if state == "Attack" or state == "Grabbing" else 0.0
	attack_turn = move_toward(attack_turn, into, delta * (12.0 if into > attack_turn else 5.0))
	yaw = lerp_angle(yaw, float(look) * deg_to_rad(BODY_TURN + BODY_TURN_IN_ATTACK * attack_turn), clampf(delta * 14.0, 0.0, 1.0))
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
	# Kicking off a wall (the sim counts wall jumps; a wall-tech jump leaves the tech going up): a back flip away from the wall.
	var wall_jumps := int(s.get("wall_jumps", 0))
	var kicked_off: bool = (wall_jumps > prev_wall_jumps and prev_wall_jumps >= 0) or (last_state == "WallTech" and state == "Airborne" and vy > 0.0)
	if kicked_off:
		flip_left = FLIP_TIME
		flip_back = true
	prev_wall_jumps = wall_jumps
	prev_jumps = jumps
	if grounded or not (state == "Airborne" or state == "LedgeJump"):
		flip_left = 0.0
	if flip_left <= 0.0:
		flip_back = false
	flip_left = maxf(0.0, flip_left - delta)
	var flip := 0.0
	if flip_left > 0.0:
		var k := 1.0 - flip_left / FLIP_TIME
		flip = TAU * k * k * (3.0 - 2.0 * k) * (-1.0 if flip_back else 1.0)
	# The ledge: a swing on the arms when it is caught, a forward roll for a ledge roll (the clip curls the body up).
	var pivot := Vector3(0, 1.1, 0)
	if ledge_swing_t >= 0.0 and state == "LedgeHang":
		ledge_swing_t += delta
		flip += deg_to_rad(22.0) * exp(-ledge_swing_t * 3.2) * sin(ledge_swing_t * 10.0)
		pivot = Vector3(0, 2.0, 0)
	if ledge_rolling and state == "LedgeGetUp":
		flip += TAU * ledge_k
		pivot = Vector3(0, 0.6, 0)
	# A roll is a somersault the way it goes (forward, or backward for a back roll), about the curled body's middle, over the roll's
	# moving frames; the `roll` clip curls the body up for it.
	var dodge := dodge_progress(s)
	if state == "Roll" and dodge >= 0.0:
		var way: int = (s.dodge as PackedInt32Array)[1]
		flip += (-1.0 if way < 0 else 1.0) * TAU * smoothstep(ROLL_SPIN_FROM, ROLL_SPIN_TO, dodge)
		# (About the curled ball's middle, high enough that the head clears the floor as it comes round.)
		pivot = Vector3(0, 0.8, 0)
	model.rotation = Vector3(deg_to_rad(lean) + flip, yaw, 0)
	# A spinning neutral air turns the whole body over once backward, in the plane of the stage (see NAIR_SPIN).
	var spin := _spin(s)
	if spin != 0.0:
		model.quaternion = Quaternion(Vector3(0, 0, 1), spin * float(facing)) * model.quaternion
		pivot = SPIN_PIVOT_3D
	# Spin about the middle of the body (or the hands, or the curled-up roll) rather than the feet.
	flip_offset = Vector3.ZERO
	if flip != 0.0 or spin != 0.0:
		flip_offset = pivot - Basis(model.quaternion) * pivot
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
	# Lying down, the body (pitched over by its lean about the feet) is raised so its arms and chest rest on the floor, not in it.
	if state == "Knockdown":
		model.position.y += KNOCKDOWN_LIFT
	elif state == "GetUp" and dodge >= 0.0:
		model.position.y += KNOCKDOWN_LIFT * (1.0 - smoothstep(0.0, 0.55, dodge))
	# A spot dodge steps back into the stage (away from the camera) and back out, so it reads as getting out of the way.
	if state == "SpotDodge" and dodge >= 0.0:
		model.position.z -= SPOT_DODGE_STEP * sin(PI * dodge)
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
	if name_label != null and not match_tag:
		name_label.text = text
		name_label.visible = text != ""


## What floats over the head. In a match only a player tag in the player's colour ("P1") over a pointer: the damage is on the match
## display, and the tag says who is who without relying on colour alone. It keeps one size on screen however far the camera pulls back
## (`TAG_PIXEL`), as a platform fighter's tags do, so a fighter far off is still easy to find. In training, the damage and the fighter's
## name, as before.
var match_tag := false
var tag_arrow: Sprite3D
const TAG_PIXEL := 0.0007
const TAG_HEIGHT := 2.6   # world units above the feet: just over the head, the tag and pointer stacked up from there on screen


func set_overhead(training: bool, tag: String, colour: Color) -> void:
	if percent_label == null or name_label == null or match_tag == not training:
		return
	match_tag = not training
	percent_label.visible = training
	if training:
		name_label.text = name_text
		name_label.visible = name_text != ""
		name_label.modulate = Color.WHITE
		name_label.position.y = 3.55
		name_label.fixed_size = false
		name_label.pixel_size = 0.0075
		name_label.offset = Vector2.ZERO
		name_label.outline_modulate = Color.BLACK
		if tag_arrow != null:
			tag_arrow.visible = false
	else:
		name_label.text = tag
		name_label.visible = true
		name_label.modulate = colour.lightened(0.1)
		name_label.outline_modulate = INK
		name_label.outline_size = 18
		name_label.position.y = TAG_HEIGHT
		name_label.fixed_size = true
		name_label.pixel_size = TAG_PIXEL
		name_label.offset = Vector2(0, 46)
		if tag_arrow == null:
			tag_arrow = Sprite3D.new()
			tag_arrow.texture = _tag_arrow_texture()
			tag_arrow.billboard = BaseMaterial3D.BILLBOARD_ENABLED
			tag_arrow.no_depth_test = true
			tag_arrow.fixed_size = true
			tag_arrow.pixel_size = TAG_PIXEL
			tag_arrow.offset = Vector2(0, 8)
			tag_arrow.render_priority = name_label.render_priority
			add_child(tag_arrow)
		tag_arrow.position.y = TAG_HEIGHT
		tag_arrow.modulate = colour.lightened(0.1)
		tag_arrow.visible = true


## The tag's pointer: a white downward triangle with a dark ink rim (tinted the player's colour by `modulate`; the rim stays dark).
static func _tag_arrow_texture() -> ImageTexture:
	if _tag_arrow_cache != null:
		return _tag_arrow_cache
	var svg := '<svg xmlns="http://www.w3.org/2000/svg" width="40" height="28"><path d="M5,5 L35,5 L20,23 Z" fill="#ffffff" stroke="#%s" stroke-width="5" stroke-linejoin="round"/></svg>' % INK.to_html(false)
	var img := Image.new()
	img.load_svg_from_string(svg, 1.0)
	_tag_arrow_cache = ImageTexture.create_from_image(img)
	return _tag_arrow_cache


static var _tag_arrow_cache: ImageTexture
var blade_pivot: Node3D
var blade_parts: Array[MeshInstance3D] = []
var staff_parts: Array[MeshInstance3D] = []
## The class that fights with a staff (the Seer): posed like the sword, drawn as a staff.
const STAFF_CLASS := 5
## How many times the Seer's neutral air whirls the staff round.
const STAFF_NAIR_TURNS := 2.0


## The fighter's class (which moveset it has): 0 longsword, 1 claws. Snapshots made before classes existed only have `char`.
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
## (None now: the neutral air is a held kick with the other leg tucked, as the reference's are, which the clip poses.)
const SPLIT_KICKS: Array[String] = []
## (Its up air and down air are claw slashes, overhead and down below, so they swing the arm.)
const BRAWLER_NO_BLADE := ["utilt", "dtilt", "dash attack", "nair", "bair", "dair", "usmash", "side special", "up special", "down special", "grab", "dash grab", "pummel", "forward throw", "back throw", "up throw", "down throw"]
## Moves that rush the whole body forward in a flame.
const BRAWLER_FLAME := ["side special", "up special"]
var last_percent := -1
var blade_angle := REST_ANGLE
var blade_length := 2.0


func _rest_length(s: Dictionary) -> float:
	# Swords are long; claws and bare hands are short.
	if _cls(s) == STAFF_CLASS:
		return 2.0
	return clampf(float(s.reach) - HAND.x - 0.65, 0.7, 3.4) if _cls(s) == 0 else 1.0


## Where the blade should point, in degrees in forward space, and how long it should be, for this frame.
func _blade_target(s: Dictionary) -> Array:
	var rest_len := _rest_length(s)
	var name: String = s.move_name
	if name == "" or (_cls(s) == STAFF_CLASS and AnimLibrary.is_kick(AnimLibrary.attack_clip(_cls(s), name, anim))):
		return [REST_ANGLE, rest_len, HAND.x]
	# The Seer's neutral air: the staff whirled round the body, held at its middle, turning from the wind-up through the last hit.
	if _cls(s) == STAFF_CLASS and name == "nair":
		var tt: PackedInt32Array = s.move_timing
		var from := maxf(1.0, float(tt[1]) * 0.5)
		var to := maxf(from + 1.0, float(tt[2]) + 2.0)
		var k := clampf((float(s.state_frame) - from) / (to - from), 0.0, 1.0)
		var turns := STAFF_NAIR_TURNS * (k * k * (3.0 - 2.0 * k))
		return [REST_ANGLE + 360.0 * turns, rest_len, HAND.x - rest_len * 0.5 * cos(deg_to_rad(REST_ANGLE + 360.0 * turns))]
	var tip: Vector3 = s.move_tip
	var hand_x := HAND.x if tip.x >= 0.0 else -0.35
	var to_tip := Vector2(tip.x - hand_x, tip.y - HAND.y)
	var swing := rad_to_deg(atan2(to_tip.y, to_tip.x))
	var length := clampf(to_tip.length() + tip.z * 0.7, 0.7, 3.6)
	if _cls(s) == STAFF_CLASS:
		length = clampf(length, 1.6, 2.6)
	elif _cls(s) != 0:
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
	# A thrust does not swing: the blade points at the hit from the start and the arm drives it in (see `_pose_blade`).
	if _cls(s) == 0 and SWORD_THRUSTS.has(name):
		wind = swing
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
## The sword's thrusts: the blade points at the hit throughout and the arm drives it in, rather than swinging round to it.
const SWORD_THRUSTS := ["usmash", "dtilt"]
## Neutral airs whose second hit circles the body (the sword's): the whole body flips over backward once while the arm holds the blade
## out, between these frames, so the blade follows the hits round. By class.
## (None now: the sword's neutral air no longer turns the body upside down. Ours is a windmill slash: the body stays upright and the arm
## sweeps the blade round the whole circle its hits make, following the live hitbox.)
const NAIR_SPIN := {}
const SPIN_PIVOT := Vector2(0.0, 1.1)
const SPIN_PIVOT_3D := Vector3(0.0, 1.1, 0.0)


## How far round the backward flip of a spinning neutral air the body is (radians, 0 when not spinning).
func _spin(s: Dictionary) -> float:
	if String(s.state) != "Attack" or String(s.move_name) != "nair" or not NAIR_SPIN.has(_cls(s)):
		return 0.0
	var w: Array = NAIR_SPIN[_cls(s)]
	var k := (float(s.state_frame) - float(w[0])) / float(int(w[1]) - int(w[0]))
	if k <= 0.0 or k >= 1.0:
		return 0.0
	return TAU * k


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
	if attacking and (_cls(s) != 0 or SWORD_THRUSTS.has(String(s.move_name))):
		arm_k = _reach_weight(s)
	# While the body spins (see NAIR_SPIN) the frame the blade is posed in turns with it, so the tip is turned back by as much: the arm
	# holds the blade out and the spin carries it round the circle the hits make.
	var spin := _spin(s)
	if spin != 0.0:
		tip = SPIN_PIVOT + (tip - SPIN_PIVOT).rotated(-spin)
	# The swing's arc (see ARCS): the sword's follows where its tip is; a claw is swept along its arc (its hitbox holds still while the
	# hand rakes through it, as the reference's do).
	var now := _arc_now(s, tip)
	if _cls(s) != 0 and not now.is_empty():
		tip = _arc_point(now)
	var hand := old_hand
	if rig != null and arm_k > 0.0:
		var to_tip := tip - shoulder
		var dist := maxf(to_tip.length(), 0.001)
		# (A fist reaches further by the strike's stretch, `strike_e`, so the stretched arm is straight on the hit.)
		var reach := minf(arm_reach, maxf(dist - 0.3, 0.1)) if _cls(s) == 0 else arm_reach * (1.0 + strike_e)
		var goal := shoulder + to_tip / dist * reach
		if use_base_body:
			# With a real arm the hand travels round the shoulder, in an arc (a hook, a swing), not along a straight line.
			var from_v := old_hand - shoulder
			var to_v := goal - shoulder
			var angle := lerp_angle(from_v.angle(), to_v.angle(), arm_k)
			hand = shoulder + Vector2.from_angle(angle) * lerpf(from_v.length(), to_v.length(), arm_k)
		else:
			hand = old_hand.lerp(goal, arm_k)
	# A fist carries on a little past its strike and settles (the `hand` spring, kicked as the hit ends).
	if _cls(s) != 0 and absf(springs.value("hand")) > 0.001:
		hand += (hand - shoulder).normalized() * springs.value("hand")
	var along := tip - hand
	var length := maxf(along.length(), 0.3)
	var swing := rad_to_deg(along.angle())
	# Forward space to model space: facing left mirrors the pose about the vertical axis.
	var angle := swing if facing > 0 else 180.0 - swing
	hand_target = Vector3(hand.x * facing, hand.y, 0.35)
	_update_arc(now, delta)
	blade_pivot.position = hand_target
	blade_pivot.rotation = Vector3(0, 0, deg_to_rad(angle))
	var thick := 1.0 if _cls(s) == 0 or _cls(s) == STAFF_CLASS else 1.7
	blade_pivot.scale = Vector3(length / MESH_LENGTH, thick, thick)


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
## Kicks the secondary-motion springs from what the fighter did this sim frame, moves them on, and places the hat and the neckwear (the
## head and spine are bent in `_animate`, after the clip; the hand in `_pose_blade`).
func _secondary(s: Dictionary, delta: float) -> void:
	var vel := Vector2(float(s.vel.x), float(s.vel.y))
	var facing := int(s.facing)
	springs.begin(int(s.frame), int(s.hitlag), delta)
	var dv := vel - prev_vel
	# (A change bigger than any move makes is a teleport, a respawn or a correction: not something to swing from.)
	if springs.new_frames > 0 and dv.length() < 0.7:
		springs.kick("hat_roll", clampf(dv.x, -0.5, 0.5) * 60.0)
		springs.kick("hat_drop", clampf(dv.y, -1.0, 1.0) * 6.0)
		# Speeding up the way it faces leaves the chest and head behind (bent back); stopping throws them forward; landing nods them.
		var ahead := dv.x * float(facing)
		var landing := maxf(dv.y, 0.0) if int(s.platform) >= 0 else 0.0
		springs.kick("spine", -ahead * 320.0 + landing * 90.0)
		springs.kick("head", -ahead * 480.0 + landing * 160.0)
		springs.kick("neck", -dv.x * 40.0)
		if prev_facing != 0 and facing != prev_facing:
			springs.kick("head_turn", float(prev_facing) * 420.0)
	# A hit on the shield: as its freeze ends the chest and head are knocked back behind it and come forward again.
	if springs.new_frames > 0 and prev_hitlag > 0 and int(s.hitlag) == 0 and String(s.state) == "Shield":
		springs.kick("spine", -90.0 - 6.0 * float(prev_hitlag))
		springs.kick("head", -110.0 - 8.0 * float(prev_hitlag))
	if springs.new_frames > 0 or int(s.hitlag) > 0:
		prev_hitlag = int(s.hitlag)
	# A strike's follow-through: as a fist's hit ends the hand carries on past it and settles back.
	var f := int(s.state_frame)
	if String(s.state) == "Attack" and _cls(s) != 0 and prev_state_frame >= 0:
		var last := int(s.move_timing[2])
		if prev_state_frame <= last and f > last:
			springs.kick("hand", 1.9)
	prev_state_frame = f if String(s.state) == "Attack" else -1
	prev_vel = vel
	prev_facing = facing
	springs.step(delta)
	if hat_pivot != null:
		hat_pivot.rotation = Vector3(0, 0, springs.value("hat_roll"))
		hat_pivot.position = HAT_PIVOT + Vector3(0, -springs.value("hat_drop"), 0)
	if neck_pivot != null:
		neck_pivot.rotation = Vector3(0, 0, springs.value("neck"))
	for t in tails:
		t.update(int(s.frame), int(s.hitlag), dv, 1.0, delta)


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
		# On a strike the arm stretches (`strike_e`, the fists only: a blade's reach is the blade), and reaches that much further.
		var e := strike_e if _cls_now != 0 else 0.0
		var upper := (elbow0 - rest_u.origin).length() * (1.0 + e)
		# The shoulder where the animated chest has it now, so the arm stays on the body as it bends and turns.
		var shoulder := rest_u.origin
		var parent := skeleton.get_bone_parent(iu)
		if parent >= 0:
			shoulder = skeleton.get_bone_global_pose(parent) * skeleton.get_bone_rest(iu).origin
		var fore_len := (wrist0 - elbow0).length() * (1.0 + e)
		var lower := fore_len + 0.07
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
		var stretch := Basis.from_scale(MotionDeform.limb_scale(e))
		skeleton.set_bone_global_pose_override(iu, Transform3D(Basis(q_upper) * rest_u.basis * stretch, shoulder), 1.0, true)
		skeleton.set_bone_global_pose_override(il, Transform3D(Basis(q_lower) * rest_l.basis * stretch, elbow), 1.0, true)
		skeleton.set_bone_global_pose_override(ih, Transform3D(Basis(q_lower) * rest_h.basis, elbow + fore * fore_len), 1.0, true)
		if _cls_now != 0:
			strike_tip = to_local(skeleton.global_transform * (elbow + fore * fore_len))
			strike_tip_ok = true


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
		# On the strike the kicking leg stretches and reaches that much further (`strike_e`).
		var e := strike_e * MotionDeform.LEG_STRETCH_FACTOR * leg_k
		var upper := (knee0 - rest_t.origin).length() * (1.0 + e)
		var shin_len := (ankle0 - knee0).length() * (1.0 + e)
		var lower := shin_len + 0.08
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
		var stretch := Basis.from_scale(MotionDeform.limb_scale(e))
		skeleton.set_bone_global_pose_override(it, Transform3D(Basis(q_upper) * rest_t.basis * stretch, hip), leg_k, true)
		skeleton.set_bone_global_pose_override(ish, Transform3D(Basis(q_lower) * rest_s.basis * stretch, knee), leg_k, true)
		skeleton.set_bone_global_pose_override(ift, Transform3D(Basis(q_lower) * rest_f.basis, knee + shin_dir * shin_len), leg_k, true)
		if side == kicker:
			strike_tip = to_local(skeleton.global_transform * (knee + shin_dir * shin_len))
			strike_tip_ok = true


# ---- Swing arcs --------------------------------------------------------------------------------------------------------------------
# A swing leaves the crescent the reference game draws: one smooth arc where the weapon's tip went, pointed at both ends and thickest a
# little behind the leading edge, a bright core along the outside fading through the trail colour to nothing on the inside. Each arc is
# authored per move (`ARCS`; the sword's are fitted to its hitboxes, the claws' to their animations): a circle about the shoulder, the
# angles it sweeps and the active frames it is drawn on. It grows with the hit, from the frame before the first active frame to the last,
# then fades from the tail over `ARC_FADE` frames. It depends only on the move and its frame, so hitlag freezes it with the swing and a
# replay seek redraws it; a move cut short (a landing) lets its arc fade on its own. Moves with no arc (thrusts, punches, kicks) draw
# none.

const ARC_FADE := 5
const ARC_STEPS := 36
const ARC_Z := 0.6
const ARC_PAST_HAND := 0.8
## By class, then move (as the sim names it): its arcs, each [centre x, centre y, radius, from degrees, to degrees, first active frame,
## last active frame] in forward space (x forward and y up from the feet; 0 degrees is forward, 90 up), frames numbered as in the
## reference's frame data. An arc marked "arm" is the claw's own path: about the fighter's shoulder, just past the hand's reach.
const ARCS := {
	0: {
		"jab": [[0.5, 1.15, 2.3, 75, -15, 5, 6]],
		"jab 2": [[0.5, 1.15, 2.3, -25, 40, 4, 5]],
		"ftilt": [[0.45, 1.1, 2.35, -80, 60, 8, 11]],
		"utilt": [[0.1, 1.2, 2.1, -10, 165, 6, 10]],
		"dash attack": [[0.55, 1.1, 2.35, -85, 70, 13, 16]],
		"fsmash": [[0.7, 1.15, 2.3, 135, -45, 10, 13]],
		"dsmash": [[0.0, 2.7, 2.6, -100, -35, 6, 7], [0.0, 2.7, 2.6, -80, -145, 21, 23]],
		"nair": [[0.4, 1.1, 2.2, -80, 50, 6, 7], [0.15, 1.1, 2.35, 55, 350, 15, 21]],
		"fair": [[0.5, 1.05, 2.3, 140, -60, 6, 8]],
		"bair": [[0.0, 1.1, 2.25, 260, 100, 7, 11]],
		"uair": [[0.1, 1.3, 2.2, -30, 180, 5, 9]],
		"dair": [[0.15, 1.1, 2.2, 30, -180, 9, 13]],
	},
	1: {
		"ftilt": [[0.0, 0.0, 0.0, 50, -50, 8, 10, "arm"]],
		"fsmash": [[0.0, 0.0, 0.0, 60, -60, 20, 23, "arm"]],
		"dsmash": [[0.0, 2.6, 2.4, -100, -35, 14, 15], [0.0, 2.6, 2.4, -80, -145, 21, 22]],
		"fair": [[0.0, 0.0, 0.0, 60, -70, 7, 9, "arm"]],
		"bair": [[0.0, 0.0, 0.0, 215, 140, 13, 15, "arm"]],
		"uair": [[0.0, 0.0, 0.0, 15, 165, 7, 9, "arm"]],
		"dair": [[0.0, 0.0, 0.0, 20, -110, 16, 17, "arm"]],
	},
	# Needles: fitted to its hitboxes (moves.rs `needles`), circles about the hip for the kicks and the shoulder for the hand strikes.
	# The up air draws one arc a kick: its three looping kicks and the final one.
	2: {
		"jab": [[0.2, 1.15, 0.85, 40, -10, 2, 2]],
		"jab 2": [[0.2, 1.0, 1.15, -35, 10, 3, 3]],
		"extra move 2": [[0.2, 0.9, 1.2, -15, 50, 5, 6]],
		"ftilt": [[0.0, 0.75, 1.25, -55, 30, 5, 8]],
		"utilt": [[0.0, 0.8, 1.45, -5, 85, 5, 8], [0.0, 0.8, 1.1, 105, 15, 16, 23]],
		"dtilt": [[0.0, 0.85, 1.25, -95, -30, 5, 6]],
		"dash attack": [[0.2, 1.0, 0.8, 55, -45, 6, 8]],
		"fsmash": [[0.0, 0.8, 1.0, -60, 10, 12, 12], [0.0, 0.85, 1.45, 60, 0, 20, 21]],
		"usmash": [[0.0, 1.0, 1.3, 35, 95, 11, 11], [0.0, 1.5, 0.95, -40, -140, 14, 15]],
		"dsmash": [[0.0, 2.0, 1.4, -100, -45, 8, 9], [0.0, 2.0, 1.4, -80, -135, 15, 16]],
		"nair": [[0.0, 0.9, 0.95, -50, -5, 3, 6]],
		"fair": [[0.2, 1.0, 0.85, 110, 0, 5, 7]],
		"bair": [[0.0, 0.9, 1.0, 220, 180, 4, 6]],
		"uair": [[0.1, 1.1, 1.25, 50, 110, 4, 6], [0.1, 1.1, 1.25, 130, 70, 8, 10], [0.1, 1.1, 1.25, 50, 110, 12, 14],
			[0.1, 1.0, 1.2, 20, 125, 23, 24]],
		"dair": [[0.0, 1.0, 0.95, -55, -95, 15, 18]],
		"down special": [[0.1, 1.0, 1.0, 20, -95, 23, 29]],
	},
}

## The Needles' Vanish: the fighter is gone (move frames, from 0) while it teleports, between the explosion and the reappearance.
const VANISH_GONE := [36, 54]
var arc: MeshInstance3D
## The arc drawn now: {"w": its ARCS entry, "k": how far it has grown 0..1, "q": how far it has faded 0..1, "facing", "cls"}, or empty.
var arc_shown := {}


func _update_arc(now: Dictionary, delta: float) -> void:
	if not now.is_empty() and float(now.q) < 1.0:
		arc_shown = now
	elif not arc_shown.is_empty():
		# The move ended, was cut short (a landing) or its arc is done: what is left fades on its own.
		arc_shown.q = maxf(float(arc_shown.q), 0.0) + delta * 60.0 / ARC_FADE
		if float(arc_shown.q) >= 1.0:
			arc_shown = {}
	_draw_arc()


## The latest of the move's arcs that has begun (two frames before its first active frame): {"w": its ARCS entry, "k": how far round
## its head is 0..1, "q": how far it has faded 0..1 (1 and more: gone), "facing", "cls"}, or empty. The sword's head is where its tip is
## (never going back); a claw's moves round at an even pace and reaches the end on the last active frame.
func _arc_now(s: Dictionary, tip: Vector2) -> Dictionary:
	if String(s.state) != "Attack":
		return {}
	var windows: Array = (ARCS.get(_cls(s), {}) as Dictionary).get(String(s.move_name), [])
	var f := float(s.state_frame) + 1.0
	var out := {}
	for w in windows:
		var first := float(w[5])
		var last := float(w[6])
		if f < first - 2.0:
			continue
		var k := clampf((f - first + 2.0) / (last - first + 2.0), 0.0, 1.0)
		if _cls(s) == 0:
			var from := deg_to_rad(float(w[3]))
			var span := deg_to_rad(float(w[4])) - from
			var ang := (tip - (_arc_circle(w)[0] as Vector2)).angle()
			var d := wrapf((ang - from) * signf(span), -PI, PI)
			var prev := float(arc_shown.k) if not arc_shown.is_empty() and arc_shown.w == w else 0.0
			# Of the two ways round, the one nearer where the head was (a long arc, like the neutral air's, goes past half a turn).
			if absf(d + TAU - prev * absf(span)) < absf(d - prev * absf(span)):
				d += TAU
			k = maxf(prev, clampf(d / absf(span), 0.0, 1.0))
		out = {"w": w, "k": k, "q": (f - last) / ARC_FADE, "facing": float(s.facing), "cls": _cls(s)}
	return out


## Where an arc's head is (forward space).
func _arc_point(now: Dictionary) -> Vector2:
	var w: Array = now.w
	var ang := lerpf(deg_to_rad(float(w[3])), deg_to_rad(float(w[4])), float(now.k))
	var g := _arc_circle(w)
	return (g[0] as Vector2) + Vector2.from_angle(ang) * float(g[1])


## An arc's circle: [centre, radius].
func _arc_circle(w: Array) -> Array:
	if w.size() > 7 and w[7] == "arm":
		return [shoulder, arm_reach + ARC_PAST_HAND]
	return [Vector2(float(w[0]), float(w[1])), float(w[2])]


func _draw_arc() -> void:
	if arc == null:
		arc = MeshInstance3D.new()
		arc.mesh = ImmediateMesh.new()
		arc.top_level = true
		arc.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		arc.extra_cull_margin = 16384.0
		arc.material_override = FxMaterial.get_material(false, 5)
		add_child(arc)
	var im: ImmediateMesh = arc.mesh
	if im.get_surface_count() > 0:
		im.clear_surfaces()
	if arc_shown.is_empty():
		return
	var w: Array = arc_shown.w
	var q := clampf(float(arc_shown.q), 0.0, 1.0)
	# The fading arc is eaten from the tail toward the head.
	var head := lerpf(deg_to_rad(float(w[3])), deg_to_rad(float(w[4])), float(arc_shown.k))
	var tail := lerpf(deg_to_rad(float(w[3])), head, q * 0.75)
	if absf(head - tail) < 0.03:
		return
	var circle := _arc_circle(w)
	var centre: Vector2 = circle[0]
	var r: float = circle[1]
	var width := r * 0.3 * (1.0 - 0.4 * q)
	var alpha := 1.0 - q
	var colours := trail_colours_of(int(arc_shown.cls))
	var main: Color = colours[0]
	var core: Color = colours[1]
	# Across the arc from the outside in (fractions of its thickness there): the bright core, the core blending into the trail colour,
	# the trail colour, and nothing.
	var bands := [0.0, 0.22, 0.55, 1.0]
	var cols := [Color(core, alpha), Color(core.lerp(main, 0.5), alpha), Color(main, 0.8 * alpha), Color(main, 0.0)]
	var facing := float(arc_shown.facing)
	var prev: Array = []
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
	for i in ARC_STEPS + 1:
		var u := float(i) / ARC_STEPS
		var dir := Vector2.from_angle(lerpf(tail, head, u))
		# Pointed at both ends, thickest a little behind the head.
		var thick := width * pow(sin(PI * pow(u, 1.4)), 0.8)
		var column: Array = []
		for b in bands:
			var p := centre + dir * (r - thick * float(b))
			column.append(to_global(Vector3(p.x * facing, p.y, ARC_Z)))
		if i > 0:
			for b in bands.size() - 1:
				var quad := [[prev[b], cols[b]], [prev[b + 1], cols[b + 1]], [column[b + 1], cols[b + 1]], [column[b], cols[b]]]
				for idx in [0, 1, 2, 0, 2, 3]:
					im.surface_set_color(quad[idx][1])
					im.surface_add_vertex(quad[idx][0])
		prev = column
	im.surface_end()


# ---- Big moments: dizzy stars, the launch stretch and speed lines --------------------------------------------------------------------

## Three cartoon stars circling the head while the shield is broken.
var dizzy: Node3D
## A strong launch: the body stretched along the way it flies, and streaks trailing behind it.
var launch_frame: Node3D
var launch_lines: Node3D
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


## A strong hit reads in the body (`MotionDeform.body_target`): squashed along the blow through the victim's freeze, released into a
## stretch along its flight (up to 42% longer, thinner across, about its middle) while it flies fast, relaxing as it slows; streaks trail
## behind it, on twos. The deformation eases in and out (`body_k`), and jumps straight to where it should be after a rollback or seek.
var body_k := 0.0
var body_dir := Vector2.UP
var body_frame := -1


func _launch_look(s: Dictionary) -> void:
	var vel := Vector2(float(s.vel.x), float(s.vel.y))
	var speed := vel.length()
	var flying: bool = String(s.state) == "Hitstun" and int(s.hitlag) == 0
	var flight: Vector2 = vel + s.get("kb_vel", Vector2.ZERO)
	var target: Array = MotionDeform.body_target(String(s.state), int(s.hitlag), bool(s.launch_pending), flight)
	var frame := int(s.frame)
	var steps := frame - body_frame
	if body_frame < 0 or steps < 0 or steps > 2:
		body_k = float(target[0])
		steps = 0
	# Per sim frame (so hitlag and a paused game hold it): into the squash fast, out into the stretch fast, relaxing slower.
	for _i in steps:
		var rate := 0.6 if float(target[0]) < 0.0 or float(target[0]) > body_k else 0.25
		body_k = lerpf(body_k, float(target[0]), rate)
	body_frame = frame
	if absf(float(target[0])) > 0.001:
		body_dir = target[1]
	launch_frame.transform = MotionDeform.along(body_k, body_dir, Vector3(0, 1.1, 0))
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


## A class's trail colours: [the trail, its bright core] (claws violet, needles teal, sword blue).
static func trail_colours_of(cls: int) -> Array:
	if cls == 1:
		return [Color(0.78, 0.3, 1.0), Color(1.0, 0.9, 1.0)]
	if cls == 2:
		return [Color(0.25, 0.85, 0.75), Color(0.9, 1.0, 0.97)]
	return [Color(0.3, 0.66, 1.0), Color(0.92, 0.98, 1.0)]


## How stretched the striking limb is this frame (`MotionDeform.strike_stretch`), and the fighter's class, for the arm and leg IK.
var strike_e := 0.0
var _cls_now := 0
## Where the striking fist or foot is this frame (local), when the arm or leg IK placed one.
var strike_tip := Vector3.ZERO
var strike_tip_ok := false
## The smear ribbon behind a fist or foot (`Smear`): its recent points (local, oldest first), how far it has faded, the class it is coloured
## for, and the sim frame it last moved on.
var smear_pts: Array = []
var smear_q := 1.0
var smear_cls := 0
var smear_frame := -1


## A strike with no authored arc (a punch or a kick of the classes without a blade) leaves a smear behind the striking fist or foot: drawn
## out a point a sim frame from `Smear.LEAD` frames before its hit (so it runs from the wind-up) to the hit, held while the hit is live,
## then faded over `ARC_FADE` frames. Nothing moves in hitlag, so a freeze holds it; a rollback or seek drops it.
func _update_smear(s: Dictionary) -> void:
	var frame := int(s.frame)
	if smear_frame >= 0 and (frame < smear_frame or frame - smear_frame > 2):
		smear_pts.clear()
	var advanced := frame != smear_frame
	smear_frame = frame
	# Drawn out over the throw (from the wind-up to the hit's first frame), held while the hit is live, then faded.
	var throwing := false
	var holding := false
	if String(s.state) == "Attack" and _cls(s) != 0 and strike_tip_ok and not (ARCS.get(_cls(s), {}) as Dictionary).has(String(s.move_name)):
		var f := float(s.state_frame)
		var first := float(s.move_timing[1])
		var last := float(s.move_timing[2])
		throwing = first > 0.0 and f >= first - Smear.LEAD and f <= first
		holding = first > 0.0 and f > first and f <= last + 1.0
	if advanced and int(s.hitlag) == 0 and not holding:
		if throwing:
			smear_pts.append(Vector3(strike_tip.x, strike_tip.y, ARC_Z))
			if smear_pts.size() > Smear.POINTS:
				smear_pts.pop_front()
			smear_q = 0.0
			smear_cls = _cls(s)
		elif not smear_pts.is_empty():
			smear_q += 1.0 / ARC_FADE
			if smear_q >= 1.0:
				smear_pts.clear()
	if smear_pts.size() >= 2 and arc != null:
		var colours := trail_colours_of(smear_cls)
		Smear.draw(arc.mesh, smear_pts, smear_q, colours[0], colours[1], to_global)


func _apply_combat(s: Dictionary, delta: float) -> void:
	var state: String = s.state
	var pct := int(s.percent)
	if pct != last_percent:
		# A hit that does not launch (a flinch, or one hit of a multi-hit move) snaps the body back.
		if last_percent >= 0 and pct > last_percent and not s.launch_pending and state != "Hitstun":
			flinch_left = FLINCH_TIME
		last_percent = pct
		percent_label.text = "%d%%" % pct
		var heat := clampf(pct / 150.0, 0.0, 1.0)
		percent_label.modulate = Color(1.0, 1.0 - 0.75 * heat, 1.0 - 0.95 * heat)
	strike_e = 0.0
	strike_tip_ok = false
	_cls_now = _cls(s)
	if state == "Attack" and s.move_timing[1] > 0:
		strike_e = MotionDeform.strike_stretch(float(s.state_frame), float(s.move_timing[1]), float(s.move_timing[2]))
	_pose_blade(s, delta)
	# The claws and needles fighters' kicks and rushes use the body, not a blade (the needles' stand-in specials are the claws', so they
	# burn and spin the same way).
	var staff: bool = _cls(s) == STAFF_CLASS
	var brawler: bool = _cls(s) != 0 and not staff
	for p in blade_parts:
		p.visible = not staff
	for p in staff_parts:
		p.visible = staff
	# Only the body moves leave the arms alone (the claws' listed ones; the needles' kicks, grabs and throws); the sword fighter always
	# has its blade.
	var body_move := false
	if _cls(s) == 1:
		body_move = BRAWLER_NO_BLADE.has(s.move_name)
	elif (_cls(s) == 2 or staff) and s.move_name != "":
		var clip := AnimLibrary.attack_clip(_cls(s), s.move_name, anim)
		body_move = AnimLibrary.is_kick(clip) or clip in ["grab", "throw", "land"]
	var swinging_arm: bool = not ((brawler or staff) and body_move)
	# (The Seer keeps its staff in hand through a kick, held at rest.)
	blade_pivot.visible = (not brawler and swinging_arm) or staff
	# Running with nothing to swing, the weapon arm pumps with the run and the weapon trails behind.
	var carrying: bool = not brawler and s.move_name == "" and (state == "Run" or state == "Dash")
	if rig != null:
		var holding: bool = swinging_arm if not brawler else (swinging_arm and state == "Attack")
		_aim_arm(int(s.facing), holding and not carrying)
		if carrying:
			_carry_weapon(int(s.facing))
		_aim_leg(s, state == "Attack" and AnimLibrary.is_kick(AnimLibrary.attack_clip(_cls(s), s.move_name, anim)), delta)
		_update_smear(s)
	# (The claws' flame rushes only: the Needles' specials are their own.)
	var rushing: bool = _cls(s) == 1 and state == "Attack" and BRAWLER_FLAME.has(s.move_name) and s.state_frame >= 12
	flame.emitting = rushing
	# The flame rush up: a corkscrew about the body's own upright axis (our take; the head stays up, the face comes round every turn).
	if _cls(s) == 1 and state == "Attack" and s.move_name == "up special" and s.state_frame >= 17:
		model.rotation.y = yaw + float(s.state_frame - 17) * 0.75

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
			var flight: Vector2 = s.vel + s.get("kb_vel", Vector2.ZERO)
			if flight.length() > LAUNCH_POSE_SPEED:
				# Thrown along the launch: arched back (the hurt pose), the head leading the way it flies and the limbs trailing,
				# as the reference game's fighters fly off a strong hit.
				var lead := atan2(flight.x * float(s.facing), flight.y)
				tumble_angle = lerp_angle(tumble_angle, lead, clampf(delta * 18.0, 0.0, 1.0))
			else:
				# Slowing down, it tumbles head over heels, faster the faster it still flies.
				tumble_angle -= clampf(flight.length() * 1.3, 0.12, 0.6) * delta * 60.0
			model.rotation.x = tumble_angle
			# About the middle of the body, not the feet (which would swing the body down through the floor).
			model.position += SPIN_PIVOT_3D - Basis.from_euler(model.rotation) * SPIN_PIVOT_3D
		elif current_clip == "hurt":
			model.rotation.x = -deg_to_rad(26.0)
	else:
		tumble_angle = 0.0
