extends SceneTree
## Every authored prop (godot/models/props, from art/blender/make_props.py) is checked before it can reach a stage
## (docs/ART_WORKFLOW.md): it has meshes, keeps to the triangle budget, and stands on its own origin; a standing prop whose lowest point
## is above or well below its origin floats or sinks wherever the game sets it down (the owner caught floating roof props twice).
## Props that are placed by their middle (crowns, logs, ranges...) are listed in CENTRED and skip the standing check.
## Run: Godot --headless --path godot --script res://tests/prop_check_test.gd [-- --report]

const Toonify := preload("res://scripts/toonify.gd")

## Placed by their middle, or seated by the game on purpose (each with why).
const CENTRED := {
	"tree_crown_a": "the crown's middle sits on the trunk's top", "tree_crown_b": "as tree_crown_a", "tree_crown_c": "as tree_crown_a",
	"log_pole": "centred on its axis under a platform", "rope_lash": "centred on the board's front",
	"hill_dome_a": "foot carried under the waterline", "hill_dome_b": "as hill_dome_a",
	"range_meadow_far": "foot carried below the horizon", "range_meadow_near": "as range_meadow_far",
	"range_grove_far": "as range_meadow_far", "range_grove_near": "as range_meadow_far",
	"dunes_canyon_far": "foot carried below the horizon", "dunes_canyon_near": "as dunes_canyon_far",
	"windmill_tower": "stands on the game's stone drum, base at 1", "palm_crown": "sits on the trunk's top",
	"lighthouse": "its rock footing runs under the sea",
}
## The most triangles one prop may have (the cottage, the busiest, is about 8800).
const BUDGET := 10000
## A standing prop's lowest point, as a share of its height: never above its origin by more than FLOAT (it would hover), and never
## below it by more than SINK (a little sink plants it in the ground; more buries it).
const FLOAT := 0.005
const SINK := 0.1

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func _initialize() -> void:
	var report := OS.get_cmdline_user_args().has("--report")
	var count := 0
	for file in DirAccess.get_files_at("res://models/props"):
		if not file.ends_with(".glb"):
			continue
		var prop_name := file.get_basename()
		var prop := Toonify.load_prop(prop_name, false)
		check(prop != null, "%s loads" % prop_name)
		if prop == null:
			continue
		count += 1
		var box := AABB()
		var first := true
		var tris := 0
		for mi: MeshInstance3D in prop.find_children("*", "MeshInstance3D", true, false):
			if mi.mesh == null:
				continue
			var xf := Transform3D.IDENTITY
			var up: Node = mi
			while up != null and up != prop:
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
		var h := maxf(box.size.y, 0.001)
		var low := box.position.y
		if report:
			print("%-18s tris %5d  height %7.2f  lowest %7.3f (%.1f%% of height)" % [prop_name, tris, h, low, 100.0 * low / h])
		if not CENTRED.has(prop_name):
			check(low <= FLOAT * h, "%s does not float (lowest point %.3f above its origin, height %.2f)" % [prop_name, low, h])
			check(low >= -SINK * h, "%s is not buried (lowest point %.3f, height %.2f)" % [prop_name, low, h])
		prop.free()
	check(count > 0, "props found")
	print("prop check test %s (%d props)" % ["FAILED" if failed else "PASSED", count])
	quit(1 if failed else 0)
