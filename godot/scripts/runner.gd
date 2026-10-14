extends "res://scripts/main.gd"
## The single-player run (docs/modes/RUNNER_PLAN.md, R1 and R1b): the versus scene with one fighter, played by the same input, controls
## and physics, on the sim's streamed chain of rooms (`sim_core::run`). This script only changes what the run presents differently: the
## camera (the sim's view, following the player both ways inside the rooms in play, and height), the stage (built room by room as the
## sim streams it, with its walls and gates), the mobs, the run's display (the map, the shop) and its results. Gameplay all lives in the
## sim, the shop included: it is a room the fighter walks into (R2a), and the same inputs that move the fighter buy the wares.
##
## From the menus: `Roster.session.run_entry` is the fighter. Launched directly: `--run` (the last fighter played, or `--fighter=<name>`),
## `--seed=N` for a fixed run.

const RunHud := preload("res://ui/run_hud.gd")
const RunResults := preload("res://ui/run_results.gd")
const RunStage := preload("res://scripts/run_stage.gd")
const MobView := preload("res://scripts/mob_view.gd")

var run_hud: CanvasLayer
var run_stage: Node3D
var run_seed := 1
var run_error := ""
var run_name := "Fighter"
var info := PackedInt32Array()
var info_before := PackedInt32Array()
var view := PackedFloat32Array()
## The view's left edge last tick and this tick (drawn between them), and the height the camera keeps.
var edge_prev := 0.0
var edge_cur := 0.0
var cam_y := 3.0
var mob_views: Array = []
var mobs_prev := PackedFloat32Array()
var mobs_cur := PackedFloat32Array()
## A banner for a moment: its text and when it fades (engine seconds).
var flash_text := ""
var flash_until := 0.0
var combo_flash := 0.0
var rng_seed := RandomNumberGenerator.new()
var shop := PackedInt32Array()
var shop_before := PackedInt32Array()

const MOB_STRIDE := 12
const ROOM_REST := 3
const INFO_SIZE := 17
const NOTES := ["", "THANK YOU!", "NOT ENOUGH COINS", "SOLD OUT", "NOT NEEDED"]
## Seconds after the last life goes before the results come up.
const RESULTS_AFTER := 1.6


func _player_count() -> int:
	return 1


func _build_world() -> void:
	super._build_world()
	# The match display is the versus one; the run has its own.
	hud.visible = false
	run_hud = RunHud.new()
	add_child(run_hud)
	run_hud.build()
	run_hud.visible = not OS.get_cmdline_user_args().has("--noui")
	for i in 16:
		var mv := MobView.new()
		mv.visible = false
		add_child(mv)
		mob_views.append(mv)


## The fighter: the menus' choice, else `--fighter=` or the last one played.
func _load_content() -> void:
	var entry: Dictionary = Roster.session.get("run_entry", {})
	if entry.is_empty():
		entry = Roster.net_entry()
	run_name = str(entry.get("name", "Fighter"))
	names[0] = run_name
	run_error = sim.run_begin(Roster.spec_bytes(entry))
	if run_error != "":
		push_error("run: " + run_error)
		content_note = "RUN NOT SET UP: " + run_error
		return
	if entry.has("look"):
		views[0].rebuild(entry.look)
	views[0].set_name_tag(run_name)
	rng_seed.randomize()


func _apply_rules() -> void:
	pass


func _rebuild_stage() -> void:
	stage_view.clear()
	stage_view.theme = StageArt.theme({"backdrop": "city", "sky_top": "", "sky_bottom": ""})
	if run_stage == null:
		run_stage = RunStage.new()
		add_child(run_stage)
	run_stage.setup(stage_view.theme)
	_apply_sky()


## The seed: `Roster.session.run_seed` or `--seed=N` for the first run, a fresh one for every run after.
func _next_seed() -> int:
	if Roster.session.has("run_seed"):
		var s := int(Roster.session.run_seed)
		Roster.session.erase("run_seed")
		return s
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--seed=") and run_seed == 1 and info.is_empty():
			return int(a.substr(7))
	return rng_seed.randi_range(1, 999999)


