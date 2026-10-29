extends SceneTree
## The sword swing (round 8, the owner's pick 2026-10-29: the sweep, round 5's A/B decided). The sweep is the game's own for every weapon
## class's cuts, with no toggle: F10 in a match no longer changes it and nothing is kept in user://animation.cfg. Presentation only.
## Run: Godot --headless --path godot --script res://tests/swing_toggle_test.gd

const FighterViewScript := preload("res://scripts/fighter_view.gd")
const Bindings := preload("res://scripts/bindings.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func _initialize() -> void:
	var view = FighterViewScript.new()
	for cls in [0, 9, 6, 13, 5]:
		check(view.sweeping({"class": cls, "char": cls, "move_name": "ftilt"}), "class %d's forward tilt sweeps" % cls)
	check(not view.sweeping({"class": 0, "char": 0, "move_name": "nair"}), "the neutral air's windmill keeps its own path")
	check(not view.sweeping({"class": 1, "char": 1, "move_name": "ftilt"}), "an unarmed class does not sweep a blade")
	check(not ("sword_sweep" in view), "no snap and sweep switch is left")
	check(not Bindings.RESERVED.has(KEY_F10), "F10 is free")
	view.free()
	print("swing default test %s" % ("FAILED" if failed else "PASSED"))
	quit(1 if failed else 0)
