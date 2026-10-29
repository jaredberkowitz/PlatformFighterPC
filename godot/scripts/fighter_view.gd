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
const VisualSettings := preload("res://scripts/visual_settings.gd")
const SHIELD_SHADER := preload("res://shaders/shield.gdshader")
const Effects := preload("res://scripts/effects.gd")
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
	launch_trail = null
	rig = null
	anim = null
	skeleton = null
	current_clip = ""
	shown_clip = ""
	face_mesh = null
	face_mat = null
	last_expression = {}
	shown_face = ""
	last_ghost = 0.0
	unkeyed = {}
	neck_pivot = null
	tails = []
	# (The dust puffs were children too: freed above, so the references go with them, or the next puff used a freed mesh.)
	dusts.clear()
	dust_age.clear()
	dust_vel.clear()
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


## The hood's opening, as how far round from the front its edge is (radians on the shell) at angle `ph` round the front (0 to the side,
## up at PI/2): snug over the brow, wider at the cheeks, and running down past the chin, so it frames the face as a hood does instead of
## a round porthole.
static func _hood_edge(ph: float) -> float:
	var up := maxf(0.0, sin(ph))
	var down := maxf(0.0, -sin(ph))
	return deg_to_rad(50.0 - 14.0 * up * up + 40.0 * pow(down, 1.6))


## A ball of radius `r` open at the front, its edge `edge(ph)` round from the front (see `_hood_edge`): both sides drawn by the material.
static func _open_shell(r: float, edge: Callable) -> ArrayMesh:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var rings := 18
	var segs := 36
	for i in rings:
		for j in segs:
			var quad := []
			for c in [[i, j], [i + 1, j], [i + 1, j + 1], [i, j + 1]]:
				var ph := TAU * float(c[1]) / float(segs)
				var th := lerpf(float(edge.call(ph)), PI, float(c[0]) / float(rings))
				quad.append(Vector3(sin(th) * cos(ph), sin(th) * sin(ph), cos(th)))
			for t in [0, 1, 2, 0, 2, 3]:
				st.set_normal(quad[t])
				st.add_vertex(quad[t] * r)
	return st.commit()


## The rolled edge of `_open_shell(r, edge)`: a tube of radius `t` along its opening, in the same space, so it lines up with the shell
## under any placement and scale.
static func _shell_lip(r: float, edge: Callable, t: float) -> ArrayMesh:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var segs := 48
	var sides := 8
	var ring := func(ph: float) -> Array:
		var th: float = edge.call(ph)
		var c := Vector3(sin(th) * cos(ph), sin(th) * sin(ph), cos(th)) * r
		var along := Vector3(cos(th) * cos(ph), cos(th) * sin(ph), -sin(th))
		return [c, c.normalized(), along]
	for j in segs:
		for k in sides:
			var quad := []
			for c in [[j, k], [j + 1, k], [j + 1, k + 1], [j, k + 1]]:
				var f: Array = ring.call(TAU * float(c[0]) / float(segs))
				var u := TAU * float(c[1]) / float(sides)
				var n: Vector3 = (f[1] as Vector3) * cos(u) + (f[2] as Vector3) * sin(u)
				quad.append([(f[0] as Vector3) + n * t, n])
			for idx in [0, 1, 2, 0, 2, 3]:
				st.set_normal(quad[idx][1])
				st.add_vertex(quad[idx][0])
	return st.commit()


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
const LOOPING := ["tumble", "idle", "idle_brawler", "idle_quick", "idle_heavy", "idle_caster", "idle_tool", "run_brawler", "run_quick", "run_heavy", "run_caster", "run_tool", "walk", "walk_brawler", "walk_quick", "walk_heavy", "walk_caster", "walk_tool", "run", "dash", "fall", "victory_a", "victory_b", "victory_c",
	"helpless", "dizzy", "buried"]
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
		# (By the clip's own name, its facing-left copy included: the library is shared, so a second fighter's build saw "run@m" here and
		# switched its loop off, and every left-facing run, walk and dash froze on its last frame and slid. The owner's dash-left bug.)
		anim.get_animation(clip_name).loop_mode = Animation.LOOP_LINEAR if LOOPING.has(String(clip_name).trim_suffix(MIRROR_SUFFIX)) else Animation.LOOP_NONE
	if use_base_body:
		_add_mirrored_clips()
	var white := toon(Color(1, 1, 1))
	# Each kind of stuff takes the light its own way (`respond`): rubbery gloves and soles, the shoes' sheen; cloth stays matte.
	var glove := respond(white, "rubber")
	var shoe := respond(toon(Color(0.27, 0.2, 0.3)), "rubber")
	var shirt := _shirt_material()
	# The shorts take the outfit colour (the accent, a shade darker) under a plain shirt, and navy under a coloured one.
	# (Khaki under the holiday shirt and olive under the outfit shirt, as the reference's blue and ranger, the character judge's round 16.)
	var shorts_colour: Color = loadout.accent_color().darkened(0.2) if loadout.shirt <= 1 else {4: Color(0.7, 0.6, 0.42), 2: Color(0.4, 0.42, 0.26)}.get(loadout.shirt, Color(0.22, 0.25, 0.42))
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
	fists = {}
	open_hands = {}
	for mi in rig.find_children("*", "MeshInstance3D", true, false):
		var part := str(mi.name)
		# Full detail always: Godot simplifies meshes that look small on screen (its automatic LOD), which chewed thin cloth's hems into
		# a torn edge in the menus' small previews.
		mi.lod_bias = 128.0
		if part.begins_with("Arm"):
			mi.material_override = arm
		elif part.begins_with("Leg"):
			mi.material_override = leg
		elif part.begins_with("FistBare") or part.begins_with("FistGlove"):
			# The closed fists (`_clench` shows them in place of the open hand).
			var gloved := part.begins_with("FistGlove")
			mi.material_override = glove if gloved else respond(toon(skin_colour), "skin")
			mi.visible = false
			if use_base_body and gloved != bare:
				fists[part.right(1)] = mi
		elif part.begins_with("Mitt"):
			mi.material_override = respond(toon(skin_colour), "skin")
			mi.visible = bare
			if bare:
				open_hands[part.right(1)] = mi
		elif part.begins_with("Hand") and bare:
			# Bare hands (the concept's mittens): the body's own colour and skin.
			mi.material_override = respond(toon(skin_colour), "skin")
			mi.visible = not mittens
		elif part.begins_with("Hand") or part.begins_with("Sole"):
			mi.material_override = glove
			if use_base_body and part.begins_with("Hand"):
				open_hands[part.right(1)] = mi
		elif part.begins_with("Collar") or part.begins_with("Cuff"):
			mi.material_override = white
			# (The glove's cuff: none on a bare hand.)
			mi.visible = not (use_base_body and part.begins_with("Cuff") and not loadout.has_gloves())
		elif part.begins_with("Strap"):
			mi.material_override = toon(loadout.accent_color())
		elif part.begins_with("ShortsCuff") or part.begins_with("ShortsPocket"):
			# The shorts' rolled cuffs and pocket flaps (make_rigged_blob.py), a shade darker than the shorts.
			# (A cuff of its own per neckwear, so the cast's shorts differ, the character judge's round 6: a white sailor's stripe with the
			# kerchief, a cream turn-up on the holiday shirt, else a shade darker than the shorts.)
			var cuff_c: Color = shorts_colour.darkened(0.12)
			if part.begins_with("ShortsCuff"):
				if loadout.neck == 5:
					cuff_c = Color(0.97, 0.96, 0.92)
				elif loadout.shirt == 4:
					cuff_c = Color(0.95, 0.9, 0.78)
			mi.material_override = _cloth(toon(cuff_c))
			mi.visible = loadout.has_belt() if part.begins_with("ShortsPocket") else loadout.has_shorts()
		elif part.begins_with("Sock"):
			# Tall thick socks, with a turned-down top (the character judge's round 1, the owner's reference): white knit.
			# (A warm cream, the turn-down a shaded step under it, the character judge's round 10: pure white tubes were the brightest mass.)
			mi.material_override = _cloth(toon(Color(0.76, 0.72, 0.64) if part.begins_with("SockTop") else Color(0.88, 0.85, 0.78)))
			mi.visible = loadout.has_shoes()
		elif part.begins_with("BootToe"):
			# The toe cap, a polished step over the leather.
			mi.material_override = toon(BOOT_LEATHER.lightened(0.14))
			mi.visible = loadout.has_shoes()
		elif part.begins_with("BootCuff"):
			mi.material_override = toon(BOOT_LEATHER.darkened(0.3))
			mi.visible = loadout.has_shoes()
		elif part.begins_with("BootSole"):
			mi.material_override = respond(toon(BOOT_SOLE), "rubber")
			mi.visible = loadout.has_shoes()
		elif part.begins_with("Boot"):
			# Hiking boots over the painted shoes (the owner's PEAK references): leather on a thick pale sole.
			mi.material_override = toon(BOOT_LEATHER)
			mi.visible = loadout.has_shoes()
		elif part.begins_with("Shorts"):
			mi.material_override = _cloth(shorts)
			mi.visible = loadout.has_shorts()
		elif part.begins_with("Belt"):
			# The belt the shirt is tucked under (the owner saw PEAK's figures have a waistline): leather, with a brass buckle.
			mi.material_override = _cloth(toon(BELT_LEATHER))
			mi.visible = loadout.has_belt()
		elif part.begins_with("Buckle"):
			mi.material_override = respond(toon(BELT_BRASS), "rubber")
			mi.visible = loadout.has_belt()
		elif part.begins_with("SleeveTrim"):
			# The band round each sleeve's end (the aviator scout pilot, the owner's reference): the outfit colour on a white shirt, white
			# on a coloured or printed one.
			mi.visible = loadout.shirt > 0
			# (Since the owner's second reference: a rolled cuff of the shirt's own cloth.)
			var cuff_cloth := _garment_shirt_material()
			if cuff_cloth != null:
				mi.material_override = _cloth(cuff_cloth)
		elif part.begins_with("ShirtCampCollar"):
			# The holiday shirt's collar, in its own print.
			mi.visible = loadout.shirt == 4
			# (A plain cream until the shirt pass, 2026-10-20, as the reference's collar's lining shows: it read as a white slab across the
			# shoulders. Its fold and edge stand off the shirt, so it reads in the same print.)
			var print_cloth := _garment_shirt_material()
			mi.material_override = _cloth(print_cloth) if print_cloth != null else _cloth(toon(Color(0.98, 0.94, 0.86)))
		elif part.begins_with("ShirtCollar") or part.begins_with("ShirtPlacket"):
			# The collar and the placket, modelled on the shirt's front (make_rigged_blob.py's `shirt_front`): a shade darker than a plain
			# shirt, white on a printed one; the collar hides under a ring of neckwear.
			mi.visible = loadout.shirt > 0 and not (part.begins_with("ShirtCollar") and (loadout.neck >= 2 or loadout.shirt == 4))
			mi.material_override = _cloth(toon(_collar_colour()))
		elif part.begins_with("Buttons"):
			# (Not under a sash, the badge pass, 2026-10-19: the band crosses the button line and the buttons poked through it as a row of
			# pale dots in the middle of the chest.)
			mi.visible = loadout.shirt > 0 and not (loadout.neck == 1 or loadout.neck == 5)
			# (No ink ring round each since the shirt pass, 2026-10-20: three ringed buttons stacked read as the figure 8.)
			mi.material_override = toon(Color(0.95, 0.9, 0.78), false)
		elif part.begins_with("Shirt"):
			var cloth := _garment_shirt_material()
			mi.visible = cloth != null
			if cloth != null:
				var shirt_mat: ShaderMaterial = (_cloth(cloth) as ShaderMaterial).duplicate()
				# An ink line along the bottom hem, so where the shirt meets the shorts reads as one clean edge at any size.
				shirt_mat.set_shader_parameter("hem_low", float(_base_rig_info().get("clothes", {}).get("waist", 0.89)) - SHIRT_HEM_BELOW_WAIST)
				shirt_mat.set_shader_parameter("hem_ink", SHIRT_HEM_INK)
				# (Soft shade at the neckline under the head, the character judge's round 8.)
				shirt_mat.set_shader_parameter("neck_shade_z", float(_base_rig_info().get("clothes", {}).get("neck", -99.0)))
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
## The base body's runs play once a cycle every clip-frames x RUN_TEMPO units of ground (make_rigged_blob.py RUN_TEMPO).
const RUN_TEMPO := 0.17
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
			# (Each family's own walk, `walk_<family>`, round 2; the clip's length sets its stride, see make_rigged_blob.py WALK_STYLES.)
			return [AnimLibrary.family_clip(_cls(s), "walk", anim), clampf(speed / WALK_TEMPO, 0.5, 3.6), -1.0]
		"Run", "Dash":
			# Cycles a second (two dash lengths each) times the clip's length in seconds.
			var stride := 2.0 * maxf(0.5, float(s.get("dash_length", 2.4)))
			# (Each family's own run, `run_<family>`: round 2 found the family runs built in pass 1 never played, because this returned the
			# shared run before the family lookup below.)
			var clip := "dash" if state == "Dash" else AnimLibrary.family_clip(_cls(s), "run", anim)
			var length := anim.get_animation(clip).length if anim != null and anim.has_animation(clip) else 0.3
			if state == "Run" and use_base_body:
				# (Round 3: the runs' feet are authored to cover RUN_TEMPO units of ground a clip frame, make_rigged_blob.py RUN_STYLES, so
				# the clip plays at the ground's pace and a planted foot stays put.)
				return [clip, clampf(speed / RUN_TEMPO, 0.5, 3.2), -1.0]
			return [clip, minf(3.2, speed * 60.0 / stride * length * STEP_CADENCE), -1.0]
		"Attack":
			return _attack_clip(s)
		"Carrying":
			# The Hauler's carry: the fighter it holds rides overhead (placed by the game at 2.8 up), so the arms stay raised under it, the
			# up air's reach held at its top.
			return [AnimLibrary.or_stand_in("attack_uair", anim), 1.0, 0.45]
		"Knockdown":
			# A trip is a fall onto the seat, not onto the back: crouched low, held, rather than laid flat (`tripped` from the bridge).
			if s.get("tripped", false):
				return ["crouch", 1.0, 1.0]
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
				return [AnimLibrary.family_clip(_cls(s), AnimLibrary.or_stand_in("idle_look", anim), anim), 1.0, float(still % FIDGET_EVERY) / FIDGET_FRAMES]
		"Airborne":
			# Just dropped through a platform (the sim ignores platforms for a few frames after it): a hop down out of a crouch.
			if int(s.get("ignore", 0)) > 0 and int(s.state_frame) < DROP_FRAMES and flip_left <= 0.0:
				return [AnimLibrary.or_stand_in("platform_drop", anim), 1.0, float(s.state_frame) / DROP_FRAMES]
		"Hitstun":
			# Sent flying by a hit that does not tumble: folded round the blow, or arched up off it when sent upward.
			var kb: Vector2 = s.get("kb_vel", Vector2.ZERO)
			# Tumbling head over heels once the launch slows (the spin is code, below): curled and flailing (animation overhaul pass 8).
			if s.tumble and int(s.hitlag) == 0 and (s.vel + kb).length() <= LAUNCH_POSE_SPEED:
				var tumble := AnimLibrary.or_stand_in("tumble", anim)
				if tumble != "tumble" or anim == null or anim.has_animation("tumble"):
					return [tumble, 1.0, -1.0]
			if not s.tumble and kb.length() > HEAVY_HITSTUN_SPEED:
				return [AnimLibrary.or_stand_in("hurt_up" if kb.y > absf(kb.x) else "hurt_heavy", anim), 1.0, -1.0]
		# (Round 8, the missing sets: states that played another state's clip. `ANIMATION_REVIEW_1.md`.)
		"Helpless":
			# The special fall: limp, arms loosely up (it played the jump's arc, a fighter still in control).
			if not grounded:
				return [AnimLibrary.or_stand_in("helpless", anim), 1.0, -1.0]
		"ItemPickup":
			# A dip and the near hand down to the floor, over the pick-up's frames (it stood in the idle).
			return [AnimLibrary.or_stand_in("item_pick", anim), 1.0, clampf(float(s.state_frame) / ITEM_PICK_FRAMES, 0.0, 1.0)]
		"ItemThrow":
			# Drawn back and whipped through over the throw's frames (it stood in the idle, or fell).
			return [AnimLibrary.or_stand_in("item_throw", anim), 1.0, clampf(float(s.state_frame) / ITEM_THROW_FRAMES, 0.0, 1.0)]
		"Tether":
			# Hanging from the tether's line: the ledge hang's arms-up hold (it fell).
			return ["ledge", 1.0, -1.0]
		"WallTech":
			# A slap against the wall and a push off it: the floor tech's pose over its first frames (it held up a shield).
			return [AnimLibrary.or_stand_in("tech", anim), 1.0, clampf(float(s.state_frame) / WALL_TECH_FRAMES, 0.0, 1.0)]
		"ShieldBreak":
			# Down on the ground after the break's launch: dazed and swaying (it held the hit reaction's flail).
			if grounded:
				return [AnimLibrary.or_stand_in("dizzy", anim), 1.0, -1.0]
		"Buried":
			# Sunk to the waist: both arms pushing at the ground in turn (it stood in the idle).
			return [AnimLibrary.or_stand_in("buried", anim), 1.0, -1.0]
		"LedgeAttack":
			# Up onto the edge, then the sweep (its hit on frames 24-26 of 55 lands on the clip's strike).
			var f: float = s.state_frame
			if f < 14.0:
				return ["ledge_climb", 1.0, clampf(f / 14.0, 0.0, 1.0) * 0.66]
			var p := 0.55 * (f - 14.0) / 11.0 if f < 25.0 else 0.55 + (f - 25.0) / 30.0 * 0.45
			return ["attack_low", 1.0, clampf(p, 0.0, 1.0)]
	# The shared clips (scripts/anim_library.gd): the same for every moveset. The dodges follow their own frames.
	var shared := AnimLibrary.state_clip(state, anim)
	# (Each unarmed family stands and runs its own way: AnimLibrary.CLASS_FAMILY.)
	shared = AnimLibrary.family_clip(_cls(s), shared, anim)
	if shared != "":
		var k := dodge_progress(s)
		if AnimLibrary.BY_PROGRESS.has(state) and k >= 0.0:
			return [shared, 1.0, k]
		return [shared, 1.0, -1.0]
	if grounded:
		return [AnimLibrary.family_clip(_cls(s), "idle", anim), 1.0, -1.0]
	if flip_left > 0.0:
		return ["air_jump", 1.0, -1.0]
	# The air arc (animation overhaul pass 2): the rise and the way down are posed by how fast the fighter climbs or falls, so a full hop,
	# a short hop and a jump off a ledge each go through the same take-off, tuck at the top and opening out on their own timing, and a
	# rollback or a replay seek redraws the same pose (it depends only on the velocity). Falling at full speed, the fall paddles.
	var vy := float(s.vel.y)
	if vy > 0.0:
		return ["jump", 1.0, clampf(1.0 - vy / AIR_RISE_SPEED, 0.0, 1.0)]
	var opening := -vy / AIR_OPEN_SPEED
	var start := AnimLibrary.or_stand_in("fall_start", anim)
	if opening < 1.0 and start != "fall":
		return [start, 1.0, clampf(opening, 0.0, 1.0)]
	return ["fall", 1.0, -1.0]


## With the environment variable PF_R8_OFF (a debugging aid for round 8's before and after images), "sets" plays these states as before
## the round's missing sets: the shared clip by state, or the jump's arc in the air.
static var r8_off := OS.get_environment("PF_R8_OFF").split(",")
const R8_SPECIAL_CLIPS := ["needle_volley", "line_cast", "vanish", "sword_counter", "reflector", "buckler_bow", "buckler_throw",
	"buckler_bomb", "staff_point", "staff_flame", "staff_guard", "great_wave", "great_rise", "nodachi_flare", "nodachi_orbs",
	"nodachi_barrier", "dagger_guard", "dagger_gun"]
const R8_SET_STATES := ["Helpless", "ItemPickup", "ItemThrow", "Tether", "WallTech", "ShieldBreak", "Buried"]


func _choose_clip_before_r8(s: Dictionary) -> Array:
	var shared := AnimLibrary.family_clip(_cls(s), AnimLibrary.state_clip(String(s.state), anim), anim)
	if shared != "":
		return [shared, 1.0, -1.0]
	if int(s.platform) >= 0:
		return [AnimLibrary.family_clip(_cls(s), "idle", anim), 1.0, -1.0]
	return ["fall", 1.0, -1.0]


## The item pick-up and throws' lengths (the ruleset's `item_pick_frames` and `item_side_frames`; the up, down and dash throws run 19-29,
## near enough for the pose) and the wall tech's slap, in sim frames (round 8).
const ITEM_PICK_FRAMES := 8.0
const ITEM_THROW_FRAMES := 24.0
const WALL_TECH_FRAMES := 12.0


## The air arc's speeds (units a frame): rising this fast or faster shows the take-off stretch (the rise is posed between it and the top);
## falling this fast shows the legs fully open (the slowest fastest fall in the roster is 0.16), and the paddling fall takes over.
const AIR_RISE_SPEED := 0.3
const AIR_OPEN_SPEED := 0.15


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
	var release: Dictionary = RELEASE_FRAME.get(_cls(s), {})
	if int(t[1]) == 0 and release.has(String(s.move_name)):
		t = PackedInt32Array([t[0], int(release[String(s.move_name)]), int(release[String(s.move_name)])])
	var start := clampf(float(t[1]), 1.0, total)
	var last := clampf(float(t[2]), start, total)
	var progress := strike_progress(float(s.state_frame), total, start, last, s.get("move_windows", PackedInt32Array()), _end_shape(s))
	var name := String(s.move_name)
	if (HELD_STRIKE.get(_cls(s), []) as Array).has(name):
		# (A hold with no hit to time it, round 3: the Settler's balloon float. Rises into its pose over HELD_STRIKE_IN frames and stays
		# there, the legs paddling, rather than running on to the stance's rest pose (where the guard's near fist sat over the face).)
		var k := smoothstep(0.0, HELD_STRIKE_IN, float(s.state_frame))
		progress = lerpf(0.0, 0.58, k) + 0.025 * k * sin(float(s.state_frame) * TAU / 36.0)
	elif (HELD_COIL.get(_cls(s), []) as Array).has(name):
		# (A charge held in the coil, its arm swinging round on top: `WINDMILL`. Round 2.)
		progress = COIL_PROGRESS + COIL_CREEP * (1.0 + sin(float(s.state_frame) * TAU / 40.0))
		if (COIL_EASE_IN.get(_cls(s), []) as Array).has(name):
			progress *= smoothstep(0.0, HELD_STRIKE_IN, float(s.state_frame))
	elif float(s.state_frame) < start and (COILED_START.get(_cls(s), []) as Array).has(name):
		# (Thrown straight out of a charge's coil: it starts coiled rather than rising to the rest pose and coiling again.)
		progress = maxf(progress, COIL_PROGRESS)
	# Which clip: the moveset's own for this move, or the shared archetype (scripts/anim_library.gd).
	var clip := AnimLibrary.attack_clip(_cls(s), s.move_name, anim)
	# (PF_R8_OFF=specials: round 8's new special clips play their stand-ins again, for before images.)
	if r8_off.has("specials") and R8_SPECIAL_CLIPS.has(clip):
		clip = String(AnimLibrary.STAND_INS.get(clip, clip))
	return [clip, 1.0, clampf(progress, 0.0, 1.0)]


## Where an attack clip is (0..1, on the shared move timeline: wind-up to 0.35, strike at 0.55, follow-through to 0.63, settled at 1) at
## move frame `f`. The swing lands on the move's first active frame and follows through while it is active; through the end lag the body
## holds the follow-through and only settles back near the end (as the reference game's moves do), so a move looks as long as it really
## is. A move with several separate hits (`windows`: [first, last] frame pairs, from the bridge's `fighter_move_windows`) strikes again
## for each: between two hits the body draws partly back toward the wind-up, then snaps forward onto the next hit's first frame, so a
## sequence of hits reads as a rhythm of strikes rather than one long held pose; the last hit gets the full follow-through. Only the
## move's frames drive it, so the animation never sets the timing.
static func strike_progress(f: float, total: float, start: float, last: float, windows: PackedInt32Array, shape := END_SHAPE) -> float:
	if f < start:
		# (Animation overhaul pass 3: the wind-up coils, holds, then snaps, as the reference's moves do, instead of drifting from rest to
		# the strike at one pace. The clip reaches its chamber key (19 of 60) by COIL_AT of the startup, creeps on through the hold, and
		# from SNAP_AT accelerates through the launch into the strike on the first active frame.)
		var u := f / start
		if u <= COIL_AT:
			return COIL_PROGRESS * coil_ease(u / COIL_AT)
		if u <= SNAP_AT:
			return COIL_PROGRESS + COIL_CREEP * (u - COIL_AT) / (SNAP_AT - COIL_AT)
		var k := (u - SNAP_AT) / (1.0 - SNAP_AT)
		return lerpf(COIL_PROGRESS + COIL_CREEP, 0.55, k * k)
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
	# (Round 3, measured against the reference's hitbox animations: the swing carries on into its follow-through and stops within a few
	# frames of the last hit, the pose is held, creeping, for about half the end lag, then the body recovers quickly and settles. It used
	# to drift home at one slow ease across the whole end lag, which read as floaty.)
	return lerpf(0.63, FOLLOW_PROGRESS, follow_k(f, last, shape)) + (1.0 - FOLLOW_PROGRESS) * home_k(f, last, total, shape, 0.0, BODY_CREEP)


## The end lag's shape, shared by the body, the weapon and the reaching limbs (animation round 3, from the reference's hitbox animations:
## `tools/art/review/timing_report.py`). After the last active frame the swing carries on (round 4: `follow_k`, the carry) and slows into
## its follow-through; the pose holds, creeping HOLD_CREEP of the way home, until HOLD_TO of the end lag; it recovers by RECOVER_TO and
## settles by the end.
const HOLD_TO := 0.48
const HOLD_CREEP := 0.15
## The body clip's creep through the hold, smaller: its keys after the follow-through already lead on toward the recovery pose.
const BODY_CREEP := 0.05
const RECOVER_TO := 0.92
## The carry (round 4): how many frames past its last hit a swing keeps going at its pace before it slows (CARRY_EASE more frames) into
## its follow-through. Round 3 stopped every swing dead within 1-2 frames of the hit; the reference's swings carry on 1-7 frames
## (`timing_report.py`'s FT). Measured per move against the reference: a sword's (`SWORD_CARRY`, the reference swordsman's: the jab's
## long cut, the aerials' rolls) and everyone else's (`CARRY_FRAMES`, the reference claws fighter's: one frame).
const CARRY_FRAMES := 1.0
## The sword's neutral air carries nothing on: its windmill already turns past the last hit, and carried further the sword hand came
## over the face (4 frames).
const SWORD_CARRY := {"jab": 4.0, "dtilt": 3.0, "uair": 2.0, "dair": 3.0, "fair": 2.0, "nair": 0.0}
const CARRY_EASE := 2.0
## How much further the body's lunge goes on through the carry (a share of the lunge).
const LUNGE_CARRY := 0.3
## The shape as one value (hold to, recover by, carry frames), and the moves whose reference shapes differ: the up tilt and back air hold
## briefly and recover slowly (both references, both classes).
const END_SHAPE := Vector3(HOLD_TO, RECOVER_TO, CARRY_FRAMES)
const MOVE_END_SHAPES := {"utilt": Vector3(0.3, 0.95, CARRY_FRAMES), "bair": Vector3(0.3, 0.95, CARRY_FRAMES)}
## Clips whose strike pose is not one to hold: the forward headbutt ends head down with the face behind the hands, so it recovers early.
const CLIP_END_SHAPES := {"body_headbutt_fwd": Vector3(0.12, 0.8, CARRY_FRAMES)}
## Per class, the moves whose reference timing differs from the shared shape (hold to, recover by): round 4, measured against the
## reference swordsman. Its forward smash holds about a third of its end lag and then recovers slowly over the rest (ours recovered in 7
## frames against 17); its up air holds briefly and floats back down (24 frames of hold against 11).
## (Round 8, the weight review (`docs/art/anim/ANIMATION_REVIEW_1.md`): every family's hold and recovery measured against its own reference
## fighter's (`timing_report.py`, the reference's measured in the browser the same way), and the moves furthest off reshaped: the quick
## family's aerials and down smash held 12-33 frames where its reference holds 2-6 and floats home over 14-26; the Longsword's forward
## air and jab held too long and recovered in 3-5; the heavy's forward air and smash, the caster's up air and forward smash.)
const CLASS_END_SHAPES := {
	0: {"fsmash": Vector2(0.2, 0.99), "uair": Vector2(0.02, EVEN + 0.99), "fair": Vector2(0.1, EVEN + 0.97), "jab": Vector2(0.08, 0.85),
		"dsmash": Vector2(0.6, 0.9), "bair": Vector2(0.55, 0.85)},
	2: {"nair": Vector2(0.04, EVEN + 0.96), "uair": Vector2(0.03, EVEN + 0.97), "bair": Vector2(0.08, EVEN + 0.97),
		"dsmash": Vector2(0.1, EVEN + 0.97), "fsmash": Vector2(0.5, 0.85)},
	8: {"fair": Vector2(0.55, 0.9), "fsmash": Vector2(0.35, 0.99), "usmash": Vector2(0.04, 0.6)},
	3: {"fair": Vector2(0.75, 0.97), "uair": Vector2(0.2, EVEN + 0.97), "fsmash": Vector2(0.22, EVEN + 0.99)},
}
## The clip's follow-through key (frame 40 of 60).
const FOLLOW_PROGRESS := 0.667
## The weapon arm and the reaching limbs come home this much of the end lag after the body, so the hips lead the recovery.
const ARM_LAG := 0.08


## How far into the follow-through the swing is, `f` frames into the move whose last active frame is `last` (0 at the last hit, 1 stopped):
## the carry. For `shape.z` frames it keeps going at one pace, then it slows to a stop over CARRY_EASE frames (round 4; round 3's ease out
## over 3 frames put most of the travel in the first frame and stopped dead after it).
static func follow_k(f: float, last: float, shape := END_SHAPE) -> float:
	var t := maxf(0.0, f - last)
	var n := shape.z
	var area := n + CARRY_EASE * 0.5
	if t <= n:
		return t / area
	var e := minf(t - n, CARRY_EASE)
	return (n + e - e * e / (2.0 * CARRY_EASE)) / area


## The frames a swing takes to come to rest after its last hit (the carry, then its ease).
static func carry_frames(shape := END_SHAPE) -> float:
	return shape.z + CARRY_EASE


## How far home from the follow-through the body is (0 held, 1 home) through the end lag; `lag` delays it (a share of the end lag) for the
## parts that follow the body home.
## (Round 6: a shape whose recover-by is written plus one (`EVEN`) recovers at one even pace instead of easing in and out, the reference
## swordsman's long, even float home from its up air.)
static func home_k(f: float, last: float, total: float, shape := END_SHAPE, lag := 0.0, creep := HOLD_CREEP) -> float:
	var k := clampf(((f - last) / maxf(1.0, total - last) - lag) / (1.0 - lag), 0.0, 1.0)
	var even := shape.y > 1.0
	var by := shape.y - EVEN if even else shape.y
	if k <= shape.x:
		return creep * k / shape.x
	if k <= by:
		var u := (k - shape.x) / (by - shape.x)
		return lerpf(creep, 0.95, u if even else u * u * (3.0 - 2.0 * u))
	var u := (k - by) / (1.0 - by)
	return lerpf(0.95, 1.0, 1.0 - (1.0 - u) * (1.0 - u))


const EVEN := 1.0


## This move's end-lag shape (`END_SHAPE`, `MOVE_END_SHAPES`, `CLIP_END_SHAPES`) with its carry (`SWORD_CARRY`).
func _end_shape(s: Dictionary) -> Vector3:
	var name := String(s.move_name)
	var shape: Vector3 = MOVE_END_SHAPES.get(name, END_SHAPE)
	if anim != null:
		shape = CLIP_END_SHAPES.get(AnimLibrary.attack_clip(_cls(s), name, anim), shape)
	if bladed(_cls(s)):
		shape.z = float(SWORD_CARRY.get(name, CARRY_FRAMES))
	var own: Dictionary = CLASS_END_SHAPES.get(_cls(s), {})
	if own.has(name):
		shape.x = (own[name] as Vector2).x
		shape.y = (own[name] as Vector2).y
	return shape


## The wind-up's way into the coil: it leaves the rest pose gently and settles into the coil (an ease in and out). It was an ease out,
## so a move's first frame was its wind-up's fastest: the body lurched out of the idle (round 3).
static func coil_ease(k: float) -> float:
	k = clampf(k, 0.0, 1.0)
	return k * k * (3.0 - 2.0 * k)


## The wind-up's shape (shares of the startup, the frames before the first active one): coiled by COIL_AT, held (creeping COIL_CREEP of
## the clip on) until SNAP_AT, then the snap into the strike. The clip's chamber key is COIL_PROGRESS of its length (frame 19 of 60).
const COIL_AT := 0.45
const SNAP_AT := 0.8
const COIL_PROGRESS := 0.317
const COIL_CREEP := 0.02
## Where a fist is drawn back to in a wind-up, from the shoulder (forward space): beside the chest, a little behind and below the shoulder.
const FIST_CHAMBER := Vector2(-0.12, -0.32)


## How far into its coil a wind-up is at move frame `f` (0 at rest, 1 coiled), held, then let go through the snap.
static func coil_k(f: float, first: float) -> float:
	var u := f / maxf(1.0, first)
	if u >= 1.0:
		return 0.0
	if u <= COIL_AT:
		return coil_ease(u / COIL_AT)
	if u <= SNAP_AT:
		return 1.0
	return 1.0 - (u - SNAP_AT) / (1.0 - SNAP_AT)


## How far a strike has travelled from its coil toward the hit at move frame `f`: nothing through the coil and the hold, then
## accelerating from SNAP_AT to the first active frame.
static func snap_k(f: float, first: float) -> float:
	var u := f / maxf(1.0, first)
	if u <= SNAP_AT:
		return 0.0
	var k := clampf((u - SNAP_AT) / (1.0 - SNAP_AT), 0.0, 1.0)
	return k * k


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


## (Turned a little more to the camera than before, after the owner's PEAK reference: both arms and more of the chest read.)
## How the fighter is turned (the "cheat" of a 3D fighter on a 2D stage), as the reference game's fighters stand: the hips and legs turned
## well toward the way it faces (70 degrees off straight-on), so strides, kicks and swings happen across the screen where they read; the
## chest twisted back so it mostly faces the camera (35 degrees off); and the head turned again a little toward the way it faces (45
## degrees off: a three-quarter face), so the two fighters look at each other while the drawn face keeps both eyes in view.
## (Round 2, the owner's pick D after the stance turn mocks: the body side-on as the reference fighters stand, hips 75-82 and chest 72-88
## off the camera, the head held back toward the camera by `FACE_AWAY_MAX` so both eyes stay on the face. Was 60, 24, 10 and 32: the chest
## and face nearly square to the camera.)
const BODY_TURN := 80.0
## How far off straight-on the body turns in the big moments (see `face_cam`), and how far into that turn it is.
const FRONTAL_TURN := 28.0
var face_cam := 0.0
## (Round 7, pass 61, the owner's pick B: 24, the chest turned 18 degrees more toward the camera than option D's 6 with the hips unchanged,
## so the neck turns about 30 degrees from the chest to hold the face 40 off the camera instead of about 48: the reference's heads ride
## the chest.)
const CHEST_TO_CAMERA := 24.0
## How far the head turns back from the chest toward the way the fighter faces (toward its opponent). The eyes in the face look that way
## too (see `set_gaze`).
const HEAD_TOWARD_FOE := 0.0
## In a move: how far the chest still opens to the camera, how much further the hips turn toward the foe, and how far the head may turn
## from the camera (a three-quarter face instead of the near front view of the rest of the time).
## (18 and 40, art review 5: at 6 and 58 a swing whose clip twists the shoulders into it turned the back to the camera, the face gone,
## where the reference's fighters turn their shoulders into a cut but keep the face toward the camera.)
## (Then -14, art review 5: turned to the camera, the chest made every forward reach cross in front of it, the arm coming out of the middle
## of the chest; the reference fighters turn the shoulders into a strike, so the striking shoulder leads and the arm extends straight
## from it. The head alone turns back to keep the face to the camera.)
const CHEST_IN_ATTACK := -14.0
## (0 since round 2: the body is side-on already, so a move only opens the chest toward its hit, `CHEST_IN_ATTACK`, never passing side-on.)
const BODY_TURN_IN_ATTACK := 0.0
const FACE_AWAY_IN_ATTACK := 40.0
## The standing pivot (round 8): the states that turn on the spot, how fast the body swings round (per second, as a lerp), how much of
## the turn still to come the head is already turned through (it leads), and that turn still to come (degrees).
const PIVOT_STATES := ["Idle", "Turn", "Walk", "Crouch", "Shield", "ShieldRelease", "Landing"]
const PIVOT_RATE := 30.0
const HEAD_TURN_LEAD := 0.7
var yaw_left := 0.0
## How far into a move the body is turned toward the foe (0 none, 1 fully), eased in and out.
var attack_turn := 0.0
## How far the body is turned round toward a hit behind it (0 none, 1 fully; see BACK_ATTACK_TURN).
var back_turn := 0.0
## How far round toward the back the body turns for a hit behind (degrees, toward the back from facing the camera).
const BACK_ATTACK_TURN := 35.0


## Facing left, the base body plays every clip mirrored (art review 5): the model turns rather than mirrors, so the arm nearest the camera
## is the left one facing right and the right one facing left, while a clip poses fixed bones. Unmirrored, a clip's free arm was the near
## arm one way and the far arm the other, and the weapon arm (always the near one) fought it. Each clip's mirror image swaps left and
## right bones and reflects the poses across the body's middle (every bone's rest frame is the model's own, so a rotation (x, y, z, w)
## reflects to (x, -y, -z, w) and a position's x changes sign).
const MIRROR_SUFFIX := "@m"


func _add_mirrored_clips() -> void:
	var lib: AnimationLibrary = anim.get_animation_library("")
	if lib == null:
		return
	for clip_name in lib.get_animation_list():
		if String(clip_name).ends_with(MIRROR_SUFFIX) or lib.has_animation(clip_name + MIRROR_SUFFIX):
			continue
		var a: Animation = lib.get_animation(clip_name).duplicate(true)
		for t in a.get_track_count():
			var path := String(a.track_get_path(t))
			if path.ends_with(".L"):
				path = path.left(-2) + ".R"
			elif path.ends_with(".R"):
				path = path.left(-2) + ".L"
			a.track_set_path(t, NodePath(path))
			match a.track_get_type(t):
				Animation.TYPE_ROTATION_3D:
					for k in a.track_get_key_count(t):
						var q: Quaternion = a.track_get_key_value(t, k)
						a.track_set_key_value(t, k, Quaternion(q.x, -q.y, -q.z, q.w))
				Animation.TYPE_POSITION_3D:
					for k in a.track_get_key_count(t):
						var v: Vector3 = a.track_get_key_value(t, k)
						a.track_set_key_value(t, k, Vector3(-v.x, v.y, v.z))
		lib.add_animation(clip_name + MIRROR_SUFFIX, a)


