extends SceneTree
## Photographs a fighter from the front, three-quarters, the side and the back, as the concept sheet shows it, for judging the base body
## against art/concept/base_body_turnaround.jpg (needs a window, so run it without --headless).
## Run: Godot --path godot --script res://tests/turnaround_shot.gd -- --out=<file.png> [--look=N] [--neutral] [--bare]
##   --look=N    the default look of fighter N (0 by default)
##   --neutral   every surface in one plain grey cel material and no face: shape only, with the shading taken out of the comparison
##   --bare      no hat, glasses, neckwear or shirt: the body alone, as on the sheet
##   --hat=N, --neck=N, --glasses=N, --shirt=N, --gear=N   wear that piece (loadout.gd's lists)

const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")
const FighterViewScript := preload("res://scripts/fighter_view.gd")


func _initialize() -> void:
	var out := "user://turnaround.png"
	var look := 0
	var neutral := false
	var bare := false
	var wear := {}
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
		elif a.begins_with("--look="):
			look = int(a.substr(7))
		elif a == "--neutral":
			neutral = true
		elif a == "--bare":
			bare = true
		else:
			for slot in ["hat", "neck", "glasses", "shirt", "gear"]:
				if a.begins_with("--%s=" % slot):
					wear[slot] = int(a.substr(slot.length() + 3))
	var bg := ColorRect.new()
	bg.color = Color(0.62, 0.62, 0.64)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_child(bg)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	root.add_child(row)
	var l: RefCounted = Loadout.default_for(look)
	if bare:
		l.hat = 0
		l.glasses = 0
		l.neck = 0
		l.shirt = 0
	for slot in wear:
		l.set(slot, wear[slot])
	for turn in [0.0, 0.8, PI / 2.0, PI]:
		var pv := Preview.new(Vector2i(400, 720), false, 6.4)
		pv.spin = false
		pv.facing_bias = turn
		row.add_child(pv)
		pv.set_loadout(l)
		if neutral:
			_neutral(pv)
	await create_timer(1.0).timeout
	root.get_viewport().get_texture().get_image().save_png(out)
	quit(0)


## Every mesh under `n` in one plain grey cel material, the face hidden.
func _neutral(n: Node) -> void:
	var grey: Material = FighterViewScript.toon(Color(0.78, 0.78, 0.8))
	for c in n.find_children("*", "GeometryInstance3D", true, false):
		if c is MeshInstance3D:
			(c as MeshInstance3D).material_override = grey
		elif c is GeometryInstance3D:
			(c as GeometryInstance3D).visible = false
