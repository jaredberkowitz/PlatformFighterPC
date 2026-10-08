extends Node3D
## Playable test bed: the Rust sim runs at 60 Hz in _physics_process; everything else only draws it.

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Roster := preload("res://scripts/roster.gd")
const StageView := preload("res://scripts/stage_view.gd")
const InputReader := preload("res://scripts/input_reader.gd")
const DebugOverlay := preload("res://scripts/debug_overlay.gd")
const Demo := preload("res://scripts/demo.gd")

const PLAYERS := 2
const SEED := 1
const CHARS := [0, 1, 0, 1]
## Which fighters play, by index into the loaded roster. `--chars=2,0` after `--` picks them (editors' playtest does).
var chosen_chars: Array = CHARS

var sim
var masks := {}
var views: Array = []
var stage_view: Node3D
var overlay: CanvasLayer
var cam: Camera3D
var ecb_nodes: Array = []
var ecb_mat: StandardMaterial3D
var min_down: Array = [0.2, 0.2]
var ui_stamp := -1
var overlay_dirty := true
var warmed := false
var snaps: Array = []
var inputs: Array = []
var prev_pos: Array = []
var cur_pos: Array = []
var paused := false
var show_ecb := true
var overlay_on := true
var demo = null
var shot_wait := ""
var perf := false
var perf_frames := 0
var perf_time := 0.0
var perf_draw := 0
var perf_prims := 0
var perf_worst := 0.0
var perf_hitches := 0
var flag_noui := false
var flag_noecb := false


func _ready() -> void:
	if not ClassDB.class_exists("SimRunner"):
		var l := Label.new()
		l.text = "The Rust bridge did not load.\nBuild it first:  cargo build -p godot-bridge"
		l.position = Vector2(30, 30)
		add_child(l)
		set_physics_process(false)
		set_process(false)
		return

	sim = ClassDB.instantiate("SimRunner")
	add_child(sim)
	for n in ["jump", "attack", "special", "shield", "grab", "strong"]:
		masks[n] = sim.button_mask(n)
	_build_world()
	_parse_demo_args()
	_load_content()
	sim.set_players(PLAYERS)
	_restart()
	_start_net()
	_build_ecb()
	_build_boxes()
	_build_projectiles()
	await _prewarm()


func _build_world() -> void:
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color(0.56, 0.78, 0.95)
	env.ambient_light_source = Environment.AMBIENT_SOURCE_COLOR
	env.ambient_light_color = Color(1.0, 0.95, 0.9)
	env.ambient_light_energy = 0.75
	var we := WorldEnvironment.new()
	we.environment = env
	add_child(we)

	var sun := DirectionalLight3D.new()
	sun.rotation_degrees = Vector3(-42, -28, 0)
	sun.light_energy = 1.0
	add_child(sun)

	cam = Camera3D.new()
	cam.fov = 30.0
	cam.position = Vector3(0, 6, 60)
	add_child(cam)

	stage_view = StageView.new()
	add_child(stage_view)
	stage_view.build(sim)

	for i in PLAYERS:
		var v := FighterView.new()
		add_child(v)
		v.build(i, Loadout.load_saved(i))
		views.append(v)
		snaps.append({})
		inputs.append({"x": 0, "y": 0, "buttons": 0})
		prev_pos.append(Vector2.ZERO)
		cur_pos.append(Vector2.ZERO)

	overlay = DebugOverlay.new()
	add_child(overlay)
	overlay.build(PLAYERS, masks)


func _parse_demo_args() -> void:
	var name := ""
	var dir := ""
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--demo="):
			name = a.substr(7)
		elif a.begins_with("--shots="):
			dir = a.substr(8)
	var user_args := OS.get_cmdline_user_args()
	flag_noui = user_args.has("--noui")
	flag_noecb = user_args.has("--noecb")
	if user_args.has("--nomsaa"):
		get_viewport().msaa_3d = Viewport.MSAA_DISABLED
	if OS.get_cmdline_user_args().has("--perf"):
		perf = true
		name = "tour"
		if not user_args.has("--vsync"):
			DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
			Engine.max_fps = 0
	if name != "":
		if dir != "":
			DirAccess.make_dir_recursive_absolute(dir)
		demo = Demo.make(name, masks, dir)
		if perf:
			demo.shots = []
			demo.end_frame = 600
		print("demo '%s' starting" % name)


