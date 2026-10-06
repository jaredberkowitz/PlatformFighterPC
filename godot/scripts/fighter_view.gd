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


func build(p: int) -> void:
	player = p
	var col: Color = COLORS[p % COLORS.size()]
	var skin := toon(col)
	var ink := toon(INK, false)
	model = Node3D.new()
	add_child(model)

	# Body, head, feet, hands. Height matches the 2.2 unit ECB.
	_part(model, _sphere(0.62), skin, Vector3(0, 0.82, 0), Vector3(1.0, 0.95, 0.9))
	_part(model, _sphere(0.8), skin, Vector3(0, 1.42, 0))
	for sx in [-1.0, 1.0]:
		_part(model, _sphere(0.24), skin, Vector3(sx * 0.32, 0.2, 0.05), Vector3(1, 0.8, 1.35))
		_part(model, _sphere(0.22), toon(Color(1, 1, 1)), Vector3(sx * 0.78, 0.85, 0.05))

	# Sash across the torso.
	var sash := BoxMesh.new()
	sash.size = Vector3(1.5, 0.2, 0.12)
	_part(model, sash, toon(SASH), Vector3(0, 0.85, 0.5), Vector3.ONE, Vector3(0, 0, -42))

	# Heavy-lidded face on the front (+z) of the head.
	for sx in [-1.0, 1.0]:
		var eye_pos := Vector3(sx * 0.31, 1.5, 0.745)
		_part(model, _sphere(1.0), toon(Color(1, 1, 1), false), eye_pos, Vector3(0.17, 0.19, 0.05))
		_part(model, _sphere(1.0), ink, eye_pos + Vector3(sx * -0.02, -0.03, 0.03), Vector3(0.08, 0.1, 0.04))
		# Heavy lid: head-coloured cap over the upper half of the eye.
		_part(model, _sphere(1.0), skin, eye_pos + Vector3(0, 0.085, 0.045), Vector3(0.2, 0.11, 0.06))
		_part(model, _sphere(1.0), ink, eye_pos + Vector3(0, 0.0, 0.05), Vector3(0.18, 0.012, 0.03))
	_part(model, _sphere(1.0), ink, Vector3(0, 1.17, 0.775), Vector3(0.14, 0.05, 0.05))

	_accessories(p, skin)

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


func _accessories(p: int, _skin: StandardMaterial3D) -> void:
	var white := toon(Color(0.97, 0.97, 1.0))
	var navy := toon(Color(0.2, 0.3, 0.7))
	match p % 4:
		0, 2:
			# Sailor cap.
			_part(model, _cyl(0.5, 0.55, 0.32), white, Vector3(0, 2.18, 0))
			_part(model, _cyl(0.62, 0.62, 0.08), white, Vector3(0, 2.02, 0))
			_part(model, _cyl(0.5, 0.5, 0.05), navy, Vector3(0, 2.1, 0), Vector3(1.04, 1.0, 1.04))
		1:
			# Aviator cap with goggles on top.
			_part(model, _sphere(0.86), toon(Color(0.78, 0.6, 0.38)), Vector3(0, 1.84, -0.14), Vector3(1.0, 0.6, 1.0))
			var ring := TorusMesh.new()
			ring.inner_radius = 0.1
			ring.outer_radius = 0.2
			for sx in [-1.0, 1.0]:
				_part(model, ring, toon(Color(0.45, 0.28, 0.12)), Vector3(sx * 0.3, 2.12, 0.28), Vector3.ONE, Vector3(70, 0, 0))
		3:
			# Straw hat.
			var straw := toon(Color(0.9, 0.78, 0.45))
			_part(model, _cyl(1.15, 1.15, 0.07), straw, Vector3(0, 2.05, 0))
			_part(model, _cyl(0.55, 0.62, 0.35), straw, Vector3(0, 2.25, 0))
			_part(model, _cyl(0.63, 0.63, 0.08), toon(Color(0.4, 0.25, 0.2)), Vector3(0, 2.14, 0))
	# Shades on players 1 and 2, like the reference.
	if p % 4 == 1 or p % 4 == 2:
		var lens := toon(Color(1.0, 0.62, 0.3), false)
		var frame := toon(INK, false)
		for sx in [-1.0, 1.0]:
			_part(model, _sphere(1.0), lens, Vector3(sx * 0.32, 1.52, 0.8), Vector3(0.25, 0.18, 0.04))
		_part(model, _sphere(1.0), frame, Vector3(0, 1.54, 0.82), Vector3(0.1, 0.03, 0.03))


## Squash pose per state as a single number: positive squashes down and out, negative stretches up.
const SQUASH := {
	"JumpSquat": 0.32, "Landing": 0.2, "Crouch": 0.28, "WaveLand": 0.24, "Turn": 0.08,
	"Dash": 0.04, "ShieldDrop": 0.0,
}
## Forward lean in degrees per state.
const LEAN := {"Dash": 16.0, "Run": 12.0, "WaveLand": 28.0, "Walk": 4.0, "LedgeAttack": 20.0}

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
	speed_lines.visible = fast_falling
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
var blade_pivot: Node3D
var spark: MeshInstance3D
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

	var hitlag: int = s.hitlag
	spark.visible = hitlag > 0 and state == "Hitstun" and s.launch_pending
	if spark.visible:
		spark.scale = Vector3.ONE * (0.5 + 0.1 * hitlag)
	# Shake while frozen in hitlag.
	model.position = Vector3(sin(float(s.frame) * 9.0) * 0.14, 0, 0) if hitlag > 0 else Vector3.ZERO
	if state == "Hitstun" and hitlag == 0:
		if s.tumble:
			model.rotation.z = float(s.frame) * 0.4 * -float(s.facing)
		else:
			model.rotation.z = deg_to_rad(26.0) * float(s.facing)
