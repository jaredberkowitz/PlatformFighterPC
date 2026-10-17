extends SceneTree
## Turns a Material Maker export (its albedo, a seamless photo-like texture) into one of the game's tiles: a greyscale value map the
## stage colour is multiplied by (`value`), or a detail overlay centred on mid grey (`detail`), at 512px. The value is normalised (the
## 2nd to 98th percentile spread over the range), softened toward a few flat bands so it reads as painted cel shading, not a photo, and
## stored in sRGB (the toon shader's `albedo_tex` is a colour texture). Materials.gd prefers these PNGs over the SVG tiles.
## Run: Godot --headless -s tools/art/mm_tile.gd -- <albedo.png> <out.png> <value|detail> [low] [high] [bands] [band_mix]
##   value: the darkest and lightest multiplier (linear light), default 0.66 and 1.0; detail: low/high are the overlay's swing round 0.5.

const SIZE := 512


func _initialize() -> void:
	var a := OS.get_cmdline_user_args()
	if a.size() < 3:
		push_error("usage: <albedo.png> <out.png> <value|detail> [low] [high] [bands] [band_mix]")
		quit(1)
		return
	var img := Image.load_from_file(a[0])
	if img == null:
		push_error("cannot read " + a[0])
		quit(1)
		return
	img.convert(Image.FORMAT_RGB8)
	img.resize(SIZE, SIZE, Image.INTERPOLATE_LANCZOS)
	var mode: String = a[2]
	var low := float(a[3]) if a.size() > 3 else (0.66 if mode == "value" else 0.3)
	var high := float(a[4]) if a.size() > 4 else (1.0 if mode == "value" else 0.7)
	var bands := int(a[5]) if a.size() > 5 else 4
	var band_mix := float(a[6]) if a.size() > 6 else 0.45
	# Luminance, and its 2nd and 98th percentiles.
	var lum := PackedFloat32Array()
	lum.resize(SIZE * SIZE)
	for y in SIZE:
		for x in SIZE:
			var c := img.get_pixel(x, y)
			lum[y * SIZE + x] = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
	var sorted := lum.duplicate()
	sorted.sort()
	var lo := sorted[int(sorted.size() * 0.02)]
	var hi := maxf(sorted[int(sorted.size() * 0.98)], lo + 0.001)
	var out := Image.create(SIZE, SIZE, false, Image.FORMAT_L8)
	for y in SIZE:
		for x in SIZE:
			var t := clampf((lum[y * SIZE + x] - lo) / (hi - lo), 0.0, 1.0)
			# Toward flat bands, softly (a painted read: a few values, gentle edges between them).
			var banded := roundf(t * float(bands - 1)) / float(bands - 1)
			t = lerpf(t, banded, band_mix)
			var v := lerpf(low, high, t)
			if mode == "value":
				v = pow(v, 1.0 / 2.2)
			out.set_pixel(x, y, Color(v, v, v))
	out.save_png(a[1])
	print("mm_tile: %s -> %s (%s, %.2f..%.2f, %d bands)" % [a[0], a[1], mode, low, high, bands])
	quit(0)
