extends SceneTree
## The style benchmark: every default look (and the darkest and palest body colours) lit by each stage mood (scripts/lighting.gd),
## against that mood's bright haze and a dark backdrop, side by side on one sheet. For each fighter it measures how well it stands out:
## `edge`, the share of its silhouette where, within a few pixels inside the rim, the fighter differs clearly from the backdrop (the
## outline's and the edge light's job), and `body`, how far its own colours sit from the backdrop's brightness (the shading's job). It fails if any fighter loses its silhouette. Needs a window (not --headless).
## Run: Godot --path godot --script res://tests/look_bench.gd -- --out=<folder>
## Writes look_bench.png and prints a line per mood and backdrop.

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
const Lighting := preload("res://scripts/lighting.gd")

const CELL := Vector2i(150, 230)
## The bodies tried: the four default players, then the darkest (midnight) and the palest (cloud) colours.
const BODIES := [-1, -2, -3, -4, 11, 8]
## A silhouette rim pixel counts as visible when its brightness differs from the backdrop's by this much (0..1).
const EDGE_STEP := 0.12
## The least share of every silhouette that must be visible.
const EDGE_MIN := 0.85
## How far into the silhouette (pixels) the rim test looks.
const RIM_DEPTH := 3

var failed := false


func _look(body: int) -> RefCounted:
	if body < 0:
		return Loadout.default_for(-body - 1)
	var l: RefCounted = Loadout.default_for(0)
	l.color = body
	return l


## One mood and backdrop: a viewport with its own world, the mood's lights and the fighters in a row.
func _stage(mood: String, dark: bool) -> Dictionary:
	var vp := SubViewport.new()
	vp.size = Vector2i(CELL.x * BODIES.size(), CELL.y)
	vp.own_world_3d = true
	vp.msaa_3d = Viewport.MSAA_4X
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	root.add_child(vp)
	var world := Node3D.new()
	vp.add_child(world)
	var env := Environment.new()
	env.background_mode = Environment.BG_COLOR
	var m: Dictionary = Lighting.MOODS[mood]
	env.background_color = (m.ambient[0] as Color).darkened(0.72) if dark else m.haze
	var we := WorldEnvironment.new()
	we.environment = env
	world.add_child(we)
	Lighting.apply(env, world, true)
	Lighting.stage_mood(env, world, {"name": mood})
	var cam := Camera3D.new()
	cam.projection = Camera3D.PROJECTION_ORTHOGONAL
	cam.size = 2.6
	cam.keep_aspect = Camera3D.KEEP_HEIGHT
	world.add_child(cam)
	cam.look_at_from_position(Vector3(0, 1.15, 10.0), Vector3(0, 1.15, 0))
	var views := []
	var span := 2.6 * float(vp.size.x) / float(vp.size.y)
	for k in BODIES.size():
		var v := FighterView.new()
		world.add_child(v)
		v.build(k % 4, _look(BODIES[k]))
		v.blade_pivot.visible = false
		v.percent_label.visible = false
		v.name_label.visible = false
		v.position = Vector3(-span * 0.5 + span * (k + 0.5) / BODIES.size(), 0.0, 0.0)
		v.model.rotation.y = 0.35
		views.append(v)
	return {"vp": vp, "views": views}


## Part of the fighter: the pixel changed from the bare backdrop by more than a faint trace (a nearly clear effect is not the fighter).
func _part(with: Color, without: Color) -> bool:
	return absf(with.r - without.r) + absf(with.g - without.g) + absf(with.b - without.b) > 0.06


## The pixels of a fighter's cell that look like the backdrop but are enclosed by the fighter (a pale belly lit the colour of a pale sky):
## they are not backdrop the silhouette borders, so no rim is measured against them. Only backdrop reachable from the cell's border is.
func _enclosed(img: Image, bare: Image, x0: int) -> Dictionary:
	var outside := {}
	var todo: Array[Vector2i] = []
	for x in range(x0, x0 + CELL.x):
		todo.append(Vector2i(x, 0))
		todo.append(Vector2i(x, CELL.y - 1))
	for y in CELL.y:
		todo.append(Vector2i(x0, y))
		todo.append(Vector2i(x0 + CELL.x - 1, y))
	while not todo.is_empty():
		var p: Vector2i = todo.pop_back()
		if p.x < x0 or p.x >= x0 + CELL.x or p.y < 0 or p.y >= CELL.y or outside.has(p):
			continue
		if _part(img.get_pixelv(p), bare.get_pixelv(p)):
			continue
		outside[p] = true
		for d in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
			todo.append(p + d)
	var inside := {}
	for y in CELL.y:
		for x in range(x0, x0 + CELL.x):
			var q := Vector2i(x, y)
			if not outside.has(q) and not _part(img.get_pixelv(q), bare.get_pixelv(q)):
				inside[q] = true
	return inside


