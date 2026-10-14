extends SceneTree
## The single-player run (docs/modes/RUNNER_PLAN.md, R1) through the bridge and the real run scene: a run starts, the stage streams,
## the display reads the run, a lost last life brings up the results, and "Run again" starts a new one.
## Run: Godot --headless --path godot --script res://tests/runner_test.gd

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
	_bridge()
	await _scene()
	await _menu()
	print("runner test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


func _bridge() -> void:
	var sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	check(not sim.run_active(), "no run until one is set up")
	check(not sim.run_start(5), "and none starts")
	var built: Dictionary = Roster.builtins()[0]
	check(sim.run_begin(Roster.spec_bytes(built)) == "", "a built-in fighter sets up a run")
	check(sim.run_start(5), "a run starts")
	check(sim.run_active(), "and is on")
	var info: PackedInt32Array = sim.run_info()
	check(info.size() == 10 and info[0] == 0 and info[7] == 3, "three lives, not over: " + str(info))
	var view: PackedFloat32Array = sim.run_view()
	check(view.size() == 6 and view[1] > 20.0, "the view: " + str(view))
	check(sim.run_pieces().size() > 0 and sim.run_platforms().size() > 0, "the first chunks are laid")
	check(sim.run_mobs().size() == 16 * 12, "sixteen mob slots")
	# Running right: the camera follows and the stage streams on.
	var layout := info[9]
	for n in 600:
		sim.set_input(0, 127, 0, 0)
		sim.tick()
	check(sim.run_view()[0] > view[0] + 5.0, "the camera moved on: %.1f" % sim.run_view()[0])
	check(sim.run_info()[1] > 10, "distance counts: %d m" % sim.run_info()[1])
	check(sim.run_info()[9] != layout, "the stage streamed")
	# A made fighter can run too.
	var made := Roster.neutral_entry("Maker", 3, Roster.Loadout.default_for(0))
	check(sim.run_begin(Roster.spec_bytes(made)) == "", "a made fighter sets up a run")
	check(sim.run_start(9), "and runs")
	# A versus match afterwards is versus again.
	sim.start(3, PackedInt32Array([0, 1]))
	check(not sim.run_active(), "a match ends the run mode")
	sim.queue_free()


func _scene() -> void:
	Roster.session = {"from_menu": true, "run_entry": Roster.builtins()[1], "run_seed": 42}
	var main: Node = load("res://runner.tscn").instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame
	await create_timer(1.5).timeout
	check(main.run_error == "", "the run is set up: " + main.run_error)
	check(main.sim.run_active() and main.run_seed == 42, "the run scene starts the menus' run: seed %d" % main.run_seed)
	check(main.PLAYERS == 1 and main.views.size() == 1, "one fighter")
	check(main.sim.frame() > 30, "and it plays: frame %d" % main.sim.frame())
	var data: Dictionary = main.run_hud.board.data
	check(data.get("name", "") == Roster.builtins()[1].name, "the display names the fighter: " + str(data.get("name", "")))
	check(int(data.get("lives", 0)) == 3 and data.has("metres") and data.has("coins") and data.has("combo"), "and reads the run: " + str(data))
	check(main.run_stage.pieces.size() >= 2, "the stage is drawn chunk by chunk: %d" % main.run_stage.pieces.size())
	check(not main.hud.visible, "the versus display is hidden")
	check(main.results == null, "no results while running")
	# Losing every life ends the run.
	for life in 3:
		main.sim.debug_place_airborne(0, main.sim.run_view()[0] - 20.0, 5.0)
		await create_timer(0.25).timeout
	check(main.sim.run_info()[0] == 1, "out of lives: the run is over")
	await create_timer(2.2).timeout
	check(main.results != null, "the results appear")
	if main.results != null:
		check(int(main.results.stats.get("seed", 0)) == 42, "with the run's seed")
		check(main.results.buttons.size() == 3, "and three choices")
		main.results.handle_key(key(KEY_ENTER))
		await process_frame
		await process_frame
		check(main.results == null and main.sim.run_info()[0] == 0 and main.sim.run_info()[7] == 3, "Run again starts a fresh run")
		check(main.run_seed != 42, "on a new seed")
	main.queue_free()
	await process_frame


func _menu() -> void:
	Roster.session = {}
	var menu: Node = load("res://menu.tscn").instantiate()
	root.add_child(menu)
	await process_frame
	var found := false
	for item in menu.ITEMS:
		if item[0] == "Run" and item[2] == "run":
			found = true
	check(found, "the main menu has a Run tile")
	menu.queue_free()
	await process_frame
	Roster.session = {"mode": "run"}
	var select: Node = load("res://select.tscn").instantiate()
	root.add_child(select)
	await process_frame
	await process_frame
	check(select.run_mode and select.count == 1, "the fighter choice is for one player in a run")
	check(not select.players_button.visible and not select.stage_button.visible, "with no match rules")
	select.queue_free()
	await process_frame
