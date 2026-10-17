extends SceneTree
## Photographs the single-player run: running right through the first rooms with the keys a player would press (the map at the
## top), then (moving room to room) a slope, a fork and the inside of a shop; R2b: each grunt kind (one telling), and the rare event
## (the runaway van, waiting and ridden); R2d: a climb, a drop, a shaft, the interiors, the skyline and each deck; running back to the
## left wall, then the results (needs a window, so run it without
## --headless).
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
	# R2b: the six grunt kinds, one at a time, a few steps from the player (the brawler telling).
	const ROLES := ["brawler", "leaper", "shieldbearer", "spearman", "lobber", "bruiser"]
	main.sim.run_debug_next_room()
	await create_timer(0.8).timeout
	for k in 6:
		main.sim.run_debug_next_room()
		await create_timer(0.3).timeout
		var g: int = main.sim.run_debug_grunt(k, 4.5, k == 0)
		await create_timer(0.7 if k == 0 else 0.45).timeout
		_shot(out, "run_grunt_%s.png" % ROLES[k])
		print("  grunt %s slot %d: %s" % [ROLES[k], g, str(main.sim.run_mobs().slice(g * 17, g * 17 + 17))])
	# Three together: a brawler, a spearman and a shieldbearer.
	main.sim.run_debug_next_room()
	await create_timer(0.3).timeout
	for k in [0, 3, 2]:
		main.sim.run_debug_grunt(k, 3.0 + k * 2.2, false)
	await create_timer(0.9).timeout
	_shot(out, "run_grunts.png")
	# R2c: a grab and a throw on a grunt (the versus grab: held, then thrown back out), and a grunt launched out of the view, knocked
	# out with the knock-out's effect.
	main.sim.run_debug_clear_grunts()
	main.sim.run_debug_next_room()
	await create_timer(0.4).timeout
	main.sim.run_debug_clear_grunts()
	# (The grunt just in front.)
	main._send_key(KEY_D, true)
	await create_timer(0.05).timeout
	main._send_key(KEY_D, false)
	await create_timer(0.3).timeout
	var held: int = main.sim.run_debug_grunt(0, 1.3 * main.sim.fighter_facing(0), false)
	main.sim.run_debug_hold_grunt(held)
	await create_timer(0.1).timeout
	main._send_key(KEY_U, true)
	await create_timer(0.05).timeout
	main._send_key(KEY_U, false)
	await create_timer(0.3).timeout
	_shot(out, "run_grab.png")
	print("  grabbed: %s" % main.sim.fighter_state(100 + held))
	main._send_key(KEY_W, true)
	await create_timer(0.05).timeout
	main._send_key(KEY_W, false)
	await create_timer(0.45).timeout
	_shot(out, "run_throw.png")
	print("  thrown: %s at %s" % [main.sim.fighter_state(100 + held), str(main.sim.fighter_pos(100 + held))])
	await create_timer(1.0).timeout
	main.sim.run_debug_clear_grunts()
	var flying: int = main.sim.run_debug_grunt(3, 2.0, false)
	main.sim.run_debug_launch_grunt(flying, 1.4, 0.35)
	var kos: int = main.grunt_kos
	for n in 40:
		await create_timer(0.03).timeout
		if main.grunt_kos > kos:
			break
	await create_timer(0.12).timeout
	_shot(out, "run_grunt_ko.png")
	print("  knocked out: %s" % str(main.grunt_kos > kos))
	# The rare event: the runaway van, waiting, then ridden.
	main.sim.run_debug_force_event()
	var van := false
	for n in 30:
		main.sim.run_debug_next_room()
		await create_timer(0.15).timeout
		if main.sim.run_info()[10] == 4:
			van = true
			break
	if van:
		await create_timer(0.8).timeout
		var v: PackedFloat32Array = main.sim.run_van()
		main.sim.debug_place_airborne(0, v[1] - 6.0, v[2] + 1.0)
		await create_timer(0.8).timeout
		_shot(out, "run_van.png")
		v = main.sim.run_van()
		main.sim.debug_place_airborne(0, v[1], v[2] + v[4] + 1.0)
		await create_timer(1.6).timeout
		_shot(out, "run_van_ride.png")
		print("  van %s info %s" % [str(main.sim.run_van()), str(main.sim.run_info())])
	print("the van came: %s" % str(van))
	# R2d: verticality. A climb, a drop, a shaft, the interiors and the skyline, each drawn on its own (only it and a few plain
	# rooms come: `run_debug_only`), then each deck.
	var targets := [["escape_climb", "run_climb.png", 0.35], ["great_climb", "run_great_climb.png", 0.5],
		["balcony_drop", "run_drop.png", 0.6], ["light_well", "run_shaft.png", 0.55], ["office", "run_interior.png", 0.4],
		["stairwell", "run_stairwell.png", 0.45], ["rooftop_bar", "run_bar.png", 0.45], ["catwalks", "run_skyline.png", 0.3],
		["cranes", "run_cranes.png", 0.4]]
	for t in targets:
		main.sim.run_debug_only(PackedStringArray([t[0], "street", "water_towers", "balcony_drop"]), 1, 11)
		var hit := false
		for n in 24:
			main.sim.run_debug_next_room()
			await create_timer(0.08).timeout
			var cur := _current(main)
			if cur.size() > 15 and main.sim.run_room_names()[int(cur[15])] == t[0]:
				main.sim.run_debug_clear_grunts()
				var x := lerpf(cur[0], cur[1], t[2])
				main.sim.debug_place_airborne(0, x, maxf(cur[6], cur[7]) + 1.0 if t[0] in ["balcony_drop"] else lerpf(cur[6], cur[7], 0.5) + 3.0)
				await create_timer(1.3).timeout
				_shot(out, t[1])
				hit = true
				break
		print("  %s: %s decks %s" % [t[0], str(hit), str(main.sim.run_decks())])
	# R2e (note B, a believable city): every other room, each on its own, for checking that nothing floats.
	var covered := ["start", "escape_climb", "great_climb", "balcony_drop", "light_well", "office", "stairwell", "rooftop_bar",
		"catwalks", "cranes", "alley", "under_alley", "sky_bridge", "under_bridge"]
	for room_name in main.sim.run_room_names():
		if room_name in covered:
			continue
		main.sim.run_debug_only(PackedStringArray([room_name, "street", "water_towers", "balcony_drop"]), 1, 3)
		var found := false
		for n in 30:
			main.sim.run_debug_next_room()
			await create_timer(0.06).timeout
			var cur := _current(main)
			if cur.size() > 15 and main.sim.run_room_names()[int(cur[15])] == room_name:
				main.sim.run_debug_clear_grunts()
				main.sim.debug_place_airborne(0, lerpf(cur[0], cur[1], 0.45), maxf(cur[6], cur[7]) + 2.0)
				await create_timer(1.2).timeout
				main.sim.run_debug_clear_grunts()
				_shot(out, "run_room_%s.png" % room_name)
				found = true
				break
		print("  room %s: %s" % [room_name, str(found)])
	main.sim.run_debug_only(PackedStringArray(["escape_climb", "street", "water_towers", "catwalks"]), 1, 5)
	var names: PackedStringArray = main.sim.run_deck_names()
	var shot_decks := {}
	for n in 40:
		main.sim.run_debug_next_room()
		await create_timer(0.08).timeout
		var cur := _current(main)
		if cur.size() < 16 or cur[12] != cur[13]:
			continue
		var d := int(cur[12])
		if shot_decks.has(d):
			continue
		main.sim.run_debug_clear_grunts()
		main.sim.debug_place_airborne(0, lerpf(cur[0], cur[1], 0.4), cur[6] + 2.0)
		await create_timer(1.3).timeout
		_shot(out, "run_deck_%s.png" % names[d])
		shot_decks[d] = true
		if shot_decks.size() == names.size():
			break
	print("  decks photographed: %s" % str(shot_decks.keys()))
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


## The current room's row (`run_rooms`), or an empty array.
func _current(main: Node) -> PackedFloat32Array:
	var info: PackedInt32Array = main.sim.run_info()
	for r in main.sim.run_rooms():
		if int(r[5]) == info[8]:
			return r
	return PackedFloat32Array()
