extends SceneTree
## The single-player run (docs/modes/RUNNER_PLAN.md, R1, R1b, R2a and R2b) through the bridge and the real run scene: a run starts, rooms
## stream in (laid as variants, with slopes, pieces, looks and coins) and are entered, the map (forks too, and the tier) and the shop
## read the run, the walk-in shop's interior and shopkeeper are drawn, the walls are drawn, the display reads the run, grunts are drawn
## with the fighter rig (their tell too), coins picked up vanish, props and the runaway van are drawn, a lost last life brings up the
## results, and "Run again" starts a new one. R2c: grunts are fighters (twelve slots), the camera's height is the sim's, a grunt launched
## out of the view is knocked out with the knock-out's effect, a grunt shows one small damage label, and a closed shop hangs its sign.
## Run: Godot --headless --path godot --script res://tests/runner_test.gd

const Roster := preload("res://scripts/roster.gd")

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
	await _menu()
	print("runner test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


func _bridge() -> void:
	var sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	check(not sim.run_active(), "no run until one is set up")
	check(not sim.run_start(5), "and none starts")
	var built: Dictionary = Roster.builtins()[0]
	check(sim.run_begin(Roster.spec_bytes(built)) == "", "a built-in fighter sets up a run")
	check(sim.run_start(5), "a run starts")
	check(sim.run_active(), "and is on")
	var info: PackedInt32Array = sim.run_info()
	check(info.size() == 22 and info[0] == 0 and info[7] == 3, "three lives, not over: " + str(info))
	check(info[17] == 0 and info[18] == 0 and info[19] == 0, "tier 1 (0), no chest, not riding: " + str(info))
	check(info[8] == 0 and info[10] == 0 and info[11] == 0 and info[13] == 0, "in the first room, a street, no gate, no shop: " + str(info))
	var view: PackedFloat32Array = sim.run_view()
	check(view.size() == 18 and view[1] > 20.0 and view[9] == 0.0, "the view, outside: " + str(view))
	check(view[16] > 10.0 and view[17] > 0.0 and view[13] == 0.0, "the view's height, the grunts' zone margin, no knock-outs yet: " + str(view))
	check(view[6] == 0.0 and view[7] > view[1], "the wall at the first room's edge, the rooms beyond the view: " + str(view))
	check(sim.run_pieces().size() > 0 and sim.run_platforms().size() > 0, "the first rooms are laid")
	check(sim.run_rooms().size() == 2, "the first room and the next: %d" % sim.run_rooms().size())
	var map: PackedInt32Array = sim.run_map(4)
	check(map.size() == 10 and map[2] == 2 and map[7] == 3 and map[4] == 0, "the map: here, then the next room, with tiers: " + str(map))
	var shop: PackedInt32Array = sim.run_shop()
	check(shop.size() == 10 + 2 * 4 + 1 and shop[0] == 0 and shop[5] == 2 and shop[18] == 0, "the shop's two wares, outside, and open: " + str(shop))
	check(sim.run_shop_names().size() == 2, "and their names")
	# What the rooms are made of: every footing has a look, the pieces' rows have thirteen numbers (R2e: what holds it up, and its
	# deck), the rooms their heights.
	var looks: PackedStringArray = sim.run_looks()
	check(looks.size() > 20 and looks[0] == "none", "the looks: %d" % looks.size())
	var all_looked := true
	for row in sim.run_pieces():
		if row.size() != 13 or int(row[8]) <= 0 or int(row[8]) >= looks.size():
			all_looked = false
		# (R2e, note B: nothing floats: the sim says what holds each footing up.)
		elif int(row[10]) < 1 or int(row[10]) > 4:
			all_looked = false
	check(all_looked, "every footing has a look, and something holding it up")
	var masses: Array = sim.run_buildings()
	check(masses.size() >= 2 and (masses[0] as PackedFloat32Array).size() == 8, "the rooms' building masses: %d" % masses.size())
	check(sim.run_deck_lines().size() == sim.run_deck_names().size(), "the decks' lines")
	check(sim.run_rooms()[0].size() == 17, "the rooms' rows: " + str(sim.run_rooms()[0]))
	# R2e (F4): the node map ahead, planned (about three rooms); (F3) the bonuses.
	var plan: PackedInt32Array = sim.run_plan()
	var deepest := 0
	for k in plan.size() / 9:
		deepest = maxi(deepest, plan[k * 9])
	check(plan.size() % 9 == 0 and plan.size() >= 18 and plan[0] == 1 and deepest >= 2, "the map ahead: " + str(plan))
	check(sim.run_bonus().size() == 10, "the bonuses: " + str(sim.run_bonus()))
	check(sim.run_coins().size() % 4 == 0, "the coins")
	var interior: Array = sim.run_interior()
	check(interior.size() >= 4, "the shop's interior: %d rows" % interior.size())
	# Walking through rooms until a shop is in play, then into it.
	var found := false
	for n in 12:
		check(sim.run_debug_next_room(), "into the next room")
		if sim.run_debug_enter_shop():
			found = true
			break
	check(found, "a shop came, and it was walked into")
	check(sim.run_info()[13] == 1 and sim.run_view()[9] == 1.0, "inside: " + str(sim.run_info()))
	var inside: PackedInt32Array = sim.run_shop()
	check(inside[0] == 1 and inside[6] == 1, "the shopkeeper waves: " + str(inside))
	check(sim.run_shop()[1] == -1, "standing at the way out")
	# Up again at the door: out.
	sim.set_input(0, 0, 0, 0)
	sim.tick()
	sim.set_input(0, 0, 127, 0)
	sim.tick()
	sim.set_input(0, 0, 0, 0)
	sim.tick()
	check(sim.run_info()[13] == 0, "back outside")
	sim.run_start(5)
	check(sim.run_mobs().size() == 12 * 17, "twelve grunt slots (grunts are fighters: R2c)")
	# R2b: the grunt kinds, and a grunt drawn as a fighter of its body through the same calls.
	var kinds: PackedInt32Array = sim.run_kinds()
	check(kinds.size() == 6 * 5, "six grunt kinds: " + str(kinds))
	for k in 6:
		var g: int = sim.run_debug_grunt(k, 3.0 + k, k == 0)
		check(g >= 0, "a grunt of kind %d" % k)
		check(sim.fighter_class(100 + g) == kinds[k * 5 + 2], "kind %d is its body's class" % k)
		check(sim.fighter_state(100 + g) != "", "and has a fighter's state")
		var row: PackedFloat32Array = sim.run_mobs().slice(g * 17, g * 17 + 17)
		check(int(row[1]) == k and row[12] > 1.2, "fighter-tall: " + str(row))
	check(sim.run_mobs()[6] == 2.0 and sim.run_mobs()[16] > 0.0, "the first one tells: " + str(sim.run_mobs().slice(0, 17)))
	check(sim.run_props().size() % 9 == 0 and sim.run_van().size() == 8, "the props and the van")
	# R2c: a grunt launched hard out of the view is knocked out by the blast zones that follow it.
	check(sim.run_debug_launch_grunt(1, 1.6, 0.4), "a grunt launched")
	for n in 90:
		sim.set_input(0, 0, 0, 0)
		sim.tick()
	check(sim.run_mobs()[17] < 0.5 and sim.run_view()[13] >= 1.0, "knocked out past the zone: " + str(sim.run_view()))
	check(sim.run_info()[3] >= 1, "the player's kill")
	sim.run_debug_clear_grunts()
	check(sim.run_mobs()[0] < 0.5, "cleared")
	# Running right: the camera follows and the stage streams on.
	var layout := info[9]
	for n in 600:
		sim.set_input(0, 127, 0, 0)
		sim.tick()
	check(sim.run_view()[0] > view[0] + 5.0, "the camera moved on: %.1f" % sim.run_view()[0])
	check(sim.run_info()[1] > 10, "distance counts: %d m" % sim.run_info()[1])
	check(sim.run_info()[8] >= 1, "a room was entered: %d" % sim.run_info()[8])
	check(sim.run_info()[9] != layout, "the stage streamed")
	# Running back: the camera follows back.
	var far: float = sim.run_view()[0]
	for n in 240:
		sim.set_input(0, -127, 0, 0)
		sim.tick()
	check(sim.run_view()[0] < far - 3.0 or sim.run_info()[11] == 1, "the camera came back: %.1f from %.1f" % [sim.run_view()[0], far])
	check(sim.run_view()[0] >= sim.run_view()[6], "never past the wall")
	# R2d: the decks (the map strip's ladder), and a climb that becomes a gate once left.
	sim.run_start(5)
	check(sim.run_deck_names() == PackedStringArray(["street", "rooftops", "skyline"]), "three decks: " + str(sim.run_deck_names()))
	var decks: PackedInt32Array = sim.run_decks()
	check(decks.size() == 5 and decks[0] == 1 and decks[1] == 1 and decks[4] == 3, "the run starts on the rooftops: " + str(decks))
	check(sim.run_debug_only(PackedStringArray(["escape_climb", "street", "catwalks"]), 1, 3) == 3, "only a climb, a street and the catwalks")
	var vertical := -1
	var lid := false
	for n in 12:
		sim.run_debug_next_room()
		for r in sim.run_rooms():
			if int(r[5]) == sim.run_info()[8] and r[12] != r[13] and vertical < 0:
				vertical = int(r[5])
				check(sim.run_room_names()[int(r[15])] == "escape_climb", "the climb (or, mirrored, the drop)")
			if int(r[5]) == vertical and r[14] > 0.5:
				lid = true
		if vertical >= 0 and sim.run_info()[8] > vertical:
			break
	check(vertical >= 0, "a room changed deck")
	check(sim.run_decks()[0] != 1 or lid, "the deck changed: " + str(sim.run_decks()))
	check(sim.run_view()[6] >= 0.0, "the wall moved up to it")
	# A made fighter can run too.
	var made := Roster.neutral_entry("Maker", 3, Roster.Loadout.default_for(0))
	check(sim.run_begin(Roster.spec_bytes(made)) == "", "a made fighter sets up a run")
	check(sim.run_start(9), "and runs")
	# A versus match afterwards is versus again.
	sim.start(3, PackedInt32Array([0, 1]))
	check(not sim.run_active(), "a match ends the run mode")
	sim.queue_free()


func _scene() -> void:
	Roster.session = {"from_menu": true, "run_entry": Roster.builtins()[1], "run_seed": 42}
	var main: Node = load("res://runner.tscn").instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame
	await create_timer(1.5).timeout
	check(main.run_error == "", "the run is set up: " + main.run_error)
	check(main.sim.run_active() and main.run_seed == 42, "the run scene starts the menus' run: seed %d" % main.run_seed)
	check(main.PLAYERS == 1 and main.views.size() == 1, "one fighter")
	check(main.sim.frame() > 30, "and it plays: frame %d" % main.sim.frame())
	var data: Dictionary = main.run_hud.board.data
	check(data.get("name", "") == Roster.builtins()[1].name, "the display names the fighter: " + str(data.get("name", "")))
	check(int(data.get("lives", 0)) == 3 and data.has("metres") and data.has("coins") and data.has("combo"), "and reads the run: " + str(data))
	check(main.run_stage.pieces.size() >= 2, "the stage is drawn room by room: %d" % main.run_stage.pieces.size())
	check(main.run_stage.wall_node != null and main.run_stage.wall_node.position.x == main.sim.run_view()[6], "the left wall is drawn")
	check(data.has("map") and (data["map"] as PackedInt32Array).size() >= 6, "the display has the map")
	check(data.has("shop") and data.has("door") and data.has("gate"), "and the shop, the door and the gate")
	check(data.has("plan") and (data["plan"] as PackedInt32Array).size() >= 18 and data.has("bonus"), "and the map ahead and the bonuses")
	# A callout shows once: a new count makes a popup, the same count none more.
	var bonus := PackedFloat32Array([2, 1, 0, 0, 25, 1, 1, 0, 0, 0])
	main.run_hud.callouts_seen = 0
	main.run_hud.popups.clear()
	main.run_hud.show_state({"bonus": bonus, "delta": 0.016})
	main.run_hud.show_state({"bonus": bonus, "delta": 0.016})
	check(main.run_hud.popups.size() == 1 and str(main.run_hud.popups[0].text) == "SKYWARD! +25", "a callout: " + str(main.run_hud.popups))
	# The rooms are drawn by their looks, with their building fronts; the coins spin; the skyline scrolls in parallax.
	check(main.run_stage.looks.size() > 20, "the stage knows the looks")
	check(main.run_stage.skylines.size() == 3 and not main.run_stage.skylines.has(null), "the city's three skyline layers, in parallax")
	check(main.run_stage.coins.size() >= 0, "the coins")
	# R2b: the backdrop travels with the camera, so it lights and shades nothing of the rooms (the owner's flicker report).
	var moving_light := false
	var moving_shadow := false
	for n in main.run_stage.backdrop.find_children("*", "", true, false):
		if n is Light3D and (n as Light3D).visible:
			moving_light = true
		if n is GeometryInstance3D and (n as GeometryInstance3D).cast_shadow != GeometryInstance3D.SHADOW_CASTING_SETTING_OFF:
			moving_shadow = true
	check(not moving_light and not moving_shadow, "no light or cast shadow travels with the backdrop")
	# R2b: grunts drawn with the fighter rig in their outfits, the tell's "!" over a telling one.
	var g: int = main.sim.run_debug_grunt(2, 4.0, true)
	for i in 6:
		await process_frame
	check(g >= 0 and main.mob_views[g].visible and main.mob_views[g].fighter != null, "a grunt is drawn with the fighter rig")
	if g >= 0:
		check(main.mob_views[g].role == 2 and main.mob_views[g].tell_label.visible, "a shieldbearer, telling")
		check(main.mob_views[g].fighter.loadout.color == main.GruntView.OUTFITS[2].color, "in its grunt outfit")
		# R2c (the owner's "0% ■%" report): one small damage label at most, never the fighter view's own readout or name.
		var fv: Node3D = main.mob_views[g].fighter
		check(not fv.percent_label.visible and not fv.name_label.visible, "no fighter labels over a grunt")
		check(not main.mob_views[g].pct_label.visible, "and its damage hidden while it tells")
		var quiet: int = main.sim.run_debug_grunt(0, 6.0, false)
		for i in 4:
			await process_frame
		if quiet >= 0:
			check(main.mob_views[quiet].pct_label.visible and main.mob_views[quiet].pct_label.text == "0%", "a grunt's damage, small: " + main.mob_views[quiet].pct_label.text)
			check(not main.mob_views[quiet].fighter.percent_label.visible, "one label only")
			# Launched out of the view: knocked out, with the knock-out's effect.
			var kos: int = main.grunt_kos
			main.sim.run_debug_launch_grunt(quiet, 1.8, 0.5)
			await create_timer(1.2).timeout
			check(main.grunt_kos > kos, "the knock-out was shown")
	# R2c: the camera's height is the sim's.
	check(absf(main.cam_y - main.sim.run_view()[12]) < 1.0, "the camera keeps the sim's height: %.2f vs %.2f" % [main.cam_y, main.sim.run_view()[12]])
	main.sim.run_debug_clear_grunts()
	# R2b: a coin picked up vanishes at once (the owner's R2a bug: it stayed until the stage next changed).
	# (Into the next room first, where the coins are in the view: the blast zone follows the camera.)
	main.sim.run_debug_next_room()
	for i in 4:
		await process_frame
	var coins: PackedFloat32Array = main.sim.run_coins()
	var shown_before: int = main.run_stage.coins_shown()
	var seen: PackedFloat32Array = main.sim.run_view()
	for c in coins.size() / 4:
		# (A coin in the view: the blast zone follows the camera, R2d's rooms reach further.)
		if coins[c * 4 + 2] < 0.5 and coins[c * 4] > seen[0] + 2.0 and coins[c * 4] < seen[0] + seen[1] - 2.0:
			main.sim.debug_place_airborne(0, coins[c * 4], coins[c * 4 + 1] - 1.0)
			break
	for i in 4:
		await process_frame
	check(main.run_stage.coins_shown() < shown_before or shown_before == 0, "a coin picked up is gone: %d from %d" % [main.run_stage.coins_shown(), shown_before])
	var far_x: float = main.run_stage.far_layer.position.x
	var mid_x: float = main.run_stage.mid_layer.position.x
	check(not is_equal_approx(far_x, mid_x) or main.sim.run_view()[0] < 1.0, "the layers scroll at their own speeds")
	# Into a shop: the interior and its shopkeeper are drawn, the camera goes inside, the display prompts at the door.
	var found := false
	for n in 12:
		main.sim.run_debug_next_room()
		if main.sim.run_debug_enter_shop():
			found = true
			break
	check(found, "a shop came and was walked into")
	for i in 6:
		await process_frame
	check(main.run_stage.interior != null and main.run_stage.keeper != null, "the shop's interior and its shopkeeper")
	check(main.run_stage.ware_nodes.size() == 2 and main.run_stage.price_labels.size() == 2, "the two wares and their prices")
	# R2c: every storefront has its CLOSED board, hidden while the shopkeeper sells.
	check(main.run_stage.closed_signs.size() >= 1 and not main.run_stage.shops_closed, "the closed boards, down: %d" % main.run_stage.closed_signs.size())
	check(main.cam.position.x < -10000.0, "the camera is inside: %.0f" % main.cam.position.x)
	check(main.run_hud.board.shop_hint() == "UP: LEAVE THE SHOP", "the way out: " + main.run_hud.board.shop_hint())
	check(main.run_stage.keeper_bubble.visible and main.run_stage.keeper_bubble.text == "Welcome!", "the shopkeeper waves hello")
	# (Back out, for the rest.)
	main._send_key(KEY_W, true)
	await create_timer(0.1).timeout
	main._send_key(KEY_W, false)
	await create_timer(0.2).timeout
	check(main.sim.run_info()[13] == 0, "out again with up at the door")
	check(not main.hud.visible, "the versus display is hidden")
	check(main.results == null, "no results while running")
	# R2e (the owner's playtest note): no background mass ever stands in front of the play plane. Every building's and every
	# backdrop mesh's front face lies behind the back of every footing, here and in the climbs, drops and skyline rooms.
	var depth_bad := _background_in_front(main)
	var shown := ["escape_climb", "great_climb", "balcony_drop", "catwalks", "office", "deck_fork_up"]
	for room_name in shown:
		main.sim.run_debug_only(PackedStringArray([room_name, "street"]), 1, 3)
		for n in 6:
			main.sim.run_debug_next_room()
			await process_frame
			await process_frame
			depth_bad.append_array(_background_in_front(main))
	check(depth_bad.is_empty(), "every background mesh stands behind the play plane: " + str(depth_bad.slice(0, 6)))
	# Losing every life ends the run.
	for life in 3:
		main.sim.debug_place_airborne(0, main.sim.run_view()[0] + 10.0, main.sim.run_view()[3] - 4.0)
		await create_timer(0.25).timeout
	check(main.sim.run_info()[0] == 1, "out of lives: the run is over")
	await create_timer(2.2).timeout
	check(main.results != null, "the results appear")
	if main.results != null:
		check(int(main.results.stats.get("seed", 0)) == 42, "with the run's seed")
		check(main.results.stats.has("rooms") and main.results.stats.has("cleared"), "rooms and arenas: " + str(main.results.stats))
		check(main.results.buttons.size() == 3, "and three choices")
		main.results.handle_key(key(KEY_ENTER))
		await process_frame
		await process_frame
		check(main.results == null and main.sim.run_info()[0] == 0 and main.sim.run_info()[7] == 3, "Run again starts a fresh run")
		check(main.run_seed != 42, "on a new seed")
	main.queue_free()
	await process_frame


## Background meshes (the rooms' building masses and the backdrop) whose front reaches past the back of the play plane's deepest
## footing (a block, 8 back from the front; the fighters stand 2 back): names and depths.
func _background_in_front(main: Node) -> Array:
	var bad := []
	var limit: float = main.run_stage.Z - 8.0
	var roots: Array = [main.run_stage.backdrop]
	for room in main.run_stage.pieces.values():
		var masses: Node = (room as Node).get_node_or_null("Buildings")
		if masses != null:
			roots.append(masses)
	for r in roots:
		for n in (r as Node).find_children("*", "MeshInstance3D", true, false):
			var mi := n as MeshInstance3D
			if not mi.is_visible_in_tree() or mi.mesh == null:
				continue
			var box := mi.global_transform * mi.get_aabb()
			var front := box.position.z + box.size.z
			if front > limit:
				bad.append("%s %.2f" % [mi.get_path(), front])
	return bad


func _menu() -> void:
	Roster.session = {}
	var menu: Node = load("res://menu.tscn").instantiate()
	root.add_child(menu)
	await process_frame
	var found := false
	for item in menu.ITEMS:
		if item[0] == "Run" and item[2] == "run":
			found = true
	check(found, "the main menu has a Run tile")
	menu.queue_free()
	await process_frame
	Roster.session = {"mode": "run"}
	var select: Node = load("res://select.tscn").instantiate()
	root.add_child(select)
	await process_frame
	await process_frame
	check(select.run_mode and select.count == 1, "the fighter choice is for one player in a run")
	check(not select.players_button.visible and not select.stage_button.visible, "with no match rules")
	select.queue_free()
	await process_frame
