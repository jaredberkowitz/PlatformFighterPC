extends Node3D
## Playable test bed: the Rust sim runs at 60 Hz in _physics_process; everything else only draws it.

const FighterView := preload("res://scripts/fighter_view.gd")
const StageView := preload("res://scripts/stage_view.gd")
const InputReader := preload("res://scripts/input_reader.gd")
const DebugOverlay := preload("res://scripts/debug_overlay.gd")
const Demo := preload("res://scripts/demo.gd")

const PLAYERS := 2
const SEED := 1
const CHARS := [0, 1, 0, 1]

var sim
var masks := {}
var views: Array = []
var stage_view: Node3D
var overlay: CanvasLayer
var cam: Camera3D
var ecb_mesh: MeshInstance3D
var snaps: Array = []
var inputs: Array = []
var prev_pos: Array = []
var cur_pos: Array = []
var paused := false
var show_ecb := true
var overlay_on := true
var demo = null
var shot_wait := ""


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
	for n in ["jump", "attack", "special", "shield", "grab"]:
		masks[n] = sim.button_mask(n)
	_build_world()
	_parse_demo_args()
	_restart()


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
		v.build(i)
		views.append(v)
		snaps.append({})
		inputs.append({"x": 0, "y": 0, "buttons": 0})
		prev_pos.append(Vector2.ZERO)
		cur_pos.append(Vector2.ZERO)

	ecb_mesh = MeshInstance3D.new()
	ecb_mesh.mesh = ImmediateMesh.new()
	add_child(ecb_mesh)

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
	if name != "":
		if dir != "":
			DirAccess.make_dir_recursive_absolute(dir)
		demo = Demo.make(name, masks, dir)
		print("demo '%s' starting" % name)


func _restart() -> void:
	sim.start(SEED, PackedInt32Array(CHARS))
	for i in PLAYERS:
		_refresh(i)
		prev_pos[i] = cur_pos[i]
	paused = false


func _refresh(i: int) -> void:
	var info: PackedInt32Array = sim.fighter_info(i)
	cur_pos[i] = sim.fighter_pos(i)
	snaps[i] = {
		"state": sim.fighter_state(i), "state_frame": sim.fighter_state_frame(i),
		"facing": sim.fighter_facing(i), "pos": cur_pos[i], "vel": sim.fighter_vel(i),
		"char": info[0], "platform": info[1], "jumps": info[2], "dodged": info[3] != 0,
		"fast_fall": info[4] != 0, "ledge": info[5], "ledge_invuln": info[6], "grabs": info[7],
		"lag": info[8], "cooldown": info[9], "ignore": info[10], "frame": sim.frame(),
	}


func _tick_once() -> void:
	for i in PLAYERS:
		prev_pos[i] = cur_pos[i]
	sim.tick()
	for i in PLAYERS:
		_refresh(i)
		if (cur_pos[i] - prev_pos[i]).length() > 2.5:
			prev_pos[i] = cur_pos[i]  # teleport-like moves (ledge get-up) should not slide


func _gather() -> void:
	for i in PLAYERS:
		if demo != null:
			inputs[i] = demo.input_at(sim.frame()) if i == 0 else {"x": 0, "y": 0, "buttons": 0}
		else:
			inputs[i] = InputReader.read(i, masks)
		sim.set_input(i, inputs[i].x, inputs[i].y, inputs[i].buttons)


func _physics_process(_delta: float) -> void:
	if shot_wait != "":
		return
	if demo != null:
		for e in demo.events_at(sim.frame()):
			if e[1] == "place":
				sim.debug_place_airborne(e[2], e[3], e[4])
				_refresh(e[2])
				prev_pos[e[2]] = cur_pos[e[2]]
			elif e[1] == "helpless":
				sim.debug_helpless(e[2])
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
	var a := _alpha()
	for i in PLAYERS:
		var p: Vector2 = prev_pos[i].lerp(cur_pos[i], a)
		views[i].apply(Vector3(p.x, p.y, 0), snaps[i], delta)
	stage_view.update_ledges(sim)
	_update_camera(a, delta)
	_draw_ecb()
	var min_down: Array = []
	for i in PLAYERS:
		min_down.append(sim.fighter_body(i)[3])
	overlay.update(snaps, inputs, {
		"frame": sim.frame(), "checksum": sim.checksum(), "version": sim.sim_version(),
		"content_hash": sim.content_hash(), "paused": paused, "history": sim.history_len(),
		"min_down": min_down,
	})


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
	var target := Vector3(clampf(center.x, -22, 22), clampf(center.y, -4, 18) + 1.6, dist)
	cam.position = cam.position.lerp(target, clampf(delta * 3.5, 0.0, 1.0))


func _draw_ecb() -> void:
	var im: ImmediateMesh = ecb_mesh.mesh
	im.clear_surfaces()
	ecb_mesh.visible = show_ecb
	if not show_ecb:
		return
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.vertex_color_use_as_albedo = true
	mat.no_depth_test = true
	im.surface_begin(Mesh.PRIMITIVE_LINES, mat)
	for i in PLAYERS:
		var p: Vector2 = prev_pos[i].lerp(cur_pos[i], _alpha())
		var body: PackedFloat32Array = sim.fighter_body(i)
		var pts := [
			p, p + Vector2(body[0], body[2]), p + Vector2(0, body[1]), p + Vector2(-body[0], body[2])]
		var col: Color = FighterView.COLORS[i]
		for k in 4:
			var a: Vector2 = pts[k]
			var b: Vector2 = pts[(k + 1) % 4]
			im.surface_set_color(col)
			im.surface_add_vertex(Vector3(a.x, a.y, 1.2))
			im.surface_set_color(col)
			im.surface_add_vertex(Vector3(b.x, b.y, 1.2))
	im.surface_end()


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	match event.keycode:
		KEY_F1:
			overlay_on = not overlay_on
			overlay.set_overlay_visible(overlay_on)
		KEY_F2:
			show_ecb = not show_ecb
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
		KEY_R:
			_restart()
		KEY_ESCAPE:
			get_tree().quit()
