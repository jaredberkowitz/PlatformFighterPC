extends SceneTree
## Replays through the bridge: a recorded match plays back to the same result, made fighters replay, tampering and training edits
## are caught, seeking works, and the Replays screen and the match scene's replay mode work.
## Run: Godot --headless --path godot --script res://tests/replay_test.gd

const Roster := preload("res://scripts/roster.gd")
const Replays := preload("res://scripts/replays.gd")
const Loadout := preload("res://scripts/loadout.gd")

var failed := false
var saved_paths: Array = []


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


func entry(name: String, class_id: int, size: int) -> Dictionary:
	var e := Roster.neutral_entry(name, class_id, Loadout.default_for(0))
	e.size = size
	return e


func scripted(frame: int, player: int) -> Array:
	var x := 127 if (frame / 30 + player) % 2 == 0 else -127
	var buttons := 0
	if frame % 45 == 10:
		buttons |= 1
	if frame % 29 == 5:
		buttons |= 2
	return [x, 0, buttons]


## Plays a short local match (a clock of 4 seconds) between a made fighter and a built-in one; returns the sim.
func play_match(frames: int) -> Node:
	var sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	var made := entry("Big Test", 1, 8)
	var loaded: Dictionary = sim.load_match_fighters(Roster.spec_bytes(made), Roster.spec_bytes(Roster.builtins()[0]))
	check(loaded.error == "", "the fighters load: " + str(loaded.error))
	sim.set_players(2)
	sim.set_match_rules(3, 4)
	sim.start(77, PackedInt32Array(loaded.chars))
	for f in frames:
		var a := scripted(f, 0)
		var b := scripted(f, 1)
		sim.set_input(0, a[0], a[1], a[2])
		sim.set_input(1, b[0], b[1], b[2])
		sim.tick()
	return sim


