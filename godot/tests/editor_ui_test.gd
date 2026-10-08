extends SceneTree
## Drives the real editor scene the way a creator would, through the same functions the buttons and text boxes call:
## change a physics number, copy a fighter, edit and add hitboxes, edit a script, copy a moveset and override a move,
## drag a platform with the mouse, save and reopen the bundle. The content must stay valid throughout and the saved bundle
## must load in the game.
## Run: Godot --headless --path godot --script res://tests/editor_ui_test.gd

var failed := false
var ed: Control


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func field(tree: Dictionary, name: String) -> String:
	for it in tree.items:
		if it.t == "field" and it.name == name:
			return it.value
	return ""


func blocks(tree: Dictionary, kind: String) -> Array:
	var out := []
	for it in tree.items:
		if it.t == "block" and it.kind == kind:
			out.append(it)
	return out


func move_tree(weapon: String, key: String) -> Dictionary:
	for it in ed.editor.get_section("weapon", weapon).items:
		if it.t == "block" and it.kind == "move" and it.name == key:
			return it
	return {}


func _initialize() -> void:
	var scene: PackedScene = load("res://editor.tscn")
	ed = scene.instantiate()
	root.add_child(ed)
	await process_frame
	await process_frame
	var fighter_tab = ed.tab_modules[0]
	var moves_tab = ed.tab_modules[1]
	var stage_tab = ed.tab_modules[2]
	var package_tab = ed.tab_modules[3]
	check(ed.editor.is_valid(), "starts valid")

	# ---- Fighter tab ----
	fighter_tab._commit("weight", "110")
	check(field(ed.editor.get_section("fighter", "duelist"), "weight") == "110", "a physics number was changed")
	check(ed.editor.is_valid(), "and stays valid")
	fighter_tab._commit("gravity", "0")
	check(not ed.editor.is_valid() and ed.problems.text.contains("gravity"), "a bad value shows in the problems list")
	ed.editor.undo()
	ed.changed("")
	check(ed.editor.is_valid(), "undo fixes it")
	fighter_tab.new_name.text = "speedster"
	fighter_tab._copy()
	check(ed.editor.get_section("fighter", "speedster").size() > 0, "a fighter was copied")
	fighter_tab._commit("run_speed", "0.5")
	var speedster: Dictionary = ed.editor.get_section("fighter", "speedster")
	check(field(speedster, "run_speed") == "0.5" and field(speedster, "inherit") == "duelist", "the copy stores only what it changes")
	check(ed.editor.is_valid(), "the copy is valid")

	# ---- Moves tab ----
	moves_tab.pick(PackedStringArray(["longsword", "jab"]))
	var jab := move_tree("longsword", "jab")
	var boxes_before := blocks(jab, "hitbox").size()
	moves_tab._set_row("hitbox", 0, "damage", "7")
	check(field(blocks(move_tree("longsword", "jab"), "hitbox")[0], "damage") == "7", "a hitbox's damage was changed")
	moves_tab._add_row("hitbox", true)
	check(blocks(move_tree("longsword", "jab"), "hitbox").size() == boxes_before + 1, "a hitbox was added")
	moves_tab._delete_row("hitbox", boxes_before)
	check(blocks(move_tree("longsword", "jab"), "hitbox").size() == boxes_before, "and removed")
	moves_tab._set_field("total_frames", "30")
	check(field(move_tree("longsword", "jab"), "total_frames") == "30", "a move's length was changed")
	check(ed.editor.is_valid(), "still valid")
	moves_tab.set_frame(5)
	check(moves_tab.view.frame == 5, "the timeline frame moves")
	# A script, written by hand and checked by the compiler.
	moves_tab.pick(PackedStringArray(["longsword", "down_special"]))
	moves_tab._set_script("script", "if frame == 3 { turn(); }")
	var has_script := false
	for it in move_tree("longsword", "down_special").items:
		has_script = has_script or (it.t == "raw" and it.kind == "script")
	check(has_script and ed.editor.is_valid(), "a script was added")
	moves_tab._set_script("script", "this is not a script")
	check(not ed.editor.is_valid(), "a broken script is reported")
	ed.editor.undo()
	ed.changed("")
	# A new moveset inherits every move; overriding one move leaves the rest inherited.
	moves_tab.new_name.text = "mystic"
	moves_tab._copy_weapon()
	check(moves_tab.weapon == "mystic" and ed.editor.is_valid(), "a moveset was copied")
	moves_tab.pick(PackedStringArray(["mystic", "fair"]))
	var parent_fair := move_tree("longsword", "fair")
	moves_tab._copy_move(parent_fair)
	moves_tab._set_row("hitbox", 0, "damage", "20")
	check(field(blocks(move_tree("mystic", "fair"), "hitbox")[0], "damage") == "20", "the copy of a move was edited")
	check(field(blocks(move_tree("longsword", "fair"), "hitbox")[0], "damage") != "20", "the original is untouched")
	check(move_tree("mystic", "jab").size() == 0, "other moves stay inherited, not copied")
	fighter_tab.pick(PackedStringArray(["speedster"]))
	fighter_tab.weapon_pick.select(fighter_tab.weapons().find("mystic"))
	fighter_tab._weapon_chosen(0)
	check(field(ed.editor.get_section("fighter", "speedster"), "weapon") == "mystic" and ed.editor.is_valid(), "the fighter uses the new moveset")

	# ---- Stage tab: drag a platform with the mouse ----
	ed.tabs.current_tab = 2
	await process_frame
	await process_frame
	var canvas: Control = stage_tab.canvas
	check(canvas.size.x > 100 and canvas.size.y > 100, "the stage canvas has a size: %s" % str(canvas.size))
	var platform: Dictionary = blocks(ed.editor.get_section("stage", ""), "platform")[1]
	var y_before := field(platform, "y").to_float()
	var grab: Vector2 = canvas.to_screen(Vector2((field(platform, "left").to_float() + field(platform, "right").to_float()) / 2.0, y_before))
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = grab
	canvas._gui_input(down)
	var motion := InputEventMouseMotion.new()
	motion.position = grab + Vector2(0, -40)
	canvas._gui_input(motion)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = motion.position
	canvas._gui_input(up)
	platform = blocks(ed.editor.get_section("stage", ""), "platform")[1]
	check(field(platform, "y").to_float() > y_before + 0.5, "dragging a platform up raised it: %f -> %s" % [y_before, field(platform, "y")])
	check(field(platform, "y") == field(platform, "bottom"), "a pass-through platform stays thin")
	var count := blocks(ed.editor.get_section("stage", ""), "platform").size()
	stage_tab._add("platform")
	check(blocks(ed.editor.get_section("stage", ""), "platform").size() == count + 1, "a platform was added")
	stage_tab._delete()
	check(blocks(ed.editor.get_section("stage", ""), "platform").size() == count, "and deleted")
	stage_tab._set_blast("blast_top", "30")
	check(field(ed.editor.get_section("stage", ""), "blast_top") == "30", "the blast zone was edited")
	check(ed.editor.is_valid(), "the stage is valid")

	# ---- Package tab: save and reopen ----
	package_tab._set_manifest("name", "UI Test Pack")
	package_tab._set_manifest("author", "tester")
	package_tab.path_edit.text = ProjectSettings.globalize_path("user://ui_test.pfc")
	package_tab._save(true)
	var reopened = ClassDB.instantiate("ContentEditor")
	check(reopened.open_file(package_tab.path_edit.text) == "" and reopened.is_valid(), "the saved bundle reopens")
	check(field(reopened.get_section("fighter", "speedster"), "weapon") == "mystic", "with the new fighter and moveset")
	var sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	check(sim.load_content(package_tab.path_edit.text) == "", "and the game loads it")
	check(sim.content_name() == "UI Test Pack", "under its name")
	check(Array(sim.fighter_names()).has("speedster"), "with the new fighter")

	# Undo all the way back is possible and keeps the document valid at every step.
	var steps := 0
	while ed.editor.can_undo() and steps < 200:
		ed.editor.undo()
		steps += 1
	ed.changed("")
	check(steps > 10, "a long edit history can be undone (%d steps)" % steps)
	check(ed.editor.is_valid(), "the original content is back")

	print("editor ui test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
