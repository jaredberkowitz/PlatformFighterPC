extends SceneTree
## Photographs every expression (FaceLook.EXPRESSIONS) on one fighter, close up, six at a time, for judging the faces side by side
## (needs a window, so run it without --headless).
## Run: Godot --path godot --script res://tests/faces_shot.gd -- --out=<folder>   writes faces_0.png, faces_1.png...

const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")
const FaceLook := preload("res://scripts/face_look.gd")


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
	for p in 6:
		var pv := Preview.new(Vector2i(213, 300), false, 5.6)
		pv.spin = false
		row.add_child(pv)
		pv.framed = true
		pv.frame_face(true)
		pv.close_up = 1.0
		var l: RefCounted = Loadout.default_for(1)
		l.hat = 0
		l.glasses = 0
		pv.set_loadout(l)
		previews.append(pv)
	var names: Array = FaceLook.EXPRESSIONS.keys()
	var pages := ceili(names.size() / 6.0)
	for page in pages:
		for p in 6:
			var i := page * 6 + p
			previews[p].visible = i < names.size()
			if i < names.size():
				previews[p].set_expression({"name": names[i]})
		await create_timer(0.8).timeout
		root.get_viewport().get_texture().get_image().save_png(out.path_join("faces_%d.png" % page))
	print(names)
	quit(0)