func _initialize() -> void:
	if not ClassDB.class_exists("SimRunner"):
		print("bridge not loaded")
		quit(1)
		return
	var bytes := _bridge()
	await _scene(bytes)
	_four(bytes)
	_stage()
	_list(bytes)
	for p in saved_paths:
		Replays.delete(p)
	print("replay test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


func _bridge() -> PackedByteArray:
	var sim = play_match(300)
	check(sim.winner() != -1, "the clock ended the match: %d" % sim.winner())
	var profiles := [Roster.profile_bytes(entry("Big Test", 1, 8)), Roster.profile_bytes(Roster.builtins()[0])]
	var bytes: PackedByteArray = sim.replay_bytes(profiles[0], profiles[1])
	check(bytes.size() > 100, "the match was recorded: %d bytes" % bytes.size())
	var info: Dictionary = sim.replay_peek(bytes)
	check(info.ok and info.frames == 240 and info.playable and info.stocks == 3 and info.time_limit == 4, "the file describes itself: " + str(info))
	check(info.winner == sim.winner(), "with the winner")
	check(Roster.parse_profile(info.cosmetics0).name == "Big Test", "and the names")
	var end_positions := [sim.fighter_pos(0), sim.fighter_pos(1)]
	var end_percent := [sim.fighter_percent(0), sim.fighter_percent(1)]

	var watcher = ClassDB.instantiate("SimRunner")
	root.add_child(watcher)
	check(watcher.replay_load(bytes) == "", "it loads")
	check(watcher.replay_length() == 240, "with its length")
	check(watcher.replay_verify(), "and verifies against its recorded ending")
	var played := 0
	while watcher.replay_tick():
		played += 1
	check(played == 240, "it plays every frame: %d" % played)
	check(watcher.winner() == sim.winner(), "to the same winner")
	check(watcher.fighter_pos(0) == end_positions[0] and watcher.fighter_pos(1) == end_positions[1], "and the same positions")
	check(watcher.fighter_percent(0) == end_percent[0] and watcher.fighter_percent(1) == end_percent[1], "and damage")
	check(watcher.fighter_scale(0) > 1.1, "the made fighter is big in the replay: %f" % watcher.fighter_scale(0))

	watcher.replay_seek(100)
	check(watcher.match_frame() == 100 and watcher.winner() == -1, "seeking goes to a frame")
	while watcher.replay_tick():
		pass
	check(watcher.fighter_pos(0) == end_positions[0] and watcher.winner() == sim.winner(), "and playing on reaches the same end")
	watcher.replay_seek(5000)
	check(watcher.match_frame() == 240, "seeking past the end stops at the end")

	# A changed input still loads but no longer verifies.
	var tampered := bytes.duplicate()
	tampered[tampered.size() - 40] = (tampered[tampered.size() - 40] + 77) % 256
	tampered[tampered.size() - 41] = (tampered[tampered.size() - 41] + 33) % 256
	tampered[tampered.size() - 90] = (tampered[tampered.size() - 90] + 99) % 256
	var other = ClassDB.instantiate("SimRunner")
	root.add_child(other)
	check(other.replay_load(tampered) == "" and not other.replay_verify(), "a tampered replay does not verify")
	check(other.replay_load(bytes.slice(0, bytes.size() - 5)) != "", "a cut-short file is refused")
	check(other.replay_load(PackedByteArray([1, 2, 3])) != "", "so is garbage")
	check(not other.replay_peek(PackedByteArray([1, 2, 3])).ok, "and peeking at it says so")

	# Training edits make a match unreproducible, so it is not recorded.
	var edited = ClassDB.instantiate("SimRunner")
	root.add_child(edited)
	edited.set_match_rules(3, 4)
	edited.start(5, PackedInt32Array([0, 1]))
	edited.debug_set_percent(0, 50.0)
	for f in 300:
		edited.tick()
	check(edited.replay_bytes(PackedByteArray(), PackedByteArray()).is_empty(), "a match edited in training is not recorded")
	var plain = ClassDB.instantiate("SimRunner")
	root.add_child(plain)
	plain.set_match_rules(3, 4)
	plain.start(5, PackedInt32Array([0, 1]))
	for f in 300:
		plain.tick()
	check(not plain.replay_bytes(PackedByteArray(), PackedByteArray()).is_empty(), "a plain base-roster match is recorded")
	for n in [sim, watcher, other, edited, plain]:
		n.queue_free()
	return bytes


func _scene(bytes: PackedByteArray) -> void:
	Roster.session = {"from_menu": true, "replay": bytes}
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	for i in 8:
		await process_frame
	await create_timer(1.5).timeout
	check(main.replay_mode, "the match scene enters replay mode")
	check(main.names[0] == "Big Test" and main.names[1] == "Duelist", "with the players' names: " + str(main.names))
	var frame: int = main.sim.match_frame()
	check(frame > 0 and frame <= 240, "and plays: frame %d" % frame)
	main._unhandled_key_input(key(KEY_SPACE))
	var paused_at: int = main.sim.match_frame()
	await create_timer(0.3).timeout
	check(main.sim.match_frame() == paused_at, "Space pauses")
	main._unhandled_key_input(key(KEY_RIGHT))
	check(main.sim.match_frame() == mini(paused_at + 300, 240), "Right jumps ahead")
	main._unhandled_key_input(key(KEY_R))
	check(main.sim.match_frame() == 0, "R goes back to the start")
	main._unhandled_key_input(key(KEY_UP))
	check(main.replay_speed == 2.0, "Up doubles the speed")
	main.queue_free()
	await process_frame


## A four-player match records, replays and verifies.
func _four(_bytes: PackedByteArray) -> void:
	var sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	var specs: Array[PackedByteArray] = []
	var profiles: Array[PackedByteArray] = []
	for i in 4:
		var e := entry("Four %d" % (i + 1), i % 2, 3 + i)
		specs.append(Roster.spec_bytes(e))
		profiles.append(Roster.profile_bytes(e))
	var loaded: Dictionary = sim.load_match_roster(specs)
	check(loaded.error == "" and loaded.chars.size() == 4, "four fighters load: " + str(loaded))
	sim.set_match_rules(2, 5)
	sim.start(31, PackedInt32Array(loaded.chars))
	check(sim.fighter_in_roster(3) and sim.fighter_in_roster(0), "four players take part")
	for f in 400:
		for p in 4:
			var a := scripted(f, p)
			sim.set_input(p, a[0], a[1], a[2])
		sim.tick()
	check(sim.winner() != -1, "the clock ended it: %d" % sim.winner())
	var bytes: PackedByteArray = sim.replay_bytes_roster(profiles)
	var info: Dictionary = sim.replay_peek(bytes)
	check(info.ok and info.players == 4 and info.frames == 300, "the record knows four players: " + str(info))
	check(Roster.parse_profile(info.cosmetics3).name == "Four 4", "and their names")
	var watcher = ClassDB.instantiate("SimRunner")
	root.add_child(watcher)
	check(watcher.replay_load(bytes) == "" and watcher.replay_verify(), "it verifies")
	check(watcher.fighter_in_roster(3), "and plays back with four")
	sim.queue_free()
	watcher.queue_free()


## A match on another stage: the stage is the simulation's, it is recorded, and the replay plays on it.
func _stage() -> void:
	var sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	check(sim.stage_count() >= 4 and sim.stage_name(0) == "Meadow" and sim.stage_name(2) == "Flat Island", "the stages are listed")
	check(sim.stage_preview(1).size() == 5, "a stage preview lists its platforms and the blast zone")
	sim.set_match_stage(2)
	var specs: Array[PackedByteArray] = [Roster.spec_bytes(Roster.builtins()[0]), Roster.spec_bytes(Roster.builtins()[1])]
	var loaded: Dictionary = sim.load_match_roster(specs)
	check(loaded.error == "" and sim.platform_count() == 1, "stage 3 has no platforms: %d" % sim.platform_count())
	sim.set_match_rules(3, 3)
	sim.start(4, PackedInt32Array(loaded.chars))
	for f in 200:
		sim.set_input(0, 127 if (f / 30) % 2 == 0 else -127, 0, 0)
		sim.tick()
	var bytes: PackedByteArray = sim.replay_bytes(PackedByteArray(), PackedByteArray())
	check(sim.replay_peek(bytes).stage == 2, "the record names its stage")
	var watcher = ClassDB.instantiate("SimRunner")
	root.add_child(watcher)
	check(watcher.replay_load(bytes) == "" and watcher.platform_count() == 1 and watcher.replay_verify(), "and replays on it")
	sim.set_match_stage(99)
	sim.queue_free()
	watcher.queue_free()


func _list(bytes: PackedByteArray) -> void:
	var path := Replays.save(bytes)
	check(path != "" and FileAccess.file_exists(path), "a replay is saved to the replay folder")
	saved_paths.append(path)
	var screen: Control = load("res://replays.tscn").instantiate()
	root.add_child(screen)
	check(screen.buttons.size() >= 1, "the Replays screen lists it")
	var found := false
	for b in screen.buttons:
		if b.text.contains("Big Test") and b.text.contains("Duelist"):
			found = true
	check(found, "with the players' names")
	screen.queue_free()
