extends SceneTree
## Photographs the built-in fighters as they are dressed (Roster.builtins), seven at a time, for judging the roster's looks and
## silhouettes side by side (needs a window, so run it without --headless).
## Run: Godot --path godot --script res://tests/roster_shot.gd -- --out=<folder>   writes roster_0.png and roster_1.png.

const Preview := preload("res://ui/preview.gd")
const Roster := preload("res://scripts/roster.gd")


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
	for p in 7:
		var pv := Preview.new(Vector2i(274, 720), false, 6.4)
		pv.spin = false
		pv.facing_bias = 0.3
		row.add_child(pv)
		previews.append(pv)
	var all: Array = Roster.builtins()
	for page in 2:
		for p in 7:
			var i := page * 7 + p
			if i < all.size():
				previews[p].set_loadout(all[i].look)
		await create_timer(1.0).timeout
		root.get_viewport().get_texture().get_image().save_png(out.path_join("roster_%d.png" % page))
	quit(0)
