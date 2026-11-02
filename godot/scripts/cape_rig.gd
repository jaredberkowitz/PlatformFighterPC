extends Node3D
## The cape (art/blender/make_gear.py `cape`, a modelled cloth laid over the shoulders and falling to the calves) hung on a short chain of
## bones down its back, each swinging on a code spring like the scarf's tail (`TailRig`): the cape lifts out behind the fighter as it runs,
## blows up as it falls, swings back as it speeds up and forward as it stops, and hangs still again at rest. The top of the cape, over the
## shoulders, stays with the body. It shares the fighter's spring rules (`MotionSprings`): stepped with the sim, frozen in hitlag and while
## paused, at rest after a rollback or seek. Presentation only.
##
## Built in the rig's own space (the cape was modelled round the base body), so the fighter puts it under the torso's frame with the torso's
## fit undone (fighter_view.gd): it then sits where it was modelled and moves with the spine.

const MotionSprings := preload("res://scripts/motion_springs.gd")

## How many bones down the cape (the first holds the shoulders still), and how far below the cape's top the first one turns (rig units).
const BONES := 5
const PIVOT_BELOW_TOP := 0.2
## Each bone's spring: stiffness falling down the chain so the hem lags most, damping, and how far it may turn (radians: forward, back).
const STIFFNESS := 70.0
const STIFFNESS_FALL := 0.8
const DAMPING := 8.0
const LIMIT_FORWARD := 0.12
const LIMIT_BACK := 0.75
## How far the cape lifts behind a fighter moving ahead (radians shared down the chain per world unit a frame of speed), behind a fall,
## and at most; how hard each frame pulls it toward that lift; and how a change of speed kicks it (the swing).
const LIFT_RUN := 2.6
const LIFT_FALL := 1.8
const LIFT_MAX := 1.0
const PULL := 4.0
const KICK := 30.0
const KICK_GROWTH := 0.5

var skeleton: Skeleton3D
var mesh_instance: MeshInstance3D
var springs: RefCounted


## Builds the skinned cape from the modelled mesh (`source`, in the rig's space) in `mat`.
func build(source: Mesh, mat: Material) -> void:
	springs = MotionSprings.new()
	var arrays := source.surface_get_arrays(0)
	var verts: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
	var top := -INF
	var hem := INF
	for v in verts:
		top = maxf(top, v.y)
		hem = minf(hem, v.y)
	# The bones hang down the middle of the cape's back, from just under the shoulders to the hem, at the cloth's own depth there.
	var first := top - PIVOT_BELOW_TOP
	skeleton = Skeleton3D.new()
	add_child(skeleton)
	var heights: Array[float] = []
	var at := Vector3.ZERO
	for i in BONES:
		var y := lerpf(first, hem, float(i) / float(BONES))
		heights.append(y)
		var p := Vector3(0.0, y, _depth_at(verts, y))
		skeleton.add_bone("c%d" % i)
		if i > 0:
			skeleton.set_bone_parent(i, i - 1)
		skeleton.set_bone_rest(i, Transform3D(Basis.IDENTITY, p - at))
		at = p
		springs.add("c%d" % i, STIFFNESS * pow(STIFFNESS_FALL, float(i)), DAMPING, -LIMIT_FORWARD, LIMIT_BACK)
	skeleton.reset_bone_poses()
	# Each point follows the bone above it, blending into the next over the stretch between them; above the first bone, the body.
	var bones := PackedInt32Array()
	var weights := PackedFloat32Array()
	for v in verts:
		var f := clampf((first - v.y) / maxf(first - hem, 0.01) * float(BONES), 0.0, float(BONES - 1))
		var k := mini(int(f), BONES - 2)
		var w := clampf(f - float(k), 0.0, 1.0)
		bones.append_array([k, k + 1, 0, 0])
		weights.append_array([1.0 - w, w, 0.0, 0.0])
	arrays[Mesh.ARRAY_BONES] = bones
	arrays[Mesh.ARRAY_WEIGHTS] = weights
	var mesh := ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	var skin := Skin.new()
	for i in BONES:
		skin.add_named_bind("c%d" % i, skeleton.get_bone_global_rest(i).affine_inverse())
	mesh_instance = MeshInstance3D.new()
	mesh_instance.mesh = mesh
	mesh_instance.skin = skin
	mesh_instance.material_override = mat
	skeleton.add_child(mesh_instance)
	mesh_instance.skeleton = NodePath("..")


## The depth of the cape's middle at height `y` (rig units), where its bone there turns.
static func _depth_at(verts: PackedVector3Array, y: float) -> float:
	var sum := 0.0
	var n := 0
	for v in verts:
		if absf(v.x) < 0.08 and absf(v.y - y) < 0.06:
			sum += v.z
			n += 1
	return sum / float(n) if n > 0 else -0.4


## Call every rendered frame with the snapshot's frame and hitlag, the fighter's velocity and its change this sim frame (world units a
## frame) and which way it faces.
func update(sim_frame: int, hitlag: int, vel: Vector2, dv: Vector2, facing: int, delta: float) -> void:
	if springs == null:
		return
	springs.begin(sim_frame, hitlag, delta)
	if springs.new_frames > 0 and dv.length() < 0.7:
		# Moving ahead or falling, the cape lifts out behind; each bone takes a share of the lift, more toward the hem.
		var lift := clampf(maxf(vel.x * float(facing), 0.0) * LIFT_RUN + maxf(-vel.y, 0.0) * LIFT_FALL, 0.0, LIFT_MAX)
		var ahead := dv.x * float(facing)
		for i in range(1, BONES):
			var name := "c%d" % i
			var share := lift / float(BONES - 1) * (0.7 + 0.6 * float(i) / float(BONES - 1))
			springs.kick(name, (share - springs.value(name)) * PULL + ahead * KICK * (1.0 + KICK_GROWTH * float(i)))
	springs.step(delta)
	for i in range(1, BONES):
		skeleton.set_bone_pose_rotation(i, Quaternion(Vector3.RIGHT, springs.value("c%d" % i)))


func reset() -> void:
	if springs != null:
		springs.reset()
	if skeleton != null:
		skeleton.reset_bone_poses()


## The meshes, for whoever tags the fighter's meshes (outline, ghost fade).
func meshes() -> Array[MeshInstance3D]:
	return [mesh_instance] if mesh_instance != null else []
