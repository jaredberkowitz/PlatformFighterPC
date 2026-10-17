extends SceneTree
## Photographs the pause screen in a battle (needs a window: run without --headless).
## Run: Godot --path godot --script res://tests/pause_shot.gd -- --out=<folder>

const Roster := preload("res://scripts/roster.gd")


func _initialize() -> void:
	var out := "user://"
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
	Roster.session = {"from_menu": true, "stocks": 3, "time": 0, "skip_countdown": true}
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	await create_timer(2.0).timeout
	var ev := InputEventKey.new()
	ev.keycode = KEY_ESCAPE
	ev.pressed = true
	main._input(ev)
	await create_timer(0.6).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("pause.png"))
	quit()
