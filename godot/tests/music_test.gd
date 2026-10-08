extends SceneTree
## The synthesised music: every track renders, loops cleanly, is audible and does not clip; the player starts, switches and mutes.
## Run: Godot --headless --path godot --script res://tests/music_test.gd

const Music := preload("res://scripts/music.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func _initialize() -> void:
	for name in Music.NAMES:
		var t0 := Time.get_ticks_msec()
		var w: AudioStreamWAV = Music.render(name)
		var ms := Time.get_ticks_msec() - t0
		var spec: Dictionary = Music.TRACKS[name]
		var n := w.data.size() / 2
		var step := int(60.0 / float(spec.bpm) / 4.0 * Music.RATE)
		check(n == step * 16 * spec.chords.size(), "%s is exactly its bars long: %d samples" % [name, n])
		check(w.loop_mode == AudioStreamWAV.LOOP_FORWARD and w.loop_end == n, "%s loops over the whole track" % name)
		var peak := 0
		var energy := 0.0
		for i in n:
			var v := absi(w.data.decode_s16(i * 2))
			peak = maxi(peak, v)
			energy += float(v) * v
		check(peak > 15000 and peak <= 30000, "%s is audible and does not clip: peak %d" % [name, peak])
		check(sqrt(energy / n) > 2500.0, "%s has body, not just a click: rms %f" % [name, sqrt(energy / n)])
		# The end meets the start without a jump, so the loop does not click.
		var first := w.data.decode_s16(0)
		var last := w.data.decode_s16((n - 1) * 2)
		check(absi(first - last) < 12000, "%s loops without a click: %d to %d" % [name, last, first])
		var seconds := float(n) / Music.RATE
		check(seconds > 6.0 and seconds < 30.0, "%s is a sensible length: %.1f s" % [name, seconds])
		print(name, ": ", snappedf(seconds, 0.1), " s, rendered in ", ms, " ms")
	# The player.
	var music: Node = Music.new()
	music.force_in_headless = true
	root.add_child(music)
	await process_frame
	music.play("menu")
	for i in 600:
		if music.current == "menu":
			break
		await create_timer(0.02).timeout
	check(music.current == "menu" and music.player.playing, "the menu theme starts once it is rendered")
	music.play("battle")
	for i in 600:
		if music.current == "battle":
			break
		await create_timer(0.02).timeout
	check(music.current == "battle" and music.player.stream == music.tracks["battle"], "and switches to the battle theme")
	music.set_muted(true)
	check(not music.player.playing, "muting stops it")
	music.set_muted(false)
	check(music.player.playing, "and unmuting resumes it")
	music.stop()
	check(not music.player.playing and music.current == "", "stop stops")
	print("music test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
