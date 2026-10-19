extends SceneTree
## The steady camera (the Display setting "Camera"): a fighter in an ordinary jump leaves the view where it was; one launched well
## above its jump height moves it; with the setting on "Follow" the view rises with the jump, as it did before.
## Run: Godot --headless --path godot --script res://tests/camera_test.gd

const Roster := preload("res://scripts/roster.gd")
const VisualSettings := preload("res://scripts/visual_settings.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func _initialize() -> void:
	if not ClassDB.class_exists("SimRunner"):
		print("bridge not loaded")
		quit(1)
		return
	var kept := VisualSettings.get_index("camera")
	VisualSettings.set_index("camera", 1)
	var steady := await _camera_rise(3.0)
	var launched := await _camera_rise(12.0)
	VisualSettings.set_index("camera", 0)
	var follow := await _camera_rise(3.0)
	VisualSettings.set_index("camera", kept)
	print("camera rise: steady jump %.2f, steady launch %.2f, follow jump %.2f" % [steady, launched, follow])
	check(absf(steady) < 0.05, "steady: an ordinary jump's height leaves the view still (moved %.2f)" % steady)
	check(launched > 1.0, "steady: a fighter far above its jump height moves the view (moved %.2f)" % launched)
	check(follow > 0.5, "follow: the view rises with the jump (moved %.2f)" % follow)
	print("camera test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


## How far the camera's aim rises when player 1 is held `height` units over the spot it stood on (the match otherwise still).
func _camera_rise(height: float) -> float:
	Roster.session = {"from_menu": true, "stocks": 3, "time": 0, "skip_countdown": true}
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame
	await create_timer(1.5).timeout
	var before: float = main.cam_base.y
	var x: float = main.sim.fighter_pos(0).x
	var ground: float = main.sim.fighter_pos(0).y
	for i in 90:
		main.sim.debug_place_airborne(0, x, ground + height)
		main._refresh(0)
		main.prev_pos[0] = main.cur_pos[0]
		await process_frame
	var rise: float = main.cam_base.y - before
	main.queue_free()
	await process_frame
	return rise
