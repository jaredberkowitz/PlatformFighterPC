extends SceneTree
## Loads the shipped content bundle through the bridge, checks it is the same roster the sim has built in, then
## plays the scripted special moves. Also checks that a bad file is refused without changing anything.
## Run: Godot --headless --path godot --script res://tests/content_e2e.gd

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func _initialize() -> void:
	if not ClassDB.class_exists("SimRunner"):
		print("bridge not loaded")
		quit(1)
		return
	var sim = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	var built_in: String = sim.content_hash()
	check(sim.content_name() == "", "the built-in roster has no bundle name")

	var base := ProjectSettings.globalize_path("res://").path_join("../content/base.pfc").simplify_path()
	var err: String = sim.load_content(base)
	check(err == "", "base.pfc loads: " + err)
	check(sim.content_name() == "Base Roster", "bundle name is " + sim.content_name())
	check(sim.content_hash() == built_in, "the file and the built-in roster are the same content")
	check(Array(sim.fighter_names()) == ["duelist", "brawler", "bruiser"], "fighter names: " + str(sim.fighter_names()))

	# A broken file is refused with a message, and the loaded roster stays.
	var bad := "user://bad.pfc"
	var f := FileAccess.open(bad, FileAccess.WRITE)
	f.store_string("bundle { schema 1 }\nfighter x { }\n")
	f.close()
	err = sim.load_content(ProjectSettings.globalize_path(bad))
	check(err != "", "a broken bundle is refused")
	check(sim.content_hash() == built_in, "content unchanged after a refused load")
	err = sim.load_content("C:/definitely/not/here.pfc")
	check(err.contains("cannot read"), "a missing file is explained: " + err)

	# Play the sword character's specials: neutral (Shield Breaker) and side (Dancing Blade).
	sim.start(7, PackedInt32Array([0, 1, 0, 1]))
	var special: int = sim.button_mask("special")
	var moved := false
	var x0: float = sim.fighter_pos(0).x
	for t in 200:
		var press := special if t % 60 == 2 else 0
		var stick_x := 127 if t >= 60 and t < 120 else 0
		sim.set_input(0, stick_x, 0, press)
		sim.tick()
		if absf(sim.fighter_pos(0).x - x0) > 1.0:
			moved = true
	check(moved, "the lunge moved the fighter")
	check(sim.frame() == 200, "200 frames ran")

	print("content e2e ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
