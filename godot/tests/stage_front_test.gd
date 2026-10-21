extends SceneTree
## The modelled stage fronts (scripts/stage_bake.gd `FRONTS`, art/blender/make_grove_front.py) against the sim's own stages: each was
## built for a stage the sim still has (its key is that stage's theme and layout), it stays under the walkable top (the turf, its lip and
## the ledges stay code-built and where fighters stand and grab), and within a unit of the blocks' outline, so no fighter passes in front
## of or behind something that is not solid. Run: Godot --headless --path godot --script res://tests/stage_front_test.gd

const StageBake := preload("res://scripts/stage_bake.gd")
const StageArt := preload("res://scripts/stage_art.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		failed = true
		print("FAIL ", what)


## `n`'s transform relative to `top` (the test runs before anything is in the tree, so there is no global transform yet).
func _xf(n: Node3D, top: Node3D) -> Transform3D:
	var t := Transform3D.IDENTITY
	while n != null and n != top:
		t = n.transform * t
		n = n.get_parent() as Node3D
	return top.transform * t


func _initialize() -> void:
	if not ClassDB.class_exists("SimRunner"):
		print("FAIL the game bridge is not loaded")
		quit(1)
		return
	var sim = ClassDB.instantiate("SimRunner")
	var stages := {}
	for i in 32:
		if not sim.use_stage(i):
			break
		var rects: Array = []
		for k in sim.platform_count():
			rects.append(sim.platform_rect(k))
		stages[StageBake.key(String(sim.stage_look().get("backdrop", "")), rects)] = rects
	check(stages.size() >= 5, "the sim's stages were read: %d" % stages.size())
	for key in StageBake.FRONTS:
		var path: String = StageBake.FRONTS[key]
		check(ResourceLoader.exists(path), "%s exists" % path)
		check(stages.has(key), "%s is built for one of the sim's stages (%s): rebuild it if the stage changed" % [path, key])
		if not stages.has(key) or not ResourceLoader.exists(path):
			continue
		var solids: Array = []
		for r in stages[key]:
			if r[4] < 0.5:
				solids.append(r)
		var top := -INF
		var low := INF
		for r in solids:
			top = maxf(top, r[2])
			low = minf(low, r[3])
		var model := (load(path) as PackedScene).instantiate() as Node3D
		var highest := -INF
		var nearest := -INF
		var outside := 0
		var verts := 0
		var where := {}
		for mi: MeshInstance3D in model.find_children("*", "MeshInstance3D", true, false):
			for s in mi.mesh.get_surface_count():
				for v: Vector3 in mi.mesh.surface_get_arrays(s)[Mesh.ARRAY_VERTEX]:
					var w := _xf(mi, model) * v
					verts += 1
					highest = maxf(highest, w.y)
					nearest = maxf(nearest, w.z)
					var y := clampf(w.y, low, top)
					var lo := INF
					var hi := -INF
					for r in solids:
						if y >= r[3] - 0.001 and y <= r[2] + 0.001:
							lo = minf(lo, r[0])
							hi = maxf(hi, r[1])
					if w.x < lo - 1.0 or w.x > hi + 1.0:
						outside += 1
						where[String(mi.name)] = int(where.get(String(mi.name), 0)) + 1
		model.free()
		check(verts > 0, "%s has geometry" % path)
		check(highest <= top - 0.25, "%s stays under the walkable top: %.2f against %.2f" % [path, highest, top])
		check(outside == 0, "%s stays within a unit of the blocks' outline: %d of %d points outside %s" % [path, outside, verts, where])
		check(nearest <= StageArt.STAGE_FRONT + 2.6, "%s stands no further forward than the turf's lip: %.2f" % [path, nearest])
	# The modelled play areas (StageBake.PLAYS, art/blender/make_meadow_play.py): what fighters walk on is the model now, so every
	# walkable top must stand at the sim's height across its span, and nothing may poke up out of it where fighters stand.
	for key in StageBake.PLAYS:
		var path: String = StageBake.PLAYS[key]
		check(ResourceLoader.exists(path), "%s exists" % path)
		check(stages.has(key), "%s is built for one of the sim's stages (%s): rebuild it if the stage changed" % [path, key])
		if not stages.has(key) or not ResourceLoader.exists(path):
			continue
		var model := (load(path) as PackedScene).instantiate() as Node3D
		var points: Array = []
		for mi: MeshInstance3D in model.find_children("*", "MeshInstance3D", true, false):
			for s in mi.mesh.get_surface_count():
				for v: Vector3 in mi.mesh.surface_get_arrays(s)[Mesh.ARRAY_VERTEX]:
					points.append(_xf(mi, model) * v)
		model.free()
		for r in stages[key]:
			# (Only a top fighters can reach: a block with another block sitting right over it is covered, as StageView has it.)
			var covered := false
			for o in stages[key]:
				if r[4] < 0.5 and o[4] < 0.5 and o != r and is_equal_approx(o[3], r[2]) and o[0] <= r[0] + 0.01 and o[1] >= r[1] - 0.01:
					covered = true
			if covered:
				continue
			var depth := 4.0 if r[4] > 0.5 else 8.0
			var x0: float = r[0] + 0.3
			var x1: float = r[1] - 0.3
			var z0 := StageArt.STAGE_FRONT - depth + 0.3
			var z1 := StageArt.STAGE_FRONT - 0.3
			var top_here := -INF
			var poking := 0
			for w: Vector3 in points:
				if w.x < x0 or w.x > x1 or w.z < z0 or w.z > z1:
					continue
				if w.y <= r[2] + 0.03:
					top_here = maxf(top_here, w.y)
				elif w.y < r[2] + 1.5:
					poking += 1
			check(absf(top_here - r[2]) <= 0.02, "%s: the walkable top over [%.1f, %.1f] stands at the sim's %.2f: %.2f" % [path, r[0], r[1], r[2], top_here])
			check(poking == 0, "%s: nothing pokes up out of the walkable top over [%.1f, %.1f]: %d points" % [path, r[0], r[1], poking])
	sim.free()
	print("stage front test %s" % ("FAILED" if failed else "PASSED"))
	quit(1 if failed else 0)
