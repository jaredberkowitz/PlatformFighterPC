extends SceneTree
## Photographs the four default fighters side by side, close up, with every face and shirt, for checking the character art against
## the reference (needs a window, so run it without --headless).
## Run: Godot --path godot --script res://tests/lineup_shot.gd -- --out=<folder>

const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")


func _initialize() -> void:
	var out := "user://"
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
	var bg := ColorRect.new()
	bg.color = Color(0.93, 0.87, 0.78)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_child(bg)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	root.add_child(row)
	var previews := []
	for p in 4:
		var pv := Preview.new(Vector2i(480, 760), false, 5.6)
		pv.spin = false
		pv.facing_bias = [0.45, 0.15, -0.15, -0.45][p]
		row.add_child(pv)
		pv.set_loadout(Loadout.default_for(p))
		previews.append(pv)
	await create_timer(1.0).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("lineup.png"))
	# Every face, then every shirt, on the first fighter.
	for f in Loadout.FACES.size() + 1:
		var l: RefCounted = Loadout.default_for(f % 4)
		l.face = f % Loadout.FACES.size()
		l.shirt = f % Loadout.SHIRTS.size()
		l.glasses = 0
		previews[f % 4].set_loadout(l)
		if f == Loadout.FACES.size():
			previews[f % 4].set_expression(Loadout.HURT)
	await create_timer(0.6).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("lineup_faces.png"))
	quit(0)
