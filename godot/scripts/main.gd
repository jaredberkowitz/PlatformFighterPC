extends Node3D
## Playable test bed: the Rust sim runs at 60 Hz in _physics_process; everything else only draws it.

const Music := preload("res://scripts/music.gd")
const StageArt := preload("res://scripts/stage_art.gd")
const SvgArt := preload("res://scripts/svg_art.gd")
const Lighting := preload("res://scripts/lighting.gd")
const Effects := preload("res://scripts/effects.gd")
const FxMaterial := preload("res://scripts/fx_material.gd")
const Particles := preload("res://scripts/particles.gd")
const POST_SHADER := preload("res://shaders/post.gdshader")
const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Roster := preload("res://scripts/roster.gd")
const StageView := preload("res://scripts/stage_view.gd")
const InputReader := preload("res://scripts/input_reader.gd")
const PadBindings := preload("res://scripts/pad_bindings.gd")
const DebugOverlay := preload("res://scripts/debug_overlay.gd")
const Demo := preload("res://scripts/demo.gd")
const MatchHud := preload("res://ui/match_hud.gd")
const Results := preload("res://ui/results.gd")
const NetLobby := preload("res://ui/net_lobby.gd")
const VisualSettings := preload("res://scripts/visual_settings.gd")
const Replays := preload("res://scripts/replays.gd")
const PauseMenu := preload("res://ui/pause_menu.gd")
const PadNav := preload("res://scripts/pad_nav.gd")
const Sfx := preload("res://scripts/sfx.gd")

## How many fighters are in this match (2 to 4): set once, before the world is built.
var PLAYERS := 2
const SEED := 1
const CHARS := [0, 1, 0, 1]
## Which fighters play, by index into the loaded roster. `--chars=2,0` after `--` picks them (editors' playtest does).
var chosen_chars: Array = CHARS

var sim
var masks := {}
var views: Array = []
var stage_view: Node3D
var overlay: CanvasLayer
var hud: CanvasLayer
var hud_before_results := true
var results: CanvasLayer
var names: Array = ["Player 1", "Player 2", "Player 3", "Player 4"]
var end_timer := 0.0
var sfx: Node
var replay_mode := false
var spectate_mode := false
var group_mode := false
var quick_mode := false
var quick_cfg := {}
var group_status := 0
var group_looks_applied := false
var group_seed := -1
var spectate_status := 0
var spectate_looks_applied := false
var replay_speed := 1.0
var replay_accum := 0.0
var cam: Camera3D
var world_env: Environment
## Where the camera would be without shake or the knock-out zoom.
var cam_base := Vector3.INF
## How hard the camera is shaking (world units), set by strong hits and settling quickly.
var cam_shake := 0.0
## The finishing material over the picture (shaders/post.gdshader), and the frame a finishing hit's two-tone moment ends.
var post_mat: ShaderMaterial
var post_stage_dist := -1.0
var impact_until := -1
## How hard the camera is punching in after a heavy hit (0..1.2), settling quickly.
var cam_punch := 0.0
## A hit that will knock a fighter out: the camera closes in on them for a moment.
var ko_focus := -1
## Who landed the finishing hit (or -1): the knock-out close-up keeps them in the frame too.
var ko_by := -1
var ko_time := 0.0
var shake_rng := RandomNumberGenerator.new()
## The shake's direction (the way the last hit sent its fighter; random for one with no direction) and where it is in its swing.
var shake_dir := Vector2.ZERO
var shake_phase := 0.0
## The shake swings this many times a second, and rolls the camera by up to this many radians per unit of shake.
const SHAKE_HZ := 15.0
const SHAKE_ROLL := 0.012
## Seconds of "3, 2, 1" before a local match starts (the sim waits).
var countdown := 0.0
var countdown_shown := -1
## A knock-out slows a local match down for a moment (real seconds left, since the engine's time is slowed).
var slowmo_until := 0
## The match frame sudden death began on (-1: not in sudden death), for its banner.
var sudden_from := -1
const PLAYER_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95), Color(0.95, 0.78, 0.3), Color(0.4, 0.8, 0.5)]
## Effects in the world that fade by themselves: [node, age, life].
var effects: Array = []
## How each fighter's match is going, for the results screen: knock-outs, falls, damage dealt and taken, and who hit them last.
var stats: Array = []
var last_match_frame := -1
var ecb_nodes: Array = []
var ecb_mat: StandardMaterial3D
var min_down: Array = [0.2, 0.2, 0.2, 0.2]
var ui_stamp := -1
var overlay_dirty := true
var warmed := false
var snaps: Array = []
var inputs: Array = []
## Computer players: a level (1 to 9) for each player the basic CPU plays (`sim_core::cpu`), 0 for a person. Set from the character
## select (the session's `cpu`), `--cpu` (player 2 at level 5), or F5 in training (player 2 on and off).
var cpu_levels: Array = [0, 0, 0, 0]
const CPU_LEVEL := 5
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
## Every measured frame's length (ms), for the 99th percentile.
var perf_deltas: PackedFloat32Array = PackedFloat32Array()
## The game's own rendering time on the GPU and the CPU (timestamps, so other programs sharing the machine count for little).
var perf_gpu := 0.0
var perf_cpu := 0.0
## Script time a frame (ms): the match's ticks (the simulation step and the fighters' views) and the frame's drawing work.
var perf_tick := 0.0
var perf_frame := 0.0
var flag_noui := false
var flag_noecb := false
var cam_override := 0.0  # `--cam=N`: a fixed camera distance for demo shots


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
	sfx = Sfx.of(self)
	for n in ["jump", "attack", "special", "shield", "grab", "strong", "c_up", "c_down", "c_left", "c_right"]:
		masks[n] = sim.button_mask(n)
	PLAYERS = _player_count()
	_build_world()
	_parse_demo_args()
	var chosen_cpu: Array = Roster.session.get("cpu", [])
	for i in mini(chosen_cpu.size(), 4):
		cpu_levels[i] = int(chosen_cpu[i])
	if OS.get_cmdline_user_args().has("--cpu"):
		cpu_levels[1] = CPU_LEVEL
	if demo != null and not demo.cpu_levels.is_empty():
		cpu_levels = demo.cpu_levels.duplicate()
	_load_content()
	# `--stage=N` (training and demos) plays on a stage from the library.
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("--stage="):
			sim.use_stage(int(arg.substr(8)))
	_apply_rules()
	_rebuild_stage()
	# A match started from the menus shows the match HUD only; F1 brings back the training readout.
	# (`--noui` demo shots look like a match from the menus, without even the match HUD; `--match` like one with it.)
	var args := OS.get_cmdline_user_args()
	if Roster.session.get("from_menu", false) or args.has("--noui") or args.has("--match"):
		overlay_on = false
		overlay.set_overlay_visible(false)
		# ...and the hitbox and hurtbox drawings (F3) and the fighters' collision outlines (F2).
		show_boxes = false
		show_ecb = false
		stage_view.set_guides(false)
	sim.set_players(PLAYERS)
	_restart()
	_apply_scales()
	_start_net()
	PadNav.attach(self)
	Music.of(self).play("battle")
	_build_ecb()
	_build_boxes()
	_build_projectiles()
	_build_items()
	await _prewarm()


func _build_world() -> void:
	var env := Environment.new()
	# A sky that fades from deep blue overhead to a pale horizon, with soft hills and clouds far behind the stage.
	var sky_mat := ProceduralSkyMaterial.new()
	sky_mat.sky_top_color = Color(0.32, 0.55, 0.9)
	sky_mat.sky_horizon_color = Color(0.78, 0.9, 0.98)
	sky_mat.ground_horizon_color = Color(0.78, 0.9, 0.98)
	sky_mat.ground_bottom_color = Color(0.55, 0.72, 0.85)
	sky_mat.sun_angle_max = 0.0
	var sky := Sky.new()
	sky.sky_material = sky_mat
	env.background_mode = Environment.BG_SKY
	env.sky = sky
	env.background_color = Color(0.56, 0.78, 0.95)
	var we := WorldEnvironment.new()
	we.environment = env
	add_child(we)
	world_env = env
	Lighting.apply(env, self)
	VisualSettings.apply_quality(env, self, get_viewport())
	Particles.Burst.density = VisualSettings.effects()

	cam = Camera3D.new()
	cam.fov = 30.0
	cam.position = Vector3(0, 6, 60)
	add_child(cam)
	# The illustrated finish over the whole scene: ink lines, paper tone and grain, vignette (shaders/post.gdshader).
	if not OS.get_cmdline_user_args().has("--nopost"):
		var post := MeshInstance3D.new()
		var quad := QuadMesh.new()
		quad.size = Vector2(2, 2)
		post.mesh = quad
		post.extra_cull_margin = 16384.0
		post.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		post_mat = ShaderMaterial.new()
		post_mat.shader = POST_SHADER
		post_mat.set_shader_parameter("impact", 0.0)
		post_mat.set_shader_parameter("far_blur", PLAY_BLUR if VisualSettings.get_index("quality") == 2 else 0.0)
		# First among the see-through things, so the effects, shields and numbers are drawn over the finished picture.
		post_mat.render_priority = -100
		post.material_override = post_mat
		post.position = Vector3(0, 0, -1)
		cam.add_child(post)

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
	hud = MatchHud.new()
	add_child(hud)
	hud.build()
	hud.visible = not OS.get_cmdline_user_args().has("--noui")


## Two, unless the menus chose a bigger free-for-all or a replay of one is being watched.
func _player_count() -> int:
	if Roster.session.has("online") and Roster.session.online.get("group", false):
		return 4
	# (Demos and tests: `--players=N`.)
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--players="):
			return clampi(int(a.substr(10)), 2, 4)
	var replay := PackedByteArray()
	if Roster.session.has("replay"):
		replay = Roster.session.replay
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--replay="):
			replay = Replays.read(a.substr(9))
	if not replay.is_empty():
		var info: Dictionary = sim.replay_peek(replay)
		return clampi(int(info.get("players", 2)), 2, 4)
	if Roster.session.has("entries"):
		return clampi(Roster.session.entries.size(), 2, 4)
	return 2


func _parse_demo_args() -> void:
	var name := ""
	var dir := ""
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--demo="):
			name = a.substr(7)
		elif a.begins_with("--shots="):
			dir = a.substr(8)
		elif a.begins_with("--cam="):
			# (A fixed camera distance for any demo, e.g. `--demo=stage --cam=60` to see a whole stage and what hangs under it.)
			cam_override = float(a.substr(6))
	var user_args := OS.get_cmdline_user_args()
	flag_noui = user_args.has("--noui")
	flag_noecb = user_args.has("--noecb")
	if user_args.has("--nomsaa"):
		get_viewport().msaa_3d = Viewport.MSAA_DISABLED
	if OS.get_cmdline_user_args().has("--perf"):
		perf = true
		RenderingServer.viewport_set_measure_render_time(get_viewport().get_viewport_rid(), true)
		# (Any demo can be measured, `--demo=stage --stage=3` for one stage's frame time; the tour by default.)
		if name == "":
			name = "tour"
		if not user_args.has("--vsync"):
			DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
			Engine.max_fps = 0
	if name != "":
		if dir != "":
			DirAccess.make_dir_recursive_absolute(dir)
		demo = Demo.make(name, masks, dir)
		if cam_override > 0.0:
			demo.cam_dist = cam_override
		if perf:
			demo.shots = []
			demo.end_frame = 600
		print("demo '%s' starting" % name)


func _restart() -> void:
	if replay_mode:
		_replay_seek(0)
		return
	if results != null:
		results.queue_free()
		results = null
		hud.visible = hud_before_results
	cam.attributes = null
	_clear_scuffs()
	end_timer = 0.0
	var chars: Array = chosen_chars
	if demo != null and demo.chars.size() > 0:
		chars = demo.chars
	sim.start(SEED, PackedInt32Array(chars))
	_start_countdown()
	proj_cur = sim.projectile_slots()
	proj_prev = proj_cur
	for i in PLAYERS:
		_refresh(i)
		prev_pos[i] = cur_pos[i]
	_rebuild_boxes()
	paused = false


## A local match from the menus opens with "3, 2, 1, GO!" (training, demos and tests skip it).
func _start_countdown() -> void:
	countdown = 0.0
	countdown_shown = -1
	var rules: PackedInt32Array = sim.match_rules()
	if not Roster.session.get("from_menu", false) or Roster.session.get("skip_countdown", false) or demo != null:
		return
	if Roster.session.has("online") or Roster.session.has("replay") or Roster.session.has("watch"):
		return
	if rules[0] > 0 or rules[1] > 0:
		countdown = 3.0


# ---- The pause screen (ui/pause_menu.gd) --------------------------------------------------------------------------------
# Esc, or a controller's Start, opens it during a match (a local one stops; online the match runs on). It takes every key while it
# is open, so nothing reaches the fighters or the training keys.

var pause_menu: CanvasLayer = null
## The free camera's offset from where it was when the camera was picked, and that place.
var pause_cam := Vector3.ZERO
var pause_cam_base := Vector3.ZERO
var hud_hidden := false


func _input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	if pause_menu != null:
		pause_menu.handle_key(event)
		get_viewport().set_input_as_handled()
		return
	if event.keycode == KEY_ESCAPE and _can_pause():
		_open_pause()
		get_viewport().set_input_as_handled()


## A match is on screen and playing: not over, not a replay, a demo or a screenshot run.
func _can_pause() -> bool:
	return warmed and results == null and net_lobby == null and not replay_mode and demo == null and shot_wait == ""


func _open_pause() -> void:
	var local := _local_match()
	var choices: Array
	if local:
		choices = [["Resume", "resume"], ["Restart", "restart"], ["Camera", "camera"], [_view_label(), "view"],
			["Show display" if hud_hidden else "Hide display", "hud"], ["Quit", "quit"]]
		paused = true
	else:
		choices = [["Resume", "resume"], [_view_label(), "view"], ["Show display" if hud_hidden else "Hide display", "hud"],
			["Leave match", "quit"]]
	pause_menu = PauseMenu.new()
	add_child(pause_menu)
	pause_menu.build(choices, not local, "PAUSED" if local else "MENU")
	pause_menu.chosen.connect(_on_pause_choice)
	sfx.play("blip", 0.9)


## The pause screen's camera-style choice: the camera it switches to.
func _view_label() -> String:
	return "View: steady" if VisualSettings.steady_camera() else "View: follow"


func _close_pause() -> void:
	if pause_menu == null:
		return
	pause_menu.queue_free()
	pause_menu = null
	pause_cam = Vector3.ZERO
	if _local_match():
		paused = false


