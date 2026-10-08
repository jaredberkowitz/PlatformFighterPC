extends SceneTree
## Online play with made fighters through the bridge, over real UDP on this machine: each side brings a fighter, both build the
## same match, the fighters have their own sizes and hitbox scales, and both agree on the game. Also ranked refusals, and the
## point budget in the creator and select screens.
## Run: Godot --headless --path godot --script res://tests/online_fighters_test.gd

const Roster := preload("res://scripts/roster.gd")
const Loadout := preload("res://scripts/loadout.gd")

var failed := false
var port := 47150


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


func entry(name: String, class_id: int, size: int, speed: int, jump: int, weight: int) -> Dictionary:
	var e := Roster.neutral_entry(name, class_id, Loadout.default_for(0))
	e.size = size
	e.speed = speed
	e.jump = jump
	e.weight = weight
	return e


## Hosts one fighter against another over UDP and returns [host, joiner, statuses] after the match has run a while.
func play(host_entry: Dictionary, join_entry: Dictionary, ranked: bool) -> Array:
	var a = ClassDB.instantiate("SimRunner")
	var b = ClassDB.instantiate("SimRunner")
	root.add_child(a)
	root.add_child(b)
	port += 2  # a host also listens for spectators on the next port
	a.set_cosmetics(Roster.profile_bytes(host_entry))
	a.set_fighter(Roster.spec_bytes(host_entry))
	a.set_ranked(ranked)
	b.set_cosmetics(Roster.profile_bytes(join_entry))
	b.set_fighter(Roster.spec_bytes(join_entry))
	check(a.net_host(port, PackedInt32Array([0, 1, 0, 1]), 2) == "", "host starts")
	check(b.net_join("127.0.0.1:%d" % port) == "", "joiner starts")
	var status := [0, 0]
	var ran := 0
	for t in 900:
		var x := 127 if (t / 40) % 2 == 0 else -127
		var buttons := 2 if t % 37 == 5 else 0
		status[0] = a.net_update(x, 0, buttons)
		status[1] = b.net_update(-x, 0, buttons)
		if status[0] == 3 or status[1] == 3:
			break
		if status[0] == 1:
			ran += 1
		if ran >= 240 and status[1] == 1:
			break
		OS.delay_msec(1)
	return [a, b, status]