func _luma(c: Color) -> float:
	return c.r * 0.2126 + c.g * 0.7152 + c.b * 0.0722


func _initialize() -> void:
	var out := "user://"
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
	var moods: Array = Lighting.MOODS.keys()
	var rows := []
	for mood in moods:
		for dark in [false, true]:
			rows.append(_stage(mood, dark))
	await create_timer(1.2).timeout
	var shots := []
	for r in rows:
		shots.append(r.vp.get_texture().get_image())
	# Again without the fighters: the backdrop alone, so the fighters' pixels are exactly the ones that changed.
	for r in rows:
		for v in r.views:
			v.visible = false
	await create_timer(0.4).timeout
	var sheet := Image.create(CELL.x * BODIES.size(), CELL.y * rows.size(), false, Image.FORMAT_RGB8)
	var worst := 1.0
	var misses := []
	for i in rows.size():
		var img: Image = shots[i]
		var bare: Image = rows[i].vp.get_texture().get_image()
		img.convert(Image.FORMAT_RGB8)
		bare.convert(Image.FORMAT_RGB8)
		sheet.blit_rect(img, Rect2i(Vector2i.ZERO, img.get_size()), Vector2i(0, CELL.y * i))
		var line := "%-7s %-6s" % [moods[i / 2], "dark" if i % 2 == 1 else "bright"]
		for k in BODIES.size():
			var x0 := k * CELL.x
			var inside := _enclosed(img, bare, x0)
			var edge_n := 0
			var edge_ok := 0
			var body_sum := 0.0
			var body_n := 0
			for y in range(1, CELL.y - 1):
				for x in range(x0 + 1, x0 + CELL.x - 1):
					if not _part(img.get_pixel(x, y), bare.get_pixel(x, y)) and not inside.has(Vector2i(x, y)):
						continue
					var back := _luma(bare.get_pixel(x, y))
					# A rim pixel: part of the fighter with backdrop next to it.
					var rim := false
					for d in [Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, 1), Vector2i(0, -1)]:
						var n := Vector2i(x + d.x, y + d.y)
						if not _part(img.get_pixel(n.x, n.y), bare.get_pixel(n.x, n.y)) and not inside.has(n):
							rim = true
					var l := _luma(img.get_pixel(x, y))
					if rim:
						edge_n += 1
						# Visible if anything of the fighter within RIM_DEPTH pixels differs clearly from the backdrop here (a dark line on a
						# dark backdrop is lost, but the edge light just inside it can still carry the silhouette).
						var seen := false
						for dy in range(-RIM_DEPTH, RIM_DEPTH + 1):
							for dx in range(-RIM_DEPTH, RIM_DEPTH + 1):
								var q := Vector2i(clampi(x + dx, x0, x0 + CELL.x - 1), clampi(y + dy, 0, CELL.y - 1))
								var qc := img.get_pixel(q.x, q.y)
								if _part(qc, bare.get_pixel(q.x, q.y)) and absf(_luma(qc) - back) >= EDGE_STEP:
									seen = true
						if seen:
							edge_ok += 1
						else:
							misses.append(Vector2i(x, y + CELL.y * i))
					else:
						body_sum += absf(l - back)
						body_n += 1
			var edge := float(edge_ok) / maxf(1.0, float(edge_n))
			var body := body_sum / maxf(1.0, float(body_n))
			worst = minf(worst, edge)
			line += "  edge %.2f body %.2f" % [edge, body]
			if edge < EDGE_MIN:
				failed = true
				line += " (LOW)"
		print(line)
	sheet.save_png(out.path_join("look_bench.png"))
	# The same sheet with every silhouette pixel that failed in red, to see where a fighter is lost.
	for m in misses:
		sheet.set_pixelv(m, Color(1, 0, 0))
	sheet.save_png(out.path_join("look_bench_misses.png"))
	print("worst silhouette %.2f (needs %.2f)" % [worst, EDGE_MIN])
	print("look bench ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
