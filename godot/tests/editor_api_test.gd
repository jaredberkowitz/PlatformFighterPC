extends SceneTree
## The editors' API end to end, as a creator would use it: open the roster, change values, get told about mistakes, undo,
## add a new fighter and a new stage, save the bundle, and play a match with it, all without touching code.
## Run: Godot --headless --path godot --script res://tests/editor_api_test.gd

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func field(tree: Dictionary, name: String) -> String:
	for it in tree.items:
		if it.t == "field" and it.name == name:
			return it.value
	return ""


func set_field(tree: Dictionary, name: String, value: String) -> void:
	for it in tree.items:
		if it.t == "field" and it.name == name:
			it.value = value
			return
	tree.items.append({"t": "field", "name": name, "value": value})


func _initialize() -> void:
	if not ClassDB.class_exists("ContentEditor"):
		print("bridge not loaded")
		quit(1)
		return
	var ed = ClassDB.instantiate("ContentEditor")
	var sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	var built_in: String = sim.content_hash()

	ed.new_from_builtin("Test Pack")
	check(ed.is_valid(), "the built-in roster is valid")
	var kinds := []
	for s in ed.sections():
		kinds.append(s.kind)
	check(kinds.has("fighter") and kinds.has("weapon") and kinds.has("stage") and kinds.has("ruleset"), "sections: " + str(kinds))
	check(ed.param_names().size() >= 70 and ed.move_keys().size() >= 38, "the parameter and move lists are available")

	# Change a value; the edit is checked at once.
	var duelist: Dictionary = ed.get_section("fighter", "duelist")
	var old_weight := field(duelist, "weight")
	set_field(duelist, "weight", "101")
	check(ed.put_section(duelist).size() == 0, "a valid edit has no problems")
	check(sim.load_content_text(ed.text(false)) == "", "the edited content loads")
	check(sim.content_hash() != built_in, "and plays differently (the hash changed)")

	# A mistake is reported with its field name, nothing can be saved, and undo takes it back.
	set_field(duelist, "walk_speed", "fast")
	var errs: PackedStringArray = ed.put_section(duelist)
	check(errs.size() > 0 and "walk_speed" in errs[0], "a bad value is named: " + str(errs))
	check(not ed.is_valid() and ed.text(false) == "", "an invalid document has no text")
	check(ed.save("user://never.pfc", false) != "", "an invalid document cannot be saved")
	check(ed.undo(), "undo")
	check(ed.is_valid(), "undo returns to a valid document")
	check(field(ed.get_section("fighter", "duelist"), "weight") == "101", "the earlier good edit is still there")
	check(ed.undo(), "undo the weight change too")
	check(field(ed.get_section("fighter", "duelist"), "weight") == old_weight, "back to the original weight")
	check(ed.redo(), "redo")

	# Range problems come from the same validator the game uses.
	var d2: Dictionary = ed.get_section("fighter", "duelist")
	set_field(d2, "gravity", "0")
	errs = ed.put_section(d2)
	check(errs.size() > 0 and "gravity" in errs[0], "validation catches zero gravity: " + str(errs))
	ed.undo()

	# A new fighter and a new stage, then save and play.
	ed.new_from_builtin("My Pack")
	var sprinter := {"kind": "fighter", "name": "sprinter", "items": []}
	set_field(sprinter, "inherit", "brawler")
	set_field(sprinter, "walk_speed", "0.45")
	set_field(sprinter, "run_speed", "0.5")
	set_field(sprinter, "dash_speed", "0.6")
	check(ed.put_section(sprinter).size() == 0, "a new fighter that inherits is valid")

	var stage: Dictionary = ed.get_section("stage", "")
	stage.name = "my_stage"
	stage.items.append({"t": "block", "kind": "platform", "name": "", "items": [
		{"t": "field", "name": "left", "value": "-2"},
		{"t": "field", "name": "right", "value": "2"},
		{"t": "field", "name": "y", "value": "7"},
		{"t": "field", "name": "bottom", "value": "7"},
		{"t": "field", "name": "pass_through", "value": "true"}]})
	check(ed.put_section(stage).size() == 0, "a stage with an extra platform is valid")
	var names := []
	for s in ed.sections():
		if s.kind == "stage":
			names.append(s.name)
	check(names == ["my_stage"], "the stage was replaced, not duplicated: " + str(names))

	var path := ProjectSettings.globalize_path("user://my_pack.pfc")
	check(ed.save(path, true) == "", "the bundle saves")
	check(sim.load_content(path) == "", "the game loads the saved bundle")
	check(sim.content_name() == "My Pack", "with its name")
	check(Array(sim.fighter_names()) == ["duelist", "brawler", "bruiser", "sprinter"], "including the new fighter: " + str(sim.fighter_names()))
	check(sim.platform_count() == 4, "and the new platform: %d" % sim.platform_count())

	# Play a short match with the new fighter: it really is faster than the brawler it came from.
	var run := func(who: int) -> float:
		sim.start(3, PackedInt32Array([who, 1, 0, 1]))
		sim.debug_stand(0, -9.0, 1)
		var x0: float = sim.fighter_pos(0).x
		for t in 40:
			sim.set_input(0, 127, 0, 0)
			sim.tick()
		return sim.fighter_pos(0).x - x0
	var slow: float = run.call(1)
	var fast: float = run.call(3)
	check(fast > slow * 1.2, "the new fighter runs faster: %f vs %f" % [fast, slow])

	# The saved file is a normal bundle: it reopens in the editor with the new content.
	var ed2 = ClassDB.instantiate("ContentEditor")
	check(ed2.open_file(path) == "" and ed2.is_valid(), "the saved bundle reopens")
	check(ed2.get_section("fighter", "sprinter").size() > 0, "with the new fighter in it")

	print("editor api test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
