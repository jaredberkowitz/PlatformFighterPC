extends SceneTree
## Prints the body's squash, frame by frame, through a landing that is frozen by hitlag for 12 frames (the sim frame held), for checking
## that the springs hold still in a freeze. Run: Godot --headless --path godot --script res://tests/freeze_probe.gd

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")


var probe_cls := 4


func snap(state: String, frame: int, hitlag: int) -> Dictionary:
	return {"state": state, "facing": 1, "vel": Vector2.ZERO, "kb_vel": Vector2.ZERO, "platform": 0, "fast_fall": false, "invuln": 0,
		"ledge_invuln": 0, "frame": frame, "shield": 1.0, "percent": 0.0, "move_name": "", "move_tip": Vector3.ZERO,
		"move_timing": PackedInt32Array([0, 0, 0]), "charge": 0, "hitlag": hitlag, "tumble": false, "launch_pending": false, "char": probe_cls,
		"class": probe_cls, "reach": 3.0, "state_frame": 0, "jumps": 1, "stocks": 3, "dodge": PackedInt32Array()}


func _initialize() -> void:
	var cls := 4
	var freeze := true
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--class="):
			cls = int(a.substr(8))
		elif a == "--nofreeze":
			freeze = false
	probe_cls = cls
	var view := FighterView.new()
	root.add_child(view)
	view.build(cls, Loadout.default_for(cls))
	await process_frame
	var air := snap("Airborne", 100, 0)
	air.platform = -1
	air.vel = Vector2(0.0, -0.3)
	view.apply(Vector3.ZERO, air, 1.0 / 60.0)
	var out := []
	var frame := 101
	for i in 30:
		var lag := 12 if freeze and i >= 3 and i < 15 else 0
		if lag == 0:
			frame += 1
		view.apply(Vector3.ZERO, snap("Landing" if i < 20 else "Idle", frame, lag), 1.0 / 60.0)
		out.append("%.4f" % view.squash)
	print("SQUASH ", ",".join(out))
	quit(0)