func _restart() -> void:
	if results != null:
		results.queue_free()
		results = null
	cam.attributes = null
	_clear_scuffs()
	end_timer = 0.0
	run_seed = _next_seed()
	if not sim.run_start(run_seed):
		push_error("run: no run is set up")
	proj_cur = sim.projectile_slots()
	proj_prev = proj_cur
	_refresh(0)
	prev_pos[0] = cur_pos[0]
	_rebuild_boxes()
	paused = false
	view = sim.run_view()
	info = sim.run_info()
	info_before = info
	edge_prev = view[0]
	edge_cur = view[0]
	cam_y = cur_pos[0].y + 3.0
	cam_base = Vector3.INF
	mobs_cur = sim.run_mobs()
	mobs_prev = mobs_cur
	shop = sim.run_shop()
	shop_before = shop
	flash_text = ""
	run_hud.visible = not OS.get_cmdline_user_args().has("--noui")
	if run_stage != null:
		run_stage.sync(sim, info[9])


func _tick_once_inner(advance := true) -> void:
	edge_prev = edge_cur
	mobs_prev = mobs_cur
	info_before = info
	shop_before = shop
	super._tick_once_inner(advance)
	view = sim.run_view()
	info = sim.run_info()
	shop = sim.run_shop()
	mobs_cur = sim.run_mobs()
	if view.size() > 0:
		edge_cur = view[0]
	_run_events()


## Presentation cues from what changed this tick: hits on mobs, mobs beaten, coins, a lost or bought life, an arena shutting and
## opening, a rest stop.
func _run_events() -> void:
	if info.size() < INFO_SIZE or info_before.size() < INFO_SIZE:
		return
	for n in mobs_cur.size() / MOB_STRIDE:
		var o := n * MOB_STRIDE
		if mobs_cur[o] < 0.5 or mobs_prev.size() <= o:
			continue
		var at := Vector3(mobs_cur[o + 3], mobs_cur[o + 4] + mobs_cur[o + 11], 0.6)
		var struck: bool = mobs_cur[o + 7] > 0.0 and mobs_prev[o + 7] == 0.0 and mobs_cur[o + 10] > mobs_prev[o + 10]
		if struck:
			sfx.play_group("punch", 1.15)
			_burst(at, Color(1.0, 0.9, 0.6), 0.5, 0.18)
		if int(mobs_cur[o + 6]) == 5 and int(mobs_prev[o + 6]) != 5:
			_burst(at, MobView.COLOURS[clampi(int(mobs_cur[o + 2]), 0, 1)].lightened(0.3), 1.4, 0.35)
	if info[2] > info_before[2]:
		sfx.play("blip", 1.9, -4.0)
		combo_flash = 1.0
	if info[7] < info_before[7] and info[0] == 0:
		_flash("-1 LIFE", 1.2)
	if info[7] > info_before[7]:
		_flash("+1 LIFE", 1.2)
	if info[11] == 1 and info_before[11] == 0:
		sfx.play("blip", 0.7, -2.0)
		_flash("ARENA!", 1.4)
	if info[12] > info_before[12]:
		sfx.play("blip", 1.4, -2.0)
		_flash("CLEAR! +%d" % (info[2] - info_before[2]), 1.6)
	if info[8] > info_before[8] and info[10] == ROOM_REST:
		_flash("REST STOP", 1.2)
	if info[0] == 1 and info_before[0] == 0:
		_flash("RUN OVER", 3.0)
	if info[15] > info_before[15]:
		sfx.play("blip", 2.4, -8.0)
	if info[13] == 1 and info_before[13] == 0:
		sfx.play("blip", 1.1, -4.0)
	if shop.size() >= 10 and shop_before.size() >= 10:
		if shop[3] > shop_before[3] and shop[2] > 0 and shop[2] < NOTES.size():
			_flash(NOTES[shop[2]], 1.0)
			sfx.play("blip", 1.8 if shop[2] == 1 else 0.6, -3.0)
		if shop[9] > shop_before[9]:
			sfx.play_group("punch", 1.4)


