extends SceneTree
## The synthesised sound effects: every one is made, audible, in range and a sensible length, and the sound watcher plays the right effects for
## what happened to a fighter between two frames.
## Run: Godot --headless --path godot --script res://tests/sfx_test.gd

const Sfx := preload("res://scripts/sfx.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func snap(state: String, percent := 0.0, stocks := 3, platform := 0, jumps := 1, tumble := false) -> Dictionary:
	return {"state": state, "percent": percent, "stocks": stocks, "platform": platform, "jumps": jumps, "tumble": tumble}


func _initialize() -> void:
	for n in Sfx.NAMES:
		var w: AudioStreamWAV = Sfx.build(n)
		var seconds := float(w.data.size() / 2) / Sfx.RATE
		check(seconds >= 0.04 and seconds <= 1.0, "%s has a sensible length: %f s" % [n, seconds])
		var peak := 0
		for i in range(0, w.data.size(), 2):
			peak = maxi(peak, absi(w.data.decode_s16(i)))
		check(peak > 3000 and peak <= 30000, "%s is audible and not clipping: peak %d" % [n, peak])
	var holder := Node.new()
	root.add_child(holder)
	await process_frame
	var sfx: Node = Sfx.of(holder)
	await process_frame
	check(sfx.streams.size() == Sfx.NAMES.size(), "the node made every effect")
	check(Sfx.of(holder) == sfx, "and there is only one")
	check(sfx.watch(0, snap("Idle"), snap("Attack")) == ["whoosh"], "starting an attack swooshes")
	check(sfx.watch(0, snap("Attack"), snap("Attack")).is_empty(), "carrying on does not")
	check(sfx.watch(1, snap("Idle", 0.0), snap("Hitstun", 6.0)) == ["hit_light"], "a small hit is a light thud")
	check(sfx.watch(1, snap("Idle", 0.0), snap("Hitstun", 18.0)) == ["hit_heavy"], "a big hit is a heavy one")
	check(sfx.watch(0, snap("Idle"), snap("JumpSquat")) == ["jump"], "a jump")
	check(sfx.watch(0, snap("Airborne", 0.0, 3, -1, 1), snap("Airborne", 0.0, 3, -1, 0)) == ["jump"], "a double jump")
	check(sfx.watch(0, snap("Airborne", 0.0, 3, -1), snap("Landing", 0.0, 3, 0)) == ["land"], "a landing")
	check(sfx.watch(0, snap("Idle"), snap("Shield")) == ["shield"], "raising the shield")
	check(sfx.watch(0, snap("Airborne", 0.0, 3, -1), snap("Idle", 0.0, 2, 0)).has("ko"), "losing a stock")
	check(sfx.watch(0, {}, snap("Idle")).is_empty(), "no earlier frame, no sound")
	sfx.toggle_mute()
	sfx.play("blip")
	var any_playing := false
	for p in sfx.voices:
		any_playing = any_playing or p.playing
	check(not any_playing, "muted means silent")
	sfx.toggle_mute()
	print("sfx test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
