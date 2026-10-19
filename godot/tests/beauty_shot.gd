extends SceneTree
## A posed close-up of the default cast for judging the models against the owner's reference (a lit group shot, not a T-pose): the four
## fighters side by side, turned three-quarters toward the middle, framed on the head and torso, each holding a moment of its idle clip
## with its own face (the character judge's round 8). Needs a window, so run it without --headless.
## Run: Godot --path godot --script res://tests/beauty_shot.gd -- --out=<file.png> [--clip=idle] [--at=0.6]

const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")


func _initialize() -> void:
	var out := "user://beauty.png"
	var clip := "idle"
	var at := 0.6
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
		elif a.begins_with("--clip="):
			clip = a.substr(7)
		elif a.begins_with("--at="):
			at = float(a.substr(5))
	var bg := ColorRect.new()
	bg.color = Color(0.86, 0.8, 0.72)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_child(bg)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	root.add_child(row)
	var views: Array = []
	for k in 4:
		var pv := Preview.new(Vector2i(320, 560), false, 4.6)
		pv.spin = false
		# Turned toward the middle of the group, as a lineup on a bench is.
		pv.facing_bias = 0.55 if k < 2 else -0.55
		row.add_child(pv)
		pv.set_loadout(Loadout.default_for(k))
		pv.body_at = Vector3(0, 1.45, 0)
		pv.body_from = Vector3(0, 1.7, 5.4)
		views.append(pv)
	await create_timer(0.3).timeout
	for pv in views:
		var v = pv.view
		if v.anim != null and v.anim.has_animation(clip):
			v.anim.play(clip)
			v.anim.seek(at * v.anim.current_animation_length, true)
			v.anim.speed_scale = 0.0
	await create_timer(0.8).timeout
	root.get_viewport().get_texture().get_image().save_png(out)
	quit(0)
