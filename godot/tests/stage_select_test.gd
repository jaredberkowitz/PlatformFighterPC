extends SceneTree
## Stage select between fighter select and the match: Start on fighter select opens it on the preview view, V switches to the full view
## on the same stage, the last card is Random, Esc goes back with everyone still locked in, the view is remembered, and Enter starts the
## match on the chosen stage. The player's own view and default stage are put back afterwards.
## Run: Godot --headless --path godot --script res://tests/stage_select_test.gd
## (`-- --out=<folder>` in a window also photographs the stage screens.)
const Roster := preload("res://scripts/roster.gd")

var out := ""
var failed := false


func key(code: int) -> void:
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		Input.parse_input_event(ev)
		await process_frame


func shot(name: String) -> void:
	if out == "":
		return
	root.get_viewport().get_texture().get_image().save_png(out.path_join(name))
	print("shot ", name)


func check(ok: bool, what: String) -> void:
	print(("PASS  " if ok else "FAIL  ") + what)
	if not ok:
		failed = true


func _initialize() -> void:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
	var view_file := "user://stage_view.txt"
	var had_view := FileAccess.file_exists(view_file)
	var old_view := FileAccess.get_file_as_string(view_file) if had_view else ""
	if had_view:
		DirAccess.remove_absolute(ProjectSettings.globalize_path(view_file))
	Roster.load_match_rules()
	var old_stage := Roster.match_stage
	Roster.match_stage = 0
	Roster.save_match_rules()
	change_scene_to_file("res://select.tscn")
	await create_timer(2.0).timeout
	for k in [KEY_D, KEY_J, KEY_LEFT, KEY_ENTER]:
		await key(k)
		await create_timer(0.15).timeout
	await key(KEY_ENTER)
	await create_timer(2.0).timeout
	check(current_scene != null and current_scene.scene_file_path == "res://stage.tscn", "Start on fighter select opens stage select")
	var s = current_scene
	check(s.view == "preview", "it opens on the preview view")
	await key(KEY_RIGHT)
	await create_timer(0.6).timeout
	shot("stage_preview.png")
	var at: int = s.index
	await key(KEY_V)
	await create_timer(0.8).timeout
	check(s.view == "full" and s.index == at, "V switches to the full view on the same stage")
	shot("stage_full.png")
	await key(KEY_RIGHT)
	await key(KEY_RIGHT)
	await key(KEY_RIGHT)
	await key(KEY_RIGHT)
	await create_timer(0.6).timeout
	check(s.index == s.names.size(), "the last card is Random")
	shot("stage_full_random.png")
	await key(KEY_ESCAPE)
	await create_timer(1.5).timeout
	var sel = current_scene
	check(sel.scene_file_path == "res://select.tscn", "Esc goes back to fighter select")
	check(sel.locked[0] and sel.locked[1] and sel.start_button.visible, "with both players still locked in")
	check(sel.picked[0].slug != sel.picked[1].slug, "on the fighters they had")
	await key(KEY_ENTER)
	await create_timer(1.5).timeout
	s = current_scene
	check(s.scene_file_path == "res://stage.tscn" and s.view == "full", "Start again: stage select remembers the full view")
	await key(KEY_LEFT)
	await key(KEY_LEFT)
	await create_timer(0.3).timeout
	var want: int = s.index
	await key(KEY_ENTER)
	await create_timer(1.5).timeout
	check(current_scene.scene_file_path == "res://main.tscn", "Enter starts the match")
	check(int(Roster.session.get("stage", -1)) == want, "on the chosen stage (%d)" % want)
	# Leave the player's own settings as they were.
	Roster.match_stage = old_stage
	Roster.save_match_rules()
	if had_view:
		var f := FileAccess.open(view_file, FileAccess.WRITE)
		f.store_string(old_view)
	else:
		DirAccess.remove_absolute(ProjectSettings.globalize_path(view_file))
	print("stage select test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
