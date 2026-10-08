extends SceneTree
## Photographs the in-match HUD and the results screen (needs a window, so run it without --headless).
## Run: Godot --path godot --script res://tests/hud_shot.gd -- --out=<folder>

const Roster := preload("res://scripts/roster.gd")


func _initialize() -> void:
	var out := "user://"
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
	Roster.session = {"from_menu": true, "stocks": 3, "time": 300}
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	await create_timer(2.5).timeout
	main.sim.debug_set_percent(0, 34.0)
	main.sim.debug_set_percent(1, 142.0)
	await create_timer(0.6).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("hud_play.png"))
	for k in 3:
		main.sim.debug_place_airborne(1, 500.0, 0.0)
		await create_timer(0.7).timeout
	await create_timer(0.3).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("hud_game.png"))
	await create_timer(2.6).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("hud_results.png"))
	quit(0)