func _on_pause_choice(action: String) -> void:
	match action:
		"resume":
			_close_pause()
		"restart":
			_close_pause()
			_restart()
		"camera":
			pause_cam = Vector3.ZERO
			pause_cam_base = cam.position
			pause_menu.set_camera_mode(true)
		"camera_done":
			pause_cam = Vector3.ZERO
			pause_menu.set_camera_mode(false)
		"view":
			# The steady or the following camera (the Display setting), switched here to compare the two in play.
			VisualSettings.set_index("camera", 1 - VisualSettings.get_index("camera"))
			_close_pause()
			_open_pause()
		"hud":
			hud_hidden = not hud_hidden
			_show_display(not hud_hidden)
			_close_pause()
		"quit":
			_close_pause()
			_quit_match()


## Shows or hides the match display (the run's too).
func _show_display(on: bool) -> void:
	if hud != null:
		hud.visible = on
	var extra = get("run_hud")
	if extra != null:
		extra.visible = on


## The free camera of the pause screen: the arrow keys or a stick move it, W and S (or the triggers) zoom.
func _pause_camera(delta: float) -> void:
	if pause_menu == null or not pause_menu.camera_mode or cam == null:
		return
	var move := Vector3.ZERO
	move.x = Input.get_axis("ui_left", "ui_right")
	move.y = Input.get_axis("ui_down", "ui_up")
	if Input.is_key_pressed(KEY_W):
		move.z -= 1.0
	if Input.is_key_pressed(KEY_S):
		move.z += 1.0
	for pad in Input.get_connected_joypads():
		var sx := Input.get_joy_axis(pad, JOY_AXIS_LEFT_X)
		var sy := Input.get_joy_axis(pad, JOY_AXIS_LEFT_Y)
		if absf(sx) > 0.2:
			move.x += sx
		if absf(sy) > 0.2:
			move.y -= sy
		move.z += Input.get_joy_axis(pad, JOY_AXIS_TRIGGER_RIGHT) - Input.get_joy_axis(pad, JOY_AXIS_TRIGGER_LEFT)
	pause_cam += Vector3(move.x * 12.0, move.y * 12.0, move.z * 20.0) * delta
	pause_cam = pause_cam.clamp(Vector3(-25, -15, -pause_cam_base.z + 6.0), Vector3(25, 15, 40))
	cam.position = pause_cam_base + pause_cam


## Leaving a match from the pause screen: back to where it came from (the online screen or character select), or, launched
## directly, quitting. (The run overrides it.)
func _quit_match() -> void:
	if Roster.session.get("from_menu", false):
		# (Leaving an online match goes to the main menu, as players expect: playtest 2026-10-19.)
		_leave_to("menu" if (net_mode or spectate_mode or group_mode or quick_mode) else "select")
	else:
		get_tree().quit()


## True when this machine alone decides when the match runs (not online, a replay or watching), so it may pause or slow it.
func _local_match() -> bool:
	return not (net_mode or group_mode or spectate_mode or replay_mode or quick_mode)


