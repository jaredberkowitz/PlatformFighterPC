extends SceneTree
## The motion polish layers (docs/MOTION_POLISH_PLAN.md): deformation follows the move's frames, hitlag freezes it, and a rollback or
## replay seek (a jump in the sim frame) never leaves a stale stretch, smear or spring behind.
## Run: Godot --headless --path godot --script res://tests/motion_test.gd

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
const MotionDeform := preload("res://scripts/motion_deform.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func snap(cls: int, state: String, move: String, frame: int, state_frame: int) -> Dictionary:
	return {"state": state, "facing": 1, "vel": Vector2.ZERO, "kb_vel": Vector2.ZERO, "platform": 0, "fast_fall": false, "invuln": 0,
		"ledge_invuln": 0, "frame": frame, "shield": 1.0, "percent": 0.0, "move_name": move, "move_tip": Vector3(1.3, 1.1, 0.6),
		"move_timing": PackedInt32Array([20, 4, 6]), "charge": 0, "hitlag": 0, "tumble": false, "launch_pending": false, "char": cls,
		"class": cls, "reach": 3.0, "state_frame": state_frame, "jumps": 1, "stocks": 3, "dodge": PackedInt32Array()}


func _initialize() -> void:
	# The strike stretch follows the move's frames: nothing in the wind-up, full on the hit, home again after the follow-through.
	check(MotionDeform.strike_stretch(1.0, 4.0, 6.0) == 0.0, "no stretch early in the wind-up")
	check(MotionDeform.strike_stretch(5.0, 4.0, 6.0) == MotionDeform.STRIKE_STRETCH, "full stretch on the hit")
	check(MotionDeform.strike_stretch(6.0 + 2.0 + MotionDeform.STRIKE_RELAX, 4.0, 6.0) == 0.0, "the stretch is gone after the follow-through")
	var sc := MotionDeform.limb_scale(0.21)
	check(absf(sc.x * sc.y * sc.z - 1.0) < 0.001, "a stretched limb keeps its volume")
	# The body: squashed in the freeze of a launching hit, stretched in a fast flight, plain otherwise.
	check(float(MotionDeform.body_target("Hitstun", 10, true, Vector2.ZERO)[0]) < 0.0, "a launching hit squashes through its freeze")
	check(float(MotionDeform.body_target("Hitstun", 0, false, Vector2(0.6, 0.3))[0]) > 0.0, "a fast launch stretches")
	check(float(MotionDeform.body_target("Hitstun", 0, false, Vector2(0.1, 0.0))[0]) == 0.0, "a slow flight does not")
	check(float(MotionDeform.body_target("Idle", 0, false, Vector2(0.6, 0.0))[0]) == 0.0, "only a launched fighter stretches")

	var view := FighterView.new()
	root.add_child(view)
	view.build(1, Loadout.default_for(1))
	await process_frame

	# A launch stretches the body; a rollback to before it (the sim frame goes back) shows the plain body at once.
	var fly := snap(1, "Hitstun", "", 100, 5)
	fly.vel = Vector2(0.7, 0.4)
	fly.tumble = true
	for f in 6:
		fly.frame = 100 + f
		view.apply(Vector3(float(f), 2.0, 0), fly, 1.0 / 60.0)
	check(view.body_k > 0.2, "a fast launch stretches the body (%.2f)" % view.body_k)
	view.apply(Vector3.ZERO, snap(1, "Idle", "", 40, 0), 1.0 / 60.0)
	check(view.body_k == 0.0 and view.launch_frame.transform.is_equal_approx(Transform3D.IDENTITY),
		"after a rollback the body is plain at once (%.2f)" % view.body_k)

	# A punch with no authored arc (the claws' jab) leaves a smear; hitlag holds it; a seek drops it.
	for f in 6:
		view.apply(Vector3.ZERO, snap(1, "Attack", "jab", 200 + f, f), 1.0 / 60.0)
	check(view.smear_pts.size() >= 2, "the jab leaves a smear (%d points)" % view.smear_pts.size())
	var held: Array = view.smear_pts.duplicate()
	var frozen := snap(1, "Attack", "jab", 206, 6)
	frozen.hitlag = 6
	for i in 4:
		view.apply(Vector3.ZERO, frozen, 1.0 / 60.0)
	check(view.smear_pts == held and view.smear_q == 0.0, "hitlag holds the smear")
	view.apply(Vector3.ZERO, snap(1, "Attack", "jab", 900, 5), 1.0 / 60.0)
	check(view.smear_pts.size() <= 1, "a seek drops the smear")
	# Secondary motion: a dash start leaves the chest and head behind; hitlag holds them; a rollback puts them back to rest at once; at
	# rest they settle to nothing.
	var run := snap(1, "Idle", "", 1000, 0)
	view.apply(Vector3.ZERO, run, 1.0 / 60.0)
	run = snap(1, "Dash", "", 1001, 1)
	run.vel = Vector2(0.28, 0.0)
	view.apply(Vector3.ZERO, run, 1.0 / 60.0)
	for f in 3:
		run.frame = 1002 + f
		view.apply(Vector3.ZERO, run, 1.0 / 60.0)
	var head: float = view.springs.value("head")
	check(head < -1.0, "a dash start leaves the head behind (%.2f degrees)" % head)
	var stuck := run.duplicate()
	stuck.hitlag = 5
	for i in 5:
		view.apply(Vector3.ZERO, stuck, 1.0 / 60.0)
	check(view.springs.value("head") == head, "hitlag holds the springs")
	run.frame = 500
	view.apply(Vector3.ZERO, run, 1.0 / 60.0)
	check(view.springs.value("head") == 0.0 and view.springs.value("spine") == 0.0, "a rollback puts the springs at rest")
	var idle := snap(1, "Idle", "", 600, 0)
	for f in 90:
		idle.frame = 600 + f
		view.apply(Vector3.ZERO, idle, 1.0 / 60.0)
	check(absf(view.springs.value("head")) < 0.05 and absf(view.springs.value("hat_roll")) < 0.01, "standing still, the springs settle")
	# A long shield hold keeps the head where it is (the shield clip has no head track: the turn toward the camera must not pile up).
	var guard := snap(0, "Shield", "", 2000, 0)
	var sword_guard := FighterView.new()
	root.add_child(sword_guard)
	sword_guard.build(0, Loadout.default_for(0))
	await process_frame
	var head_at := []
	for f in 120:
		guard.frame = 2000 + f
		guard.state_frame = f
		sword_guard.apply(Vector3.ZERO, guard, 1.0 / 60.0)
		if f == 20 or f == 119:
			head_at.append(sword_guard.skeleton.get_bone_pose_rotation(sword_guard.head_bone))
	check((head_at[0] as Quaternion).angle_to(head_at[1]) < deg_to_rad(8.0), "the head holds still behind a long shield")
	# The sword's moves have authored arcs and draw no smear.
	var sword := FighterView.new()
	root.add_child(sword)
	sword.build(0, Loadout.default_for(0))
	await process_frame
	for f in 6:
		sword.apply(Vector3.ZERO, snap(0, "Attack", "ftilt", 300 + f, f), 1.0 / 60.0)
	check(sword.smear_pts.is_empty(), "a move with an arc draws no smear")

	print("motion test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
