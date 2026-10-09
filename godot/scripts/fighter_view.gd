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
static var _outline_mat: StandardMaterial3D


static func toon(c: Color, outline := true) -> StandardMaterial3D:
	var key := "%s%s" % [c.to_html(), outline]
	if _mat_cache.has(key):
		return _mat_cache[key]
	var m := StandardMaterial3D.new()
	_mat_cache[key] = m
	m.albedo_color = c
	m.diffuse_mode = BaseMaterial3D.DIFFUSE_TOON
	m.specular_mode = BaseMaterial3D.SPECULAR_TOON
	m.roughness = 1.0
	# A soft rim of light on the edges, for the warm cel look.
	m.rim_enabled = true
	m.rim = 0.2
	m.rim_tint = 0.5
	if outline:
		if _outline_mat == null:
			_outline_mat = StandardMaterial3D.new()
			_outline_mat.albedo_color = INK
			_outline_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
			_outline_mat.cull_mode = BaseMaterial3D.CULL_FRONT
			_outline_mat.grow = true
			_outline_mat.grow_amount = 0.035
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
	rig = null
	anim = null
	skeleton = null
	current_clip = ""
	face_parts = {}
	last_expression = {}
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
	_mesh_cache.clear()
	_outline_mat = null


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


## Builds the rigged blob (arms, legs and animation clips from art/blender/make_rigged_blob.py). Returns false if it is not available, and
## the fighter is then built from parts or spheres.
func _build_rig(skin: StandardMaterial3D) -> bool:
	var rig_path := RIG_LONG_PATH if long_limbs else RIG_PATH
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
	# The shorts take the outfit colour (the accent, a shade darker), so the body reads as a shirt over shorts.
	var shorts := toon(loadout.accent_color().darkened(0.2))
	for mi in rig.find_children("*", "MeshInstance3D", true, false):
		var part := str(mi.name)
		if part.begins_with("Hand") or part.begins_with("Shin") or part.begins_with("Sole") or part.begins_with("Collar"):
			mi.material_override = white
		elif part.begins_with("Shorts"):
			mi.material_override = shorts
		elif part.begins_with("Foot"):
			mi.material_override = shoe
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
	if long_limbs:
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
			return ["walk", clampf(speed / 0.09, 0.5, 2.0), -1.0]
		"Run":
			return ["run", clampf(speed / 0.22, 0.6, 1.5), -1.0]
		"Dash":
			return ["dash", clampf(speed / 0.3, 0.7, 1.6), -1.0]
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
		"Idle", "Turn", "LedgeGetUp", "LedgeAttack", "Respawn":
			return ["idle", 1.0, -1.0]
	if grounded:
		return ["idle", 1.0, -1.0]
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
	if f < start:
		progress = f / start * 0.55
	elif f <= last:
		progress = 0.55 + (f - start) / maxf(1.0, last - start) * 0.2
	else:
		progress = 0.75 + (f - last) / maxf(1.0, total - last) * 0.25
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


## Plays the right clip for this frame and moves the head and torso followers with their bones.
func _animate(s: Dictionary, delta: float) -> void:
	if anim == null:
		return
	var pick: Array = _choose_clip(s)
	var clip: String = pick[0]
	if clip != current_clip:
		anim.play(clip, 0.1 if current_clip != "" and not clip.begins_with("attack") else 0.0)
		current_clip = clip
	if float(pick[2]) >= 0.0:
		anim.pause()
		anim.seek(float(pick[2]) * anim.get_animation(clip).length, true)
	elif int(s.hitlag) == 0:
		# (Hitlag freezes the pose along with everything else.)
		anim.speed_scale = float(pick[1])
		anim.advance(delta)
	var to_model := _skeleton_to_model()
	var head_delta: Transform3D = skeleton.get_bone_global_pose(head_bone) * head_rest_inv
	var spine_delta: Transform3D = skeleton.get_bone_global_pose(spine_bone) * spine_rest_inv
	head_rig.transform = to_model * head_delta * to_model.affine_inverse() * head_fit
	torso_rig.transform = to_model * spine_delta * to_model.affine_inverse() * torso_fit


