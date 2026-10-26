extends SceneTree
## The sword swing's A/B (round 5): F10 in a match flips the Longsword's swing between the snap and the sweep, shows a note saying which,
## and keeps the choice in user://animation.cfg (restored to what it was when the test ends). Presentation only: the sim is untouched.
## Run (a window, for the screenshot): Godot --path godot --script res://tests/swing_toggle_test.gd [-- --out=<png>]

const Roster := preload("res://scripts/roster.gd")
const FighterView := preload("res://scripts/fighter_view.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func key(code: int) -> InputEventKey:
	var ev := InputEventKey.new()
	ev.keycode = code
	ev.physical_keycode = code
	ev.pressed = true
	return ev


func _initialize() -> void:
	if not ClassDB.class_exists("SimRunner"):
		print("bridge not loaded")
		quit(1)
		return
	var out := ""
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
	FighterView.load_sweep()
	var was: bool = FighterView.sword_sweep
	Roster.session = {"from_menu": true, "stocks": 3, "time": 0, "skip_countdown": true}
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame
	var hash_before: String = main.sim.content_hash() if main.sim.has_method("content_hash") else ""
	main._unhandled_key_input(key(KEY_F10))
	await process_frame
	check(FighterView.sword_sweep != was, "F10 flips the swing")
	check(main.note_label != null and main.note_label.text == "Sword swing: " + ("sweep" if FighterView.sword_sweep else "snap"),
		"and says which: " + (main.note_label.text if main.note_label != null else "no note"))
	var cfg := ConfigFile.new()
	check(cfg.load(FighterView.SWEEP_PATH) == OK and bool(cfg.get_value("swing", "sweep", not FighterView.sword_sweep)) == FighterView.sword_sweep,
		"and keeps it in the user's settings")
	if out != "":
		await create_timer(0.3).timeout
		var img := root.get_viewport().get_texture().get_image()
		img.save_png(out)
	main._unhandled_key_input(key(KEY_F10))
	await process_frame
	check(FighterView.sword_sweep == was, "F10 again flips it back")
	if hash_before != "":
		check(main.sim.content_hash() == hash_before, "the content is untouched")
	FighterView.set_sweep(was)
	main.queue_free()
	await process_frame
	print("swing toggle test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