func _refresh(i: int) -> void:
	var info: PackedInt32Array = sim.fighter_info(i)
	var cb: PackedInt32Array = sim.fighter_combat(i)
	cur_pos[i] = sim.fighter_pos(i)
	snaps[i] = {
		"state": sim.fighter_state(i), "state_frame": sim.fighter_state_frame(i),
		"facing": sim.fighter_facing(i), "pos": cur_pos[i], "vel": sim.fighter_vel(i), "kb_vel": sim.fighter_kb_vel(i),
		"char": info[0], "class": sim.fighter_class(i), "platform": info[1], "jumps": info[2], "dodged": info[3] != 0,
		"fast_fall": info[4] != 0, "ledge": info[5], "ledge_invuln": info[6], "grabs": info[7],
		"lag": info[8], "cooldown": info[9], "ignore": info[10], "frame": sim.frame(),
		"percent": sim.fighter_percent(i), "stocks": cb[0], "hitlag": cb[1], "hitstun": cb[2],
		"move_id": cb[3], "tumble": cb[4] != 0, "invuln": cb[5], "launch_pending": cb[6] != 0,
		"charge": sim.fighter_charge(i), "shield": sim.fighter_shield(i) / sim.shield_max(), "shield_tilt": sim.fighter_shield_tilt(i), "move_name": sim.fighter_move_name(i), "move_timing": sim.fighter_move_timing(i),
		"move_windows": sim.fighter_move_windows(i), "dodge": sim.fighter_dodge(i),
		"move_tip": sim.fighter_move_tip(i), "reach": sim.fighter_weapon_reach(i), "dash_length": sim.fighter_dash_length(i), "wall_jumps": sim.fighter_wall_jumps(i),
		"tripped": sim.has_method("fighter_tripped") and bool(sim.fighter_tripped(i)),
		"egg": float(sim.fighter_egg_radius(i)) if sim.has_method("fighter_egg_radius") else 0.0,
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


## How the match is won: the menus' choice (stocks and time limit), or `--stocks=N --time=SECONDS` after `--`. With neither it is
## free play (nobody is eliminated), which keeps demos, training and the test launchers as they were.
## Draws the stage the simulation now has (the match content may have swapped it).
func _rebuild_stage() -> void:
	stage_view.clear()
	stage_view.build(sim)
	_apply_sky()


## The sky comes from the stage's theme (and the colours the stage editor set).
func _apply_sky() -> void:
	if world_env != null and not stage_view.theme.is_empty():
		world_env.sky = StageArt.sky(stage_view.theme)
		world_env.background_mode = Environment.BG_SKY
		# The haze behind the stage takes the colour of the stage's horizon.
		world_env.fog_light_color = stage_view.theme.sky_horizon
		# The light, the shadows' hue and the rim light follow the stage's theme.
		Lighting.stage_mood(world_env, self, stage_view.theme, post_mat)


func _apply_rules() -> void:
	if replay_mode:
		return
	var stocks := 0
	var seconds := 0
	if Roster.session.has("stocks"):
		stocks = int(Roster.session.stocks)
		seconds = int(Roster.session.get("time", 0))
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--stocks="):
			stocks = int(a.substr(9))
		elif a.begins_with("--time="):
			seconds = int(a.substr(7))
	sim.set_match_rules(stocks, seconds)


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
	if Roster.session.has("online") and Roster.session.online.get("watch", false):
		var watch_error: String = sim.spectate_start(Roster.session.online.addr)
		if watch_error == "":
			spectate_mode = true
			return
		push_error("watch: " + watch_error)
		content_note = "CANNOT WATCH: " + watch_error
	# Watching a replay: the file rebuilds the match's content and its first state.
	var replay_bytes := PackedByteArray()
	if Roster.session.has("replay"):
		replay_bytes = Roster.session.replay
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--replay="):
			replay_bytes = Replays.read(a.substr(9))
	if not replay_bytes.is_empty():
		var replay_error: String = sim.replay_load(replay_bytes)
		if replay_error == "":
			replay_mode = true
			var info: Dictionary = sim.replay_peek(replay_bytes)
			for i in PLAYERS:
				var profile := Roster.parse_profile(info["cosmetics%d" % i], i)
				views[i].rebuild(profile.look)
				views[i].set_name_tag(profile.name)
				if profile.name != "":
					names[i] = profile.name
			return
		push_error("replay: " + replay_error)
		content_note = "REPLAY NOT LOADED: " + replay_error
	# Coming from the menus: the match's fighters build the match content (the base roster plus any made fighters), the
	# same way an online match does, so it can be recorded and replayed.
	sim.set_match_stage(int(Roster.session.get("stage", 0)))
	if Roster.session.has("entries") and Roster.session.entries.size() >= 2:
		var specs: Array[PackedByteArray] = []
		for i in PLAYERS:
			specs.append(Roster.spec_bytes(Roster.session.entries[i]))
		var loaded: Dictionary = sim.load_match_roster(specs)
		var load_error: String = loaded.error
		if load_error == "":
			chosen_chars = []
			for c in loaded.chars:
				chosen_chars.append(c)
			while chosen_chars.size() < 4:
				chosen_chars.append(0)
			content_note = ""
			for i in PLAYERS:
				var e: Dictionary = Roster.session.entries[i]
				views[i].rebuild(e.look)
				views[i].set_name_tag(e.name)
				names[i] = e.name
			return
		push_error("content: " + load_error)
		content_note = "CONTENT NOT LOADED: " + load_error
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
				names[i] = e.name
			return
		push_error("content: " + err_text)
		content_note = "CONTENT NOT LOADED: " + err_text
	var asked := path != ""
	if not asked:
		path = Roster.base_content_path()
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
var net_failed := ""
var net_lines: Array[String] = []
## The online lobby while it is open (`ui/net_lobby.gd`), and the choice it last had (selected again in the next one).
var net_lobby: CanvasLayer = null
var lobby_fighter := -1
var lobby_stage := 0
## Online duels from the online screen go through the lobby before every match.
var lobby_on := false
## The code socket lent to this match (a bridge `NetCode`), given back when the match is left so the code stays the same.
var lent_code: RefCounted = null


## The connection to make, from the online screen (`Roster.session.online`) or from launch arguments. Empty means no network.
func _net_config() -> Dictionary:
	if Roster.session.has("online"):
		var o: Dictionary = Roster.session.online
		if o.get("watch", false):
			return {}
		return {
			"host": o.host, "relay": o.addr if o.relay else "", "addr": o.addr, "port": o.port, "room": o.room,
			"delay": o.delay, "chars": chosen_chars, "ranked": o.ranked, "group": o.get("group", false),
			"quick": o.get("quick", false), "netcode": o.get("netcode", null),
		}
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
	if host_port < 0 and join_addr == "":
		return {}
	return {
		"host": host_port >= 0, "relay": relay, "addr": join_addr, "port": host_port, "room": room, "delay": delay,
		"chars": chars, "ranked": OS.get_cmdline_user_args().has("--ranked"), "group": false, "quick": false,
	}


func _start_net() -> void:
	var cfg := _net_config()
	if cfg.is_empty():
		return
	# Everyone brings their own fighter (the online screen's choice, `--fighter=<slug>`, or the one last played in the menus)
	# and its look and name; a few bytes of each travel to the other side in the handshake. The ranked rule is the host's.
	var me := Roster.net_entry()
	sim.set_match_stage(int(Roster.session.get("stage", 0)))
	sim.set_cosmetics(Roster.profile_bytes(me))
	sim.set_fighter(Roster.spec_bytes(me))
	sim.set_ranked(cfg.ranked)
	local_slot = 0 if cfg.host else 1
	var err := ""
	if cfg.quick:
		err = sim.quickmatch_start(cfg.addr)
		if err != "":
			push_error(err)
			net_failed = err
			return
		quick_mode = true
		quick_cfg = cfg
		print("quick match: looking")
		return
	_connect_pair(cfg, me)


## Connects to the opponent once the host/join question is settled (typed rooms, or what the quick-match queue decided).
func _connect_pair(cfg: Dictionary, me: Dictionary) -> void:
	var err := ""
	if cfg.group:
		if cfg.host:
			err = sim.group_host_start(cfg.port, cfg.delay)
		else:
			err = sim.group_join(cfg.addr)
		if err != "":
			push_error(err)
			net_failed = err
			return
		group_mode = true
		names[0] = me.name
		print("group mode: ", "host" if cfg.host else "guest")
		return
	var code: RefCounted = cfg.get("netcode", null)
	lent_code = code
	# A duel from the online screen starts in the lobby: both players choose a fighter and a stage there (the owner, 19 October).
	lobby_on = Roster.session.has("online")
	sim.net_use_lobby(lobby_on)
	if lobby_on:
		lobby_stage = int(Roster.session.get("stage", 0))
		sim.set_match_stage(lobby_stage)
	if code != null:
		# Connected by code on the online screen: the match plays over the socket that found the way through.
		if cfg.host:
			err = sim.net_host_code(code, PackedInt32Array(cfg.chars), cfg.delay)
		else:
			err = sim.net_join_code(code)
	elif cfg.host and cfg.relay != "":
		err = sim.net_host_relay(cfg.relay, cfg.room, PackedInt32Array(cfg.chars), cfg.delay)
	elif cfg.host:
		err = sim.net_host(cfg.port, PackedInt32Array(cfg.chars), cfg.delay)
	elif cfg.relay != "":
		err = sim.net_join_relay(cfg.relay, cfg.room)
	else:
		err = sim.net_join(cfg.addr)
	if err != "":
		push_error(err)
		net_lines.append(err)
		net_failed = err
		return
	net_mode = true
	views[local_slot].rebuild(me.look)
	views[local_slot].set_name_tag(me.name)
	names[local_slot] = me.name
	print("network mode: ", "host" if cfg.host else "joiner")


## Every fighter is drawn at its body size (the simulation's `hitbox_scale`, which comes from its size stat), and a fighter whose body is
## broader than the standard one (the Hauler's: its ECB half width against `STANDARD_HALF_WIDTH`) that much bigger again, so the big
## fighter looks as big as it is to hit. (Taller-but-standard-width bodies keep their drawn size.)
const STANDARD_HALF_WIDTH := 0.8


func _apply_scales() -> void:
	for i in PLAYERS:
		var broad := 1.0
		var body: PackedFloat32Array = sim.fighter_body(i)
		if body.size() > 0 and body[0] > STANDARD_HALF_WIDTH + 0.01:
			broad = body[0] / STANDARD_HALF_WIDTH
		views[i].scale = Vector3.ONE * sim.fighter_scale(i) * broad


## One frame of networked play: read the local keyboard, let the rollback session simulate (or wait), then draw.
## A controller plays during a match, so it only navigates on the results screen and when watching a replay.
func pad_scheme(_pad: int) -> Dictionary:
	# The pause screen: the controller steers it like a menu (Start or B resumes); during play Start opens it.
	if pause_menu != null:
		return {"up": KEY_UP, "down": KEY_DOWN, "confirm": KEY_ENTER, "back": KEY_ESCAPE, "start": KEY_ESCAPE}
	if replay_mode:
		return {"up": KEY_UP, "down": KEY_DOWN, "left": KEY_LEFT, "right": KEY_RIGHT, "confirm": KEY_SPACE, "back": KEY_ESCAPE, "start": KEY_SPACE, "menu": KEY_ESCAPE}
	if results != null or net_lobby != null:
		return PadNav.DEFAULT
	if _can_pause():
		return {"start": KEY_ESCAPE}
	return {}


## Watching a host's match: the bridge plays the stream a little behind the live game.
func _spectate_step() -> void:
	spectate_status = sim.spectate_update()
	if spectate_status >= 1 and not spectate_looks_applied and sim.spectate_cosmetics(0).size() > 0:
		spectate_looks_applied = true
		for i in 2:
			var profile := Roster.parse_profile(sim.spectate_cosmetics(i), i)
			views[i].rebuild(profile.look)
			views[i].set_name_tag(profile.name)
			if profile.name != "":
				names[i] = profile.name
		_rebuild_stage()
		_apply_scales()
	if results != null and sim.winner() == -1:
		# The host started a rematch and the stream moved on to it.
		results.queue_free()
		results = null
		end_timer = 0.0
		_rebuild_stage()
		_apply_scales()
	if spectate_status == 1:
		_tick_once(false)
	else:
		overlay_dirty = true


## Looking for an opponent: when the relay pairs us, the one who waited hosts and the other joins, in the room it names.
func _quick_step() -> void:
	var found: Dictionary = sim.quickmatch_poll()
	if found.is_empty():
		overlay_dirty = true
		return
	quick_mode = false
	var cfg := quick_cfg.duplicate()
	cfg.quick = false
	cfg.host = found.host
	cfg.room = found.room
	cfg.relay = cfg.addr
	local_slot = 0 if cfg.host else 1
	print("quick match: found room ", found.room, " as ", "host" if cfg.host else "joiner")
	_connect_pair(cfg, Roster.net_entry())


## A group match (three or four players): the lobby, then the match.
func _group_step() -> void:
	var r: Dictionary = InputReader.read(0, masks)
	inputs[0] = r
	group_status = sim.group_update(r.x, r.y, r.buttons)
	for line in sim.group_take_log():
		net_lines.append(line)
		print(line)
	if net_lines.size() > 4:
		net_lines = net_lines.slice(net_lines.size() - 4)
	if group_status == 1 or group_status == 2:
		# Names and looks by slot, once the match has started (and again for a rematch).
		if not group_looks_applied:
			group_looks_applied = true
			var lobby: Array = sim.group_lobby()
			for i in mini(lobby.size(), PLAYERS):
				var profile := Roster.parse_profile(lobby[i], i)
				views[i].rebuild(profile.look)
				views[i].set_name_tag(profile.name)
				names[i] = profile.name if profile.name != "" else "Player %d" % (i + 1)
			_rebuild_stage()
			_apply_scales()
		if results != null and sim.winner() == -1:
			results.queue_free()
			results = null
			end_timer = 0.0
			_rebuild_stage()
			_apply_scales()
		if group_status == 1:
			_tick_once(false)
	else:
		overlay_dirty = true


## What the lobby looks like in words.
func _group_text() -> String:
	var lobby: Array = sim.group_lobby()
	var who := []
	for i in lobby.size():
		if lobby[i].size() > 0:
			var n: String = Roster.parse_profile(lobby[i], i).name
			who.append(n if n != "" else "Player %d" % (i + 1))
	match group_status:
		0:
			var line := "LOBBY (%d joined): %s" % [sim.group_players(), ", ".join(who)]
			if sim.group_slot() == 0:
				line += "\nPress Enter to start the match"
			else:
				line += "\nWaiting for the host to start..."
			return line
		3:
			return "The host refused this match: another version, a full lobby, or a fighter that is not allowed."
		4:
			return "Starting the match..."
		2:
			return "Waiting for the other players..."
	return ""


func _replay_step() -> void:
	if paused:
		return
	replay_accum += replay_speed
	while replay_accum >= 1.0:
		replay_accum -= 1.0
		if not sim.replay_tick():
			break
		_tick_once(false)


func _replay_seek(frame: int) -> void:
	sim.replay_seek(clampi(frame, 0, sim.replay_length()))
	_tick_once(false)
	for i in PLAYERS:
		prev_pos[i] = cur_pos[i]


func _replay_key(event: InputEventKey) -> void:
	match event.keycode:
		KEY_SPACE, KEY_P:
			paused = not paused
		KEY_LEFT:
			_replay_seek(sim.match_frame() - 300)
		KEY_RIGHT:
			_replay_seek(sim.match_frame() + 300)
		KEY_UP:
			replay_speed = minf(replay_speed * 2.0, 4.0)
		KEY_DOWN:
			replay_speed = maxf(replay_speed / 2.0, 0.25)
		KEY_R:
			_replay_seek(0)
		KEY_PERIOD:
			if paused and sim.replay_tick():
				_tick_once(false)
		KEY_COMMA:
			if paused:
				_replay_seek(sim.match_frame() - 1)
		KEY_F2:
			show_ecb = not show_ecb
		KEY_F3:
			show_boxes = not show_boxes
			_rebuild_boxes()
		KEY_ESCAPE:
			get_tree().change_scene_to_file("res://replays.tscn")


func _replay_text() -> String:
	var at: int = sim.match_frame() / 60
	var total: int = sim.replay_length() / 60
	return "REPLAY  %d:%02d / %d:%02d   x%s%s\nSpace pause   Left / Right jump 5 s   Up / Down speed   R restart   Esc back" % [at / 60, at % 60, total / 60, total % 60, str(replay_speed), "   PAUSED" if paused else ""]


func _net_step() -> void:
	var local: int = maxi(sim.net_local_player(), 0)
	var r: Dictionary = InputReader.read(0, masks, sim.fighter_keeps_dash(local))
	inputs[local] = r
	net_status = sim.net_update(r.x, r.y, r.buttons)
	if net_status == 4:
		_lobby_step()
		return
	if net_lobby != null and net_status != 3:
		_close_lobby()
	elif net_lobby != null:
		_lobby_left()
	# Once the handshake is done the other player's look arrives (or is missing, and they keep the default look).
	if (net_status == 1 or net_status == 2) and not their_look_applied:
		their_look_applied = true
		_rebuild_stage()
		var theirs := Roster.parse_profile(sim.net_their_cosmetics(), 1 - local_slot)
		views[1 - local_slot].rebuild(theirs.look)
		views[1 - local_slot].set_name_tag(theirs.name)
		names[1 - local_slot] = theirs.name
		_apply_scales()
	for line in sim.net_take_log():
		net_lines.append(line)
		print(line)
	if net_lines.size() > 4:
		net_lines = net_lines.slice(net_lines.size() - 4)
	if results != null and net_status == 1 and sim.winner() == -1:
		# The rematch is on: a fresh match has replaced the finished one.
		results.queue_free()
		results = null
		end_timer = 0.0
		_apply_scales()
		_rebuild_stage()
	elif results != null and sim.net_rematch_state()[1] == 1:
		results.set_note("The other player wants a rematch!" if sim.net_rematch_state()[0] == 0 else "Rematch: starting...")
	if net_status == 1:
		_tick_once(false)
	else:
		overlay_dirty = true


## The lobby is open (net_update said 4): open its screen if it is not up yet (the first match, or a rematch that both asked
## for, which closes the results), and show the other player's choices.
func _lobby_step() -> void:
	if results != null:
		results.queue_free()
		results = null
		end_timer = 0.0
	if net_lobby == null:
		_open_lobby()
	var view: Dictionary = sim.net_lobby_view()
	net_lobby.show_view(view.get("theirs", {}))
	for line in sim.net_take_log():
		net_lines.append(line)
	overlay_dirty = true


func _open_lobby() -> void:
	_close_pause()
	var entries := Roster.all()
	if lobby_fighter < 0:
		var me := Roster.net_entry()
		lobby_fighter = 0
		for i in entries.size():
			if entries[i].slug == me.slug:
				lobby_fighter = i
	net_lobby = NetLobby.new()
	add_child(net_lobby)
	# The match's display (timer, percents) hides while the lobby is up: it showed through the lobby's veil, the timer's plate peeking
	# out above the LOBBY banner (the UI pass, docs/art/ui/UI_ANALYSIS.md).
	if hud != null:
		hud.visible = false
	net_lobby.build(entries, lobby_fighter, lobby_stage)
	net_lobby.set_friend_name(names[1 - local_slot] if names[1 - local_slot] != "Player %d" % (2 - local_slot) else "")
	net_lobby.changed.connect(_on_lobby_changed)
	net_lobby.leave.connect(_quit_match)
	# A new match will bring the other player's look again, and its own stage.
	their_look_applied = false
	_on_lobby_changed(lobby_fighter, lobby_stage, false, false)


func _on_lobby_changed(fighter_index: int, stage: int, fighter_locked: bool, stage_locked: bool) -> void:
	var entries := Roster.all()
	lobby_fighter = clampi(fighter_index, 0, entries.size() - 1)
	lobby_stage = stage
	var e: Dictionary = entries[lobby_fighter]
	sim.net_lobby_set(Roster.spec_bytes(e), Roster.profile_bytes(e), fighter_locked, stage, stage_locked)


## Both locked in: the match is starting. This player's fighter is the one chosen in the lobby.
func _close_lobby() -> void:
	var entries := Roster.all()
	var e: Dictionary = entries[clampi(lobby_fighter, 0, entries.size() - 1)]
	Roster.save_last(e.slug)
	views[local_slot].rebuild(e.look)
	views[local_slot].set_name_tag(e.name)
	names[local_slot] = e.name
	net_lobby.queue_free()
	net_lobby = null
	_show_display(not hud_hidden)


## The other player left while the lobby was open.
func _lobby_left() -> void:
	for line in sim.net_take_log():
		net_lines.append(line)
	net_lobby.show_left()


func _tick_once(advance := true) -> void:
	var t0 := Time.get_ticks_usec()
	_tick_once_inner(advance)
	if perf and warmed:
		perf_tick += float(Time.get_ticks_usec() - t0) / 1000.0


func _tick_once_inner(advance := true) -> void:
	for i in PLAYERS:
		prev_pos[i] = cur_pos[i]
	if advance:
		sim.tick()
	proj_prev = proj_cur
	proj_cur = sim.projectile_slots()
	var befores: Array = []
	for i in PLAYERS:
		befores.append(snaps[i])
		_refresh(i)
	_track_stats(befores)
	for i in PLAYERS:
		var before: Dictionary = befores[i]
		sfx.watch(i, before, snaps[i])
		# A hit is hitlag starting with damage taken (a flinch or one hit of a multi-hit move counts too); a block is hitlag starting in
		# the shield.
		var struck := int(before.get("hitlag", 0)) == 0 and int(snaps[i].hitlag) > 0
		if struck and (snaps[i].launch_pending or float(snaps[i].percent) > float(before.get("percent", 0.0)) + 0.01):
			_on_hit(i, befores)
		elif struck and snaps[i].state == "Shield":
			_on_block(i)
		# A strong launch has just begun (its hitlag over): a blue streak along the line it flies, as bent by DI.
		if before.get("launch_pending", false) and not snaps[i].launch_pending and snaps[i].get("tumble", false) and _in_play(i):
			var kb: Vector2 = sim.fighter_kb_vel(i)
			if kb.length() > 0.01:
				Effects.launch_line(self, Vector3(cur_pos[i].x, cur_pos[i].y + 1.1, 0.9), kb.normalized())
		_on_events(i, before, snaps[i])
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
		if im.get_surface_count() > 0:
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
			inputs[i] = InputReader.read(0, masks, sim.fighter_keeps_dash(0))
		elif cpu_levels[i] > 0 and (demo == null or not demo.cpu_levels.is_empty()) and not replay_mode:
			var c: PackedInt32Array = sim.cpu_input(i, cpu_levels[i])
			inputs[i] = {"x": c[0], "y": c[1], "buttons": c[2]}
		elif demo != null:
			if i == 0:
				inputs[i] = demo.input_at(sim.frame())
			elif i == 1 and demo.timeline2.size() > 0:
				inputs[i] = demo.input_at2(sim.frame())
			else:
				inputs[i] = {"x": 0, "y": 0, "buttons": 0}
		else:
			inputs[i] = InputReader.read(i, masks, sim.fighter_keeps_dash(i))
		sim.set_input(i, inputs[i].x, inputs[i].y, inputs[i].buttons)


func _physics_process(_delta: float) -> void:
	if not warmed or shot_wait != "":
		return
	if replay_mode:
		_replay_step()
		return
	if spectate_mode:
		_spectate_step()
		return
	if group_mode:
		_group_step()
		return
	if quick_mode:
		_quick_step()
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
	if countdown > 0.0:
		countdown -= 1.0 / 60.0
		var shown := ceili(countdown)
		if shown != countdown_shown:
			countdown_shown = shown
			sfx.play("go" if shown == 0 else "blip", 1.0 if shown == 0 else 1.2)
		overlay_dirty = true
		return
	_tick_once()
	if demo != null:
		var shot: String = demo.shot_at(sim.frame())
		if shot != "":
			shot_wait = shot
			_capture(shot)
		elif demo.key_poses and shot_wait == "":
			var key := _key_pose()
			if key != "":
				shot_wait = key
				_capture(key)
		if sim.frame() >= demo.end_frame and shot_wait == "":
			if perf:
				var sorted := perf_deltas.duplicate()
				sorted.sort()
				var p99: float = sorted[mini(sorted.size() - 1, int(sorted.size() * 0.99))] if sorted.size() > 0 else 0.0
				print("PERF p99 %.2f ms" % p99)
				print("PERF %d frames in %.2fs = %.1f fps, %.2f ms avg, worst %.1f ms, %d frames over 25 ms, %d draw calls, %d primitives, GPU %.2f ms, render CPU %.2f ms, ticks %.2f ms, frame scripts %.2f ms" % [perf_frames, perf_time, perf_frames / perf_time, 1000.0 * perf_time / perf_frames, perf_worst * 1000.0, perf_hitches, perf_draw / perf_frames, perf_prims / perf_frames, perf_gpu / perf_frames, perf_cpu / perf_frames, perf_tick / perf_frames, perf_frame / perf_frames])
			print("demo '%s' finished at frame %d checksum %s" % [demo.name, sim.frame(), sim.checksum()])
			get_tree().quit()


## For the pose atlas (`--demo=poses`): player 1's move name and pose ("a" wind-up, "b" strike, "c" follow-through) when this frame is one.
var key_pose_done := {}
func _key_pose() -> String:
	var move: String = sim.fighter_move_name(0)
	var t: PackedInt32Array = sim.fighter_move_timing(0)
	if move == "" or t.size() < 3 or t[1] <= 0:
		return ""
	var f: int = sim.fighter_state_frame(0)
	for k in [[maxi(1, t[1] - 4), "a"], [t[1], "b"], [mini(t[0] - 1, t[2] + 4), "c"]]:
		var label := "%s_%s_%s" % [demo.name, move, k[1]]
		if f == k[0] and not key_pose_done.has(label):
			key_pose_done[label] = true
			return label
	return ""


## `--pick=x,y` (with a demo's shots): prints every mesh whose on-screen bounds cover that pixel, nearest first, with its material, so
## the art can find what draws a given spot without guessing (art review 6: several rounds spent hunting a mesh by elimination).
func _pick(px: Vector2) -> void:
	var cam := get_viewport().get_camera_3d()
	if cam == null:
		return
	var hits := []
	for n in get_tree().root.find_children("*", "MeshInstance3D", true, false):
		var mi := n as MeshInstance3D
		if not mi.is_visible_in_tree() or mi.mesh == null:
			continue
		var box: AABB = mi.global_transform * mi.get_aabb()
		var lo := Vector2(INF, INF)
		var hi := Vector2(-INF, -INF)
		var ahead := false
		for k in 8:
			var c := box.get_endpoint(k)
			if cam.is_position_behind(c):
				continue
			ahead = true
			var sp := cam.unproject_position(c)
			lo = lo.min(sp)
			hi = hi.max(sp)
		if ahead and px.x >= lo.x and px.x <= hi.x and px.y >= lo.y and px.y <= hi.y:
			var mat = mi.material_override if mi.material_override != null else mi.mesh.surface_get_material(0)
			var mat_name := "none" if mat == null else (str(mat.resource_name) if mat.resource_name != "" else mat.get_class() + ("(" + str((mat as ShaderMaterial).shader.resource_path) + ")" if mat is ShaderMaterial and (mat as ShaderMaterial).shader != null else ""))
			hits.append([cam.global_position.distance_to(box.get_center()), str(mi.get_path()), mat_name, box.size])
	hits.sort_custom(func(a, b): return a[0] < b[0])
	print("PICK %s: %d meshes" % [px, hits.size()])
	for h in hits.slice(0, 12):
		print("  %.1f  %s  [%s]  size %s" % h)


func _capture(label: String) -> void:
	await RenderingServer.frame_post_draw
	await RenderingServer.frame_post_draw
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--pick="):
			var xy := a.substr(7).split(",")
			_pick(Vector2(float(xy[0]), float(xy[1])))
	var img := get_viewport().get_texture().get_image()
	var path := "%s/%s.png" % [demo.out_dir if demo.out_dir != "" else "user://", label]
	img.save_png(path)
	# The camera beside it (for authoring stage art in Blender from the same view, docs/art/stages/BAKE_PILOT.md).
	var c := get_viewport().get_camera_3d()
	if c != null:
		var o := c.global_transform.origin
		var f := -c.global_transform.basis.z
		var u := c.global_transform.basis.y
		FileAccess.open(path.get_basename() + ".cam.json", FileAccess.WRITE).store_string(JSON.stringify({"fov": c.fov,
			"origin": [o.x, o.y, o.z], "forward": [f.x, f.y, f.z], "up": [u.x, u.y, u.z],
			"size": [img.get_width(), img.get_height()]}))
	print("saved ", path)
	shot_wait = ""


func _alpha() -> float:
	if paused or shot_wait != "" or demo != null:
		return 1.0
	return Engine.get_physics_interpolation_fraction()


func _process(delta: float) -> void:
	var t0 := Time.get_ticks_usec()
	_process_inner(delta)
	_pause_camera(delta)
	_sun_rays()
	if perf and warmed:
		perf_frame += float(Time.get_ticks_usec() - t0) / 1000.0


func _process_inner(delta: float) -> void:
	if perf and warmed:
		perf_frames += 1
		perf_time += delta
		perf_worst = maxf(perf_worst, delta)
		perf_deltas.append(delta * 1000.0)
		if delta > 0.025:
			perf_hitches += 1
		perf_draw += int(RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_TOTAL_DRAW_CALLS_IN_FRAME))
		perf_prims += int(RenderingServer.get_rendering_info(RenderingServer.RENDERING_INFO_TOTAL_PRIMITIVES_IN_FRAME))
		var vp := get_viewport().get_viewport_rid()
		perf_gpu += RenderingServer.viewport_get_measured_render_time_gpu(vp)
		perf_cpu += RenderingServer.viewport_get_measured_render_time_cpu(vp) + RenderingServer.get_frame_setup_time_cpu()
	var a := _alpha()
	# Effects run on the match's clock (see FxMaterial.tick).
	FxMaterial.tick((float(sim.frame()) + a) / 60.0)
	if post_mat != null:
		var two_tone := 1.0 if int(sim.frame()) < impact_until else 0.0
		if float(post_mat.get_shader_parameter("impact")) != two_tone:
			post_mat.set_shader_parameter("impact", two_tone)
	for i in PLAYERS:
		var p: Vector2 = prev_pos[i].lerp(cur_pos[i], a)
		views[i].apply(Vector3(p.x, p.y, 0), snaps[i], delta)
		# A fighter who has lost its last stock leaves the stage.
		views[i].visible = _in_play(i)
	_update_hud(delta)
	_update_camera(a, delta)
	_update_effects(delta)
	if not flag_noecb:
		_update_ecb(a)
	_position_boxes(a)
	_update_projectiles(a)
	_update_items()
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