## The clip to play for `clip` this way round (see `_add_mirrored_clips`).
func _sided(clip: String, facing: int) -> String:
	if use_base_body and facing < 0 and anim != null and anim.has_animation(clip + MIRROR_SUFFIX):
		return clip + MIRROR_SUFFIX
	return clip


var shown_clip := ""


## The dash and the runs (the shared one and each family's, not the stop out of a run): the clips whose stride carries on from one to the
## next and whose footfalls squash the body.
static func _gait_clip(c: String) -> bool:
	return c == "dash" or c == "run" or (c.begins_with("run_") and c != "run_stop")


## Plays the right clip for this frame and moves the head and torso followers with their bones.
func _animate(s: Dictionary, delta: float) -> void:
	if anim == null:
		return
	var pick: Array = _choose_clip(s) if not r8_off.has("sets") or not R8_SET_STATES.has(String(s.state)) else _choose_clip_before_r8(s)
	var clip: String = pick[0]
	# Where the stride is: a dash to a run goes on in step, and a new dash (also one the other way) starts on its push-off.
	var gait := -1.0
	if _gait_clip(current_clip):
		gait = fmod(anim.current_animation_position / maxf(anim.current_animation_length, 0.001), 1.0)
	var new_dash: bool = s.state == "Dash" and (current_clip != "dash" or int(s.state_frame) < gait_frame)
	gait_frame = int(s.state_frame) if s.state == "Dash" else -1
	var shown := _sided(clip, int(s.facing))
	if clip == current_clip and shown != shown_clip:
		# (Turned round mid-clip: the other side's clip, at the same point.)
		var at := anim.current_animation_position
		anim.play(shown, 0.0)
		anim.seek(at, true)
		shown_clip = shown
	if clip != current_clip:
		# A clip that follows the move (or the ledge climb) starts at once: it is posed by seeking while paused, and seeking never moves
		# a cross-fade on, so it would stay showing the clip before.
		var follows: bool = float(pick[2]) >= 0.0
		# Likewise a hit's pose: a fighter frozen by hitlag is already in it for the whole freeze (the animation does not move on then).
		var frozen: bool = int(s.hitlag) > 0
		anim.play(shown, (JUMP_SQUAT_BLEND if clip == "jumpsquat" else 0.1) if current_clip != "" and not follows and not frozen and not clip.begins_with("attack") else 0.0)
		shown_clip = shown
		if frozen and not follows:
			anim.advance(0.0)
		current_clip = clip
		if clip != "dash" and _gait_clip(clip) and gait >= 0.0:
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
	elif anim.current_animation != "":
		# (Round 4: frozen, the clip is still put back on its frame every frame, so the layers on top (the flinch, the body follow, the
		# face limiter) start again from the clip's pose. Not advanced, the bones kept last frame's layered pose and every layer was
		# added again on top: through a pummel's freeze the grabbed fighter's head bent round in a circle and its hand crossed the face.)
		anim.seek(anim.current_animation_position, true)
	_rest_unkeyed(clip)
	_bulk_arms(_cls(s))
	_body_follow(s, delta)
	_lag_bones(s)
	_face_camera(int(s.facing), delta)
	_follow_bones()


## Arm bulk by class: [forearm scale, hand scale] (the bone's own axes: x and z across it, y along it). The Hauler (Bracers) has huge
## forearms and fists, thick across and a little long; every other class its plain arms. Applied to the pose every frame (`_bulk_arms`)
## and to the arm IK's overrides (`_aim_arm`), so the bulk holds through its swings.
const ARM_BULK := {7: [Vector3(1.55, 1.08, 1.55), Vector3(1.45, 1.35, 1.45)]}

## The base body's proportions on top of its model (art review 10, after the reference cast and the crossover fighter's plumber: a big
## head that carries the face and the hat at play size, big gloved hands and chunky feet): uniform scales on the end bones, so a bent
## elbow or knee is never sheared. The head grows up from the neck; hats and face pieces follow it (`_follow_bones`).
## (The body rebuild modelled the head to the proportion study's height; the reference's heads are wide ovals, so it is widened here,
## across and front to back, the bone's own x and z.)
## (Wider and squatter again, the character pass, 2026-10-19: the reference cast's heads are broad squat ovals, wider than the shoulders.)
const HEAD_SIZE := Vector3(1.45, 1.1, 1.32)
## (1.4, the character judge's round 7: the reference's gloves are about half the head's width, a main shape.)
const HAND_SIZE := 1.0   # (the character judge's round 19 wanted the hand about 0.3 of the head's height; with the slimmer wrist (the hand grows with it) 0.8 read tiny to the owner)
## How strong the skin's faint painted grain is (`skin_grain` in shaders/skin.gdshader), the character pass, 2026-10-19.
const SKIN_GRAIN := 0.0   # (0.15 until the matte pass, 2026-10-19: the reference cast's skin carries a visible soft speckle; 0.3 until the
                           # finish pass, 2026-10-20: with the cloth's weave it read as felt, the owner's "sloppy", against the references'
                           # clean surfaces)
## (0.9, the character judge's round 1: the reference's shoes are small; big rounded ones read as clown shoes.)
const FOOT_SIZE := 0.9


## This class's arm bulk ([forearm, hand] scales) with the body's own hand size in it.
func _arm_bulk(cls: int) -> Array:
	var b: Array = ARM_BULK.get(cls, [Vector3.ONE, Vector3.ONE])
	return [b[0], (b[1] as Vector3) * (HAND_SIZE if use_base_body else 1.0)]


## The body's proportions for a fighter shown outside a match (the menus' previews), whose pose is not stepped by `apply`.
func apply_proportions(cls := -1) -> void:
	_bulk_arms(cls)
	# (And the hat and face pieces onto the grown head, as `apply` does.)
	_follow_bones()


func _bulk_arms(cls: int) -> void:
	if skeleton == null:
		return
	if use_base_body:
		if head_bone >= 0:
			skeleton.set_bone_pose_scale(head_bone, Loadout.HEAD_SCALES.get(Loadout.HEADS[clampi(int(loadout.get("head")), 0, Loadout.HEADS.size() - 1)], HEAD_SIZE) if loadout != null else HEAD_SIZE)
		for side in ["L", "R"]:
			var f := skeleton.find_bone("foot." + side)
			if f >= 0:
				skeleton.set_bone_pose_scale(f, Vector3.ONE * FOOT_SIZE)
	var b: Array = _arm_bulk(cls)
	for side in ["L", "R"]:
		var il := skeleton.find_bone("armL." + side)
		var ih := skeleton.find_bone("hand." + side)
		if il >= 0:
			skeleton.set_bone_pose_scale(il, b[0])
		if ih >= 0:
			skeleton.set_bone_pose_scale(ih, b[1])


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
	_slide_face()


## How much of the camera's angle round the head the face drawing follows (face.gdshader `face_slide`), and the most it slides: the far eye
## stays readable at three-quarters, while the face still turns with the head (the face pass, 2026-10-19).
const FACE_SLIDE := 0.45
var glasses_node: Node3D
var glasses_base := Transform3D()
const FACE_SLIDE_MAX := 0.5
## Seen from behind, the slide fades out between these angles round the head (radians from its front): it is for the far eye at
## three-quarters, and from behind it only pulled the face round into view at the head's edge.
const FACE_SLIDE_BACK := Vector2(1.75, 2.6)


func _slide_face() -> void:
	if face_mat == null or not head_rig.is_inside_tree():
		return
	var cam := get_viewport().get_camera_3d() if is_inside_tree() else null
	if cam == null:
		return
	# The camera's direction in the head's own frame (its front is +z), the frame's scale taken out.
	var head := head_rig.global_transform.orthonormalized()
	var local := head.affine_inverse() * cam.global_position
	var around := atan2(local.x, local.z)
	var slide := clampf(around * FACE_SLIDE, -FACE_SLIDE_MAX, FACE_SLIDE_MAX)
	slide *= 1.0 - smoothstep(FACE_SLIDE_BACK.x, FACE_SLIDE_BACK.y, absf(around))
	face_mat.set_shader_parameter("face_slide", slide)
	# The glasses turn round the head with the drawing, so the eyes stay behind their lenses (in the head's design space, a ball about
	# 1.42 up).
	if glasses_node != null and is_instance_valid(glasses_node):
		var turn := Basis(Vector3.UP, slide)
		var pivot := Vector3(0.0, 1.42, 0.0)
		glasses_node.transform = Transform3D(turn, pivot - turn * pivot) * glasses_base


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
	if s.state == "Attack" and tip != Vector3.ZERO and spine_bone >= 0 and _spin(s) == 0.0 and not _held_pose(s):
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
				# (Pose review 2026-10-17: 14 on the base body; at 26 a high kick arched the round body back, belly first.)
				target_bend = -minf((phi - 100.0) * 0.85, 14.0 if use_base_body else 50.0)
			elif phi < -40.0:
				# (Less: kicks behind already lean the body forward in their clips.)
				target_bend = minf((-40.0 - phi) * 0.4, 10.0 if use_base_body else 24.0)
		else:
			# An arm reaches almost anywhere: lean into low swings in front, a little back from ones overhead or behind.
			if phi > 0.0 and phi < 80.0:
				target_bend = (80.0 - phi) * 0.35
			# (Round 8, line of action C: the chest bends further toward a strike in front, so the body lines up behind it.)
			if use_base_body and phi > 0.0 and phi < 130.0 and int(s.platform) >= 0:
				target_bend += float(_line_of_action(s).y)
			elif phi > 140.0 and not use_base_body:
				# (Not on the base body: it reaches overhead from the shoulders with the chest lifted; bent back, the round body read as
				# inhaling. Pose review 2026-10-17.)
				target_bend = -(phi - 140.0) * 0.4
			elif phi < -60.0:
				target_bend = (-60.0 - phi) * 0.3
		# Look at the hit: up for a hit overhead, down for a low one, by how high it is (about half of the angle to it).
		var rise := rad_to_deg(atan2(to.y, absf(to.x)))
		# (Less far up for a kick: the body already leans back under a high one, and the face must stay toward the camera.)
		# (The base body only glances up, 8 at most: its big head tipped back shows the chin, not the face. Pose review 2026-10-17.)
		target_look = clampf(rise * 0.5, -20.0, 8.0 if use_base_body else (12.0 if kicking else 30.0))
		# In proportion to how far into the move the limb is (its blend toward the hitbox). The base body's clips already lean and look
		# into its swings and kicks, and its legs reach high on their own, so it takes half of this.
		# (A kick on the base body takes more of the counter-lean: the reference fighters' whole body lines up with a kick, standing tall
		# under a high one and leaning back from one in front. Art review 5.)
		var k := (leg_k if kicking else arm_k) * ((0.85 if kicking else 0.5) if use_base_body else 1.0)
		# (A sword on the base body: its clips already lean into the cut, and the extra bend folded a low cut nearly double with the head
		# down at the blade, where the reference keeps the back long and the head up. Art review 5.)
		if use_base_body and bladed(_cls(s)) and not kicking:
			k *= 0.4
			target_look = maxf(target_look, -6.0)
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
		# (Round 4: by sim frames, as the springs are, so a hit's freeze holds the flinch where the hit put it and a seek ends it. On real
		# time it ran on through the freeze: a grabbed fighter's head bent back and forward again within each pummel's hitlag, and its
		# weapon hand crossed the face, 5-8 frames a pummel.)
		flinch_left = 0.0 if springs.jumped else maxf(0.0, flinch_left - float(springs.new_frames) / 60.0)
		var k := flinch_left / FLINCH_TIME
		_bend_bone(spine_bone, -16.0 * k, int(s.facing))
		_bend_bone(head_bone, -12.0 * k, int(s.facing))
	# Open the chest to the camera and turn the head back toward the opponent (see BODY_TURN). In a move the chest turns into it instead,
	# toward the opponent (`attack_turn`), so a strike is thrown at the foe and not at the camera.
	_turn_bone(spine_bone, -lerpf(CHEST_TO_CAMERA, CHEST_IN_ATTACK, attack_turn) * float(s.facing))
	_turn_bone(head_bone, HEAD_TOWARD_FOE * float(s.facing))
	if use_base_body and not turning_over and not r7_off.has("chest"):
		_keep_chest_side_on()


## The chest never turns its back to the camera (round 7): past CHEST_SOFT degrees from the camera its turn is eased toward CHEST_MOST and
## never past it. A clip's shoulder turn into a strike, amplified, swung the side-on chest round to 150-180 (the Longsword's jab, tilts,
## smashes and specials, 15% of its frames), while the face limit held the head toward the camera: the neck twisted up to 131 degrees, an
## owl's head. Side-on is the stance's own rule (pass 28: a move "never passes side-on"); the reference fighters' heads ride the chest.
## (Pass 61, the owner's pick B: 72 and 90, moved 18 toward the camera with the stance's chest (`CHEST_TO_CAMERA`), so a strike's chest
## stops at side-on and the neck stays within about 50 of it; at 90 and 108 a strike swung the neck from 30 to 60 on every move.)
const CHEST_SOFT := 72.0
const CHEST_MOST := 90.0
## With the environment variable PF_R7_OFF (a debugging aid for round 7's before and after images), the round's layers named in it are
## left out: "chest" (`_keep_chest_side_on`), "level" (`_level_head` and FACE_UP_MAX), "ease" (`_ease_neck`), "legs" (`_leg_hinge` and
## the kick's knee pole), "tumble" (the settle out of a tumble), e.g. PF_R7_OFF=ease,legs.
static var r7_off := OS.get_environment("PF_R7_OFF").split(",")


func _keep_chest_side_on() -> void:
	var g: Basis = skeleton.get_bone_global_pose(spine_bone).basis * skeleton.get_bone_global_rest(spine_bone).basis.inverse()
	var ahead: Vector3 = (_skeleton_to_frame().basis * g * Vector3(0, 0, 1))
	if Vector2(ahead.x, ahead.z).length() < 0.3:
		return
	var away := rad_to_deg(atan2(ahead.x, ahead.z))
	var a := absf(away)
	if a <= CHEST_SOFT:
		return
	var span := CHEST_MOST - CHEST_SOFT
	var kept := CHEST_SOFT + span * (1.0 - exp(-(a - CHEST_SOFT) / span))
	_turn_bone(spine_bone, -signf(away) * (a - kept))


## The line of action in grounded strikes (round 8, the owner's pick C after the mock, 2026-10-28): [how much of a level strike's swing the
## whole body leans into (it was 0.35 for every family, about 4 degrees), the chest's extra bend toward a strike in front (degrees)], by
## family, fitted to each family's reference: the wolf's and dinosaur's strikes and the swordsman's lunges run nearly horizontal; the
## ninja's high kicks and the plumber's palm stand taller; the villager's tool swings stay upright.
const LINE_OF_ACTION := {"sword": Vector2(1.6, 24.0), "brawler": Vector2(1.6, 24.0), "heavy": Vector2(1.6, 24.0),
	"quick": Vector2(0.9, 12.0), "caster": Vector2(1.1, 14.0), "tool": Vector2(0.7, 8.0)}


## Classes standing apart from their family's line: the Seer (staff), in the sword stance, stands tall as the staff reference does (bent
## over like the swordsmen, its casting hand came over the face for 31 frames of its neutral special).
const CLASS_LINE_OF_ACTION := {STAFF_CLASS: Vector2(0.7, 8.0)}


func _line_of_action(s: Dictionary) -> Vector2:
	if CLASS_LINE_OF_ACTION.has(_cls(s)):
		return CLASS_LINE_OF_ACTION[_cls(s)]
	return LINE_OF_ACTION.get(String(AnimLibrary.CLASS_FAMILY.get(_cls(s), "sword")), LINE_OF_ACTION.sword)


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
const FACE_AWAY_MAX := 40.0
## Nor does it look down further than this (a pitched-down head shows its crown, and the hat).
const FACE_DOWN_MAX := 18.0
## The most the head is tipped back to keep that (degrees), so the neck never bends further than a neck can.
const FACE_UP_MOST := 45.0
## Nor does it look up further than this (round 7: a body arched back under an up air, or rocked back in a forward air's wind-up, tipped
## the face 40 up, showing the chin; the reference fighters' heads stay near level and look along the move).
const FACE_UP_MAX := 24.0
## The head is kept near upright (round 7, after the reference fighters, whose heads stay level while the body leans and twists): its tilt
## to the side about the way it looks is cut to this share, and to no more than HEAD_ROLL_MAX degrees. The side-on stance's sideways
## chest bend had left every idle, walk, run and shield with the head cocked about 18 degrees, gone again in the air: a tilt that came
## and went on every jump.
const HEAD_ROLL_KEEP := 0.3
const HEAD_ROLL_MAX := 7.0
## The head eases toward the way all of that asks it to look (round 7), a share of the way each sim frame and never faster than
## NECK_MAX_STEP degrees a frame: a clip changing (an attack's first frame is posed at once), or a limit above taking hold or letting go,
## turned the head 20 to 77 degrees in one frame, the jolts at the start and end of moves, jumps and landings. Slow motion passes through
## (the idle's nod lags it by about a frame). Eased in the stage frame, so the body's own turns and leans carry the head at once; while
## the body turns over on purpose (a flip, a roll, a tumble) it rides the body exactly.
const NECK_EASE := 0.4
const NECK_MAX_STEP := 20.0
var neck_q := Quaternion.IDENTITY
var neck_ok := false
## Whether the whole body is turning over on purpose this frame (a flip, a roll, a tumble), when the face's limit against the world is off.
var turning_over := false


func _face_camera(facing: int, delta := 1.0 / 60.0) -> void:
	if not use_base_body or skeleton == null or head_bone < 0 or not skeleton.is_inside_tree():
		return
	# (Three times, round 2: a head pitched far down has no clear turn, and lifting it changes how far it is turned, so one pass left a curled
	# aerial's face 30 down and 75 away; measured again after the first pass, both limits hold.)
	face_fix_turn = 0.0
	face_fix_lift = 0.0
	# (Round 8: turning round on the spot, the head goes first, already turned through most of the turn still to come.)
	if absf(yaw_left) > 1.0:
		_turn_bone(head_bone, yaw_left * HEAD_TURN_LEAD)
	for _pass in 3:
		_face_camera_once(facing)
	if not turning_over and not r7_off.has("level"):
		_level_head()
	if not r7_off.has("ease"):
		_ease_neck(delta)


## Cuts the head's tilt to the side (about the way it looks, against the world's vertical) to HEAD_ROLL_KEEP of itself.
func _level_head() -> void:
	var head_delta: Transform3D = skeleton.get_bone_global_pose(head_bone) * head_rest_inv
	var to_world := skeleton.global_transform.basis
	var fwd: Vector3 = (to_world * head_delta.basis * Vector3(0, 0, 1)).normalized()
	if absf(fwd.y) > 0.85:
		return
	var up: Vector3 = (to_world * head_delta.basis * Vector3(0, 1, 0)).normalized()
	var roll := _roll_about(fwd, up)
	var want := clampf(roll * HEAD_ROLL_KEEP, -HEAD_ROLL_MAX, HEAD_ROLL_MAX)
	if absf(roll - want) > 0.05:
		_rotate_bone_about(head_bone, (to_world.inverse() * fwd).normalized(), want - roll)


## The neck's own turn (the head bone's pose on the chest) eased toward the one asked for (see NECK_EASE). On the springs' clock: held
## through a hit's freeze and a pause, and straight on its target after a rollback or a replay seek, so either redraws a frame exactly.
func _ease_neck(delta: float) -> void:
	var to_frame := _skeleton_to_frame().basis.orthonormalized()
	var parent := skeleton.get_bone_parent(head_bone)
	var want := (to_frame * skeleton.get_bone_global_pose(head_bone).basis.orthonormalized()).get_rotation_quaternion()
	if not neck_ok or springs.jumped or turning_over:
		neck_q = want
		neck_ok = true
		return
	if springs.flowing:
		var frames := delta * 60.0
		var angle := rad_to_deg(neck_q.angle_to(want))
		if angle > 0.01:
			# (Round 8: turning round on the spot the head leads, so it is let go faster than the body.)
			var pivoting := absf(yaw_left) > 1.0
			var ease := 0.8 if pivoting else NECK_EASE
			var step := minf(angle * (1.0 - pow(1.0 - ease, frames)), (45.0 if pivoting else NECK_MAX_STEP) * frames)
			neck_q = neck_q.slerp(want, clampf(step / angle, 0.0, 1.0))
	var global_basis := to_frame.inverse() * Basis(neck_q)
	var pg := skeleton.get_bone_global_pose(parent).basis.orthonormalized()
	skeleton.set_bone_pose_rotation(head_bone, (pg.inverse() * global_basis).get_rotation_quaternion())


func _face_camera_once(facing: int) -> void:
	# The pitch first, then the turn: a head looking nearly straight down has no clear turn, so turning it first spun it round.
	var head_delta: Transform3D = skeleton.get_bone_global_pose(head_bone) * head_rest_inv
	var ahead: Vector3 = (_skeleton_to_frame().basis * head_delta.basis * Vector3(0, 0, 1)).normalized()
	var down := -rad_to_deg(asin(clampf(ahead.y, -1.0, 1.0)))
	# (Round 2: also against the world, not only the leaning body. The whole body's pitch (`lean`: an aerial's counter-pitch, a curl over
	# a forward air) is outside the stage frame, so a body pitched 50 forward looked at the floor with only the hat showing, judged frame
	# by frame in the Longsword's forward, up and down airs. Not while the body turns over on purpose: a flip, a roll, a tumble.)
	if not turning_over:
		var world_ahead: Vector3 = (skeleton.global_transform.basis * head_delta.basis * Vector3(0, 0, 1)).normalized()
		down = maxf(down, -rad_to_deg(asin(clampf(world_ahead.y, -1.0, 1.0))))
	if down > FACE_DOWN_MAX:
		# (Lifted about the head's own side-to-side axis, round 2: bent about the camera's axis, a head turned well away from the camera
		# rolled over sideways instead of looking up.)
		var head_ahead: Vector3 = head_delta.basis * Vector3(0, 0, 1)
		var up_skel: Vector3 = (skeleton.global_transform.basis.inverse() * Vector3.UP).normalized()
		var axis := head_ahead.cross(up_skel)
		# (Round 4: a head looking nearly straight down, or down and back, is lifted toward the way the fighter faces. Lifted toward its
		# own little sideways lean, which flips with a hair's change of pose, it came up looking backwards: the Embers' forward air
		# turned its face away for 15 frames from its last hit, showing the top of its head.)
		var fwd_skel: Vector3 = skeleton.global_transform.basis.inverse() * Vector3(float(facing), 0.0, 0.0)
		fwd_skel = (fwd_skel - up_skel * fwd_skel.dot(up_skel)).normalized()
		var level := head_ahead - up_skel * head_ahead.dot(up_skel)
		if level.length() < 0.5 * head_ahead.length() or level.dot(fwd_skel) < 0.0:
			axis = fwd_skel.cross(up_skel)
		if axis.length() > 0.01:
			_rotate_bone_about(head_bone, axis.normalized(), minf(down - FACE_DOWN_MAX, FACE_UP_MOST))
			face_fix_lift += minf(down - FACE_DOWN_MAX, FACE_UP_MOST)
		head_delta = skeleton.get_bone_global_pose(head_bone) * head_rest_inv
		ahead = (_skeleton_to_frame().basis * head_delta.basis * Vector3(0, 0, 1)).normalized()
	elif down < -FACE_UP_MAX and not turning_over and absf(ahead.y) < 0.95 and not r7_off.has("level"):
		# (Round 7: and no further up than FACE_UP_MAX, tipped down about its own side-to-side axis.)
		var up_ahead: Vector3 = head_delta.basis * Vector3(0, 0, 1)
		var up_axis := up_ahead.cross((skeleton.global_transform.basis.inverse() * Vector3.UP).normalized())
		var world_up := rad_to_deg(asin(clampf((skeleton.global_transform.basis * up_ahead).normalized().y, -1.0, 1.0)))
		var over := minf(-down, world_up) - FACE_UP_MAX
		if over > 0.0 and up_axis.length() > 0.01:
			_rotate_bone_about(head_bone, up_axis.normalized(), -over)
			face_fix_lift -= over
			head_delta = skeleton.get_bone_global_pose(head_bone) * head_rest_inv
			ahead = (_skeleton_to_frame().basis * head_delta.basis * Vector3(0, 0, 1)).normalized()
	# The turn, only for a head that is not still looking well down (its turn is not clear then).
	if absf(ahead.y) < 0.8:
		var away := rad_to_deg(atan2(ahead.x, ahead.z))
		var limit := lerpf(FACE_AWAY_MAX, FACE_AWAY_IN_ATTACK, attack_turn)
		if absf(away) > limit:
			_turn_bone(head_bone, -signf(away) * (absf(away) - limit))
			face_fix_turn += absf(away) - limit


## Turns a bone by `degrees` about `axis` (skeleton space), on top of its pose.
func _rotate_bone_about(bone: int, axis: Vector3, degrees: float) -> void:
	var parent := skeleton.get_bone_parent(bone)
	if parent < 0:
		return
	var turn := Basis(axis, deg_to_rad(degrees))
	var g := skeleton.get_bone_global_pose(bone)
	var pg := skeleton.get_bone_global_pose(parent)
	skeleton.set_bone_pose_rotation(bone, (pg.basis.inverse() * (turn * g.basis)).get_rotation_quaternion())


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


## How big a raised weapon is in the victory pose (evenly, so the hilt keeps its shape), so the blade stays inside the results panel.
const VICTORY_WEAPON_SCALE := 0.7


## A victory pose for the results screen: plays one of the looping victory clips (by class: the sword is raised high, the claws fighter
## pumps a fist; `cheer` makes anyone throw both arms up and hop), with the weapon held up in the raised hand. Call every
## frame with the time since the last.
func play_victory(cls: int, delta: float, cheer := false) -> void:
	if anim == null:
		return
	var clip := "victory_c" if cheer else ("victory_b" if not bladed(cls) else "victory_a")
	set_expression(Loadout.HAPPY_FACE)
	_blink(delta)
	if current_clip != clip and anim.has_animation(clip):
		anim.play(clip, 0.15)
		current_clip = clip
		shown_clip = clip
	anim.speed_scale = 1.0
	anim.advance(delta)
	_follow_bones()
	# The sword, gripped in the raised hand and pointing up.
	var armed := bladed(cls) and not cheer
	# (Every armed class's own weapon, shrunk evenly to fit the results panel: it drew only the longsword or the greatsword, squeezed to
	# 0.7 of its length alone, which flattened the hilt.)
	_show_weapon_of(cls if armed else -1)
	if armed:
		var hand := skeleton.find_bone("hand.R")
		var at: Vector3 = _skeleton_to_frame() * skeleton.get_bone_global_pose(hand).origin
		blade_pivot.position = at
		blade_pivot.rotation = Vector3(0, 0, PI / 2.0)
		blade_pivot.scale = Vector3.ONE * VICTORY_WEAPON_SCALE


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


## States where the hands open (see `_clench`).
## States in which a held weapon trails in the loose hand instead of being held in guard (see `carrying`).
const REELING_STATES := ["Hitstun", "Tumble", "Knockdown", "GetUp", "ShieldBreak", "Grabbed"]
const OPEN_HAND_STATES := ["Grabbing", "Grabbed", "Hitstun", "Tumble", "LedgeHang", "LedgeGetUp", "Knockdown", "GetUp", "ShieldBreak",
	"Respawn", "Helpless", "ItemPickup", "ItemThrow"]
## Moves whose reference opens the hand (from the moveset reports' art notes): by class, the sim's move names. A hand holding a weapon
## stays closed on it. Embers' cape, the Staff's warp and reflector, the Pockets' catch and release, the Popgun's banana and latch, the
## Dagger's guard and counter stance (an open palm), the Nodachi's flare.
const OPEN_HAND_MOVES := {3: ["side special"], 5: ["up special", "down special"], 10: ["neutral special"],
	11: ["side special", "down special"], 12: ["down special"],
	13: ["neutral special", "extra move 3", "extra move 4", "extra move 5", "side special"]}


## Which arm holds the weapon: on the base body the one nearest the camera (the model's left facing right), so a swing's arm reads in
## front of the body; it was the far one, hidden behind the torso through every strike with only the blade showing (art review 5). The
## old built body keeps its right.
func _weapon_side(facing: int) -> String:
	if use_base_body:
		return "L" if facing > 0 else "R"
	return "R" if facing > 0 else "L"


## The base body's closed fists and the open hands they stand in for, by side ("L", "R"): see `_clench`.
var fists := {}
var open_hands := {}


## Closes a hand into a fist (or opens it): a hand that grips a weapon, and an unarmed fighter's hands while it attacks. (An open hand
## beside a sword's hilt read as not holding it; a punch with an open hand read as a slap.)
## The look's menu stance (Loadout.POSES, the character judge's round 10): its clip playing slowly from a resting moment, the hands open
## or closed by the stance. Menus only; a match poses the fighter from its state.
func hold_pose() -> void:
	if anim == null or loadout == null:
		return
	var p: String = Loadout.POSES[clampi(int(loadout.get("pose")), 0, Loadout.POSES.size() - 1)]
	var clip: String = Loadout.POSE_CLIPS.get(p, "idle")
	if not anim.has_animation(clip):
		clip = "idle"
	if anim.has_animation(clip):
		anim.play(clip)
		anim.seek(0.6 * anim.current_animation_length, true)
		anim.speed_scale = 0.35
		current_clip = clip
		shown_clip = clip
	var open: bool = Loadout.POSE_OPEN.get(p, false)
	if not fists.is_empty():
		_clench(not open, not open)


func step_pose(delta: float) -> void:
	if anim != null and anim.is_playing():
		anim.advance(0.0)
	_follow_bones()
	if menu_weapon >= 0:
		_pose_menu_weapon()


## The class whose weapon a menu fighter holds, or -1 for none (the creator, fighter select and the main menu: art polish 2026-10-29;
## the menus showed every fighter bare-handed, so choosing a weapon showed nothing).
var menu_weapon := -1


## Shows only class `cls`'s weapon parts (the pivot weapons, the claws on both fists, the knife in the weapon hand).
func _show_weapon_of(cls: int, facing := 1) -> void:
	blade_pivot.visible = bladed(cls) or cls == STAFF_CLASS
	for p in blade_parts:
		p.visible = cls == 0 or cls == BUCKLER_CLASS
	for p in great_parts:
		p.visible = cls == GREATSWORD_CLASS
	for p in staff_parts:
		p.visible = cls == STAFF_CLASS
	for p in nodachi_parts:
		p.visible = cls == NODACHI_CLASS
	for p in claw_parts:
		p.visible = cls == 1
	for p in dagger_parts:
		p.visible = cls == DAGGER_CLASS and String(p.get_meta("side")) == _weapon_side(facing)


## A menu fighter's weapon in the weapon hand, at the class's resting carry as in a match (`_rest_angle`: a sword trailing low and back,
## the big blades lower, the staff upright), at its true size.
func _pose_menu_weapon() -> void:
	if skeleton == null:
		return
	_show_weapon_of(menu_weapon)
	if not blade_pivot.visible:
		return
	var hand := skeleton.find_bone("hand." + _weapon_side(1))
	if hand < 0:
		return
	blade_pivot.position = _skeleton_to_frame() * skeleton.get_bone_global_pose(hand).origin
	blade_pivot.rotation = Vector3(0, 0, deg_to_rad(_rest_angle({"class": menu_weapon, "char": menu_weapon})))
	blade_pivot.scale = Vector3.ONE
	if fists.has(_weapon_side(1)):
		(fists[_weapon_side(1)] as Node3D).visible = true
		if open_hands.has(_weapon_side(1)):
			(open_hands[_weapon_side(1)] as Node3D).visible = false


func _clench(free: bool, weapon: bool, facing := 1) -> void:
	for side in fists:
		var closed: bool = weapon if side == _weapon_side(facing) else free
		(fists[side] as Node3D).visible = closed
		if open_hands.has(side):
			(open_hands[side] as Node3D).visible = not closed


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
	# (The folded paper medallion with the effects' paper look; the older painted bubble with `--fx=ink`.)
	sm.set_shader_parameter("paper", FxMaterial.paper)
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
	# The modelled weapons on the pivot (art/blender/make_weapons.py, 2026-10-28: they replaced the code-built ones, laid out the same).
	# The longsword (the Duelist's and the Wanderer's): a slim diamond-section blade with a blue fuller, a gold hilt and a wrapped grip.
	var gold := respond(toon(Color(0.92, 0.76, 0.3)), "gold")
	blade_parts.append_array(_pivot_weapon("longsword", {"metal": respond(toon(Color(0.82, 0.87, 0.96)), "metal"),
		"dark": respond(toon(Color(0.36, 0.5, 0.82)), "metal"), "gold": gold, "band": toon(SASH)}))
	# The modelled weapons worn on the hands (art/blender/make_weapons.py): the Claws class's claws, one on each fist.
	_mount_weapon_models()
	# The Seer's staff (class STAFF_CLASS), on the same pivot so it is posed as the blade is: a pale wooden shaft held near its foot, a
	# gold collar and ring, and a glowing crystal in the ring.
	staff_parts.clear()
	var staff_gold := respond(toon(Color(0.95, 0.78, 0.32)), "gold")
	staff_parts.append_array(_pivot_weapon("staff", {"wood": toon(Color(0.86, 0.72, 0.52)), "gold": staff_gold, "band": toon(SASH),
		"gem": _unshaded_mat(Color(0.6, 0.95, 1.0))}))

	# The Champion's greatsword (class GREATSWORD_CLASS), on the same pivot: a broad single-edged blade with a bright bevel, a dark fuller
	# and a chisel point, a heavy bronze guard, a long leather grip for two hands and a round pommel.
	great_parts.clear()
	var bronze := respond(toon(Color(0.78, 0.56, 0.3)), "gold")
	great_parts.append_array(_pivot_weapon("greatsword", {"metal": respond(toon(Color(0.78, 0.82, 0.9)), "metal"),
		"edge": _unshaded_mat(Color(0.95, 0.98, 1.0)), "dark": respond(toon(Color(0.42, 0.46, 0.58)), "metal"), "gold": bronze,
		"bronze": respond(toon(Color(0.5, 0.36, 0.22)), "gold"), "leather": toon(Color(0.36, 0.24, 0.2))}))

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


## (1.4 until the muted trial, 2026-10-18: a strong rim on every fighter read as glossy and striking.)
const FIGHTER_RIM_BOOST := 1.1


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
				# (None with the Display setting Fighter outline off: the match's ink pass, shaders/post.gdshader, then draws its own fine line
				# round the fighter, and the menus show it with none, as the reference figures are drawn.)
				var weight := FIGHTER_OUTLINE_WEIGHT if VisualSettings.fighter_outline() else 0.0
				(node as GeometryInstance3D).set_instance_shader_parameter("outline_weight", weight)
				# (Clothes keep the plain rim: a strong one picks out every fold as a bright streak.)
				var cloth := str(node.name).begins_with("Shirt") or str(node.name).begins_with("Shorts")
				(node as GeometryInstance3D).set_instance_shader_parameter("rim_boost", 1.0 if cloth else FIGHTER_RIM_BOOST)
				(node as GeometryInstance3D).set_instance_shader_parameter("fighter", 1.0)
				# The line art in the fighter's own colour, much darkened (see `outline_tint` in shaders/outline.gdshader).
				# (A pale body gets a darker line, so its outline still parts it from a pale sky.)
				var body_c: Color = loadout.body_color() if loadout != null else INK
				var darken := lerpf(OUTLINE_TINT_DARKEN, OUTLINE_TINT_DARKEN_PALE, smoothstep(0.55, 0.9, body_c.get_luminance()))
				var line_c: Color = body_c.darkened(darken) if loadout != null else INK
				# (A very dark body, midnight, gets a lighter line in its own hue instead: a near-black line on a near-black body and a dusk
				# backdrop left no silhouette at all.)
				if loadout != null:
					line_c = line_c.lerp(body_c.lightened(0.5), 1.0 - smoothstep(0.22, 0.3, body_c.get_luminance()))
				# (And all of it, not mixed with the ink: the slimmer limbs of the rebuilt body put more of a dark fighter's outline against a
				# dark backdrop. Art review 5.)
				var tint_a := OUTLINE_TINT
				if loadout != null:
					tint_a = lerpf(1.0, OUTLINE_TINT, smoothstep(0.22, 0.3, body_c.get_luminance()))
				(node as GeometryInstance3D).set_instance_shader_parameter("outline_tint", Color(line_c, tint_a))
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


var ghost_now := 0.0
## Invincibility's pale flicker: how strong, at most, and its colour (a cool white, so it reads as "can't be hit", not as a hit flash).
const GHOST_FLASH := 0.5
const GHOST_COLOUR := Color(0.82, 0.92, 1.0)


func _flash(s: Dictionary, delta: float) -> void:
	var target := 0.0
	var colour := Color.WHITE
	if ghost_now > 0.0:
		colour = GHOST_COLOUR
		target = ghost_now * GHOST_FLASH
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
## How far down the drop shadow reaches (a full jump and more), and the share of that depth over which it fades at the far end: it stays
## readable through a jump, so where a fighter will land is always marked, and fades only well above a platform's reach.
const SHADOW_DEPTH := 14.0
const SHADOW_FADE := 0.22
const SHADOW_DEPTH_GROUNDED := 2.4


## Puts every mesh under `node` on render layer 2 (seen and lit as usual), so the contact shadow, which only projects on layer 1, skips them.
func _on_fighter_layer(node: Node) -> void:
	for g in node.find_children("*", "GeometryInstance3D", true, false):
		(g as GeometryInstance3D).layers = 2


## The fighters on bright floors (fidelity review 16 item 1, the owner's pick A): the contact shadow firmer (0.8 from 0.64) and 15% wider;
## StageBake turns the meadow's and grove's lawns cooler (`LAWN_COOL`). `--fighter-floor=T` brings back the shadow as before.
static func _fighter_floor() -> String:
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--fighter-floor="):
			return arg.substr(16).to_upper()
	return "A"


