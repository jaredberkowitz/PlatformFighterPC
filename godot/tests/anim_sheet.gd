extends SceneTree
## Renders a contact sheet of one animation, frame by frame, for judging motion (needs a window, so run it without --headless).
## Run: Godot --path godot --script res://tests/anim_sheet.gd -- --what=run --class=0 --out=<file.png>
##   --what=run | dash | skid | <move name> (an attack such as fsmash, nair, jab)
##   --zoom=k       closer in (2 = twice as close), --facing=-1 to face left
##   --tip=x,y      where the move's hitbox is (forward space, from the feet), --timing=total,first,last its frames

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Lighting := preload("res://scripts/lighting.gd")

const CELL := Vector2i(380, 440)
const COLUMNS := 5


func _initialize() -> void:
	var what := "run"
	var cls := 0
	var out := "user://sheet.png"
	var tip := Vector3(1.6, 1.0, 0.6)
	var zoom := 1.0
	var facing := 1
	var timing := PackedInt32Array([40, 12, 16])
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--what="):
			what = a.substr(7)
		elif a.begins_with("--class="):
			cls = int(a.substr(8))
		elif a.begins_with("--out="):
			out = a.substr(6)
		elif a.begins_with("--facing="):
			facing = int(a.substr(9))
		elif a.begins_with("--zoom="):
			zoom = float(a.substr(7))
		elif a.begins_with("--tip="):
			var xy := a.substr(6).split(",")
			tip = Vector3(float(xy[0]), float(xy[1]), 0.6)
		elif a.begins_with("--timing="):
			var t := a.substr(9).split(",")
			timing = PackedInt32Array([int(t[0]), int(t[1]), int(t[2])])
	var vp := SubViewport.new()
	vp.size = CELL
	vp.own_world_3d = true
	vp.msaa_3d = Viewport.MSAA_4X
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	root.add_child(vp)
	var world := Node3D.new()
	vp.add_child(world)
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	env.background_color = Color(0.93, 0.88, 0.8)
	var we := WorldEnvironment.new()
	we.environment = env
	world.add_child(we)
	Lighting.apply(env, world, true)
	var cam := Camera3D.new()
	cam.fov = 30.0
	world.add_child(cam)
	cam.look_at_from_position(Vector3(1.2, 1.6, 8.5 / zoom), Vector3(0.6, 1.4, 0))
	var floor_mesh := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(8, 0.1, 3)
	floor_mesh.mesh = box
	floor_mesh.material_override = FighterView.toon(Color(0.62, 0.8, 0.45), false)
	floor_mesh.position = Vector3(0, -0.05, 0)
	world.add_child(floor_mesh)
	var view := FighterView.new()
	world.add_child(view)
	view.build(cls, Loadout.default_for(cls))

	var frames: Array = []
	match what:
		"run", "dash":
			for f in range(0, 20, 2):
				frames.append(f)
		"skid":
			for f in 12:
				frames.append(f)
		"idle", "walk":
			for f in range(0, 24 if what == "idle" else 20, 2):
				frames.append(f)
		_:
			for f in range(0, timing[0], 2):
				frames.append(f)
	var sheet := Image.create(CELL.x * COLUMNS, CELL.y * int(ceil(frames.size() / float(COLUMNS))), false, Image.FORMAT_RGBA8)
	var n := 0
	var sim_frame := 0
	for f in frames:
		var state := "Attack"
		var vel := Vector2.ZERO
		var move := what
		var grounded := not ["nair", "fair", "bair", "uair", "dair"].has(what)
		# (Idle is shown at half speed so its bounce spreads over the sheet.)
		match what:
			"run":
				state = "Run"
				vel = Vector2(0.24, 0)
				move = ""
			"dash":
				state = "Dash"
				vel = Vector2(0.27, 0)
				move = ""
			"skid":
				state = "Turn"
				vel = Vector2(-0.2, 0)
				move = ""
			"idle":
				state = "Idle"
				move = ""
			"walk":
				state = "Walk"
				vel = Vector2(0.11, 0)
				move = ""
		# Run the view at 60 frames a second up to this frame (one sim frame per step).
		var steps := 2
		for i in steps:
			sim_frame += 1
			var s := {"state": state, "facing": facing, "vel": vel, "platform": 0 if grounded else -1, "fast_fall": false, "invuln": 0,
				"ledge_invuln": 0, "frame": sim_frame, "shield": 1.0, "percent": 0.0, "move_name": move,
				"move_tip": tip if move != "" else Vector3.ZERO, "move_timing": timing, "charge": 0, "hitlag": 0,
				"tumble": false, "launch_pending": false, "char": cls, "class": cls, "reach": 3.0, "state_frame": f, "jumps": 1, "stocks": 3}
			view.apply(Vector3(0, 0 if grounded else 0.6, 0), s, 1.0 / 60.0)
		await process_frame
		await process_frame
		var img := vp.get_texture().get_image()
		img.convert(Image.FORMAT_RGBA8)
		sheet.blit_rect(img, Rect2i(Vector2i.ZERO, CELL), Vector2i((n % COLUMNS) * CELL.x, (n / COLUMNS) * CELL.y))
		n += 1
	sheet.save_png(out)
	quit(0)
