extends SceneTree
## Photographs the single-player run: running right through the first rooms with the keys a player would press (the map at the
## top), then (moving room to room) a slope, a fork and the inside of a shop, running back to the left wall, then the results (needs a
## window, so run it without --headless).
## Run: Godot --path godot --script res://tests/run_shot.gd -- --out=<folder>

const Roster := preload("res://scripts/roster.gd")


func _shot(out: String, name: String) -> void:
	root.get_viewport().get_texture().get_image().save_png(out.path_join(name))
	print("shot %s" % name)


func _initialize() -> void:
	var out := "user://"
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
	Roster.session = {"from_menu": true, "run_entry": Roster.builtins()[0], "run_seed": 7}
	var main: Node = load("res://runner.tscn").instantiate()
	root.add_child(main)
	await create_timer(2.0).timeout
	_shot(out, "run_start.png")
	main._send_key(KEY_D, true)
	for shot in 3:
		for k in 8:
			main._send_key(KEY_J, true)
			await create_timer(0.05).timeout
			main._send_key(KEY_J, false)
			if k % 3 == 0:
				main._send_key(KEY_SPACE, true)
				await create_timer(0.05).timeout
				main._send_key(KEY_SPACE, false)
			await create_timer(0.25).timeout
		_shot(out, "run_%d.png" % shot)
		print("  info %s" % str(main.sim.run_info()))
	main._send_key(KEY_D, false)
	# Room to room: a slope (rising, falling or a valley), a fork, and a shop to walk into.
	var got := {"slope": false, "fork": false, "shop": false}
	for n in 40:
		if got.slope and got.fork and got.shop:
			break
		main.sim.run_debug_next_room()
		await create_timer(1.2).timeout
		var info: PackedInt32Array = main.sim.run_info()
		var current: PackedFloat32Array
		for r in main.sim.run_rooms():
			if int(r[5]) == info[8]:
				current = r
		if current.is_empty():
			continue
		if not got.slope and int(current[10]) != 0:
			# On the slope's middle.
			main.sim.debug_place_airborne(0, (current[0] + current[1]) * 0.5, maxf(current[6], current[7]) + 4.0)
			await create_timer(1.0).timeout
			_shot(out, "run_slope.png")
			got.slope = true
		elif not got.fork and current[8] > 0.5:
			# On the high road's first catwalk, looking along both roads.
			main.sim.debug_place_airborne(0, current[0] + 22.0, current[6] + 7.0)
			await create_timer(1.0).timeout
			_shot(out, "run_fork.png")
			got.fork = true
		elif not got.shop and main.sim.run_debug_enter_shop():
			await create_timer(0.6).timeout
			_shot(out, "run_shop_inside.png")
			# Walk to the first ware.
			main._send_key(KEY_D, true)
			await create_timer(0.45).timeout
			main._send_key(KEY_D, false)
			await create_timer(0.8).timeout
			_shot(out, "run_shop_ware.png")
			print("  shop %s" % str(main.sim.run_shop()))
			main._send_key(KEY_A, true)
			await create_timer(0.6).timeout
			main._send_key(KEY_A, false)
			await create_timer(0.3).timeout
			main._send_key(KEY_W, true)
			await create_timer(0.1).timeout
			main._send_key(KEY_W, false)
			await create_timer(0.5).timeout
			got.shop = true
	print("photographed %s" % str(got))
	# Back to the wall (the oldest room kept is at most two behind).
	main._send_key(KEY_A, true)
	await create_timer(6.0).timeout
	main._send_key(KEY_A, false)
	await create_timer(0.6).timeout
	_shot(out, "run_wall.png")
	print("wall: view %s info %s" % [str(main.sim.run_view()), str(main.sim.run_info())])
	for life in 3:
		main.sim.debug_place_airborne(0, main.sim.run_view()[0] + 10.0, main.sim.run_view()[3] - 4.0)
		await create_timer(0.3).timeout
	await create_timer(2.5).timeout
	_shot(out, "run_results.png")
	main.queue_free()
	quit(0)