## A victory pose for the results screen: plays one of the looping victory clips (by class: the sword and the maul are raised high, the
## claws fighter pumps a fist; `cheer` makes anyone throw both arms up and hop), with the weapon held up in the raised hand. Call every
## frame with the time since the last.
func play_victory(cls: int, delta: float, cheer := false) -> void:
	if anim == null:
		return
	var clip := "victory_c" if cheer else ("victory_b" if cls == 1 else "victory_a")
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
		var at: Vector3 = to_model * skeleton.get_bone_global_pose(hand).origin
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


## The face's moving parts, so the expression can change (see set_expression).
var face_parts := {}


## Builds the fighter. `l` is its cosmetic loadout (see loadout.gd); without one the player's default look is used.
func build(p: int, l: RefCounted = null) -> void:
	player = p
	loadout = l if l != null else Loadout.default_for(p)
	var col: Color = loadout.body_color()
	var skin := toon(col)
	var ink := toon(INK, false)
	model = Node3D.new()
	add_child(model)

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
	_face(skin, ink)
	_hat()
	_glasses()

	shield = MeshInstance3D.new()
	shield.mesh = _sphere(1.5)
	var sm := StandardMaterial3D.new()
	sm.albedo_color = Color(0.4, 0.7, 1.0, 0.35)
	sm.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	sm.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
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
	blade_pivot = Node3D.new()
	model.add_child(blade_pivot)
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
	flame = MeshInstance3D.new()
	flame.mesh = _sphere(1.0)
	var flame_mat := StandardMaterial3D.new()
	flame_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	flame_mat.albedo_color = Color(1.0, 0.45, 0.1, 0.55)
	flame_mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	flame.material_override = flame_mat
	flame.position = Vector3(0, 1.1, 0)
	flame.visible = false
	add_child(flame)


const EYE_Y := 1.5
const EYE_DX := 0.31


## The face is drawn on the front of the head: eyes with heavy lids, brows and a mouth. `set_expression` poses them.
func _face(skin: StandardMaterial3D, ink: StandardMaterial3D) -> void:
	face_parts = {"lids": [], "lines": [], "brows": [], "mouth": null, "skin": skin}
	for sx in [-1.0, 1.0]:
		var eye_pos := Vector3(sx * EYE_DX, EYE_Y, 0.745)
		_part(head_rig, _sphere(1.0), toon(Color(1, 1, 1), false), eye_pos, Vector3(0.17, 0.19, 0.05))
		_part(head_rig, _sphere(1.0), ink, eye_pos + Vector3(sx * -0.02, -0.03, 0.03), Vector3(0.08, 0.1, 0.04))
		face_parts.lids.append(_part(head_rig, _sphere(1.0), skin, eye_pos, Vector3(0.2, 0.1, 0.06)))
		face_parts.lines.append(_part(head_rig, _sphere(1.0), ink, eye_pos, Vector3(0.18, 0.012, 0.03)))
		# A catch-light in each eye and a touch of blush on the cheeks.
		_part(head_rig, _sphere(1.0), toon(Color(1, 1, 1), false), eye_pos + Vector3(sx * -0.02 + 0.03, 0.035, 0.075), Vector3(0.04, 0.045, 0.02))
		var blush := StandardMaterial3D.new()
		blush.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		blush.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		blush.albedo_color = Color(1.0, 0.45, 0.5, 0.35)
		_part(head_rig, _sphere(1.0), blush, Vector3(sx * (EYE_DX + 0.12), EYE_Y - 0.2, 0.7), Vector3(0.12, 0.06, 0.03))
		var brow := BoxMesh.new()
		brow.size = Vector3(0.3, 0.05, 0.05)
		face_parts.brows.append(_part(head_rig, brow, ink, eye_pos + Vector3(0, 0.3, 0.03)))
	face_parts.mouth = _part(head_rig, _sphere(1.0), ink, Vector3(0, 1.17, 0.775), Vector3(0.14, 0.05, 0.05))
	set_expression(Loadout.FACES[loadout.face])


