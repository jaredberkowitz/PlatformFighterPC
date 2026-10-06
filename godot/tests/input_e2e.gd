extends SceneTree
## End-to-end check of keyboard input to sim, headless: sends real key events through Godot's Input,
## reads them with InputReader exactly as the game does, ticks the Rust sim, and prints what happened.
## Run: Godot --headless --path godot --script res://tests/input_e2e.gd

const InputReader := preload("res://scripts/input_reader.gd")

var sim
var masks := {}
var failures := 0


func _key(code: Key, down: bool) -> void:
	var e := InputEventKey.new()
	e.physical_keycode = code
	e.keycode = code
	e.pressed = down
	Input.parse_input_event(e)


## pattern: array of [frames, s_key_down]. Returns the fast_fall flag history and raw stick_y values.
func _run_pattern(label: String, pattern: Array, expect_fast_fall: bool) -> void:
	InputReader._down_state.clear()
	sim.start(1, PackedInt32Array([0, 1, 0, 1]))
	sim.debug_place_airborne(0, 0.0, 60.0)
	for _i in 12:  # fall for a bit with no keys
		await physics_frame
		var r: Dictionary = InputReader.read(0, masks)
		sim.set_input(0, r.x, r.y, r.buttons)
		sim.tick()
	var ys: Array = []
	var fast := false
	var fast_at := -1
	var n := 0
	for step in pattern:
		_key(KEY_S, step[1])
		for _i in step[0]:
			await physics_frame
			var r: Dictionary = InputReader.read(0, masks)
			sim.set_input(0, r.x, r.y, r.buttons)
			sim.tick()
			ys.append(r.y)
			var info: PackedInt32Array = sim.fighter_info(0)
			if info[4] != 0 and not fast:
				fast = true
				fast_at = n
			n += 1
	_key(KEY_S, false)
	var ok := fast == expect_fast_fall
	if not ok:
		failures += 1
	print("%s  %-30s fast_fall=%s (expected %s)" % ["PASS" if ok else "FAIL", label, fast, expect_fast_fall])
	if not ok:
		print("   stick_y: ", ys)


func _initialize() -> void:
	if not ClassDB.class_exists("SimRunner"):
		print("bridge not loaded")
		quit(1)
		return
	sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	for k in ["jump", "attack", "special", "shield", "grab"]:
		masks[k] = sim.button_mask(k)
	await _run_pattern("single tap", [[3, true], [20, false]], false)
	await _run_pattern("hold down", [[20, true]], false)
	await _run_pattern("quick double tap 3f/3f/3f", [[3, true], [3, false], [3, true], [10, false]], true)
	await _run_pattern("double tap 5f/6f/5f", [[5, true], [6, false], [5, true], [10, false]], true)
	await _run_pattern("relaxed double tap 4f/16f/4f", [[4, true], [16, false], [4, true], [10, false]], true)
	await _run_pattern("slow double tap 4f/20f/4f", [[4, true], [20, false], [4, true], [10, false]], true)
	await _run_pattern("two separate taps 4f/40f/4f", [[4, true], [40, false], [4, true], [10, false]], false)
	print("e2e input test: %s" % ("ALL PASSED" if failures == 0 else "%d FAILED" % failures))
	quit(1 if failures > 0 else 0)