## How close (world units, across and up) two fighters are for their tags to step apart.
const TAG_CROWD := Vector2(2.2, 2.6)


func _update_hud(delta: float) -> void:
	for i in mini(PLAYERS, views.size()):
		views[i].set_overhead(overlay_on and not flag_noui, "P%d" % (i + 1), PLAYER_COLORS[i % PLAYER_COLORS.size()])
	# Close combat: the tags of fighters near each other step apart toward their own sides and fade a little.
	for i in mini(PLAYERS, views.size()):
		if not _in_play(i):
			continue
		var push := 0.0
		var near := false
		for j in mini(PLAYERS, views.size()):
			if j == i or not _in_play(j):
				continue
			var d: Vector2 = cur_pos[i] - cur_pos[j]
			if absf(d.x) < TAG_CROWD.x and absf(d.y) < TAG_CROWD.y:
				near = true
				push += (TAG_CROWD.x - absf(d.x)) * (1.0 if d.x > 0.0 or (d.x == 0.0 and i > j) else -1.0)
		views[i].nudge_tag(clampf(push * 0.45, -1.0, 1.0), 0.75 if near else 1.0, delta)
	var rules: PackedInt32Array = sim.match_rules()
	var frame: int = sim.match_frame()
	var winner: int = sim.winner()
	var playing: bool = not net_mode or net_status == 1
	var clock := ""
	var urgent := false
	var sudden: bool = sim.has_method("sudden_death") and sim.sudden_death()
	if sudden and sudden_from < 0:
		sudden_from = frame
		sfx.play("whoosh", 0.6)
		cam_shake = maxf(cam_shake, 0.4)
	elif not sudden:
		sudden_from = -1
	if sudden:
		pass
	elif rules[1] > 0:
		var left: int = ceili(maxi(0, rules[1] * 60 - frame) / 60.0)
		clock = "%02d:%02d" % [left / 60, left % 60]
		urgent = left <= 10 and winner == -1
	elif rules[0] > 0:
		clock = "%02d:%02d" % [frame / 3600, (frame / 60) % 60]
	var banner := ""
	var alpha := 1.0
	if countdown > 0.0:
		banner = "%d" % ceili(countdown)
		alpha = clampf(1.0 - (float(ceili(countdown)) - countdown) * 0.35, 0.65, 1.0)
	elif winner != -1:
		banner = "TIME!" if (rules[1] > 0 and frame >= rules[1] * 60 and not sudden) else "GAME!"
	elif sudden and frame - sudden_from < 150:
		# Sudden death: the time ran out level on stocks; the leaders play on at 300% with one stock each.
		banner = "SUDDEN DEATH!"
		alpha = clampf(float(150 - (frame - sudden_from)) / 40.0, 0.0, 1.0)
	elif playing and frame < 75 and (rules[0] > 0 or rules[1] > 0):
		banner = "GO!"
		alpha = clampf(float(75 - frame) / 30.0, 0.0, 1.0)
	var percent := []
	var stocks := []
	var alive := []
	var in_match := []
	var classes := []
	var looks := []
	# The Boost gauge where a fighter has one (the Champion): [fill 0..1, the powered mode running, its time left 0..1], or [] for
	# none. (Read only when the bridge has it.)
	var gauges := []
	var has_gauge: bool = sim.has_method("fighter_gauge_max")
	for i in PLAYERS:
		var g := []
		if has_gauge:
			var top := float(sim.fighter_gauge_max(i))
			if top > 0.0:
				# (The powered mode's length is the body's; 900 frames for the Champion, until the bridge gives it.)
				var mode_frames: float = float(sim.fighter_gauge_frames_max(i)) if sim.has_method("fighter_gauge_frames_max") else 900.0
				g = [clampf(float(sim.fighter_gauge(i)) / top, 0.0, 1.0), bool(sim.fighter_gauge_full(i)), clampf(float(sim.fighter_gauge_frames(i)) / mode_frames, 0.0, 1.0)]
		gauges.append(g)
		classes.append(int(snaps[i].get("class", snaps[i].get("char", 0))))
		looks.append(views[i].loadout if i < views.size() else null)
		percent.append(snaps[i].get("percent", 0.0))
		stocks.append(snaps[i].get("stocks", 0))
		alive.append(sim.fighter_active(i))
		in_match.append(sim.fighter_in_roster(i) and not (group_mode and group_status in [0, 3, 4]))
	hud.show_state({
		"names": names.slice(0, PLAYERS), "percent": percent, "stocks": stocks, "alive": alive, "in_match": in_match,
		"unlimited": rules[0] == 0, "clock": clock, "urgent": urgent, "banner": banner, "banner_alpha": alpha,
		"counting_down": rules[1] > 0, "classes": classes, "looks": looks, "gauges": gauges,
		"status": _net_status_text(),
		"offscreen": _offscreen_markers(),
	})
	# (Not while a new online match is still being set up: the last match's winner is still in the state then.)
	if winner != -1 and results == null and net_lobby == null and not (net_mode and net_status in [0, 4]) and not replay_mode and not (spectate_mode and spectate_status != 1 and sim.spectate_behind() > 0):
		end_timer += delta
		if end_timer > 2.0:
			_show_results(winner)


## Writes the finished match to the replay folder (it is simply not saved if the match cannot be reproduced).
func _save_replay() -> String:
	var profiles := []
	var entries: Array = Roster.session.get("entries", [])
	for i in PLAYERS:
		var e: Dictionary = entries[i] if i < entries.size() else {"look": Loadout.default_for(i), "name": names[i]}
		profiles.append(Roster.profile_bytes(e))
	var list: Array[PackedByteArray] = []
	for p in profiles:
		list.append(p)
	return Replays.save(sim.replay_bytes_roster(list))


func _show_results(winner: int) -> void:
	var saved := _save_replay() if not spectate_mode else ""
	var cards := []
	for i in PLAYERS:
		var st: Dictionary = stats[i] if i < stats.size() else {}
		cards.append({"name": names[i], "stocks": snaps[i].get("stocks", 0), "percent": snaps[i].get("percent", 0.0), "winner": i == winner,
			"kos": st.get("kos", 0), "falls": st.get("falls", 0), "dealt": st.get("dealt", 0.0), "taken": st.get("taken", 0.0),
			"look": views[i].loadout, "class": sim.fighter_class(i)})
	var heading := "DRAW!" if winner < 0 else "%s wins!" % names[winner]
	var choices := []
	if group_mode:
		choices = [["Rematch", "rematch"], ["Main Menu", "menu"]] if sim.group_slot() == 0 else [["Main Menu", "menu"]]
	elif spectate_mode:
		choices = [["Back", "menu"]]
	elif net_mode and Roster.session.get("from_menu", false):
		choices = [["Rematch", "rematch"], ["Main Menu", "menu"]]
	elif net_mode:
		choices = [["Rematch", "rematch"], ["Quit", "quit"]]
	elif Roster.session.get("from_menu", false):
		choices = [["Rematch", "rematch"], ["Character Select", "select"], ["Main Menu", "menu"]]
	else:
		choices = [["Rematch", "rematch"], ["Quit", "quit"]]
	# The stage behind the results goes soft (depth of field kept for presentation, never for play), so the cards and the winner lead.
	var dof := CameraAttributesPractical.new()
	dof.dof_blur_far_enabled = true
	dof.dof_blur_far_distance = 2.0
	dof.dof_blur_far_transition = 6.0
	dof.dof_blur_amount = RESULTS_BLUR
	cam.attributes = dof
	# (The match display steps aside while the results are up.)
	hud_before_results = hud.visible
	hud.visible = false
	results = Results.new()
	add_child(results)
	results.build(heading, cards, choices)
	results.chosen.connect(_on_result)
	if saved != "":
		results.set_note("Replay saved. Watch it from the main menu.")


func _on_result(action: String) -> void:
	match action:
		"rematch":
			if group_mode:
				sim.group_restart()
				results.set_note("Starting another match with the same players...")
			elif net_mode:
				sim.net_request_rematch()
				results.set_note("Rematch requested. Waiting for the other player...")
			else:
				_restart()
		"select":
			_leave_to("select")
		"menu":
			_leave_to("menu")
		_:
			get_tree().quit()


## Back to a menu screen, hanging up first if this was an online match.
func _leave_to(screen: String) -> void:
	Engine.time_scale = 1.0
	if quick_mode:
		sim.quickmatch_stop()
		quick_mode = false
		if screen == "menu":
			screen = "online"
	if group_mode:
		sim.net_leave()
		group_mode = false
		if screen == "menu":
			screen = "online"
	if spectate_mode:
		sim.spectate_stop()
		spectate_mode = false
		if screen == "menu":
			screen = "online"
	if net_mode:
		# A match by code gives its socket back, so this player's code stays the same (`Roster.netcode`).
		if lent_code != null:
			sim.net_leave_to_code(lent_code)
			lent_code = null
		else:
			sim.net_leave()
		net_mode = false
	get_tree().change_scene_to_file("res://%s.tscn" % screen)


