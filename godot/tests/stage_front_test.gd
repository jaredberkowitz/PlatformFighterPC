extends SceneTree
## The modelled stage fronts (scripts/stage_bake.gd `FRONTS`, art/blender/make_grove_front.py) against the sim's own stages: each was
## built for a stage the sim still has (its key is that stage's theme and layout), it stays under the walkable top (the turf, its lip and
## the ledges stay code-built and where fighters stand and grab), and within a unit of the blocks' outline, so no fighter passes in front
## of or behind something that is not solid. Run: Godot --headless --path godot --script res://tests/stage_front_test.gd

const StageBake := preload("res://scripts/stage_bake.gd")
const StageArt := preload("res://scripts/stage_art.gd")
const WorldMotion := preload("res://scripts/world_motion.gd")

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
	# The contact shade (art/blender/light_stage.py, review 2's idea 3): every modelled front, play area and ground carries it, its
	# vertex colours' alpha under 1 throughout (the shade, kept so the next run can take it out), so a piece rebuilt without lighting it
	# again is caught.
	var lit: Array = StageBake.FRONTS.values() + StageBake.PLAYS.values()
	for g in StageBake.GROUNDS.values():
		lit.append(g[0])
	for path: String in lit:
		if not ResourceLoader.exists(path):
			continue
		var model := (load(path) as PackedScene).instantiate() as Node3D
		var unlit := 0
		for mi: MeshInstance3D in model.find_children("*", "MeshInstance3D", true, false):
			for s in mi.mesh.get_surface_count():
				var cols = mi.mesh.surface_get_arrays(s)[Mesh.ARRAY_COLOR]
				if cols == null or (cols as PackedColorArray).is_empty() or (cols as PackedColorArray)[0].a > 0.999:
					unlit += 1
		model.free()
		check(unlit == 0, "%s carries the contact shade: %d surfaces without it (run art/blender/light_stage.py after rebuilding it)" % [path, unlit])
	# The paper props (StageBake.PAPER_KITS, art/blender/make_paper_props.py): every kit has each prop the builders hand their places to
	# (StageArt._paper), each part in a family the game paints, so no prop falls back to the built one or goes grey.
	for theme_name in StageBake.PAPER_KITS:
		var path: String = StageBake.PAPER_KITS[theme_name]
		check(ResourceLoader.exists(path), "%s exists" % path)
		if not ResourceLoader.exists(path):
			continue
		var kit := (load(path) as PackedScene).instantiate()
		for prop_name in ["hay_bale", "barrel", "signpost", "rock", "fence_post", "fence_rail", "flower", "mushroom", "stump", "crate",
				"lamp_post", "life_ring", "potted_plant", "cactus", "water_tower", "ac_unit", "roof_vent"]:
			var prop := kit.find_child("prop_" + prop_name, true, false)
			check(prop != null, "%s has prop_%s" % [path, prop_name])
			if prop == null:
				continue
			for mi: MeshInstance3D in prop.find_children("*", "MeshInstance3D", true, false):
				var m := mi.mesh.surface_get_material(0)
				var fam := (m.resource_name if m != null else "").get_slice(".", 0).trim_suffix("_back")
				check(StageArt.PAPER_COLOURS.has(fam), "%s prop_%s: part %s is in a family the game paints (%s)" % [path, prop_name, mi.name, fam])
		kit.free()
	# The paper houses (StageBake.HOUSE_KIT, art/blender/make_paper_houses.py): each house `_cottage` and `_barn` hand their places to,
	# each part in a family the game paints, so no house falls back to the old cottage or goes grey.
	check(ResourceLoader.exists(StageBake.HOUSE_KIT), "%s exists" % StageBake.HOUSE_KIT)
	if ResourceLoader.exists(StageBake.HOUSE_KIT):
		var houses := (load(StageBake.HOUSE_KIT) as PackedScene).instantiate()
		for kind in StageArt.TOWN_TYPES + ["barn"]:
			var house := houses.find_child("prop_house_" + kind, true, false)
			check(house != null, "%s has prop_house_%s" % [StageBake.HOUSE_KIT, kind])
			if house == null:
				continue
			for mi: MeshInstance3D in house.find_children("*", "MeshInstance3D", true, false):
				var m := mi.mesh.surface_get_material(0)
				var fam := (m.resource_name if m != null else "").get_slice(".", 0)
				check(StageArt.PAPER_COLOURS.has(fam), "prop_house_%s: part %s is in a family the game paints (%s)" % [kind, mi.name, fam])
		houses.free()
	# The mood light and the world-building (StageBake.MOODS and WORLDS, art/blender/make_mood.py and make_world.py): built for one of
	# the sim's stages, and over the play space (the stage's span, from its lowest top up to ten units over its highest) they stay
	# behind the fighters' plane, so no beam, lantern, gull or boat ever crosses a fighter.
	var pieces: Array = []
	for key in StageBake.MOODS:
		pieces.append([key, StageBake.MOODS[key]])
	for key in StageBake.WORLDS:
		pieces.append([key, StageBake.WORLDS[key]])
	for piece in pieces:
		var key: String = piece[0]
		var path: String = piece[1]
		check(ResourceLoader.exists(path), "%s exists" % path)
		check(stages.has(key), "%s is built for one of the sim's stages (%s): rebuild it if the stage changed" % [path, key])
		if not stages.has(key) or not ResourceLoader.exists(path):
			continue
		var x0 := INF
		var x1 := -INF
		var y0 := INF
		var y1 := -INF
		for r in stages[key]:
			x0 = minf(x0, r[0] - 1.0)
			x1 = maxf(x1, r[1] + 1.0)
			y0 = minf(y0, r[2])
			y1 = maxf(y1, r[2] + 10.0)
		var model := (load(path) as PackedScene).instantiate() as Node3D
		var verts := 0
		var crossing := 0
		for mi: MeshInstance3D in model.find_children("*", "MeshInstance3D", true, false):
			for s in mi.mesh.get_surface_count():
				for v: Vector3 in mi.mesh.surface_get_arrays(s)[Mesh.ARRAY_VERTEX]:
					var w := _xf(mi, model) * v
					verts += 1
					if w.x > x0 and w.x < x1 and w.y > y0 and w.y < y1 and w.z > -0.4:
						crossing += 1
		# (What moves in it, art/blender/make_world.py's `move_<kind>_<n>` pivots, is a kind scripts/world_motion.gd knows.)
		var unknown: Array = []
		for m in model.find_children("move_*", "Node3D", true, false):
			if not WorldMotion.KINDS.has(WorldMotion.kind_of(m)):
				unknown.append(String(m.name))
		model.free()
		check(verts > 0, "%s has geometry" % path)
		check(crossing == 0, "%s stays behind the fighters' plane over the play space: %d of %d points in front" % [path, crossing, verts])
		check(unknown.is_empty(), "%s: every moving thing is a kind the world's motion knows: %s" % [path, unknown])
	sim.free()
	print("stage front test %s" % ("FAILED" if failed else "PASSED"))
	quit(1 if failed else 0)