## Poses the face from a dictionary like the entries of Loadout.FACES (lid, mouth_w, mouth_h, mouth_tilt, brow).
func set_expression(e: Dictionary) -> void:
	if face_parts.is_empty() or e == last_expression:
		return
	last_expression = e
	var lid: float = e.lid
	for i in 2:
		var sx := -1.0 if i == 0 else 1.0
		var cap: MeshInstance3D = face_parts.lids[i]
		cap.scale = Vector3(0.2, maxf(0.1 * lid * 2.0, 0.001), 0.06)
		cap.position = Vector3(sx * EYE_DX, EYE_Y + 0.19 - 0.19 * lid, 0.79)
		var line: MeshInstance3D = face_parts.lines[i]
		line.position = Vector3(sx * EYE_DX, EYE_Y + 0.19 - 0.38 * lid, 0.795)
		var brow: MeshInstance3D = face_parts.brows[i]
		brow.visible = absf(float(e.brow)) >= 1.0
		brow.rotation_degrees = Vector3(0, 0, float(e.brow) * sx)
	var mouth: MeshInstance3D = face_parts.mouth
	mouth.scale = Vector3(e.mouth_w, e.mouth_h, 0.05)
	mouth.rotation_degrees = Vector3(0, 0, e.mouth_tilt)


var last_expression := {}


func _neck(_skin: StandardMaterial3D) -> void:
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
			_part(torso_rig, ring, toon(SASH), centre, Vector3(0.88, 1.0, 1.0), Vector3(0, 0, -42))
			# Two round badges pinned on the front of the sash.
			var badge := CylinderMesh.new()
			badge.top_radius = 0.075
			badge.bottom_radius = 0.075
			badge.height = 0.03
			badge.radial_segments = 16
			_part(torso_rig, badge, toon(Color(1.0, 0.82, 0.3)), centre + Vector3(-0.17, 0.2, 0.55), Vector3.ONE, Vector3(90, 0, 0))
			_part(torso_rig, badge, toon(accent), centre + Vector3(0.05, 0.0, 0.58), Vector3.ONE, Vector3(90, 0, 0))
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


func _hat() -> void:
	var accent: Color = loadout.accent_color()
	var white := toon(Color(0.97, 0.97, 1.0))
	match loadout.hat:
		1:
			# Sailor cap: white crown and brim with a band in the accent colour.
			_part(head_rig, _cyl(0.5, 0.55, 0.32), white, Vector3(0, 2.18, 0))
			_part(head_rig, _cyl(0.62, 0.62, 0.08), white, Vector3(0, 2.02, 0))
			_part(head_rig, _cyl(0.5, 0.5, 0.05), toon(accent), Vector3(0, 2.1, 0), Vector3(1.04, 1.0, 1.04))
		2:
			# Aviator cap with goggles on top.
			_part(head_rig, _sphere(0.86), toon(Color(0.78, 0.6, 0.38)), Vector3(0, 1.84, -0.14), Vector3(1.0, 0.6, 1.0))
			var ring := TorusMesh.new()
			ring.inner_radius = 0.1
			ring.outer_radius = 0.2
			for sx in [-1.0, 1.0]:
				_part(head_rig, ring, toon(Color(0.45, 0.28, 0.12)), Vector3(sx * 0.3, 2.12, 0.28), Vector3.ONE, Vector3(70, 0, 0))
		3:
			# Straw hat with a band in the accent colour.
			var straw := toon(Color(0.9, 0.78, 0.45))
			_part(head_rig, _cyl(1.15, 1.15, 0.07), straw, Vector3(0, 2.05, 0))
			_part(head_rig, _cyl(0.55, 0.62, 0.35), straw, Vector3(0, 2.25, 0))
			_part(head_rig, _cyl(0.63, 0.63, 0.08), toon(accent), Vector3(0, 2.14, 0))
		4:
			# Beanie with a pompom.
			_part(head_rig, _sphere(0.84), toon(accent), Vector3(0, 1.78, 0), Vector3(1.0, 0.7, 1.0))
			_part(head_rig, _sphere(0.18), white, Vector3(0, 2.38, 0))
		5:
			# Crown: a gold band with five points.
			var gold := toon(Color(0.96, 0.8, 0.25))
			_part(head_rig, _cyl(0.55, 0.58, 0.2), gold, Vector3(0, 2.1, 0))
			for i in 5:
				var a := TAU * i / 5.0
				_part(head_rig, _cyl(0.0, 0.11, 0.32), gold, Vector3(sin(a) * 0.5, 2.36, cos(a) * 0.5))


