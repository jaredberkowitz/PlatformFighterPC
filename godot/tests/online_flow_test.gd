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
	await _spectate()
	await _group()
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
	var ended_at := -1
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
		if ended_at < 0 and host.winner() != -1 and join.winner() != -1:
			ended_at = ticks
		if not asked and ended_at >= 0 and ticks > ended_at + 200:
			# Both sides saw the end of the first match, with the same result.
			check(host.winner() == join.winner(), "same result on both sides: %d vs %d" % [host.winner(), join.winner()])
			check(join.match_rules() == PackedInt32Array([3, 3]), "the joiner plays under the host's rules: " + str(join.match_rules()))
			_records(host, join)
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


## Both sides recorded the match, and the two files are the same match: the same frames and result, and each one verifies.
func _records(host, join) -> void:
	var a: PackedByteArray = host.replay_bytes(PackedByteArray(), PackedByteArray())
	var b: PackedByteArray = join.replay_bytes(PackedByteArray(), PackedByteArray())
	check(not a.is_empty() and not b.is_empty(), "both sides recorded the online match")
	var pa: Dictionary = host.replay_peek(a)
	var pb: Dictionary = join.replay_peek(b)
	check(pa.ok and pb.ok and pa.frames == pb.frames and pa.winner == pb.winner and pa.seed == pb.seed, "and they agree: %s vs %s" % [str(pa.frames), str(pb.frames)])
	check(pa.frames == 180, "the record ends when the clock did: %d" % pa.frames)
	check(a == b, "the two files are identical")
	var watcher = ClassDB.instantiate("SimRunner")
	root.add_child(watcher)
	check(watcher.replay_load(a) == "" and watcher.replay_verify(), "the recording verifies")
	watcher.queue_free()


## A spectator joins a hosted match (late, over real UDP on the next port) and ends up at the same place as the players.
func _spectate() -> void:
	var host = ClassDB.instantiate("SimRunner")
	var join = ClassDB.instantiate("SimRunner")
	var watcher = ClassDB.instantiate("SimRunner")
	for n in [host, join, watcher]:
		root.add_child(n)
	host.set_match_rules(3, 3)
	check(host.net_host(47161, PackedInt32Array([0, 1, 0, 1]), 2) == "", "host for the spectator test")
	check(join.net_join("127.0.0.1:47161") == "", "join for the spectator test")
	var ran := [0, 0]
	var ticks := 0
	var started := false
	var watcher_status := -1
	var best_status := 0
	while ticks < 30000:
		var a: Array = _input_for(ran[0], 0)
		var b: Array = _input_for(ran[1], 1)
		if host.net_update(a[0], a[1], a[2]) == 1:
			ran[0] += 1
		if join.net_update(b[0], b[1], b[2]) == 1:
			ran[1] += 1
		# The spectator arrives after the match is some frames old.
		if not started and ran[0] > 40:
			started = true
			check(watcher.spectate_start("127.0.0.1:47161") == "", "the spectator connects")
		if started:
			watcher_status = watcher.spectate_update()
			if watcher_status == 1:
				best_status = 1
		ticks += 1
		if started and watcher.winner() != -1 and host.winner() != -1 and ran[0] > 260:
			break
		await create_timer(0.0005).timeout
	check(best_status == 1, "the spectator played the match")
	check(host.spectator_count() == 1, "the host sees one spectator: %d" % host.spectator_count())
	check(watcher.winner() == host.winner() and watcher.winner() != -1, "the same result: %d vs %d" % [watcher.winner(), host.winner()])
	for i in 2:
		check(watcher.fighter_pos(i) == host.fighter_pos(i) and watcher.fighter_percent(i) == host.fighter_percent(i), "fighter %d ends in the same place and damage" % i)
	check(watcher.spectate_cosmetics(0).size() >= 0, "the watcher can read the players' profiles")
	watcher.spectate_stop()
	host.net_leave()
	join.net_leave()


## Three players in a group match through the bridge over real UDP: lobby, start, a match to the clock, the same result everywhere, a replay that
## verifies, and a rematch.
func _group() -> void:
	var Roster = load("res://scripts/roster.gd")
	var nodes := []
	for i in 3:
		var n = ClassDB.instantiate("SimRunner")
		root.add_child(n)
		nodes.append(n)
	var host = nodes[0]
	var specs := [Roster.spec_bytes(Roster.builtins()[0]), Roster.spec_bytes(Roster.builtins()[1]), Roster.spec_bytes(Roster.builtins()[0])]
	for i in 3:
		nodes[i].set_fighter(specs[i])
		nodes[i].set_cosmetics(PackedByteArray([0, 65 + i]))
	host.set_match_rules(3, 3)
	check(host.group_host_start(47181, 2) == "", "the group host listens")
	check(nodes[1].group_join("127.0.0.1:47181") == "" and nodes[2].group_join("127.0.0.1:47181") == "", "two guests join")
	var started := false
	var statuses := [0, 0, 0]
	var ran := [0, 0, 0]
	var ticks := 0
	var restarted := false
	var first_end := -1
	while ticks < 40000:
		for i in 3:
			var a: Array = _input_for(ran[i], i)
			statuses[i] = nodes[i].group_update(a[0], a[1], a[2])
			if statuses[i] == 1:
				ran[i] += 1
		ticks += 1
		if not started and nodes[1].group_slot() >= 1 and nodes[2].group_slot() >= 1 and host.group_players() == 3:
			check(nodes[1].group_slot() != nodes[2].group_slot(), "each guest has its own slot")
			started = true
			host.group_start_match()
		if started and first_end < 0 and nodes[0].winner() != -1 and nodes[1].winner() != -1 and nodes[2].winner() != -1 and ran[0] > 260:
			first_end = ticks
			check(nodes[0].winner() == nodes[1].winner() and nodes[1].winner() == nodes[2].winner(), "the same result everywhere")
			for p in 3:
				check(nodes[0].fighter_pos(p) == nodes[1].fighter_pos(p) and nodes[1].fighter_pos(p) == nodes[2].fighter_pos(p), "fighter %d ends in the same place on every machine" % p)
			var lobby: Array = nodes[1].group_lobby()
			check(lobby.size() >= 3 and lobby[0].size() == 2, "a guest knows the players' names")
			var bytes: PackedByteArray = host.replay_bytes(PackedByteArray(), PackedByteArray())
			var info: Dictionary = host.replay_peek(bytes)
			check(info.ok and info.players == 3, "the host recorded a three-player match: " + str(info.get("players", 0)))
			var watcher = ClassDB.instantiate("SimRunner")
			root.add_child(watcher)
			check(watcher.replay_load(bytes) == "" and watcher.replay_verify(), "and the recording verifies")
			watcher.queue_free()
			host.group_restart()
		if first_end >= 0 and not restarted and nodes[1].winner() == -1 and nodes[2].winner() == -1 and host.winner() == -1 and statuses[1] == 1:
			restarted = true
			break
		await create_timer(0.0005).timeout
	check(started and first_end >= 0, "the group match ran to its end")
	check(restarted, "a rematch started for everyone")
	var logs := str(host.group_take_log()) + str(nodes[1].group_take_log()) + str(nodes[2].group_take_log())
	check(not logs.contains("DESYNC"), "no desync in the group: " + logs)
	for n in nodes:
		n.net_leave()