func _flash(text: String, seconds: float) -> void:
	flash_text = text
	flash_until = Time.get_ticks_msec() / 1000.0 + seconds


func _process_inner(delta: float) -> void:
	super._process_inner(delta)
	var a := _alpha()
	if run_stage != null and info.size() >= 10:
		run_stage.sync(sim, info[9])
		run_stage.animate(delta)
		if view.size() >= 12 and view[9] > 0.5:
			run_stage.build_interior(sim)
			run_stage.animate_interior(shop, delta)
		elif view.size() > 1:
			run_stage.follow(lerpf(edge_prev, edge_cur, a) + view[1] * 0.5, cam_y)
	var count := mini(mob_views.size(), mobs_cur.size() / MOB_STRIDE)
	for n in count:
		var o := n * MOB_STRIDE
		var mv: Node3D = mob_views[n]
		if mobs_cur[o] < 0.5:
			mv.visible = false
			continue
		var b := int(mobs_cur[o + 2])
		if mv.behavior != b or not is_equal_approx(mv.radius, mobs_cur[o + 11]):
			mv.build(b, mobs_cur[o + 11])
		var now := Vector2(mobs_cur[o + 3], mobs_cur[o + 4])
		var was := now
		if mobs_prev.size() > o + 4 and mobs_prev[o] > 0.5:
			was = Vector2(mobs_prev[o + 3], mobs_prev[o + 4])
		var slice := []
		for k in MOB_STRIDE:
			slice.append(mobs_cur[o + k])
		mv.apply(was.lerp(now, a), slice, delta)


func _update_hud(delta: float) -> void:
	views[0].set_overhead(false, "P1", PLAYER_COLORS[0])
	if info.size() < 10:
		return
	combo_flash = move_toward(combo_flash, 0.0, delta * 3.0)
	var banner := ""
	var alpha := 1.0
	var now := Time.get_ticks_msec() / 1000.0
	if run_error != "":
		banner = "NO RUN"
	elif flash_text != "" and now < flash_until:
		banner = flash_text
		alpha = clampf((flash_until - now) / 0.4, 0.0, 1.0)
	elif sim.frame() < 75:
		banner = "RUN!"
		alpha = clampf(float(75 - sim.frame()) / 30.0, 0.0, 1.0)
	var inside := info.size() >= INFO_SIZE and info[13] == 1
	run_hud.show_state({
		"metres": info[1], "coins": info[2], "kills": info[3], "combo": info[4], "best": info[5], "mult": info[6],
		"step": 5, "max_mult": 5, "lives": info[7], "max_lives": maxi(3, info[7]), "percent": snaps[0].get("percent", 0.0),
		"name": run_name, "banner": banner, "banner_alpha": alpha, "combo_flash": combo_flash,
		"map": sim.run_map(4), "door": info.size() >= INFO_SIZE and info[14] == 1 and not inside,
		"gate": info.size() >= INFO_SIZE and info[11] == 1, "shop": shop, "shop_names": sim.run_shop_names(),
	})
	if info[0] == 1 and results == null:
		end_timer += delta
		if end_timer > RESULTS_AFTER:
			_show_results(-2)