func _initialize() -> void:
	if not ClassDB.class_exists("ContentEditor"):
		print("bridge not loaded")
		quit(1)
		return
	for e in Roster.saved():
		if e.slug.begins_with("zz_test"):
			Roster.delete(e.slug)
	_budget_rules()
	_online()
	await _screens()
	for e in Roster.saved():
		if e.slug.begins_with("zz_test"):
			Roster.delete(e.slug)
	print("online fighters test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


func _budget_rules() -> void:
	check(Roster.budget() == 20, "the budget is 20 points")
	check(Roster.points(entry("x", 0, 5, 5, 5, 5)) == 20 and Roster.ranked_legal(entry("x", 0, 5, 5, 5, 5)), "neutral spends exactly the budget")
	check(not Roster.ranked_legal(entry("x", 0, 9, 9, 9, 9)), "all nines are over it")
	check(Roster.ranked_legal(entry("x", 0, 2, 9, 8, 1)), "a trade-off build is legal")
	var back: Dictionary = Roster.from_json(Roster.to_json(entry("zz_test Casual", 0, 9, 9, 9, 9)))
	check(not back.casual, "casual defaults to off")
	var casual := entry("zz_test Casual", 0, 9, 9, 9, 9)
	casual.casual = true
	check(Roster.from_json(Roster.to_json(casual)).casual, "and is kept in the file when set")
	# Profile bytes: look and name travel together and read back; garbage gives the default.
	var e := entry("zz_test Sam", 1, 3, 4, 5, 6)
	var profile := Roster.parse_profile(Roster.profile_bytes(e))
	check(profile.name == "zz_test Sam" and profile.look.equals(e.look), "a profile round-trips")
	check(Roster.parse_profile(PackedByteArray([200, 1, 2])).name == "", "a bad profile gives no name")
	check(Roster.parse_profile(PackedByteArray()).look.equals(Loadout.default_for(0)), "and the default look")


func _online() -> void:
	var big := entry("zz_test Big", 0, 9, 4, 4, 4)
	var small := entry("zz_test Small", 1, 1, 7, 6, 6)
	var r: Array = play(big, small, false)
	var a = r[0]
	var b = r[1]
	check(r[2][0] == 1 and r[2][1] == 1, "both sides are running: %s" % str(r[2]))
	check(a.fighter_scale(0) > 1.25 and a.fighter_scale(1) < 0.75, "the host sees a big fighter 1 and a small fighter 2: %f %f" % [a.fighter_scale(0), a.fighter_scale(1)])
	check(is_equal_approx(a.fighter_scale(0), b.fighter_scale(0)) and is_equal_approx(a.fighter_scale(1), b.fighter_scale(1)), "and the joiner agrees")
	check(Array(a.fighter_names()).size() == 5 and Array(b.fighter_names()).size() == 5, "both built the same five-fighter roster (three built in, two made)")
	check(a.content_hash() == b.content_hash(), "with the same content hash")
	var specs: Array = a.net_fighter_specs()
	check(specs.size() == 2 and specs[0] == Roster.spec_bytes(big) and specs[1] == Roster.spec_bytes(small), "the specs were exchanged")
	var shared: int = mini(a.frame(), b.frame())
	check(shared > 100, "a match was played (%d frames)" % shared)
	var theirs := Roster.parse_profile(a.net_their_cosmetics(), 1)
	check(theirs.name == "zz_test Small", "the host got the joiner's name: " + theirs.name)

	# Built-in against made also works, and built-in against built-in is the old behaviour.
	var duelist: Dictionary = Roster.builtins()[0]
	r = play(duelist, small, false)
	check(r[2][0] == 1 and r[2][1] == 1, "a built-in fighter against a made one plays")
	r = play(Roster.builtins()[1], duelist, false)
	check(r[2][0] == 1 and r[2][1] == 1 and Array(r[0].fighter_names()).size() == 3, "two built-in fighters keep the base roster")

	# Ranked rules refuse an over-budget fighter, from either side; a casual match accepts it.
	r = play(big, small, true)
	check(r[2][0] == 3 or r[2][1] == 3, "ranked refuses a fighter over the budget: %s" % str(r[2]))
	var greedy := entry("zz_test Greedy", 0, 9, 9, 9, 9)
	r = play(small, greedy, true)
	check(r[2][0] == 3 or r[2][1] == 3, "even when the joiner brings it")
	var lines: PackedStringArray = r[0].net_take_log() + r[1].net_take_log()
	check(lines.size() > 0 and "not allowed" in "\n".join(lines), "with a message that says why: " + str(lines))
	r = play(entry("zz_test A", 0, 4, 6, 5, 5), entry("zz_test B", 1, 6, 4, 5, 5), true)
	check(r[2][0] == 1 and r[2][1] == 1, "two legal fighters play under ranked rules")


func _screens() -> void:
	# ---- Creator: the budget blocks raising stats, casual lifts it ----
	var creator: Control = load("res://creator.tscn").instantiate()
	root.add_child(creator)
	await process_frame
	await process_frame
	var speed = creator.stat_rows["speed"]
	speed.step(1)
	check(speed.value == 5, "a stat cannot go up when the 20 points are spent")
	creator.stat_rows["size"].step(-1)
	speed.step(1)
	check(speed.value == 6 and Roster.points(creator.current_entry()) == 20, "lowering one stat frees a point for another")
	speed.step(1)
	check(speed.value == 6, "and only one")
	creator.selectors["rules"].set_index(1)
	speed.step(1)
	speed.step(1)
	check(speed.value == 8, "casual rules have no limit")
	check(creator.points_tag.text.contains("CASUAL"), "and the points tag says so: " + creator.points_tag.text)
	creator.selectors["rules"].set_index(0)
	creator.name_edit.text = "zz_test Over"
	creator._finish()
	check(creator.status.text.contains("budget"), "saving an over-budget ranked fighter is refused: " + creator.status.text)
	check(Roster.name_problem("zz_test Over") == "" and not FileAccess.file_exists(Roster.path_of("zz_test_over")), "and nothing was saved")
	creator.queue_free()

	# ---- Select: ranked rules keep out fighters over the budget ----
	var greedy := entry("zz_test Greedy2", 0, 9, 9, 9, 9)
	greedy.casual = true
	Roster.save(greedy)
	await process_frame
	var select: Control = load("res://select.tscn").instantiate()
	root.add_child(select)
	await process_frame
	await process_frame
	var index := -1
	for i in select.entries.size():
		if select.entries[i].slug == "zz_test_greedy2":
			index = i
	check(index >= 2, "the casual fighter is in the grid")
	select.cursor[0] = index
	select._lock(0)
	check(select.locked[0], "under casual rules it can be picked")
	select._unlock(0)
	select._toggle_ranked()
	check(select.ranked, "ranked rules are on")
	select.cursor[0] = index
	select._lock(0)
	check(not select.locked[0], "under ranked rules it cannot")
	check(select.status.text.contains("budget"), "and the screen says why: " + select.status.text)
	select.cursor[0] = 0
	select._lock(0)
	select._toggle_ranked()
	select._toggle_ranked()
	check(select.locked[0], "a legal fighter stays picked when ranked rules are switched on")
	select.queue_free()
