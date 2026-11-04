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
	# (The base body takes MotionDeform.BASE_LAUNCH_SHARE of the stretch: stretched far, the round body read as melting.)
	check(view.body_k > 0.1, "a fast launch stretches the body (%.2f)" % view.body_k)
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
	# The body's squash and lean springs (round 2): a landing squash holds still through hitlag and is on its target after a seek.
	var land := snap(1, "Airborne", "", 700, 10)
	land.platform = -1
	land.vel = Vector2(0.0, -0.3)
	view.apply(Vector3.ZERO, land, 1.0 / 60.0)
	land = snap(1, "Landing", "", 701, 0)
	view.apply(Vector3.ZERO, land, 1.0 / 60.0)
	land.frame = 702
	view.apply(Vector3.ZERO, land, 1.0 / 60.0)
	var squashed: float = view.squash
	var lagged := land.duplicate()
	lagged.hitlag = 8
	for i in 6:
		view.apply(Vector3.ZERO, lagged, 1.0 / 60.0)
	check(view.squash == squashed, "hitlag holds the body's squash (%.3f then %.3f)" % [squashed, view.squash])
	land.frame = 400
	view.apply(Vector3.ZERO, land, 1.0 / 60.0)
	check(view.squash_vel == 0.0 and view.lean_vel == 0.0, "a seek puts the body's springs on their targets")
	var idle := snap(1, "Idle", "", 600, 0)
	for f in 90:
		idle.frame = 600 + f
		view.apply(Vector3.ZERO, idle, 1.0 / 60.0)
	check(absf(view.springs.value("head")) < 0.05 and absf(view.springs.value("hat_roll")) < 0.01, "standing still, the springs settle")
	# A scarf's tail (a spring chain) swings on a dash start and is at rest at once after a rollback.
	var scarf_look: RefCounted = Loadout.default_for(0)
	scarf_look.neck = 3
	var scarfed := FighterView.new()
	root.add_child(scarfed)
	scarfed.build(0, scarf_look)
	await process_frame
	check(scarfed.tails.size() == 1, "the scarf has a tail chain")
	var go := snap(0, "Idle", "", 3000, 0)
	scarfed.apply(Vector3.ZERO, go, 1.0 / 60.0)
	go = snap(0, "Dash", "", 3001, 1)
	go.vel = Vector2(0.28, 0.0)
	for f in 4:
		go.frame = 3001 + f
		scarfed.apply(Vector3.ZERO, go, 1.0 / 60.0)
	var tail: Node3D = scarfed.tails[0]
	check(absf(tail.joints[-1].rotation.z) > 0.02, "the scarf's tail swings on a dash start")
	go.frame = 10
	scarfed.apply(Vector3.ZERO, go, 1.0 / 60.0)
	check(tail.joints[-1].rotation.z == 0.0, "a rollback puts the tail at rest")
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
	# A wall jump (the sim's wall-jump count goes up in the air) is a back flip; landing ends it.
	var wj := snap(1, "Airborne", "", 4000, 3)
	wj.platform = -1
	wj.wall_jumps = 0
	view.apply(Vector3(0, 3, 0), wj, 1.0 / 60.0)
	wj.frame = 4001
	wj.wall_jumps = 1
	wj.vel = Vector2(-0.2, 0.4)
	view.apply(Vector3(0, 3, 0), wj, 1.0 / 60.0)
	check(view.flip_left > 0.0 and view.flip_back, "a wall jump starts a back flip")
	# The sword's moves have authored arcs and draw no smear.
	var sword := FighterView.new()
	root.add_child(sword)
	sword.build(0, Loadout.default_for(0))
	await process_frame
	for f in 6:
		sword.apply(Vector3.ZERO, snap(0, "Attack", "ftilt", 300 + f, f), 1.0 / 60.0)
	check(sword.smear_pts.is_empty(), "a move with an arc draws no smear")

	# Round 9, the reactions. A flinch on the ground keeps its recoil past the freeze: the body leans back off the hit in the freeze and
	# goes on leaning back after it (it stood straight up in one frame when the freeze ended).
	var hurt := FighterView.new()
	root.add_child(hurt)
	hurt.build(1, Loadout.default_for(1))
	await process_frame
	var flinch := snap(1, "Hitstun", "", 5000, 0)
	flinch.facing = -1
	flinch.hitstun = 20
	flinch.hitlag = 4
	for f in 4:
		flinch.frame = 5000 + f
		flinch.hitlag = 4 - f
		hurt.apply(Vector3.ZERO, flinch, 1.0 / 60.0)
	var frozen_lean := hurt.model.rotation.x
	flinch.hitlag = 0
	flinch.frame = 5004
	flinch.state_frame = 1
	flinch.hitstun = 19
	flinch.kb_vel = Vector2(0.1, 0.0)
	hurt.apply(Vector3.ZERO, flinch, 1.0 / 60.0)
	check(frozen_lean < -0.05 and hurt.model.rotation.x < frozen_lean * 0.8, "a ground flinch goes on recoiling after the freeze (%.2f, then %.2f)" % [frozen_lean, hurt.model.rotation.x])
	# A launch's clip is chosen on its first frame and kept while the knockback slows (it went from the arch to the air's flinch).
	var pop := snap(1, "Hitstun", "", 5100, 0)
	pop.platform = -1
	pop.facing = -1
	pop.kb_vel = Vector2(0.05, 0.22)
	hurt.apply(Vector3(0, 0.5, 0), pop, 1.0 / 60.0)
	var first_clip := hurt.hit_clip
	pop.frame = 5113
	pop.state_frame = 13
	pop.kb_vel = Vector2(0.03, 0.12)
	for f in 13:
		pop.frame = 5101 + f
		pop.state_frame = 1 + f
		hurt.apply(Vector3(0, 0.5, 0), pop, 1.0 / 60.0)
	check(first_clip == "hurt_up" and hurt.hit_clip == "hurt_up", "a pop upward keeps its clip as it slows (%s, then %s)" % [first_clip, hurt.hit_clip])
	# A tumble turns in the plane of the stage: the body's up stays across the screen, never tipped toward or away from the camera.
	var spin := snap(1, "Hitstun", "", 5200, 0)
	spin.platform = -1
	spin.tumble = true
	spin.kb_vel = Vector2(0.15, 0.05)
	var most_depth := 0.0
	for f in 30:
		spin.frame = 5200 + f
		spin.state_frame = f
		hurt.apply(Vector3(0, 3.0, 0), spin, 1.0 / 60.0)
		most_depth = maxf(most_depth, absf(hurt.model.global_transform.basis.y.normalized().z))
	check(absf(hurt.tumble_angle) > 0.5 and most_depth < 0.2, "a tumble spins in the stage's plane (turned %.2f, depth %.2f)" % [hurt.tumble_angle, most_depth])
	# Out of a tumbling hitstun into the air the damage fall goes on flailing; a seek ends it.
	var fallen := snap(1, "Airborne", "", 5230, 0)
	fallen.platform = -1
	hurt.apply(Vector3(0, 3.0, 0), fallen, 1.0 / 60.0)
	check(hurt.damage_fall and hurt.current_clip == "tumble", "a damage fall keeps tumbling (%s)" % hurt.current_clip)
	fallen.frame = 5400
	hurt.apply(Vector3(0, 3.0, 0), fallen, 1.0 / 60.0)
	check(not hurt.damage_fall, "a seek ends the damage fall")
	# (Sim 148.) A run let go slides on in the idle state, its state frame running on from the run's: the stop is braced from the run's
	# end, by the view's own count, and a seek ends it.
	var runner := snap(0, "Run", "", 6000, 30)
	runner.vel = Vector2(0.25, 0.0)
	hurt.apply(Vector3.ZERO, runner, 1.0 / 60.0)
	var stopping := snap(0, "Idle", "", 6001, 31)
	stopping.vel = Vector2(0.23, 0.0)
	hurt.apply(Vector3.ZERO, stopping, 1.0 / 60.0)
	check(hurt.stop_k >= 0.0 and hurt.current_clip.begins_with("run_stop"), "a run's stop is braced (%s)" % hurt.current_clip)
	stopping.frame = 6400
	hurt.apply(Vector3.ZERO, stopping, 1.0 / 60.0)
	check(hurt.stop_k < 0.0, "a seek ends the stop")
	# (Sim 149.) A tech roll (kind 1) of 40 frames that moves until frame 24 has rolled over by then and stands up over the rest; a shield
	# roll (kind 0) keeps its progress by its whole length.
	var tech := {"state": "Roll", "state_frame": 24, "dodge": PackedInt32Array([40, 1, 0, 40, 1, 24])}
	check(absf(FighterView.dodge_progress(tech) - FighterView.ROLL_MOVE_END) < 0.001, "a tech roll has rolled over by its last moving frame")
	tech.state_frame = 40
	check(absf(FighterView.dodge_progress(tech) - 1.0) < 0.001, "a tech roll stands up by its end")
	var shield_roll := {"state": "Roll", "state_frame": 15, "dodge": PackedInt32Array([30, 1, 0, 30, 0, 24])}
	check(absf(FighterView.dodge_progress(shield_roll) - 0.5) < 0.001, "a shield roll goes by its whole length")

	# (Round 9.) A knockdown slams down: lying by its third frame (laid down by the lean's spring it tipped over like a felled tree, flat by
	# its sixth to ninth).
	var slammer := FighterView.new()
	root.add_child(slammer)
	slammer.build(1, Loadout.default_for(1))
	await process_frame
	for f in 10:
		slammer.apply(Vector3.ZERO, snap(1, "Idle", "", 8500 + f, f), 1.0 / 60.0)
	for f in 3:
		slammer.apply(Vector3.ZERO, snap(1, "Knockdown", "", 8510 + f, f), 1.0 / 60.0)
	var lying_lean: float = FighterView.KNOCKDOWN_BACK if slammer.lying_back else float(FighterView.LEAN.Knockdown)
	check(absf(slammer.lean - lying_lean) < absf(lying_lean) * 0.1, "a knockdown is down by its third frame (%.1f of %.1f)" % [slammer.lean, lying_lean])

	# (Round 9.) A trip sits on its seat (the get-up's sitting pose), not face down under a crouch, and its stand-up starts from there.
	var trip := snap(0, "Knockdown", "", 8000, 5)
	trip.tripped = true
	hurt.apply(Vector3.ZERO, trip, 1.0 / 60.0)
	check(hurt.current_clip.begins_with("getup_up") and hurt.trip_seat, "a trip sits on its seat (%s)" % hurt.current_clip)
	var up := snap(0, "GetUp", "", 8001, 0)
	up.dodge = PackedInt32Array([26, 0, 0, 26])
	hurt.apply(Vector3.ZERO, up, 1.0 / 60.0)
	check(hurt._getup_k(up) >= FighterView.TRIP_SEAT - 0.001, "a trip's stand-up starts from the seat (%.2f)" % hurt._getup_k(up))

	# (Round 9, attack weight.) A grounded strike's lean is held through its hold where the strike left it: the Longsword's forward tilt
	# rises from a low cut in front to over the head, and its body went on pitching back through the hold; a seek into the hold leans as
	# the hit says.
	var cut := FighterView.new()
	root.add_child(cut)
	cut.build(0, Loadout.default_for(0))
	await process_frame
	var tips := [Vector3(2.2, 0.3, 0.7), Vector3(2.4, 1.1, 0.7), Vector3(2.2, 1.9, 0.7), Vector3(1.7, 2.6, 0.7)]
	var held_at := {}
	for f in 30:
		var st := snap(0, "Attack", "ftilt", 7000 + f, f)
		st.move_timing = PackedInt32Array([32, 7, 10])
		st.move_tip = tips[clampi(f - 7, 0, 3)]
		cut.apply(Vector3.ZERO, st, 1.0 / 60.0)
		if f == 13 or f == 24:
			held_at[f] = cut.lean
	check(held_at[13] > 4.0 and absf(held_at[24] - held_at[13]) < 1.5, "a strike's lean holds through the hold (%.1f, then %.1f)" % [held_at[13], held_at[24]])
	var sought := snap(0, "Attack", "ftilt", 9000, 20)
	sought.move_timing = PackedInt32Array([32, 7, 10])
	sought.move_tip = tips[3]
	cut.apply(Vector3.ZERO, sought, 1.0 / 60.0)
	check(absf(cut.lean) < 0.5, "a seek into the hold leans as the hit says (%.1f)" % cut.lean)

	# (Round 9.) A dash's push-off: a foot planted as the dash sets off stays where it stood while the body goes (it slid with the body).
	var dasher := FighterView.new()
	root.add_child(dasher)
	dasher.build(0, Loadout.default_for(0))
	await process_frame
	for f in 30:
		dasher.apply(Vector3.ZERO, snap(0, "Idle", "", 9500 + f, f), 1.0 / 60.0)
	var stood: Dictionary = dasher.feet_last.duplicate(true)
	var dash_go := snap(0, "Dash", "", 9530, 0)
	dash_go.vel = Vector2(0.28, 0.0)
	dasher.apply(Vector3(0.28, 0.0, 0.0), dash_go, 1.0 / 60.0)
	var most := 0.0
	for side in stood:
		if float(stood[side][1]) < 0.03:
			most = maxf(most, absf((dasher.feet_last[side][0] as Vector3).x - (stood[side][0] as Vector3).x))
	check(most < 0.02, "a dash's planted feet stay where they stood as it sets off (moved %.3f)" % most)

	print("motion test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
