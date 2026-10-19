extends SceneTree
## A close-up sheet of a fighter's hands for judging their shape alone (the owner, 2026-10-19): the arms raised out as in a T-pose, the
## right hand open and then closed, each from the front and from the side, framed tight. Needs a window, so run it without --headless.
## Run: Godot --path godot --script res://tests/hand_sheet.gd -- --out=<file.png> [--look=N]

const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Turnaround := preload("res://tests/turnaround_shot.gd")


func _initialize() -> void:
	var out := "user://hands.png"
	var look := 0
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
		elif a.begins_with("--look="):
			look = int(a.substr(7))
	var bg := ColorRect.new()
	bg.color = Color(0.62, 0.62, 0.64)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_child(bg)
	var row := HBoxContainer.new()
	root.add_child(row)
	var shots := [[false, 0], [false, 1], [true, 0], [true, 1]]
	var views: Array = []
	for sh in shots:
		var pv := Preview.new(Vector2i(320, 320), false, 3.0)
		pv.spin = false
		pv.facing_bias = 0.0
		row.add_child(pv)
		pv.set_loadout(Loadout.default_for(look))
		views.append([pv, sh[0], sh[1]])
	await create_timer(0.3).timeout
	var t: Object = Turnaround.new()
	for entry in views:
		var pv = entry[0]
		t._tpose(pv.view)
		pv.view._clench(entry[1], entry[1])
	await create_timer(0.3).timeout
	for entry in views:
		var pv = entry[0]
		var sk: Skeleton3D = pv.view.skeleton
		var h := sk.find_bone("hand.R")
		var wrist: Vector3 = sk.global_transform * sk.get_bone_global_pose(h).origin
		var arm := sk.find_bone("armL.R")
		var elbow: Vector3 = sk.global_transform * sk.get_bone_global_pose(arm).origin
		var along := (wrist - elbow).normalized()
		var at := wrist + along * 0.22
		# Front: from the camera's side; side: from beyond the fingertips, looking back along the arm.
		var from: Vector3 = at + Vector3(0, 0.15, 2.0) if entry[2] == 0 else at + along * 2.0 + Vector3(0, 0.15, 0.3)
		pv.camera.look_at_from_position(from, at)
	await create_timer(0.6).timeout
	root.get_viewport().get_texture().get_image().save_png(out)
	quit(0)