## What the player needs to know about the connection: waiting, refused, desyncs, the other player leaving. Empty when all is well.
func _net_status_text() -> String:
	if replay_mode:
		return _replay_text()
	if group_mode:
		var text := _group_text()
		for l in net_lines:
			if l.contains("DESYNC") or l.contains("disconnected"):
				text += "\n" + l.left(90)
		return text.strip_edges()
	if spectate_mode:
		match spectate_status:
			0:
				return "Watching: connecting to the match..."
			2:
				return "Watching: waiting for the host's frames (%d buffered)" % sim.spectate_behind()
			3:
				return "This match cannot be watched: it runs another version of the game."
		return "WATCHING   (Esc to leave)"
	if quick_mode:
		return "Looking for an opponent...   (Esc to cancel)"
	if not net_mode:
		return net_failed
	var lines: Array = []
	match net_status:
		0:
			lines.append("Connecting... waiting for the other player.")
		2:
			lines.append("Waiting for the other player's moves...")
		3:
			lines.append("The match was refused.")
	for l in net_lines:
		if net_status != 1 or l.contains("DESYNC") or l.contains("disconnected"):
			lines.append(l.left(90))
	return "\n".join(lines.slice(maxi(0, lines.size() - 3)))


func _net_text() -> String:
	if not net_mode:
		return ""
	var text: String = sim.net_info()
	if net_status == 2:
		text += "   (waiting for the other player)"
	for l in net_lines:
		text += "\n" + l
	return text


## False for a fighter that lost its last stock.
func _in_play(i: int) -> bool:
	return sim.fighter_active(i) or not sim.fighter_in_roster(i)


## A fighter was just hit (its hitlag started): the camera shakes with the hit's strength, and if the launch will knock the fighter
## out, it closes in on them for a moment (presentation only).
## How far the camera aims below the fighters, per unit of distance (about 4% of the view's height), to keep them above the HUD.
const HUD_LIFT := 0.022
## How soft the stage behind the results screen goes.
const RESULTS_BLUR := 0.09
## Far scenery's blur in play (pixels at 1080 lines, `far_blur` in shaders/post.gdshader): the Best tier only. (3, was 1, art review 10:
## a camera's focus on the fight, so the far scenery's fine detail calms behind it.) (1, 2026-10-19: the far scenery is now big clean
## folded planes with no fine detail to calm, and the owner's reference is crisp to the horizon; at 3 the folds smeared.)
const PLAY_BLUR := 1.0
## Hitlag (frames) above which a hit shakes the camera.
const HIT_SHAKE_FROM := 9.0


## The controller that plays fighter `i` on this machine, or -1 (a computer player, a replay, someone online).
func _pad_of(i: int) -> int:
	if replay_mode or spectate_mode or demo != null:
		return -1
	var pad := -1
	if _local_match():
		if cpu_levels[i] == 0:
			pad = i
	elif i == local_slot:
		pad = 0
	return pad if pad >= 0 and pad in Input.get_connected_joypads() else -1


## Rumbles the controller playing fighter `i` (if it has rumble on): `weak` and `strong` motors 0..1 for `seconds`.
func _rumble(i: int, weak: float, strong: float, seconds: float) -> void:
	if i < 0:
		return
	var pad := _pad_of(i)
	if pad >= 0 and PadBindings.rumble(pad):
		Input.start_joy_vibration(pad, clampf(weak, 0.0, 1.0), clampf(strong, 0.0, 1.0), seconds)


func _on_hit(i: int, befores: Array = []) -> void:
	var lag: int = snaps[i].hitlag
	var strength := clampf((float(lag) - 4.0) / 14.0, 0.0, 1.0)
	# Rumble: the fighter hit feels it with the hit's weight, through its hit freeze. (Who landed it gets a lighter one, below.)
	_rumble(i, 0.35 + 0.4 * strength, 0.15 + 0.85 * strength, maxf(0.08, float(lag) / 60.0))
	# The camera shakes only for a hit that matters (more than a jab's or a tilt's hitlag), harder the stronger it is.
	cam_shake = maxf(cam_shake, clampf((float(lag) - HIT_SHAKE_FROM) * 0.045, 0.0, 0.9))
	if not _in_play(i):
		return
	# Who landed it: the fighter that froze in an attack on the same frame (or the last one to hit this fighter).
	var by := -1
	for j in PLAYERS:
		if j != i and _in_play(j) and snaps[j].get("state", "") == "Attack" and int(snaps[j].get("hitlag", 0)) > 0:
			by = j
	if by < 0 and i < stats.size():
		by = stats[i].last_hitter
	var colour: Color = PLAYER_COLORS[by % PLAYER_COLORS.size()] if by >= 0 else Color(1.0, 0.85, 0.4)
	_rumble(by, 0.2 + 0.3 * strength, 0.4 * strength, maxf(0.06, float(lag) / 90.0))
	# Where the hit met the fighter: toward the hitter's live hitbox from the middle of the body, no further than its edge. What landed
	# it (a blade, a blow, fire, claws) picks the effect and the sound.
	var middle := Vector2(cur_pos[i].x, cur_pos[i].y + 1.1)
	var contact := middle
	var kind: int = Effects.Kind.BLOW
	if by >= 0:
		var cls := int(snaps[by].get("class", 1))
		var move := str(snaps[by].get("move_name", ""))
		var tip: Vector3 = snaps[by].get("move_tip", Vector3.ZERO)
		var facing := float(snaps[by].get("facing", 1))
		if tip != Vector3.ZERO:
			var tip_at := Vector2(cur_pos[by].x + tip.x * facing, cur_pos[by].y + tip.y)
			# (A little toward the hitter, so even a big burst leaves the struck fighter's face in view.)
			contact = middle + (tip_at - middle).limit_length(1.0)
		if FighterView.bladed(cls) and _weapon_move(move) and not (cls == FighterView.GREATSWORD_CLASS and move in ["jab", "jab 2", "dtilt"]):
			kind = Effects.Kind.BLADE
		elif cls == 1 and FighterView.BRAWLER_FLAME.has(move):
			kind = Effects.Kind.FIRE
		elif cls == 1 and _weapon_move(move):
			kind = Effects.Kind.CLAW
	var at := Vector3(contact.x, contact.y, 0.8)
	# The effect is stretched the way the hit sends the fighter: away from whoever hit it, and up.
	var dir := Vector2(1.0 if by < 0 or cur_pos[by].x <= cur_pos[i].x else -1.0, 0.55).normalized()
	Effects.hit(self, at, dir, colour, strength, kind)
	# A heavy hit on a fighter standing on the stage scuffs the floor under it.
	if strength >= SCUFF_FROM and int(snaps[i].get("platform", -1)) >= 0:
		_scuff(cur_pos[i], dir.x)
	# The shake swings along the line the hit sends the fighter, so a heavy blow reads as a push in that direction.
	if float(lag) > HIT_SHAKE_FROM:
		shake_dir = dir
		shake_phase = 0.0
	# The three tiers of hit: a light one is a small spark; a medium one adds a short dart the way it sends the fighter; a strong one a
	# bigger burst, a ring and a long spear (scripts/effects.gd).
	if strength > 0.25:
		Effects.impact(self, at, dir, colour, strength)
	# The hit's sound, by what landed it: a blade, a blow or the maul (fire sounds like a blow).
	sfx.hit(0 if kind == Effects.Kind.BLADE or kind == Effects.Kind.CLAW else 1, strength)
	# A heavy hit punches the camera in for a moment and flashes the screen.
	if strength > 0.55:
		cam_punch = maxf(cam_punch, 0.6 + 0.6 * strength)
		_screen_flash(0.18 + 0.2 * strength)
	if not replay_mode and sim.fighter_will_ko(i, 200):
		ko_focus = i
		ko_by = by
		# The finishing hit: two frames in two tones. The one that decides the match holds the close-up longer, in slow motion.
		if VisualSettings.two_tone():
			impact_until = int(sim.frame()) + 2
		var deciding := _deciding(i, 1)
		ko_time = 1.3 if deciding else 0.75
		cam_shake = maxf(cam_shake, 0.9)
		if deciding and _local_match() and demo == null:
			Engine.time_scale = 0.25
			slowmo_until = Time.get_ticks_msec() + 650


## Whether a move of a held weapon's class is swung with the weapon (grabs, throws and pummels are hands, and sound like blows).
static func _weapon_move(name: String) -> bool:
	return not (name.contains("grab") or name.ends_with("throw") or name == "pummel")


## A hit on a raised shield: a ring of light on the bubble, the camera nudged a little.
func _on_block(i: int) -> void:
	var lag: int = snaps[i].hitlag
	_rumble(i, 0.3, 0.1, 0.07)
	cam_shake = maxf(cam_shake, minf(0.35, float(lag) * 0.015))
	Effects.shield_hit(self, Vector3(cur_pos[i].x, cur_pos[i].y + 1.1, 0.9), clampf((float(lag) - 4.0) / 12.0, 0.0, 1.0))
	sfx.play_group("glass", 1.0, -2.0)
	sfx.play("shield", 1.4, -8.0)


## A white flash over the screen that fades at once (a heavy hit), at most `alpha`.
var flash_rect: ColorRect
var flash_alpha := 0.0


func _screen_flash(alpha: float) -> void:
	if flash_rect == null:
		var layer := CanvasLayer.new()
		layer.layer = 5
		add_child(layer)
		flash_rect = ColorRect.new()
		flash_rect.set_anchors_preset(Control.PRESET_FULL_RECT)
		flash_rect.mouse_filter = Control.MOUSE_FILTER_IGNORE
		flash_rect.color = Color(1, 1, 1, 0)
		layer.add_child(flash_rect)
	flash_alpha = maxf(flash_alpha, alpha * VisualSettings.flashes())


## Keeps the match statistics up to date from two snapshots of every fighter. Who hit whom is read from hitlag starting together: the
## victim's launch and an attacker frozen in its attack on the same frame (with two fighters, it can only be the other one).
func _track_stats(befores: Array) -> void:
	var frame: int = sim.match_frame()
	if stats.size() != PLAYERS or frame < last_match_frame:
		stats = []
		for i in PLAYERS:
			stats.append({"kos": 0, "falls": 0, "dealt": 0.0, "taken": 0.0, "last_hitter": -1})
	last_match_frame = frame
	for i in PLAYERS:
		var before: Dictionary = befores[i]
		var now: Dictionary = snaps[i]
		if before.is_empty():
			continue
		if int(before.get("hitlag", 0)) == 0 and int(now.hitlag) > 0 and now.launch_pending:
			var hitter := -1
			for j in PLAYERS:
				if j != i and snaps[j].get("state", "") == "Attack" and int(snaps[j].get("hitlag", 0)) > 0 and int(befores[j].get("hitlag", 0)) == 0:
					hitter = j
			if hitter < 0 and PLAYERS == 2:
				hitter = 1 - i
			stats[i].last_hitter = hitter
		var gained: float = float(now.get("percent", 0.0)) - float(before.get("percent", 0.0))
		if gained > 0.0:
			stats[i].taken += gained
			var by: int = stats[i].last_hitter
			if by >= 0:
				stats[by].dealt += gained
		var ko: bool = _knocked_out(before, now)
		if ko:
			stats[i].falls += 1
			var by: int = stats[i].last_hitter
			if by >= 0 and by != i:
				stats[by].kos += 1
			stats[i].last_hitter = -1


## Whether the fighter was knocked out between two snapshots: it lost a stock, or (in free play, where no stock is lost) it has just
## come back on the revival platform. Not a jump in invincibility: ledge options and get-ups grant some too.
func _knocked_out(before: Dictionary, now: Dictionary) -> bool:
	return int(now.get("stocks", 0)) < int(before.get("stocks", 0)) or (now.get("state", "") == "Respawn" and before.get("state", "") != "Respawn")


## Cues for things the sim reports between two snapshots of a fighter: a clank, a wall tech, a knock-out.
func _on_events(i: int, before: Dictionary, now: Dictionary) -> void:
	if before.is_empty():
		return
	if now.state == "ShieldBreak" and before.state != "ShieldBreak":
		_rumble(i, 0.8, 1.0, 0.6)
	if now.state == "Rebound" and before.state != "Rebound":
		sfx.play_group("metal_heavy", 1.1, 0.0)
		cam_shake = maxf(cam_shake, 0.25)
		_burst(Vector3(cur_pos[i].x + 0.9 * float(now.facing), cur_pos[i].y + 1.1, 0.6), Color(1.0, 0.95, 0.6), 0.9, 0.3)
	# Starting a fast fall: a little star flashes beside the fighter (as the reference game marks it).
	if now.get("fast_fall", false) and not before.get("fast_fall", false) and int(now.get("platform", 0)) < 0:
		Effects.sparkle(self, Vector3(cur_pos[i].x + 0.7, cur_pos[i].y + 1.9, 0.8))
		sfx.play("whoosh", 1.6)
	# A ring at the feet on every jump, brighter for a midair one.
	if now.state == "Airborne" and before.state == "JumpSquat":
		Effects.jump_ring(self, Vector3(cur_pos[i].x, cur_pos[i].y, 0.0), false)
	elif now.state == "Airborne" and before.state == "Airborne" and int(now.get("jumps", 0)) < int(before.get("jumps", 0)):
		Effects.jump_ring(self, Vector3(cur_pos[i].x, cur_pos[i].y, 0.0), true)
	if now.state == "WallTech" and before.state != "WallTech":
		sfx.play("land", 1.3)
		_burst(Vector3(cur_pos[i].x, cur_pos[i].y + 1.0, 0.6), Color(0.85, 0.95, 1.0), 1.1, 0.35)
	if _knocked_out(before, now):
		_ko_blast(i, before.get("pos", cur_pos[i]))


## A flash of light that grows and fades (clanks, techs).
func _burst(at: Vector3, color: Color, size: float, life: float) -> void:
	var m := MeshInstance3D.new()
	var mesh := SphereMesh.new()
	mesh.radius = 0.5
	mesh.height = 1.0
	m.mesh = mesh
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.blend_mode = BaseMaterial3D.BLEND_MODE_ADD
	mat.albedo_color = color
	m.material_override = mat
	m.position = at
	m.scale = Vector3.ONE * size * 0.3
	add_child(m)
	effects.append([m, 0.0, life, size])


## Whether knocking out fighter `i`, which has `stocks` left, ends the match: its last stock, with one other fighter still in it.
func _deciding(i: int, stocks: int) -> bool:
	if int(snaps[i].get("stocks", 0)) != stocks:
		return false
	var others := 0
	for j in PLAYERS:
		if j != i and _in_play(j) and int(snaps[j].get("stocks", 0)) > 0:
			others += 1
	return others == 1