func _glasses() -> void:
	var lens := toon(Color(1.0, 0.62, 0.3), false)
	var frame := toon(INK, false)
	match loadout.glasses:
		1:
			# Shades, like the reference.
			for sx in [-1.0, 1.0]:
				_part(head_rig, _sphere(1.0), lens, Vector3(sx * 0.32, 1.52, 0.8), Vector3(0.25, 0.18, 0.04))
			_part(head_rig, _sphere(1.0), frame, Vector3(0, 1.54, 0.82), Vector3(0.1, 0.03, 0.03))
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


## Squash pose per state as a single number: positive squashes down and out, negative stretches up.
const SQUASH := {
	"JumpSquat": 0.32, "Landing": 0.2, "Crouch": 0.28, "WaveLand": 0.24, "Turn": 0.08,
	"Dash": 0.04, "ShieldDrop": 0.0, "Roll": 0.3, "SpotDodge": 0.34, "ShieldBreak": 0.22, "Grabbed": 0.14, "Knockdown": 0.42, "GetUp": 0.2,
}
## Forward lean in degrees per state.
const LEAN := {"Dash": 16.0, "Run": 12.0, "WaveLand": 28.0, "Walk": 4.0, "LedgeAttack": 20.0, "Roll": 24.0, "ShieldBreak": 32.0, "Grabbing": 8.0, "Grabbed": -10.0, "Knockdown": 78.0, "GetUp": 24.0}

# Damped springs make landings and takeoffs read as soft and elastic instead of linear and stiff.
const SPRING_K := 420.0
const SPRING_DAMP := 22.0  # a bit under critical (2*sqrt(K) = 41), so there is a small rebound

var squash := 0.0
var squash_vel := 0.0
var lean_vel := 0.0
var was_grounded := false
var last_vy := 0.0


## `s` is a dictionary of sim state (see main.gd `_refresh`).
func apply(pos: Vector3, s: Dictionary, delta: float) -> void:
	# The brawler has longer limbs: swap to its rig the first time we see its class.
	if rig != null and (_cls(s) == 1) != long_limbs:
		long_limbs = _cls(s) == 1
		rebuild(loadout)
	position = pos
	var state: String = s.state
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
	squash = clampf(squash, -0.35, 0.55)

	# Volume-preserving: squashing down widens the body.
	model.scale = Vector3(1.0 + 0.5 * squash, 1.0 - squash, 1.0 + 0.5 * squash)
	yaw = lerp_angle(yaw, float(facing) * deg_to_rad(36.0), clampf(delta * 14.0, 0.0, 1.0))
	model.rotation = Vector3(0, yaw, -float(facing) * deg_to_rad(lean))

	# Air dodge and ledge invincibility read as ghostly.
	var ghost := 0.0
	if state == "AirDodge":
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
		(shield.material_override as StandardMaterial3D).albedo_color = col
	speed_lines.visible = fast_falling
	_animate(s, delta)
	_apply_combat(s, delta)


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
var flame: MeshInstance3D
## One clip per move where there is one (the rest share a clip by move type, below in `_attack_clip`).
const SWORD_CLIPS := {"ftilt": "sword_ftilt", "utilt": "sword_utilt", "dtilt": "sword_dtilt", "fsmash": "sword_fsmash",
	"usmash": "sword_usmash", "dsmash": "sword_dsmash"}
