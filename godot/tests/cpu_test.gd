extends SceneTree
## The basic computer opponent: through the bridge it plays a real match (both sides), always the same way from the same start, and
## the character select's C key makes every player but player 1 a computer.
## Run: Godot --headless --path godot --script res://tests/cpu_test.gd

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
	var first: Array = _play()
	var again: Array = _play()
	check(first[0] > 20.0, "two computer players land hits on each other: %.1f%% dealt" % first[0])
	check(first[1] == again[1], "the same start plays out the same way (%d, %d)" % [first[1], again[1]])
	await _select()
	print("cpu test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


## Fifteen seconds of computer against computer. Returns [damage dealt in all, the final content of the match as a hash].
func _play() -> Array:
	var sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	sim.start(11, PackedInt32Array([0, 1]))
	for frame in 900:
		for p in 2:
			var c: PackedInt32Array = sim.cpu_input(p, 7)
			sim.set_input(p, c[0], c[1], c[2])
		sim.tick()
	var dealt: float = sim.fighter_percent(0) + sim.fighter_percent(1)
	var where := hash([sim.fighter_pos(0), sim.fighter_pos(1), dealt])
	sim.queue_free()
	return [dealt, where]


func _select() -> void:
	var scene: Node = (load("res://select.tscn") as PackedScene).instantiate()
	root.add_child(scene)
	await process_frame
	scene._toggle_cpu()
	check(not scene.cpu[0] and scene.cpu[1], "C makes player 2 a computer, never player 1")
	check(scene.locked[1], "and locks it in on a fighter")
	scene._toggle_cpu()
	check(not scene.cpu[1], "pressed again, a person")
	scene.queue_free()