func _restart() -> void:
	var chars: Array = chosen_chars
	if demo != null and demo.chars.size() > 0:
		chars = demo.chars
	sim.start(SEED, PackedInt32Array(chars))
	proj_cur = sim.projectile_slots()
	proj_prev = proj_cur
	for i in PLAYERS:
		_refresh(i)
		prev_pos[i] = cur_pos[i]
	_rebuild_boxes()
	paused = false


func _refresh(i: int) -> void:
	var info: PackedInt32Array = sim.fighter_info(i)
	var cb: PackedInt32Array = sim.fighter_combat(i)
	cur_pos[i] = sim.fighter_pos(i)
	snaps[i] = {
		"state": sim.fighter_state(i), "state_frame": sim.fighter_state_frame(i),
		"facing": sim.fighter_facing(i), "pos": cur_pos[i], "vel": sim.fighter_vel(i),
		"char": info[0], "platform": info[1], "jumps": info[2], "dodged": info[3] != 0,
		"fast_fall": info[4] != 0, "ledge": info[5], "ledge_invuln": info[6], "grabs": info[7],
		"lag": info[8], "cooldown": info[9], "ignore": info[10], "frame": sim.frame(),
		"percent": sim.fighter_percent(i), "stocks": cb[0], "hitlag": cb[1], "hitstun": cb[2],
		"move_id": cb[3], "tumble": cb[4] != 0, "invuln": cb[5], "launch_pending": cb[6] != 0,
		"charge": sim.fighter_charge(i), "shield": sim.fighter_shield(i) / sim.shield_max(), "move_name": sim.fighter_move_name(i), "move_timing": sim.fighter_move_timing(i),
		"move_tip": sim.fighter_move_tip(i), "reach": sim.fighter_weapon_reach(i),
	}


## Sends a real key event through Godot's Input, so scripted demos exercise the same path as a player.
func _send_key(code: int, down: bool) -> void:
	var e := InputEventKey.new()
	e.physical_keycode = code
	e.keycode = code
	e.pressed = down
	Input.parse_input_event(e)


# ---- Content -----------------------------------------------------------------------------------------
# The roster is read from a content bundle (see docs/CONTENT.md). By default that is content/base.pfc next to the
# godot folder; `--content=PATH` (after `--`) picks another. If neither loads, the built-in roster is used.

var content_note := ""


func _load_content() -> void:
	var path := ""
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--content="):
			path = a.substr(10)
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--chars=") and not a.contains("--host"):
			var picked := []
			for c in a.substr(8).split(","):
				picked.append(int(c))
			while picked.size() < 4:
				picked.append(0)
			chosen_chars = picked
	# Coming from the menus: the match's content (the base roster plus any fighters made in the creator) is ready.
	if Roster.session.has("content_text"):
		var err_text: String = sim.load_content_text(Roster.session.content_text)
		if err_text == "":
			chosen_chars = Roster.session.chars.duplicate()
			while chosen_chars.size() < 4:
				chosen_chars.append(0)
			content_note = ""
			for i in mini(PLAYERS, Roster.session.entries.size()):
				var e: Dictionary = Roster.session.entries[i]
				views[i].rebuild(e.look)
				views[i].set_name_tag(e.name)
				views[i].scale = Vector3.ONE * (float(Roster.session.sizes[i]) / 100.0)
			return
		push_error("content: " + err_text)
		content_note = "CONTENT NOT LOADED: " + err_text
	var asked := path != ""
	if not asked:
		path = ProjectSettings.globalize_path("res://").path_join("../content/base.pfc").simplify_path()
		if not FileAccess.file_exists(path):
			return
	var err: String = sim.load_content(path)
	if err != "":
		push_error("content: " + err)
		content_note = "CONTENT NOT LOADED (using the built-in roster): " + err
		return
	content_note = "content: %s" % sim.content_name()
	print("content loaded from ", path, " (", sim.content_name(), ", ", sim.content_hash(), ")")


