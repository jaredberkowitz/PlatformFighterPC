extends SceneTree
## Compares two stills pixel by pixel (docs/GRAPHICS_FIDELITY_PLAN.md: a change meant to keep the picture must keep it). Prints the share
## of pixels that differ at all, the mean difference and the largest (0..255).
##   godot --headless --path godot --script res://tests/image_diff.gd -- a.png b.png


func _initialize() -> void:
	var args := OS.get_cmdline_user_args()
	var a := Image.load_from_file(args[0])
	var b := Image.load_from_file(args[1])
	a.convert(Image.FORMAT_RGB8)
	b.convert(Image.FORMAT_RGB8)
	if a.get_size() != b.get_size():
		print("different sizes")
		quit(1)
		return
	var differ := 0
	var total := 0.0
	var worst := 0.0
	for y in a.get_height():
		for x in a.get_width():
			var d := a.get_pixel(x, y) - b.get_pixel(x, y)
			var m := maxf(absf(d.r), maxf(absf(d.g), absf(d.b))) * 255.0
			if m > 0.5:
				differ += 1
			total += m
			worst = maxf(worst, m)
	var n := a.get_width() * a.get_height()
	print("%s vs %s: %.4f of pixels differ, mean %.3f, worst %.0f" % [args[0].get_file(), args[1].get_file(), float(differ) / n, total / n, worst])
	quit(0)
