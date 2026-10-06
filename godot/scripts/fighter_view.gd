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
const POSES := {
	"JumpSquat": [Vector3(1.18, 0.7, 1.18), 0.0],
	"Landing": [Vector3(1.15, 0.8, 1.15), 0.0],
	"Crouch": [Vector3(1.12, 0.7, 1.12), 0.0],
	"Dash": [Vector3(1.0, 0.95, 1.0), 16.0],
	"Run": [Vector3(1.0, 0.97, 1.0), 12.0],
	"WaveLand": [Vector3(1.1, 0.78, 1.1), 28.0],
	"Walk": [Vector3(1.0, 1.0, 1.0), 4.0],
	"Turn": [Vector3(0.9, 1.0, 0.9), 0.0],
	"LedgeAttack": [Vector3(1.0, 1.0, 1.0), 20.0],
}

var player := 0
var model: Node3D
var shield: MeshInstance3D
var meshes: Array[MeshInstance3D] = []
var yaw := 0.0
var scale_now := Vector3.ONE
var lean := 0.0


static func toon(c: Color, outline := true) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.albedo_color = c
	m.diffuse_mode = BaseMaterial3D.DIFFUSE_TOON
	m.specular_mode = BaseMaterial3D.SPECULAR_TOON
	m.roughness = 1.0
	if outline:
		var o := StandardMaterial3D.new()
		o.albedo_color = INK
		o.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		o.cull_mode = BaseMaterial3D.CULL_FRONT
		o.grow = true
		o.grow_amount = 0.035
		m.next_pass = o
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
	var s := SphereMesh.new()
	s.radius = r
	s.height = r * 2.0
	s.radial_segments = 32
	s.rings = 16
	return s


func _cyl(top: float, bottom: float, h: float) -> CylinderMesh:
	var c := CylinderMesh.new()
	c.top_radius = top
	c.bottom_radius = bottom
	c.height = h
	c.radial_segments = 32
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


## `s` is a dictionary of sim state (see main.gd `_snapshot`). `alpha` is the render interpolation.
func apply(pos: Vector3, s: Dictionary, delta: float) -> void:
	position = pos
	var state: String = s.state
	var facing: int = s.facing
	var pose: Array = POSES.get(state, [Vector3.ONE, 0.0])
	var target_scale: Vector3 = pose[0]
	var target_lean: float = pose[1]
	if state == "Airborne" or state == "Helpless":
		var vy: float = s.vel.y
		target_scale = Vector3(0.94, 1.08, 0.94) if vy > 0.06 else (Vector3(0.97, 1.04, 0.97) if vy < -0.12 else Vector3.ONE)
	var k := clampf(delta * 22.0, 0.0, 1.0)
	scale_now = scale_now.lerp(target_scale, k)
	lean = lerpf(lean, target_lean, k)
	model.scale = scale_now
	yaw = lerp_angle(yaw, float(facing) * deg_to_rad(36.0), clampf(delta * 18.0, 0.0, 1.0))
	model.rotation = Vector3(0, yaw, -float(facing) * deg_to_rad(lean))

	# Air dodge and ledge invincibility read as ghostly.
	var ghost := 0.0
	if state == "AirDodge":
		ghost = 0.55
	elif s.ledge_invuln > 0:
		ghost = 0.45 if (s.frame / 3) % 2 == 0 else 0.1
	for m in meshes:
		m.transparency = ghost
	shield.visible = state == "Shield" or state == "ShieldDrop"