# ---- Network play -----------------------------------------------------------------------------------
# Launch arguments (after `--`):
#   --host=PORT                 host a match over UDP         (add --chars=0,1 to pick characters, --delay=2 for input delay)
#   --join=IP:PORT              join one
#   --relay=IP:PORT --room=N    host or join through a relay server (with --host=0 or --join=-)
# Both players use player 1's keys on their own keyboard.

var net_mode := false
var local_slot := 0
var their_look_applied := false
var net_status := 0
var net_lines: Array[String] = []


func _start_net() -> void:
	var host_port := -1
	var join_addr := ""
	var relay := ""
	var room := 0
	var delay := 2
	var chars: Array = chosen_chars
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--host="):
			host_port = int(a.substr(7))
		elif a.begins_with("--join="):
			join_addr = a.substr(7)
		elif a.begins_with("--relay="):
			relay = a.substr(8)
		elif a.begins_with("--room="):
			room = int(a.substr(7))
		elif a.begins_with("--delay="):
			delay = int(a.substr(8))
		elif a.begins_with("--chars="):
			chars = []
			for c in a.substr(8).split(","):
				chars.append(int(c))
	# Everyone plays with their own saved look (player 1's slot); it travels to the other side in the handshake.
	var mine := Loadout.load_saved(0)
	sim.set_cosmetics(mine.to_bytes())
	local_slot = 0 if (host_port >= 0) else 1
	var err := ""
	if host_port >= 0 and relay != "":
		err = sim.net_host_relay(relay, room, PackedInt32Array(chars), delay)
	elif host_port >= 0:
		err = sim.net_host(host_port, PackedInt32Array(chars), delay)
	elif join_addr != "" and relay != "":
		err = sim.net_join_relay(relay, room)
	elif join_addr != "":
		err = sim.net_join(join_addr)
	else:
		return
	if err != "":
		push_error(err)
		net_lines.append(err)
		return
	net_mode = true
	views[local_slot].rebuild(mine)
	print("network mode: ", "host" if host_port >= 0 else "joiner")


## One frame of networked play: read the local keyboard, let the rollback session simulate (or wait), then draw.
func _net_step() -> void:
	var local: int = maxi(sim.net_local_player(), 0)
	var r: Dictionary = InputReader.read(0, masks)
	inputs[local] = r
	net_status = sim.net_update(r.x, r.y, r.buttons)
	# Once the handshake is done the other player's look arrives (or is missing, and they keep the default look).
	if net_status >= 1 and not their_look_applied:
		their_look_applied = true
		var theirs := Loadout.from_bytes(sim.net_their_cosmetics(), 1 - local_slot)
		views[1 - local_slot].rebuild(theirs)
	for line in sim.net_take_log():
		net_lines.append(line)
		print(line)
	if net_lines.size() > 4:
		net_lines = net_lines.slice(net_lines.size() - 4)
	if net_status == 1:
		_tick_once(false)
	else:
		overlay_dirty = true


func _tick_once(advance := true) -> void:
	for i in PLAYERS:
		prev_pos[i] = cur_pos[i]
	if advance:
		sim.tick()
	proj_prev = proj_cur
	proj_cur = sim.projectile_slots()
	for i in PLAYERS:
		_refresh(i)
		if (cur_pos[i] - prev_pos[i]).length() > 2.5:
			prev_pos[i] = cur_pos[i]  # teleport-like moves (ledge get-up) should not slide
	_rebuild_boxes()
	_update_combos()
	_rebuild_paths()