func _update_camera(a: float, delta: float) -> void:
	if view.size() < 2:
		return
	var width: float = view[1]
	var edge := lerpf(edge_prev, edge_cur, a)
	if view.size() >= 12 and view[9] > 0.5:
		# Inside the shop: the camera on its middle (the street outside stands still meanwhile).
		edge = view[10] + view[11] * 0.5 - width * 0.5
	# (At 30 degrees of view, 16:9, the visible width is about 0.95 of the distance: the screen's edges are the sim's.)
	var dist := width / 0.95
	var p: Vector2 = prev_pos[0].lerp(cur_pos[0], a)
	# Height follows the fighter outside a dead zone, gently, and never below the street.
	var want := clampf(p.y + 2.0, view[3] + 9.0, view[4] - 8.0)
	if view.size() >= 12 and view[9] > 0.5:
		want = 4.0
		cam_y = want
	if absf(want - cam_y) > 2.0:
		cam_y = lerpf(cam_y, want - signf(want - cam_y) * 2.0, clampf(delta * 3.0, 0.0, 1.0))
	var target := Vector3(edge + width * 0.5, cam_y + 1.6 - dist * HUD_LIFT, dist)
	if cam_base == Vector3.INF:
		cam_base = target
	cam_base = Vector3(target.x, lerpf(cam_base.y, target.y, clampf(delta * 6.0, 0.0, 1.0)), target.z)
	var offset := Vector3.ZERO
	var roll := 0.0
	if cam_shake > 0.001:
		if shake_dir == Vector2.ZERO:
			shake_dir = Vector2.from_angle(shake_rng.randf() * TAU)
		shake_phase += delta * SHAKE_HZ * TAU
		var along := sin(shake_phase) * cam_shake
		offset = Vector3(shake_dir.x * along, shake_dir.y * along, 0.0)
		roll = sin(shake_phase * 0.9 + 0.5) * cam_shake * SHAKE_ROLL
		cam_shake = move_toward(cam_shake, 0.0, delta * 2.6)
	else:
		shake_dir = Vector2.ZERO
		shake_phase = 0.0
	if flash_rect != null:
		flash_alpha = move_toward(flash_alpha, 0.0, delta * 3.5)
		flash_rect.color = Color(1, 1, 1, flash_alpha)
	cam_punch = 0.0
	var lift := cam_base.z * tan(deg_to_rad(CAMERA_PITCH))
	var calm := VisualSettings.shake()
	cam.position = cam_base + offset * calm + Vector3(0, lift, 0)
	cam.rotation = Vector3(-deg_to_rad(CAMERA_PITCH), 0, roll * calm)
	if post_mat != null and absf(cam.position.z - post_stage_dist) > 0.25:
		post_stage_dist = cam.position.z
		post_mat.set_shader_parameter("stage_dist", post_stage_dist)


func _show_results(_winner: int) -> void:
	var dof := CameraAttributesPractical.new()
	dof.dof_blur_far_enabled = true
	dof.dof_blur_far_distance = 2.0
	dof.dof_blur_far_transition = 6.0
	dof.dof_blur_amount = RESULTS_BLUR
	cam.attributes = dof
	run_hud.visible = false
	var choices := [["Run again", "again"], ["Quit", "quit"]]
	if Roster.session.get("from_menu", false):
		choices = [["Run again", "again"], ["Change fighter", "select"], ["Main menu", "menu"]]
	results = RunResults.new()
	add_child(results)
	results.build({"metres": info[1], "coins": info[2], "kills": info[3], "best": info[5], "rooms": info[8] + 1, "cleared": info[12],
		"seed": run_seed, "name": run_name}, choices)
	results.chosen.connect(_on_result)


func _on_result(action: String) -> void:
	match action:
		"again":
			_restart()
		"select":
			Roster.session = {"mode": "run"}
			_leave_to("select")
		"menu":
			_leave_to("menu")
		_:
			get_tree().quit()


## A run has no training keys: pause, mute, the debug overlay, starting over, and Esc back to the fighter choice (or quitting).
func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	if results != null and event.keycode != KEY_ESCAPE:
		results.handle_key(event)
		return
	match event.keycode:
		KEY_F4:
			Music.of(self).set_muted(sfx.toggle_mute())
		KEY_F1:
			overlay_on = not overlay_on
			overlay.set_overlay_visible(overlay_on)
		KEY_P:
			paused = not paused
		KEY_ESCAPE:
			if Roster.session.get("from_menu", false):
				Roster.session = {"mode": "run"}
				_leave_to("select")
			else:
				get_tree().quit()