const KICK_CLIPS := {"nair": "kick_nair", "bair": "kick_bair", "uair": "kick_uair", "dair": "kick_dair", "utilt": "kick_up",
	"dtilt": "kick_low", "dash attack": "kick_dash"}
## How far the kicking leg is pulled toward the hitbox (eases in and out like the arm).
var leg_k := 0.0

## The brawler fights with feet and body, not a blade: these moves draw no weapon.
const BRAWLER_NO_BLADE := ["utilt", "dtilt", "dash attack", "nair", "bair", "dair", "uair", "side special", "up special", "down special", "grab", "dash grab", "pummel", "forward throw", "back throw", "up throw", "down throw"]
## Moves that rush the whole body forward in a flame.
const BRAWLER_FLAME := ["side special", "up special"]
var last_percent := -1
var blade_angle := REST_ANGLE
var blade_length := 2.0


func _rest_length(s: Dictionary) -> float:
	# Swords are long; claws are short.
	return clampf(float(s.reach) - HAND.x - 0.2, 0.7, 3.4) if _cls(s) != 1 else 1.0


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
	# Wind-up comes from the opposite side of the swing.
	var wind := swing + 100.0
	if sin(deg_to_rad(swing)) > 0.55 or cos(deg_to_rad(swing)) < 0.0:
		wind = swing - 100.0
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
		var k := clampf((f - t[2]) / maxf(1.0, t[0] - t[2]), 0.0, 1.0)
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
	_update_trail(s, tip - Vector2.from_angle(deg_to_rad(blade_angle)) * radius * 0.7, radius)
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
		var shoulder := rest_u.origin
		var elbow0 := rest_l.origin
		var wrist0 := rest_h.origin
		var upper := (elbow0 - shoulder).length()
		var lower := (wrist0 - elbow0).length() + 0.07
		var to_skel := _skeleton_to_model().affine_inverse()
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
	leg_k = move_toward(leg_k, 1.0 if live else 0.0, delta * 12.0)
	var facing := int(s.facing)
	# The front leg kicks forward and up, the back leg backward.
	var front := "R" if facing > 0 else "L"
	var back := "L" if facing > 0 else "R"
	var kicker := front if tip.x >= 0.0 else back
	for side in ["L", "R"]:
		var it := skeleton.find_bone("thigh." + side)
		var ish := skeleton.find_bone("shin." + side)
		var ift := skeleton.find_bone("foot." + side)
		if side != kicker or leg_k <= 0.01:
			skeleton.set_bone_global_pose_override(it, Transform3D(), 0.0, false)
			skeleton.set_bone_global_pose_override(ish, Transform3D(), 0.0, false)
			skeleton.set_bone_global_pose_override(ift, Transform3D(), 0.0, false)
			continue
		var rest_t := skeleton.get_bone_global_rest(it)
		var rest_s := skeleton.get_bone_global_rest(ish)
		var rest_f := skeleton.get_bone_global_rest(ift)
		var hip := rest_t.origin
		var knee0 := rest_s.origin
		var ankle0 := rest_f.origin
		var upper := (knee0 - hip).length()
		var lower := (ankle0 - knee0).length() + 0.08
		var to_skel := _skeleton_to_model().affine_inverse()
		# The hitbox centre in model space; the foot reaches it (or as near as the leg allows).
		var goal: Vector3 = to_skel * Vector3(tip.x * float(facing), tip.y, 0.2)
		var d := goal - hip
		var dist := clampf(d.length(), 0.05, upper + lower - 0.002)
		var dir := d.normalized()
		var cos_a := clampf((upper * upper + dist * dist - lower * lower) / (2.0 * upper * dist), -1.0, 1.0)
		var ang := acos(cos_a)
		# The knee bends forward and a little up.
		var pole := Vector3(0.0, 0.4, 1.0)
		var bend := (pole - dir * pole.dot(dir)).normalized()
		var knee := hip + dir * (cos(ang) * upper) + bend * (sin(ang) * upper)
		var shin_dir := (goal - knee).normalized()
		var q_upper := Quaternion((knee0 - hip).normalized(), (knee - hip).normalized())
		var q_lower := Quaternion((ankle0 - knee0).normalized(), shin_dir)
		skeleton.set_bone_global_pose_override(it, Transform3D(Basis(q_upper) * rest_t.basis, hip), leg_k, true)
		skeleton.set_bone_global_pose_override(ish, Transform3D(Basis(q_lower) * rest_s.basis, knee), leg_k, true)
		skeleton.set_bone_global_pose_override(ift, Transform3D(Basis(q_lower) * rest_f.basis, knee + shin_dir * (ankle0 - knee0).length()), leg_k, true)