## Training mode: counts hits on a fighter that is being kept in hitstun or a grab, and the damage of the string.
func _update_combos() -> void:
	for i in PLAYERS:
		var s: Dictionary = snaps[i]
		var helpless: bool = s.state in ["Hitstun", "Grabbed", "ShieldBreak", "Knockdown"] or s.hitlag > 0 and s.launch_pending
		if s.percent > last_pct[i] + 0.001:
			if combo_idle[i] > 40 or combo_hits[i] == 0:
				combo_hits[i] = 0
				combo_start[i] = last_pct[i]
			combo_hits[i] += 1
			combo_idle[i] = 0
		elif s.percent < last_pct[i]:
			combo_hits[i] = 0
		last_pct[i] = s.percent
		combo_idle[i] = 0 if helpless else combo_idle[i] + 1
		s["combo_hits"] = combo_hits[i] if combo_idle[i] < 120 else 0
		s["combo_damage"] = s.percent - combo_start[i]
		var launch: PackedFloat32Array = sim.fighter_launch(i)
		s["launch_kb"] = launch[0]
		s["launch_angle"] = launch[1]


## Training mode: for a fighter in hitstun, the path it will fly (white: no DI, yellow: holding the stick as it is now).
func _rebuild_paths() -> void:
	if path_nodes.is_empty():
		for _i in PLAYERS:
			var mi := MeshInstance3D.new()
			mi.mesh = ImmediateMesh.new()
			add_child(mi)
			path_nodes.append(mi)
	for i in PLAYERS:
		var im: ImmediateMesh = path_nodes[i].mesh
		im.clear_surfaces()
		var s: Dictionary = snaps[i]
		if not show_di or not (s.state == "Hitstun" or (s.hitlag > 0 and s.launch_pending)):
			continue
		var stick: Dictionary = inputs[i]
		for pass_i in 2:
			var path: PackedVector2Array = sim.predict_path(i, 70, 0 if pass_i == 0 else stick.x, 0 if pass_i == 0 else stick.y)
			var color := Color(1, 1, 1, 0.8) if pass_i == 0 else Color(1.0, 0.9, 0.2, 0.95)
			im.surface_begin(Mesh.PRIMITIVE_LINE_STRIP, box_mat)
			var origin: Vector2 = cur_pos[i]
			im.surface_set_color(color)
			im.surface_add_vertex(Vector3(origin.x, origin.y + 1.1, 1.6))
			for p in path:
				im.surface_set_color(color)
				im.surface_add_vertex(Vector3(p.x, p.y + 1.1, 1.6))
			im.surface_end()


func _gather() -> void:
	for i in PLAYERS:
		if demo != null and demo.real_keys and i == 0:
			inputs[i] = InputReader.read(0, masks)
		elif demo != null:
			if i == 0:
				inputs[i] = demo.input_at(sim.frame())
			elif i == 1 and demo.timeline2.size() > 0:
				inputs[i] = demo.input_at2(sim.frame())
			else:
				inputs[i] = {"x": 0, "y": 0, "buttons": 0}
		else:
			inputs[i] = InputReader.read(i, masks)
		sim.set_input(i, inputs[i].x, inputs[i].y, inputs[i].buttons)


func _physics_process(_delta: float) -> void:
	if not warmed or shot_wait != "":
		return
	if net_mode:
		_net_step()
		return
	if demo != null:
		for e in demo.events_at(sim.frame()):
			if e[1] == "place":
				sim.debug_place_airborne(e[2], e[3], e[4])
				_refresh(e[2])
				prev_pos[e[2]] = cur_pos[e[2]]
			elif e[1] == "helpless":
				sim.debug_helpless(e[2])
			elif e[1] == "key":
				_send_key(e[2], e[3])
			elif e[1] == "stand":
				sim.debug_stand(e[2], e[3], e[4])
				_refresh(e[2])
				prev_pos[e[2]] = cur_pos[e[2]]
			elif e[1] == "shield":
				sim.debug_set_shield(e[2], e[3])
				_refresh(e[2])
			elif e[1] == "percent":
				sim.debug_set_percent(e[2], e[3])
				_refresh(e[2])
	_gather()
	if paused:
		return
	_tick_once()
	if demo != null:
		var shot: String = demo.shot_at(sim.frame())
		if shot != "":
			shot_wait = shot
			_capture(shot)
		if sim.frame() >= demo.end_frame and shot_wait == "":
			if perf:
				print("PERF %d frames in %.2fs = %.1f fps, %.2f ms avg, worst %.1f ms, %d frames over 25 ms, %d draw calls, %d primitives" % [perf_frames, perf_time, perf_frames / perf_time, 1000.0 * perf_time / perf_frames, perf_worst * 1000.0, perf_hitches, perf_draw / perf_frames, perf_prims / perf_frames])
			print("demo '%s' finished at frame %d checksum %s" % [demo.name, sim.frame(), sim.checksum()])
			get_tree().quit()