## A knock-out: a beam of the player's colour bursts from where the fighter left the stage with a shockwave and a flash, the camera
## shakes, and (offline) the match slows for a moment; longer, with a brighter flash, for the knock-out that ends the match.
func _ko_blast(i: int, at: Vector2) -> void:
	var deciding := _deciding(i, 0)
	_rumble(i, 1.0, 1.0, 0.5 if deciding else 0.35)
	var color: Color = PLAYER_COLORS[i % PLAYER_COLORS.size()]
	# The beam, pointed back toward the middle of the stage.
	var inward := (Vector2(0, 4) - at).normalized()
	var beam := Effects.Beam.new()
	beam.material_override = Effects._material(false)
	beam.colour = color
	beam.width = 3.2 if deciding else 2.6
	beam.position = Vector3(at.x, at.y + 1.0, 0.0)
	beam.rotation.z = atan2(inward.y, inward.x) - PI / 2.0
	add_child(beam)
	# Sparks, confetti stars and smoke burst back toward the stage with the beam.
	# (A little way in from the blast line, so the burst is on screen.)
	var burst_at := at + inward * 13.0
	Particles.knock_out(self, Vector3(burst_at.x, burst_at.y + 1.0, 0.5), color, inward)
	for k in 2:
		var wave := Effects.Ring.new()
		wave.material_override = Effects._material(true)
		wave.colour = Color(1, 1, 1, 0.95) if k == 0 else Color(color.r, color.g, color.b, 0.9)
		wave.from = 0.5 + k * 0.6
		wave.to = (7.0 if deciding else 5.0) - k * 1.5
		wave.thickness = 0.7 - k * 0.3
		wave.life = 0.45 + k * 0.1
		wave.position = Vector3(burst_at.x, burst_at.y + 1.0, 0.6)
		add_child(wave)
	_screen_flash(0.55 if deciding else 0.3)
	cam_shake = maxf(cam_shake, 1.8 if deciding else 1.4)
	if _local_match() and demo == null:
		Engine.time_scale = 0.25 if deciding else 0.35
		slowmo_until = Time.get_ticks_msec() + (1000 if deciding else 450)


## Grows and fades the effects, and ends the knock-out slow motion.
func _update_effects(delta: float) -> void:
	if slowmo_until > 0 and Time.get_ticks_msec() >= slowmo_until:
		slowmo_until = 0
		Engine.time_scale = 1.0
	var keep: Array = []
	for e in effects:
		var node: Node3D = e[0]
		e[1] += delta
		var t: float = e[1] / e[2]
		if t >= 1.0 or not is_instance_valid(node):
			if is_instance_valid(node):
				node.queue_free()
			continue
		if node is MeshInstance3D:
			node.scale = Vector3.ONE * float(e[3]) * (0.3 + 1.2 * t)
			(node.material_override as StandardMaterial3D).albedo_color.a = 1.0 - t
		else:
			node.scale = Vector3(1.0 + t * 0.6, 1.0, 1.0 + t * 0.6)
			for c in node.get_children():
				var m: StandardMaterial3D = (c as MeshInstance3D).material_override
				m.albedo_color.a = 0.9 * (1.0 - t)
		keep.append(e)
	effects = keep


## Fighters outside the camera's view: where to draw their marker on the screen edge, in their colour, with their percent.
func _offscreen_markers() -> Array:
	var out: Array = []
	if cam == null:
		return out
	var view := get_viewport().get_visible_rect().size
	var margin := 70.0
	for i in PLAYERS:
		if not _in_play(i) or not sim.fighter_active(i):
			continue
		var p: Vector2 = cur_pos[i]
		var world := Vector3(p.x, p.y + 1.1, 0)
		if cam.is_position_behind(world):
			continue
		var s := cam.unproject_position(world)
		if s.x >= 0 and s.y >= 0 and s.x <= view.x and s.y <= view.y:
			continue
		var at := Vector2(clampf(s.x, margin, view.x - margin), clampf(s.y, margin, view.y - margin))
		var dir := (s - at).normalized()
		out.append({"at": at, "dir": dir, "color": PLAYER_COLORS[i % PLAYER_COLORS.size()], "label": "P%d" % (i + 1), "percent": int(snaps[i].get("percent", 0.0))})
	return out


## Degrees the match camera looks down at the stage.
const CAMERA_PITCH := 10.0
## The nearest the match camera comes (world units). At 26 it shows about 25 units across, the whole main stage (22, the reference
## game's stage length) with a little to spare, and a fighter (2.2 tall) is about a sixth of the screen's height, as the reference
## game frames a close exchange. It came in to 19 before, where a fighter filled over a fifth of the screen and the stage did not fit,
## so every speed (which follow the reference's published numbers) read faster than the reference's.
const CAMERA_NEAREST := 26.0
## The nearest the knock-out close-up comes (it never comes in past the match framing it starts from).
const KO_NEAREST := 21.0


## The steady camera (the Display setting, on by default): a fighter in the air counts at the height it last stood on until it is more
## than this far above it (world units: above any fighter's full hop, the tallest 5.2), and only by what it rises past that; below
## it (off an edge, or dropping through a platform) it counts where it is. So ordinary jumps leave the view still, as the reference's
## camera does, and launches, double jumps up high and recoveries still move it.
const CAM_JUMP_BAND := 5.5
## The height each fighter last stood on, for the steady camera.
var cam_ground: Array[float] = [NAN, NAN, NAN, NAN]


## Where the camera frames fighter `i` this frame (`p`, its drawn position): itself, or with the steady camera its height held while
## it is in an ordinary jump.
func _camera_point(i: int, p: Vector2) -> Vector2:
	if int(snaps[i].get("platform", -1)) >= 0 or is_nan(cam_ground[i]):
		cam_ground[i] = cur_pos[i].y
	if not VisualSettings.steady_camera():
		return p
	var g: float = cam_ground[i]
	if p.y < g:
		return p
	return Vector2(p.x, g + maxf(0.0, p.y - g - CAM_JUMP_BAND))


func _update_camera(a: float, delta: float) -> void:
	var lo := Vector2(1e9, 1e9)
	var hi := Vector2(-1e9, -1e9)
	for i in PLAYERS:
		if not _in_play(i):
			continue
		var p: Vector2 = _camera_point(i, prev_pos[i].lerp(cur_pos[i], a))
		lo = lo.min(p)
		hi = hi.max(p + Vector2(0, 2.2))
	if lo.x > hi.x:
		return
	var center := (lo + hi) / 2.0
	# Fit both fighters (plus margin) in view: visible width at distance d is about d * 0.95 at 30 deg fov, 16:9.
	# (Close together, the camera comes in, but never nearer than CAMERA_NEAREST: see there.)
	var spread := maxf(hi.x - lo.x + 14.0, (hi.y - lo.y + 8.0) * 1.78)
	var dist := clampf(spread / 0.95, CAMERA_NEAREST, 85.0)
	if demo != null and demo.cam_dist > 0.0:
		dist = demo.cam_dist
	# (Aimed a little low, by a share of the view's height, so fighters on the ground sit clear of the match display along the bottom.)
	var target := Vector3(clampf(center.x, -12, 12), clampf(center.y, -3, 10) + 1.6 - dist * HUD_LIFT, dist)
	var rate := 3.5
	if demo != null and demo.cam_focus >= 0:
		var f: Vector2 = prev_pos[demo.cam_focus].lerp(cur_pos[demo.cam_focus], a)
		target = Vector3(f.x, f.y + 1.1, dist)
		cam_base = target
	if cam_base == Vector3.INF:
		cam_base = cam.position
	if ko_time > 0.0 and ko_focus >= 0:
		# The knock-out close-up: in on the finishing hit, but framing the one who landed it as well as the one sent flying (it framed
		# only the one sent flying, at 17, and the winner of the exchange lost sight of themselves: playtest 2026-10-19).
		ko_time -= delta
		var v: Vector2 = prev_pos[ko_focus].lerp(cur_pos[ko_focus], a)
		var lo2 := v
		var hi2 := v + Vector2(0, 2.2)
		if ko_by >= 0 and ko_by != ko_focus and _in_play(ko_by):
			var w: Vector2 = prev_pos[ko_by].lerp(cur_pos[ko_by], a)
			lo2 = lo2.min(w)
			hi2 = hi2.max(w + Vector2(0, 2.2))
		var mid := (lo2 + hi2) / 2.0
		var fit := maxf(hi2.x - lo2.x + 9.0, (hi2.y - lo2.y + 6.0) * 1.78) / 0.95
		target = Vector3(mid.x, mid.y + 0.4, clampf(fit, KO_NEAREST, dist))
		rate = 6.0
	cam_base = cam_base.lerp(target, clampf(delta * rate, 0.0, 1.0))
	# A heavy hit's punch-in: the camera jumps closer and eases back.
	var punch := Vector3(0, 0, -cam_base.z * 0.12 * cam_punch)
	cam_punch = move_toward(cam_punch, 0.0, delta * 4.0)
	if flash_rect != null:
		flash_alpha = move_toward(flash_alpha, 0.0, delta * 3.5)
		flash_rect.color = Color(1, 1, 1, flash_alpha)
	var offset := Vector3.ZERO
	var roll := 0.0
	if cam_shake > 0.001:
		# A damped swing along the hit's line with a smaller one across it and a slight roll (not a fresh random jolt every frame, which
		# read as jitter). A shake with no direction of its own picks one.
		if shake_dir == Vector2.ZERO:
			shake_dir = Vector2.from_angle(shake_rng.randf() * TAU)
		shake_phase += delta * SHAKE_HZ * TAU
		var along := sin(shake_phase) * cam_shake
		var across := sin(shake_phase * 1.7 + 1.3) * cam_shake * 0.35
		offset = Vector3(shake_dir.x * along - shake_dir.y * across, shake_dir.y * along + shake_dir.x * across, 0.0)
		roll = sin(shake_phase * 0.9 + 0.5) * cam_shake * SHAKE_ROLL
		cam_shake = move_toward(cam_shake, 0.0, delta * 2.6)
	else:
		shake_dir = Vector2.ZERO
		shake_phase = 0.0
	# The camera looks slightly down at the stage (raised by as much as it tilts, so the point it frames stays put): the tops of the
	# platforms and the fighters' shadows on them show, and the stage has depth.
	var lift := cam_base.z * tan(deg_to_rad(CAMERA_PITCH))
	# (The comfort setting scales the shake and the punch-in; scripts/visual_settings.gd.)
	var calm := VisualSettings.shake()
	cam.position = cam_base + (offset + punch) * calm + Vector3(0, lift, 0)
	cam.rotation = Vector3(-deg_to_rad(CAMERA_PITCH), 0, roll * calm)
	_update_xray()
	# The distance haze starts at the stage plane (shaders/post.gdshader).
	if post_mat != null and absf(cam.position.z - post_stage_dist) > 0.25:
		post_stage_dist = cam.position.z
		post_mat.set_shader_parameter("stage_dist", post_stage_dist)


## Scuffs left on the stage by heavy hits (presentation only): a few decals reused in turn, cleared when a match starts, none on the
## Fast tier. `SCUFF_FROM` is the least hit strength (0..1, from its hitlag) that leaves one.
const SCUFF_FROM := 0.6
const SCUFFS := 6
var scuffs: Array = []
var scuff_next := 0


func _scuff(at: Vector2, side: float) -> void:
	if VisualSettings.get_index("quality") == 0:
		return
	if scuffs.size() < SCUFFS:
		var d := Decal.new()
		d.texture_albedo = SvgArt.texture("res://art/stage/scuff.svg")
		# Thin, and faded toward its top, so it marks the floor and not the feet on it.
		d.size = Vector3(1.8, 0.24, 0.9)
		d.upper_fade = 1.0
		d.lower_fade = 0.3
		add_child(d)
		scuffs.append(d)
	var s: Decal = scuffs[scuff_next % scuffs.size()] if scuffs.size() >= SCUFFS else scuffs[scuffs.size() - 1]
	scuff_next += 1
	s.position = Vector3(at.x + 0.3 * side, at.y + 0.02, 0.1)
	s.rotation = Vector3(0, 0, 0)
	s.scale = Vector3(-1.0 if side < 0.0 else 1.0, 1.0, 1.0)
	s.visible = true


func _clear_scuffs() -> void:
	for s in scuffs:
		s.visible = false
	scuff_next = 0


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
		# (Hidden until `_update_ecb` shows it: with `--noecb` that never runs, and the outlines would stand at the origin in the shots.)
		mi.visible = false
		add_child(mi)
		ecb_nodes.append(mi)
		min_down[i] = body[3]


func _update_ecb(a: float) -> void:
	for i in PLAYERS:
		ecb_nodes[i].visible = show_ecb and _in_play(i)
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
	# Every kind of effect once, in front of the camera behind the cover: each hit kind, the impact, a block, the fast-fall star, jump
	# rings, dust, sparks, embers, a knock-out burst and the flame; then a small card for every effect material made, the trails' included.
	var warm := Node3D.new()
	add_child(warm)
	var at := Vector3(cam.position.x, cam.position.y - 2.0, 0.0)
	for kind in [Effects.Kind.BLADE, Effects.Kind.BLOW, Effects.Kind.FIRE]:
		Effects.hit(warm, at, Vector2(1, 0.5).normalized(), Color.WHITE, 1.0, kind)
	Effects.impact(warm, at, Vector2.RIGHT, Color.WHITE, 1.0)
	Effects.shield_hit(warm, at, 1.0)
	Effects.sparkle(warm, at)
	Effects.jump_ring(warm, at, true)
	Effects.jump_ring(warm, at, false)
	Particles.dust(warm, at, 1.0)
	Particles.knock_out(warm, at, Color.WHITE, Vector2.UP)
	var fire := Particles.flame(warm)
	fire.emitting = true
	warm.add_child(fire)
	for on_top in [true, false]:
		FxMaterial.get_material(on_top, 5)
	for m in FxMaterial.all():
		var card := MeshInstance3D.new()
		var quad := QuadMesh.new()
		quad.size = Vector2(0.5, 0.5)
		card.mesh = quad
		card.material_override = m
		card.position = at
		warm.add_child(card)
	for _i in 6:
		FxMaterial.tick(float(_i) / 60.0)
		await get_tree().process_frame
	warm.queue_free()
	FxMaterial.stop_clock()
	for v in views:
		v.prewarm(false)
	overlay.prewarm_text(false)
	layer.queue_free()
	warmed = true


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	if replay_mode:
		_replay_key(event)
		return
	if group_mode and group_status == 0 and sim.group_slot() == 0 and (event.keycode == KEY_ENTER or event.keycode == KEY_KP_ENTER) and results == null:
		sim.group_start_match()
		return
	if results != null and event.keycode != KEY_ESCAPE:
		results.handle_key(event)
		return
	# Training keys change the sim directly, which a networked match must never do.
	if (net_mode or group_mode or spectate_mode) and event.keycode in [KEY_F5, KEY_F6, KEY_F7, KEY_F8, KEY_P, KEY_PERIOD, KEY_COMMA, KEY_R]:
		return
	match event.keycode:
		KEY_F4:
			Music.of(self).set_muted(sfx.toggle_mute())
		KEY_F5:
			# Player 2 played by the computer, or by a person again.
			cpu_levels[1] = 0 if cpu_levels[1] > 0 else CPU_LEVEL
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
			if Roster.session.get("from_menu", false):
				_leave_to("online" if (net_mode or spectate_mode or group_mode or quick_mode) else "select")
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
	var circle := _unit_circle()
	for i in PLAYERS:
		var mi := MultiMeshInstance3D.new()
		var mm := MultiMesh.new()
		mm.transform_format = MultiMesh.TRANSFORM_3D
		mm.use_colors = true
		mm.mesh = circle
		mm.instance_count = MAX_BOXES
		mm.visible_instance_count = 0
		mi.multimesh = mm
		mi.material_override = box_mat
		add_child(mi)
		box_nodes.append(mi)


