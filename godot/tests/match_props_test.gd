extends SceneTree
## The match props (godot/models/match_props, from art/blender/make_match_props.py; drawn by scripts/match_props.gd and, for energy
## shots, scripts/shot_paint.gd): every prop a shot or an item kind names loads and is coloured, keeps to the prop budget
## (docs/ART_WORKFLOW.md: 3,000 triangles), names its parts with words the palette knows (a new word would fall back to wood brown), and
## the things that stand or lie on the floor have their base on their origin (they would float or sink where the item rests). No shot
## is both modelled and painted. The props pass of 2026-11-04: until then those shots were all one purple bolt, and the peel, the
## sapling, the tree and the wood chip were not drawn at all.
## Run: Godot --headless --path godot --script res://tests/match_props_test.gd

const MatchProps := preload("res://scripts/match_props.gd")
const ShotPaint := preload("res://scripts/shot_paint.gd")

const BUDGET := 3000
## Props that stand or lie on the floor (base on Z = 0 as built).
const ON_FLOOR := ["banana_peel", "sapling", "tree", "wood_chip"]

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func _initialize() -> void:
	var names := {}
	for k in MatchProps.SHOTS:
		if String(MatchProps.SHOTS[k]) != "":
			names[String(MatchProps.SHOTS[k])] = true
	for k in MatchProps.ITEMS:
		names[String(MatchProps.ITEMS[k])] = true
	names["arrow"] = true
	var before := {}
	for id in Node.get_orphan_node_ids():
		before[id] = true
	var holder := Node3D.new()
	root.add_child(holder)
	for prop_name in names:
		var n := MatchProps.make(prop_name)
		check(n != null, "%s loads" % prop_name)
		if n == null:
			continue
		holder.add_child(n)
		var tris := 0
		var box := AABB()
		var first := true
		for mi: MeshInstance3D in n.find_children("*", "MeshInstance3D", true, false):
			if mi.mesh == null:
				continue
			var word := String(mi.name).get_slice("_", 0)
			check(MatchProps.PALETTE.has(word), "%s's part %s has a colour (word '%s')" % [prop_name, mi.name, word])
			check(mi.material_override != null, "%s's part %s is coloured" % [prop_name, mi.name])
			var xf := Transform3D.IDENTITY
			var up: Node = mi
			while up != null and up != n:
				xf = (up as Node3D).transform * xf
				up = up.get_parent()
			var b := xf * mi.get_aabb()
			box = b if first else box.merge(b)
			first = false
			for s in mi.mesh.get_surface_count():
				var arrays := mi.mesh.surface_get_arrays(s)
				var idx = arrays[Mesh.ARRAY_INDEX]
				tris += (idx.size() if idx != null else (arrays[Mesh.ARRAY_VERTEX] as PackedVector3Array).size()) / 3
		check(not first, "%s has meshes" % prop_name)
		check(tris <= BUDGET, "%s within %d triangles (%d)" % [prop_name, BUDGET, tris])
		if prop_name in ON_FLOOR:
			var h := maxf(box.size.y, 0.001)
			check(absf(box.position.y) <= 0.02 * h + 0.005, "%s stands on its origin (lowest point %.3f)" % [prop_name, box.position.y])
		print("%-13s tris %4d  size %s" % [prop_name, tris, box.size])
	for k in MatchProps.SHOTS:
		check(not ShotPaint.PAINTED.has(k), "shot %s is not both modelled and painted" % k)
	# Every painted kind draws without error, flying and bursting.
	var mi := MeshInstance3D.new()
	mi.mesh = ImmediateMesh.new()
	holder.add_child(mi)
	var kinds := {}
	for k in ShotPaint.PAINTED:
		kinds[ShotPaint.PAINTED[k]] = int(k.y)
	for kind in kinds:
		for bursting in [false, true]:
			mi.set_meta("paint", "")
			ShotPaint.draw(mi, kind, Vector3.ZERO, Vector2(0.3, -0.1), 1.0, 6, bursting, kinds[kind])
			check((mi.mesh as ImmediateMesh).get_surface_count() == 1, "painted %s (bursting %s) draws" % [kind, bursting])
	# Slots as main.gd keeps them: built under a node in the tree, used, then freed with it.
	var slot := MatchProps.Slot.new(holder)
	for prop_name in names:
		MatchProps.use(slot, prop_name)
	MatchProps.update_shot(slot, "tree", [10, 44], Vector3(1, 2, 0.3), Vector2(0.075, -0.1), 1.0, 20, 100.0, 0)
	MatchProps.update_item(slot, 6, 30, 0, 1.0, false)
	holder.free()
	# Nothing is left outside the tree (the release build crashes on quit with such nodes alive: orphan_test.gd).
	var left := []
	for id in Node.get_orphan_node_ids():
		if not before.has(id):
			var n := instance_from_id(id) as Node
			if n != null and n.get_parent() == null:
				left.append("%s (%s)" % [n.name, n.get_class()])
	check(left.is_empty(), "no match prop outlives its slot (%s)" % ", ".join(left))
	print("match props test %s (%d props, %d painted kinds)" % ["FAILED" if failed else "PASSED", names.size(), kinds.size()])
	quit(1 if failed else 0)