func _capture(label: String) -> void:
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var path := "%s/%s.png" % [demo.out_dir if demo.out_dir != "" else "user://", label]
	img.save_png(path)
	print("saved ", path)
	shot_wait = ""


func _alpha() -> float:
	if paused or shot_wait != "" or demo != null:
		return 1.0
	return Engine.get_physics_interpolation_fraction()


func _process(delta: float) -> void:
	if perf and warmed:
		perf_frames += 1
		perf_time += delta
		perf_worst = maxf(perf_worst, delta)
		if delta > 0.025:
			perf_hitches += 1
		perf_draw += int(RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_TOTAL_DRAW_CALLS_IN_FRAME))
		perf_prims += int(RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_TOTAL_PRIMITIVES_IN_FRAME))
	var a := _alpha()
	for i in PLAYERS:
		var p: Vector2 = prev_pos[i].lerp(cur_pos[i], a)
		views[i].apply(Vector3(p.x, p.y, 0), snaps[i], delta)
	_update_camera(a, delta)
	if not flag_noecb:
		_update_ecb(a)
	_position_boxes(a)
	_update_projectiles(a)
	if flag_noui or not overlay_on:
		return
	# Text layout is the expensive part of the overlay, and nothing in it changes between sim ticks.
	var stamp: int = int(sim.frame()) * 4 + int(paused) * 2 + int(overlay_dirty)
	if stamp == ui_stamp:
		return
	ui_stamp = stamp
	overlay_dirty = false
	stage_view.update_ledges(sim)
	overlay.update(snaps, inputs, {
		"frame": sim.frame(), "checksum": sim.checksum(), "version": sim.sim_version(),
		"content_hash": sim.content_hash(), "paused": paused, "history": sim.history_len(),
		"min_down": min_down,
		"net": _net_text(),
		"note": content_note,
	})


func _net_text() -> String:
	if not net_mode:
		return ""
	var text: String = sim.net_info()
	if net_status == 2:
		text += "   (waiting for the other player)"
	for l in net_lines:
		text += "\n" + l
	return text


func _update_camera(a: float, delta: float) -> void:
	var lo := Vector2(1e9, 1e9)
	var hi := Vector2(-1e9, -1e9)
	for i in PLAYERS:
		var p: Vector2 = prev_pos[i].lerp(cur_pos[i], a)
		lo = lo.min(p)
		hi = hi.max(p + Vector2(0, 2.2))
	var center := (lo + hi) / 2.0
	# Fit both fighters (plus margin) in view: visible width at distance d is about d * 0.95 at 30 deg fov, 16:9.
	var spread := maxf(hi.x - lo.x + 18.0, (hi.y - lo.y + 10.0) * 1.78)
	var dist := clampf(spread / 0.95, 24.0, 85.0)
	if demo != null and demo.cam_dist > 0.0:
		dist = demo.cam_dist
	var target := Vector3(clampf(center.x, -12, 12), clampf(center.y, -3, 10) + 1.6, dist)
	cam.position = cam.position.lerp(target, clampf(delta * 3.5, 0.0, 1.0))


