extends SceneTree
## The pause screen in a battle and in a run: Esc opens it and stops the match, the keys go to it and not to the fighters, Resume
## carries on, Restart starts over, Camera moves the view and comes back, Hide display hides the display, and Quit asks once more.
## Run: Godot --headless --path godot --script res://tests/pause_test.gd

const Roster := preload("res://scripts/roster.gd")

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
	await _battle()
	await _run()
	print("pause test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


func _press(main: Node, code: int) -> void:
	main._input(key(code))
	await process_frame


func _battle() -> void:
	Roster.session = {"from_menu": true, "stocks": 3, "time": 0, "skip_countdown": true}
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame
	await create_timer(0.6).timeout
	await _press(main, KEY_ESCAPE)
	check(main.pause_menu != null and main.paused, "Esc opens the pause screen and stops the match")
	var frame: int = main.sim.frame()
	await create_timer(0.3).timeout
	check(main.sim.frame() == frame, "the match stands still while paused")
	# Down then Enter: Restart.
	await _press(main, KEY_DOWN)
	check(main.pause_menu.index == 1, "Down moves to Restart")
	await _press(main, KEY_ESCAPE)
	check(main.pause_menu == null and not main.paused, "Esc resumes")
	await create_timer(0.2).timeout
	check(main.sim.frame() > frame, "and the match carries on")
	# Camera: the view moves with the keys, and Esc brings the menu back.
	await _press(main, KEY_ESCAPE)
	main.pause_menu._pick("camera")
	await process_frame
	check(main.pause_menu.camera_mode, "Camera frees the view")
	var before: Vector3 = main.cam.position
	main.pause_cam = Vector3(3, 0, 0)
	main._pause_camera(0.016)
	check(main.cam.position.x > before.x + 2.0, "and moves it")
	await _press(main, KEY_ESCAPE)
	check(main.pause_menu != null and not main.pause_menu.camera_mode, "Esc leaves the free camera for the menu")
	# Hide display.
	main.pause_menu._pick("hud")
	await process_frame
	check(main.pause_menu == null and not main.hud.visible, "Hide display hides the match display and resumes")
	main._show_display(true)
	main.hud_hidden = false
	# Quit asks once more.
	await _press(main, KEY_ESCAPE)
	await _press(main, KEY_UP)
	check(main.pause_menu.actions[main.pause_menu.index] == "quit", "Up from Resume wraps to Quit")
	await _press(main, KEY_ENTER)
	check(main.pause_menu != null and main.pause_menu.confirm_quit, "Quit asks again first")
	await _press(main, KEY_UP)
	check(not main.pause_menu.confirm_quit, "moving away cancels it")
	# Restart.
	main.pause_menu._pick("restart")
	await process_frame
	check(main.pause_menu == null and not main.paused, "Restart closes the menu")
	main.queue_free()
	await process_frame


func _run() -> void:
	Roster.session = {"from_menu": true, "run_entry": Roster.builtins()[0], "run_seed": 7}
	var main: Node = load("res://runner.tscn").instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame
	await create_timer(0.6).timeout
	await _press(main, KEY_ESCAPE)
	check(main.pause_menu != null and main.paused, "a run pauses the same way")
	var frame: int = main.sim.frame()
	await create_timer(0.3).timeout
	check(main.sim.frame() == frame, "and stands still")
	main.pause_menu._pick("hud")
	await process_frame
	check(main.run_hud != null and not main.run_hud.visible, "Hide display hides the run's display too")
	main.queue_free()
	await process_frame
