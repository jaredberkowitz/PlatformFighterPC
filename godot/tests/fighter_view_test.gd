extends SceneTree
## What a fighter's model shows: the sword fighter's blade is there in every move (it never vanishes), the brawler never has one,
## the brawler uses the long-limbed rig and the sword fighter the normal one.
## Run: Godot --headless --path godot --script res://tests/fighter_view_test.gd

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func snap(char_id: int, state: String, move: String, frame: int) -> Dictionary:
	return {"state": state, "facing": 1, "vel": Vector2.ZERO, "platform": -1, "fast_fall": false, "invuln": 0, "ledge_invuln": 0, "frame": frame,
		"shield": 1.0, "percent": 0.0, "move_name": move, "move_tip": Vector3(1.6, 1.2, 0.6), "move_timing": PackedInt32Array([40, 8, 14]), "charge": 0,
		"hitlag": 0, "tumble": false, "launch_pending": false, "char": char_id, "class": char_id, "reach": 3.0, "state_frame": 6}


func _initialize() -> void:
	var sword := FighterView.new()
	var fists := FighterView.new()
	var hammer := FighterView.new()
	root.add_child(sword)
	root.add_child(fists)
	root.add_child(hammer)
	sword.build(0, Loadout.default_for(0))
	fists.build(1, Loadout.default_for(1))
	hammer.build(2, Loadout.default_for(2))
	await process_frame
	await process_frame
	var moves := ["jab", "ftilt", "utilt", "dtilt", "dash attack", "fsmash", "usmash", "dsmash", "nair", "fair", "bair", "uair", "dair",
		"neutral special", "side special", "up special", "down special", "grab", "dash grab", "pummel", "forward throw", "back throw",
		"up throw", "down throw"]
	var frame := 0
	for m in moves:
		frame += 1
		sword.apply(Vector3.ZERO, snap(0, "Attack", m, frame), 1.0 / 60.0)
		check(sword.blade_pivot.visible, "the sword fighter keeps its blade in %s" % m)
		fists.apply(Vector3.ZERO, snap(1, "Attack", m, frame), 1.0 / 60.0)
		check(not fists.blade_pivot.visible, "the brawler never has a blade in %s" % m)
		hammer.apply(Vector3.ZERO, snap(2, "Attack", m, frame), 1.0 / 60.0)
		check(hammer.blade_pivot.visible, "the bruiser always has its maul in %s" % m)
		check(hammer.hammer_parts[0].visible and not hammer.blade_parts[0].visible, "and it is a hammer, not a blade, in %s" % m)
		check(sword.blade_parts[0].visible and not sword.hammer_parts[0].visible, "the sword fighter's weapon is a blade in %s" % m)
	for state in ["Idle", "Walk", "Run", "Shield", "Hitstun", "Airborne", "Roll", "LedgeHang"]:
		sword.apply(Vector3.ZERO, snap(0, state, "", frame), 1.0 / 60.0)
		check(sword.blade_pivot.visible, "the blade is there while %s" % state)
	check(not hammer.long_limbs, "the bruiser has the ordinary rig")
	check(not sword.long_limbs and fists.long_limbs, "the brawler uses the long-limbed rig, the sword fighter does not")
	print("fighter view test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
