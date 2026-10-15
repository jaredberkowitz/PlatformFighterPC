extends SceneTree
## Measures flicker in the run (the owner's R2a report: "lighting flickers and shadow issues"): runs right for a few seconds and, every
## rendered frame, compares the picture with the last one in bands of the screen that only the streamed stage and the backdrop cover
## (the top and bottom strips, away from the fighter and the display). A steady pan changes a band smoothly; a light or a shadow that
## pops shows as a spike. Prints the spikes per band and the worst. `--no-backdrop-lights` and `--no-backdrop-shadows` turn the
## backdrop's practical lights or its cast shadows off first, to see which is to blame. Needs a window (not --headless).
## Run: Godot --path godot --script res://tests/run_flicker_probe.gd -- [--no-backdrop-lights] [--no-backdrop-shadows]

const Roster := preload("res://scripts/roster.gd")


func _band_mean(img: Image, y0: int, y1: int) -> float:
	var sum := 0.0
	var n := 0
	for y in range(y0, y1, 4):
		for x in range(0, img.get_width(), 4):
			var c := img.get_pixel(x, y)
			sum += c.r * 0.3 + c.g * 0.59 + c.b * 0.11
			n += 1
	return sum / maxf(1.0, float(n))


func _initialize() -> void:
	var args := OS.get_cmdline_user_args()
	Roster.session = {"from_menu": true, "run_entry": Roster.builtins()[0], "run_seed": 7}
	var main: Node = load("res://runner.tscn").instantiate()
	root.add_child(main)
	await create_timer(2.0).timeout
	if args.has("--no-backdrop-lights") or args.has("--no-backdrop-shadows"):
		var lights := 0
		var casters := 0
		for n in main.run_stage.backdrop.find_children("*", "", true, false):
			if n is Light3D and args.has("--no-backdrop-lights"):
				(n as Light3D).visible = false
				lights += 1
			if n is GeometryInstance3D and args.has("--no-backdrop-shadows"):
				(n as GeometryInstance3D).cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
				casters += 1
		print("turned off: %d backdrop lights, %d backdrop shadow casters" % [lights, casters])
	var total_lights := 0
	for n in main.find_children("*", "Light3D", true, false):
		total_lights += 1
		var l := n as Light3D
		print("  light %s %s shadow %s at %s" % [l.get_class(), str(main.get_path_to(l)), str(l.shadow_enabled), str(l.global_position.snapped(Vector3.ONE * 0.1))])
	print("lights in the scene: %d" % total_lights)
	main._send_key(KEY_D, true)
	await create_timer(0.5).timeout
	var h := 0
	var last := []
	var prev_img: Image = null
	var prev_d := PackedFloat32Array()
	var reversals := 0
	var spikes := [0, 0]
	var worst := [0.0, 0.0]
	var diffs := [[], []]
	for f in 240:
		await process_frame
		var img := root.get_viewport().get_texture().get_image()
		h = img.get_height()
		# Pixels that brighten and darken again (or the reverse) on consecutive frames, sharply: a pan moves edges along, a flicker
		# throws them back and forth.
		if prev_img != null:
			var d := PackedFloat32Array()
			var k := 0
			for y in range(int(h * 0.12), int(h * 0.82), 6):
				for x in range(0, img.get_width(), 6):
					var a := img.get_pixel(x, y)
					var b := prev_img.get_pixel(x, y)
					var dl := (a.r + a.g + a.b) / 3.0 - (b.r + b.g + b.b) / 3.0
					d.append(dl)
					if prev_d.size() > k and absf(dl) > 0.12 and absf(prev_d[k]) > 0.12 and signf(dl) != signf(prev_d[k]):
						reversals += 1
					k += 1
			prev_d = d
		prev_img = img
		var bands := [_band_mean(img, int(h * 0.16), int(h * 0.3)), _band_mean(img, int(h * 0.62), int(h * 0.8))]
		if not last.is_empty():
			for b in 2:
				var d := absf(bands[b] - last[b])
				diffs[b].append(d)
				worst[b] = maxf(worst[b], d)
		last = bands
	for b in 2:
		var arr: Array = diffs[b]
		arr.sort()
		var median: float = arr[arr.size() / 2]
		for d in diffs[b]:
			if d > maxf(0.01, median * 6.0):
				spikes[b] += 1
		print("band %s: median change %.4f, worst %.4f, spikes %d" % ["high" if b == 0 else "low", median, worst[b], spikes[b]])
	print("sharp back-and-forth pixels: %d over 240 frames" % reversals)
	main._send_key(KEY_D, false)
	main.queue_free()
	quit(0)