## The ECB outline is a static diamond mesh per fighter (its shape never changes in a match),
## so showing it costs one node transform per frame instead of rebuilding geometry.
func _build_ecb() -> void:
	for n in ecb_nodes:
		n.queue_free()
	ecb_nodes.clear()
	ecb_mat = StandardMaterial3D.new()
	ecb_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	ecb_mat.vertex_color_use_as_albedo = true
	ecb_mat.no_depth_test = true
	for i in PLAYERS:
		var body: PackedFloat32Array = sim.fighter_body(i)
		var pts := [Vector2(0, 0), Vector2(body[0], body[2]), Vector2(0, body[1]), Vector2(-body[0], body[2])]
		var im := ImmediateMesh.new()
		im.surface_begin(Mesh.PRIMITIVE_LINES, ecb_mat)
		for k in 4:
			var a: Vector2 = pts[k]
			var b: Vector2 = pts[(k + 1) % 4]
			im.surface_set_color(FighterView.COLORS[i])
			im.surface_add_vertex(Vector3(a.x, a.y, 1.2))
			im.surface_set_color(FighterView.COLORS[i])
			im.surface_add_vertex(Vector3(b.x, b.y, 1.2))
		im.surface_end()
		var mi := MeshInstance3D.new()
		mi.mesh = im
		add_child(mi)
		ecb_nodes.append(mi)
		min_down[i] = body[3]


func _update_ecb(a: float) -> void:
	for i in PLAYERS:
		ecb_nodes[i].visible = show_ecb
		if show_ecb:
			var p: Vector2 = prev_pos[i].lerp(cur_pos[i], a)
			ecb_nodes[i].position = Vector3(p.x, p.y, 0)


## Renders every transient effect for a few frames behind a cover, so their shaders compile
## now instead of causing a hitch the first time they show up mid-match.
func _prewarm() -> void:
	var cover := ColorRect.new()
	cover.color = Color(0.56, 0.78, 0.95)
	cover.set_anchors_preset(Control.PRESET_FULL_RECT)
	var layer := CanvasLayer.new()
	layer.layer = 1
	layer.add_child(cover)
	add_child(layer)
	for v in views:
		v.prewarm(true)
	overlay.prewarm_text(true)
	for _i in 6:
		await get_tree().process_frame
	for v in views:
		v.prewarm(false)
	overlay.prewarm_text(false)
	layer.queue_free()
	warmed = true


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	# Training keys change the sim directly, which a networked match must never do.
	if net_mode and event.keycode in [KEY_F6, KEY_F7, KEY_F8, KEY_P, KEY_PERIOD, KEY_COMMA, KEY_R]:
		return
	match event.keycode:
		KEY_F1:
			overlay_on = not overlay_on
			overlay.set_overlay_visible(overlay_on)
		KEY_F2:
			show_ecb = not show_ecb
		KEY_F3:
			show_boxes = not show_boxes
			_rebuild_boxes()
		KEY_F9:
			show_di = not show_di
			_rebuild_paths()
		KEY_F6:
			sim.debug_set_percent(1, sim.fighter_percent(1) + 25.0)
			_refresh(1)
		KEY_F7:
			sim.debug_set_percent(0, 0.0)
			sim.debug_set_percent(1, 0.0)
			_refresh(0)
			_refresh(1)
		KEY_F8:
			sim.debug_stand(0, -1.2, 1)
			sim.debug_stand(1, 1.2, -1)
			for i in PLAYERS:
				_refresh(i)
				prev_pos[i] = cur_pos[i]
			_rebuild_boxes()
		KEY_P:
			paused = not paused
		KEY_PERIOD:
			if paused:
				_gather()
				_tick_once()
		KEY_COMMA:
			if paused and sim.step_back():
				for i in PLAYERS:
					_refresh(i)
					prev_pos[i] = cur_pos[i]
				_rebuild_boxes()
		KEY_R:
			_restart()
		KEY_ESCAPE:
			# From the menus, Esc goes back to character select; launched directly, it quits.
			if Roster.session.get("from_menu", false) and not net_mode:
				get_tree().change_scene_to_file("res://select.tscn")
			else:
				get_tree().quit()


