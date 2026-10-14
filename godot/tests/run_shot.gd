extends SceneTree
## Photographs the single-player run: running right through the first rooms with the keys a player would press (the map at the
## top), running back to the left wall, the shop's panel, then the results (needs a window, so run it without --headless).
## Run: Godot --path godot --script res://tests/run_shot.gd -- --out=<folder>

const Roster := preload("res://scripts/roster.gd")


func _initialize() -> void:
	var out := "user://"
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
	Roster.session = {"from_menu": true, "run_entry": Roster.builtins()[0], "run_seed": 7}
	var main: Node = load("res://runner.tscn").instantiate()
	root.add_child(main)
	await create_timer(2.0).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("run_start.png"))
	main._send_key(KEY_D, true)
	for shot in 6:
		for k in 8:
			main._send_key(KEY_J, true)
			await create_timer(0.05).timeout
			main._send_key(KEY_J, false)
			if k % 3 == 0:
				main._send_key(KEY_SPACE, true)
				await create_timer(0.05).timeout
				main._send_key(KEY_SPACE, false)
			await create_timer(0.25).timeout
		root.get_viewport().get_texture().get_image().save_png(out.path_join("run_%d.png" % shot))
		print("shot %d: %s" % [shot, str(main.sim.run_info())])
	main._send_key(KEY_D, false)
	# Back to the wall (the oldest room kept is at most two behind).
	main._send_key(KEY_A, true)
	await create_timer(6.0).timeout
	main._send_key(KEY_A, false)
	await create_timer(0.6).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("run_wall.png"))
	print("wall: view %s info %s" % [str(main.sim.run_view()), str(main.sim.run_info())])
	# The shop's panel (as the sim gives it when open).
	var data: Dictionary = main.run_hud.board.data.duplicate()
	data["shop"] = PackedInt32Array([1, 0, 1, 2, 75, 2, 0, 30, 0, 1, 1, 150, 1, 0])
	data["shop_names"] = main.sim.run_shop_names()
	data["banner"] = ""
	main.set_process(false)
	main.run_hud.show_state(data)
	await create_timer(0.3).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("run_shop.png"))
	main.set_process(true)
	for life in 3:
		main.sim.debug_place_airborne(0, main.sim.run_view()[0] - 20.0, 5.0)
		await create_timer(0.3).timeout
	await create_timer(2.5).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("run_results.png"))
	main.queue_free()
	# The way in: the main menu's Run tile, then the fighter choice in its run mode.
	Roster.session = {}
	var menu: Node = load("res://menu.tscn").instantiate()
	root.add_child(menu)
	await create_timer(2.0).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("run_menu.png"))
	menu.queue_free()
	Roster.session = {"mode": "run"}
	var select: Node = load("res://select.tscn").instantiate()
	root.add_child(select)
	await create_timer(2.0).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("run_select.png"))
	quit(0)
