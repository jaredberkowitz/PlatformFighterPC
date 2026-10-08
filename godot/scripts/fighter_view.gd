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
## Where the head and torso sit in the rig compared with the sphere-built look the face, hats and glasses were designed for.
const HEAD_FIT := Transform3D(Basis(Vector3(0.825, 0, 0), Vector3(0, 0.825, 0), Vector3(0, 0, 0.825)), Vector3(0, 1.56 - 1.42 * 0.825, 0))
const TORSO_FIT := Transform3D(Basis(Vector3(0.8, 0, 0), Vector3(0, 0.8, 0), Vector3(0, 0, 0.72)), Vector3(0, 0.98 - 0.82 * 0.8, 0))
const LOOPING := ["idle", "walk", "run", "fall"]
## The rig is read once and copied for every fighter (reading it again renames its bones).
static var _rig_template: Node3D
static var _rig_tried := false

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


## Builds the rigged blob (arms, legs and animation clips from art/blender/make_rigged_blob.py). Returns false if it is not available, and
## the fighter is then built from parts or spheres.
func _build_rig(skin: StandardMaterial3D) -> bool:
	if not _rig_tried:
		_rig_tried = true
		var path := ProjectSettings.globalize_path(RIG_PATH)
		if FileAccess.file_exists(path):
			var doc := GLTFDocument.new()
			var state := GLTFState.new()
			if doc.append_from_file(path, state) == OK:
				_rig_template = doc.generate_scene(state)
	if _rig_template == null:
		return false
	var scene: Node3D = _rig_template.duplicate()
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
	for mi in rig.find_children("*", "MeshInstance3D", true, false):
		var part := str(mi.name)
		if part.begins_with("Hand") or part.begins_with("Shin"):
			mi.material_override = white
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
	head_rig.transform = HEAD_FIT
	torso_rig.transform = TORSO_FIT
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
			return ["walk", clampf(speed / 0.12, 0.5, 2.2), -1.0]
		"Run", "Dash":
			return ["run", clampf(speed / 0.3, 0.7, 1.8), -1.0]
		"Crouch", "JumpSquat", "Landing", "WaveLand", "SpotDodge", "Roll":
			return ["crouch", 1.0, -1.0]
		"Shield", "ShieldDrop":
			return ["shield", 1.0, -1.0]
		"Hitstun", "ShieldBreak", "Grabbed":
			return ["hurt", 1.0, -1.0]
		"Attack":
			var t: PackedInt32Array = s.move_timing
			var name: String = s.move_name
			var progress := clampf(float(s.state_frame) / maxf(1.0, float(t[0])), 0.0, 1.0)
			var clip := "attack_swing"
			if s.char == 1 and BRAWLER_NO_BLADE.has(name):
				clip = "attack_kick"
			elif name.begins_with("dtilt") or name.begins_with("dair") or name.begins_with("down"):
				clip = "attack_low"
			return [clip, 1.0, progress]
		"Idle", "Turn", "Knockdown", "GetUp", "LedgeHang", "LedgeGetUp", "LedgeAttack", "Grabbing":
			return ["idle", 1.0, -1.0]
	if grounded:
		return ["idle", 1.0, -1.0]
	return ["jump", 1.0, -1.0] if float(s.vel.y) > 0.02 else ["fall", 1.0, -1.0]


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
	var to_model: Transform3D = skeleton.get_parent().transform if skeleton.get_parent() != null else Transform3D()
	var head_delta: Transform3D = skeleton.get_bone_global_pose(head_bone) * head_rest_inv
	var spine_delta: Transform3D = skeleton.get_bone_global_pose(spine_bone) * spine_rest_inv
	head_rig.transform = to_model * head_delta * to_model.affine_inverse() * HEAD_FIT
	torso_rig.transform = to_model * spine_delta * to_model.affine_inverse() * TORSO_FIT