func _contact_shadow() -> void:
	if _shadow_tex == null:
		# (A crisp, even ellipse in a deep cool shade at half strength, the judge's round 39: near-black at 0.85 with a long soft tail, it
		# read as a scorch mark on the pale deck, ragged where the planks step; mixed at half it reads as a deeper shade of whatever it
		# falls on, as the reference's shadows do, and its clean edge marks the feet.)
		var g := Gradient.new()
		# (A little fuller, round 40: the fighters on the light lawns needed a firmer footing.)
		# (Firmer again, fidelity review 16 item 1, the owner's pick A: on the bright floors the fighters lost their footing.)
		var a := 0.8 if _fighter_floor() != "T" else 0.64
		g.set_color(0, Color(0.14, 0.08, 0.2, a))
		g.set_color(1, Color(0.14, 0.08, 0.2, 0.0))
		g.add_point(0.68, Color(0.14, 0.08, 0.2, a * 0.94))
		g.add_point(0.86, Color(0.14, 0.08, 0.2, 0.1))
		_shadow_tex = GradientTexture2D.new()
		_shadow_tex.gradient = g
		_shadow_tex.fill = GradientTexture2D.FILL_RADIAL
		_shadow_tex.fill_from = Vector2(0.5, 0.5)
		_shadow_tex.fill_to = Vector2(1.0, 0.5)
		_shadow_tex.width = 64
		_shadow_tex.height = 64
	contact_shadow = Decal.new()
	contact_shadow.texture_albedo = _shadow_tex
	contact_shadow.size = Vector3(3.0, SHADOW_DEPTH, 2.6) * (Vector3(1.15, 1.0, 1.15) if _fighter_floor() != "T" else Vector3.ONE)
	# Only on surfaces facing up (not down the front of the stage).
	contact_shadow.normal_fade = 0.6
	# (Nudged toward the camera so it shows in front of the feet, which would otherwise hide it from the low match camera.)
	contact_shadow.position = Vector3(0, 0.8 - SHADOW_DEPTH / 2.0, 0.45)
	# Only the stage takes it: the fighter's own meshes are on layer 2 (see `build`).
	contact_shadow.cull_mask = 1
	contact_shadow.upper_fade = 0.0  # (no fade toward the feet; the floor is at the top of the box)
	contact_shadow.lower_fade = SHADOW_FADE
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
			return toon(Color(1, 1, 1), true, SvgArt.texture("res://art/cloth/hibiscus.svg", {"#ff00ff": accent}), Vector2(2.4, 1.2))
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
			return toon(Color(1, 1, 1), true, SvgArt.texture("res://art/cloth/hibiscus.svg", {"#ff00ff": accent}), Vector2(2.4, 2.4))
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
				now = {"width": 0.0022, "min_width": 0.028, "max_width": 0.2}[w]
			_cloth_outline.set_shader_parameter(w, float(now) * CLOTH_OUTLINE_THIN)
	var c: ShaderMaterial = (m as ShaderMaterial).duplicate()
	c.shader = TOON_CLOTH_SHADER
	# A softer step from light to shadow than the body's: on cloth the hard one picked out every small bump as a row of teeth.
	c.set_shader_parameter("ramp_softness", CLOTH_RAMP_SOFTNESS)
	# A woven texture painted into the cloth (art review 6: PEAK's garments carry a fine printed weave; ours were flat fills), faint
	# strokes along the garment from `brush.gdshaderinc`, laid by the cloth's rest coordinates so it moves with it.
	c.set_shader_parameter("brush_strength", CLOTH_WEAVE)
	c.set_shader_parameter("brush_scale", CLOTH_WEAVE_SCALE)
	c.next_pass = _cloth_outline
	_cloth_cache[m] = c
	return c


## The sash's badges: how many, where along the band (degrees round it), how much bigger than built, and how much of their enamel colour
## is muted toward grey (0.45 until the badge pass, 2026-10-19: pale, they read as rings, not badges).
const SASH_BADGES := 4
const SASH_BADGE_FROM := 44.0
const SASH_BADGE_STEP := 31.0
const SASH_BADGE_SIZE := 1.2
const SASH_BADGE_MUTE := 0.25
const CLOTH_OUTLINE_PUSH_BACK := 2.2
const CLOTH_OUTLINE_THIN := 0.65
## How much the rim light flares with a hit's flash (times its usual strength at a full flash).
const HIT_RIM_FLARE := 2.5
## How much of its own colour a fighter's outline takes, and how dark that colour is made first.
const OUTLINE_TINT := 0.9
## The fighters' line weight against the scenery's 0.6 (outline.gdshader): still the strongest line on screen, but a thinner, warmer line
## like the concept art's, not a heavy black one.
## (Thinner and more of the fighter's own colour since the muted trial: the reference figures have no ink line at all.)
const FIGHTER_OUTLINE_WEIGHT := 0.62
const OUTLINE_TINT_DARKEN := 0.45
const OUTLINE_TINT_DARKEN_PALE := 0.8
const CLOTH_RAMP_SOFTNESS := 0.32
## (Finer and fainter, 2026-10-18: long strokes on a pale shirt read as wood grain; the reference's cloth is a fine fleck.)
const CLOTH_WEAVE := 0.0   # (a fine faint cotton weave since the weave pass, 2026-10-19: 0.32 at scale 0.12 drew strokes 18 cm apart that read as ripples across the cloth; 0.12 until the finish pass, 2026-10-20, see SKIN_GRAIN)
const CLOTH_WEAVE_SCALE := 1.2
## The shirt's bottom hem: how far below the waist it is cut (art/blender/make_rigged_blob.py) and how wide its ink line is (rest units).
const SHIRT_HEM_BELOW_WAIST := 0.012
const SHIRT_HEM_INK := 0.01
const BELT_LEATHER := Color(0.4, 0.26, 0.18)
## (Darker, the character judge's round 1: the reference's shoes are a small dark leather.)
const BOOT_LEATHER := Color(0.28, 0.17, 0.13)
## (Dark, a step under the leather, the character judge's round 3: pale soles read as a second outline under each foot.)
## (A step lighter than the leather again, a lit edge round the foot, the character judge's round 11; pale, it read as a second outline.)
const BOOT_SOLE := Color(0.46, 0.36, 0.3)
const BELT_BRASS := Color(0.95, 0.74, 0.32)


## The collar's and placket's colour: the shirt's own a shade darker when it is plain, white on a print.
func _collar_colour() -> Color:
	match loadout.shirt:
		1:
			# (The outfit colour on a white shirt, as the reference scout's green collar, the aviator scout pilot.)
			return loadout.accent_color()
		2:
			# (A clear step darker, so the collar reads on the shirt, the character judge's round 6.)
			return loadout.accent_color().darkened(0.28)
	return SHIRT_WHITE


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
	m.set_shader_parameter("skin_grain", SKIN_GRAIN)
	# The hat's line on the head for its soft shade (shaders/skin.gdshader `hat_z`): a share of the head's radius above its middle, per hat.
	var hb: Array = _base_rig_info().get("head_ball", [])
	var hat_line: float = {1: 0.5, 2: 0.32, 3: 0.48, 10: 0.62}.get(loadout.hat, 99.0)
	if hb.size() == 3 and hat_line < 90.0:
		m.set_shader_parameter("hat_z", float(hb[0]) + float(hb[2]) * hat_line)
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
			m.set_shader_parameter("shirt_tex", SvgArt.texture("res://art/cloth/hibiscus.svg", {"#ff00ff": accent}))
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
	# (The head's ball in the rest pose, for the chin line: height, depth, radius.)
	var ball: Array = _base_rig_info().get("head_ball", [])
	if ball.size() == 3:
		m.set_shader_parameter("head_ball", Vector3(float(ball[0]), float(ball[1]), float(ball[2])))
	if regions.has("torso_half_width"):
		m.set_shader_parameter("torso_half_width", float(regions.torso_half_width))
	for name in ["sleeve", "arm_radius"]:
		if regions.has(name):
			m.set_shader_parameter(name, float(regions[name]))
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


## A flat-coloured material that ignores the light (a weapon's lit edge, a staff's glowing crystal), cached by colour.
static func _unshaded_mat(c: Color) -> StandardMaterial3D:
	var key := c.to_html()
	if _unshaded_cache.has(key):
		return _unshaded_cache[key]
	var m := StandardMaterial3D.new()
	m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	m.albedo_color = c
	_unshaded_cache[key] = m
	return m


static var _unshaded_cache := {}


