extends RefCounted
## Brings an imported model into the cel style: every surface's plain material (as art/blender/import_generated.py exports them: a flat
## colour, or a palette texture) is swapped for the cel material of the same colour (FighterView.toon), with the ink outline. Presentation
## only. `load_prop` loads one of the props in godot/models/props/ ready to place.

const FighterView := preload("res://scripts/fighter_view.gd")
const PROPS := "res://models/props/"


## Swaps the materials of every mesh under `node`. Returns the number of surfaces changed.
static func apply(node: Node, outline := true) -> int:
	var changed := 0
	if node is MeshInstance3D:
		var mi := node as MeshInstance3D
		if mi.mesh != null:
			for i in mi.mesh.get_surface_count():
				# (The model's own material, not one swapped in before, so applying twice changes nothing.)
				var m := mi.mesh.surface_get_material(i)
				if m == null:
					m = mi.get_active_material(i)
				var colour := Color.WHITE
				var texture: Texture2D = null
				if m is BaseMaterial3D:
					colour = (m as BaseMaterial3D).albedo_color
					texture = (m as BaseMaterial3D).albedo_texture
				mi.set_surface_override_material(i, FighterView.toon(colour, outline, texture))
				changed += 1
	for c in node.get_children():
		changed += apply(c, outline)
	return changed


## A prop from godot/models/props/<name>.glb in the cel style, or null if there is no such prop.
static func load_prop(prop_name: String, outline := true) -> Node3D:
	var path := PROPS + prop_name + ".glb"
	if not ResourceLoader.exists(path):
		return null
	var scene := load(path) as PackedScene
	if scene == null:
		return null
	var node := scene.instantiate() as Node3D
	apply(node, outline)
	return node
