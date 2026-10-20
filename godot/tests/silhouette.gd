extends SceneTree
## Prints a figure's silhouette span per row: for each column band, the leftmost and rightmost pixel that differs from the background.
## For docs/art/characters/REFERENCE.md: the reference images and our T-pose (turnaround_shot.gd --tpose) measured alike.
## Run: Godot --headless --path godot --script res://tests/silhouette.gd -- --img=<file> --bands=x0:x1,x0:x1 [--step=5] [--tol=0.12] [--bg=r,g,b]

func _initialize() -> void:
	var path := ""
	var bands := []
	var step := 5
	var tol := 0.12
	var bg := Color(-1, 0, 0)
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--img="):
			path = a.substr(6)
		elif a.begins_with("--bands="):
			for b in a.substr(8).split(","):
				var p := b.split(":")
				bands.append([int(p[0]), int(p[1])])
		elif a.begins_with("--step="):
			step = int(a.substr(7))
		elif a.begins_with("--tol="):
			tol = float(a.substr(6))
		elif a.begins_with("--bg="):
			var c := a.substr(5).split(",")
			bg = Color(float(c[0]), float(c[1]), float(c[2]))
	var im := Image.load_from_file(path)
	if bg.r < 0.0:
		bg = im.get_pixel(5, 5)
	print("size ", im.get_size(), " bg ", bg)
	for y in range(0, im.get_height(), step):
		var line := "%4d" % y
		var any := false
		for b in bands:
			var l := -1
			var r := -1
			for x in range(b[0], b[1]):
				var c := im.get_pixel(x, y)
				# The background is a flat colour, lighter at the top: compare against the same row's band edge too.
				var edge := im.get_pixel(b[0], y)
				var d := minf(_dist(c, bg), _dist(c, edge))
				if d > tol:
					if l < 0:
						l = x
					r = x
			if l >= 0:
				any = true
				line += "  %4d-%4d w%4d" % [l, r, r - l + 1]
			else:
				line += "  %15s" % "-"
		if any:
			print(line)
	quit(0)


func _dist(a: Color, b: Color) -> float:
	return Vector3(a.r - b.r, a.g - b.g, a.b - b.b).length()