## A crescent trail behind the hitbox, like the one a fast punch or slash leaves. Each simulation frame the centre of the hitbox (the fist,
## or the part of the blade that hits) is added to a path; the path is drawn as a smooth ribbon that is thickest at the hitbox and tapers
## to nothing behind it, with a bright core inside a coloured edge. While the hitbox is live a thin ring marks where it is. The points are
## kept in world space, so the trail stays where the swing was while the fighter moves on. It is cosmetic: it only reads the move.
const TRAIL_FRAMES := 16
const TRAIL_WIDTH := 0.42
const TRAIL_SMOOTH := 4
var trail: MeshInstance3D
var trail_points: Array = []     # [{pos: Vector3, frame: int}]
var last_trail_frame := -1


func _make_trail() -> void:
	trail = MeshInstance3D.new()
	trail.mesh = ImmediateMesh.new()
	trail.top_level = true
	trail.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	var m := StandardMaterial3D.new()
	m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	m.vertex_color_use_as_albedo = true
	m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	m.cull_mode = BaseMaterial3D.CULL_DISABLED
	m.no_depth_test = true
	m.render_priority = 5
	trail.material_override = m
	add_child(trail)


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


func _puff(side: float, count: int) -> void:
	for n in count:
		var i := next_dust
		next_dust = (next_dust + 1) % DUSTS
		dusts[i].global_position = global_position + Vector3(side * 0.35, 0.12, 0.3)
		dust_vel[i] = Vector3(side * 2.2, 0.6, 0.0)
		dust_age[i] = 0.0
		dusts[i].visible = true


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
	if _cls(s) == 1:
		return [Color(0.78, 0.3, 1.0), Color(1.0, 0.9, 1.0)]
	if _cls(s) == 2:
		return [Color(0.95, 0.28, 0.12), Color(1.0, 0.92, 0.8)]
	return [Color(1.0, 0.62, 0.12), Color(1.0, 1.0, 0.85)]


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
func _update_trail(s: Dictionary, centre: Vector2, radius: float) -> void:
	if trail == null:
		_make_trail()
	var t: PackedInt32Array = s.move_timing
	var f: float = s.state_frame
	var swinging: bool = s.state == "Attack" and t[1] > 0 and f >= 1.0 and f <= t[2] + 5.0 and s.move_tip != Vector3.ZERO
	var dangerous: bool = swinging and f >= t[1] and f <= t[2]
	var frame: int = s.frame
	var facing: float = float(s.facing)
	var base := Vector3(position.x, position.y, 0.0)
	var here := base + Vector3(centre.x * facing, centre.y, 0.7)
	if swinging and int(s.hitlag) == 0 and frame != last_trail_frame:
		last_trail_frame = frame
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
		_ribbon(im, path, smooth_ages, TRAIL_WIDTH, colours[0], 0.8)
		_ribbon(im, path, smooth_ages, TRAIL_WIDTH * 0.38, colours[1], 1.0)
	if dangerous:
		_ring(im, here, maxf(radius * 0.8, 0.25), colours[0])


