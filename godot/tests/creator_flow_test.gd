extends SceneTree
## The creator-to-battle flow: names are checked, fighters are saved and reloaded, a match is assembled with them, and the
## stats really change how they move in the simulation. Then the screens themselves are driven with key presses.
## Run: Godot --headless --path godot --script res://tests/creator_flow_test.gd

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


func entry(name: String, class_id: int, size: int, speed := 5, jump := 5, weight := 5) -> Dictionary:
	var e := Roster.neutral_entry(name, class_id, Loadout.default_for(0))
	e.size = size
	e.speed = speed
	e.jump = jump
	e.weight = weight
	return e


func _initialize() -> void:
	if not ClassDB.class_exists("ContentEditor"):
		print("bridge not loaded")
		quit(1)
		return
	for e in Roster.saved():
		if e.slug.begins_with("zz_test"):
			Roster.delete(e.slug)
	_names()
	await _saving_and_matches()
	await _screens()
	for e in Roster.saved():
		if e.slug.begins_with("zz_test"):
			Roster.delete(e.slug)
	print("creator flow test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


func _names() -> void:
	check(Roster.name_problem("") != "", "an empty name is refused")
	check(Roster.name_problem("   ") != "", "a blank name is refused")
	check(Roster.name_problem("bad/name") != "", "odd characters are refused")
	check(Roster.name_problem("x".repeat(25)) != "", "a long name is refused")
	check(Roster.name_problem("Duelist") != "", "a built-in name is refused")
	check(Roster.name_problem("duelist") != "", "even in another case")
	check(Roster.name_problem("zz_test Good Name") == "", "a normal name is fine")
	check(Roster.slug_of("Big Bob!") == "big_bob", "slugs are tidy: " + Roster.slug_of("Big Bob!"))


func _saving_and_matches() -> void:
	var big := entry("zz_test Big", 0, 9)
	var tiny := entry("zz_test Tiny", 0, 1)
	check(Roster.save(big) == "" and Roster.save(tiny) == "", "fighters save")
	var names := []
	for e in Roster.saved():
		names.append(e.name)
	check(names.has("zz_test Big") and names.has("zz_test Tiny"), "and are listed again: " + str(names))
	var back: Dictionary = Roster.from_json(Roster.to_json(big))
	check(back.size == 9 and back.name == "zz_test Big" and back["class"] == 0 and back.look.equals(big.look), "a fighter survives its file")
	check(Roster.from_json("not json").is_empty(), "a damaged file is ignored")
	check(Roster.from_json('{"name": "x", "size": 99, "speed": -4}').size == 9, "stats from a file are kept in range")

	# Assemble a match with both and run it.
	var built: Dictionary = Roster.build_content([big, tiny])
	check(built.error == "", "content builds: " + built.error)
	check(built.chars.size() == 2 and built.chars[0] != built.chars[1], "each fighter has its own index: " + str(built.chars))
	var sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	check(sim.load_content_text(built.text) == "", "the game loads it")
	check(Array(sim.fighter_names()).has("zz_test_big") and Array(sim.fighter_names()).has("zz_test_tiny"), "with both fighters: " + str(sim.fighter_names()))
	var run_distance := func(index: int) -> float:
		sim.start(5, PackedInt32Array([index, 0, 0, 0]))
		sim.debug_stand(0, -9.0, 1)
		var x0: float = sim.fighter_pos(0).x
		for t in 50:
			sim.set_input(0, 127, 0, 0)
			sim.tick()
		return sim.fighter_pos(0).x - x0
	var jump_peak := func(index: int) -> float:
		sim.start(5, PackedInt32Array([index, 0, 0, 0]))
		sim.debug_stand(0, -9.0, 1)
		var y0: float = sim.fighter_pos(0).y
		var peak := 0.0
		for t in 70:
			sim.set_input(0, 0, 0, sim.button_mask("jump") if t < 20 else 0)
			sim.tick()
			peak = maxf(peak, sim.fighter_pos(0).y - y0)
		return peak
	var big_i: int = built.chars[0]
	var tiny_i: int = built.chars[1]
	check(run_distance.call(tiny_i) > run_distance.call(big_i) + 0.5, "the small fighter runs faster than the big one: %f vs %f" % [run_distance.call(tiny_i), run_distance.call(big_i)])
	check(jump_peak.call(tiny_i) > jump_peak.call(big_i) + 0.5, "and jumps higher: %f vs %f" % [jump_peak.call(tiny_i), jump_peak.call(big_i)])

	# The same two fighters in a different order keep their own numbers.
	var swapped: Dictionary = Roster.build_content([tiny, big])
	check(swapped.chars[0] != swapped.chars[1], "the order does not matter")

	# Built-in fighters play exactly as before: the match content for them is the base roster.
	var plain: Dictionary = Roster.build_content(Roster.builtins())
	check(plain.chars == [0, 1], "built-in fighters keep their own slots: " + str(plain.chars))
	var base_sim = ClassDB.instantiate("SimRunner")
	root.add_child(base_sim)
	var built_in_hash: String = base_sim.content_hash()
	check(base_sim.load_content_text(plain.text) == "", "the base roster loads")
	check(base_sim.content_hash() == built_in_hash, "and is the same content as the built-in one")

	# Renaming and deleting.
	Roster.delete("zz_test_tiny")
	var left := []
	for e in Roster.saved():
		left.append(e.slug)
	check(not left.has("zz_test_tiny") and left.has("zz_test_big"), "a deleted fighter is gone, the others stay")
	check(Roster.name_problem("zz_test Big", "zz_test_big") == "", "a fighter can be saved again under its own name")
	check(Roster.name_problem("zz_test Big", "") != "", "but a new fighter cannot take a saved name")
	await process_frame


func _screens() -> void:
	# ---- Creator ----
	var creator: Control = load("res://creator.tscn").instantiate()
	root.add_child(creator)
	await process_frame
	await process_frame
	check(creator.selectors["class"].index == 0 and creator.stat_rows["size"].value == 5, "a new fighter starts neutral")
	# Casual rules: this test raises stats freely (the budget has its own test).
	creator.selectors["rules"].set_index(1)
	creator._set_focus(creator.rows.find(creator.rows.filter(func(r): return r.id == "size")[0]))
	creator._unhandled_key_input(key(KEY_RIGHT))
	creator._unhandled_key_input(key(KEY_RIGHT))
	check(creator.stat_rows["size"].value == 7, "right arrow raises the focused stat")
	creator._unhandled_key_input(key(KEY_LEFT))
	check(creator.stat_rows["size"].value == 6, "left lowers it")
	var before: Dictionary = creator.bars.values.duplicate()
	creator.stat_rows["size"].set_value(9)
	check(creator.bars.values["Weight"] > before["Weight"] and creator.bars.values["Run speed"] < before["Run speed"], "bigger shows heavier and slower in the bars")
	check(creator.preview.size_percent > 120.0, "and a bigger fighter on the stand: %f" % creator.preview.size_percent)
	creator.name_edit.text = "zz_test Made"
	creator.selectors["hat"].set_index(4)
	var made: Dictionary = creator.current_entry()
	check(made.name == "zz_test Made" and made.size == 9 and made.look.hat == 4, "the entry reflects the screen")
	check(Roster.save(made) == "", "it saves")
	creator.queue_free()
	await process_frame
	var reopened: Control = load("res://creator.tscn").instantiate()
	Roster.edit_slug = "zz_test_made"
	root.add_child(reopened)
	await process_frame
	check(reopened.name_edit.text == "zz_test Made" and reopened.stat_rows["size"].value == 9 and reopened.selectors["hat"].index == 4, "a saved fighter reopens for editing")
	reopened.name_edit.text = ""
	check(Roster.name_problem(reopened.current_entry().name, reopened.editing_slug) != "", "saving without a name is refused")
	reopened.queue_free()

	# ---- Character select ----
	var select: Control = load("res://select.tscn").instantiate()
	root.add_child(select)
	await process_frame
	await process_frame
	var saved_index := -1
	for i in select.entries.size():
		if select.entries[i].slug == "zz_test_made":
			saved_index = i
	check(saved_index >= 2, "a saved fighter is in the select grid after the built-in ones")
	select.cursor[0] = saved_index
	select._unhandled_key_input(key(KEY_J))
	check(select.locked[0] and select.picked[0].slug == "zz_test_made", "player 1 locks in the saved fighter")
	select._unhandled_key_input(key(KEY_K))
	check(not select.locked[0], "and can take it back")
	select._unhandled_key_input(key(KEY_D))
	select._unhandled_key_input(key(KEY_J))
	select._unhandled_key_input(key(KEY_LEFT))
	select._unhandled_key_input(key(KEY_ENTER))
	check(select.locked[0] and select.locked[1], "both players can lock in")
	check(select.start_button.visible, "and then the start button shows")
	select.queue_free()

	# ---- Main menu ----
	var menu: Control = load("res://menu.tscn").instantiate()
	root.add_child(menu)
	await process_frame
	check(menu.buttons.size() == 6 and menu.index == 0, "the menu starts on Play")
	menu._unhandled_key_input(key(KEY_DOWN))
	check(menu.index == 1, "down moves to Online")
	menu._unhandled_key_input(key(KEY_UP))
	menu._unhandled_key_input(key(KEY_UP))
	check(menu.index == 5, "and it wraps")
	menu.queue_free()