## The meshes of the modelled blob (Body, Head, FootL, FootR, HandL, HandR), read straight from the glTF file so no editor import is
## needed; empty if the file is missing or damaged (the fighter then falls back to spheres).
static func blob_parts() -> Dictionary:
	if _parts_tried:
		return _parts
	_parts_tried = true
	var path := ProjectSettings.globalize_path(PARTS_PATH)
	if not FileAccess.file_exists(path):
		return _parts
	var doc := GLTFDocument.new()
	var state := GLTFState.new()
	if doc.append_from_file(path, state) != OK:
		return _parts
	var scene := doc.generate_scene(state)
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
	blade_pivot = Node3D.new()
	model.add_child(blade_pivot)
	var blade_mesh := BoxMesh.new()
	blade_mesh.size = Vector3(2.7, 0.16, 0.09)
	_part(blade_pivot, blade_mesh, toon(Color(0.86, 0.91, 0.99)), Vector3(1.55, 0, 0))
	var guard_mesh := BoxMesh.new()
	guard_mesh.size = Vector3(0.12, 0.6, 0.12)
	_part(blade_pivot, guard_mesh, toon(Color(0.92, 0.76, 0.3)), Vector3(0.18, 0, 0))
	var grip_mesh := BoxMesh.new()
	grip_mesh.size = Vector3(0.4, 0.12, 0.12)
	_part(blade_pivot, grip_mesh, toon(SASH), Vector3(-0.02, 0, 0))

	# Impact spark, shown during hitlag.
	spark = MeshInstance3D.new()
	spark.mesh = _sphere(0.7)
	var spark_mat := StandardMaterial3D.new()
	spark_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	spark_mat.albedo_color = Color(1.0, 0.97, 0.7, 0.85)
	spark_mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
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
var spark: MeshInstance3D
var flame: MeshInstance3D
## The brawler fights with feet and body, not a blade: these moves draw no weapon.
const BRAWLER_NO_BLADE := ["utilt", "dtilt", "dash attack", "bair", "dair", "uair", "side special", "up special", "down special", "grab", "dash grab", "pummel", "forward throw", "back throw", "up throw", "down throw"]
## Moves that rush the whole body forward in a flame.
const BRAWLER_FLAME := ["side special", "up special"]
var last_percent := -1
var blade_angle := REST_ANGLE
var blade_length := 2.0


func _rest_length(s: Dictionary) -> float:
	# Swords are long; claws are short.
	return clampf(float(s.reach) - HAND.x - 0.2, 0.7, 3.4) if s.char == 0 else 1.0


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
	if s.char == 1:
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
	# Forward space to model space: facing left mirrors the pose about the vertical axis.
	var angle := blade_angle if facing > 0 else 180.0 - blade_angle
	blade_pivot.position = Vector3(float(target[2]) * facing, HAND.y, 0.35)
	blade_pivot.rotation = Vector3(0, 0, deg_to_rad(angle))
	blade_pivot.scale = Vector3(blade_length / MESH_LENGTH, 1.0 if s.char == 0 else 1.7, 1.0 if s.char == 0 else 1.7)


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
	var brawler: bool = s.char == 1
	blade_pivot.visible = not (brawler and BRAWLER_NO_BLADE.has(s.move_name))
	var rushing: bool = brawler and state == "Attack" and BRAWLER_FLAME.has(s.move_name) and s.state_frame >= 12
	flame.visible = rushing
	if rushing:
		flame.scale = Vector3(1.3, 1.5, 1.3) * (1.0 + 0.12 * sin(float(s.frame) * 1.7))
	if brawler and state == "Attack" and s.move_name == "up special" and s.state_frame >= 17:
		model.rotation.z = float(s.frame) * 0.6 * -float(s.facing)

	var hitlag: int = s.hitlag
	# The hurt face is a cosmetic event: it shows while the fighter is being hit and goes back afterwards.
	set_expression(Loadout.HURT if state == "Hitstun" else Loadout.FACES[loadout.face])
	spark.visible = hitlag > 0 and state == "Hitstun" and s.launch_pending
	if spark.visible:
		spark.scale = Vector3.ONE * (0.5 + 0.1 * hitlag)
	# Shake while frozen in hitlag.
	model.position = Vector3(sin(float(s.frame) * 9.0) * 0.14, 0, 0) if hitlag > 0 else Vector3.ZERO
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