## How each kind of stuff on a fighter takes the light (shaders/toon_light.gdshaderinc), so skin, cloth, rubber and metal read apart while
## sharing one cel look: [parameter: value].
const RESPONSES := {
	# (Skin is matte and soft-stepped, as the concept art's bodies: flat colour that turns gently into shade, no glossy bands.)
	# (Matte again, 2026-10-18: the glossy highlight read as shiny plastic beside the reference's matte figures.)
	"skin": {"ramp_softness": 0.16, "sheen": 0.04},
	"rubber": {"ramp_softness": 0.12, "sheen": 0.16},
	"metal": {"ramp_softness": 0.08, "sheen": 0.0, "metal": 1.0, "rim_strength": 0.55},
	"gold": {"ramp_softness": 0.1, "sheen": 0.0, "metal": 0.75, "rim_strength": 0.5},
	# Painted metal (car bodies, a rooftop unit): a crisp step, a broad glossy sheen and a little of the metal band, so it reads as
	# enamel over steel rather than bare metal or matte wood.
	"paint": {"ramp_softness": 0.08, "sheen": 0.4, "metal": 0.3, "rim_strength": 0.45},
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
		_shirt_collar()
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


## How much wider the sash is drawn over a shirt (the shirt stands off the skin, make_rigged_blob.py SHIRT_LIFT).
const SASH_OVER_SHIRT := 1.36   # (1.2 until the badge pass, 2026-10-19: the puffed shirt broke through the band in white wedges)


## Neckwear for the rigged torso (TORSO_CENTRE_Y, TORSO_RADII). Rings are tori sized to hug it, so nothing pokes through the body whatever
## the pose.
func _neck_on_rig(accent: Color) -> void:
	# (The kerchief and sash together, neck 5: the reference's sailor wears both, the character judge's round 5.)
	for piece in ([2, 1] if loadout.neck == 5 else [loadout.neck]):
		_neck_piece(accent, piece)
	if loadout.neck == 5 and loadout.shirt > 0:
		_sailor_collar(accent)
	if loadout.neck == 1 and loadout.shirt == 2:
		_kerchief_knot(Color(0.7, 0.16, 0.16))


## A small neckerchief knotted at the throat over the pointed collar (the ranger, after the reference, the character judge's round 6): a
## knot and two short tails, one mesh.
func _kerchief_knot(c: Color) -> void:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var top := TORSO_CENTRE_Y + TORSO_RADII.y * 0.86
	var z := TORSO_RADII.z * 1.12
	var knot := BoxMesh.new()
	knot.size = Vector3(0.1, 0.08, 0.06)
	# (On the open side of the chest, clear of the sash across the other.)
	# (At the collar's centre, high at the throat above where the sash crosses, the character judge's round 7: on the chest the round
	# knot read as a rose.)
	var off := Vector3(0.0, 0.06, -0.02)
	st.append_from(knot, 0, Transform3D(Basis.IDENTITY, Vector3(0, top - 0.05, z) + off))
	for sx in [-1.0, 1.0]:
		var tail := PrismMesh.new()
		tail.size = Vector3(0.1, 0.14, 0.025)
		# (Short, splayed outward and lying on the shirt, the character judge's round 8: long, one read as a necktie and stood off the chest.)
		st.append_from(tail, 0, Transform3D(Basis.from_euler(Vector3(-0.35, 0, PI + sx * 0.7)), Vector3(sx * 0.07, top - 0.12, z - 0.03) + off))
	_part(torso_rig, st.commit(), _fabric(c), Vector3.ZERO)


## The sailor's square collar (the character judge's round 5, after the owner's reference): a broad flap hanging square down the back and
## running forward over the shoulders to the kerchief's knot, in the outfit colour with two white stripes along its edge. One mesh, two
## surfaces (the cloth and the stripes), for the loadout's mesh budget.
func _sailor_collar(accent: Color) -> void:
	var cloth := SurfaceTool.new()
	cloth.begin(Mesh.PRIMITIVE_TRIANGLES)
	var stripes := SurfaceTool.new()
	stripes.begin(Mesh.PRIMITIVE_TRIANGLES)
	var top := TORSO_CENTRE_Y + TORSO_RADII.y * 0.82
	var add := func(st: SurfaceTool, size: Vector3, at: Vector3, rot_deg: Vector3) -> void:
		var b := BoxMesh.new()
		b.size = size
		st.append_from(b, 0, Transform3D(Basis.from_euler(rot_deg * PI / 180.0), at))
	# The back flap, square, leaning back with the shoulders.
	# (Out past the loose shirt, which stands off the body.)
	var back_z := -TORSO_RADII.z * 1.1
	# (In two strips bending with the back, shorter, the character judge's round 7: one flat plate stood off as a fin.)
	add.call(cloth, Vector3(0.8, 0.2, 0.05), Vector3(0, top - 0.06, back_z + 0.1), Vector3(52, 0, 0))
	add.call(cloth, Vector3(0.78, 0.22, 0.05), Vector3(0, top - 0.24, back_z + 0.02), Vector3(22, 0, 0))
	for k in 2:
		add.call(stripes, Vector3(0.7 - k * 0.1, 0.026, 0.034), Vector3(0, top - 0.32 + k * 0.05, back_z - 0.0 + k * 0.018), Vector3(22, 0, 0))
	# Over each shoulder and down to the knot in front.
	for sx in [-1.0, 1.0]:
		# (Lying flat over the shoulders only, no lapels down the front, the character judge's round 6: the lapels, the kerchief and the
		# sash met in one cluttered knot.)
		add.call(cloth, Vector3(0.2, 0.6, 0.03), Vector3(sx * 0.42, top + 0.06, -0.08), Vector3(90, 0, sx * 14.0))
	var mesh := cloth.commit()
	stripes.commit(mesh)
	var mi := _part(torso_rig, mesh, null, Vector3.ZERO)
	mi.set_surface_override_material(0, _fabric(accent))
	mi.set_surface_override_material(1, _fabric(Color(0.97, 0.96, 0.92)))


func _neck_piece(accent: Color, kind: int) -> void:
	var centre := Vector3(0, TORSO_CENTRE_Y, 0)
	match kind:
		1:
			# (A raised collar band round the neck in the shirt's collar colour, the character judge's round 1 after the owner's reference: flat
			# collar points on the chest left a hard cut where the head meets the shirt.)
			if loadout.shirt > 0:
				# (Snug round the neck with two small points turned down at the front, the character judge's round 2: wide, it read as a bar.)
				# (Narrower and raised to meet the head, the points in toward the middle, the character judge's round 3: a bar across the chest.)
				var cat := _collar(_collar_colour(), 0.74, 0.12)
				var point := PrismMesh.new()
				point.size = Vector3(0.13, 0.12, 0.04)
				for sx in [-1.0, 1.0]:
					_part(torso_rig, point, _fabric(_collar_colour()), Vector3(sx * 0.07, cat - 0.04, 0.42), Vector3.ONE, Vector3(-10, 0, 180.0 - sx * 22.0))
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
			# (Over the shirt, which since the body rebuild hangs looser: fitted to the bare torso, the band sank into the cloth in pieces.)
			var over := SASH_OVER_SHIRT if loadout.shirt > 0 else 1.0
			# (Twice as wide, the character judge's round 1: the reference's sash is a broad band.)
			# (Deeper too, the cap pass, 2026-10-19: at 0.45 of the widening the puffed shirt showed through the band on the back.)
			sash.scale = Vector3(over, 1.9, 1.0 + (over - 1.0) * 0.8)
			var strap := _gear_mesh("neckwear", "fabric_sash")
			if strap != null:
				_part(sash, strap, _cloth(toon(SASH, false)), Vector3.ZERO)
			else:
				_part(sash, ring, toon(SASH, false), Vector3.ZERO, Vector3(0.88, 1.7, 1.0))
			# Six small square enamel badges along the band, facing out from it (the aviator scout pilot, 2026-10-19, after the owner's
			# reference: three big plain dots read as buttons): each a brass plate with a coloured enamel face and a pale mark on it.
			# (Smaller, spaced apart and of different shapes, each with its own little mark, the character judge's round 1: six bright
			# squares in a tight row read as stickers.)
			var square := BoxMesh.new()
			# (A fifth smaller, thinner rims and muted, the character judge's round 2: the brightest things on the fighter.)
			square.size = Vector3(0.061, 0.02, 0.061)
			var round_plate := CylinderMesh.new()
			round_plate.top_radius = 0.037
			round_plate.bottom_radius = 0.037
			round_plate.height = 0.024
			round_plate.radial_segments = 14
			var plates := [square, round_plate, square, round_plate, square, round_plate]
			var sq_face := BoxMesh.new()
			sq_face.size = Vector3(0.052, 0.026, 0.052)
			var rd_face := CylinderMesh.new()
			rd_face.top_radius = 0.031
			rd_face.bottom_radius = 0.031
			rd_face.height = 0.03
			rd_face.radial_segments = 14
			var faces := [sq_face, rd_face, sq_face, rd_face, sq_face, rd_face]
			var dot := CylinderMesh.new()
			dot.top_radius = 0.014
			dot.bottom_radius = 0.014
			dot.height = 0.036
			dot.radial_segments = 8
			var bar := BoxMesh.new()
			bar.size = Vector3(0.04, 0.036, 0.012)
			var tri := PrismMesh.new()
			tri.size = Vector3(0.036, 0.036, 0.03)
			var marks := [tri, dot, bar, dot, tri, bar]
			var brass := respond(toon(Color(0.86, 0.68, 0.32)), "gold")
			var colours := [Color(0.92, 0.32, 0.28), accent, Color(0.3, 0.52, 0.86), Color(0.98, 0.8, 0.28), Color(0.4, 0.7, 0.42),
				Color(0.62, 0.42, 0.8)]
			# (Four, spread along the strap from the shoulder to the hip, a little bigger, since the badge pass, 2026-10-19: six packed into the
			# middle of the chest every 13.5 degrees merged into one cluster of rings at any distance; the reference's sit apart.)
			# (All in one mesh, a surface per material, so the sash keeps to the loadout's mesh budget.)
			var tools := {}
			var order: Array = []
			for k in SASH_BADGES:
				var t := deg_to_rad(SASH_BADGE_FROM + SASH_BADGE_STEP * k)
				var out := Vector3(0.88 * cos(t), 0.0, sin(t)).normalized()
				var at_b := Vector3(0.88 * 0.655 * cos(t), 0.0, 0.655 * sin(t))
				var turn := Basis(Quaternion(Vector3.UP, out)) * Basis(Vector3.UP, deg_to_rad(8.0 * float(k % 3 - 1))) * Basis.from_scale(Vector3.ONE * SASH_BADGE_SIZE)
				for piece in [[plates[k], "brass"], [faces[k], "face%d" % k], [marks[k], "mark"]]:
					var key: String = piece[1]
					if not tools.has(key):
						var st := SurfaceTool.new()
						st.begin(Mesh.PRIMITIVE_TRIANGLES)
						tools[key] = st
						order.append(key)
					(tools[key] as SurfaceTool).append_from(piece[0], 0, Transform3D(turn, at_b))
			var badges := ArrayMesh.new()
			for key in order:
				(tools[key] as SurfaceTool).commit(badges)
			var bm := _part(sash, badges, null, Vector3.ZERO)
			for i in order.size():
				var key: String = order[i]
				var c_b: Color = colours[int(key.substr(4))] if key.begins_with("face") else Color.WHITE
				var m: Material = brass if key == "brass" else (toon(Color(0.98, 0.95, 0.88)) if key == "mark" else respond(toon(c_b.lerp(Color(0.55, 0.5, 0.5), SASH_BADGE_MUTE)), "rubber"))
				bm.set_surface_override_material(i, m)
			# The sash's knotted end hanging at the hip, swinging with the body (`TailRig`).
			# (No knotted tail since the character judge's round 1: the reference's sash ends at the hip.)
		2:
			# Neckerchief: a collar ring round the top of the torso with a point hanging in front.
			# (Snug round the neck and up to the head, the character judge's round 4: wide, it read as a bar across the chest.)
			var at := _collar(accent, 0.74, 0.12)
			# (A folded triangle of cloth pointing down, 2026-10-18: a square turned on its corner read as a flat plastic diamond.)
			var bib := PrismMesh.new()
			bib.size = Vector3(0.46, 0.34, 0.06)
			# (On a pivot at the knot, so it swings with the body: the `neck` spring.)
			neck_pivot = Node3D.new()
			neck_pivot.position = Vector3(0, at - 0.02, 0.5)
			torso_rig.add_child(neck_pivot)
			var kerchief := _gear_mesh("neckwear", "fabric_kerchief")
			var knot := _gear_mesh("neckwear", "fabric_knot")
			if kerchief != null and knot != null:
				var out := Vector3(0, 0, 0.1 if loadout.shirt > 0 else 0.03)
				_part(neck_pivot, kerchief, _fabric(accent), out, Vector3.ONE, Vector3(-14, 0, 0))
				_part(neck_pivot, knot, _fabric(accent.darkened(0.08)), out)
			else:
				_part(neck_pivot, bib, _fabric(accent), Vector3(0, -0.15, 0), Vector3.ONE, Vector3(-8, 0, 180))
		3:
			# Scarf: the collar ring and a tail hanging down the front on one side.
			# (Up at the neck over the shirt's neckline and fuller, the scarf pass, 2026-10-20: at the plain collar's height it sank inside
			# the puffed shirt and showed only as a thin stripe across the chest, its tail hidden inside the shirt.)
			var at := _collar(accent, SCARF_SNUG, SCARF_LIFT, SCARF_THICK)
			# (A chain of pieces on springs, so it swings and whips with the body: `TailRig`.)
			_tail(Vector3(0.24, at - 0.06, 0.62), _fabric(accent), 0.52, 0.22, 0.07, 4)
		4:
			# Cape: a collar ring and a broad cloth hanging down the back to the knees, on springs, so it streams behind the fighter's runs
			# and jumps: the biggest change to the outline the wardrobe has.
			var at := _collar(accent)
			_tail(Vector3(0, at - 0.02, -0.42), _fabric(accent.darkened(0.08)), 1.15, 0.92, 0.05, 5)


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


## A shirt's collar on the rigged torso (the owner's PEAK references, 2026-10-14: their shirts have a collar, a button placket and banded
## sleeves; ours were plain tubes of colour): two pointed flaps folded down under the chin, a placket with three buttons down the front.
## Only with a shirt and no ring of neckwear (which covers the neck already).
func _shirt_collar() -> void:
	if not use_base_body or loadout.shirt <= 0 or loadout.neck >= 2:
		return
	if rig != null and rig.find_child("ShirtCollar", true, false) != null:
		return
	var info := _base_rig_info()
	var at := 1.1
	if info.has("neck") and torso_fit.basis.y.y > 0.001:
		at = (float(info.neck) - 0.05 - torso_fit.origin.y) / torso_fit.basis.y.y
	var k := clampf((at - TORSO_CENTRE_Y) / TORSO_RADII.y, -0.99, 0.99)
	var front := TORSO_RADII.z * sqrt(1.0 - k * k)
	# (Was two white boxes turned into diamonds, which met under the chin and read as a bow tie. Now two pointed flaps in the shirt's own
	# cloth, apart at the throat in a V, each lying down and out over the chest: the collar of PEAK's scout shirts.)
	var flap_col: Color
	match loadout.shirt:
		1:
			flap_col = SHIRT_WHITE.darkened(0.05)
		2:
			flap_col = loadout.accent_color().darkened(0.1)
		_:
			flap_col = SHIRT_WHITE
	var flap := PrismMesh.new()
	flap.size = Vector3(0.17, 0.2, 0.035)
	for sx in [-1.0, 1.0]:
		var fy := at - 0.1
		var fk := clampf((fy - TORSO_CENTRE_Y) / TORSO_RADII.y, -0.99, 0.99)
		var fx: float = sx * 0.12
		var flap_z := TORSO_RADII.z * sqrt(1.0 - fk * fk) * cos(asin(clampf(fx / TORSO_RADII.x, -0.9, 0.9))) + 0.05
		# Upside down (pointing at the ground), leaned out from the throat and turned to lie on the curve of the chest.
		_part(torso_rig, flap, toon(flap_col), Vector3(fx, fy, flap_z), Vector3.ONE, Vector3(-18, sx * 22.0, 180.0 - sx * 28.0))
	var placket := BoxMesh.new()
	placket.size = Vector3(0.07, 0.5, 0.03)
	var below := clampf((at - 0.36 - TORSO_CENTRE_Y) / TORSO_RADII.y, -0.99, 0.99)
	var front_low := TORSO_RADII.z * sqrt(1.0 - below * below)
	_part(torso_rig, placket, toon(flap_col.darkened(0.08)), Vector3(0, at - 0.36, (front + front_low) * 0.5 + 0.035), Vector3.ONE,
		Vector3(-rad_to_deg(atan2(front - front_low, 0.5)) * 0.6, 0, 0))
	for j in 3:
		var y := at - 0.2 - 0.14 * j
		var kk := clampf((y - TORSO_CENTRE_Y) / TORSO_RADII.y, -0.99, 0.99)
		_part(torso_rig, _sphere(0.035), toon(Color(0.95, 0.9, 0.78)), Vector3(0, y, TORSO_RADII.z * sqrt(1.0 - kk * kk) + 0.06), Vector3(1, 1, 0.6))


## The collar ring of the neckerchief and the scarf, just under the chin (where the head meets the shoulders, from base_rig.json's `neck`).
## Returns its height in the torso's design space, for what hangs from it.
## The hats modelled in Blender (art/blender/make_gear.py, godot/models/gear), by hat slot (the owner, 2026-10-18: moving the
## accessories from built primitives to models over time).
const GEAR_HATS := {1: "sailor_cap", 2: "aviator_cap", 3: "straw_hat", 4: "beanie", 5: "crown", 7: "plumed_helm", 8: "swept_spikes",
	9: "topknot", 10: "dixie_cap"}


## Puts the modelled accessory `name` on the hat pivot, each part coloured by what it is made of (its name's first word): fabric in
## `accent`, canvas white cloth, leather, metal (brass), glass, straw. False if the model is missing.
func _gear_model(name: String, accent: Color) -> bool:
	var path := "res://models/gear/%s.glb" % name
	if not ResourceLoader.exists(path):
		return false
	var scene := load(path) as PackedScene
	if scene == null:
		return false
	var model := scene.instantiate() as Node3D
	model.position = -HAT_PIVOT
	hat_pivot.add_child(model)
	# (A fine low-contrast weave, the character judge's round 6: coarse, it read as a picnic cloth.)
	var straw := toon(Color(1, 1, 1), true, SvgArt.texture("res://art/cloth/straw_fine.svg", {"#ff00ff": Color(0.92, 0.8, 0.48)}), Vector2(9, 9))
	var brass := (respond(toon(Color(0.78, 0.64, 0.38)), "gold") as ShaderMaterial).duplicate() as ShaderMaterial
	brass.set_shader_parameter("metal", 0.3)
	brass.set_shader_parameter("rim_strength", 0.2)
	# (A deep blued steel: a pale one vanished against light backgrounds.)
	var steel := (respond(toon(Color(0.4, 0.43, 0.52)), "metal") as ShaderMaterial).duplicate() as ShaderMaterial
	steel.set_shader_parameter("metal", 0.3)
	steel.set_shader_parameter("rim_strength", 0.15)
	var gold := (respond(toon(Color(0.86, 0.66, 0.24)), "gold") as ShaderMaterial).duplicate() as ShaderMaterial
	gold.set_shader_parameter("metal", 0.3)
	gold.set_shader_parameter("rim_strength", 0.2)
	# (A model's "metal" is the helm's steel, or the caps' brass; "brass" and "gold" name themselves.)
	var mats := {"fabric": _fabric(accent), "canvas": _cloth(toon(Color(0.95, 0.94, 0.9))), "leather": _cloth(toon(Color(0.4, 0.25, 0.17))),
		"metal": steel if name == "plumed_helm" else brass, "brass": brass, "gold": gold, "glass": toon(Color(0.66, 0.84, 0.95), false),
		"straw": straw, "hair": toon(FaceLook.BROW_COLORS[clampi(loadout.brow_color, 0, FaceLook.BROW_COLORS.size() - 1)]),
		# (Cream fleece, the aviator scout pilot, 2026-10-19.)
		# (A soft curly painted tile, the character judge's round 1: the fleece read as a smooth ring.)
		"navy": _cloth(toon(Color(0.2, 0.28, 0.58))),
		"fleece": toon(Color(1, 1, 1), true, SvgArt.texture("res://art/cloth/fleece.svg", {"#ff00ff": Color(0.93, 0.86, 0.72)}), Vector2(1, 2))}
	# One mesh per material, its parts merged (the loadout budget, docs plan 7.3: the crown's set stones and the helm's plume feathers, a
	# mesh each, came to 16 and 24 meshes on one hat).
	var groups := {}
	var order: Array = []
	for mi: MeshInstance3D in model.find_children("*", "MeshInstance3D", true, false):
		var kind := String(mi.name).get_slice("_", 0)
		if not mats.has(kind):
			kind = "_accent"
		if not groups.has(kind):
			groups[kind] = []
			order.append(kind)
		groups[kind].append(mi)
	for kind in order:
		var st := SurfaceTool.new()
		for mi: MeshInstance3D in groups[kind]:
			var xf := Transform3D.IDENTITY
			var up: Node = mi
			while up != null and up != model:
				xf = (up as Node3D).transform * xf
				up = up.get_parent()
			for surface in mi.mesh.get_surface_count():
				st.append_from(mi.mesh, surface, xf)
		var one := MeshInstance3D.new()
		one.name = String(kind)
		one.mesh = st.commit()
		one.material_override = mats.get(kind, toon(accent))
		model.add_child(one)
		for mi: MeshInstance3D in groups[kind]:
			mi.get_parent().remove_child(mi)
			mi.queue_free()
		meshes.append(one)
	return true


## A fabric accessory's material (hats, neckwear, bands): the cloth's softer step and fine weave (`_cloth`), as the shirts and shorts
## have (2026-10-18, the owner after the reference cast: the accessories in flat plastic read cheaper than the clothes).
func _fabric(c: Color) -> Material:
	return _cloth(toon(c))


func _collar(accent: Color, snug := 1.0, lift := 0.0, thick := 1.0) -> float:
	var ring := TorusMesh.new()
	ring.inner_radius = 0.44
	ring.outer_radius = 0.58
	ring.rings = 36
	ring.ring_segments = 10
	var at := 1.1
	var info := _base_rig_info()
	if info.has("neck") and torso_fit.basis.y.y > 0.001:
		at = (float(info.neck) - COLLAR_BELOW_NECK + lift - torso_fit.origin.y) / torso_fit.basis.y.y
	# (A flatter band of cloth rather than a round tube, 2026-10-18: the tube read as a plastic ring.)
	# (The modelled collar, art/blender/make_gear.py `neckwear`, where there is one: a rolled band of cloth gathered in folds.)
	var modelled := _gear_mesh("neckwear", "fabric_collar")
	if modelled != null:
		# (A size out over a shirt, which stands off the body since the body rebuild.)
		var over := (1.14 if loadout.shirt > 0 else 1.0) * snug
		_part(torso_rig, modelled, _fabric(accent), Vector3(0, at, 0), Vector3(over, thick, over))
	else:
		_part(torso_rig, ring, _fabric(accent), Vector3(0, at, 0), Vector3(1.06, 0.62, 0.96))
	return at


## How far below the neck (where the head meets the shoulders) the collar ring sits: its top just shows under the chin.
const COLLAR_BELOW_NECK := 0.1
## The scarf's collar: how snug, how far up and how much thicker than the plain collar ring (the scarf pass, 2026-10-20).
const SCARF_SNUG := 0.98
const SCARF_LIFT := 0.1
const SCARF_THICK := 1.6


## The hat sits on a pivot at the crown of the head that lags behind sudden moves and bounces on landings (the `hat_roll` and `hat_drop` springs, see `_secondary`).
const HAT_PIVOT := Vector3(0, 1.95, 0)
## How much bigger than modelled every hat is drawn (`_hat`).
const HAT_SIZE := 1.22
var hat_pivot: Node3D


func _hat() -> void:
	hat_pivot = Node3D.new()
	hat_pivot.position = HAT_PIVOT
	# (Oversized, as the reference's hats are, art review 10: at play size the hats read as small caps and every fighter as the same
	# round head; a bigger hat is most of a fighter's silhouette. Scaled about the top of the head, so each still sits on it.)
	# (Only the hats that sit on top of the head: scaled, the caps that wrap it (aviator cap, beanie, hood, topknot) came down over the eyes.)
	if loadout.hat in [1, 3, 5, 7, 8]:
		# (Only part of the head shape's stretch reaches the hat, the character judge's round 14: on the wide bun head the aviator's hat
		# spread into a flat platter. The hat fits the head without taking its proportions.)
		var hs: Vector3 = Loadout.HEAD_SCALES.get(Loadout.HEADS[clampi(int(loadout.get("head")), 0, Loadout.HEADS.size() - 1)], HEAD_SIZE)
		var k := Vector3(pow(HEAD_SIZE.x / hs.x, 0.7), pow(HEAD_SIZE.y / hs.y, 0.7), pow(HEAD_SIZE.z / hs.z, 0.7))
		hat_pivot.scale = k * HAT_SIZE
	head_rig.add_child(hat_pivot)
	var accent: Color = loadout.accent_color()
	var white := toon(Color(0.97, 0.97, 1.0))
	# The modelled hats (art/blender/make_gear.py) where there is one; the built ones below stand in.
	if GEAR_HATS.has(loadout.hat) and _gear_model(GEAR_HATS[loadout.hat], accent):
		return
	match loadout.hat:
		1:
			# Sailor cap: white crown and brim with a band in the accent colour.
			var canvas := _cloth(white)
			_part(hat_pivot, _cyl(0.5, 0.55, 0.32), canvas, Vector3(0, 2.18, 0) - HAT_PIVOT)
			_part(hat_pivot, _cyl(0.62, 0.62, 0.08), canvas, Vector3(0, 2.02, 0) - HAT_PIVOT)
			_part(hat_pivot, _cyl(0.5, 0.5, 0.05), _fabric(accent), Vector3(0, 2.1, 0) - HAT_PIVOT, Vector3(1.04, 1.0, 1.04))
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
			_part(hat_pivot, _cyl(0.63, 0.63, 0.08), _fabric(accent), Vector3(0, 2.14, 0) - HAT_PIVOT)
		4:
			# Beanie with a pompom.
			_part(hat_pivot, _sphere(0.84), _fabric(accent), Vector3(0, 1.78, 0) - HAT_PIVOT, Vector3(1.0, 0.7, 1.0))
			_part(hat_pivot, _sphere(0.18), _cloth(white), Vector3(0, 2.38, 0) - HAT_PIVOT)
		6:
			# Hood: a soft shell of dark charcoal cloth (it reads against any body colour) wrapping over the head and forward round the face,
			# its edge turned back in a soft fold of a lighter shade, a cowl of the same cloth lying round the shoulders, and a point behind
			# that droops. (A shell open at the front, seen from both sides like the clothes: a closed ball drew its front across the face,
			# which showed as a disc in a dark ring; and a wide cream rim round a small opening read as a porthole.)
			# (In the outfit colour, a shade deeper, since the hood pass, 2026-10-20: near-black charcoal was the heaviest mass on the
			# fighter, a dark slab round the face; the reference's hood is the character's own coloured cloth.)
			var charcoal := accent.darkened(0.18)
			var cloth := _cloth(toon(charcoal))
			var at := Vector3(0, 1.52, 0.02) - HAT_PIVOT
			var fit := Vector3(1.1, 1.06, 1.02)
			_part(hat_pivot, _open_shell(0.9, _hood_edge), cloth, at, fit)
			_part(hat_pivot, _shell_lip(0.9, _hood_edge, 0.06), toon(charcoal.lightened(0.24)), at, fit)
			# The cowl: an open cone of the same cloth from inside the hood's lower edge flaring out over the shoulders.
			var cowl := CylinderMesh.new()
			# (Shorter and barely flared, the hood pass: a wide stiff cone read as a flat bar across the shoulders.)
			cowl.top_radius = 0.66
			cowl.bottom_radius = 0.78
			cowl.height = 0.3
			cowl.radial_segments = 28
			cowl.rings = 2
			cowl.cap_top = false
			cowl.cap_bottom = false
			_part(hat_pivot, cowl, cloth, Vector3(0, 0.68, -0.08) - HAT_PIVOT, Vector3(1.0, 1.0, 0.9), Vector3(10, 0, 0))
			_part(hat_pivot, _cyl(0.0, 0.3, 0.8), cloth, Vector3(0, 1.95, -0.72) - HAT_PIVOT, Vector3.ONE, Vector3(-62, 0, 0))
		7:
			# Plumed helm: a steel cap with a brow band and a tall plume in the accent colour sweeping back off the crest.
			# (A darker steel with a quieter shine, 2026-10-18: pale steel at full metal read as a white glare.)
			var steel := (respond(toon(Color(0.58, 0.6, 0.68)), "metal") as ShaderMaterial).duplicate() as ShaderMaterial
			steel.set_shader_parameter("metal", 0.22)
			steel.set_shader_parameter("rim_strength", 0.2)
			_part(hat_pivot, _sphere(0.86), steel, Vector3(0, 1.76, -0.04) - HAT_PIVOT, Vector3(1.0, 0.7, 1.0))
			_part(hat_pivot, _cyl(0.84, 0.84, 0.1), respond(toon(Color(0.95, 0.78, 0.32)), "gold"), Vector3(0, 1.88, -0.04) - HAT_PIVOT, Vector3(1.0, 1.0, 1.02))
			var plume := _fabric(accent)
			for k in 5:
				var f := float(k) / 4.0
				_part(hat_pivot, _sphere(1.0), plume, Vector3(0, 2.42 + 0.2 * sin(f * PI) - 0.1 * f, -0.05 - 0.95 * f) - HAT_PIVOT,
					Vector3(0.16, 0.48 - 0.12 * f, 0.4), Vector3(-25 - 45 * f, 0, 0))
		8:
			# Swept spikes: a cap of hair over the crown with thick, blunt spikes swept up and back, in the brow colour (a hairstyle, worn in
			# the hat slot).
			var hair := toon(FaceLook.BROW_COLORS[clampi(loadout.brow_color, 0, FaceLook.BROW_COLORS.size() - 1)])
			_part(hat_pivot, _sphere(0.86), hair, Vector3(0, 1.74, -0.08) - HAT_PIVOT, Vector3(1.02, 0.62, 1.0))
			# [x, y, z, sideways lean, length]: a fan across the crown and a few swept back, each a thick cone leaning up and back.
			var spikes := [[-0.55, 2.1, 0.15, 48.0, 1.0], [-0.22, 2.3, 0.1, 20.0, 1.2], [0.18, 2.32, 0.08, -14.0, 1.2], [0.52, 2.12, 0.12, -44.0, 1.0],
				[-0.35, 2.2, -0.35, 32.0, 1.05], [0.3, 2.2, -0.38, -28.0, 1.05], [0.0, 2.05, -0.62, 0.0, 0.95]]
			for sp in spikes:
				_part(hat_pivot, _cyl(0.0, 0.3, float(sp[4])), hair, Vector3(float(sp[0]), float(sp[1]), float(sp[2])) - HAT_PIVOT, Vector3.ONE,
					Vector3(-34, 0, float(sp[3])))
		9:
			# Topknot: close-cropped hair over the crown and a round knot of hair on top, tied with a band in the accent colour.
			var hair := toon(FaceLook.BROW_COLORS[clampi(loadout.brow_color, 0, FaceLook.BROW_COLORS.size() - 1)])
			_part(hat_pivot, _sphere(0.86), hair, Vector3(0, 1.72, -0.1) - HAT_PIVOT, Vector3(1.02, 0.6, 1.0))
			_part(hat_pivot, _sphere(0.32), hair, Vector3(0, 2.36, -0.15) - HAT_PIVOT, Vector3(1.0, 0.9, 1.0))
			_part(hat_pivot, _cyl(0.2, 0.2, 0.1), _fabric(accent), Vector3(0, 2.12, -0.13) - HAT_PIVOT)
		5:
			# Crown: a gold band with five points.
			# (Gold as metal, 2026-10-18: flat yellow read as card.)
			var gold := (respond(toon(Color(0.86, 0.66, 0.24)), "gold") as ShaderMaterial).duplicate() as ShaderMaterial
			gold.set_shader_parameter("metal", 0.3)
			gold.set_shader_parameter("rim_strength", 0.2)
			_part(hat_pivot, _cyl(0.55, 0.58, 0.2), gold, Vector3(0, 2.1, 0) - HAT_PIVOT)
			for i in 5:
				var a := TAU * i / 5.0
				_part(hat_pivot, _cyl(0.0, 0.11, 0.32), gold, Vector3(sin(a) * 0.5, 2.36, cos(a) * 0.5) - HAT_PIVOT)


func _glasses() -> void:
	# (Dark brown wire, the aviator scout pilot, after the owner's reference.)
	var frame := toon(Color(0.3, 0.17, 0.12), false)
	# The glasses are placed in design space through their own fit (see `_glasses_fit`): a node under the head whose transform turns
	# the head's fit into it.
	var glasses_rig := Node3D.new()
	glasses_rig.transform = head_fit.affine_inverse() * glasses_fit
	head_rig.add_child(glasses_rig)
	glasses_node = glasses_rig
	glasses_base = glasses_rig.transform
	var to_head: Transform3D = glasses_rig.transform
	# The glasses' space is stretched across by the eyes' spacing and up by their size: a lens is made round again by this.
	lens_aspect = glasses_fit.basis.get_scale().y / maxf(0.001, glasses_fit.basis.get_scale().x)
	# The modelled frames (art/blender/make_gear.py `glasses`), each part at its own size, seated per lens by `_on_face`; the built ones
	# stand in without them.
	var g_rim_specs := _gear_mesh("glasses", "metal_specs_rim")
	var g_rim_shades := _gear_mesh("glasses", "frame_shades_rim")
	var g_lens := _gear_mesh("glasses", "glass_shades_lens")
	var g_cup := _gear_mesh("glasses", "leather_goggle_cup")
	var g_bridge := _gear_mesh("glasses", "frame_bridge")
	var modelled := g_rim_specs != null and g_rim_shades != null and g_lens != null and g_cup != null and g_bridge != null
	match loadout.glasses:
		1:
			# Shades: tinted glass the eyes show through, in a dark frame with a heavier brow bar, and a bridge.
			# (A light peach tint with white glare streaks, shaders/lens.gdshader, the character pass, 2026-10-19, after the reference cast's
			# shades: a heavy flat orange turned the eyes behind brown.)
			var lens := ShaderMaterial.new()
			lens.shader = preload("res://shaders/lens.gdshader")
			if modelled:
				for sx in [-1.0, 1.0]:
					_on_face(_part(glasses_rig, g_lens, lens, Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.04, Vector3.ONE)
					_on_face(_part(glasses_rig, g_rim_shades, frame, Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.05, Vector3.ONE)
				_on_face(_part(glasses_rig, g_bridge, frame, Vector3.ZERO), to_head, 0.0, 1.56, 0.06, Vector3.ONE)
			else:
				var rim := TorusMesh.new()
				rim.inner_radius = 0.95
				rim.outer_radius = 1.05
				for sx in [-1.0, 1.0]:
					_on_face(_part(glasses_rig, _sphere(1.0), lens, Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.04, Vector3(0.25, 0.18, 0.03))
					_on_face(_part(glasses_rig, rim, frame, Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.045, Vector3(0.25, 0.04, 0.18), true)
				_on_face(_part(glasses_rig, _sphere(1.0), frame, Vector3.ZERO), to_head, 0.0, 1.56, 0.06, Vector3(0.1, 0.025, 0.025))
		2:
			# Goggles: deep padded leather cups with glass, and a strap round the head.
			var leather := _cloth(toon(Color(0.45, 0.28, 0.12)))
			for sx in [-1.0, 1.0]:
				if modelled:
					_on_face(_part(glasses_rig, g_cup, leather, Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.04, Vector3.ONE)
				else:
					var ring := TorusMesh.new()
					ring.inner_radius = 0.14
					ring.outer_radius = 0.28
					_on_face(_part(glasses_rig, ring, leather, Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.0, Vector3.ONE, true)
				_on_face(_part(glasses_rig, _sphere(1.0), toon(Color(0.7, 0.9, 1.0, 1.0), false), Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.02,
					Vector3(0.14, 0.14, 0.02))
			var strap := TorusMesh.new()
			strap.inner_radius = 0.78
			strap.outer_radius = 0.84
			_part(glasses_rig, strap, _cloth(toon(Color(0.3, 0.2, 0.1))), Vector3(0, 1.52, 0), Vector3(1, 0.6, 1))
		3:
			# Round specs: thin flat-faced wire rims and a bridge.
			for sx in [-1.0, 1.0]:
				if modelled:
					# (Closer to the face, the helm pass, 2026-10-19: with the finer wire, 0.05 off it stood out from the head side-on.)
					_on_face(_part(glasses_rig, g_rim_specs, frame, Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.03, Vector3.ONE)
				else:
					var thin := TorusMesh.new()
					thin.inner_radius = 0.17
					thin.outer_radius = 0.21
					_on_face(_part(glasses_rig, thin, frame, Vector3.ZERO), to_head, sx * 0.32, 1.52, 0.0, Vector3.ONE, true)
			if modelled:
				_on_face(_part(glasses_rig, g_bridge, frame, Vector3.ZERO), to_head, 0.0, 1.54, 0.06, Vector3.ONE)
			else:
				_on_face(_part(glasses_rig, _sphere(1.0), frame, Vector3.ZERO), to_head, 0.0, 1.54, 0.02, Vector3(0.1, 0.02, 0.02))


static var _gear_meshes := {}


## One part's mesh from a modelled accessory file (godot/models/gear/<file>.glb), or null without it. Cached.
static func _gear_mesh(file: String, part: String) -> Mesh:
	var key := file + "/" + part
	if _gear_meshes.has(key):
		return _gear_meshes[key]
	var mesh: Mesh = null
	var path := "res://models/gear/%s.glb" % file
	if ResourceLoader.exists(path):
		var scene := load(path) as PackedScene
		if scene != null:
			var node := scene.instantiate()
			var mi := node.find_child(part, true, false) as MeshInstance3D
			if mi != null:
				mesh = mi.mesh
			node.free()
	_gear_meshes[key] = mesh
	return mesh


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
## Out of a tumble in the air, the share of the tumble's turn left after each sim frame (round 7): upright in about six frames.
const TUMBLE_SETTLE := 0.55
## Flying faster than this (world units a frame), a launched fighter is thrown head first along its path rather than tumbling.
const LAUNCH_POSE_SPEED := 0.32
## The hurt recoil (round 3 item 3): the whole body snaps away from the hit over HURT_SNAP_FRAMES, holds a moment, then recovers to
## upright by HURT_RECOVERED of the hitstun, by as much as HURT_RECOIL_MIN to HURT_RECOIL_MAX degrees by the hit's strength (its hitstun,
## HURT_WEAK to HURT_STRONG frames). It used to lean back 26 degrees for the whole stun, which read as one pose with the head thrown back.
const HURT_SNAP_FRAMES := 3.0
const HURT_RECOIL_MIN := 12.0
const HURT_RECOIL_MAX := 26.0
const HURT_WEAK := 8.0
const HURT_STRONG := 32.0
const HURT_RECOVERED := 0.7


func _recoil(s: Dictionary) -> void:
	model.rotation.x = deg_to_rad(hurt_recoil(s))


## The body's pitch (degrees, the model's x rotation) in a hit reaction that does not tumble: away from the hit (back off a blow from the
## front, forward off one from behind), by the snapshot only, so a freeze holds it and a rollback redraws it.
static func hurt_recoil(s: Dictionary) -> float:
	var left := float(s.get("hitstun", 0))
	var f := float(s.state_frame)
	var total := maxf(1.0, f + left)
	var strength := clampf((total - HURT_WEAK) / (HURT_STRONG - HURT_WEAK), 0.0, 1.0)
	var amount := lerpf(HURT_RECOIL_MIN, HURT_RECOIL_MAX, strength)
	var kb: Vector2 = s.get("kb_vel", Vector2.ZERO)
	# (Rotation about x leans the body back when negative, facing either way.)
	var away := 1.0 if kb.x * float(s.facing) > 0.02 else -1.0
	var k := 0.55
	if int(s.get("hitlag", 0)) <= 0:
		var hold := HURT_SNAP_FRAMES + 2.0 + 5.0 * strength
		var back := maxf(hold + 2.0, total * HURT_RECOVERED)
		if f < HURT_SNAP_FRAMES:
			var u := f / HURT_SNAP_FRAMES
			k = 0.55 + 0.45 * (1.0 - (1.0 - u) * (1.0 - u))
		elif f < hold:
			k = 1.0
		elif f < back:
			var u := (f - hold) / (back - hold)
			k = 1.0 - u * u * (3.0 - 2.0 * u)
		else:
			k = 0.0
	return away * amount * k
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
## On the take-off the body is stretched at once to this (negative stretches up), then the spring eases it to the rise's own stretch.
const TAKE_OFF_STRETCH := -0.15
## On touching down the body is squashed at once to LAND_IMPACT plus LAND_IMPACT_PER_SPEED times the fall speed (units per frame, up to
## 0.35), still pushing down at LAND_IMPACT_PUSH times it, and the spring rebounds it to the landing's own squash.
const LAND_IMPACT := 0.1
const LAND_IMPACT_PER_SPEED := 0.55
const LAND_IMPACT_PUSH := 6.0
## Weight by class (round 2, the owner: the heavy fighters moved as light as the quick ones): [the body springs' stiffness, the landing
## squash] as shares of the usual. The heavy fighters (Greatsword, Bracers, Clutch) sink deeper on landing and settle slower, with less
## bounce; the quick ones (Needles, Dagger) snap back faster and lighter. Presentation only: the sim's timing is untouched.
const CLASS_WEIGHT := {6: [0.62, 1.25], 7: [0.62, 1.25], 8: [0.62, 1.25], 2: [1.3, 0.8], 12: [1.3, 0.8]}


func _weight(s: Dictionary) -> Array:
	return CLASS_WEIGHT.get(_cls(s), [1.0, 1.0])
## How far down the jump's crouch starts, on its first frame (the spring eases it on to `SQUASH.JumpSquat`): the crouch lasts three
## frames, and eased in by the spring from standing it only reached its depth as the feet left the ground, so it hardly read (the
## owner's playtest notes, 2026-10-19: "no crouch before a jump"). (A kick to the spring instead overshot into a flat pancake.)
const JUMP_SQUAT_START := 0.2
## How long the jump's crouch clip takes to blend in (seconds; other clips take 0.1, longer than the whole crouch).
const JUMP_SQUAT_BLEND := 0.03

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
	ik_fade = 0.0
	neck_ok = false
	body_spring_frame = -1
	body_still = 0.0
	for t in tails:
		t.reset()
## How much a run's footfall squashes the body (and its stride stretches it), before the body's weight.
const STEP_SQUASH := 0.07
## Where the run cycle was last frame, for the footfalls.
var last_step_phase := 0.0


## The squash, lean and lunge springs' clock: the sim frame they last saw and how long the sim has not moved on.
var body_spring_frame := -1
var body_still := 0.0


## How many seconds the squash, lean and lunge springs move on this update: `delta` while the sim is moving, 0 while the fighter is frozen
## by hitlag or the game is paused, and -1 after a jump in the sim frame (a rollback or a seek: the springs go straight to their targets).
func _body_spring_time(s: Dictionary, delta: float) -> float:
	var f := int(s.frame)
	var jumped := body_spring_frame >= 0 and (f < body_spring_frame or f - body_spring_frame > 2)
	var moved := body_spring_frame >= 0 and f > body_spring_frame and not jumped
	body_spring_frame = f
	if jumped:
		return -1.0
	body_still = 0.0 if moved else body_still + delta
	if int(s.hitlag) > 0 or body_still >= MotionSprings.PAUSED_AFTER:
		return 0.0
	return delta


## With the environment variable PF_FACE_COVER set (a debugging aid for the motion audit), every frame in which a hand or the weapon's
## hilt is in front of the head and within `COVER_R` of the face's middle on screen is printed: "COVER <class> <state> <move> <frame>".
static var log_cover := OS.has_environment("PF_FACE_COVER")
const COVER_R := 0.4
const FRONT_COVER_Z := 0.15
## With the environment variable PF_ARM_SHOW set (round 8, the shoulder mock's measure): every frame in which a hand is raised above its
## shoulder is printed, "ARMSHOW <class> <state> <move> <frame> <side> <shown> <height>": shown 1 when the hand stands clear of the head
## on screen (outside the head's disc, or in front of it), and the hand's height above the head's middle (world units).
static var log_arm_show := OS.has_environment("PF_ARM_SHOW")


func _log_arm_show(s: Dictionary) -> void:
	if skeleton == null or head_bone < 0:
		return
	var hp: Transform3D = skeleton.global_transform * skeleton.get_bone_global_pose(head_bone)
	var centre: Vector3 = hp.origin + hp.basis.y.normalized() * HEAD_CENTRE_UP * hp.basis.y.length()
	var r: float = float((_base_rig_info().get("head_ball", [0.0, 0.0, 0.36]) as Array)[2]) * hp.basis.x.length()
	for side in ["L", "R"]:
		var h: Vector3 = skeleton.global_transform * skeleton.get_bone_global_pose(skeleton.find_bone("hand." + side)).origin
		var sh: Vector3 = skeleton.global_transform * skeleton.get_bone_global_pose(skeleton.find_bone("armU." + side)).origin
		if h.y <= sh.y + 0.05:
			continue
		var shown := Vector2(h.x - centre.x, h.y - centre.y).length() > r or h.z > centre.z + r * 0.5
		print("ARMSHOW %d %s %s %d %s %d %.3f" % [_cls(s), s.state, String(s.move_name).replace(" ", "_"), int(s.state_frame), side,
			1 if shown else 0, h.y - centre.y])


## With the environment variable PF_ARM_HIDE set (round 8, the owner's Longsword note "the arm is sometimes hidden underneath the
## body"): every frame, "ARMHIDE <class> <state> <move> <frame> <hand hidden> <forearm hidden>" for the weapon arm, hidden being inside the
## torso's outline on screen and behind its middle in depth (the torso: TORSO_UP above the spine joint, TORSO_HALF across and up).
static var log_arm_hide := OS.has_environment("PF_ARM_HIDE")
const TORSO_UP := 0.12
const TORSO_HALF := Vector2(0.28, 0.32)
const LEGS_HALF := Vector2(0.26, 0.32)
const LEGS_DOWN := 0.2
const TORSO_DEPTH := 0.3
const LEG_R := 0.19


func _log_arm_hide(s: Dictionary) -> void:
	var ispine := skeleton.find_bone("spine") if skeleton != null else -1
	if ispine < 0:
		return
	var to_frame := _skeleton_to_frame()
	var sp := skeleton.get_bone_global_pose(ispine)
	var t := to_frame * (sp.origin + sp.basis.y.normalized() * TORSO_UP)
	var side := _weapon_side(int(s.facing))
	var hand := to_frame * skeleton.get_bone_global_pose(skeleton.find_bone("hand." + side)).origin
	var elbow := to_frame * skeleton.get_bone_global_pose(skeleton.find_bone("armL." + side)).origin
	var fore := (hand + elbow) * 0.5
	# (And the hips and legs: the low carry's hand at thigh height went behind the near thigh, the owner's "underneath the body".)
	var hb := to_frame * skeleton.get_bone_global_pose(skeleton.find_bone("hips")).origin
	var near_z := -INF
	for leg in ["L", "R"]:
		near_z = maxf(near_z, (to_frame * skeleton.get_bone_global_pose(skeleton.find_bone("thigh." + leg)).origin).z)
	var hid := func(q: Vector3) -> int:
		var d := Vector2((q.x - t.x) / TORSO_HALF.x, (q.y - t.y) / TORSO_HALF.y)
		# (Behind the near surface: inside the body's thickness is hidden too, not only behind its middle.)
		if d.length() < 1.0 and q.z < t.z + TORSO_DEPTH * sqrt(1.0 - d.length_squared()):
			return 1
		var e := Vector2((q.x - hb.x) / LEGS_HALF.x, (q.y - (hb.y - LEGS_DOWN)) / LEGS_HALF.y)
		return 1 if e.length() < 1.0 and q.z < near_z + LEG_R * sqrt(1.0 - e.length_squared()) else 0
	print("ARMHIDE %d %s %s %d %d %d" % [_cls(s), s.state, String(s.move_name).replace(" ", "_") if s.move_name != "" else "-",
		int(s.state_frame), hid.call(hand), hid.call(fore)])


## How far the face may turn from the camera (its forward axis' depth, world space) and still count as seen.
const FACE_AWAY_Z := -0.2


func _log_face_cover(s: Dictionary) -> void:
	if skeleton == null or head_bone < 0:
		return
	var hp: Transform3D = skeleton.global_transform * skeleton.get_bone_global_pose(head_bone)
	var centre: Vector3 = hp.origin + hp.basis.y.normalized() * 0.3
	# (Round 8: a face turned away from the camera, the back of the head toward it in a whole-body turn, cannot be covered.)
	if hp.basis.z.normalized().z < FACE_AWAY_Z:
		return
	# (Round 4: PF_FACE_COVER=front measures from the drawn face, FACE_FWD ahead of the head's middle, where side-on it really is.)
	if OS.get_environment("PF_FACE_COVER") == "front":
		centre.x += FACE_FWD * float(s.facing)
		# (Round 5: and a point must be nearer the camera than the drawn face's own depth, a little in front of the head's middle: a hand
		# sent behind the head counted, though the head hid it.)
		centre.z += FRONT_COVER_Z
	var points := []
	var names := []
	for side in ["L", "R"]:
		points.append(skeleton.global_transform * skeleton.get_bone_global_pose(skeleton.find_bone("hand." + side)).origin)
		names.append(side + ("w" if side == _weapon_side(int(s.facing)) else ""))
	if blade_pivot != null and blade_pivot.visible:
		points.append(blade_pivot.global_position)
		names.append("hilt")
	# (And the feet, round 3: an overhead kick's leg crossed the face with both hands clear.)
	for side in ["L", "R"]:
		points.append(skeleton.global_transform * skeleton.get_bone_global_pose(skeleton.find_bone("foot." + side)).origin)
		names.append("foot" + side)
		var knee := skeleton.global_transform * skeleton.get_bone_global_pose(skeleton.find_bone("shin." + side)).origin
		points.append((knee + points[points.size() - 1]) * 0.5)
		names.append("shin" + side)
	for i in points.size():
		var p: Vector3 = points[i]
		if p.z > centre.z and Vector2(p.x - centre.x, p.y - centre.y).length() < COVER_R:
			print("COVER %d %s %s %d %s" % [_cls(s), s.state, String(s.move_name).replace(" ", "_"), int(s.state_frame), names[i]])
			return


## `s` is a dictionary of sim state (see main.gd `_refresh`).
func apply(pos: Vector3, s: Dictionary, delta: float) -> void:
	# The old built body's brawler has longer limbs: swap to its rig the first time we see its class. (The base body is one rig for every
	# class, so it never rebuilds here: a rebuild on the first frame of a match also left that fighter's per-mesh shader values, the ink
	# pass's object id among them, stale on part of the body, which drew a line round its face.)
	if rig != null and not use_base_body and (_cls(s) == 1) != long_limbs:
		long_limbs = _cls(s) == 1
		rebuild(loadout)
	position = pos
	# The hit freeze shake (as the genre's leaders do): a fighter frozen by a hit it takes vibrates side to side, harder for a harder hit
	# (by the hitlag left) and settling as the freeze runs out, so every hit lands with weight. From the sim's frame, so a replay matches.
	if int(s.hitlag) > 0 and s.launch_pending:
		var k := clampf(float(s.hitlag) / 16.0, 0.0, 1.0)
		var side := 1.0 if int(s.frame) % 2 == 0 else -1.0
		position += Vector3(side * (0.04 + 0.16 * k), side * 0.03 * k, 0.0)
	var state: String = s.state
	# Flash Step: nothing to see while it teleports.
	if model != null:
		var gone: Array = FLASH_STEP_GONE if _cls(s) == 2 else WARP_GONE
		model.visible = not ((_cls(s) == 2 or _cls(s) == STAFF_CLASS) and state == "Attack" and s.move_name == "up special"
			and int(s.state_frame) >= gone[0] and int(s.state_frame) <= gone[1])
	_ledge_motion(s, pos)
	var facing: int = s.facing
	var vy: float = s.vel.y
	var grounded: bool = s.platform >= 0
	# Standing, the drop shadow reaches only just past the floor (the full depth also printed it on whatever lies under the stage: two
	# dark ovals on the beach's rocks and sea, art review 5); in the air it reaches all the way down, so the landing spot stays marked.
	if contact_shadow != null:
		var depth: float = SHADOW_DEPTH_GROUNDED if grounded else SHADOW_DEPTH
		if contact_shadow.size.y != depth:
			contact_shadow.size.y = depth
			contact_shadow.position.y = 0.8 - depth / 2.0

	# Touching down: the body squashes at once, deeper the faster it fell, and the spring rebounds it from there. (Animation overhaul
	# step 9: kicked from the fall's shape, the spring took four frames to reach its depth, so the squash peaked as the landing lag ended,
	# not on the impact, and overshot to 0.4 after an aerial.)
	if grounded and not was_grounded:
		var impact := clampf(absf(last_vy), 0.0, 0.35)
		var heft: float = float(_weight(s)[1])
		squash = maxf(squash, (LAND_IMPACT + impact * LAND_IMPACT_PER_SPEED) * heft)
		squash_vel = impact * LAND_IMPACT_PUSH * heft
	if state == "JumpSquat" and last_state != "JumpSquat":
		squash = maxf(squash, JUMP_SQUAT_START)
		squash_vel = maxf(squash_vel, 0.0)
	was_grounded = grounded
	if not grounded:
		last_vy = vy
	# Leaving the jump squat: a kick into a tall stretch, so the take-off springs up out of the crouch rather than only rebounding.
	if last_state == "JumpSquat" and state != "JumpSquat" and not grounded:
		# (Stretched at once on the first frame in the air, step 9: kicked from the crouch, the stretch peaked four frames after the
		# feet left the ground, so the first airborne frames still read as the crouch.)
		squash = minf(squash, TAKE_OFF_STRETCH)
		squash_vel = -TAKE_OFF_KICK * 0.3

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
			# (Not on the base body: the limb or weapon reaches for the hit, the body does not grow toward it. The owner,
			# 2026-10-17.)
			target_squash = 0.0 if use_base_body else -0.2
	if state == "Airborne" or state == "Helpless":
		target_squash = -0.1 if vy > 0.06 else (-0.05 if vy < -0.12 else 0.0)
	var target_lean: float = LEAN.get(state, 0.0)
	# (Round 8: dazed on the ground after a shield break the body stands and sways (`dizzy`); the lean was for the hurt pose it held.)
	if state == "ShieldBreak" and grounded and use_base_body:
		target_lean = 4.0
	# Standing up from the floor: the body rises from lying (the knockdown's lean) as the `getup` clip sits it up and gets the feet
	# under it.
	if state == "GetUp" and dodge_progress(s) >= 0.0:
		target_lean = LEAN.Knockdown * (1.0 - smoothstep(0.0, 0.55, dodge_progress(s)))
	if heavy_landing:
		target_lean = 8.0
	# (A rocket's flight: the body laid along its path, round 2.)
	if state == "Attack" and (FLIGHT_MOVES.get(_cls(s), []) as Array).has(String(s.move_name)) and Vector2(s.vel).length() > 0.05:
		target_lean = clampf(rad_to_deg(atan2(float(s.vel.x) * float(s.facing), float(s.vel.y))), -80.0, 95.0)
	# The whole body swings with the hit (in the manner of the reference game's big aerials): it pitches into a hit in front or below,
	# arches back under one overhead and curls forward away from one behind, as far as the limb is into its reach.
	if state == "Attack" and s.move_tip != Vector3.ZERO and not _held_pose(s):
		var tip: Vector3 = s.move_tip
		var phi := rad_to_deg(atan2(tip.x, -(tip.y - 1.1)))   # 0 straight down, 90 ahead, 180 overhead, negative behind
		var swing := clampf((130.0 - phi) * 0.32, -22.0, 30.0) if phi >= 0.0 else clampf(-phi * 0.18, 0.0, 22.0)
		if not grounded and use_base_body and bladed(_cls(s)) and String(s.move_name) == "nair":
			target_lean += _nair_roll(s)
		elif not grounded and use_base_body:
			target_lean += _air_counter(s, phi) * _reach_weight(s)
			# (Round 8, review item 18: an aerial whose body curls right over onto its hit, the dinosaur's headbutt down onto a hit low
			# ahead; at the swing's 18 degrees the round body lunged flat, the head level with the hit's top.)
			if not r8_off.has("curl"):
				target_lean += float((AIR_CURL.get(_cls(s), {}) as Dictionary).get(String(s.move_name), 0.0)) * _reach_weight(s)
		else:
			# (A body with legs leans with its spine and hips, from its clips and the body-follow; tipping the whole model would pivot it
			# on its feet, so the base body keeps only a little of this on the ground.)
			# (Never tipped back under an overhead hit on the base body: it reaches up with the arm, not the whole body. The owner,
			# 2026-10-17.)
			if use_base_body:
				swing = maxf(swing, 0.0)
			target_lean += swing * _reach_weight(s) * (float(_line_of_action(s).x) if use_base_body else 1.0)
			# (The greatsword's weight, pass 10, the pose review's item: the whole body rocks back to load the huge blade through the coil
			# and is thrown forward into the cut, rather than standing upright while the blade moves.)
			if _cls(s) == GREATSWORD_CLASS and grounded and not _body_clip(_cls(s), String(s.move_name)):
				var mt: PackedInt32Array = s.move_timing
				target_lean += -GREAT_LOAD_BACK * coil_k(float(s.state_frame), float(mt[1])) + GREAT_LOAD_IN * _reach_weight(s)
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
		elif f >= first and f <= last:
			target_lunge = 0.32 * lunge_k
		elif f > last:
			# (Round 3: the lunge is held through the hold and drawn back with the recovery, as the reference's lunges are; it sprang back
			# two frames after the hit.)
			# (Round 4: through the carry the body goes on forward a little behind the swing, then settles into the hold.)
			var shape := _end_shape(s)
			var carry := LUNGE_CARRY * follow_k(f, last, shape)
			target_lunge = 0.32 * lunge_k * (1.0 + carry) * (1.0 - home_k(f, last, float(s.move_timing[0]), shape))
	var fast_falling: bool = s.fast_fall and not grounded and (state == "Airborne" or state == "Helpless" or state == "ShieldDrop")
	if fast_falling:
		target_squash = -0.28

	# Fixed sub-steps keep the spring stable at any frame rate.
	# (Round 2: stepped only while the sim moves on, as the motion springs are (`MotionSprings`): frozen through hitlag and a pause, so a
	# strong hit's freeze no longer wobbles, and put straight on their targets after a rollback or a replay seek.)
	var body_dt := _body_spring_time(s, delta)
	if body_dt < 0.0:
		squash = target_squash
		lean = target_lean
		lunge = target_lunge
		squash_vel = 0.0
		lean_vel = 0.0
		lunge_vel = 0.0
		body_dt = 0.0
	var steps := ceili(body_dt / 0.008)
	var h := body_dt / maxi(steps, 1)
	# (A heavier body's springs are softer and damped toward critical, so it sinks and settles slowly without bouncing; a light one snaps.)
	var wk: float = float(_weight(s)[0])
	var sk := SPRING_K * wk
	var sd := SPRING_DAMP * sqrt(wk) * (1.25 if wk < 1.0 else 1.0)
	for _i in steps:
		squash_vel += (sk * (target_squash - squash) - sd * squash_vel) * h
		squash += squash_vel * h
		lean_vel += (sk * (target_lean - lean) - sd * lean_vel) * h
		lean += lean_vel * h
		lunge_vel += (SPRING_K * 1.6 * (target_lunge - lunge) - SPRING_DAMP * 1.2 * lunge_vel) * h
		lunge += lunge_vel * h
	squash = clampf(squash, -0.35, 0.55)

	# Running, every footfall squashes the body a little and every stride between them stretches it, in step with the clip.
	var step_squash := 0.0
	if (state == "Run" or state == "Dash") and anim != null and _gait_clip(current_clip):
		var length := anim.get_animation(current_clip).length
		var phase := fmod(anim.current_animation_position / maxf(length, 0.001), 1.0)
		# (Round 4: by the body's weight, `CLASS_WEIGHT`: a heavy body squashes about half as much again on each footfall, a light one
		# less, so the heavy run lands its weight.)
		var heft: float = float(_weight(s)[1])
		step_squash = STEP_SQUASH * heft * heft * cos((phase - 0.06) * 4.0 * PI)
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
	# In the big moments (hit, a broken shield, the respawn) the body swings round toward the camera, so the face and the reaction read
	# (the owner's direction, 2026-10-14: turned more frontal at times, as PEAK's characters are).
	var showy := 1.0 if state == "Hitstun" or state == "ShieldBreak" or state == "Respawn" else 0.0
	face_cam = move_toward(face_cam, showy, delta * (8.0 if showy > face_cam else 3.0))
	var turn_deg := lerpf(BODY_TURN + BODY_TURN_IN_ATTACK * attack_turn, FRONTAL_TURN, face_cam)
	# A hit behind (a back air, a back throw's swing): the body swings round through the camera toward the back for the strike, the chest
	# facing the swing, and back again after (the owner, 2026-10-17, after the reference's back airs). By the move's reach, so it
	# turns in with the wind-up and home through the end lag.
	var behind := 0.0
	if use_base_body and state == "Attack" and s.move_tip != Vector3.ZERO and float(s.move_tip.x) < -0.2:
		behind = _reach_weight(s)
	back_turn = move_toward(back_turn, behind, delta * 10.0)
	turn_deg = lerpf(turn_deg, -BACK_ATTACK_TURN, back_turn)
	# (Round 8, the standing pivot: turning round on the spot the body swings round about twice as fast, as the reference fighters
	# turn in a frame or two, and the head leads it (`_face_camera`); it swung round at the run's pace, the head with it.)
	var pivot_turn: bool = grounded and use_base_body and not r8_off.has("pivot") and PIVOT_STATES.has(state) \
		and absf(float(s.vel.x)) <= 0.08
	var yaw_to := float(look) * deg_to_rad(turn_deg)
	yaw = lerp_angle(yaw, yaw_to, clampf(delta * (PIVOT_RATE if pivot_turn else 14.0), 0.0, 1.0))
	yaw_left = rad_to_deg(wrapf(yaw_to - yaw, -PI, PI)) if pivot_turn else 0.0
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
		# (Round 7: a flip cut short in the air (an aerial or an air dodge out of a midair jump) settles upright over a few frames, as a
		# tumble does (`TUMBLE_SETTLE`), instead of turning up to 180 degrees in one.)
		if flip_left > 0.0 and not grounded and not r7_off.has("tumble"):
			var cut := 1.0 - flip_left / FLIP_TIME
			tumble_angle = wrapf(TAU * cut * cut * (3.0 - 2.0 * cut) * (-1.0 if flip_back else 1.0), -PI, PI)
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
	var move_flip := _move_flip(s)
	if move_flip != 0.0:
		flip += move_flip
		pivot = Vector3(0, 0.9, 0)
	var throw_body := _throw_body(s)
	var state_body := _state_body(s)
	model.rotation = Vector3(deg_to_rad(lean + throw_body.x + state_body.x + _air_tuck(s)) + flip, yaw + _pivot_spin(s) + throw_body.y,
		deg_to_rad(state_body.y))
	turning_over = flip != 0.0 or _spin(s) != 0.0 or state == "Hitstun" or state == "Knockdown" or state == "GetUp" or absf(tumble_angle) > 0.02
	# A spinning neutral air turns the whole body over once backward, in the plane of the stage (see NAIR_SPIN).
	var spin := _spin(s)
	if spin != 0.0:
		model.quaternion = Quaternion(Vector3(0, 0, 1), spin * float(facing)) * model.quaternion
		pivot = SPIN_PIVOT_3D
	# Spin about the middle of the body (or the hands, or the curled-up roll) rather than the feet.
	flip_offset = Vector3.ZERO
	# (In the air the whole body turns about its middle when it leans (see `_air_counter`), as a body with nothing under it does; on the
	# ground it leans over its feet.)
	if flip == 0.0 and spin == 0.0 and not grounded and absf(lean) > 0.3:
		pivot = AIR_PIVOT
	if flip != 0.0 or spin != 0.0 or (not grounded and absf(lean) > 0.3):
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
	# (Shown as a pale flicker over the fighter (`_flash`), not see-through: transparency let the dark outline shell behind the body
	# show through it, and over a sky the fighter turned a muddy brown.)
	ghost_now = ghost
	shield.visible = state == "Shield" or state == "ShieldDrop"
	if shield.visible:
		# The bubble shrinks and turns from blue through yellow to red as its health runs out.
		var hp: float = clampf(s.shield, 0.0, 1.0)
		shield.scale = Vector3.ONE * (0.45 + 0.55 * hp)
		# The stick's light-press tilt moves the bubble (sim 134; the sim's bubble, so pokes match what is drawn).
		var tilt: Vector2 = s.get("shield_tilt", Vector2.ZERO)
		shield.position = Vector3(tilt.x, 1.1 + tilt.y, 0)
		var col := Color(0.24, 0.56, 1.0, 0.35).lerp(Color(1.0, 0.3, 0.22, 0.45), 1.0 - hp)
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
	# A throw carried by the whole body (`_throw_body`): the step and the drop or rise.
	var throw_move := _throw_body(s)
	model.position += Vector3(throw_move.z * float(s.facing), throw_move.w, 0.0)
	_blink(delta)
	if log_motion and state in ["Attack", "Hitstun", "Run", "Dash", "Landing"]:
		_log_motion(s)
	if log_head:
		_log_head_legs(s)


## With the environment variable PF_MOTION_TRACE set (a debugging aid for the timing audit, round 3), every frame of an attack, a hit
## reaction or a run prints where the body's joints and the weapon's tip are, relative to the fighter's feet with forward as +x:
## "TRACE <class> <state> <move> <state frame> <hitlag> <name>:x,y ...". `tools/art/review/timing_report.py` turns it into speed and pose curves.
static var log_motion := OS.has_environment("PF_MOTION_TRACE")


func _log_motion(s: Dictionary) -> void:
	if skeleton == null:
		return
	var f := float(s.facing)
	var line := "TRACE %d %s %s %d %d" % [_cls(s), s.state, String(s.move_name).replace(" ", "_"), int(s.state_frame), int(s.hitlag)]
	for b in ["hips", "spine", "head", "hand.L", "hand.R", "foot.L", "foot.R"]:
		var bi := skeleton.find_bone(b)
		if bi < 0:
			continue
		var p: Vector3 = skeleton.global_transform * skeleton.get_bone_global_pose(bi).origin - global_position
		line += " %s:%.3f,%.3f" % [b, p.x * f, p.y]
	if blade_pivot != null and blade_pivot.visible:
		var t: Vector3 = blade_pivot.to_global(Vector3(_blade_tip_x(_cls(s)), 0.0, 0.0)) - global_position
		line += " tip:%.3f,%.3f" % [t.x * f, t.y]
	line += " pitch:%.1f" % rad_to_deg(model.rotation.x)
	if head_bone >= 0:
		var hd: Transform3D = skeleton.get_bone_global_pose(head_bone) * head_rest_inv
		var look: Vector3 = (skeleton.global_transform.basis * hd.basis * Vector3(0, 0, 1)).normalized()
		line += " look:%.2f,%.2f,%.2f" % [look.x * f, look.y, look.z]
	print(line)


## With the environment variable PF_HEAD_TRACE set (a debugging aid for round 7's head and leg audit), every frame of every state prints
## the head's and the legs' angles in the world, forward along the facing as +x and toward the camera as +z:
## "HL <player> <class> <state> <move> <state frame> <hitlag> <facing> <grounded> yaw:<model> head:<yaw>,<pitch>,<roll>
## chest:<yaw>,<pitch> hips:<yaw> rel:<head yaw on the chest>,<pitch> fix:<face limiter turn>,<lift> N:<near leg> F:<far leg>". Each
## leg is "<knee flex>,<knee yaw>,<knee up>,<shin twist>,<off hinge>,<toe yaw>,<toe pitch>,<sole roll>,<ankle x>,<ankle y>,<ankle z>,
## <knee z>": yaws in degrees from forward (0) toward the camera (+90); `tools/art/review/head_leg_report.py` reads it.
static var log_head := OS.has_environment("PF_HEAD_TRACE")
var face_fix_turn := 0.0
var face_fix_lift := 0.0


func _bone_world(bone: String) -> Transform3D:
	var i := skeleton.find_bone(bone)
	var g: Transform3D = skeleton.global_transform * skeleton.get_bone_global_pose(i)
	var d: Basis = (skeleton.global_transform.basis * skeleton.get_bone_global_pose(i).basis * skeleton.get_bone_global_rest(i).basis.inverse()).orthonormalized()
	return Transform3D(d, g.origin - global_position)


## A bone's own axis (0 x, 1 y along the bone) in the world, as posed or at rest.
func _bone_axis(bone: String, rest: bool, which := 0) -> Vector3:
	var i := skeleton.find_bone(bone)
	var b: Basis = skeleton.get_bone_global_rest(i).basis if rest else skeleton.get_bone_global_pose(i).basis
	return (skeleton.global_transform.basis * b[which]).normalized()


static func _hinge_twist(a: Vector3, b: Vector3, about: Vector3) -> float:
	a = (a - about * a.dot(about)).normalized()
	b = (b - about * b.dot(about)).normalized()
	return rad_to_deg(atan2(about.dot(a.cross(b)), a.dot(b)))


static func _roll_about(fwd: Vector3, up: Vector3) -> float:
	var ref := Vector3.UP - fwd * fwd.dot(Vector3.UP)
	if ref.length() < 0.05:
		return 0.0
	ref = ref.normalized()
	var u := (up - fwd * fwd.dot(up)).normalized()
	return rad_to_deg(atan2(fwd.dot(ref.cross(u)), ref.dot(u)))


func _log_head_legs(s: Dictionary) -> void:
	if skeleton == null or head_bone < 0 or not skeleton.is_inside_tree():
		return
	var f := float(s.facing)
	var fl := func(v: Vector3) -> Vector3: return Vector3(v.x * f, v.y, v.z)
	var head := _bone_world("head")
	var hf: Vector3 = fl.call(head.basis * Vector3(0, 0, 1))
	var hu: Vector3 = fl.call(head.basis * Vector3(0, 1, 0))
	var chest := _bone_world("spine")
	var cf: Vector3 = fl.call(chest.basis * Vector3(0, 0, 1))
	var hips := _bone_world("hips")
	var pf: Vector3 = fl.call(hips.basis * Vector3(0, 0, 1))
	var rel: Vector3 = (chest.basis.inverse() * head.basis) * Vector3(0, 0, 1)
	var mv := String(s.move_name).replace(" ", "_")
	var line := "HL %d %d %s %s %d %d %d %d yaw:%.1f" % [player, _cls(s), s.state, mv if mv != "" else "-", int(s.state_frame),
		int(s.hitlag), int(f), 1 if int(s.platform) >= 0 else 0, rad_to_deg(model.rotation.y) * f]
	line += " head:%.1f,%.1f,%.1f" % [rad_to_deg(atan2(hf.x, hf.z)), rad_to_deg(asin(clampf(hf.y, -1.0, 1.0))), _roll_about(hf, hu)]
	line += " chest:%.1f,%.1f hips:%.1f" % [rad_to_deg(atan2(cf.x, cf.z)), rad_to_deg(asin(clampf(cf.y, -1.0, 1.0))), rad_to_deg(atan2(pf.x, pf.z))]
	line += " rel:%.1f,%.1f" % [rad_to_deg(atan2(rel.x, rel.z)) * f, rad_to_deg(asin(clampf(rel.y, -1.0, 1.0)))]
	line += " fix:%.1f,%.1f t:%d ov:%d" % [face_fix_turn, face_fix_lift, int(s.frame), 1 if turning_over else 0]
	# (Round 8: how much of the held weapon lies over our own face or chest on screen, a share of 12 points along it (0 to 1), and
	# whether it is drawn in front of them: "wc:<face>,<chest>".)
	var wf := 0.0
	var wcst := 0.0
	if blade_pivot != null and blade_pivot.visible:
		var hp: Transform3D = skeleton.global_transform * skeleton.get_bone_global_pose(head_bone)
		var face_c: Vector3 = hp.origin + hp.basis.y.normalized() * 0.3
		var chest_c: Vector3 = skeleton.global_transform * skeleton.get_bone_global_pose(spine_bone).origin + Vector3(0.0, 0.3, 0.0)
		for i in 12:
			var pt: Vector3 = blade_pivot.to_global(Vector3(MESH_LENGTH * (float(i) + 0.5) / 12.0, 0.0, 0.0))
			if pt.z > face_c.z - 0.1 and Vector2(pt.x - face_c.x, pt.y - face_c.y).length() < 0.55:
				wf += 1.0 / 12.0
			elif pt.z > chest_c.z - 0.1 and Vector2(pt.x - chest_c.x, pt.y - chest_c.y).length() < 0.5:
				wcst += 1.0 / 12.0
	line += " wc:%.2f,%.2f" % [wf, wcst]
	var near := "L" if f > 0.0 else "R"
	var far := "R" if f > 0.0 else "L"
	for side in [near, far]:
		var th := _bone_world("thigh." + side)
		var sh := _bone_world("shin." + side)
		var ft := _bone_world("foot." + side)
		var hip: Vector3 = th.origin
		var knee: Vector3 = sh.origin
		var ankle: Vector3 = ft.origin
		var up_leg := knee - hip
		var low_leg := ankle - knee
		var flex := rad_to_deg(up_leg.angle_to(low_leg))
		var axis := (ankle - hip).normalized()
		var kd: Vector3 = fl.call(up_leg - axis * up_leg.dot(axis))
		var kyaw := rad_to_deg(atan2(kd.z, kd.x)) if kd.length() > 0.01 else 0.0
		var kup := rad_to_deg(asin(clampf(kd.normalized().y, -1.0, 1.0))) if kd.length() > 0.01 else 0.0
		# The shin's twist on the thigh: the angle between their hinge axes (each bone's x, a knee's hinge) about the shin.
		# (Each bone's own x axis, its hinge, as posed; less the angle between them at rest, so a pure hinge reads 0.)
		var sd := low_leg.normalized()
		var tx := _bone_axis("thigh." + side, false)
		var sx := _bone_axis("shin." + side, false)
		var rest_twist := _hinge_twist(_bone_axis("thigh." + side, true), _bone_axis("shin." + side, true),
			(_bone_axis("shin." + side, true, 1)).normalized())
		var twist := _hinge_twist(tx, sx, sd) - rest_twist
		# How far the knee's bend plane is off the thigh's hinge (0: it bends about its own hinge).
		var off := 0.0
		if flex > 8.0:
			var n := up_leg.cross(low_leg).normalized()
			off = rad_to_deg(acos(clampf(absf(n.dot(tx)), 0.0, 1.0)))
		var toe: Vector3 = fl.call(ft.basis * Vector3(0, 0, 1))
		var sole: Vector3 = fl.call(ft.basis * Vector3(0, 1, 0))
		var a: Vector3 = fl.call(ankle)
		line += " %s:%.0f,%.0f,%.0f,%.0f,%.0f,%.0f,%.0f,%.0f,%.3f,%.3f,%.3f,%.3f" % ["N" if side == near else "F", flex, kyaw, kup, twist * f, off,
			rad_to_deg(atan2(toe.z, toe.x)), rad_to_deg(asin(clampf(toe.y, -1.0, 1.0))), _roll_about(toe, sole), a.x, a.y, a.z, knee.z]
	print(line)


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
const REST_ANGLE_NEAR := 32.0
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
const TAG_PIXEL := 0.00056
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
		name_label.outline_size = 14
		name_label.position.y = TAG_HEIGHT
		name_label.fixed_size = true
		name_label.pixel_size = TAG_PIXEL
		name_label.offset = Vector2(0, 50)
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


## Moves the match tag `dx` world units sideways and fades it to `alpha` (close combat: the tags of fighters side by side step apart so
## they never stack over the fighters' heads). Eased, so tags glide rather than jump.
func nudge_tag(dx: float, alpha: float, delta: float) -> void:
	if name_label == null or not match_tag:
		return
	var k := clampf(delta * 10.0, 0.0, 1.0)
	name_label.position.x = lerpf(name_label.position.x, dx, k)
	name_label.modulate.a = lerpf(name_label.modulate.a, alpha, k)
	if tag_arrow != null:
		tag_arrow.position.x = name_label.position.x * 0.5
		tag_arrow.modulate.a = name_label.modulate.a


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
## The Claws class's modelled claws, one on each hand (shown only for that class).
var claw_parts: Array[Node3D] = []
## The Dagger's knife, one on each hand (only the weapon hand's shows), and the Nodachi's long sword (on the blade's pivot).
var dagger_parts: Array[Node3D] = []
var nodachi_parts: Array[Node3D] = []
## How far past the knuckles the claws reach (forward space): the claw arcs and the fist's reach count it.
const CLAW_LENGTH := 0.3
## Where a modelled hand weapon's origin sits on the hand: this far from the wrist along the forearm's line.
const HAND_WEAPON_FROM_WRIST := 0.17


## Hangs the modelled hand weapons on the hand bones (base body only): each points along the forearm's line from the wrist, the back of
## the hand up (outward from the body at rest), so the fist's clench and every swing carry it. The model is built in that frame (+X out
## from the wrist, +Y the back of the hand).
func _mount_weapon_models() -> void:
	claw_parts.clear()
	dagger_parts.clear()
	nodachi_parts.clear()
	var steel := respond(toon(Color(0.5, 0.56, 0.68)), "metal")
	# The Nodachi: on the blade's pivot, laid out as the longsword is (blade 0.2 to 2.9 along x), so it is posed and stretched the same.
	# (`_model_scene` hands back a new copy each time: it goes on the fighter itself. A copy of it kept and left out of the tree outlived
	# the fighter, and the packaged game crashed on quit with the models still alive.)
	var long_sword := _model_scene("res://models/weapons/nodachi.glb")
	if long_sword != null:
		var nd: Node3D = long_sword
		blade_pivot.add_child(nd)
		_paint_weapon(nd, respond(toon(Color(0.86, 0.9, 0.97)), "metal"), toon(Color(0.2, 0.2, 0.32)))
		nd.visible = false
		nodachi_parts.append(nd)
	if not use_base_body or skeleton == null:
		return
	# (One copy per hand is made from these; they are freed after.)
	var claw_scene := _model_scene("res://models/weapons/claw.glb")
	var dagger_scene := _model_scene("res://models/weapons/dagger.glb")
	for side in ["L", "R"]:
		var ih := skeleton.find_bone("hand." + side)
		var il := skeleton.find_bone("armL." + side)
		if ih < 0 or il < 0:
			continue
		var hand_rest := skeleton.get_bone_global_rest(ih)
		var fore := (hand_rest.origin - skeleton.get_bone_global_rest(il).origin).normalized()
		var out := Vector3(-1.0 if side == "L" else 1.0, 0.0, 0.0)
		var inv := hand_rest.basis.orthonormalized().inverse()
		var x := (inv * fore).normalized()
		var y := (inv * out)
		y = (y - x * y.dot(x)).normalized()
		var frame := Transform3D(Basis(x, y, x.cross(y)), x * HAND_WEAPON_FROM_WRIST)
		var holder := BoneAttachment3D.new()
		holder.bone_name = "hand." + side
		skeleton.add_child(holder)
		if claw_scene != null:
			var claw: Node3D = claw_scene.duplicate()
			claw.transform = frame
			holder.add_child(claw)
			_paint_weapon(claw, steel, toon(trail_colours_of(1)[0].darkened(0.25)))
			claw.visible = false
			claw_parts.append(claw)
		if dagger_scene != null:
			var knife: Node3D = dagger_scene.duplicate()
			knife.transform = frame
			knife.set_meta("side", side)
			holder.add_child(knife)
			_paint_weapon(knife, respond(toon(Color(0.78, 0.83, 0.92)), "metal"), toon(Color(0.42, 0.26, 0.5)))
			knife.visible = false
			dagger_parts.append(knife)
	for source in [claw_scene, dagger_scene]:
		if source != null:
			source.free()


## Hangs a modelled pivot weapon (res://models/weapons/<name>.glb) on the blade's pivot and colours each part by the first word of its
## name (make_weapons.py: metal, dark, gold, band, ...). Returns its meshes, which take the hit flash with the body's.
func _pivot_weapon(name: String, mats: Dictionary) -> Array[MeshInstance3D]:
	var out: Array[MeshInstance3D] = []
	var scene := _model_scene("res://models/weapons/%s.glb" % name)
	if scene == null:
		push_warning("missing weapon model " + name)
		return out
	var root: Node3D = scene
	blade_pivot.add_child(root)
	for m in root.find_children("*", "MeshInstance3D", true, false):
		var mi := m as MeshInstance3D
		var kind := String(mi.name).get_slice("_", 0)
		mi.material_override = mats.get(kind, mats.values()[0])
		meshes.append(mi)
		out.append(mi)
	return out


## Colours a modelled weapon by its parts' names (make_weapons.py): `band*` the class's wrap, everything else the steel.
func _paint_weapon(root: Node3D, steel: Material, band: Material) -> void:
	for m in root.find_children("*", "MeshInstance3D", true, false):
		(m as MeshInstance3D).material_override = band if String(m.name).begins_with("band") else steel
## The class that fights with a staff (the Seer): posed like the sword, drawn as a staff.
const STAFF_CLASS := 5
## How far up the staff's shaft (toward its head) the other hand holds it in a swing (world units): the weapon hand is near its foot.
const STAFF_GRIP_APART := 0.55
## The class that fights with a greatsword (the Champion): posed like the sword (its blade follows the hits), drawn as a broad two-handed
## blade with the other hand on the grip; its kicks (the first two jabs, the sliding down tilt) and its throws keep the blade at rest.
const GREATSWORD_CLASS := 6
var great_parts: Array[MeshInstance3D] = []
## The greatsword's thrusts (the blade points at the hit throughout): the down air, driven straight down.
const GREAT_THRUSTS := ["dair"]
## How far (degrees) the greatsword's body rocks back to load a grounded swing, and leans into the cut (see `apply`, pass 10).
const GREAT_LOAD_BACK := 12.0
const GREAT_LOAD_IN := 14.0


## Whether a class swings a sword whose blade follows its hits (the Duelist's longsword, the Champion's greatsword).
static func bladed(cls: int) -> bool:
	return cls == 0 or cls == GREATSWORD_CLASS or cls == BUCKLER_CLASS or cls == NODACHI_CLASS


## The class that fights with a sword and a round shield (the Wanderer): the longsword's blade, and the shield on the other arm.
const BUCKLER_CLASS := 9
## The Dagger's class (a knife in the weapon hand, hung on the hand bone) and the Nodachi's (a very long sword on the blade's pivot).
const DAGGER_CLASS := 12
const NODACHI_CLASS := 13
var shield_disc: Node3D
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
## (Animation overhaul pass 4, after the reference's frames: the jab and the forward smash wind up further back, the blade behind the
## shoulder and head rather than upright in front of the face: 70 and 80 were.)
const SWING_FROM := {"jab": 120.0, "jab 2": -60.0, "ftilt": -70.0, "dash attack": -70.0, "fsmash": 95.0, "fair": 75.0, "dair": 75.0,
	"nair": 225.0, "uair": -60.0, "bair": 60.0}
## Swings whose wind-up goes the short way round from rest and into the strike (angles taken round the circle, not as numbers): the
## sword's neutral air raises the blade up and back behind the body, then brings it round underneath into the rising cut in front.
const SWING_ROUND := ["nair"]
## Kicks that send both legs out (front and back), each aimed by the game.
## (None now: the neutral air is a held kick with the other leg tucked, as the reference's are, which the clip poses.)
const SPLIT_KICKS: Array[String] = []
## (Its up air and down air are claw slashes, overhead and down below, so they swing the arm.)
const BRAWLER_NO_BLADE := ["utilt", "dtilt", "dash attack", "nair", "bair", "dair", "usmash", "side special", "up special", "down special", "grab", "dash grab", "pummel", "forward throw", "back throw", "up throw", "down throw"]
## Moves that rush the whole body forward in a flame.
const BRAWLER_FLAME := ["side special", "up special"]
var last_percent := -1
var blade_angle := REST_ANGLE
var rest_hand_y := 0.95
var blade_length := 2.0


## The weapon's resting angle (degrees, forward space): the longsword held up and a little forward; the greatsword carried over the
## shoulder, its blade lying back across the back (pointing up and behind), so it never hides the fighter or the stage in front.
func _rest_angle(s: Dictionary) -> float:
	if _cls(s) == GREATSWORD_CLASS or (_cls(s) == NODACHI_CLASS and use_base_body):
		return GREAT_REST_ANGLE
	# (Round 8, the owner: held weapons out of the way. A one-handed sword is carried low at the hip, the blade trailing down and back
	# below the chest, as the swordsman reference's; a staff stands upright beside the body, as the staff reference's.)
	if use_base_body and _cls(s) == STAFF_CLASS:
		return STAFF_REST_ANGLE
	if use_base_body and bladed(_cls(s)):
		return SWORD_REST_ANGLE
	# (Held in the near hand on the base body, the guard's fist is in front of the chest, so a blade up at 68 crossed the face: pointed
	# forward and up at the foe instead, a fencer's guard.)
	return REST_ANGLE_NEAR if use_base_body else REST_ANGLE


## (Round 8, the owner: the big blades (the Greatsword's and the Nodachi's) are carried one-handed low at the hip, the blade trailing low
## behind under the body's middle, as the broadsword reference's; they rested up over the shoulder (152 degrees, the hand at 1.25),
## the blade across the back and above the head. A one-handed sword trails the same way, shorter; the staff stands upright.)
const GREAT_REST_ANGLE := 196.0
const SWORD_REST_ANGLE := 214.0
const STAFF_REST_ANGLE := 78.0
## Where the hand holds a weapon at rest (forward space): low at the hip, a little ahead of it (the staff's at arm's length beside the
## body). The greatsword's was up by the shoulder.
## (Round 8, the owner's Longsword note "the arm is sometimes hidden underneath the body": a little further forward, 0.1 and 0.12 until
## then, so the fist reads beside the body, slightly forward, as the swordsman reference's.)
const GREAT_REST_HAND := Vector2(0.2, 0.6)
const SWORD_REST_HAND := Vector2(0.22, 0.6)
const STAFF_REST_HAND := Vector2(0.62, 0.8)
## How far toward the camera a resting weapon's hand is (in front of the body it crossed the chest; the swing's own depth is 0.35).
## (0.1 until the owner's note: at thigh height the fist sank into the near thigh, only the pommel showing, in 64-100% of the carry's
## frames; out past the thigh's surface it shows, the carry as low as before.)
const REST_HAND_Z := 0.3
## The staff stands on the far side of the body, upright (round 8: on the near side, held at the hip, its shaft ran up over the chest
## and the face; behind, it shows above the shoulder and below the hip, as the staff reference's stands clear of her front).
const STAFF_REST_Z := -0.15
## How far in front of (positive) or behind the body the greatsword's hand is, eased between the two.
var great_z := -0.45
## The hips' height in the sword family's battle stance: the guard is held for it, and lowered as far as the body sinks below it.
## (0.86 since round 8's stance C, the owner's pick: the battle stance stands taller, its hips 0.85-0.93 against 0.63-0.65.)
const GUARD_HIPS := 0.86
## How far the resting blade dips (degrees) for each unit the body sinks below the stance.
const GUARD_DIP := 260.0
## The greatsword's grip in depth at rest (round 8: beside the near leg, the blade trailing low; it was -0.45, the blade across the back).
const GREAT_REST_Z := 0.3   # (0.05 until the owner's hidden-arm note, as REST_HAND_Z)
## How far the greatsword's grip moves in depth a sim frame, between its rest behind the back and its swing in front.
const GREAT_Z_STEP := 0.135
## The greatsword's length (hand to point), whatever the hit: the blade reaches about 2 units past the grip.
const GREAT_LENGTH := 2.45


## Blades held in a pose of their own through a move (round 8): [angle, hand's distance ahead], ["rest"] for the low carry, or keys
## [move frame, angle, hand ahead, hand height] eased between (angles unwrapped, so a sweep goes the way it is written).
const HELD_BLADE := {
	0: {"down special": [40.0, 0.6]},
	# The Seer: the reticle's staff pointed ahead at the chest (at the shoulder it ran under the chin); the flame's tipped back behind the head, then thrust out level and lifted;
	# the counter's held up ahead of the body, tipped forward clear of the face (at 72 from 0.8 ahead it ran up across it).
	5: {"neutral special": [[0, 78.0, 0.62, 0.8], [4, 6.0, 0.85, 0.85], [8, 8.0, 0.95, 0.98], [60, 8.0, 0.95, 0.98], [80, 78.0, 0.62, 0.8]],
		"side special": [[0, 78.0, 0.62, 0.8], [12, 150.0, 0.2, 1.0], [22, 0.0, 1.0, 1.15], [44, 0.0, 1.0, 1.15], [58, 80.0, 0.65, 1.2],
			[80, 78.0, 0.62, 0.8]],
		"down special": [[0, 78.0, 0.62, 0.8], [6, 64.0, 1.0, 1.0], [70, 64.0, 1.0, 1.0], [86, 78.0, 0.62, 0.8]]},
	# The Champion: the wave's blade dragged low behind, swept under through the front and up overhead, held, then lowered behind to the
	# carry; the rising slash's from the carry under to low ahead (the first hit), straight up (the second) and tipping back overhead.
	6: {"neutral special": [[0, 196.0, 0.1, 0.6], [8, 215.0, 0.0, 0.75], [14, 330.0, 0.55, 0.85], [17, 400.0, 0.7, 1.15],
			[22, 455.0, 0.55, 1.45], [44, 460.0, 0.5, 1.45], [62, 556.0, 0.1, 0.6]],
		"up special": [[0, 196.0, 0.1, 0.6], [4, 350.0, 0.5, 0.8], [6, 360.0, 0.55, 0.9], [9, 450.0, 0.6, 1.25], [30, 460.0, 0.5, 1.4],
			[56, 480.0, 0.4, 1.35]]},
	# The Exile: the flare and the orbs keep the blade low; the barrier raises it upright ahead, then cuts ahead and down (the miss answer,
	# 38-53) and lowers it behind to the carry.
	13: {"neutral special": ["rest"], "extra move 3": ["rest"], "extra move 4": ["rest"], "extra move 5": ["rest"], "side special": ["rest"],
		# The blade dash: crouched with the blade trailing low behind through the hold (the reference's), then it follows the aimed dash.
		"up special": [[0, 196.0, 0.1, 0.6], [10, 205.0, -0.05, 0.65], [23, 205.0, -0.05, 0.65], "hit"],
		"down special": [[0, 196.0, 0.1, 0.6], [6, 76.0, 1.0, 1.0], [30, 76.0, 1.0, 1.0], [36, 104.0, 0.75, 1.25], [40, -5.0, 0.95, 1.05],
			[52, -20.0, 0.95, 0.95], [70, -164.0, 0.1, 0.6]]},
	# The Phantom: the gun and the curse bolt from the free hand, the guard behind the raised free arm; the knife kept low (round 8: it
	# reached for the shot out ahead, the whole body lunging after it).
	12: {"neutral special": ["rest"], "side special": ["rest"], "extra move 3": ["rest"], "extra move 4": ["rest"], "extra move 5": ["rest"],
		"extra move 6": ["rest"], "extra move 7": ["rest"], "extra move 8": ["rest"], "down special": ["rest"], "extra move 9": ["rest"]},
	9: {"neutral special": ["rest"], "extra move 4": ["rest"], "extra move 9": ["rest"], "extra move 10": ["rest"], "side special": ["rest"],
		"extra move 5": ["rest"], "extra move 6": ["rest"], "extra move 7": ["rest"], "down special": ["rest"], "extra move 8": ["rest"]},
}


## A move whose weapon holds a pose of its own (`HELD_BLADE`): a cast, a throw from the free hand, a counter's guard. Its body is the
## clip's alone; the lean and bend toward the move's hit (a projectile's or a counter bubble's, out ahead) bent it into a lunge (round 8).
func _held_pose(s: Dictionary) -> bool:
	if not use_base_body or r8_off.has("specials"):
		return false
	var held: Array = (HELD_BLADE.get(_cls(s), {}) as Dictionary).get(String(s.move_name), [])
	if not held.is_empty() and held[held.size() - 1] is String and held[0] is Array:
		return float(s.state_frame) <= float((held[held.size() - 2] as Array)[0])
	return not held.is_empty()


## The deepest a low-carried blade dips (degrees; 270 is straight down).
const LOW_CARRY_DIP_MAX := 250.0


## Whether this fighter carries its blade low, trailing behind (the swords, the Greatsword, the Exile's), rather than the staff upright.
func at_low_carry(s: Dictionary) -> bool:
	return _cls(s) != STAFF_CLASS and (bladed(_cls(s)) or _cls(s) == GREATSWORD_CLASS)


## Where the hand holds the weapon at rest (forward space), by class (round 8).
func _rest_hand(s: Dictionary) -> Vector2:
	if _cls(s) == GREATSWORD_CLASS or (_cls(s) == NODACHI_CLASS and use_base_body):
		return GREAT_REST_HAND
	if use_base_body and _cls(s) == STAFF_CLASS:
		return STAFF_REST_HAND
	if use_base_body and bladed(_cls(s)):
		return SWORD_REST_HAND
	return HAND


func _rest_length(s: Dictionary) -> float:
	# (Round 8: a modelled blade at rest is its own length, the hand to its tip.)
	if use_base_body and BLADE_TIP_X_BY.has(_cls(s)) and not r8_off.has("length"):
		return _blade_tip_x(_cls(s))
	# Swords are long; claws and bare hands are short.
	if _cls(s) == STAFF_CLASS:
		return 2.0
	if _cls(s) == GREATSWORD_CLASS:
		return GREAT_LENGTH
	return clampf(float(s.reach) - HAND.x - 0.65, 0.7, 3.4) if bladed(_cls(s)) else 1.0


## Where the blade should point, in degrees in forward space, and how long it should be, for this frame.
func _blade_target(s: Dictionary) -> Array:
	var rest_len := _rest_length(s)
	var name: String = s.move_name
	# (Round 8: moves whose blade holds a pose of its own rather than reaching for a hit: the Longsword's counter raises it up and ahead
	# of the face, the swordsman reference's guard; the Wanderer's free-hand specials keep it in the low carry.)
	var held_blade: Array = (HELD_BLADE.get(_cls(s), {}) as Dictionary).get(name, [])
	if use_base_body and not held_blade.is_empty() and not r8_off.has("specials"):
		if held_blade[0] is String:
			var rest_hand := _rest_hand(s)
			return [_rest_angle(s), rest_len, rest_hand.x, rest_hand.y]
		var f := float(s.state_frame)
		# (Keys ending "hit": held until the last key, then the blade follows the move's hit again, as an aimed dash's.)
		if held_blade[0] is Array and held_blade[held_blade.size() - 1] is String:
			held_blade = held_blade.slice(0, held_blade.size() - 1)
			if f > float((held_blade[held_blade.size() - 1] as Array)[0]):
				held_blade = []
		if held_blade.is_empty():
			pass
		elif held_blade[0] is Array:
			# (Keys: eased from one to the next by the move's frame; the fifth entry marks it a held pose that is not the carry.)
			var a: Array = held_blade[0]
			var b: Array = held_blade[0]
			for key in held_blade:
				b = key
				if float(key[0]) >= f:
					break
				a = key
			var k := 1.0 if float(b[0]) <= float(a[0]) else smoothstep(float(a[0]), float(b[0]), f)
			return [lerpf(float(a[1]), float(b[1]), k), rest_len, lerpf(float(a[2]), float(b[2]), k), lerpf(float(a[3]), float(b[3]), k), true]
		else:
			return [float(held_blade[0]), rest_len, float(held_blade[1])]
	if name == "" or ((_cls(s) == STAFF_CLASS or _cls(s) == GREATSWORD_CLASS) and _body_clip(_cls(s), name)):
		var rh := _rest_hand(s)
		return [_rest_angle(s), rest_len, rh.x, rh.y]
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
	if bladed(_cls(s)) or _cls(s) == STAFF_CLASS:
		# (A real weapon does not stretch to the hit: the longsword, the greatsword and the staff are always their own length, pointed at
		# the hit.)
		length = rest_len
	elif not bladed(_cls(s)):
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
	if _thrust(_cls(s), name):
		wind = swing
	var t: PackedInt32Array = s.move_timing  # total, first active, last active
	var f: float = s.state_frame
	var start := maxf(1.0, float(t[1]))
	var swing_from := start * SNAP_AT
	var angle := swing
	var len := length
	# (Round 5, the owner's A/B: `sword_sweep`. The Longsword's blade sweeps through its hit instead of snapping onto it and stopping:
	# it sets off sooner and arrives SWEEP_IN short of the hitbox's middle, crosses the hitbox through the active frames (still inside it),
	# and carries on from the far side. The sim's hitboxes and timing are the same either way.)
	var sweep := 0.0
	var travel := signf(angle_difference(deg_to_rad(wind), deg_to_rad(swing)))
	if sweeping(s):
		sweep = minf(SWEEP_IN, rad_to_deg(asin(clampf(float(tip.z) * 0.8 / maxf(length, 0.5), 0.0, 1.0))))
		swing_from = start * SWEEP_FROM
	if sweep > 0.0 and f >= swing_from and f < start:
		var u := (f - swing_from) / maxf(1.0, start - swing_from)
		angle = lerpf(wind, swing - travel * sweep, 0.5 * u + 0.5 * u * u)
	elif sweep > 0.0 and f >= start and f <= float(t[2]):
		var p := (f - start + 0.5) / (float(t[2]) - start + 1.0)
		angle = swing + travel * sweep * (2.0 * p - 1.0)
	elif f < swing_from:
		# The wind-up eases into its anticipation (fast away from rest, slowing as it coils) and holds there (pass 3: the coil holds
		# until SNAP_AT of the startup, as the body's does)...
		var k := coil_ease(f / maxf(1.0, start * COIL_AT))
		angle = lerpf(_rest_angle(s), wind, k)
		if SWING_ROUND.has(name):
			angle = rad_to_deg(lerp_angle(deg_to_rad(_rest_angle(s)), deg_to_rad(wind), k))
		len = lerpf(rest_len, length, k)
	elif f < start:
		# ...and the strike accelerates into the hit, so the blade whips through rather than gliding at one speed.
		var k := snap_k(f, start)
		angle = lerpf(wind, swing, k)
		if SWING_ROUND.has(name):
			angle = rad_to_deg(lerp_angle(deg_to_rad(wind), deg_to_rad(swing), k))
	elif f == float(t[2]) and _carry_lead(t) > 0.0 and not _thrust(_cls(s), name) and _end_shape(s).z > 0.0:
		# (Round 4: on the last frame of a hit that lasts more than one, the cut is already carrying on, still inside the hitbox: the
		# blade had stopped on the hit for a frame, then started again.)
		angle = swing + signf(angle_difference(deg_to_rad(wind), deg_to_rad(swing))) * CARRY_DEG * _carry_lead(t)
	elif f > t[2]:
		# The follow-through: the blade carries on past its last hit the way it was travelling, then eases home through the rest of the
		# end lag, as the reference's swordsmen finish a cut, rather than holding still at the hit and snapping back late.
		# (Round 3: it is held through the hold and comes home just after the body: `home_k`. Round 4: it carries on at CARRY_DEG a
		# frame for the move's carry, then slows to a stop: `follow_k`.)
		var shape := _end_shape(s)
		var carry_deg := CARRY_DEG * (shape.z + CARRY_EASE * 0.5) if shape.z > 0.0 else 0.0
		var past := swing + travel * sweep + travel * carry_deg * follow_k(f, float(t[2]) - _carry_lead(t), shape)
		# (Round 4: on the ground the held cut stops where the floor stops it, the hand as low as a lunge takes it (`HELD_HAND_Y`).
		# Held below it, the floor check pinned the tip still while the angle came home underneath, so the recovery showed only in its
		# last few frames: the forward smash measured 7 frames of recovery against the reference's 17.)
		if s.platform >= 0 and to_tip.x >= 0.0 and (bladed(_cls(s)) or _cls(s) == STAFF_CLASS):
			var floor_deg := rad_to_deg(asin(clampf((GROUND_CLEARANCE - HELD_HAND_Y) / length, -1.0, 1.0)))
			if past < floor_deg and past > -120.0:
				past = floor_deg
		# (Round 4: a cut that ends raised is held leaning forward, no steeper than HELD_UP_MAX: held upright, or carried on over the top,
		# the short arm left the hilt over the eyes, and drawn back behind the head the blade vanished behind it.)
		# (Not the Nodachi's: its long blade held forward came home across the face, 3 frames.)
		if s.platform >= 0 and _cls(s) != NODACHI_CLASS and past > HELD_UP_MAX and past < 160.0 and swing < 160.0:
			past = lerpf(past, HELD_UP_MAX, follow_k(f, float(t[2]) - _carry_lead(t), shape))
		var home := home_k(f, float(t[2]), float(t[0]), shape, ARM_LAG)
		var rest_deg := _rest_angle(s)
		# (Round 8, the owner's Longsword note on the neutral air: in the air the blade comes home the short way round. Taken as numbers,
		# a cut ending level in front (-8) went up over the front and round to the low carry behind (214), the blade swinging up again at
		# the end of the recovery; the swordsman lowers it down and back.)
		if int(s.platform) < 0 and use_base_body and not r8_off.has("nair"):
			rest_deg = past + rad_to_deg(angle_difference(deg_to_rad(past), deg_to_rad(rest_deg)))
		angle = lerpf(past, rest_deg, home)
		len = lerpf(length, rest_len, home)
	# (Round 8, the same note: between two authored arcs of one move (`ARCS`), the blade goes on from where the first ended toward where
	# the next starts, the way it travels, rather than pointing at a hit that is over. The neutral air's dropped forward and down between its
	# rising cut and its windmill, then snapped back up; the swordsman's stays up in front and rises a little into the windmill.)
	var gap := _arc_gap(s)
	if not gap.is_empty() and use_base_body and not r8_off.has("nair"):
		angle = float(gap[0])
	return [angle, len, hand_x]


## Between two of a move's authored arcs (`ARCS`): [the blade's angle, eased from the first's end to the next's start], else empty.
func _arc_gap(s: Dictionary) -> Array:
	var arcs: Array = (ARCS.get(_cls(s), {}) as Dictionary).get(String(s.move_name), [])
	var f := float(s.state_frame)
	for k in range(arcs.size() - 1):
		var a: Array = arcs[k]
		var b: Array = arcs[k + 1]
		if f > float(a[6]) and f < float(b[5]):
			var u := smoothstep(float(a[6]), float(b[5]), f)
			return [lerpf(float(a[4]), float(b[3]), u)]
	return []


## The carry starts this much before the last active frame when the hit lasts more than one frame (round 4), so the swing never stops on
## the hit and starts again.
static func _carry_lead(t: PackedInt32Array) -> float:
	return 0.5 if t[2] > t[1] else 0.0


## How far (radians) a raking hand carries on round its arc past the last hit (round 4: the carry, at about half a sword's pace).
func _limb_carry(s: Dictionary) -> float:
	var t: PackedInt32Array = s.move_timing
	if t.size() < 3 or float(s.state_frame) <= float(t[2]):
		return 0.0
	# (Not over the head: a hit overhead carried on round its arc comes down across the face.)
	var tip: Vector3 = s.move_tip
	if tip.y > 1.3 and absf(tip.x) < 1.1:
		return 0.0
	var shape := _end_shape(s)
	var k := follow_k(float(s.state_frame), float(t[2]), shape) * (1.0 - home_k(float(s.state_frame), float(t[2]), float(t[0]), shape, ARM_LAG))
	return deg_to_rad(LIMB_CARRY_DEG * (shape.z + CARRY_EASE * 0.5)) * k


const LIMB_CARRY_DEG := 20.0


## How low the hand is in a held lunge (forward space), for where the floor stops a held cut.
const HELD_HAND_Y := 0.6
## The swing A/B (round 5, the owner's call to make by playtest): false is the snap (the blade arrives on the hit at full speed and stands
## on it while it is live), true the sweep (`sweeping`). F10 in a match flips it; kept in user://animation.cfg. Presentation only.
static var sword_sweep := false
static var _sweep_loaded := false
const SWEEP_PATH := "user://animation.cfg"
## The sweep's arrival short of the hitbox's middle (degrees, at most; less for a small hitbox, so the blade is always inside it while it
## is live), and when (a share of the startup) the swing sets off.
const SWEEP_IN := 14.0
const SWEEP_FROM := 0.5


static func load_sweep() -> void:
	if _sweep_loaded:
		return
	_sweep_loaded = true
	var cfg := ConfigFile.new()
	if cfg.load(SWEEP_PATH) == OK:
		sword_sweep = bool(cfg.get_value("swing", "sweep", false))
	# (`--swing=sweep` or `--swing=snap` for this run only, for captures; not saved.)
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--swing="):
			sword_sweep = a.substr(8) == "sweep"


static func set_sweep(on: bool) -> void:
	_sweep_loaded = true
	sword_sweep = on
	var cfg := ConfigFile.new()
	cfg.load(SWEEP_PATH)
	cfg.set_value("swing", "sweep", on)
	cfg.save(SWEEP_PATH)


## Whether this move's blade sweeps through its hit (the A/B on, the Longsword, a cut: not a thrust and not the neutral air's windmill).
func sweeping(s: Dictionary) -> bool:
	load_sweep()
	return sword_sweep and _cls(s) == 0 and not _thrust(_cls(s), String(s.move_name)) and String(s.move_name) != "nair"


## The steepest (degrees above forward) a grounded cut is held at through its end lag.
const HELD_UP_MAX := 68.0


## How fast (degrees a frame) a blade carries on past its last hit (the reference's sword cuts carry about 25-30 degrees on in their first
## 1-2 frames, the jab's long cut further). Round 3 carried every cut 30 degrees, most of it in one frame.
const CARRY_DEG := 22.0


## The weapon arm's shoulder in forward space (x forward, y up), and how far the hand reaches from it in a swing.
const SHOULDER := Vector2(0.5, 1.15)
const ARM_REACH := 0.5
## How much of `arm_reach` (shoulder to wrist, and the hand) a blade's arm reaches on the base body (see `_pose_blade`): a soft bend.
const BLADE_ARM_REACH := 0.95
var arm_k := 0.0
var hand_target := Vector3.ZERO
## Where the greatsword's other hand holds the grip (model space), set with `hand_target`.
var off_hand_target := Vector3.ZERO
## The sword's thrusts: the blade points at the hit throughout and the arm drives it in, rather than swinging round to it.
const SWORD_THRUSTS := ["usmash", "dtilt"]
## The Wanderer's thrusts (after its reference): the down air plunges the blade straight down and the up air drives it straight up.
const BUCKLER_THRUSTS := ["dair", "uair"]


## Whether move `name` of class `cls` is a thrust (the blade points at the hit throughout and the arm drives it in).
static func _thrust(cls: int, name: String) -> bool:
	return (cls == 0 and SWORD_THRUSTS.has(name)) or (cls == GREATSWORD_CLASS and GREAT_THRUSTS.has(name)) 		or (cls == BUCKLER_CLASS and BUCKLER_THRUSTS.has(name))
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


## A fist's forward air on the base body, after the reference's (the owner's gold standard, 2026-10-17): the fist cocked up behind the
## head in the wind-up, brought over the top onto the hit on the first active frame, and carried on down to low in front through the
## follow-through, while the clip leans the body back and then curls it forward over the punch. Degrees about the shoulder, 0 forward.
const OVERHEAD_FROM := 125.0
const OVERHEAD_TO := -55.0


## Punches (by class) whose fist is drawn much further back than `FIST_CHAMBER` through the wind-up (forward space from the shoulder): the
## rusher's charged punch, the fist pulled right back behind the hip over its long wind-up, as its reference's. (Animation overhaul pass 12.)
const DEEP_CHAMBER := {4: {"neutral special": Vector2(-0.62, -0.3), "extra move 0": Vector2(-0.62, -0.3),
	"extra move 1": Vector2(-0.62, -0.3), "extra move 2": Vector2(-0.62, -0.3)},
	# (The Hauler's giant punch too, thrown from its charge's coil: round 2.)
	7: {"extra move 0": Vector2(-0.62, -0.3), "extra move 1": Vector2(-0.62, -0.3), "extra move 2": Vector2(-0.62, -0.3),
	"extra move 3": Vector2(-0.62, -0.3)}}


## Sword moves (by class) whose coil raises the sword hand overhead with the blade hanging down behind the head: [the hand's angle about the
## shoulder, the blade's angle] in degrees (0 forward, 90 up). The Longsword's forward smash, after its reference's frames 6-9 (the owner's
## open item from passes 3 and 4: the coil held the blade level behind the hips). Animation overhaul pass 16.
const BLADE_OVERHEAD := {0: {"fsmash": [100.0, 225.0]}}
## The depth (model space, toward the camera positive) the raised sword hand is drawn back to.
const BLADE_OVERHEAD_Z := -0.25


## Charges (by class) held in the clip's coil for as long as they last (the move has no hit of its own): the Hauler's giant punch wind-up,
## planted wide and low (`gaunt_charge`), while its arm swings round (`WINDMILL`). (Round 2: it stood in a plain
## guard, fists at the chest, for the whole charge.)
const HELD_COIL := {7: ["neutral special"], 11: ["up special", "extra move 5"], 2: ["neutral special", "extra move 3"],
	9: ["neutral special"]}
## Held coils entered from the stance, which ease into the coil over HELD_STRIKE_IN frames rather than snapping into it (round 8).
const COIL_EASE_IN := {2: ["neutral special"], 9: ["neutral special"]}
## Moves with no hitbox of their own (a projectile or a script throws the hit) and the frame their strike lands on, by class (round 8: the
## Needles' volley throws its first needle on frame 11 and casts its line on 13). Without it such a move struck on its first frame.
const RELEASE_FRAME := {2: {"extra move 4": 11, "side special": 13}, 0: {"down special": 6},
	9: {"extra move 4": 1, "extra move 9": 1, "extra move 10": 1, "side special": 27, "extra move 5": 27, "extra move 6": 2,
		"extra move 7": 2, "down special": 16, "extra move 8": 2},
	5: {"neutral special": 14, "side special": 22, "down special": 6},
	13: {"extra move 3": 3, "extra move 4": 3, "extra move 5": 3, "side special": 14},
	12: {"neutral special": 11, "side special": 15, "extra move 3": 2, "extra move 4": 2, "extra move 5": 2, "extra move 6": 2,
		"extra move 7": 2, "extra move 8": 15, "down special": 2, "extra move 9": 2}}
## Moves held in their strike pose for as long as they last (no hit times them): the Settler hangs from its balloons, both hands up on
## the strings over its head.
const HELD_STRIKE := {10: ["up special", "extra move 8"], 13: ["neutral special"], 5: ["down special"], 12: ["down special", "extra move 9"]}
const HELD_STRIKE_IN := 8.0
## Moves thrown straight out of such a charge: they start in the coil, the fist drawn back (`DEEP_CHAMBER`), instead of from the rest pose.
const COILED_START := {7: ["extra move 0", "extra move 1", "extra move 2", "extra move 3"]}
## Arms swung round and round through a charge, after the reference's giant punch: [frames a turn at the start, frames a turn at full
## charge, frames to full charge]. The fist goes over the top and down in front (an overarm windmill), faster as the charge builds, behind
## the head at the top so it never crosses the face. By the move's frame only, so a freeze holds it and a seek redraws it.
const WINDMILL := {7: {"neutral special": [30.0, 11.0, 110.0]}}
## How far out along the arm the windmilling fist goes (of the arm's reach), and its depth at the bottom and the top of the circle.
const WINDMILL_REACH := 0.95
const WINDMILL_Z := Vector2(0.5, -0.3)


## [the windmilling fist's angle about the shoulder (radians, 0 forward, pi/2 up), its depth], or [] when no arm is swinging round.
func _windmill(s: Dictionary) -> Array:
	if String(s.state) != "Attack" or s.move_name == "":
		return []
	var w: Array = (WINDMILL.get(_cls(s), {}) as Dictionary).get(String(s.move_name), [])
	if w.is_empty():
		return []
	var f := minf(float(s.state_frame), float(w[2]))
	# Turns so far: the rate goes evenly from 1 / w0 to 1 / w1 over w2 frames (and stays there after).
	var turns := f / float(w[0]) + f * f / (2.0 * float(w[2])) * (1.0 / float(w[1]) - 1.0 / float(w[0]))
	turns += maxf(0.0, float(s.state_frame) - float(w[2])) / float(w[1])
	# (Starting low behind the hip, swung back and up, over the top and down in front: the angle falls.)
	var angle := deg_to_rad(240.0) - TAU * turns
	var up := maxf(0.0, sin(angle))
	return [angle, lerpf(WINDMILL_Z.x, WINDMILL_Z.y, up)]


## Grounded hand-weapon moves (by class) whose wind-up cocks the hand up behind the head, as the overhead forward air does, so the strike
## comes over the top and down onto the hit: the Dagger's forward smash, a downward knife cut (animation overhaul pass 11), and the
## Hauler's forward air, both fists raised and hammered down in front (pass 12). Degrees about
## the shoulder (0 forward, 90 up): further back than the forward air's, so the raised knife sits behind the head, not across the face.
## Each is [degrees, depth]: the depth (model space, toward the camera positive) the cocked hand is drawn back to, behind the head for the
## knife, beside it for the fists (hidden behind it, they did not read).
## (The knife's 150 and -0.5, round 2: at 165 and -0.3 its point showed over the top of the head through the coil.)
const OVERHEAD_CHAMBER := {12: {"fsmash": [150.0, -0.5]}, 7: {"fair": [110.0, 0.05]}}


## Moves (by class) that spin the whole body about its upright axis, arms out, as their references do: [first state frame, last, turns],
## eased in and out. The all-rounder's down air (a spinning top), the Hauler's neutral air and its grounded and aerial spins, the
## Wanderer's spin attack (pass 14: its sword held out, SPIN_BLADE_TIP), the Settler's and the Scamp's neutral airs (pass 15). The pose is
## the arms-out clip (`spin_arms`, `spin_arms_air`); the turn depends only on the move's frame, so hitlag freezes it and a seek redraws
## it. (Animation overhaul pass 13.)
const SPIN_MOVES := {3: {"dair": [3, 24, 3.0]}, 7: {"nair": [6, 27, 1.5], "up special": [16, 64, 6.0], "extra move 6": [2, 66, 7.0]},
	9: {"up special": [5, 14, 2.0], "extra move 0": [6, 48, 6.0]}, 10: {"nair": [1, 23, 2.5]}, 11: {"nair": [6, 20, 2.0]}}
## Where a sword is held while its body spins (forward space: out level at the hip, so the turning body sweeps it round, as the
## Wanderer's spin attack does; the blade does not also follow the hitboxes, which go round with the body).
const SPIN_BLADE_TIP := Vector2(2.2, 0.95)


## Moves (by class) that turn the whole body over forward, head over heels, in the plane of the stage: [first state frame, last, turns],
## eased in and out. The Scamp's monkey flip (round 2). By the move's frame, so a freeze holds it and a seek redraws it.
const FLIP_MOVES := {11: {"side special": [12, 24, 1.0], "extra move 9": [12, 24, 1.0]}}
## Moves (by class) whose body pitches along its flight path (upright going straight up, laid forward going sideways): the Scamp's rocket.
const FLIGHT_MOVES := {11: ["extra move 6"]}


func _move_flip(s: Dictionary) -> float:
	if String(s.state) != "Attack" or s.move_name == "":
		return 0.0
	var w: Array = (FLIP_MOVES.get(_cls(s), {}) as Dictionary).get(String(s.move_name), [])
	if w.is_empty():
		return 0.0
	var k := clampf((float(s.state_frame) - float(w[0])) / maxf(1.0, float(w[1]) - float(w[0])), 0.0, 1.0)
	if k <= 0.0 or k >= 1.0:
		return 0.0
	return TAU * float(w[2]) * (k * k * (3.0 - 2.0 * k))


## How far round a spinning move has turned the body (radians, 0 when not spinning).
func _whirl(s: Dictionary) -> float:
	if String(s.state) != "Attack" or s.move_name == "":
		return 0.0
	var w: Array = (SPIN_MOVES.get(_cls(s), {}) as Dictionary).get(String(s.move_name), [])
	if w.is_empty():
		return 0.0
	var k := clampf((float(s.state_frame) - float(w[0])) / maxf(1.0, float(w[1]) - float(w[0])), 0.0, 1.0)
	if k <= 0.0 or k >= 1.0:
		return 0.0
	# (Eased in and out: a smoothstep on the turn, so it winds up and settles rather than starting and stopping dead.)
	return TAU * float(w[2]) * (k * k * (3.0 - 2.0 * k))


## Unarmed moves (by class) thrown with both fists together, the other arm reaching beside the striking one (`_aim_arm` with two hands):
## the Hauler's clap far in front, both fists hammered down in front (its forward air) and the aerial clap (its overhead clap is left one-handed:
## two fists straight up covered the face). (Animation
## overhaul pass 12, after its reference's; the clip alone could not, as an attack clip's free arm is held in close to the body.)
const BOTH_FISTS := {7: ["fsmash", "fair", "extra move 9"], 11: ["ftilt"]}
## Where the other fist sits against the striking one (model space: a little lower and further from the camera).
const BOTH_FISTS_APART := Vector3(0.0, -0.08, -0.32)


func both_fists(s: Dictionary) -> bool:
	return s.move_name != "" and int(s.state_frame) > 0 and (BOTH_FISTS.get(_cls(s), []) as Array).has(String(s.move_name))


## Whether this move is a fist's overhead forward air (see OVERHEAD_FROM).
func _overhead_punch(s: Dictionary) -> bool:
	return String(s.move_name) == "fair" and AnimLibrary.attack_clip(_cls(s), "fair", anim) == "attack_fair"


## The overhead punch's arm angle (radians about the shoulder) this frame, given the hit's direction `hit`.
func _overhead_angle(s: Dictionary, hit: float) -> float:
	var t: PackedInt32Array = s.move_timing
	var f := float(s.state_frame)
	var first := maxf(1.0, float(t[1]))
	var last := maxf(first, float(t[2]))
	var from := deg_to_rad(OVERHEAD_FROM)
	if f < first - 2.0:
		return from
	if f < first:
		# (A plain lerp, not the shortest way round: from up behind the head it comes over the top.)
		return lerpf(from, hit, (f - first + 2.0) / 2.0)
	return lerpf(hit, deg_to_rad(OVERHEAD_TO), clampf((f - first) / (last - first + 4.0), 0.0, 1.0))


func _pose_blade(s: Dictionary, delta: float) -> void:
	var target: Array = _blade_target(s)
	# (Round 3: out of a move, the guard goes down with the body when it sinks below the battle stance, in a jump's crouch or a landing,
	# and the blade dips with it. Held at the stance's height and angle, the blade crossed the sword fighters' mouths on every jump.)
	var sink := 0.0
	# (Round 8: and through a move that keeps the blade in the carry, HELD_BLADE "rest": undipped it was turned flat behind by the floor.)
	var held: Array = (HELD_BLADE.get(_cls(s), {}) as Dictionary).get(String(s.move_name), [])
	var held_rest: bool = target.size() == 4 and not held.is_empty()
	if (s.move_name == "" or held_rest) and use_base_body and skeleton != null and int(s.platform) >= 0:
		var hb := skeleton.find_bone("hips")
		if hb >= 0:
			sink = maxf(0.0, GUARD_HIPS - (_skeleton_to_frame() * skeleton.get_bone_global_pose(hb).origin).y)
			target[0] = float(target[0]) - sink * GUARD_DIP * (1.0 if cos(deg_to_rad(float(target[0]))) >= 0.0 else -1.0)
			# (Round 8: a blade carried low and trailing behind dips no further than straight down: past it, the owner's hidden-arm note, it
			# swung forward under the body after a walk or a landing.)
			if use_base_body and at_low_carry(s):
				target[0] = clampf(float(target[0]), 180.0, LOW_CARRY_DIP_MAX)
	# A little smoothing so the rest pose and quick direction changes do not pop.
	var follow := clampf(delta * 30.0, 0.0, 1.0)
	if s.move_name != "" and s.state_frame > 0:
		follow = 1.0  # while attacking, follow the animation exactly so it matches the hitbox
		# (Round 8: from the low carry the blade rises through the wind-up's first frames, not in one: the carry is far from where a
		# swing's wind-up starts. Exact again from the frame before the first hit.)
		var first_hit := float((s.move_timing as PackedInt32Array)[1])
		# (Not for a held pose's keys, `HELD_BLADE`: they ease already, and lagging them swept the Seer's staff across its face.)
		if use_base_body and (bladed(_cls(s)) or _cls(s) == STAFF_CLASS) and first_hit > 2.0 and float(s.state_frame) < first_hit - 1.0 \
				and target.size() != 5:
			follow = clampf(float(s.state_frame) / (first_hit - 1.0), 0.25, 1.0)
	elif use_base_body and (bladed(_cls(s)) or _cls(s) == STAFF_CLASS):
		# (Back down to the carry over about a third of a second, not snapped there.)
		follow = clampf(delta * 12.0, 0.0, 1.0)
	blade_angle = lerp_angle(deg_to_rad(blade_angle), deg_to_rad(float(target[0])), follow)
	blade_angle = rad_to_deg(blade_angle)
	blade_length = lerpf(blade_length, float(target[1]), follow)
	var facing: int = s.facing
	# Where the blade's tip is (forward space). It is fixed by the move's hitboxes and never moves because of the arm.
	var at_rest: bool = target.size() == 4
	var great_rest: bool = _cls(s) == GREATSWORD_CLASS and at_rest
	# (Round 8: the rest hand's height is the class's own, `_rest_hand`; it rises to the swing's through the wind-up and comes back down
	# through the recovery.)
	rest_hand_y = lerpf(rest_hand_y, float(target[3]) if at_rest else HAND.y, clampf(delta * (14.0 if at_rest else 30.0), 0.0, 1.0))
	if target.size() == 5:
		rest_hand_y = float(target[3])
	var old_hand := Vector2(float(target[2]), rest_hand_y)
	old_hand.y -= sink
	var tip := old_hand + Vector2.from_angle(deg_to_rad(blade_angle)) * blade_length
	if bladed(_cls(s)) and _whirl(s) != 0.0:
		tip = SPIN_BLADE_TIP
	# In a swing the hand leaves its resting spot and sweeps round the shoulder, so the arm throws the blade through its arc; the blade
	# then runs from the hand to the same tip.
	var attacking: bool = s.move_name != "" and s.state_frame > 0 and not (at_rest and s.move_name != "")
	arm_k = move_toward(arm_k, 1.0 if attacking else 0.0, delta * 9.0)
	# A punch reaches out with the wind-up and comes home in the recovery, like a kick (a blade stays in the hand throughout).
	if attacking and (not bladed(_cls(s)) or _thrust(_cls(s), String(s.move_name))):
		arm_k = _reach_weight(s)
	elif attacking and bladed(_cls(s)) and float(s.state_frame) > float((s.move_timing as PackedInt32Array)[2]):
		# (Pass 4: after the last hit the sword arm draws back in as the blade eases home, the elbow bending, instead of staying locked
		# out straight through the end lag; the pose review's "reaching for a hug".)
		arm_k = _reach_weight(s)
	# While the body spins (see NAIR_SPIN) the frame the blade is posed in turns with it, so the tip is turned back by as much: the arm
	# holds the blade out and the spin carries it round the circle the hits make.
	var spin := _spin(s)
	if spin != 0.0:
		tip = SPIN_PIVOT + (tip - SPIN_PIVOT).rotated(-spin)
	# The swing's arc (see ARCS): the sword's follows where its tip is; a claw is swept along its arc (its hitbox holds still while the
	# hand rakes through it, as the reference's do).
	var now := _arc_now(s, tip)
	if not bladed(_cls(s)) and not now.is_empty():
		tip = _arc_point(now, _limb_carry(s))
	var hand := old_hand
	# Where the weapon arm's shoulder really is now (forward space): the body turns and leans in a move, and the rest-pose shoulder was
	# often well away from it, so the planned reach ended short of the real arm's length and the sword hand stayed bent at the chin.
	var shoulder := shoulder
	# (Round 2: for every class, not only the swords. A fist's chamber and overhead cock were placed from the rest-pose shoulder, 0.4 ahead
	# of the real one, so a fist "drawn back beside the chest" sat out in front of the face and the claws covered it through every wind-up.)
	if use_base_body and skeleton != null:
		var iu := skeleton.find_bone("armU." + _weapon_side(facing))
		if iu >= 0:
			var live := _skeleton_to_frame() * skeleton.get_bone_global_pose(iu).origin
			shoulder = Vector2(live.x * facing, live.y)
	# (Pass 3: a punch chambers first. Through the wind-up the fist is drawn back beside the chest and held there, then thrown from
	# there with the snap.)
	var from_hand := old_hand
	# (How far a hand cocked up behind the head, OVERHEAD_CHAMBER, is drawn back behind it in depth, so the raised knife does not cross
	# the face.)
	var behind := 0.0
	var behind_z := 0.35
	if attacking and not bladed(_cls(s)) and not _thrust(_cls(s), String(s.move_name)):
		var mt: PackedInt32Array = s.move_timing
		if mt.size() >= 2 and float(s.state_frame) < float(mt[1]):
			# (An overhead forward air cocks the fist up behind the head instead, as the reference's does, both arms raised.)
			var chamber: Vector2 = shoulder + (DEEP_CHAMBER.get(_cls(s), {}) as Dictionary).get(String(s.move_name), FIST_CHAMBER)
			var overhead: Dictionary = OVERHEAD_CHAMBER.get(_cls(s), {})
			if use_base_body and _overhead_punch(s):
				chamber = shoulder + Vector2.from_angle(deg_to_rad(OVERHEAD_FROM)) * arm_reach * 0.85
			elif use_base_body and overhead.has(String(s.move_name)):
				var oc: Array = overhead[String(s.move_name)]
				chamber = shoulder + Vector2.from_angle(deg_to_rad(float(oc[0]))) * arm_reach * 0.85
				behind = coil_k(float(s.state_frame), float(mt[1]))
				behind_z = float(oc[1])
			var ck := coil_k(float(s.state_frame), float(mt[1]))
			if (COILED_START.get(_cls(s), []) as Array).has(String(s.move_name)) and float(s.state_frame) <= SNAP_AT * float(mt[1]):
				ck = 1.0
			from_hand = old_hand.lerp(chamber, ck)
			hand = from_hand
	if rig != null and arm_k > 0.0:
		var to_tip := tip - shoulder
		var dist := maxf(to_tip.length(), 0.001)
		# (A fist reaches further by the strike's stretch, `strike_e`, so the stretched arm is straight on the hit.)
		var reach := minf(arm_reach, maxf(dist - 0.3, 0.1)) if bladed(_cls(s)) else arm_reach * (1.0 + _arm_e())
		# (On the base body a blade's arm reaches out most of its length whatever the hit's distance: the arm and weapon extend to the hit,
		# never the body, and the blade carries on past a near hitbox. The owner, 2026-10-17. A little short of straight: a locked elbow
		# read as reaching for a hug, pose review.)
		if bladed(_cls(s)) and use_base_body:
			reach = arm_reach * BLADE_ARM_REACH * (1.0 + _arm_e())
		var dir := to_tip / dist
		# (A blade raised high is held out in front of the face, not over it: the arm is short against the big head, so straight up the
		# hilt ended in front of the face and the arm hid behind the blade. Art review 5, after the reference's up smash, whose arm is
		# straight and clear of the head.)
		if bladed(_cls(s)) and use_base_body and dir.angle() > deg_to_rad(HIGH_BLADE_ARM) and dir.angle() < deg_to_rad(150.0):
			dir = Vector2.from_angle(deg_to_rad(HIGH_BLADE_ARM))
		var goal := shoulder + dir * reach
		# A fist's forward air on the base body is an overhead punch (`_overhead_angle`), not a jab at the hitbox.
		if use_base_body and not bladed(_cls(s)) and _overhead_punch(s):
			goal = shoulder + Vector2.from_angle(_overhead_angle(s, dir.angle())) * reach
		if use_base_body:
			# With a real arm the hand travels round the shoulder, in an arc (a hook, a swing), not along a straight line.
			var from_v := from_hand - shoulder
			var to_v := goal - shoulder
			var angle := lerp_angle(from_v.angle(), to_v.angle(), arm_k)
			hand = shoulder + Vector2.from_angle(angle) * lerpf(from_v.length(), to_v.length(), arm_k)
		else:
			hand = from_hand.lerp(goal, arm_k)
	# (A charge's arm swinging round and round, WINDMILL: the fist on a circle about the shoulder, over the top behind the head.)
	var mill := _windmill(s)
	var mill_z := 0.0
	if use_base_body and mill.size() == 2:
		# (About the arm's live shoulder, which the camera turn puts behind the chest's middle: about the rest one, the circle sat out in
		# front of the face and the arm went round behind the head.)
		var centre := shoulder
		var mu := skeleton.find_bone("armU." + _weapon_side(facing)) if skeleton != null else -1
		if mu >= 0:
			var live_u := _skeleton_to_frame() * skeleton.get_bone_global_pose(mu).origin
			centre = Vector2(live_u.x * facing, live_u.y)
		hand = centre + Vector2.from_angle(float(mill[0])) * arm_reach * WINDMILL_REACH
		mill_z = float(mill[1])
	# (A sword raised overhead in the coil, BLADE_OVERHEAD: the hand goes up above the head and the blade hangs down behind it, as the
	# reference swordsman's forward smash; through the snap the hand comes down and the blade comes over the top onto the cut.)
	var blade_up := 0.0
	var overhead_blade: Array = (BLADE_OVERHEAD.get(_cls(s), {}) as Dictionary).get(String(s.move_name), [])
	if attacking and use_base_body and not overhead_blade.is_empty():
		var mt2: PackedInt32Array = s.move_timing
		if mt2.size() >= 2 and float(s.state_frame) < float(mt2[1]):
			blade_up = coil_k(float(s.state_frame), float(mt2[1]))
			var reach_up := arm_reach * BLADE_ARM_REACH
			var up_hand := shoulder + Vector2.from_angle(deg_to_rad(float(overhead_blade[0]))) * reach_up
			var blade_len := blade_length
			var base_deg := wrapf(rad_to_deg((tip - hand).angle()), -90.0, 270.0)
			hand = hand.lerp(up_hand, blade_up)
			tip = hand + Vector2.from_angle(deg_to_rad(lerpf(base_deg, float(overhead_blade[1]), blade_up))) * blade_len
	# A fist carries on a little past its strike and settles (the `hand` spring, kicked as the hit ends).
	if not bladed(_cls(s)) and absf(springs.value("hand")) > 0.001:
		hand += (hand - shoulder).normalized() * springs.value("hand")
	# Standing on the ground the tip never passes into the floor (a low cut whose arm drops into a deep lunge): the blade turns up about
	# the hand, keeping its length, just enough to clear it.
	# (Round 8: a weapon hand that comes up by the face is kept in front of it, out past the drawn face and no higher than the chin, as the
	# reference fighters hold a raised sword out ahead, instead of being sent behind the head (`HEAD_BEHIND_Z`), where the blade seemed
	# to come out of the neck with no hand showing. The blade still runs from the hand to its tip.)
	var front_hand := false
	if use_base_body and not r8_off.has("front") and bladed(_cls(s)) and _cls(s) != NODACHI_CLASS and blade_up == 0.0 and mill.is_empty() 			and skeleton != null and head_bone >= 0:
		front_hand = true
		var lag_now: bool = attacking and float(s.state_frame) > float((s.move_timing as PackedInt32Array)[2])
		var kf := _behind_head_k(hand, facing, FACE_FWD if lag_now and _cls(s) != NODACHI_CLASS else 0.0)
		if kf > 0.0:
			var hp := skeleton.get_bone_global_pose(head_bone)
			var hc := _skeleton_to_frame() * (hp.origin + hp.basis.y.normalized() * HEAD_CENTRE_UP)
			# (Out past the drawn face for a blade pointing ahead; past the back of the head for one pointing behind (a back air), so the
			# blade never crosses the face to reach its tip.)
			var ahead := tip.x >= hc.x * facing
			var clear_x := hc.x * facing + (FACE_FWD + HEAD_CLEAR_R * FRONT_CLEAR if ahead else -HEAD_CLEAR_R * FRONT_CLEAR)
			var clear := Vector2(clear_x, minf(hand.y, hc.y - HEAD_CLEAR_R * FRONT_BELOW))
			hand = hand.lerp(clear, kf)
	if s.platform >= 0 and tip.y < GROUND_CLEARANCE and hand.y > GROUND_CLEARANCE:
		var reach_len := (tip - hand).length()
		var dy := clampf((GROUND_CLEARANCE - hand.y) / maxf(reach_len, 0.001), -1.0, 1.0)
		var dx := sqrt(maxf(0.0, 1.0 - dy * dy)) * signf(tip.x - hand.x if absf(tip.x - hand.x) > 0.001 else 1.0)
		tip = hand + Vector2(dx, dy) * reach_len
	var along := tip - hand
	var length := maxf(along.length(), 0.3)
	var swing := rad_to_deg(along.angle())
	# Forward space to model space: facing left mirrors the pose about the vertical axis.
	var angle := swing if facing > 0 else 180.0 - swing
	# (The greatsword at rest lies behind the body, its blade across the back; swinging, it comes round in front.)
	var hand_z := lerpf(0.35, behind_z, behind * (1.0 - arm_k))
	if use_base_body and at_rest and (bladed(_cls(s)) or _cls(s) == STAFF_CLASS):
		hand_z = lerpf(STAFF_REST_Z if _cls(s) == STAFF_CLASS else REST_HAND_Z, 0.35, arm_k)
	if mill.size() == 2:
		hand_z = mill_z
	# (Round 2: a hand that comes up in front of the face, a sword hilt raised overhead in an up tilt or a claw cocked for a forward air,
	# goes behind the head instead, so the drawn face is never covered; the blade or claw still shows round the head. Judged frame by
	# frame at real speed, the hilt sat over the eyes for 8 frames of the Longsword's up tilt and the claws covered the face through the
	# forward air's wind-up.)
	elif use_base_body and blade_up == 0.0 and skeleton != null and head_bone >= 0 and _cls(s) != GREATSWORD_CLASS and not front_hand:
		# (Round 4: a sword held up through its end lag is measured from the drawn face, FACE_FWD ahead of the head's middle: the
		# Longsword's forward tilt held its hilt over the eyes for about 10 frames.)
		var end_lag: bool = attacking and float(s.state_frame) > float((s.move_timing as PackedInt32Array)[2])
		var held_up: bool = end_lag and bladed(_cls(s)) and _cls(s) != NODACHI_CLASS
		var k := _behind_head_k(hand, facing, FACE_FWD if held_up else 0.0)
		if k > 0.0:
			hand_z = lerpf(hand_z, HEAD_BEHIND_Z, k)
	# (Round 8: the Seer's staff is long and runs past the hand both ways, so its shaft crossed the face where its hands never came near
	# the head: the forward smash's wind-up laid it across the face, the up air stood it beside the chin. Wherever the shaft passes the
	# drawn face, the staff goes behind the head instead, eased by how near; it still shows above and below it.)
	if use_base_body and _cls(s) == STAFF_CLASS and skeleton != null and head_bone >= 0 and not r8_off.has("shaft"):
		var kshaft := _shaft_face_k(hand, tip, facing)
		if kshaft > 0.0:
			hand_z = minf(hand_z, lerpf(hand_z, HEAD_BEHIND_Z, kshaft))
	# (The raised sword is drawn back behind the head in depth, so the hanging blade does not cross the face.)
	if blade_up > 0.0:
		hand_z = lerpf(hand_z, BLADE_OVERHEAD_Z, blade_up)
	if _cls(s) == GREATSWORD_CLASS:
		# (By sim frames since round 3, so a freeze holds it and a seek puts it straight there; it moved on real time.)
		if springs.jumped:
			great_z = GREAT_REST_Z if great_rest else 0.35
		else:
			great_z = move_toward(great_z, GREAT_REST_Z if great_rest else 0.35, float(springs.new_frames) * GREAT_Z_STEP)
		hand_z = great_z
		# (Round 3: its two-handed grip goes behind the head too when the swing brings it up past the face, the broad blade still
		# showing round the head; it covered the face for 81 frames of its up moves and forward air.)
		if use_base_body and skeleton != null and head_bone >= 0 and not front_hand:
			hand_z = minf(hand_z, lerpf(hand_z, HEAD_BEHIND_Z, _behind_head_k(hand, facing)))
	hand_target = Vector3(hand.x * facing, hand.y, hand_z)
	# The greatsword is held in both hands: the other one a little further down the grip (toward the pommel), just behind the blade.
	var back_along := Vector2.from_angle(deg_to_rad(angle)) * -0.3
	# (The staff is held in both hands too, the other hand further down the shaft, so it swings as a staff, not poked one-handed:
	# animation overhaul pass 10, the pose review's staff item.)
	if _cls(s) == STAFF_CLASS:
		back_along = Vector2.from_angle(deg_to_rad(angle)) * STAFF_GRIP_APART
	off_hand_target = hand_target + Vector3(back_along.x, back_along.y, -0.15)
	# (A two-fisted blow, BOTH_FISTS: the other fist goes with the striking one, just beside and behind it, so both arms swing together.)
	if both_fists(s):
		off_hand_target = hand_target + BOTH_FISTS_APART
	_update_arc(now, delta)
	blade_pivot.position = hand_target
	blade_pivot.rotation = Vector3(0, 0, deg_to_rad(angle))
	var thick := 1.0 if bladed(_cls(s)) or _cls(s) == STAFF_CLASS else 1.7
	blade_pivot.scale = Vector3(length / MESH_LENGTH, thick, thick)
	# (Round 8: the modelled weapons at their own size; stretched to each hit's reach, their hilts were squashed or drawn out.)
	if use_base_body and (bladed(_cls(s)) or _cls(s) == STAFF_CLASS) and not r8_off.has("grip"):
		blade_pivot.scale = Vector3.ONE


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
		# (Pass 3: the limb stays in its coil through the hold and reaches out with the snap.)
		return snap_k(f, first)
	elif f > last:
		# (It stays out through the hold, then comes home just after the body: round 3, `home_k`.)
		return 1.0 - home_k(f, last, total, _end_shape(s), ARM_LAG)
	return k * k * (3.0 - 2.0 * k)


## The whole body answering a limb thrown out in the air (degrees of forward pitch, about the body's middle; the reference's aerials,
## exaggerated): an up air tips the body right back so the kick or the slash goes up and over (`AIR_BACK` at most), a back air pitches it
## forward, head down, so the leg drives out behind (`AIR_FORWARD`), a kick out in front leans the body back against the leg
## (`AIR_KICK_BACK`), a swing in front and below curls the body over it. `phi` is the hit's direction from the body (0 straight down, 90
## ahead, 180 overhead, negative behind). Weighted by the move's reach, so it eases in with the wind-up and out in the recovery, and
## sprung (`lean`), so it overshoots a little and settles.
const AIR_BACK := 36.0
const AIR_BACK_BASE := 12.0
const AIR_FORWARD := 50.0
## Aerials curled further forward over their hit (degrees, on top of `_air_counter`), by class (round 8): the Clutch's forward air, the
## headbutt down onto a hit low ahead, curls the body head-down as the dinosaur's does (a pitch, not a flip).
const AIR_CURL := {8: {"fair": 34.0}, 0: {"fair": 22.0}}
const AIR_KICK_BACK := 18.0
const AIR_KICK_BACK_BASE := 6.0
const AIR_PIVOT := Vector3(0, 1.0, 0)


## The sword's neutral air (animation overhaul pass 7, the owner's option B): the body stays upright and tucked while the arm windmills
## the blade round, and rolls back with the blade as it goes over the top and round behind, at most NAIR_ROLL degrees (never upside
## down), easing back upright as the blade comes round underneath to the front. (The general air counter-pitch pitched it forward,
## head down, as the blade passed behind.)
const NAIR_ROLL := 32.0


func _nair_roll(s: Dictionary) -> float:
	var w: PackedInt32Array = s.get("move_windows", PackedInt32Array())
	var t: PackedInt32Array = s.move_timing
	var first := float(w[w.size() - 2]) if w.size() >= 4 else float(t[1])
	var last := float(w[w.size() - 1]) if w.size() >= 4 else float(t[2])
	var k := clampf((float(s.state_frame) - first + 2.0) / maxf(1.0, last - first + 4.0), 0.0, 1.0)
	return -NAIR_ROLL * sin(PI * k)


func _air_counter(s: Dictionary, phi: float) -> float:
	var clip := AnimLibrary.attack_clip(_cls(s), s.move_name, anim)
	var kick := AnimLibrary.is_kick(clip)
	# (An overhead arc carried on past the top and round behind is still overhead: an up air keeps tipping back as it finishes.)
	if phi < -125.0:
		phi += 360.0
	if phi < 0.0:
		# Behind: pitch forward, more the further round behind it is.
		return clampf(-phi * 0.42, 0.0, AIR_FORWARD)
	# (The base body tips back only a little: the limb or weapon reaches overhead, not the whole body; 36 laid the round body flat. The
	# owner, 2026-10-17.)
	var back := AIR_BACK_BASE if use_base_body else AIR_BACK
	if phi > 200.0:
		return -back
	if phi > 120.0:
		# Overhead: tip back.
		return -clampf((phi - 105.0) * 0.62, 0.0, back)
	if kick and phi > 45.0:
		# A kick out in front: the body leans back against the leg (a jackknife). (Less on the base body: on top of its clips' lean it
		# laid the round body on its back, the head below the hips. Pose review 2026-10-17.)
		return -(AIR_KICK_BACK_BASE if use_base_body else AIR_KICK_BACK)
	# A swing in front: curl over it a little; straight below (a stomp, a drill) the body stays nearly upright over its legs.
	return clampf((110.0 - phi) * 0.24, 0.0, 18.0) * smoothstep(20.0, 60.0, phi) + 6.0 * (1.0 - smoothstep(20.0, 60.0, phi))


## Puts the weapon in the weapon hand as the clip moves it, swept back and up behind the runner (carried in a run).
const CARRY_ANGLE := 158.0


func _carry_weapon(facing: int) -> void:
	var hand := skeleton.find_bone("hand." + _weapon_side(facing))
	if hand < 0:
		return
	blade_pivot.position = _skeleton_to_frame() * skeleton.get_bone_global_pose(hand).origin
	var angle := CARRY_ANGLE if facing > 0 else 180.0 - CARRY_ANGLE
	blade_pivot.rotation = Vector3(0, 0, deg_to_rad(angle))


## (Round 8, the art director's grip pass: the weapon was placed where the hand was, but the fist kept its own turn, so the guard sat beside
## or behind the fist, or the grip ran through the wrist.) Turns the weapon hand so the grip passes through the curled fingers, the thumb's
## side toward the tip (the fist's knuckle row along the weapon's line, `make_rigged_blob.py` `fist`), the wrist bending as little as
## that allows, and puts the weapon's grip in the middle of the fist.
const GRIP_FROM_WRIST := 0.17


func _grip_hand(hb: int) -> void:
	var il := skeleton.get_bone_parent(hb)
	if il < 0:
		return
	var from_frame := _skeleton_to_frame().affine_inverse()
	var b: Vector3 = (from_frame.basis * Vector3(cos(blade_pivot.rotation.z), sin(blade_pivot.rotation.z), 0.0)).normalized()
	var pose := skeleton.get_bone_global_pose(hb)
	var f: Vector3 = (pose.origin - skeleton.get_bone_global_pose(il).origin).normalized()
	if absf(f.dot(b)) > 0.97:
		return
	var rest := skeleton.get_bone_global_rest(hb)
	var down_r: Vector3 = (rest.origin - skeleton.get_bone_global_rest(il).origin).normalized()
	var front_r := Vector3(0, 0, 1) - down_r * down_r.z
	front_r = front_r.normalized()
	var d := (f - b * f.dot(b)).normalized()
	var turn := Basis(d, b, b.cross(d)) * Basis(down_r, front_r, front_r.cross(down_r)).inverse()
	var scale := pose.basis.get_scale()
	var hand := Basis(turn.get_rotation_quaternion()) * Basis(rest.basis.get_rotation_quaternion()) * Basis.from_scale(scale)
	skeleton.set_bone_global_pose_override(hb, Transform3D(hand, pose.origin), 1.0, true)
	var grip: Vector3 = pose.origin + d * GRIP_FROM_WRIST * scale.y
	blade_pivot.position = _skeleton_to_frame() * grip


## How hard a change of vertical speed throws the hat down onto the head (or up off it).
const HAT_DROP_KICK := 13.0


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
		# (Step 9: at 6 the hat dipped about 0.04 on a landing, a twentieth of the head; now about twice that.)
		springs.kick("hat_drop", clampf(dv.y, -1.0, 1.0) * HAT_DROP_KICK)
		# Speeding up the way it faces leaves the chest and head behind (bent back); stopping throws them forward; landing nods them.
		var ahead := dv.x * float(facing)
		var landing := maxf(dv.y, 0.0) if int(s.platform) >= 0 else 0.0
		springs.kick("spine", -ahead * 320.0 + landing * 90.0)
		springs.kick("head", -ahead * 480.0 + landing * 160.0)
		springs.kick("neck", -dv.x * 40.0)
		# (Not when turning round on the spot, round 8: there the head leads the turn rather than looking back the old way.)
		if prev_facing != 0 and facing != prev_facing and not (int(s.platform) >= 0 and PIVOT_STATES.has(String(s.state))
				and absf(float(s.vel.x)) <= 0.08 and not r8_off.has("pivot")):
			springs.kick("head_turn", float(prev_facing) * 420.0)
	# A hit on the shield: as its freeze ends the chest and head are knocked back behind it and come forward again.
	if springs.new_frames > 0 and prev_hitlag > 0 and int(s.hitlag) == 0 and String(s.state) == "Shield":
		springs.kick("spine", -90.0 - 6.0 * float(prev_hitlag))
		springs.kick("head", -110.0 - 8.0 * float(prev_hitlag))
	if springs.new_frames > 0 or int(s.hitlag) > 0:
		prev_hitlag = int(s.hitlag)
	# A strike's follow-through: as a fist's hit ends the hand carries on past it and settles back.
	var f := int(s.state_frame)
	if String(s.state) == "Attack" and not bladed(_cls(s)) and prev_state_frame >= 0:
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


## The Bounder's egg round a fighter shut in it (state `Encased`, the bridge's `fighter_egg_radius`): a cream egg standing on the
## fighter's feet, inked, with soft spots in green, rocking a little as the fighter struggles; when it goes it breaks into shell pieces.
## The fighter inside is hidden. Made the first time it is needed. Presentation only.
var egg_node: Node3D
var egg_was := 0.0
const EGG_SHELL := Color(1.0, 0.96, 0.86)
const EGG_SPOT := Color(0.42, 0.72, 0.36)


func _egg(s: Dictionary) -> void:
	var r := float(s.get("egg", 0.0))
	if r <= 0.0:
		if egg_was > 0.0:
			Particles.shell_pieces(get_parent(), global_position + Vector3(0, egg_was * 1.12, 0.4), egg_was, EGG_SPOT, EGG_SHELL)
		egg_was = 0.0
		if egg_node != null:
			egg_node.visible = false
		return
	egg_was = r
	if egg_node == null:
		egg_node = Node3D.new()
		add_child(egg_node)
		var shell := MeshInstance3D.new()
		var sm := SphereMesh.new()
		sm.radial_segments = 28
		sm.rings = 16
		shell.mesh = sm
		shell.material_override = toon(EGG_SHELL)
		egg_node.add_child(shell)
		var rng := RandomNumberGenerator.new()
		rng.seed = 77
		for k in 7:
			var a := rng.randf_range(-1.1, 1.1)
			var e := rng.randf_range(-0.6, 0.8)
			var dir := Vector3(sin(a) * cos(e), sin(e), cos(a) * cos(e))
			var spot := MeshInstance3D.new()
			spot.mesh = SphereMesh.new()
			spot.material_override = toon(EGG_SPOT, false)
			var sr := rng.randf_range(0.14, 0.24)
			spot.look_at_from_position(dir * 0.5 * 0.97, dir * 0.97, Vector3.UP)
			spot.scale = Vector3(sr, sr, sr * 0.3)
			shell.add_child(spot)
	egg_node.visible = true
	model.visible = false
	# (The unit sphere is 1 across: an egg a little taller than wide, standing on the feet.)
	egg_node.position = Vector3(0, r * 1.12, 0)
	var shell_mesh: MeshInstance3D = egg_node.get_child(0)
	shell_mesh.scale = Vector3(r * 2.0, r * 2.3, r * 2.0)
	var t := float(s.frame)
	egg_node.rotation.z = 0.07 * sin(t * 0.55) + 0.05 * sin(t * 1.7)


## The Wanderer's round shield (made the first time it is needed): a wooden disc with a steel rim and boss in the accent colour, held on
## the forearm of the arm that does not hold the sword, faced toward the camera. Hidden for every other class. Presentation only.
func _place_buckler(s: Dictionary) -> void:
	var mine := _cls(s) == BUCKLER_CLASS
	if shield_disc == null:
		if not mine:
			return
		shield_disc = Node3D.new()
		model.add_child(shield_disc)
		# The modelled round shield (make_weapons.py `round_shield`, facing the camera): a domed wooden face, a steel rim and rivets, and
		# a boss in the fighter's accent colour.
		var disc := _model_scene("res://models/weapons/round_shield.glb")
		if disc != null:
			var mats := {"wood": toon(Color(0.62, 0.42, 0.28)), "metal": respond(toon(Color(0.7, 0.72, 0.8)), "metal"),
				"band": respond(toon(loadout.accent_color()), "gold")}
			var root: Node3D = disc
			shield_disc.add_child(root)
			for m in root.find_children("*", "MeshInstance3D", true, false):
				(m as MeshInstance3D).material_override = mats.get(String(m.name).get_slice("_", 0), mats.wood)
				meshes.append(m)
			_on_fighter_layer(shield_disc)
	shield_disc.visible = mine
	if not mine or skeleton == null:
		return
	var facing := int(s.facing)
	var hb := skeleton.find_bone("armL." + ("R" if _weapon_side(facing) == "L" else "L"))
	if hb < 0:
		return
	var at: Vector3 = _skeleton_to_frame() * skeleton.get_bone_global_pose(hb).origin
	shield_disc.position = stage_frame.transform * at + Vector3(0.12 * facing, -0.05, 0.25)


## The pivot grab's turn (radians added to the body's yaw): the simulation turns the fighter round at once as the grab starts, so the
## body spins back from the way it was running, round through the camera, and faces its new way by the grab's first active frame
## (the playtest notes, 2026-10-19: the pivot grab showed no turn).
const PIVOT_SPIN := PI


## Throws carried by the whole body (round 8, review item 33). One clip with the arms posed per throw read almost as before at the match
## camera (the raised arms hide behind the big head, the release under the hit's burst); the body's whole shape does read there. Eased in
## up to the throw's release (its first active frame) and held, then home over the second half of its end lag: the back throw turns the
## body round to face behind, through the camera; the forward throw pitches into a step; the down throw squats and pitches onto the slam;
## the up throw dips, then rises onto the toes as it heaves (not arched back: the owner's rule, 2026-10-17, the body never tips back under an
## overhead hit). The back throw turns home from the release, so the face is hidden only briefly. Returns [pitch (degrees, forward), turn (radians), step
## ahead, rise] as a Vector4.
const THROW_TURN := PI
const THROW_FWD_PITCH := 16.0
const THROW_FWD_STEP := 0.3
const THROW_DOWN_PITCH := 24.0
const THROW_DOWN_DROP := 0.24
const THROW_UP_DIP := 0.16
const THROW_UP_RISE := 0.38


func _throw_body(s: Dictionary) -> Vector4:
	var name := String(s.move_name)
	if String(s.state) != "Attack" or not name.ends_with(" throw") or not use_base_body or r8_off.has("throws"):
		return Vector4.ZERO
	var t: PackedInt32Array = s.move_timing
	var f := float(s.state_frame)
	var total := maxf(2.0, float(t[0]))
	var release := clampf(float(t[1]), 2.0, total)
	var into := smoothstep(0.0, release, f)
	var home := 1.0 - smoothstep(release + (total - release) * 0.5, total, f)
	var w := into * home
	match name:
		"back throw":
			var back_home := 1.0 - smoothstep(release, release + (total - release) * 0.6, f)
			return Vector4(0.0, THROW_TURN * float(s.facing) * into * back_home, 0.0, 0.0)
		"forward throw":
			return Vector4(THROW_FWD_PITCH * w, 0.0, THROW_FWD_STEP * w, 0.0)
		"down throw":
			return Vector4(THROW_DOWN_PITCH * w, 0.0, 0.0, -THROW_DOWN_DROP * w)
		"up throw":
			var rise := -THROW_UP_DIP * sin(PI * into) if f < release else THROW_UP_RISE * home * (1.0 - smoothstep(release, release + 12.0, f) * 0.5)
			return Vector4(0.0, 0.0, 0.0, rise)
	return Vector4.ZERO


## The tucked curl (round 8, the owner's pick B, 2026-10-28, of three mocked options): where the references flip or turn the whole body in
## an up air or a back air, ours curls forward over the hit (head down, knees up) by the first hit, holds it through the first half of the
## end lag and uncurls, stopping well short of a flip and never tipped back under an overhead hit. Degrees of the curl, by class: the
## swordsman's up and back airs (Longsword), the broadsword's turning back slash (Greatsword), the long blade's tumbling up slash (Exile);
## the ninja's upside-down up kick and turned back kick (Needles), the same two of the Phantom's reference (less: at 70 and 45 its knife
## hand came by the face for 5 frames); the dinosaur's flat tail flip
## (Clutch). The other classes' up and back airs stay upright in their references. (Eased with the reach alone, it showed for a frame or
## two; the references hold theirs.)
const AIR_TUCK := {
	0: {"uair": 55.0, "bair": 40.0},
	6: {"bair": 40.0},
	13: {"uair": 55.0},
	2: {"uair": 70.0, "bair": 45.0},
	12: {"uair": 55.0, "bair": 25.0},
	8: {"uair": 80.0},
}


func _air_tuck(s: Dictionary) -> float:
	if not use_base_body or String(s.state) != "Attack" or int(s.platform) >= 0 or r8_off.has("tuck"):
		return 0.0
	var deg: float = (AIR_TUCK.get(_cls(s), {}) as Dictionary).get(String(s.move_name), 0.0)
	if deg == 0.0:
		return 0.0
	var t: PackedInt32Array = s.move_timing
	var f := float(s.state_frame)
	var total := maxf(2.0, float(t[0]))
	var first := clampf(float(t[1]), 1.0, total)
	var last := clampf(float(t[2]), first, total)
	var home := 1.0 - smoothstep(last + 2.0, last + 2.0 + (total - last) * 0.5, f)
	return deg * smoothstep(0.0, first, f) * home


## The special states read in the whole body (round 8, review item 34): on this body the raised arms of the special fall hide behind
## the head and buried shows only the head, so the state is carried by the body's swing, which reads at the match camera. Dazed after a
## shield break, the body wobbles round in a slow circle under the stars; in the special fall it slumps forward, limp, and rocks slowly;
## buried, the head waggles side to side as it struggles. Returns [pitch, roll] in degrees (pitch forward, roll about the body's front).
const DAZE_LEAN := 6.0
const DAZE_ROLL := 9.0
const DAZE_FRAMES := 80.0
const HELPLESS_SLUMP := 8.0
const HELPLESS_ROCK := 8.0
const HELPLESS_FRAMES := 50.0
const BURIED_ROLL := 8.0
const BURIED_FRAMES := 24.0


func _state_body(s: Dictionary) -> Vector2:
	if not use_base_body or r8_off.has("states"):
		return Vector2.ZERO
	var f := float(s.frame)
	var settle := smoothstep(0.0, 16.0, float(s.state_frame))
	match String(s.state):
		"ShieldBreak":
			if int(s.platform) >= 0:
				var a := TAU * f / DAZE_FRAMES
				return Vector2(DAZE_LEAN * sin(a), DAZE_ROLL * cos(a)) * settle
		"Helpless":
			if int(s.platform) < 0:
				var a := TAU * f / HELPLESS_FRAMES
				return Vector2(HELPLESS_SLUMP + HELPLESS_ROCK * sin(a), 0.5 * HELPLESS_ROCK * sin(a * 0.7)) * settle
		"Buried":
			return Vector2(0.0, BURIED_ROLL * sin(TAU * f / BURIED_FRAMES)) * settle
	return Vector2.ZERO


func _pivot_spin(s: Dictionary) -> float:
	if String(s.state) != "Attack" or String(s.move_name) != "pivot grab":
		return 0.0
	var first := maxf(1.0, float(s.move_timing[1]))
	var k := clampf(float(s.state_frame) / first, 0.0, 1.0)
	return PIVOT_SPIN * float(s.facing) * (1.0 - k * k * (3.0 - 2.0 * k))


## Whether a move of class `cls` is done with the body (a kick, a grab, a throw, a landing hit) rather than the weapon.
func _body_clip(cls: int, move: String) -> bool:
	var clip := AnimLibrary.attack_clip(cls, move, anim)
	return AnimLibrary.is_kick(clip) or AnimLibrary.is_body(clip) or clip in ["grab", "throw", "land", "crouch"]


## Two-bone arm IK: the weapon arm reaches `hand_target` (model space), and with `two_hands` the other arm reaches `off_hand_target` (the
## greatsword's grip). Returns nothing; with `holding` false the arms go back to the clip.
## A hand nearer the head's middle than HEAD_CLEAR_R (units, about the drawn head's half width) is drawn back to HEAD_BEHIND_Z in depth;
## the head's middle is HEAD_CENTRE_UP above the head bone (the neck).
const HEAD_CLEAR_R := 0.55
const HEAD_BEHIND_Z := -0.35
## (Round 8) Where a weapon hand by the face is kept instead: this share of HEAD_CLEAR_R out past the drawn face, and this share of it
## below the head's middle.
const FRONT_CLEAR := 0.9
const FRONT_BELOW := 0.55
const HEAD_CENTRE_UP := 0.36
## How far ahead of the head's middle the drawn face is, side-on (forward space).
const FACE_FWD := 0.3
## How many sim frames an arm's reach takes to let go when it ends, and at most how many a move's reach takes to come in (fewer when the
## move strikes sooner); the reach's weight while letting go, and the targets it lets go from.
const IK_FADE_FRAMES := 6.0
const IK_EASE_IN := 3.0
var ik_fade := 0.0
var ik_last_hand := Vector3.ZERO
var ik_last_off := Vector3.ZERO
var ik_last_two := false


func _aim_arm(facing: int, holding: bool, two_hands := false, amount := 1.0) -> void:
	if skeleton == null:
		return
	arms_held.clear()
	for side in ["L", "R"]:
		var mine: bool = side == _weapon_side(facing)
		var iu := skeleton.find_bone("armU." + side)
		var il := skeleton.find_bone("armL." + side)
		var ih := skeleton.find_bone("hand." + side)
		if not holding or not (mine or two_hands):
			skeleton.set_bone_global_pose_override(iu, Transform3D(), 0.0, false)
			skeleton.set_bone_global_pose_override(il, Transform3D(), 0.0, false)
			skeleton.set_bone_global_pose_override(ih, Transform3D(), 0.0, false)
			continue
		arms_held.append(side)
		_ik_arm(side, mine, hand_target if mine else off_hand_target, amount)


## The sides whose arm the IK placed this frame (`_aim_arm`).
var arms_held: Array = []


## Turns the blade up about its hilt, keeping its length, just enough that its tip clears the floor (`GROUND_CLEARANCE`), measured in the
## world: the body's lean and squash carry the blade's frame with them.
func _lift_tip_off_floor() -> void:
	var tip_x := _blade_tip_x(_cls_now)
	var tip_y := func() -> float: return (blade_pivot.to_global(Vector3(tip_x, 0.0, 0.0)) - global_position).y
	var hilt_y := (blade_pivot.global_position - global_position).y
	if hilt_y <= GROUND_CLEARANCE or tip_y.call() >= GROUND_CLEARANCE:
		return
	# (A bisection between where it points and well up ahead, the way it already points: the least turn that clears. The frame leans
	# with the body, so its level is not the world's.)
	var low := blade_pivot.rotation.z
	var high := 1.2 if cos(low) >= 0.0 else PI - 1.2
	# (Round 8, the owner's Longsword note: the rotation reads back wrapped into -180..180, so a blade trailing back and down (214, read
	# as -146) was turned toward the high mark the long way round, through straight ahead: it swung forward under the body. Unwrapped onto
	# the high mark's side first, it turns the short way, up behind.)
	low = wrapf(low, 0.0, TAU) if cos(low) < 0.0 else wrapf(low, -PI, PI)
	for _i in 12:
		blade_pivot.rotation.z = (low + high) * 0.5
		if tip_y.call() < GROUND_CLEARANCE:
			low = blade_pivot.rotation.z
		else:
			high = blade_pivot.rotation.z
	blade_pivot.rotation.z = high


## How far a hand at `hand` (forward space) should go behind the head in depth: 1 over the face's middle, easing to 0 past its edge.
## `fwd` moves the middle measured from forward toward the drawn face (round 4: side-on the face is drawn on the front of the head, so a
## hilt held up just ahead of the head's middle was over the eyes while it measured clear).
func _behind_head_k(hand: Vector2, facing: int, fwd := 0.0) -> float:
	var hp := skeleton.get_bone_global_pose(head_bone)
	var hc3 := _skeleton_to_frame() * (hp.origin + hp.basis.y.normalized() * HEAD_CENTRE_UP)
	var near := (hand - Vector2(hc3.x * facing + fwd, hc3.y)).length() / HEAD_CLEAR_R
	return 1.0 - smoothstep(0.75, 1.15, near)


## How near a weapon's shaft (forward space, from a little behind the hand to the tip) passes to the drawn face, as the weight for drawing
## it behind the head (1 across the face, easing to 0 a little past its edge), round 8.
func _shaft_face_k(hand: Vector2, tip: Vector2, facing: int) -> float:
	var hp := skeleton.get_bone_global_pose(head_bone)
	var hc3 := _skeleton_to_frame() * (hp.origin + hp.basis.y.normalized() * HEAD_CENTRE_UP)
	var face := Vector2(hc3.x * facing + FACE_FWD, hc3.y)
	var seg := tip - hand
	var from := hand - seg.normalized() * SHAFT_BEHIND_HAND
	seg = tip - from
	# (From the drawn face and from the head's middle, the nearer: turned toward the camera, the face reaches back past FACE_FWD.)
	var near := INF
	for p in [face, Vector2(hc3.x * facing, hc3.y)]:
		var u := clampf((p - from).dot(seg) / maxf(seg.length_squared(), 0.0001), 0.0, 1.0)
		near = minf(near, (from + seg * u - p).length() / HEAD_CLEAR_R)
	return 1.0 - smoothstep(SHAFT_FACE_IN, SHAFT_FACE_OUT, near)


## The shaft's reach of the face (shares of HEAD_CLEAR_R): behind the head from the drawn face's own size, eased out by a little past it (at
## the head's whole size, 0.75-1.15, a staff held level at the chest went behind the body and vanished from the forward smash's strike).
const SHAFT_FACE_IN := 0.6
const SHAFT_FACE_OUT := 0.95


## How far the staff's foot runs past the weapon hand (world units).
const SHAFT_BEHIND_HAND := 0.3


## A hand covering the drawn face, whatever put it there (a clip's free arm, an overhead reach, a two-handed grip), is moved out to the
## face's edge on screen and back toward the head, by the arm's IK (round 2, the real-speed audit: hands crossed the face in 2-50 frames of
## every class's up moves, raised specials and two-handed swings, `PF_FACE_COVER`). Eased in by how near the face it is, so it never pops.
const FACE_GUARD_R := 0.45
## How far out (a share of FACE_GUARD_R) a hand that fell short of the face's edge is sent the second time.
const GUARD_FURTHER := 1.5


func _guard_face(facing: int, on_ground := true) -> void:
	if not use_base_body or skeleton == null or head_bone < 0:
		return
	var to_frame := _skeleton_to_frame()
	var hp := skeleton.get_bone_global_pose(head_bone)
	var centre := to_frame * (hp.origin + hp.basis.y.normalized() * 0.3)
	for side in ["L", "R"]:
		var ih := skeleton.find_bone("hand." + side)
		var p := to_frame * skeleton.get_bone_global_pose(ih).origin
		if p.z <= centre.z:
			continue
		# (Round 5: a sword hand is measured from the drawn face too, FACE_FWD ahead of the head's middle, whichever is nearer: the
		# Longsword's guard came up under the mouth after its neutral air's windmill while it measured clear of the head's middle. Not the
		# staff's two-handed grip, which this pushed onto the face.)
		var from := Vector2(centre.x, centre.y)
		# (And the Dagger's knife hand on the ground, round 5: held up past the cheek in its forward tilt, 6 frames. Not in the air: its
		# forward air holds the knife beside the jaw, and sent out it went behind the head and the knife vanished.)
		if (bladed(_cls_now) or (_cls_now == DAGGER_CLASS and on_ground)) and side == _weapon_side(facing):
			var face := from + Vector2(FACE_FWD * float(facing), 0.0)
			if (Vector2(p.x, p.y) - face).length() < (Vector2(p.x, p.y) - from).length():
				from = face
		var near := (Vector2(p.x, p.y) - from).length() / FACE_GUARD_R
		var k := 1.0 - smoothstep(0.8, 1.2, near)
		if k <= 0.0:
			continue
		# (Out past the face's edge on screen and half way back toward the head: the short arms cannot reach behind the big head at
		# face height, so pushed straight back the hand stopped in front of it.)
		var out := Vector2(p.x, p.y) - from
		out = out.normalized() if out.length() > 0.01 else Vector2(float(facing), 1.0).normalized()
		var edge := from + out * FACE_GUARD_R * 1.05
		var guard := Vector3(edge.x, edge.y, lerpf(p.z, centre.z + HEAD_BEHIND_Z, 0.5))
		var mine: bool = side == _weapon_side(facing)
		if arms_held.has(side):
			_ik_arm(side, mine, p.lerp(guard, k), 1.0)
		else:
			_ik_arm(side, mine, guard, k)
		# (Round 5: an arm too short to reach the face's edge stopped just inside it, a fist at the chin: the Settler's forward tilt, held
		# 8 frames. Then it is sent further out the same way, and further back in depth, once more.)
		var got := to_frame * skeleton.get_bone_global_pose(ih).origin
		if k > 0.5 and (Vector2(got.x, got.y) - from).length() < FACE_GUARD_R * 0.95:
			var further := from + out * FACE_GUARD_R * GUARD_FURTHER
			var again := Vector3(further.x, further.y, lerpf(got.z, centre.z + HEAD_BEHIND_Z, 0.75))
			if arms_held.has(side):
				_ik_arm(side, mine, p.lerp(again, k), 1.0)
			else:
				_ik_arm(side, mine, again, k)


## Raised arms beside the head, not over it (round 8, the owner 2026-10-28, after the shoulder mock: keep the shoulders, raise the arms
## out to the side). The short arm cannot reach above the big head, so a hand raised above its shoulder ended inside the head's outline on
## screen, behind it, and the raise did not read (the Settler's up air, the special fall, the item throw, the swords' overhead swings).
## Such a hand is carried out sideways on screen to BESIDE_OUT of the head's radius from its middle, ahead or behind as it already leans
## (the weapon hand ahead, the free hand behind when it is square under the head: a V), as high as the arm can reach there, eased in by
## how far inside the outline and how far above the shoulder it is. A weapon hand stays no higher than the chin (the 8c rule); the
## weapon follows the hand, so a sword raised overhead arcs over the head from beside it with the fist showing.
const BESIDE_OUT := 1.4
## How far the hand may reach (a share of the arm's full length) when it is placed beside the head.
const BESIDE_REACH := 0.95


func _arms_beside_head(facing: int) -> void:
	if not use_base_body or skeleton == null or head_bone < 0:
		return
	var to_frame := _skeleton_to_frame()
	var hp := skeleton.get_bone_global_pose(head_bone)
	var c := to_frame * (hp.origin + hp.basis.y.normalized() * HEAD_CENTRE_UP * hp.basis.y.length())
	var r: float = float((_base_rig_info().get("head_ball", [0.0, 0.0, 0.36]) as Array)[2]) * (to_frame.basis * hp.basis).x.length()
	for side in ["L", "R"]:
		var ih := skeleton.find_bone("hand." + side)
		var iu := skeleton.find_bone("armU." + side)
		var il := skeleton.find_bone("armL." + side)
		var p := to_frame * skeleton.get_bone_global_pose(ih).origin
		var sh := to_frame * skeleton.get_bone_global_pose(iu).origin
		var inside := Vector2(p.x - c.x, p.y - c.y).length() / r
		var k := (1.0 - smoothstep(0.95, 1.2, inside)) * smoothstep(0.03, 0.12, p.y - sh.y)
		# (Only a hand behind the head's middle in depth, which the head hides: one in front of it is the face guard's, and carried out
		# beside the head it came up across the face, the Settler's tool hand.)
		if k <= 0.0 or p.z > c.z:
			continue
		var mine: bool = side == _weapon_side(facing)
		var off := p.x - c.x
		var sgn := signf(off) if absf(off) > 0.06 else (float(facing) if mine else -float(facing))
		var reach := ((skeleton.get_bone_global_rest(il).origin - skeleton.get_bone_global_rest(iu).origin).length()
			+ (skeleton.get_bone_global_rest(ih).origin - skeleton.get_bone_global_rest(il).origin).length() + 0.07)
		reach *= BESIDE_REACH * (to_frame.basis.x.length())
		# (Carried out ahead, a weapon hand the arm could not take far enough stopped by the drawn face: then it goes out behind the head
		# instead, the face kept clear first.)
		for attempt in 2:
			var tx := c.x + sgn * r * BESIDE_OUT
			var dx := clampf(tx - sh.x, -reach * 0.98, reach * 0.98)
			var ty := minf(p.y, sh.y + sqrt(maxf(0.0, reach * reach - dx * dx)))
			if mine and (bladed(_cls_now) or _cls_now == STAFF_CLASS):
				ty = minf(ty, c.y - r * FRONT_BELOW)
			# (Behind the head's middle in depth, so it is never carried across the face.)
			var target := Vector3(sh.x + dx, maxf(ty, sh.y + 0.04), minf(p.z, c.z - 0.05))
			if arms_held.has(side):
				_ik_arm(side, mine, p.lerp(target, k), 1.0)
			else:
				_ik_arm(side, mine, target, k)
			var got := to_frame * skeleton.get_bone_global_pose(ih).origin
			var face := Vector2(c.x + FACE_FWD * float(facing), c.y - r * 0.2)
			if attempt == 1 or sgn != float(facing) or (Vector2(got.x, got.y) - face).length() > COVER_R:
				break
			sgn = -float(facing)


## Two-bone IK for one arm: the hand to `target` (stage frame), blended in by `amount` over the clip's pose.
func _ik_arm(side: String, mine: bool, target: Vector3, amount: float) -> void:
		var iu := skeleton.find_bone("armU." + side)
		var il := skeleton.find_bone("armL." + side)
		var ih := skeleton.find_bone("hand." + side)
		var rest_u := skeleton.get_bone_global_rest(iu)
		var rest_l := skeleton.get_bone_global_rest(il)
		var rest_h := skeleton.get_bone_global_rest(ih)
		var elbow0 := rest_l.origin
		var wrist0 := rest_h.origin
		# On a strike the arm stretches (`strike_e`), and reaches that much further. (A blade's arm too on the base body, whose arm raised
		# overhead ended at the big head's brow and held the sword at the face: the arm extends to the swing, never the body. The owner,
		# 2026-10-17.)
		var e := _arm_e() if not bladed(_cls_now) or use_base_body else 0.0
		var upper := (elbow0 - rest_u.origin).length() * (1.0 + e)
		# The shoulder where the animated chest has it now, so the arm stays on the body as it bends and turns.
		var shoulder := rest_u.origin
		var parent := skeleton.get_bone_parent(iu)
		if parent >= 0:
			shoulder = skeleton.get_bone_global_pose(parent) * skeleton.get_bone_rest(iu).origin
		var fore_len := (wrist0 - elbow0).length() * (1.0 + e)
		var lower := fore_len + 0.07
		var to_skel := _skeleton_to_frame().affine_inverse()
		var goal: Vector3 = to_skel * target
		var d := goal - shoulder
		var dist := clampf(d.length(), 0.05, upper + lower - 0.002)
		var dir := d.normalized()
		var cos_a := clampf((upper * upper + dist * dist - lower * lower) / (2.0 * upper * dist), -1.0, 1.0)
		var ang := acos(cos_a)
		# The elbow points down and out to the side, as a person's does reaching forward. (It pointed a little back: on a reach in front the
		# upper arm ran back behind the torso while the forearm came forward to the hand, an arm behind and in front at once. Art review 5.)
		var pole := Vector3(0.8 if side == "R" else -0.8, -1.0, 0.1)
		var bend := (pole - dir * pole.dot(dir)).normalized()
		var elbow := shoulder + dir * (cos(ang) * upper) + bend * (sin(ang) * upper)
		var fore := (goal - elbow).normalized()
		# A human elbow (the owner's rule, art review 5): the arm turns as a hinge. The upper arm is twisted so the elbow's hinge lies across
		# the plane of the bend, and the forearm folds only forward about that hinge, toward the inside of the elbow, never sideways or
		# backward. (Turned by the shortest rotation, the upper arm's twist was left to chance, and the elbow's fold could face the wrong way.)
		var q_upper: Quaternion
		var q_lower: Quaternion
		var hinged := _arm_hinge(elbow0 - rest_u.origin, wrist0 - elbow0, elbow - shoulder, goal - elbow)
		if hinged.is_empty():
			q_upper = Quaternion((elbow0 - shoulder).normalized(), (elbow - shoulder).normalized())
			q_lower = Quaternion((wrist0 - elbow0).normalized(), fore)
		else:
			q_upper = hinged[0]
			q_lower = hinged[1]
		var stretch := Basis.from_scale(MotionDeform.limb_scale(e))
		skeleton.set_bone_global_pose_override(iu, Transform3D(Basis(q_upper) * rest_u.basis * stretch, shoulder), amount, true)
		# (The class's arm bulk carried into the override, which replaces the pose's own scale.)
		var bulk: Array = _arm_bulk(_cls_now)
		skeleton.set_bone_global_pose_override(il, Transform3D(Basis(q_lower) * rest_l.basis * stretch * Basis.from_scale(bulk[0]), elbow), amount, true)
		skeleton.set_bone_global_pose_override(ih, Transform3D(Basis(q_lower) * rest_h.basis * Basis.from_scale(bulk[1]), elbow + fore * fore_len * float((bulk[0] as Vector3).y)), amount, true)
		if not bladed(_cls_now) and mine:
			strike_tip = to_local(skeleton.global_transform * (elbow + fore * fore_len))
			strike_tip_ok = true


## The rotations that take an arm from its rest pose to a bend (skeleton space; the arm's rest frame is the model's own), as a hinge:
## [upper, lower]. `up0`/`fore0` are the rest upper arm and forearm, `up1`/`fore1` the wanted ones. The hinge axis is the model's x at
## rest (a positive turn about it folds the forearm toward the front); at the wanted pose it is set across the plane of the bend, on the
## side that makes the fold a forward one. Empty if the arm is nearly straight (the hinge is then free, and the shortest turn serves).
static func _arm_hinge(up0: Vector3, fore0: Vector3, up1: Vector3, fore1: Vector3) -> Array:
	var u0 := up0.normalized()
	var f0 := fore0.normalized()
	var u1 := up1.normalized()
	var f1 := fore1.normalized()
	var bend := u1.cross(f1)
	if bend.length() < 0.08:
		return []
	# The rest hinge: the model's x, square to the upper arm; and which way round a forward fold turns about it.
	var h0 := (Vector3.RIGHT - u0 * Vector3.RIGHT.dot(u0)).normalized()
	# The bend's axis, on whichever side lies nearer where the arm's own hinge goes by the shortest turn: the other side twists the upper
	# arm half a turn about its length, which broke the shoulder of the right arm (the weapon arm facing left). Art review 6.
	var h1 := bend.normalized()
	var carried := Quaternion(u0, u1) * h0
	if h1.dot(carried) < 0.0:
		h1 = -h1
	var rest_u := Basis(h0, u0, h0.cross(u0))
	var want_u := Basis(h1, u1, h1.cross(u1))
	var h0f := (h0 - f0 * h0.dot(f0)).normalized()
	var h1f := (h1 - f1 * h1.dot(f1)).normalized()
	var rest_f := Basis(h0f, f0, h0f.cross(f0))
	var want_f := Basis(h1f, f1, h1f.cross(f1))
	return [(want_u * rest_u.inverse()).get_rotation_quaternion(), (want_f * rest_f.inverse()).get_rotation_quaternion()]


## A leg's rotations from rest to a bend as a hinge, as `_arm_hinge` does for the arm: [thigh, shin] (skeleton space). The knee's axis is the
## model's x at rest; at the wanted pose it lies across the bend (`hint`, the bend's own axis, when the leg is nearly straight), on the side
## nearer where the rest axis is carried, so neither bone twists about its length.
static func _leg_hinge(up0: Vector3, low0: Vector3, up1: Vector3, low1: Vector3, hint: Vector3) -> Array:
	var u0 := up0.normalized()
	var l0 := low0.normalized()
	var u1 := up1.normalized()
	var l1 := low1.normalized()
	var h1 := u1.cross(l1)
	if h1.length() < 0.08:
		h1 = hint - u1 * hint.dot(u1)
		if h1.length() < 0.01:
			return []
	h1 = h1.normalized()
	var h0 := (Vector3.RIGHT - u0 * Vector3.RIGHT.dot(u0)).normalized()
	if h1.dot(Quaternion(u0, u1) * h0) < 0.0:
		h1 = -h1
	var rest_u := Basis(h0, u0, h0.cross(u0))
	var want_u := Basis(h1, u1, h1.cross(u1))
	var h0l := (h0 - l0 * h0.dot(l0)).normalized()
	var h1l := (h1 - l1 * h1.dot(l1)).normalized()
	var rest_l := Basis(h0l, l0, h0l.cross(l0))
	var want_l := Basis(h1l, l1, h1l.cross(l1))
	return [(want_u * rest_u.inverse()).get_rotation_quaternion(), (want_l * rest_l.inverse()).get_rotation_quaternion()]


## About where the hips are (forward space, height), for how steeply a kick rises.
const KICK_HIP_Y := 0.6
## The steepest a kick rises on the base body (degrees above level, see `_aim_leg`).
const HIGH_KICK := 65.0
## Kicks that rise at least this far above level from the hip whatever their hit's height (round 8, review item 12): the Needles'
## forward tilt is the ninja's tall, straight high kick, the leg held up near head height; aimed at its chest-high hit it rose 32 degrees
## and read as a knee-high kick.
const KICK_RAISE := {2: {"ftilt": 50.0}}
## How far out from the hip a raised kick's foot is aimed at least (the leg straight).
const KICK_RAISE_REACH := 1.3
## How far ahead of the hips an overhead kick's foot is kept, so the leg rises past the face rather than across it (round 3).
const KICK_CLEAR_X := 0.8
## Where (depth, toward the camera positive) a kick behind and overhead puts its foot: behind the head.
const KICK_BEHIND_Z := -0.35
## The steepest (degrees above level, from the hip) an overhead kick is aimed, so the foot ends ahead of the chin, not on the face.
const OVERHEAD_KICK := 40.0


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
		var e := strike_e * MotionDeform.LEG_STRETCH_FACTOR * (MotionDeform.BASE_LEG_STRETCH if use_base_body else 1.0) * leg_k
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
		# (Round 4: a kick does not carry on past its hit round the hip as a raking hand does: carried up, the short leg folded across the
		# chest and chin for 8-17 frames of the low kicks and the up kicks. The body's lunge carries it on instead.)
		# On the base body a kick never rises steeper than HIGH_KICK above level: its short legs kicked straight up ended at the chest, in
		# front of the round belly, and the foot read as a second fist. Angled up and forward the foot ends up at head height clear of the
		# body, a high kick that reads (after the reference's high forward kicks: the owner, 2026-10-17).
		if use_base_body:
			var from_hip := Vector2(aim.x, aim.y - KICK_HIP_Y)
			var raise_to: float = (KICK_RAISE.get(_cls(s), {}) as Dictionary).get(String(s.move_name), -1.0)
			if raise_to > 0.0 and aim.x >= 0.0 and from_hip.angle() < deg_to_rad(raise_to) and not r8_off.has("kicks"):
				from_hip = Vector2.from_angle(deg_to_rad(raise_to)) * maxf(from_hip.length(), KICK_RAISE_REACH)
			var a := from_hip.angle()
			var limit := deg_to_rad(HIGH_KICK)
			if aim.x >= 0.0 and a > limit and a <= PI * 0.5:
				from_hip = Vector2.from_angle(limit) * from_hip.length()
			elif aim.x < 0.0 and a < PI - limit and a >= PI * 0.5:
				from_hip = Vector2.from_angle(PI - limit) * from_hip.length()
			aim = Vector3(from_hip.x, from_hip.y + KICK_HIP_Y, aim.z)
		# (A little toward the camera, so on the turned body the kicking leg passes in front of the torso instead of through it; less for a
		# high kick, where it outweighed the height and pointed the leg at the camera.)
		var rise := rad_to_deg(atan2(aim.y - KICK_HIP_Y, maxf(absf(aim.x), 0.001)))
		var toward := lerpf(0.75, 0.4, smoothstep(20.0, 50.0, rise))
		# (Round 3: a kick overhead, its foot above the chest and not far out in front, rises ahead of the face instead of across it; aimed
		# at the hit straight above, toward the camera, the leg crossed the face for 12 frames of the Gauntlets' up air, and sent behind
		# the head the short leg vanished. It reads as a high kick past the face, KICK_CLEAR_X ahead of the hips.)
		if use_base_body and aim.x >= -0.2:
			var overhead := smoothstep(1.0, 1.45, aim.y) * (1.0 - smoothstep(0.5, 0.9, absf(aim.x)))
			if overhead > 0.0:
				aim.x = lerpf(aim.x, maxf(aim.x, KICK_CLEAR_X), overhead)
				toward = lerpf(toward, 0.25, overhead)
				# (Round 4: and no steeper than OVERHEAD_KICK above level from the hip. The short leg cannot reach a hit overhead, so aimed
				# at it steeply it stopped at full stretch with the foot on the face: the Needles' up tilt and up air, 8 frames. Laid
				# lower, the foot ends out ahead of the chin, a high kick under the hit.)
				var fh := Vector2(aim.x, aim.y - KICK_HIP_Y)
				var top := deg_to_rad(lerpf(HIGH_KICK, OVERHEAD_KICK, overhead))
				if fh.angle() > top:
					fh = Vector2.from_angle(top) * fh.length()
					aim = Vector3(fh.x, fh.y + KICK_HIP_Y, aim.z)
		# (Round 5: a kick behind and up past the chest, the far leg's, goes behind the head in depth, as a raised hand does: drawn toward
		# the camera like a kick in front, its shin crossed the face through the Embers' up air, 9 frames, the most in live play.)
		if use_base_body and side == back and aim.x < 0.0:
			toward = lerpf(toward, KICK_BEHIND_Z, smoothstep(1.0, 1.4, aim.y))
		var goal: Vector3 = to_skel * Vector3(aim.x * float(facing), aim.y, toward)
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
		# (Round 7: a leg kicked out behind and up (a split kick's back leg, a back kick) keeps its knee pointing down, as a knee can; the
		# turn of the pole with the leg pointed it backward, and the knee folded the wrong way: the Needles' neutral air, 66% of its
		# frames, the Claws' 37%. And each knee turns a little out from the body, the near one toward the camera and the far one away: both
		# toward the camera crossed the far knee in front of the near leg.)
		var r7_legs := not r7_off.has("legs")
		if sin_phi < 0.0 and cos_phi < 0.0 and r7_legs:
			sin_phi = -1.0
			cos_phi = 0.0
		var out := 0.3 if side == front or not r7_legs else -0.15
		var pole_m := Vector3(float(facing) * cos_phi, sin_phi, out)
		var pole: Vector3 = (to_skel.basis * pole_m).normalized()
		var bend := (pole - dir * pole.dot(dir)).normalized()
		var knee := hip + dir * (cos(ang) * upper) + bend * (sin(ang) * upper)
		var shin_dir := (goal - knee).normalized()
		# (Round 7: as a hinge, the thigh and shin turned about one knee axis across the bend, as the arm's elbow is (`_arm_hinge`). Each
		# turned by its own shortest rotation, the shin was twisted on the thigh by up to 176 degrees (the Clutch's neutral air; the Needles'
		# forward smash 94, the Claws' up air 66) and the knee bent off its hinge, and the foot rolled with it.)
		var q_upper := Quaternion((knee0 - rest_t.origin).normalized(), (knee - hip).normalized())
		var q_lower := Quaternion((ankle0 - knee0).normalized(), shin_dir)
		var hinged := _leg_hinge(knee0 - rest_t.origin, ankle0 - knee0, knee - hip, shin_dir, dir.cross(bend))
		if not hinged.is_empty() and r7_legs:
			q_upper = hinged[0]
			q_lower = hinged[1]
		var stretch := Basis.from_scale(MotionDeform.limb_scale(e))
		skeleton.set_bone_global_pose_override(it, Transform3D(Basis(q_upper) * rest_t.basis * stretch, hip), leg_k, true)
		skeleton.set_bone_global_pose_override(ish, Transform3D(Basis(q_lower) * rest_s.basis * stretch, knee), leg_k, true)
		skeleton.set_bone_global_pose_override(ift, Transform3D(Basis(q_lower) * rest_f.basis * Basis.from_scale(Vector3.ONE * (FOOT_SIZE if use_base_body else 1.0)), knee + shin_dir * shin_len), leg_k, true)
		if side == kicker:
			strike_tip = to_local(skeleton.global_transform * (knee + shin_dir * shin_len))
			strike_tip_ok = true


# ---- Swing arcs --------------------------------------------------------------------------------------------------------------------
# (Since the paper effects pass, 2026-10-21, the owner: until the animation overhaul each arc follows the move's hitboxes, not the
# animated blade or hand. Its circle keeps the authored centre and sweep, but its radius is the live hitboxes' outer edge and its head
# goes where the furthest hitbox is, never back, and on at least an even pace through the active frames, so the arc shows where the hit
# reaches whatever the pose does. `_hit_edge`.)
# A swing leaves the crescent the reference game draws: one smooth arc where the weapon's tip went, pointed at both ends and thickest a
# little behind the leading edge, a bright core along the outside fading through the trail colour to nothing on the inside. Each arc is
# authored per move (`ARCS`; the sword's are fitted to its hitboxes, the claws' to their animations): a circle about the shoulder, the
# angles it sweeps and the active frames it is drawn on. It grows with the hit, from the frame before the first active frame to the last,
# then fades from the tail over `ARC_FADE` frames. It depends only on the move and its frame, so hitlag freezes it with the swing and a
# replay seek redraws it; a move cut short (a landing) lets its arc fade on its own. Moves with no arc (thrusts, punches, kicks) draw
# none.

const ARC_FADE := 3
## How much of the swing the arc shows behind its head (degrees), how thick it is (a share of its radius), and the most the hitboxes may
## grow its radius past the authored circle's. (After the reference game's sword trails, the owner, 2026-10-21.)
const ARC_TAIL_DEGREES := 110.0
const ARC_WIDTH := 0.22
const ARC_MAX_GROW := 1.15
## The highest a sword arm points (degrees above forward) when the blade goes up overhead.
const HIGH_BLADE_ARM := 62.0
## How see-through the swing arcs and smears are: painted, but a trail of light, not a solid shape (the fighter shows through it).
## (0.62, art review 6: at 0.82 the pale crescent was the biggest, most solid shape on screen at the moment of a hit, over both fighters.)
const ARC_OPACITY := 0.62
const ARC_STEPS := 36
const ARC_Z := 0.6
## (Animation overhaul pass 5: the Claws class now wears modelled claws, so its arcs run at the claw tips, about this far past the wrist's
## reach; at 0.8 they floated well clear of the bare fist.)
const ARC_PAST_HAND := 0.6
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
		# (Ends level and a little down, as the reference's cut does, not into the floor at -45.)
		"fsmash": [[0.7, 1.15, 2.3, 135, -22, 10, 13]],
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
		"fair": [[0.0, 0.0, 0.0, 110, -60, 7, 9, "arm"]],
		"bair": [[0.0, 0.0, 0.0, 215, 140, 13, 15, "arm"]],
		"uair": [[0.0, 0.0, 0.0, 15, 165, 7, 9, "arm"]],
		"dair": [[0.0, 0.0, 0.0, 20, -110, 16, 17, "arm"]],
	},
	# Champion (greatsword): fitted to the moveset's timings (docs/movesets/greatsword/IMPLEMENTATION_REPORT.md), circles about the shoulder
	# a little wider than the longsword's for the longer blade. Its kicks (jabs 1 and 2, the down tilt) and the driven-down down air
	# draw none.
	6: {
		"jab 3": [[0.5, 1.15, 2.4, 70, -25, 6, 7]],
		"ftilt": [[0.45, 1.1, 2.45, 95, -35, 9, 10]],
		"utilt": [[0.0, 1.2, 2.3, 200, 10, 6, 9]],
		"dash attack": [[0.55, 1.1, 2.5, -60, 45, 9, 12]],
		"fsmash": [[0.7, 1.15, 2.4, 120, -15, 18, 19], [0.7, 1.15, 2.4, -40, 35, 23, 24], [0.7, 1.15, 2.5, 140, -25, 27, 28]],
		"usmash": [[0.0, 1.2, 2.4, 15, 195, 12, 16]],
		"dsmash": [[0.0, 2.7, 2.7, -100, -35, 8, 9], [0.0, 2.7, 2.7, -80, -145, 21, 23]],
		"nair": [[0.2, 1.1, 2.35, 10, 370, 5, 15]],
		"fair": [[0.5, 1.05, 2.4, 125, -60, 17, 20]],
		"bair": [[0.0, 1.1, 2.3, 215, 140, 11, 12]],
		"uair": [[0.1, 1.3, 2.3, 175, 60, 7, 10]],
		"ledge attack": [[0.3, 0.8, 2.3, 120, -20, 20, 22]],
	},
	# Wanderer (Buckler, pass 14): its sword's swings, which drew no trail at all (the class had no entry), fitted to its hitboxes'
	# angles about the shoulder, a little smaller than its hitboxes reach (its sword is shorter than they are), on its active frames
	# (content/base.pfc) put two later, so the trail starts as the blade arrives instead of ahead of it. Its thrusts (jab 3, up and down
	# airs), kicks (neutral and back airs) and the spin (the body turns, SPIN_MOVES) draw none.
	9: {
		"jab": [[0.3, 1.1, 2.0, 60, -15, 8, 9]],
		"jab 2": [[0.3, 1.1, 2.0, -35, 25, 6, 7]],
		"ftilt": [[0.3, 1.1, 2.1, 110, -15, 17, 21]],
		"utilt": [[0.2, 1.15, 2.1, 165, 55, 9, 13]],
		"dtilt": [[0.3, 1.1, 2.0, 10, -55, 11, 12]],
		"dash attack": [[0.3, 1.1, 2.1, 95, -15, 22, 25]],
		"fsmash": [[0.5, 1.15, 2.2, 120, -12, 19, 20]],
		"dsmash": [[0.0, 2.6, 2.4, -100, -35, 13, 14], [0.0, 2.6, 2.5, -80, -150, 25, 26]],
		"fair": [[0.4, 1.05, 2.1, 130, -20, 17, 18]],
	},
	# Nodachi (pass 10): its long blade's swings, circles about the shoulder wider than the longsword's for the longer blade, on its own
	# active frames (content/base.pfc). The level cuts (forward tilt, forward smash) sweep across in front; the plunging down air draws
	# none.
	13: {
		"jab": [[0.5, 1.15, 2.1, 70, -15, 4, 6]],
		"ftilt": [[0.45, 1.15, 2.2, 35, -20, 13, 15]],
		"fsmash": [[0.7, 1.15, 2.6, 110, -15, 23, 24]],
		"dash attack": [[0.55, 1.1, 2.2, -60, 40, 13, 18]],
		"fair": [[0.5, 1.05, 2.2, 140, -60, 12, 17]],
		"bair": [[0.0, 1.1, 2.1, 260, 100, 14, 15]],
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

## The Needles' Flash Step: the fighter is gone (move frames, from 0) while it teleports, between the explosion and the reappearance.
const FLASH_STEP_GONE := [36, 54]
## The Seer's Warp: gone (move frames) while it travels, back on the next.
const WARP_GONE := [16, 32]
var arc: MeshInstance3D
## The arc drawn now: {"w": its ARCS entry, "k": how far it has grown 0..1, "q": how far it has faded 0..1, "facing", "cls"}, or empty.
var arc_shown := {}


func _update_arc(now: Dictionary, delta: float) -> void:
	if not now.is_empty() and float(now.q) < 1.0:
		# (A sword arc keeps where the blade's tip last put it, `_blade_arc_on_tip`, through its fade.)
		if not arc_shown.is_empty() and arc_shown.w == now.w:
			for key in ["c", "r", "head", "z"]:
				if arc_shown.has(key):
					now[key] = arc_shown[key]
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
		var same: bool = not arc_shown.is_empty() and arc_shown.w == w
		var prev := float(arc_shown.k) if same else 0.0
		var r := float(arc_shown.get("r", 0.0)) if same else 0.0
		var centre: Vector2 = _arc_circle(w)[0]
		var edge := _hit_edge(s, centre)
		if not edge.is_empty():
			# The arc reaches the hitboxes' outer edge (the furthest yet in this swing), and its head is where that edge is.
			# (No further than a little past the authored circle, so one far hitbox never makes a giant arc.)
			r = minf(maxf(r, float(edge[1])), float(_arc_circle(w)[1]) * ARC_MAX_GROW)
			var from := deg_to_rad(float(w[3]))
			var span := deg_to_rad(float(w[4])) - from
			var d := wrapf((float(edge[0]) - from) * signf(span), -PI, PI)
			# Of the two ways round, the one nearer where the head was (a long arc, like the neutral air's, goes past half a turn).
			if absf(d + TAU - prev * absf(span)) < absf(d - prev * absf(span)):
				d += TAU
			k = maxf(k, clampf(d / absf(span), 0.0, 1.0))
		k = maxf(prev, k)
		out = {"w": w, "k": k, "q": (f - last) / ARC_FADE, "facing": float(s.facing), "cls": _cls(s), "r": r}
	return out


## Where the live hitboxes reach as seen from `centre` (forward space): [angle, distance] of the furthest hitbox's middle (on a sword,
## the tip), or empty when no hitbox is live. (Its middle, not its outer edge: the reference game's trail follows the blade's tip, and
## the hitboxes reach well past it; at their outer edge the arcs were far too big, the owner, 2026-10-21.)
func _hit_edge(s: Dictionary, centre: Vector2) -> Array:
	var hits: PackedFloat32Array = s.get("hits", PackedFloat32Array())
	var pos: Vector2 = s.get("pos", Vector2.ZERO)
	var facing := float(s.facing)
	var best := -1.0
	var ang := 0.0
	for k in range(0, hits.size(), 4):
		var rel := Vector2((hits[k] - pos.x) * facing, hits[k + 1] - pos.y) - centre
		var reach := rel.length()
		if reach > best:
			best = reach
			ang = rel.angle()
	return [] if best < 0.0 else [ang, best]


## Where an arc's head is (forward space).
func _arc_point(now: Dictionary, carry := 0.0) -> Vector2:
	var w: Array = now.w
	var ang := lerpf(deg_to_rad(float(w[3])), deg_to_rad(float(w[4])), float(now.k))
	# (Round 4: past the last hit the hand carries on round the arc, `carry` radians, rather than stopping dead at its end.)
	ang += carry * signf(float(w[4]) - float(w[3]))
	var g := _arc_circle(w)
	return (g[0] as Vector2) + Vector2.from_angle(ang) * _arc_radius(now)


## An arc's radius now: the hitboxes' outer edge once one has been live, the authored circle's before.
func _arc_radius(now: Dictionary) -> float:
	var r := float(now.get("r", 0.0))
	return r if r > 0.0 else float(_arc_circle(now.w)[1])


## An arc's circle: [centre, radius].
func _arc_circle(w: Array) -> Array:
	if w.size() > 7 and w[7] == "arm":
		return [shoulder, arm_reach + ARC_PAST_HAND]
	return [Vector2(float(w[0]), float(w[1])), float(w[2])]


## The blade's tip as the mesh has it (the blade mesh ends 2.9 along its pivot's x).
const BLADE_TIP_X := 2.9
## (Round 8, the owner 2026-10-29: "the sword itself is a bit too long".) The modelled blades' own tips by class, fitted to the references'
## blade against their height (art/blender/make_weapons.py LONGSWORD_END, GREATSWORD_END): the Longsword's and the Wanderer's 1.64, the
## Greatsword's 2.55; others as the mesh. The swing arcs stay on the hit's edge, BLADE_TIP_X along the blade (the hit reaches past the
## shorter blade, as the swordsman reference's tipper does), so they carry the reach.
const BLADE_TIP_X_BY := {0: 1.64, BUCKLER_CLASS: 1.64, GREATSWORD_CLASS: 2.55}


func _blade_tip_x(cls: int) -> float:
	if not use_base_body or r8_off.has("length"):
		return BLADE_TIP_X
	return float(BLADE_TIP_X_BY.get(cls, BLADE_TIP_X))
## The classes whose swing arcs follow the blade's live tip (`_blade_arc_on_tip`).
const TIP_ARC_CLASSES := [0, BUCKLER_CLASS]
## The depth the arc being drawn is at (the blade's own while a sword swing is live, else ARC_Z).
var arc_z := ARC_Z


## (Animation overhaul pass 6: a sword's swing arc follows the blade's tip.) While the swing is live the arc is redrawn on a circle about
## the weapon arm's shoulder through the tip where the blade really is this frame (after the arm's reach and the hilt's correction), its
## head on the tip, at the blade's depth: the authored circles (`ARCS`) were fixed round a standing shoulder and drawn in front of the
## body, so on a lunge or a lean the crescent floated off the blade. The authored arc still sets which way the swing runs and how far it
## has come. It depends only on this frame's pose, so hitlag freezes it and a seek redraws it.
func _blade_arc_on_tip(s: Dictionary) -> void:
	# (The longsword's and the buckler's blades only: on the greatsword the redrawn arc vanished, pass 10, so its authored arcs stay.)
	if arc_shown.is_empty() or not TIP_ARC_CLASSES.has(_cls(s)) or not blade_pivot.visible or skeleton == null:
		return
	if String(s.state) != "Attack" or float(arc_shown.q) > 0.0:
		return
	var iu := skeleton.find_bone("armU." + _weapon_side(int(s.facing)))
	if iu < 0:
		return
	var facing := float(s.facing)
	var tip3 := to_local(blade_pivot.to_global(Vector3(BLADE_TIP_X, 0.0, 0.0)))
	var sh3 := to_local(skeleton.to_global(skeleton.get_bone_global_pose(iu).origin))
	var tip := Vector2(tip3.x * facing, tip3.y)
	var centre := Vector2(sh3.x * facing, sh3.y)
	var rel := tip - centre
	if rel.length() < 0.3:
		return
	var w: Array = arc_shown.w
	var from := deg_to_rad(float(w[3]))
	var span := deg_to_rad(float(w[4])) - from
	# The head's angle measured from the arc's start the way the swing runs (a long arc, like the neutral air's, goes past half a turn).
	var d := fposmod((rel.angle() - from) * signf(span), TAU)
	if d > absf(span) + PI * 0.5:
		d -= TAU
	# (Not yet round to where the swing starts, or the blade is swinging the other way from the authored arc, as some of the
	# greatsword's do: the authored arc stays, rather than a tail swept backward or none at all. Pass 10.)
	if d <= 0.0:
		return
	arc_shown["c"] = centre
	arc_shown["r"] = rel.length()
	arc_shown["head"] = from + signf(span) * d
	arc_shown["z"] = tip3.z
	_draw_arc()


func _draw_arc() -> void:
	if arc == null:
		arc = MeshInstance3D.new()
		arc.mesh = ImmediateMesh.new()
		arc.top_level = true
		arc.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		arc.extra_cull_margin = 16384.0
		# (Painted: as it fades it breaks up in ragged chunks, like a drawn smear wiped off a frame at a time.)
		# (See-through as a trail of light is, `ARC_OPACITY`, but painted: its own copy of the material.)
		var m := (FxMaterial.get_material(false, 5, null, 1.0, 1.0) as ShaderMaterial).duplicate() as ShaderMaterial
		# (A little more solid in paper, 0.75: at 0.62 the pale crescent faded into the light skies.)
		m.set_shader_parameter("opacity", 0.75 if FxMaterial.paper else ARC_OPACITY)
		arc.material_override = m
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
	if arc_shown.has("head"):
		head = float(arc_shown.head)
	# Only the last stretch of the swing shows, a comet's tail behind the head (the reference game's trail never shows the whole sweep
	# at once), eaten from the tail toward the head as it fades.
	var start := deg_to_rad(float(w[3]))
	var swept := head - start
	start = head - signf(swept) * minf(absf(swept), deg_to_rad(ARC_TAIL_DEGREES))
	var tail := lerpf(start, head, q * 0.75)
	if absf(head - tail) < 0.03:
		return
	var circle := _arc_circle(w)
	var centre: Vector2 = arc_shown.get("c", circle[0])
	var r: float = _arc_radius(arc_shown)
	arc_z = float(arc_shown.get("z", ARC_Z))
	# (As thick as the authored circle's arc, however far the hitboxes reach: at the hitboxes' radius a smash's arc was a slab.)
	# (0.22 of the radius: the reference's trail is a thin bright sliver; at 0.34 ours read as a heavy blade of its own.)
	var width := float(circle[1]) * ARC_WIDTH * (1.0 - 0.35 * q)
	var alpha := 1.0 - q
	var colours := trail_colours_of(int(arc_shown.cls))
	# Painted as the knock-out blast is (effects.gd `Beam`): layers of solid paint back to front, an ink silhouette round the whole
	# crescent, the body in a deeper, purer version of the class colour, a lit band along the outside edge, and a white-hot core near the
	# head. The inside edge is scalloped like a brushstroke's. (Flecks thrown off the tail read as a chain of beads, so there are none.) Solid, not a soft
	# gradient, so it reads on a pale sky and over a fighter of the same colour (the ink rim parts it from anything).
	var main: Color = colours[0]
	var body := Color.from_hsv(main.h, minf(1.0, main.s * 1.35), main.v * 0.82)
	var lit := body.lerp(Color.WHITE, 0.35)
	var core: Color = colours[1]
	var ink := Color(INK.r, INK.g, INK.b)
	if FxMaterial.paper:
		# Paper: a pale cut edge where the ink line was, the core cream rather than white light.
		ink = Color(Effects.CUT.r, Effects.CUT.g, Effects.CUT.b)
		core = core.lerp(Color(1.0, 0.95, 0.82), 0.5)
	var facing := float(arc_shown.facing)
	var seed := float(w[3]) * 0.37 + float(w[5])
	var span := head - tail
	var ink_w := clampf(r * 0.035, 0.05, 0.1)
	# Bands across the crescent from the outside in, side by side rather than stacked (so the trail can be see-through without the ink
	# showing through it): the ink edge, the lit band, the body, the ink edge along the scalloped inside. Each [from, to] is [offset out
	# (world units), share of the thickness in, scalloped]. The hot core is laid over the lit band near the head.
	var lit_in := 0.3
	var bands := [[[ink_w, 0.0, false], [0.0, 0.0, false], Color(ink, alpha)], [[0.0, 0.0, false], [0.0, lit_in, false], Color(lit, alpha)],
		[[0.0, lit_in, false], [0.0, 1.0, true], Color(body, alpha)], [[0.0, 1.0, true], [-ink_w, 1.0, true], Color(ink, alpha)]]
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
	for band in bands:
		_arc_band(im, centre, r, width, tail, span, 0.0, band[0], band[1], band[2], facing, seed)
	_arc_band(im, centre, r, width, tail, span, 0.45, [-0.07 * width, 0.0, false], [0.0, 0.14, false], Color(core, alpha), facing, seed)
	im.surface_end()


## One band of a swing arc, between two edges across it (each [offset out in world units, share of the thickness in, scalloped]), from
## `u0` along the arc to its head.
func _arc_band(im: ImmediateMesh, centre: Vector2, r: float, width: float, tail: float, span: float, u0: float, a: Array, b: Array,
		col: Color, facing: float, seed: float) -> void:
	var prev_a := Vector3.ZERO
	var prev_b := Vector3.ZERO
	for i in ARC_STEPS + 1:
		var u := lerpf(u0, 1.0, float(i) / ARC_STEPS)
		var dir := Vector2.from_angle(tail + span * u)
		# Pointed at both ends, thickest a little behind the head; the inside edge scalloped like a brushstroke's.
		var thick := width * pow(sin(PI * pow(u, 1.4)), 0.8)
		# (Gentler in paper: a clean cut edge with a little hand-cut wobble, not a brushstroke's scallops.)
		var scallop := 1.0 + (0.05 if FxMaterial.paper else 0.16) * sin(u * 23.0 + seed) * sin(u * 7.0 + seed * 1.7)
		var ra := r + float(a[0]) - thick * float(a[1]) * (scallop if a[2] else 1.0)
		var rb := r + float(b[0]) - thick * float(b[1]) * (scallop if b[2] else 1.0)
		var pa := centre + dir * ra
		var pb := centre + dir * rb
		var va := to_global(Vector3(pa.x * facing, pa.y, arc_z))
		var vb := to_global(Vector3(pb.x * facing, pb.y, arc_z))
		if i > 0:
			for v in [prev_a, prev_b, vb, prev_a, vb, va]:
				im.surface_set_color(col)
				im.surface_add_vertex(v)
		prev_a = va
		prev_b = vb


# ---- Big moments: dizzy stars, the launch stretch and speed lines --------------------------------------------------------------------

## Three cartoon stars circling the head while the shield is broken.
var dizzy: Node3D
## A strong launch: the body stretched along the way it flies, and streaks trailing behind it.
var launch_frame: Node3D
var launch_lines: Node3D
var launch_trail: Node3D
## The launch trail's colour in the paper look: the player's (set by the match).
var trail_tint := Color(0.3, 0.55, 0.95)
## (Units per frame: 0.5 was faster than nearly any launch, so the streaks and trail never showed. Art review 6.)
const LAUNCH_LINES_FROM := 0.2


## A five-pointed star of radius `r`: an ink outline behind a yellow star with a pale middle, in one mesh (vertex colours).
static func _cartoon_star(r: float) -> ArrayMesh:
	var verts := PackedVector3Array()
	var cols := PackedColorArray()
	# (In the paper look the outline is the pale cut edge.)
	var rim := Color(1.0, 0.97, 0.9) if FxMaterial.paper else Color(0.12, 0.07, 0.12)
	for layer in [[1.28 if not FxMaterial.paper else 1.18, rim, -0.01], [1.0, Color(1.0, 0.82, 0.22), 0.0], [0.45, Color(1.0, 0.97, 0.8), 0.01]]:
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
	# (Far less on the base body: stretched 42% the round body read as a melting tube with its legs merged. Pose review.)
	if use_base_body and float(target[0]) > 0.0:
		target[0] = float(target[0]) * MotionDeform.BASE_LAUNCH_SHARE
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
	# (By the whole flight, knockback included: the fighter's own velocity is small in a launch, so the streaks never showed.)
	var lines_on := flying and flight.length() > LAUNCH_LINES_FROM
	# The launch trail (scripts/effects.gd `LaunchTrail`), behind the body along the way it flies.
	if launch_trail == null and lines_on:
		launch_trail = Effects.LaunchTrail.new()
		launch_trail.tint = trail_tint
		launch_trail.material_override = Effects._material(false)
		add_child(launch_trail)
	if launch_trail != null:
		launch_trail.feed(global_position + Vector3(0, 1.1, 0), lines_on)
	if launch_lines == null and lines_on:
		launch_lines = Node3D.new()
		var mat := StandardMaterial3D.new()
		mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		mat.albedo_color = Color(1.0, 1.0, 0.97, 0.75)
		if FxMaterial.paper:
			# Paper strips in a pale wash of the player's colour, a little wider.
			mat.albedo_color = Color(trail_tint.lerp(Color(1.0, 0.96, 0.88), 0.55), 0.9)
		mat.disable_fog = true
		for i in 5:
			var mi := MeshInstance3D.new()
			var q := QuadMesh.new()
			q.size = Vector2(1.0, 0.09 if FxMaterial.paper else 0.06)
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
	var dir := flight.normalized()
	speed = flight.length()
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
## The trail: a ring of puffs ([where, age, size, seed]) drawn as cel-shaded smoke balls into one mesh (FxMaterial's `puff`), each
## swelling and breaking up over PUFF_LIFE seconds of sim time (so hitlag holds them and a replay draws them the same).
const PUFF_LIFE := 0.6
var puffs: Array = []
var puff_mesh: MeshInstance3D
var next_puff := 0
var last_puff_at := Vector3.INF
var last_puff_frame := -1


func _launch_smoke(s: Dictionary) -> void:
	if puff_mesh == null:
		puff_mesh = MeshInstance3D.new()
		puff_mesh.mesh = ImmediateMesh.new()
		puff_mesh.top_level = true
		puff_mesh.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		puff_mesh.extra_cull_margin = 16384.0
		puff_mesh.material_override = FxMaterial.get_material(false, 2, null, 0.0, 0.0, true)
		add_child(puff_mesh)
		puff_mesh.global_transform = Transform3D.IDENTITY
		for i in PUFFS:
			puffs.append([Vector3.ZERO, 99.0, 1.0, Vector2.ZERO])
	var frame: int = s.frame
	if frame == last_puff_frame:
		return
	var dt := 1.0 / 60.0 * float(maxi(1, frame - last_puff_frame)) if last_puff_frame >= 0 else 1.0 / 60.0
	last_puff_frame = frame
	for p in puffs:
		p[1] += dt
	var here := global_position + Vector3(0, 1.0, 0)
	if s.state == "Hitstun" and int(s.hitlag) == 0 and s.tumble and (last_puff_at == Vector3.INF or here.distance_to(last_puff_at) > 0.45):
		puffs[next_puff] = [here + Vector3(0, 0, -0.3), 0.0, randf_range(0.8, 1.1), Vector2(randf(), randf())]
		next_puff = (next_puff + 1) % PUFFS
		last_puff_at = here
	elif s.state != "Hitstun":
		last_puff_at = Vector3.INF
	var im: ImmediateMesh = puff_mesh.mesh
	im.clear_surfaces()
	var cam := get_viewport().get_camera_3d()
	var right := cam.global_transform.basis.x if cam != null else Vector3.RIGHT
	var up := cam.global_transform.basis.y if cam != null else Vector3.UP
	var live := false
	for p in puffs:
		var life: float = p[1] / PUFF_LIFE
		if life >= 1.0:
			continue
		if not live:
			im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
			live = true
		var size: float = p[2] * (0.6 + life * 0.9)
		var at: Vector3 = p[0] + Vector3(0, life * 0.4, 0)
		var r := right * size * 0.5
		var u := up * size * 0.5
		var corners := [at - r - u, at + r - u, at + r + u, at - r + u]
		var uvs := [Vector2(0, 1), Vector2(1, 1), Vector2(1, 0), Vector2(0, 0)]
		for idx in [0, 1, 2, 0, 2, 3]:
			im.surface_set_color(Color(0.98, 0.97, 1.0, 1.0 - life))
			im.surface_set_uv(uvs[idx])
			im.surface_set_uv2(p[3])
			im.surface_add_vertex(corners[idx])
	if live:
		im.surface_end()


## A class's trail colours: [the trail, its bright core] (claws violet, needles teal, greatsword amber, sword blue).
static func trail_colours_of(cls: int) -> Array:
	if cls == 1:
		return [Color(0.78, 0.3, 1.0), Color(1.0, 0.9, 1.0)]
	if cls == 2:
		return [Color(0.25, 0.85, 0.75), Color(0.9, 1.0, 0.97)]
	if cls == GREATSWORD_CLASS:
		return [Color(1.0, 0.62, 0.22), Color(1.0, 0.96, 0.82)]
	if cls == NODACHI_CLASS:
		return [Color(0.42, 0.42, 0.95), Color(0.92, 0.94, 1.0)]
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


## The striking arm's stretch now: `strike_e`, more on the base body (MotionDeform.BASE_ARM_STRETCH).
func _arm_e() -> float:
	return strike_e * (MotionDeform.BASE_ARM_STRETCH if use_base_body else 1.0)



## A strike with no authored arc (a punch or a kick of the classes without a blade) leaves a smear behind the striking fist or foot: drawn
## out a point a sim frame from `Smear.LEAD` frames before its hit (so it runs from the wind-up) to the hit, held while the hit is live,
## then faded over `ARC_FADE` frames. Nothing moves in hitlag, so a freeze holds it; a rollback or seek drops it.
const SMEAR_FORWARD := 0.08


func _update_smear(s: Dictionary) -> void:
	var frame := int(s.frame)
	if smear_frame >= 0 and (frame < smear_frame or frame - smear_frame > 2):
		smear_pts.clear()
	var advanced := frame != smear_frame
	smear_frame = frame
	# Drawn out over the throw (from the wind-up to the hit's first frame), held while the hit is live, then faded.
	var throwing := false
	var holding := false
	# (No fist smear for a held weapon, a staff or a sword with no authored arc: it drew a punch's ribbon across the chest. Pass 10.)
	if String(s.state) == "Attack" and not bladed(_cls(s)) and _cls(s) != STAFF_CLASS and strike_tip_ok and not (ARCS.get(_cls(s), {}) as Dictionary).has(String(s.move_name)):
		var f := float(s.state_frame)
		var first := float(s.move_timing[1])
		var last := float(s.move_timing[2])
		# (From the snap only, round 2: from LEAD frames before the hit a short move's ribbon also traced the fist drawing back into its
		# coil, a ribbon behind the hips going the wrong way.)
		throwing = first > 0.0 and f >= maxf(first - Smear.LEAD, floorf(SNAP_AT * first)) and f <= first
		holding = first > 0.0 and f > first and f <= last + 1.0
	if advanced and int(s.hitlag) == 0 and not holding:
		if throwing:
			# (At the fist's or foot's own depth, a little toward the camera: drawn at the arcs' fixed depth, ARC_Z, the ribbon floated
			# apart from the limb, a tube across the hips under a claw's swipe. Round 2.)
			smear_pts.append(Vector3(strike_tip.x, strike_tip.y, strike_tip.z + SMEAR_FORWARD))
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
	var great: bool = _cls(s) == GREATSWORD_CLASS
	var brawler: bool = not bladed(_cls(s)) and not staff
	for p in blade_parts:
		p.visible = not staff and not great and _cls(s) != NODACHI_CLASS
	for p in staff_parts:
		p.visible = staff
	for p in great_parts:
		p.visible = great
	for p in claw_parts:
		p.visible = _cls(s) == 1
	for p in dagger_parts:
		p.visible = _cls(s) == DAGGER_CLASS and String(p.get_meta("side")) == _weapon_side(int(s.facing))
	var nodachi: bool = _cls(s) == NODACHI_CLASS
	for p in nodachi_parts:
		p.visible = nodachi
	# Only the body moves leave the arms alone (the claws' listed ones; the needles' kicks, grabs and throws); the sword fighter always
	# has its blade.
	var body_move := false
	if _cls(s) == 1:
		body_move = BRAWLER_NO_BLADE.has(s.move_name)
	elif (_cls(s) == 2 or staff or great) and s.move_name != "":
		body_move = _body_clip(_cls(s), s.move_name)
	# (A clip that poses both arms itself, whatever the class: a headbutt, both hands up on the balloon strings or the firework, a shrug,
	# gripping the rocket. Its tool hand is not pulled to a hit.)
	if s.move_name != "" and not body_move:
		var own := AnimLibrary.attack_clip(_cls(s), s.move_name, anim)
		body_move = AnimLibrary.is_body(own) or AnimLibrary.OWN_ARMS.has(own)
	var swinging_arm: bool = not ((brawler or staff or great) and body_move)
	# (The Seer keeps its staff in hand through a kick, held at rest, as the Champion does its greatsword.)
	blade_pivot.visible = (not brawler and swinging_arm) or staff or great
	# Fists closed through the fight (the owner, art review 5: open, extended hands do not read as combat). A hand opens only where a hand
	# would: grabbing and holding, caught or hit (flailing), hanging from a ledge.
	var open_hands: bool = OPEN_HAND_STATES.has(state) or (s.move_name != "" and OPEN_HAND_MOVES.get(_cls(s), []).has(String(s.move_name)))
	_clench(not open_hands, not open_hands or blade_pivot.visible, int(s.facing))
	# Running with nothing to swing, the weapon arm pumps with the run and the weapon trails behind.
	var carrying: bool = not brawler and s.move_name == "" and (state == "Run" or state == "Dash")
	# Hit, the weapon arm goes with the body's reaction (the clip) and the weapon trails in the loose hand, as in the run: held up in
	# its guard through the hitstun, the sword crossed the face and the launch pointed it ahead like a thrust. Pose review.
	if not brawler and REELING_STATES.has(state):
		carrying = true
	if rig != null:
		var holding: bool = swinging_arm if not brawler else (swinging_arm and state == "Attack")
		# The Wanderer's round shield, on the other forearm, faced outward toward the camera, a little in front of the body.
		_place_buckler(s)
		# (The Champion swings two-handed; a kick or a throw lets go with the other hand.)
		var two := ((great or staff) and swinging_arm and state == "Attack") or (state == "Attack" and both_fists(s))
		# (Round 2: the arm reaching for a hit no longer snaps. It eases in over a move's first frames, from the clip's pose, finished
		# before the first active frame; and when the reach ends (a punch's move over, a sword fighter breaking into a run) it lets go
		# over IK_FADE_FRAMES sim frames from where it was, instead of jumping to the clip's pose in one frame.)
		var hold_now: bool = holding and not carrying
		var amount := 1.0
		if hold_now:
			ik_fade = 1.0
			ik_last_hand = hand_target
			ik_last_off = off_hand_target
			ik_last_two = two
			if state == "Attack" and brawler:
				var first := float((s.move_timing as PackedInt32Array)[1])
				var ease_in := minf(IK_EASE_IN, first - 1.0) if first > 1.0 else 0.0
				if ease_in > 0.0:
					var k := clampf(float(s.state_frame) / ease_in, 0.0, 1.0)
					amount = k * k * (3.0 - 2.0 * k)
		elif springs.jumped:
			ik_fade = 0.0
		elif ik_fade > 0.0:
			ik_fade = maxf(0.0, ik_fade - float(springs.new_frames) / IK_FADE_FRAMES)
		if not hold_now and ik_fade > 0.0:
			hand_target = ik_last_hand
			off_hand_target = ik_last_off
			two = ik_last_two
			amount = ik_fade * ik_fade * (3.0 - 2.0 * ik_fade)
		_aim_arm(int(s.facing), hold_now or ik_fade > 0.0, two, amount)
		_guard_face(int(s.facing), int(s.platform) >= 0)
		if not r8_off.has("beside") and not turning_over and not two:
			_arms_beside_head(int(s.facing))
		# The hilt goes where the hand actually got to (an arm shorter than the swing wants stops short of its target): the weapon is
		# always in the hand, never floating in front of it.
		if holding and not carrying and skeleton != null:
			var hb := skeleton.find_bone("hand." + _weapon_side(int(s.facing)))
			if hb >= 0:
				var held: Vector3 = _skeleton_to_frame() * skeleton.get_bone_global_pose(hb).origin
				var along := Vector3(cos(blade_pivot.rotation.z), sin(blade_pivot.rotation.z), 0.0)
				blade_pivot.position = Vector3(held.x, held.y, blade_pivot.position.z) + along * 0.08
				# (Round 3: and the tip stays out of the floor where the hand really is. An arm that could not reach let the hilt sit lower
				# than the blade was aimed from, and a held follow-through then sank the Greatsword's blade into the stage for 20 frames of
				# its forward smash: only the guard showed.)
				if int(s.platform) >= 0:
					_lift_tip_off_floor()
				# (Round 8, the grip: the fist closes round the weapon, the knuckles along its line, and the weapon sits in the fist.)
				if use_base_body and not r8_off.has("grip") and (bladed(_cls(s)) or _cls(s) == STAFF_CLASS):
					_grip_hand(hb)
					if int(s.platform) >= 0:
						_lift_tip_off_floor()
		_blade_arc_on_tip(s)
		if carrying:
			_carry_weapon(int(s.facing))
		_aim_leg(s, state == "Attack" and AnimLibrary.is_kick(AnimLibrary.attack_clip(_cls(s), s.move_name, anim)), delta)
		_update_smear(s)
		if log_cover:
			_log_face_cover(s)
		if log_arm_show:
			_log_arm_show(s)
		if log_arm_hide:
			_log_arm_hide(s)
	# (The claws' flame rushes only: the Needles' specials are their own.)
	var rushing: bool = _cls(s) == 1 and state == "Attack" and BRAWLER_FLAME.has(s.move_name) and s.state_frame >= 12
	flame.emitting = rushing
	# The flame rush up: a corkscrew about the body's own upright axis (our take; the head stays up, the face comes round every turn).
	if _cls(s) == 1 and state == "Attack" and s.move_name == "up special" and s.state_frame >= 17:
		model.rotation.y = yaw + float(s.state_frame - 17) * 0.75
	# Spinning moves (SPIN_MOVES): the whole body turns about its upright axis with the arms out, by the move's own frame.
	var whirl := _whirl(s)
	if whirl != 0.0:
		model.rotation.y = yaw + whirl * float(s.facing)

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
	_egg(s)
	# Buried: sunk to about half its height in the ground (the bridge's `Buried`), shaking a little as it struggles.
	if state == "Buried":
		model.position = Vector3(0.05 * sin(float(s.frame) * 2.1), -1.0, 0)
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
			_recoil(s)
	elif state == "Hitstun" and not s.tumble and (s.get("kb_vel", Vector2.ZERO) as Vector2).length() <= HEAVY_HITSTUN_SPEED:
		# (Frozen by the hit: already snapped part of the way off it.)
		_recoil(s)
		tumble_angle = 0.0
	elif int(s.platform) < 0 and absf(tumble_angle) > 0.02 and not springs.jumped and not r7_off.has("tumble"):
		# (Round 7: out of a tumble in the air (a jump, an air dodge or an aerial out of it) the body comes upright over a few sim frames,
		# the short way round, instead of in one: it turned up to 171 degrees in a frame, the head with it, in every computer match.)
		tumble_angle = wrapf(tumble_angle, -PI, PI) * pow(TUMBLE_SETTLE, float(springs.new_frames))
		model.rotation.x += tumble_angle
		model.position += SPIN_PIVOT_3D - Basis.from_euler(model.rotation) * SPIN_PIVOT_3D - (SPIN_PIVOT_3D - Basis.from_euler(Vector3(model.rotation.x - tumble_angle, model.rotation.y, model.rotation.z)) * SPIN_PIVOT_3D)
	else:
		tumble_angle = 0.0
