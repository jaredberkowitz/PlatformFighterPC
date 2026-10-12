extends SceneTree
## Numbers for judging a still (docs/GRAPHICS_FIDELITY_PLAN.md): for each PNG given after `--`, its luminance percentiles (p2, p50, p98),
## the spread p98 - p2, the mean saturation and the share of pixels brighter than 0.9. No window needed.
##   godot --headless --path godot --script res://tests/shot_stats.gd -- a.png b.png ...


func _initialize() -> void:
	for path in OS.get_cmdline_user_args():
		var img := Image.load_from_file(path)
		if img == null or img.is_empty():
			print("%s: could not load" % path)
			continue
		img.convert(Image.FORMAT_RGB8)
		# Every second pixel each way is plenty for percentiles and keeps it quick.
		var lum := PackedFloat32Array()
		var sat_sum := 0.0
		var bright := 0
		for y in range(0, img.get_height(), 2):
			for x in range(0, img.get_width(), 2):
				var c := img.get_pixel(x, y)
				var l := 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
				lum.append(l)
				sat_sum += c.s
				if l > 0.9:
					bright += 1
		lum.sort()
		var n := lum.size()
		var p := func(q: float) -> float: return lum[clampi(int(q * (n - 1)), 0, n - 1)]
		print("%s: p2 %.3f p50 %.3f p98 %.3f spread %.3f sat %.3f bright %.4f" % [path.get_file(), p.call(0.02), p.call(0.5), p.call(0.98),
			p.call(0.98) - p.call(0.02), sat_sum / n, float(bright) / n])
	quit(0)
