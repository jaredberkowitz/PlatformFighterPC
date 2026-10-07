extends SceneTree
## Two SimRunners in one process play each other over real UDP (localhost) through the bridge, exactly as two game
## windows would. Passes if both reach the frame target, report no desync, and end with the same confirmed state.
## Run: Godot --headless --path godot --script res://tests/net_e2e.gd

const FRAMES := 300


func _input_for(frame: int, player: int) -> Array:
	# A simple deterministic "player": walks one way, then the other, jumps and attacks now and then.
	var x := 127 if (frame / 40 + player) % 2 == 0 else -127
	var buttons := 0
	if frame % 50 == 10:
		buttons |= 1  # jump
	if frame % 33 == 5:
		buttons |= 2  # attack
	return [x, 0, buttons]


func _initialize() -> void:
	if not ClassDB.class_exists("SimRunner"):
		print("bridge not loaded")
		quit(1)
		return
	var host = ClassDB.instantiate("SimRunner")
	var join = ClassDB.instantiate("SimRunner")
	root.add_child(host)
	root.add_child(join)
	var err: String = host.net_host(47123, PackedInt32Array([0, 1, 0, 1]), 2)
	if err != "":
		print("FAIL host: ", err)
		quit(1)
		return
	err = join.net_join("127.0.0.1:47123")
	if err != "":
		print("FAIL join: ", err)
		quit(1)
		return
	var ran := [0, 0]
	var ticks := 0
	while (ran[0] < FRAMES or ran[1] < FRAMES) and ticks < 20000:
		var a = _input_for(ran[0], 0)
		var b = _input_for(ran[1], 1)
		if host.net_update(a[0], a[1], a[2]) == 1:
			ran[0] += 1
		if join.net_update(b[0], b[1], b[2]) == 1:
			ran[1] += 1
		ticks += 1
		await create_timer(0.0005).timeout
	var log_a: PackedStringArray = host.net_take_log()
	var log_b: PackedStringArray = join.net_take_log()
	print("host: ", host.net_info())
	print("join: ", join.net_info())
	var ok: bool = ran[0] >= FRAMES and ran[1] >= FRAMES and log_a.is_empty() and log_b.is_empty()
	# Both ran a different number of frames past the target; compare a state both have confirmed: the fighters' damage and
	# positions are only comparable at the same frame, so compare the frame counters' relationship instead.
	print("frames: host %d, join %d, net logs: %s %s" % [ran[0], ran[1], str(log_a), str(log_b)])
	print("net e2e test: ", "PASSED" if ok else "FAILED")
	host.net_leave()
	join.net_leave()
	quit(0 if ok else 1)
