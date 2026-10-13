extends Node3D
## A hanging cloth tail (a scarf's end, a sash's end) as a short chain of segments, each hung from the one above on a code spring
## (docs/MOTION_POLISH_PLAN.md phase 2b; the phase 0 spike ruled out `SpringBoneSimulator3D` for this rig). When the body speeds up, stops
## or rises and falls, each joint is kicked a little more than the one above it, so the tail swings and whips and then hangs still again
## ("restraint at rest"). It shares the fighter's spring rules (`MotionSprings`): stepped with the sim, frozen in hitlag and while paused,
## at rest after a rollback or seek. Presentation only.

const MotionSprings := preload("res://scripts/motion_springs.gd")

## Each joint's stiffness falls and its kick grows down the chain, so the end lags the most.
const STIFFNESS := 150.0
const STIFFNESS_FALL := 0.72
const DAMPING := 7.0
const KICK := 34.0
const KICK_GROWTH := 0.55
const LIMIT := 0.7

var joints: Array[Node3D] = []
var springs: RefCounted


## Builds the chain hanging straight down from this node: `segments` pieces of `length` in all, `width` across and `thickness` deep, in
## `mat`, each piece a slightly narrower box than the one above, the last one pointed (cut at an angle).
func build(mat: Material, length: float, width: float, thickness: float, segments := 4) -> void:
	springs = MotionSprings.new()
	var parent: Node3D = self
	var piece := length / float(segments)
	for i in segments:
		var joint := Node3D.new()
		joint.position = Vector3(0, 0 if i == 0 else -piece, 0)
		parent.add_child(joint)
		joints.append(joint)
		var box := BoxMesh.new()
		var w := width * (1.0 - 0.08 * float(i))
		# (A little overlap, so no gap opens between two pieces as they bend.)
		box.size = Vector3(w, piece * 1.12, thickness)
		var mi := MeshInstance3D.new()
		mi.mesh = box
		mi.material_override = mat
		mi.position = Vector3(0, -piece * 0.5, 0)
		if i == segments - 1:
			mi.rotation = Vector3(0, 0, deg_to_rad(8.0))
		joint.add_child(mi)
		springs.add("j%d" % i, STIFFNESS * pow(STIFFNESS_FALL, float(i)), DAMPING, -LIMIT, LIMIT)
		parent = joint


## The meshes, for whoever tags the fighter's meshes (outline, ghost fade).
func meshes() -> Array[MeshInstance3D]:
	var out: Array[MeshInstance3D] = []
	for j in joints:
		for c in j.get_children():
			if c is MeshInstance3D:
				out.append(c)
	return out


## Call every rendered frame with the snapshot's frame and hitlag and the change of velocity this sim frame (`dv`, world units a frame).
## `side` is +1 or -1: which way a push along the stage swings the tail in this node's own frame.
func update(sim_frame: int, hitlag: int, dv: Vector2, side: float, delta: float) -> void:
	if springs == null:
		return
	springs.begin(sim_frame, hitlag, delta)
	if dv.length() < 0.7:
		for i in joints.size():
			var k := KICK * (1.0 + KICK_GROWTH * float(i))
			springs.kick("j%d" % i, -dv.x * side * k + absf(dv.y) * k * 0.25 * (1.0 if i % 2 == 0 else -1.0))
	springs.step(delta)
	for i in joints.size():
		joints[i].rotation = Vector3(0, 0, springs.value("j%d" % i))


func reset() -> void:
	if springs != null:
		springs.reset()
	for j in joints:
		j.rotation = Vector3.ZERO