const MAX_BOXES := 32


## A ring of radius 1 drawn in lines: every box is one of these, placed, sized and coloured.
static func _unit_circle() -> ArrayMesh:
	var verts := PackedVector3Array()
	var segments := 20
	for k in segments:
		var a := Vector2.from_angle(TAU * k / segments)
		var b := Vector2.from_angle(TAU * (k + 1) / segments)
		verts.append(Vector3(a.x, a.y, 0))
		verts.append(Vector3(b.x, b.y, 0))
	var arrays := []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = verts
	var mesh := ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_LINES, arrays)
	return mesh


## Each box is an instance of one ring mesh (`MultiMesh`), moved and recoloured each tick: making and freeing a mesh per tick instead
## cost milliseconds a frame on the GPU driver.
func _rebuild_boxes() -> void:
	if box_nodes.is_empty():
		return
	for i in PLAYERS:
		var mm: MultiMesh = box_nodes[i].multimesh
		if not show_boxes:
			mm.visible_instance_count = 0
			continue
		var origin: Vector2 = cur_pos[i]
		var n := 0
		var hurt: PackedFloat32Array = sim.fighter_hurtboxes(i)
		for k in range(0, hurt.size(), 3):
			n = _box(mm, n, Vector2(hurt[k], hurt[k + 1]) - origin, hurt[k + 2], Color(0.3, 1.0, 0.4, 0.9))
		var hits: PackedFloat32Array = sim.fighter_hitboxes(i)
		for k in range(0, hits.size(), 4):
			var sweet: bool = hits[k + 3] < 0.5
			n = _box(mm, n, Vector2(hits[k], hits[k + 1]) - origin, hits[k + 2], Color(1.0, 0.2, 0.2) if sweet else Color(1.0, 0.6, 0.15))
		mm.visible_instance_count = n


func _box(mm: MultiMesh, n: int, c: Vector2, r: float, color: Color) -> int:
	if n >= MAX_BOXES:
		return n
	mm.set_instance_transform(n, Transform3D(Basis.from_scale(Vector3(r, r, 1.0)), Vector3(c.x, c.y, 1.5)))
	mm.set_instance_color(n, color)
	return n + 1


func _position_boxes(a: float) -> void:
	for i in PLAYERS:
		var p: Vector2 = prev_pos[i].lerp(cur_pos[i], a)
		box_nodes[i].position = Vector3(p.x, p.y, 0)


# ---- Items (sim 111) --------------------------------------------------------------------------------
# A placeholder look for every item slot until the modelled items come (the owner's schedule puts models and animation near the end):
# the Wanderer's bomb (kind 2) is a dark navy ball with a fuse and a spark that flashes red for its last 45 frames, and bursts in a painted
# fire burst; its arrow (kind 3) is a shaft with a steel head and fletching. A held item sits in the holder's hand side. Read only from
# the bridge's `item_*` (presentation only).

const ITEM_BOMB := 2
const ITEM_ARROW := 3
## The bomb's fuse flashes over its last frames.
const ITEM_FLASH := 45
var item_nodes: Array = []
var item_burst_shown: Array = []
## Per slot: which way the item last moved (x sign), so a lodged arrow keeps pointing the way it flew.
var item_last_dir: Array = []
## A lodged arrow's tilt (degrees below level) when the sim gives no angle (`item_angle`, sim 113, is 0 for one it has none for).
const ITEM_LODGED_TILT := 35.0
## How fast a thrown bomb turns (degrees a frame since it was thrown).
const ITEM_BOMB_SPIN := 18.0


func _build_items() -> void:
	if not sim.has_method("item_count"):
		return
	for k in int(sim.item_count()):
		var n := Node3D.new()
		n.visible = false
		add_child(n)
		# (The bomb's node at the ball's middle, so it spins about it.)
		var bomb := Node3D.new()
		bomb.name = "bomb"
		bomb.position = Vector3(0, 0.36, 0)
		n.add_child(bomb)
		var ball := MeshInstance3D.new()
		var sm := SphereMesh.new()
		sm.radius = 0.36
		sm.height = 0.72
		ball.mesh = sm
		ball.material_override = FighterView.toon(Color(0.2, 0.22, 0.36))
		ball.position = Vector3.ZERO
		bomb.add_child(ball)
		var cap := MeshInstance3D.new()
		var cm := CylinderMesh.new()
		cm.top_radius = 0.1
		cm.bottom_radius = 0.12
		cm.height = 0.12
		cap.mesh = cm
		cap.material_override = FighterView.toon(Color(0.7, 0.72, 0.8))
		cap.position = Vector3(0, 0.38, 0)
		bomb.add_child(cap)
		var fuse := MeshInstance3D.new()
		var fm := CylinderMesh.new()
		fm.top_radius = 0.025
		fm.bottom_radius = 0.03
		fm.height = 0.22
		fuse.mesh = fm
		fuse.material_override = FighterView.toon(Color(0.86, 0.78, 0.6), false)
		fuse.position = Vector3(0.04, 0.52, 0)
		fuse.rotation_degrees = Vector3(0, 0, -20)
		bomb.add_child(fuse)
		var spark := MeshInstance3D.new()
		var spm := SphereMesh.new()
		spm.radius = 0.08
		spm.height = 0.16
		spark.mesh = spm
		var spark_mat := StandardMaterial3D.new()
		spark_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		spark_mat.albedo_color = Color(1.0, 0.85, 0.3)
		spark.material_override = spark_mat
		spark.position = Vector3(0.08, 0.64, 0)
		spark.name = "spark"
		bomb.add_child(spark)
		var arrow := Node3D.new()
		arrow.name = "arrow"
		n.add_child(arrow)
		var shaft := MeshInstance3D.new()
		var bm := BoxMesh.new()
		bm.size = Vector3(1.1, 0.06, 0.06)
		shaft.mesh = bm
		shaft.material_override = FighterView.toon(Color(0.66, 0.46, 0.3))
		shaft.position = Vector3(0, 0.15, 0)
		arrow.add_child(shaft)
		var head := MeshInstance3D.new()
		var hm := CylinderMesh.new()
		hm.top_radius = 0.0
		hm.bottom_radius = 0.1
		hm.height = 0.22
		head.mesh = hm
		head.material_override = FighterView.respond(FighterView.toon(Color(0.74, 0.76, 0.84)), "metal")
		head.position = Vector3(0.64, 0.15, 0)
		head.rotation_degrees = Vector3(0, 0, -90)
		arrow.add_child(head)
		for side in [-1.0, 1.0]:
			var fl := MeshInstance3D.new()
			var flm := BoxMesh.new()
			flm.size = Vector3(0.22, 0.12, 0.02)
			fl.mesh = flm
			fl.material_override = FighterView.toon(Color(0.92, 0.4, 0.36))
			fl.position = Vector3(-0.48, 0.15 + side * 0.06, 0)
			fl.rotation_degrees = Vector3(0, 0, side * 20.0)
			arrow.add_child(fl)
		_as_play_object(n)
		item_nodes.append(n)
		item_burst_shown.append(false)
		item_last_dir.append(1.0)


func _update_items() -> void:
	for k in item_nodes.size():
		var n: Node3D = item_nodes[k]
		var kind := int(sim.item_kind(k))
		var rest := int(sim.item_rest(k))
		n.visible = kind != 0 and rest != 4
		var at := Vector3(sim.item_pos(k).x, sim.item_pos(k).y, 0.4)
		if rest == 4:
			# Bursting: a painted fire burst, once.
			if not item_burst_shown[k] and kind == ITEM_BOMB:
				Effects.hit(self, at + Vector3(0, 0.4, 0), Vector2.UP, Color(1.0, 0.5, 0.1), 1.0, Effects.Kind.FIRE)
				item_burst_shown[k] = true
			continue
		item_burst_shown[k] = false
		if not n.visible:
			continue
		var holder := int(sim.item_holder(k))
		var facing := 1.0
		if holder >= 0 and holder < snaps.size():
			facing = float(snaps[holder].facing)
			var hp: Vector2 = cur_pos[holder]
			# (Held in the hand on the facing side, at about hip height.)
			at = Vector3(hp.x + 0.55 * facing, hp.y + 0.7, 0.6)
		n.position = at
		var bomb: Node3D = n.get_node("bomb")
		var arrow: Node3D = n.get_node("arrow")
		bomb.visible = kind == ITEM_BOMB
		arrow.visible = kind == ITEM_ARROW
		var vel: Vector2 = sim.item_vel(k) if sim.has_method("item_vel") else Vector2.ZERO
		if absf(vel.x) > 0.001:
			item_last_dir[k] = signf(vel.x)
		if holder >= 0:
			item_last_dir[k] = facing
		var way: float = item_last_dir[k]
		# The arrow: held level in the hand; in flight along its velocity; lodged at the angle it flew in at (the sim's).
		arrow.scale = Vector3.ONE
		if holder >= 0 or (rest != 3 and vel.length() < 0.001):
			arrow.rotation.z = 0.0 if way > 0.0 else PI
		elif rest == 3:
			var lodged: int = int(sim.item_angle(k)) if sim.has_method("item_angle") else 0
			if lodged != 0:
				arrow.rotation.z = deg_to_rad(float(lodged))
			else:
				arrow.rotation.z = deg_to_rad(-ITEM_LODGED_TILT) if way > 0.0 else deg_to_rad(180.0 + ITEM_LODGED_TILT)
		else:
			arrow.rotation.z = vel.angle()
		# The bomb spins while it flies after a throw, the way it was thrown.
		var flight: int = int(sim.item_flight(k)) if sim.has_method("item_flight") else 0
		bomb.rotation.z = 0.0 if holder >= 0 or flight <= 0 or vel.length() < 0.001 else deg_to_rad(-ITEM_BOMB_SPIN * float(flight) * way)
		if kind == ITEM_BOMB:
			var left := int(sim.item_life_left(k))
			var flash := left >= 0 and left <= ITEM_FLASH and (int(sim.frame()) / 4) % 2 == 0
			var spark: MeshInstance3D = bomb.get_node("spark")
			(spark.material_override as StandardMaterial3D).albedo_color = Color(1.0, 0.25, 0.15) if flash else Color(1.0, 0.85, 0.3)
			spark.scale = Vector3.ONE * (1.8 if flash else 1.0)
			(bomb.get_child(0) as MeshInstance3D).set_instance_shader_parameter("flash", Color(1.0, 0.2, 0.1, 0.5 if flash else 0.0))


# ---- Projectiles ------------------------------------------------------------------------------------
# Each slot can show a bolt (the claws' blaster and anything else), a needle (the Needles moveset's Needle Volley: thin, teal, pointing
# where it flies) or the Needles' pulling line (a small weight with a cord back to the thrower's hand).

## Numbers per slot in `projectile_slots` (see the bridge).
const PROJECTILE_STRIDE := 9
## The Needles class (2) and the move slots its needles and line come from (ext4 = 32, side special = 14).
const NEEDLE_FROM := [2, 32]
const LINE_FROM := [2, 14]
## The Champion's (class 6) sword wave: its neutral special (13), and the powered wave (ext6 = 34, ext7 = 35), which is taller and turns
## into a spinning burst where it hits (`projectile_bursting`, when the bridge has it).
const WAVE_CLASS := 6
const WAVE_MOVES := [13, 34, 35]
const WAVE_POWERED := [34, 35]
## The Wanderer's (Buckler, class 9) sword wave: fired with its forward smash (5) at 0%, a horizontal blade of light about 1.75 long.
const BLADE_WAVE_FROM := [9, 5]
const BLADE_WAVE_LENGTH := 1.75
## The Bounder's (class 8) up special throw (15): drawn as a basketball (the owner's call), its radius about the shot's (3 reference
## units), spinning the way it flies, popping in a painted burst where it bursts.
const BALL_FROM := [8, 15]
const BALL_RADIUS := 0.42
var proj_balls: Array = []
var proj_ball_burst: Array = []

var proj_nodes: Array = []
var proj_needles: Array = []
var proj_lines: Array = []
## Per slot: the painted wave (an ImmediateMesh redrawn on twos) and its spin for a burst.
var proj_waves: Array = []
var proj_prev := PackedFloat32Array()
var proj_cur := PackedFloat32Array()


func _build_projectiles() -> void:
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.albedo_color = Color(0.78, 0.4, 1.0)
	var needle_mat := StandardMaterial3D.new()
	needle_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	needle_mat.albedo_color = Color(0.75, 1.0, 0.95)
	var line_mat := StandardMaterial3D.new()
	line_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	line_mat.albedo_color = Color(0.25, 0.85, 0.75)
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
		# A needle: a long thin box along x, turned to its flight.
		var nd := MeshInstance3D.new()
		var stick := BoxMesh.new()
		stick.size = Vector3(0.75, 0.05, 0.05)
		nd.mesh = stick
		nd.material_override = needle_mat
		nd.visible = false
		add_child(nd)
		proj_needles.append(nd)
		# The line: its weight, and the cord drawn fresh each frame.
		var ln := MeshInstance3D.new()
		var weight := SphereMesh.new()
		weight.radius = 0.16
		weight.height = 0.32
		ln.mesh = weight
		ln.material_override = line_mat
		ln.visible = false
		add_child(ln)
		var cord := MeshInstance3D.new()
		cord.mesh = ImmediateMesh.new()
		cord.material_override = line_mat
		cord.top_level = true
		cord.extra_cull_margin = 16384.0
		cord.visible = false
		add_child(cord)
		proj_lines.append([ln, cord])
		var wv := MeshInstance3D.new()
		wv.mesh = ImmediateMesh.new()
		wv.material_override = FxMaterial.get_material(true, 5, null, 1.0, 1.0)
		wv.extra_cull_margin = 64.0
		wv.visible = false
		add_child(wv)
		proj_waves.append(wv)
		proj_balls.append(_basketball())
		proj_ball_burst.append(false)


