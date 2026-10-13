extends SceneTree
## Tiles demo shots into one strip per move for reviewing animation: every `<prefix>_mNN_<move>_fFF.png` in a folder, cropped round the
## middle of the screen (where a gallery demo's camera holds the fighter) and laid side by side.
## Run: Godot --headless --path godot --script res://tests/montage.gd -- <folder> [crop size, default 420]

func _initialize() -> void:
	var a := OS.get_cmdline_user_args()
	var dir: String = a[0]
	var crop := int(a[1]) if a.size() > 1 else 420
	var groups := {}
	for f in DirAccess.get_files_at(dir):
		var i := f.find("_m")
		if i < 0 or not f.ends_with(".png") or f.begins_with("strip_"):
			continue
		var key := f.substr(i + 1, f.rfind("_f") - i - 1)
		if not groups.has(key):
			groups[key] = []
		groups[key].append(f)
	for key in groups:
		var files: Array = groups[key]
		files.sort()
		var cell := crop / 2
		var out := Image.create(cell * files.size(), cell, false, Image.FORMAT_RGB8)
		for k in files.size():
			var img := Image.load_from_file(dir.path_join(files[k]))
			img.convert(Image.FORMAT_RGB8)
			var c := img.get_size() / 2
			var r := img.get_region(Rect2i(c.x - crop / 2, c.y - crop / 2 - crop / 8, crop, crop))
			r.resize(cell, cell, Image.INTERPOLATE_BILINEAR)
			out.blit_rect(r, Rect2i(Vector2i.ZERO, r.get_size()), Vector2i(cell * k, 0))
		out.save_png(dir.path_join("strip_%s.png" % key))
	quit(0)
