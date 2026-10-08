extends SceneTree
## Stocks, the winner, the clock, the HUD and the results screen, through the bridge and the real match scene.
## Run: Godot --headless --path godot --script res://tests/match_flow_test.gd

const Roster := preload("res://scripts/roster.gd")
const Loadout := preload("res://scripts/loadout.gd")

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
	await _free_for_all()
	_menu_choices()
	print("match flow test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


func _bridge() -> void:
	var sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	check(sim.match_rules() == PackedInt32Array([0, 0]), "free play by default: " + str(sim.match_rules()))
	sim.set_match_rules(2, 90)
	sim.start(3, PackedInt32Array([0, 1]))
	check(sim.match_rules() == PackedInt32Array([2, 90]), "rules apply at the next start")
	check(sim.fighter_info(0).size() > 0 and sim.fighter_combat(0)[0] == 2, "each fighter starts with the stocks: %d" % sim.fighter_combat(0)[0])
	check(sim.winner() == -1 and sim.fighter_active(1) and sim.fighter_in_roster(1), "nobody has won yet")
	for n in 2:
		sim.debug_place_airborne(1, 500.0, 0.0)
		sim.tick()
	check(not sim.fighter_active(1) and sim.fighter_in_roster(1), "a fighter out of stocks leaves the match")
	check(sim.winner() == 0, "and the other one wins: %d" % sim.winner())
	var frame: int = sim.match_frame()
	sim.tick()
	check(sim.match_frame() == frame + 1 and sim.winner() == 0, "the result stays")
	# Out-of-range values are pulled in.
	sim.set_match_rules(200, 99999)
	sim.start(3, PackedInt32Array([0, 1]))
	check(sim.match_rules()[0] == 9 and sim.match_rules()[1] == 3600, "wild rules are clamped: " + str(sim.match_rules()))

	# The clock decides a match nobody finishes.
	sim.set_match_rules(3, 2)
	sim.start(3, PackedInt32Array([0, 1]))
	sim.debug_set_percent(0, 80.0)
	for t in 130:
		sim.tick()
	check(sim.winner() == 1, "time up: the fighter with less damage wins: %d" % sim.winner())
	sim.queue_free()


func _scene() -> void:
	# No content_text: the scene loads the base roster itself, with the menus' rules.
	Roster.session = {"from_menu": true, "stocks": 1, "time": 0}
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame
	await create_timer(1.0).timeout
	check(main.sim.match_rules()[0] == 1, "the match scene applies the menus' stocks: " + str(main.sim.match_rules()))
	check(main.results == null, "no results while playing")
	main.sim.debug_place_airborne(1, 500.0, 0.0)
	await create_timer(0.5).timeout
	check(main.sim.winner() == 0, "the scene's sim has a winner: %d" % main.sim.winner())
	check(not main.views[1].visible and main.views[0].visible, "the beaten fighter is gone from the stage")
	await create_timer(2.6).timeout
	check(main.results != null, "the results screen appears")
	if main.results != null:
		check(main.results.title.contains("Player 1"), "and names the winner: " + main.results.title)
		check(main.results.buttons.size() == 3, "with three choices")
		main.results.handle_key(key(KEY_ENTER))
		await process_frame
		await process_frame
		check(main.results == null and main.sim.winner() == -1 and main.sim.fighter_active(1), "Rematch starts a fresh match")
	main.queue_free()
	await process_frame


func _free_for_all() -> void:
	var entries := []
	for i in 4:
		var e := Roster.neutral_entry("Fighter %d" % (i + 1), i % 2, Loadout.default_for(i))
		e.size = 4 + i
		entries.append(e)
	Roster.session = {"from_menu": true, "stocks": 1, "time": 0, "entries": entries}
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame
	await create_timer(1.0).timeout
	check(main.PLAYERS == 4, "the match scene builds four fighters: %d" % main.PLAYERS)
	check(main.views.size() == 4 and main.sim.fighter_in_roster(3) and main.sim.fighter_active(3), "all four are in the match")
	check(main.names[2] == "Fighter 3", "with their names: " + str(main.names))
	check(main.sim.fighter_scale(3) > main.sim.fighter_scale(0), "and their own sizes")
	main.sim.debug_place_airborne(1, 500.0, 0.0)
	await create_timer(0.4).timeout
	check(main.sim.winner() == -1 and not main.sim.fighter_active(1), "one out of four: the match goes on")
	main.sim.debug_place_airborne(3, 500.0, 0.0)
	main.sim.debug_place_airborne(2, 500.0, 0.0)
	await create_timer(0.5).timeout
	check(main.sim.winner() == 0, "the last one standing wins: %d" % main.sim.winner())
	await create_timer(2.6).timeout
	check(main.results != null and main.results.card_data.size() == 4, "and the results screen shows four fighters")
	main.queue_free()
	await process_frame


func _menu_choices() -> void:
	check(Roster.stocks_text(0) == "Free play" and Roster.stocks_text(1) == "1 stock" and Roster.stocks_text(3) == "3 stocks", "stock labels")
	check(Roster.time_text(0) == "No limit" and Roster.time_text(300) == "5:00", "time labels")