func _update_projectiles(a: float) -> void:
	var n := PROJECTILE_STRIDE
	for k in proj_nodes.size():
		var i := k * n
		var active: bool = proj_cur.size() > i + n - 1 and proj_cur[i] > 0.5
		var from := [int(proj_cur[i + 6]), int(proj_cur[i + 7])] if active else []
		var needle: bool = active and from == NEEDLE_FROM
		var line: bool = active and from == LINE_FROM
		var blade_wave: bool = active and int(from[0]) == BLADE_WAVE_FROM[0] and int(from[1]) == BLADE_WAVE_FROM[1]
		var wave: bool = active and (blade_wave or (int(from[0]) == WAVE_CLASS and WAVE_MOVES.has(int(from[1]))))
		var ball: bool = active and from == BALL_FROM
		var ball_bursting: bool = ball and sim.has_method("projectile_bursting") and bool(sim.projectile_bursting(k))
		proj_waves[k].visible = wave
		(proj_balls[k] as Node3D).visible = ball and not ball_bursting
		proj_nodes[k].visible = active and not needle and not line and not wave and not ball
		proj_needles[k].visible = needle
		proj_lines[k][0].visible = line
		proj_lines[k][1].visible = line
		if not ball:
			proj_ball_burst[k] = false
		if not active:
			continue
		var pos := Vector2(proj_cur[i + 1], proj_cur[i + 2])
		if proj_prev.size() > i + n - 1 and proj_prev[i] > 0.5:
			pos = Vector2(proj_prev[i + 1], proj_prev[i + 2]).lerp(pos, a)
		var at := Vector3(pos.x, pos.y, 0.3)
		if wave:
			var bursting: bool = sim.has_method("projectile_bursting") and bool(sim.projectile_bursting(k))
			if blade_wave:
				var scale_pct: float = float(sim.projectile_scale(k)) if sim.has_method("projectile_scale") else 100.0
				_draw_blade_wave(proj_waves[k], at, float(proj_cur[i + 3]), clampf(scale_pct / 100.0, 0.5, 1.5) if scale_pct > 0.0 else 1.0)
			else:
				_draw_wave(proj_waves[k], at, float(proj_cur[i + 3]), WAVE_POWERED.has(int(from[1])), bursting)
		elif ball:
			var b: Node3D = proj_balls[k]
			b.position = at
			var vx: float = proj_cur[i + 4]
			b.rotation.z -= vx * 0.9 / BALL_RADIUS * 0.016 * 60.0 * get_process_delta_time() if absf(vx) > 0.0 else 0.0
			if ball_bursting and not proj_ball_burst[k]:
				Effects.hit(self, at, Vector2.UP, Color(0.95, 0.5, 0.18), 0.8, Effects.Kind.BLOW)
			proj_ball_burst[k] = ball_bursting
		elif needle:
			proj_needles[k].position = at
			proj_needles[k].rotation.z = Vector2(proj_cur[i + 4], proj_cur[i + 5]).angle()
		elif line:
			proj_lines[k][0].position = at
			_draw_cord(proj_lines[k][1], int(proj_cur[i + 8]), at, a)
		else:
			proj_nodes[k].position = at


## A basketball `BALL_RADIUS` round (the Bounder's thrown ball): orange rubber, inked, its dark seams the two great circles and two
## curved side seams. Hidden until a slot shows it.
func _basketball() -> Node3D:
	var n := Node3D.new()
	n.visible = false
	add_child(n)
	var ball := MeshInstance3D.new()
	var sm := SphereMesh.new()
	sm.radius = BALL_RADIUS
	sm.height = BALL_RADIUS * 2.0
	sm.radial_segments = 28
	sm.rings = 16
	ball.mesh = sm
	ball.material_override = FighterView.respond(FighterView.toon(Color(0.95, 0.5, 0.18)), "rubber")
	n.add_child(ball)
	var seam := FighterView.toon(Color(0.18, 0.1, 0.08), false)
	for sd in [[1.0, Vector3.ZERO, Vector3(90, 0, 0)], [1.0, Vector3.ZERO, Vector3.ZERO],
			[0.72, Vector3(-0.68, 0, 0), Vector3(0, 0, 90)], [0.72, Vector3(0.68, 0, 0), Vector3(0, 0, 90)]]:
		var ring := MeshInstance3D.new()
		var tm := TorusMesh.new()
		tm.inner_radius = BALL_RADIUS * float(sd[0]) * 0.985
		tm.outer_radius = BALL_RADIUS * float(sd[0]) * 1.03
		tm.rings = 36
		tm.ring_segments = 5
		ring.mesh = tm
		ring.material_override = seam
		ring.position = (sd[1] as Vector3) * BALL_RADIUS
		ring.rotation_degrees = sd[2]
		n.add_child(ring)
	_as_play_object(n)
	return n


## Things in play (thrown balls, items) take an object id in the fighters' range, so the stage's quieting (`stage_quiet` in
## shaders/post.gdshader) leaves their colour alone and the ink outlines them like fighters.
const PLAY_OBJECT_ID := 0.58


func _as_play_object(n: Node) -> void:
	for g in n.find_children("*", "GeometryInstance3D", true, false):
		(g as GeometryInstance3D).set_instance_shader_parameter("object_id", PLAY_OBJECT_ID)


## The Champion's wave at `at` flying toward `dir` (+1 right): a painted crescent of amber light standing on its edge, its bright
## edge leading, layered like the hit bursts (an ink rim, amber, gold and a white-hot core along the front), its tips flickering on
## twos; `powered` makes it about 1.7 tall (else 1.1). Bursting, it is a spinning ring of three amber blades 1.8 round instead.
func _draw_wave(node: MeshInstance3D, at: Vector3, dir: float, powered: bool, bursting: bool) -> void:
	node.position = at
	node.scale = Vector3.ONE
	var im: ImmediateMesh = node.mesh
	var step := int(Time.get_ticks_msec() / 33)
	if node.has_meta("step") and int(node.get_meta("step")) == step and node.get_meta("burst", false) == bursting 			and node.get_meta("kind", "") == "wave":
		return
	node.set_meta("step", step)
	node.set_meta("burst", bursting)
	node.set_meta("kind", "wave")
	im.clear_surfaces()
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
	var layers := [[1.0, Color(0.3, 0.06, 0.02, 1.0)], [0.9, Color(1.0, 0.36, 0.02)], [0.66, Color(1.0, 0.66, 0.08)], [0.34, Color(1.0, 0.95, 0.7)]]
	if bursting:
		var spin := float(step) * 0.55
		for layer in layers:
			var r := 1.8 * float(layer[0])
			for b in 3:
				var a0 := spin + TAU * b / 3.0
				for j in 6:
					var t0 := float(j) / 6.0
					var t1 := float(j + 1) / 6.0
					var w0 := 0.42 * r * (1.0 - t0)
					var w1 := 0.42 * r * (1.0 - t1)
					var p0 := Vector3(cos(a0 + t0 * 1.6), sin(a0 + t0 * 1.6), 0) * r
					var p1 := Vector3(cos(a0 + t1 * 1.6), sin(a0 + t1 * 1.6), 0) * r
					var q0 := p0 * (1.0 - w0 / r)
					var q1 := p1 * (1.0 - w1 / r)
					for v in [p0, p1, q1, p0, q1, q0]:
						im.surface_set_color(layer[1])
						im.surface_add_vertex(v)
	else:
		var h := 1.7 if powered else 1.1
		var n := 10
		for layer in layers:
			var k := float(layer[0])
			for j in n:
				var u0 := float(j) / n * 2.0 - 1.0
				var u1 := float(j + 1) / n * 2.0 - 1.0
				# The crescent: its front edge bows forward, its back edge less, so it is thickest in the middle.
				var jag0 := 0.06 * sin(float(step) * 1.7 + j * 2.1)
				var jag1 := 0.06 * sin(float(step) * 1.7 + (j + 1) * 2.1)
				var f0 := Vector3(dir * (0.55 * (1.0 - u0 * u0) * k + jag0), u0 * h * 0.5 * k, 0)
				var f1 := Vector3(dir * (0.55 * (1.0 - u1 * u1) * k + jag1), u1 * h * 0.5 * k, 0)
				var b0 := Vector3(dir * (0.55 * (1.0 - u0 * u0) * k - 0.42 * k * (1.0 - u0 * u0)), u0 * h * 0.5 * k, 0)
				var b1 := Vector3(dir * (0.55 * (1.0 - u1 * u1) * k - 0.42 * k * (1.0 - u1 * u1)), u1 * h * 0.5 * k, 0)
				for v in [f0, f1, b1, f0, b1, b0]:
					im.surface_set_color(layer[1])
					im.surface_add_vertex(v)
	im.surface_end()


## The Wanderer's sword wave at `at` flying toward `dir`: a horizontal blade of light `BLADE_WAVE_LENGTH` long, pointed at the front and
## swept back to a forked tail, painted in layers as the knock-out blast and the swing arcs are (an ink silhouette, the longsword's blue,
## a pale band along its top edge, a white-hot core toward the point), its edges flickering on twos. (The sim's charge scaling of its
## size, x0.8 to x1.1, is the bridge's `projectile_scale`, passed as `size`.)
func _draw_blade_wave(node: MeshInstance3D, at: Vector3, dir: float, size := 1.0) -> void:
	node.position = at
	node.scale = Vector3.ONE * size
	var im: ImmediateMesh = node.mesh
	var step := int(Time.get_ticks_msec() / 33)
	if node.has_meta("step") and int(node.get_meta("step")) == step and node.get_meta("kind", "") == "blade":
		return
	node.set_meta("step", step)
	node.set_meta("kind", "blade")
	im.clear_surfaces()
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
	var length := BLADE_WAVE_LENGTH
	var half_h := 0.38
	# [scale of the outline, upward shift (share of the height), colour, from (0 tail .. 1 point)]
	var layers := [[1.0, 0.0, Color(0.1, 0.07, 0.12), 0.0], [0.86, 0.0, Color(0.14, 0.36, 0.86), 0.0],
		[0.5, 0.3, Color(0.5, 0.76, 1.0), 0.25], [0.26, 0.2, Color(1.0, 1.0, 0.96), 0.5]]
	var n := 12
	for layer in layers:
		var k := float(layer[0])
		var lift := float(layer[1]) * half_h
		var col: Color = layer[2]
		var from := float(layer[3])
		for j in n:
			var u0 := lerpf(from, 1.0, float(j) / n)
			var u1 := lerpf(from, 1.0, float(j + 1) / n)
			var pts := []
			for u in [u0, u1]:
				# Thickest a third of the way back from the point, pointed at both ends, the top edge bowed up a little.
				# (Each layer tapers to a point at its own start, so the inner bands are slivers inside the blade, not cut off square.)
				var w: float = (float(u) - from) / (1.0 - from)
				var thick: float = half_h * k * pow(sin(PI * pow(w, 1.5)), 0.7) * (1.0 + 0.08 * sin(float(step) * 1.9 + float(u) * 9.0))
				var x: float = dir * (float(u) - 0.55) * length * (0.85 + 0.15 * k)
				var bow: float = 0.12 * sin(PI * float(u))
				pts.append([Vector3(x, lift + bow + thick, 0), Vector3(x, lift + bow - thick * 0.75, 0)])
			for v in [pts[0][0], pts[1][0], pts[1][1], pts[0][0], pts[1][1], pts[0][1]]:
				im.surface_set_color(col)
				im.surface_add_vertex(v)
	im.surface_end()


## The pulling line's cord: from the thrower's hand (about chest height, a little in front) to the weight.
func _draw_cord(cord: MeshInstance3D, owner: int, to: Vector3, a: float) -> void:
	var im: ImmediateMesh = cord.mesh
	im.clear_surfaces()
	if owner < 0 or owner >= cur_pos.size():
		return
	var p: Vector2 = cur_pos[owner]
	if prev_pos.size() > owner:
		p = (prev_pos[owner] as Vector2).lerp(p, a)
	var hand := Vector3(p.x + 0.4 * signf(to.x - p.x), p.y + 1.1, 0.3)
	# A thin ribbon facing the camera (two triangles), sagging a little in the middle.
	var mid := (hand + to) * 0.5 + Vector3(0, -0.15, 0)
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLE_STRIP)
	for pt in [hand, mid, to]:
		im.surface_add_vertex(pt + Vector3(0, 0.025, 0))
		im.surface_add_vertex(pt - Vector3(0, 0.025, 0))
	im.surface_end()


func _exit_tree() -> void:
	# Effects outside a match (the menus) go back to real time.
	FxMaterial.stop_clock()


## Where each fighter is, for the scenery in front of them to go see-through over them (shaders/xray.gdshaderinc): its body's middle and
## half size on the fighters' plane; none for a slot with no fighter shown.
func _update_xray() -> void:
	for i in 4:
		var v := Vector4(0, 0, 1, 0)
		if i < views.size() and is_instance_valid(views[i]) and views[i].visible:
			var p: Vector3 = views[i].global_position
			# (Its bottom at the feet.)
			v = Vector4(p.x, p.y + 1.1, 0.85, 1.1)
		RenderingServer.global_shader_parameter_set("pf_xray_%d" % i, v)


## The light shafts from a stage's low sun (`rays` in shaders/post.gdshader): where the sun is on screen this frame, and how strong its
## shafts are (the stage's palette `rays`; none on the Fast tier, or without a sun, or with the sun behind the camera).
func _sun_rays() -> void:
	if post_mat == null or cam == null:
		return
	var strength := 0.0
	var sun := StageArt.sun_world
	if sun.is_finite() and VisualSettings.get_index("quality") > 0 and stage_view != null and not cam.is_position_behind(sun):
		strength = float(stage_view.theme.get("rays", 0.0))
		var px := cam.unproject_position(sun)
		post_mat.set_shader_parameter("sun_uv", px / get_viewport().get_visible_rect().size)
		if stage_view.theme.has("sky_glow"):
			post_mat.set_shader_parameter("rays_colour", (stage_view.theme.sky_glow as Array)[0])
	post_mat.set_shader_parameter("rays", strength)
