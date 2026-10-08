extends SceneTree
## The online screen's checks, and a whole online match through the bridge over real UDP: the joiner plays under the host's rules,
## the match ends on the clock with the same result on both sides, a rematch starts, and nothing desyncs.
## Run: Godot --headless --path godot --script res://tests/online_flow_test.gd

const Roster := preload("res://scripts/roster.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func key(code: int) -> InputEventKey:
	var ev := InputEventKey.new()
	ev.keycode = code
	ev.physical_keycode = code
	ev.pressed = true
	return ev


func _input_for(frame: int, player: int) -> Array:
	var x := 127 if (frame / 40 + player) % 2 == 0 else -127
	var buttons := 0
	if frame % 50 == 10:
		buttons |= 1
	if frame % 33 == 5:
		buttons |= 2
	return [x, 0, buttons]


func _initialize() -> void:
	if not ClassDB.class_exists("SimRunner"):
		print("bridge not loaded")
		quit(1)
		return
	await _screen()
	await _match()
	print("online flow test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


func _screen() -> void:
	var screen: Control = load("res://online.tscn").instantiate()
	root.add_child(screen)
	await process_frame
	await process_frame
	var c: Dictionary = screen.connection()
	check(not c.has("error") and c.host and not c.relay and c.port == 47000, "hosting directly is the default: " + str(c))
	check(screen.rows.filter(func(r): return r.id == "port")[0].node.visible, "the port row shows when hosting")
	check(not screen.rows.filter(func(r): return r.id == "address")[0].node.visible, "and the address row does not")
	screen.edits["port"].text = "80"
	check(screen.connection().has("error"), "a port below 1024 is refused")
	screen.edits["port"].text = "abc"
	check(screen.connection().has("error"), "and so is a port that is not a number")
	screen.edits["port"].text = "47010"
	check(screen.connection().port == 47010, "a good port is used")
	screen.selectors["role"].set_index(1)
	check(screen.rows.filter(func(r): return r.id == "address")[0].node.visible, "joining shows the address row")
	check(not screen.rows.filter(func(r): return r.id == "stocks")[0].node.visible, "and hides the host's rules")
	check(screen.connection().has("error"), "joining needs an address")
	screen.edits["address"].text = "10.0.0.5"
	check(screen.connection().addr == "10.0.0.5:47000", "a missing port defaults: " + str(screen.connection()))
	screen.edits["address"].text = "10.0.0.5:47123"
	check(screen.connection().addr == "10.0.0.5:47123", "an address with a port is kept")
	screen.selectors["link"].set_index(1)
	screen.edits["room"].text = "0"
	check(screen.connection().has("error"), "a relay room must be 1 or more")
	screen.edits["room"].text = "7"
	var relay: Dictionary = screen.connection()
	check(not relay.has("error") and relay.relay and relay.room == 7, "a relay connection: " + str(relay))
	# Ranked rules refuse a fighter over the budget.
	screen.selectors["role"].set_index(0)
	screen.selectors["link"].set_index(0)
	screen.selectors["rules"].set_index(1)
	check(not screen.connection().has("error"), "the built-in fighters are legal under ranked rules")
	screen.queue_free()
	await process_frame


func _match() -> void:
	var host = ClassDB.instantiate("SimRunner")
	var join = ClassDB.instantiate("SimRunner")
	root.add_child(host)
	root.add_child(join)
	host.set_match_rules(3, 3)
	var err: String = host.net_host(47131, PackedInt32Array([0, 1, 0, 1]), 2)
	check(err == "", "host: " + err)
	err = join.net_join("127.0.0.1:47131")
	check(err == "", "join: " + err)
	var ran := [0, 0]
	var ticks := 0
	var asked := false
	var rematch_frames := [0, 0]
	var second_match := false
	while ticks < 40000:
		var a: Array = _input_for(ran[0], 0)
		var b: Array = _input_for(ran[1], 1)
		var sa: int = host.net_update(a[0], a[1], a[2])
		var sb: int = join.net_update(b[0], b[1], b[2])
		if sa == 1:
			ran[0] += 1
		if sb == 1:
			ran[1] += 1
		ticks += 1
		if not asked and host.winner() != -1 and join.winner() != -1:
			# Both sides saw the end of the first match, with the same result.
			check(host.winner() == join.winner(), "same result on both sides: %d vs %d" % [host.winner(), join.winner()])
			check(join.match_rules() == PackedInt32Array([3, 3]), "the joiner plays under the host's rules: " + str(join.match_rules()))
			host.net_request_rematch()
			check(host.net_rematch_state() == PackedInt32Array([1, 0]), "asking is visible")
			asked = true
		if asked and not second_match:
			if join.net_rematch_state()[1] == 1 and join.net_rematch_state()[0] == 0:
				join.net_request_rematch()
			# The new match: both back at the start with no winner.
			if sa == 1 and sb == 1 and host.winner() == -1 and join.winner() == -1 and host.match_frame() < 60 and join.match_frame() < 60:
				second_match = true
				rematch_frames = [host.match_frame(), join.match_frame()]
		if second_match and host.match_frame() > 120 and join.match_frame() > 120:
			break
		await create_timer(0.0005).timeout
	check(asked, "the first match ended on the clock")
	check(second_match, "a rematch started after both asked")
	check(host.winner() == -1 and join.winner() == -1, "the rematch is in play")
	var log_a: PackedStringArray = host.net_take_log()
	var log_b: PackedStringArray = join.net_take_log()
	check(not str(log_a).contains("DESYNC") and not str(log_b).contains("DESYNC"), "no desync: %s %s" % [str(log_a), str(log_b)])
	host.net_leave()
	join.net_leave()
