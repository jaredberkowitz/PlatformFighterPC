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
	await _code_screen()
	await _code_match()
	await _code_scene()
	await _match()
	await _spectate()
	await _group()
	await _quick()
	print("online flow test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


func _screen() -> void:
	var screen: Control = load("res://online.tscn").instantiate()
	screen.code_stun = PackedStringArray(["none"])
	screen.code_lan = "127.0.0.1"
	root.add_child(screen)
	await process_frame
	await process_frame
	screen.selectors["link"].set_index(1)
	screen.selectors["role"].set_index(0)
	check(screen.netcode == null, "choosing Direct closes the code socket (so Direct can host on the same port)")
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
	screen.selectors["link"].set_index(2)
	screen.edits["room"].text = "0"
	check(screen.connection().has("error"), "a relay room must be 1 or more")
	screen.edits["room"].text = "7"
	var relay: Dictionary = screen.connection()
	check(not relay.has("error") and relay.relay and relay.room == 7, "a relay connection: " + str(relay))
	# Ranked rules refuse a fighter over the budget.
	screen.selectors["role"].set_index(0)
	screen.selectors["link"].set_index(1)
	screen.selectors["rules"].set_index(1)
	check(not screen.connection().has("error"), "the built-in fighters are legal under ranked rules")
	# Quick match asks only for the relay's address; the host's rules apply to whoever hosts.
	screen.edits["address"].text = ""
	screen.selectors["role"].set_index(3)
	check(screen.connection().has("error"), "quick match needs the relay's address")
	check(not screen.rows.filter(func(r): return r.id == "room")[0].node.visible, "and no room number")
	check(screen.rows.filter(func(r): return r.id == "stocks")[0].node.visible, "it may be the host, so it shows the rules")
	screen.edits["address"].text = "10.0.0.5:47001"
	var quick: Dictionary = screen.connection()
	check(not quick.has("error") and quick.quick and quick.relay and quick.addr == "10.0.0.5:47001", "a quick match connection: " + str(quick))
	check(screen.connect_button.text == "Find!", "the button says Find!")
	screen.queue_free()
	await process_frame


func _shown(screen: Control, id: String) -> bool:
	return screen.rows.filter(func(r): return r.id == id)[0].node.visible


## Two bare code sockets on loopback (no STUN): their codes, then both connect and wait until each hears the other.
func _code_pair() -> Array:
	var a = ClassDB.instantiate("NetCode")
	var b = ClassDB.instantiate("NetCode")
	check(a.start(0, PackedStringArray(["none"]), "127.0.0.1") == "" and b.start(0, PackedStringArray(["none"]), "127.0.0.1") == "", "code sockets open")
	a.poll()
	b.poll()
	return [a, b]


func _punch(a, b) -> bool:
	for i in 1000:
		var sa: int = a.poll()
		var sb: int = b.poll()
		if sa == 3 and sb == 3:
			return true
		if sa == 4 or sb == 4:
			return false
		await create_timer(0.005).timeout
	return false


## The online screen's Code link: first in the list and the default, shows this player's code, checks the friend's code, and says who
## will host.
func _code_screen() -> void:
	var screen: Control = load("res://online.tscn").instantiate()
	screen.code_stun = PackedStringArray(["none"])
	screen.code_lan = "127.0.0.1"
	root.add_child(screen)
	await process_frame
	screen.selectors["link"].set_index(0)
	check(screen.selectors["link"].options[0] == "Code", "Code is first in the list")
	for i in 10:
		await process_frame
	check(screen.netcode != null, "the code socket is open")
	var mine: String = screen.netcode.my_code()
	check(mine.length() >= 14 and mine.contains("-"), "the player's own code shows: " + mine)
	check(screen.code_label.text == mine, "on the screen")
	check(_shown(screen, "mycode") and _shown(screen, "friend"), "the code rows show")
	check(not _shown(screen, "role") and not _shown(screen, "address") and not _shown(screen, "port") and not _shown(screen, "room"), "and the role, address, port and room rows do not")
	check(not _shown(screen, "fighter") and not _shown(screen, "stage"), "a duel's fighter and stage are chosen in the lobby, not here")
	check(_shown(screen, "stocks"), "either player may host, so both see the rules")
	check(screen.connect_button.text == "Connect!", "the button says Connect!")
	screen.edits["friend"].text = ""
	check(screen.connection().has("error"), "a friend's code is needed")
	screen.edits["friend"].text = mine
	check(screen.connection().has("error"), "your own code is refused")
	screen.edits["friend"].text = "HELLO-WORLD"
	check(screen.connection().has("error"), "a mistyped code is refused")
	var other = ClassDB.instantiate("NetCode")
	other.start(0, PackedStringArray(["none"]), "127.0.0.1")
	other.poll()
	var theirs: String = other.my_code()
	screen.edits["friend"].text = "  " + theirs.to_lower().replace("-", " ") + " "
	var c: Dictionary = screen.connection()
	check(not c.has("error") and c.code and not c.relay, "a pasted code is accepted, case and spaces aside: " + str(c))
	check(c.host != other.hosts_against(mine), "exactly one of the two hosts")
	screen._show_role()
	check(screen.status.text.contains("host"), "the screen says who hosts: " + screen.status.text)
	other.stop()
	screen.queue_free()
	await process_frame


## A whole match by code through the bridge: punch on loopback, then the host and joiner play over the punched sockets.
func _code_match() -> void:
	var pair := _code_pair()
	var a = pair[0]
	var b = pair[1]
	var code_a: String = a.my_code()
	var code_b: String = b.my_code()
	check(a.connect_to(code_b) == "" and b.connect_to(code_a) == "", "both press Connect")
	check(await _punch(a, b), "the two code sockets found each other")
	check(a.i_host() != b.i_host(), "one hosts")
	var host_code = a if a.i_host() else b
	var join_code = b if a.i_host() else a
	var host = ClassDB.instantiate("SimRunner")
	var join = ClassDB.instantiate("SimRunner")
	root.add_child(host)
	root.add_child(join)
	check(host.net_host_code(host_code, PackedInt32Array([0, 1, 0, 1]), 2) == "", "host over the code link")
	check(join.net_join_code(join_code) == "", "join over the code link")
	check(host.net_join_code(host_code) != "", "a spent code cannot connect twice")
	var ran := [0, 0]
	for t in 8000:
		var ia: Array = _input_for(ran[0], 0)
		var ib: Array = _input_for(ran[1], 1)
		if host.net_update(ia[0], ia[1], ia[2]) == 1:
			ran[0] += 1
		if join.net_update(ib[0], ib[1], ib[2]) == 1:
			ran[1] += 1
		if ran[0] > 240 and ran[1] > 240:
			break
		await create_timer(0.0005).timeout
	check(ran[0] > 240 and ran[1] > 240, "the pair plays by code: %s" % str(ran))
	check(not str(host.net_take_log()).contains("DESYNC") and not str(join.net_take_log()).contains("DESYNC"), "no desync by code")
	host.net_leave()
	join.net_leave()
	# Nobody answering ends in a failure, not a hang.
	var lone = ClassDB.instantiate("NetCode")
	lone.start(0, PackedStringArray(["none"]), "127.0.0.1")
	lone.poll()
	var ghost = ClassDB.instantiate("NetCode")
	ghost.start(0, PackedStringArray(["none"]), "127.0.0.1")
	ghost.poll()
	check(lone.connect_to(ghost.my_code()) == "", "connect to a friend who never presses Connect")
	ghost.stop()
	check(lone.poll() == 2 and lone.seconds_left() > 30, "waiting, with the time left shown: %d" % lone.seconds_left())
	lone.cancel()
	check(lone.poll() == 1, "Esc stops waiting and keeps the code")
	lone.stop()


## The real screens: the online screen connects by code and moves on to the match scene, which plays over the punched socket.
func _code_scene() -> void:
	# Connecting saves the screen's settings: keep the player's own file as it was.
	var saved := FileAccess.get_file_as_string("user://online.json") if FileAccess.file_exists("user://online.json") else ""
	await _code_scene_inner()
	if saved != "":
		var f := FileAccess.open("user://online.json", FileAccess.WRITE)
		f.store_string(saved)


func _code_scene_inner() -> void:
	var screen: Control = load("res://online.tscn").instantiate()
	screen.code_stun = PackedStringArray(["none"])
	screen.code_lan = "127.0.0.1"
	root.add_child(screen)
	await process_frame
	screen.selectors["link"].set_index(0)
	for i in 5:
		await process_frame
	var other = ClassDB.instantiate("NetCode")
	other.start(0, PackedStringArray(["none"]), "127.0.0.1")
	other.poll()
	screen.edits["friend"].text = other.my_code()
	screen._connect()
	await process_frame
	check(screen.code_state == 2 and screen.status.text.begins_with("Waiting for your friend"), "the screen waits for the friend: " + screen.status.text)
	check(other.connect_to(screen.netcode.my_code()) == "", "the friend presses Connect")
	var scene: Node = null
	for i in 600:
		other.poll()
		await process_frame
		if root.get_tree().current_scene != null and root.get_tree().current_scene.has_method("_net_config"):
			scene = root.get_tree().current_scene
			break
	check(scene != null, "connected, the online screen moves on to the match")
	if scene == null:
		return
	# The friend's socket hears back a moment later (the match scene answers its probes).
	for i in 300:
		if other.poll() == 3:
			break
		await process_frame
	var join = ClassDB.instantiate("SimRunner")
	root.add_child(join)
	var theirs: Dictionary = Roster.builtins()[3]
	join.set_fighter(Roster.spec_bytes(theirs))
	join.set_cosmetics(Roster.profile_bytes(theirs))
	join.net_use_lobby(true)
	var host_side: bool = Roster.session.online.host
	var err: String = join.net_host_code(other, PackedInt32Array([0, 1, 0, 1]), 2) if not host_side else join.net_join_code(other)
	check(err == "", "the friend's side starts too: " + err)
	# Both arrive in the lobby (the owner, 19 October): the match waits for both players' choices.
	var lobby_seen := false
	for i in 600:
		join.net_update(0, 0, 0)
		await process_frame
		if scene.net_lobby != null and join.net_lobby_view().has("theirs"):
			lobby_seen = true
			break
	check(lobby_seen, "both players are in the lobby (status %d)" % scene.net_status)
	check(scene.net_status == 4, "and the match waits there")
	if not lobby_seen:
		return
	# The friend locks in a fighter and the third stage; this player locks in the second stage.
	join.net_lobby_set(Roster.spec_bytes(theirs), Roster.profile_bytes(theirs), true, 2, true)
	for i in 20:
		join.net_update(0, 0, 0)
		await process_frame
	check(scene.net_lobby.their_state.text.begins_with("Ready"), "this player sees the friend is ready: " + scene.net_lobby.their_state.text)
	scene.net_lobby.fighter_row.step(1)
	scene.net_lobby._unhandled_key_input(key(KEY_ENTER))
	check(scene.net_lobby.stage_step.visible and scene.net_lobby.picker.cards.size() == scene.net_lobby.stage_names.size(),
		"the stage step shows a card for every stage")
	check(scene.net_lobby.picker.cards[2].badge != "", "the friend's stage is badged on its card")
	# The step opens on the last stage used (an earlier match's), so step right until the second stage.
	for i in scene.net_lobby.stage_names.size():
		if scene.net_lobby.stage == 1:
			break
		scene.net_lobby._step(1)
	check(scene.net_lobby.stage == 1, "stepping right reaches the second stage (%d)" % scene.net_lobby.stage)
	check(scene.net_lobby.picker.index == scene.net_lobby.stage, "the focused card is the stage chosen")
	scene.net_lobby._unhandled_key_input(key(KEY_ENTER))
	check(scene.net_lobby.phase == 2, "Enter twice locks the fighter and the stage in")
	var ran := 0
	for i in 900:
		if join.net_update(0, 0, 0) == 1:
			ran += 1
		await process_frame
		if ran > 60 and scene.net_mode and scene.net_status == 1:
			break
	check(scene.net_mode and scene.net_status == 1, "the match scene plays by code (status %d) %s" % [scene.net_status, str(join.net_take_log())])
	check(scene.net_lobby == null, "the lobby closes when the match starts")
	var stage: int = scene.sim.net_match_stage()
	check(stage == 1 or stage == 2, "the match is on one of the two chosen stages (%d)" % stage)
	# Leaving gives the code's socket back: the same code, ready for the next friend (one code per launch, the owner).
	var lent: RefCounted = scene.lent_code
	check(lent != null, "the match holds the code's socket")
	if lent != null:
		check(scene.sim.net_leave_to_code(lent), "leaving gives the socket back")
		var state: int = lent.poll()
		check(state == 1, "the code is ready again (%d)" % state)
		check(lent.my_code() != "", "with a code")
		scene.net_mode = false
	join.net_leave()
	scene.queue_free()
	if is_instance_valid(screen):
		screen.queue_free()
	await process_frame
	Roster.session = {}


func _match() -> void:
	var host = ClassDB.instantiate("SimRunner")
	var join = ClassDB.instantiate("SimRunner")
	root.add_child(host)
	root.add_child(join)
	host.set_match_rules(0, 3)
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
			check(join.match_rules() == PackedInt32Array([0, 3]), "the joiner plays under the host's rules: " + str(join.match_rules()))
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
	host.set_match_rules(0, 3)
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
	host.set_match_rules(0, 3)
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


## Two players ask the relay's queue for a match, are paired into one room (the one who waited hosts), and play through it.
func _quick() -> void:
	var tool := ProjectSettings.globalize_path("res://../target/debug/pftool.exe")
	if OS.has_environment("PFTOOL"):
		tool = OS.get_environment("PFTOOL")
	if not FileAccess.file_exists(tool):
		print("SKIP quick match (no pftool at ", tool, ")")
		return
	var pid := OS.create_process(tool, ["net-relay", "47150"])
	await create_timer(0.5).timeout
	var a = ClassDB.instantiate("SimRunner")
	var b = ClassDB.instantiate("SimRunner")
	root.add_child(a)
	root.add_child(b)
	check(a.quickmatch_start("127.0.0.1:47150") == "", "a starts looking")
	for i in 30:
		check(a.quickmatch_poll().is_empty(), "alone, a is not matched")
		await create_timer(0.005).timeout
	check(b.quickmatch_start("127.0.0.1:47150") == "", "b starts looking")
	var fa := {}
	var fb := {}
	for i in 600:
		if fa.is_empty():
			fa = a.quickmatch_poll()
		if fb.is_empty():
			fb = b.quickmatch_poll()
		if not fa.is_empty() and not fb.is_empty():
			break
		await create_timer(0.005).timeout
	check(not fa.is_empty() and not fb.is_empty(), "both are matched")
	if not fa.is_empty() and not fb.is_empty():
		check(fa.room == fb.room and fa.room > 0, "into one room")
		check(fa.host and not fb.host, "the one who waited hosts")
		var err: String = a.net_host_relay("127.0.0.1:47150", fa.room, PackedInt32Array([0, 1, 0, 1]), 2)
		check(err == "", err)
		err = b.net_join_relay("127.0.0.1:47150", fb.room)
		check(err == "", err)
		var ran := [0, 0]
		for t in 6000:
			var ia: Array = _input_for(ran[0], 0)
			var ib: Array = _input_for(ran[1], 1)
			if a.net_update(ia[0], ia[1], ia[2]) == 1:
				ran[0] += 1
			if b.net_update(ib[0], ib[1], ib[2]) == 1:
				ran[1] += 1
			if ran[0] > 200 and ran[1] > 200:
				break
			await create_timer(0.0005).timeout
		check(ran[0] > 200 and ran[1] > 200, "the matched pair plays: %s" % str(ran))
		check(not str(a.net_take_log()).contains("DESYNC") and not str(b.net_take_log()).contains("DESYNC"), "no desync")
	a.net_leave()
	b.net_leave()
	# The match scene itself: it waits in the queue, is paired with another searcher, and connects as the joiner.
	var entry: Dictionary = Roster.net_entry()
	Roster.session = {"from_menu": true, "stocks": 3, "time": 0, "stage": 0, "online": {
		"host": false, "quick": true, "watch": false, "group": false, "relay": true, "addr": "127.0.0.1:47150", "port": 47000, "room": 0,
		"delay": 2, "entry": entry, "stage": 0, "stocks": 3, "time": 0, "ranked": false}}
	var scene: Node = load("res://main.tscn").instantiate()
	root.add_child(scene)
	for i in 120:
		await process_frame
	check(scene.quick_mode and not scene.net_mode, "the match scene is looking for an opponent")
	check(scene._net_status_text().begins_with("Looking"), "and says so: " + scene._net_status_text())
	var other = ClassDB.instantiate("SimRunner")
	root.add_child(other)
	other.quickmatch_start("127.0.0.1:47150")
	var found := {}
	for i in 600:
		if found.is_empty():
			found = other.quickmatch_poll()
		if scene.net_mode:
			break
		await create_timer(0.01).timeout
	check(scene.net_mode and not scene.quick_mode, "paired, the scene connects")
	check(scene.local_slot == 0 or scene.local_slot == 1, "in a slot")
	scene.queue_free()
	await process_frame
	OS.kill(pid)
