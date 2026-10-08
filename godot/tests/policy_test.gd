extends SceneTree
## The name policy in the game: your own fighter names, and names that arrive from other players.
## Run: Godot --headless --path godot --script res://tests/policy_test.gd

const Roster := preload("res://scripts/roster.gd")
const Loadout := preload("res://scripts/loadout.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func _initialize() -> void:
	if not ClassDB.class_exists("ContentEditor"):
		print("bridge not loaded")
		quit(1)
		return
	check(Roster.name_problem("Big Bertha") == "", "an ordinary name is fine")
	check(Roster.name_problem("Mario") != "", "someone else's trademark is refused")
	check(Roster.name_problem("M4r10 Kart") != "", "even disguised")
	check(Roster.name_problem("Super Smash Bros") != "", "a franchise name too")
	check(Roster.name_problem("Wolf") == "", "a plain word is fine")
	# Names that arrive over the network are cleaned before they are shown.
	var look: RefCounted = Loadout.default_for(0)
	var make := func(text: String) -> PackedByteArray:
		var lb: PackedByteArray = look.to_bytes()
		var out := PackedByteArray([lb.size()])
		out.append_array(lb)
		out.append_array(text.to_utf8_buffer())
		return out
	check(Roster.parse_profile(make.call("Duelist")).name == "Duelist", "a good name comes through")
	check(Roster.parse_profile(make.call("  Big   Bob ")).name == "Big Bob", "spaces are tidied")
	check(Roster.parse_profile(make.call("Evil\u0007\u001b[31mRed")).name == "Evil31mRed", "control characters are dropped: " + Roster.parse_profile(make.call("Evil\u0007\u001b[31mRed")).name)
	check(Roster.parse_profile(make.call("Mario")).name == "", "a blocked name becomes empty (the caller's default is used)")
	check(Roster.parse_profile(make.call("y".repeat(60))).name.length() <= 24, "a long name is cut")
	print("policy test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
