extends SceneTree
## Photographs a fighter from the front, three-quarters, the side and the back, as the concept sheet shows it, for judging the base body
## against art/concept/base_body_turnaround.jpg (needs a window, so run it without --headless).
## Run: Godot --path godot --script res://tests/turnaround_shot.gd -- --out=<file.png> [--look=N] [--neutral] [--bare]
##   --look=N    the default look of fighter N (0 by default)
##   --neutral   every surface in one plain grey cel material and no face: shape only, with the shading taken out of the comparison
##   --tpose     arms raised straight out to the sides, the hands open (for judging limb and hand shapes without overlap)
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
	var tpose := false
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
		elif a == "--tpose":
			tpose = true
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
	for turn in ([0.0, 0.8, PI / 2.0] if tpose else [0.0, 0.8, PI / 2.0, PI]):
		var pv := Preview.new(Vector2i(560, 720) if tpose else Vector2i(400, 720), false, 8.6 if tpose else 6.4)
		pv.spin = false
		pv.facing_bias = turn
		row.add_child(pv)
		pv.set_loadout(l)
		if neutral:
			_neutral(pv)
	if tpose:
		await create_timer(0.3).timeout
		for pv in row.get_children():
			_tpose(pv.view)
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


## Raises both arms straight out to the sides and opens the hands, the clip stopped, so every limb and hand is seen whole.
func _tpose(v) -> void:
	if v.anim != null:
		v.anim.stop()
	var sk: Skeleton3D = v.skeleton
	if sk == null:
		return
	sk.reset_bone_poses()
	for side in ["L", "R"]:
		var b := sk.find_bone("armU." + side)
		if b < 0:
			continue
		var g := sk.get_bone_global_pose(b)
		var hand := sk.find_bone("hand." + side)
		var tip := sk.get_bone_global_pose(hand).origin if hand >= 0 else g.origin + Vector3(0, -1, 0)
		var now := (tip - g.origin).normalized()
		var want := Vector3(1.0 if (tip - g.origin).x > 0.0 else -1.0, -0.05, 0.0).normalized()
		var turn := Quaternion(now, want)
		var parent := sk.get_bone_parent(b)
		var pg := sk.get_bone_global_pose(parent) if parent >= 0 else Transform3D.IDENTITY
		var new_g := Transform3D(Basis(turn) * g.basis, g.origin)
		sk.set_bone_pose(b, pg.affine_inverse() * new_g)
	v._clench(false, false)
	v._follow_bones()
