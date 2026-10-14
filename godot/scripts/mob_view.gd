extends Node3D
## A mob, drawn as a placeholder in the house style (a round blob body with a simple face; see docs/ART_DIRECTION.md): a walker is a
## squat brick-red blob, a flyer a small violet one with flapping wings. It flashes while winding up an attack, stretches into a lunge
## or dive, squashes when hit, and pops when beaten. Presentation only; the sim (`sim_core::run`) decides everything.
## The art director's enemy designs replace this (docs/modes/RUNNER_PLAN.md, R1 progress).

const MODE_WINDUP := 2
const MODE_ATTACK := 3
const MODE_BURST := 5

const COLOURS := [Color(0.78, 0.33, 0.27), Color(0.55, 0.4, 0.85)]

var body: MeshInstance3D
var body_mat: StandardMaterial3D
var eyes: Array = []
var wings: Array = []
var behavior := -1
var radius := 0.5
var t := 0.0
var squash := 0.0


func build(kind_behavior: int, r: float) -> void:
	behavior = kind_behavior
	radius = r
	for c in get_children():
		c.queue_free()
	eyes.clear()
	wings.clear()
	body = MeshInstance3D.new()
	var s := SphereMesh.new()
	s.radius = r
	s.height = r * 2.0
	body.mesh = s
	body_mat = StandardMaterial3D.new()
	body_mat.albedo_color = COLOURS[clampi(behavior, 0, 1)]
	body_mat.roughness = 0.8
	body.material_override = body_mat
	add_child(body)
	for side in [-1.0, 1.0]:
		var white := MeshInstance3D.new()
		var ws := SphereMesh.new()
		ws.radius = r * 0.24
		ws.height = r * 0.48
		white.mesh = ws
		var wm := StandardMaterial3D.new()
		wm.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		wm.albedo_color = Color.WHITE
		white.material_override = wm
		body.add_child(white)
		var pupil := MeshInstance3D.new()
		var ps := SphereMesh.new()
		ps.radius = r * 0.12
		ps.height = r * 0.24
		pupil.mesh = ps
		var pm := StandardMaterial3D.new()
		pm.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		pm.albedo_color = Color(0.09, 0.1, 0.2)
		pupil.material_override = pm
		white.add_child(pupil)
		pupil.position = Vector3(0, 0, r * 0.14)
		eyes.append([white, side])
	if behavior == 1:
		for side in [-1.0, 1.0]:
			var wing := MeshInstance3D.new()
			var q := BoxMesh.new()
			q.size = Vector3(r * 1.3, r * 0.12, r * 0.7)
			wing.mesh = q
			var wm2 := StandardMaterial3D.new()
			wm2.albedo_color = COLOURS[1].lightened(0.3)
			wing.material_override = wm2
			add_child(wing)
			wings.append([wing, side])


## `m`: [active, kind, behavior, x, y, facing, mode, hitlag, hitstun, stamina left, percent, radius] (see `SimRunner.run_mobs`).
func apply(pos: Vector2, m: Array, delta: float) -> void:
	t += delta
	var facing: float = m[5]
	var mode := int(m[6])
	var hitlag := int(m[7])
	var hitstun := int(m[8])
	position = Vector3(pos.x, pos.y, 0.0)
	if hitlag > 0 and hitstun > 0:
		squash = 1.0
	squash = move_toward(squash, 0.0, delta * 4.0)
	var sx := 1.0 + squash * 0.35
	var sy := 1.0 - squash * 0.3
	if mode == MODE_ATTACK:
		sx *= 1.25
		sy *= 0.85
	elif mode == MODE_WINDUP:
		sy *= 0.9 + 0.08 * sin(t * 40.0)
	var bob := 0.0 if behavior == 0 else sin(t * 6.0) * 0.12
	body.position = Vector3(0, radius * sy + bob, 0)
	body.scale = Vector3(sx, sy, 1.0)
	var base: Color = COLOURS[clampi(behavior, 0, 1)]
	if mode == MODE_WINDUP:
		body_mat.albedo_color = base.lerp(Color(1, 1, 0.8), 0.5 + 0.5 * sin(t * 30.0))
	elif hitstun > 0:
		body_mat.albedo_color = base.lightened(0.35)
	else:
		body_mat.albedo_color = base
	for e in eyes:
		var white: MeshInstance3D = e[0]
		var side: float = e[1]
		white.position = Vector3(facing * radius * 0.45 + side * radius * 0.28, radius * 0.25, radius * 0.75)
		white.scale = Vector3(1, 0.6 if mode == MODE_WINDUP else 1.0, 1)
	for w in wings:
		var wing: MeshInstance3D = w[0]
		var side: float = w[1]
		var flap := sin(t * (26.0 if mode == MODE_ATTACK else 14.0)) * 0.6
		wing.position = Vector3(side * radius * 0.2, radius * 1.3 + bob, -radius * 0.2 * side)
		wing.rotation = Vector3(0, 0, flap * side)
	visible = mode != MODE_BURST