# ---- Hitbox and hurtbox display (training) ----------------------------------------------------------
# Rebuilt only when the sim ticks (60 Hz), positioned each frame by interpolation.

var show_boxes := true
var show_di := true
var path_nodes: Array[MeshInstance3D] = []
var combo_hits := [0, 0, 0, 0]
var combo_start := [0.0, 0.0, 0.0, 0.0]
var combo_idle := [999, 999, 999, 999]
var last_pct := [0.0, 0.0, 0.0, 0.0]
var box_nodes: Array = []
var box_mat: StandardMaterial3D


func _build_boxes() -> void:
	box_mat = StandardMaterial3D.new()
	box_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	box_mat.vertex_color_use_as_albedo = true
	box_mat.no_depth_test = true
	for i in PLAYERS:
		var mi := MeshInstance3D.new()
		mi.mesh = ImmediateMesh.new()
		add_child(mi)
		box_nodes.append(mi)


func _circle(im: ImmediateMesh, c: Vector2, r: float, color: Color) -> void:
	var segments := 20
	for k in segments:
		var a0 := TAU * k / segments
		var a1 := TAU * (k + 1) / segments
		im.surface_set_color(color)
		im.surface_add_vertex(Vector3(c.x + cos(a0) * r, c.y + sin(a0) * r, 1.5))
		im.surface_set_color(color)
		im.surface_add_vertex(Vector3(c.x + cos(a1) * r, c.y + sin(a1) * r, 1.5))


func _rebuild_boxes() -> void:
	if box_nodes.is_empty():
		return
	for i in PLAYERS:
		var im: ImmediateMesh = box_nodes[i].mesh
		im.clear_surfaces()
		if not show_boxes:
			continue
		var origin: Vector2 = cur_pos[i]
		im.surface_begin(Mesh.PRIMITIVE_LINES, box_mat)
		var hurt: PackedFloat32Array = sim.fighter_hurtboxes(i)
		for k in range(0, hurt.size(), 3):
			_circle(im, Vector2(hurt[k], hurt[k + 1]) - origin, hurt[k + 2], Color(0.3, 1.0, 0.4, 0.9))
		var hits: PackedFloat32Array = sim.fighter_hitboxes(i)
		for k in range(0, hits.size(), 4):
			var sweet: bool = hits[k + 3] < 0.5
			_circle(im, Vector2(hits[k], hits[k + 1]) - origin, hits[k + 2], Color(1.0, 0.2, 0.2) if sweet else Color(1.0, 0.6, 0.15))
		im.surface_end()


func _position_boxes(a: float) -> void:
	for i in PLAYERS:
		var p: Vector2 = prev_pos[i].lerp(cur_pos[i], a)
		box_nodes[i].position = Vector3(p.x, p.y, 0)


# ---- Projectiles ------------------------------------------------------------------------------------

var proj_nodes: Array = []
var proj_prev := PackedFloat32Array()
var proj_cur := PackedFloat32Array()


func _build_projectiles() -> void:
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.albedo_color = Color(0.78, 0.4, 1.0)
	for k in 8:
		var mi := MeshInstance3D.new()
		var bolt := SphereMesh.new()
		bolt.radius = 0.35
		bolt.height = 0.7
		mi.mesh = bolt
		mi.scale = Vector3(2.2, 0.75, 0.75)
		mi.material_override = mat
		mi.visible = false
		add_child(mi)
		proj_nodes.append(mi)


func _update_projectiles(a: float) -> void:
	for k in proj_nodes.size():
		var i := k * 4
		var active: bool = proj_cur.size() > i and proj_cur[i] > 0.5
		proj_nodes[k].visible = active
		if active:
			var pos := Vector2(proj_cur[i + 1], proj_cur[i + 2])
			if proj_prev.size() > i and proj_prev[i] > 0.5:
				pos = Vector2(proj_prev[i + 1], proj_prev[i + 2]).lerp(pos, a)
			proj_nodes[k].position = Vector3(pos.x, pos.y, 0.3)
