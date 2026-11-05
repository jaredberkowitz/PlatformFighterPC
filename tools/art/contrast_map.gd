extends SceneTree
## The contrast map (fidelity review 19, docs/art/stages/FIDELITY_REVIEW_19.md): where a frame's contrast lies, so the fighters can be
## checked against what competes with them for the eye. Each image is split into a 12 by 8 grid of blocks, and each block's contrast (the
## standard deviation of its luma) is measured. For each image it prints the six most contrasty blocks, the fighters' two blocks and the
## four under the fight, each with its rank of the 96 (1 is the most contrasty).
##
## The fighters' blocks assume the stage demo's pose at the match camera, 1920x1080 (`--demo=stage --stage=<n> --noui --shots=<dir>`):
## columns 5 and 6 of row 4 (x 0.42 to 0.58, y 0.50 to 0.625). The blocks under the fight are columns 4 to 7 of row 5. Row 3, above the
## fighters, holds their heads and the P1/P2 markers. The owner's polish guide: nothing under the fight should be the most contrasty thing
## on screen, so the eye lands on the fighters. A rank moves with the stage behind the fighters; their own contrast is theirs.
##
##   godot --headless --script tools/art/contrast_map.gd -- <image.png> [more images] [--out=<dir>]
##
## `--out=` also writes each image with its six most contrasty blocks outlined in orange and the fighters' two in cyan
## (`<dir>/<name>_contrast.png`).

const COLS := 12
const ROWS := 8
const FIGHTERS := [Vector2i(5, 4), Vector2i(6, 4)]
const UNDER := [Vector2i(4, 5), Vector2i(5, 5), Vector2i(6, 5), Vector2i(7, 5)]


func _init() -> void:
	var out_dir := ""
	var paths: Array[String] = []
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out_dir = a.substr(6)
		else:
			paths.append(a)
	if paths.is_empty():
		printerr("contrast map: give one or more images (see the script's header)")
	for p in paths:
		var img := Image.load_from_file(p)
		if img == null:
			printerr("contrast map: cannot read %s" % p)
			continue
		img.convert(Image.FORMAT_RGB8)
		var ranked := _blocks(img)
		ranked.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return float(a.contrast) > float(b.contrast))
		var rank := {}
		for i in ranked.size():
			rank[ranked[i].at] = [i + 1, float(ranked[i].contrast)]
		var leads := PackedStringArray()
		for i in 6:
			var b: Dictionary = ranked[i]
			var at: Vector2i = b.at
			leads.append("#%d (%d,%d) x %.2f y %.2f contrast %.3f" % [i + 1, at.x, at.y, (at.x + 0.5) / COLS, (at.y + 0.5) / ROWS, float(b.contrast)])
		print(p.get_file())
		print("  leads: " + " | ".join(leads))
		print("  fighters: " + _ranks(FIGHTERS, rank))
		print("  under the fight: " + _ranks(UNDER, rank))
		if out_dir != "":
			DirAccess.make_dir_recursive_absolute(out_dir)
			var file := out_dir.path_join(p.get_file().get_basename() + "_contrast.png")
			_draw(img, ranked, file)
			print("  wrote " + file)
	quit()


## Each block's contrast: {at: Vector2i(column, row), contrast}, sampled every third pixel.
func _blocks(img: Image) -> Array:
	var w := img.get_width()
	var h := img.get_height()
	var out: Array = []
	for r in ROWS:
		for c in COLS:
			var s := 0.0
			var s2 := 0.0
			var n := 0.0
			for y in range(r * h / ROWS, (r + 1) * h / ROWS, 3):
				for x in range(c * w / COLS, (c + 1) * w / COLS, 3):
					var l := img.get_pixel(x, y).get_luminance()
					s += l
					s2 += l * l
					n += 1.0
			var m := s / n
			out.append({"at": Vector2i(c, r), "contrast": sqrt(maxf(s2 / n - m * m, 0.0))})
	return out


func _ranks(blocks: Array, rank: Dictionary) -> String:
	var parts := PackedStringArray()
	for at: Vector2i in blocks:
		parts.append("(%d,%d) #%d %.3f" % [at.x, at.y, int(rank[at][0]), float(rank[at][1])])
	return " | ".join(parts)


## The image with the six most contrasty blocks outlined in orange and the fighters' two in cyan.
func _draw(img: Image, ranked: Array, file: String) -> void:
	var w := img.get_width() / COLS
	var h := img.get_height() / ROWS
	var t := maxi(img.get_width() / 400, 2)
	for i in 6:
		var at: Vector2i = ranked[i].at
		_outline(img, Rect2i(at.x * w, at.y * h, w, h), t, Color(1.0, 0.48, 0.0))
	for at: Vector2i in FIGHTERS:
		_outline(img, Rect2i(at.x * w + t * 2, at.y * h + t * 2, w - t * 4, h - t * 4), t, Color(0.0, 0.9, 1.0))
	img.save_png(file)


func _outline(img: Image, r: Rect2i, t: int, c: Color) -> void:
	img.fill_rect(Rect2i(r.position.x, r.position.y, r.size.x, t), c)
	img.fill_rect(Rect2i(r.position.x, r.end.y - t, r.size.x, t), c)
	img.fill_rect(Rect2i(r.position.x, r.position.y, t, r.size.y), c)
	img.fill_rect(Rect2i(r.end.x - t, r.position.y, t, r.size.y), c)