## A strip along `path`, `width` across at full strength, thinning toward the old end (high age) and fading out.
func _ribbon(im: ImmediateMesh, path: Array, ages: Array, width: float, colour: Color, alpha: float) -> void:
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLE_STRIP)
	for i in path.size():
		var along: Vector3 = (path[mini(i + 1, path.size() - 1)] - path[maxi(i - 1, 0)])
		along.z = 0.0
		var side := Vector3(-along.y, along.x, 0.0).normalized() if along.length() > 0.0001 else Vector3.UP
		var age: float = ages[i]
		var strength := pow(clampf(1.0 - age, 0.0, 1.0), 0.8)
		var half := width * 0.5 * strength
		var c := Color(colour.r, colour.g, colour.b, alpha * clampf(strength * 1.4, 0.0, 1.0))
		im.surface_set_color(c)
		im.surface_add_vertex(path[i] + side * half)
		im.surface_set_color(c)
		im.surface_add_vertex(path[i] - side * half)
	im.surface_end()


## A thin ring marking the live hitbox.
func _ring(im: ImmediateMesh, centre: Vector3, radius: float, colour: Color) -> void:
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLE_STRIP)
	var c := Color(colour.r, colour.g, colour.b, 0.9)
	for i in 33:
		var a := TAU * float(i) / 32.0
		var dir := Vector3(cos(a), sin(a), 0.0)
		im.surface_set_color(c)
		im.surface_add_vertex(centre + dir * (radius + 0.04))
		im.surface_set_color(c)
		im.surface_add_vertex(centre + dir * (radius - 0.04))
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
	if rig != null:
		_aim_arm(int(s.facing), swinging_arm if not brawler else (swinging_arm and state == "Attack"))
		_aim_leg(s, brawler and state == "Attack" and KICK_CLIPS.has(s.move_name), delta)
	var rushing: bool = brawler and state == "Attack" and BRAWLER_FLAME.has(s.move_name) and s.state_frame >= 12
	flame.visible = rushing
	if rushing:
		flame.scale = Vector3(1.3, 1.5, 1.3) * (1.0 + 0.12 * sin(float(s.frame) * 1.7))
	if brawler and state == "Attack" and s.move_name == "up special" and s.state_frame >= 17:
		model.rotation.z = float(s.frame) * 0.6 * -float(s.facing)

	var hitlag: int = s.hitlag
	# The hurt face is a cosmetic event: it shows while the fighter is being hit and goes back afterwards.
	set_expression(Loadout.HURT if state == "Hitstun" else Loadout.FACES[loadout.face])
	spark.visible = hitlag > 0 and ((state == "Hitstun" and s.launch_pending) or state == "Rebound")
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
	var shaking: bool = hitlag > 0 and (s.launch_pending or state == "Shield")
	if shaking:
		var amp := (0.05 + 0.012 * hitlag) * (0.4 if state == "Shield" else 1.0)
		var t := float(s.frame)
		model.position = Vector3(sin(t * 9.0) * amp, cos(t * 7.3) * amp * 0.5, 0)
	else:
		model.position = Vector3.ZERO
	_launch_smoke(s)
	_dust(s)
	_revival_platform(s)
	# Charging a smash attack: the glow grows and the body trembles harder the longer it is held.
	var charge: int = s.charge
	if charge > 0 and state == "Attack":
		var amount := clampf(charge / 60.0, 0.0, 1.0)
		spark.visible = (s.frame / 3) % 2 == 0
		spark.scale = Vector3.ONE * (0.4 + 0.9 * amount)
		model.position = Vector3(sin(float(s.frame) * 2.3) * 0.04 * (1.0 + 2.0 * amount), 0, 0)
	if state == "Hitstun" and hitlag == 0:
		if s.tumble:
			model.rotation.z = float(s.frame) * 0.4 * -float(s.facing)
		else:
			model.rotation.z = deg_to_rad(26.0) * float(s.facing)
