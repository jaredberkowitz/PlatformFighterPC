extends SceneTree
# How much two captures differ: -- <a> <b> [<out>]: prints the mean absolute difference (0..1) and the share of pixels that changed
# by more than 2%; with <out>, writes the difference image (brighter = more changed).
func _initialize() -> void:
	var a := OS.get_cmdline_user_args()
	var x := Image.load_from_file(a[0])
	var y := Image.load_from_file(a[1])
	x.convert(Image.FORMAT_RGB8)
	y.convert(Image.FORMAT_RGB8)
	var out := Image.create(x.get_width(), x.get_height(), false, Image.FORMAT_RGB8)
	var total := 0.0
	var changed := 0
	for j in x.get_height():
		for i in x.get_width():
			var c := x.get_pixel(i, j)
			var d := y.get_pixel(i, j)
			var e := (absf(c.r - d.r) + absf(c.g - d.g) + absf(c.b - d.b)) / 3.0
			total += e
			if e > 0.02:
				changed += 1
			out.set_pixel(i, j, Color(e * 4.0, e * 4.0, e * 4.0))
	var n := float(x.get_width() * x.get_height())
	print("mean %.4f  changed %.2f%%" % [total / n, 100.0 * changed / n])
	if a.size() > 2:
		out.save_png(a[2])
	quit()
