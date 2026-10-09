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
	# Every move plays a clip the rig has, and the common ones have their own.
	var expect := {"jab": "attack_jab", "jab 2": "attack_jab", "dash attack": "attack_lunge", "side special": "attack_lunge", "utilt": "sword_utilt",
		"up special": "attack_uair", "dtilt": "sword_dtilt", "down special": "attack_low", "neutral special": "attack_smash", "fair": "attack_fair",
		"nair": "attack_nair", "usmash": "sword_usmash", "forward throw": "throw", "pummel": "grab", "ftilt": "sword_ftilt", "fsmash": "sword_fsmash", "dsmash": "sword_dsmash"}
	for m in moves + ["jab 2", "jab 3"]:
		var clip: String = sword._choose_clip(snap(0, "Attack", m, 10))[0]
		check(sword.anim != null and sword.anim.has_animation(clip), "the rig has the clip %s for %s" % [clip, m])
	# The claws fighter: its kicks and its blaster have their own clips.
	var kicks := {"nair": "kick_nair", "bair": "kick_bair", "uair": "kick_uair", "dair": "kick_dair", "utilt": "kick_up", "dtilt": "kick_low",
		"dash attack": "kick_dash", "neutral special": "blaster"}
	for m in kicks:
		var clip: String = fists._choose_clip(snap(1, "Attack", m, 10))[0]
		check(clip == kicks[m] and fists.anim.has_animation(clip), "claws %s plays %s (got %s)" % [m, kicks[m], clip])
	for m in expect:
		var clip: String = sword._choose_clip(snap(0, "Attack", m, 10))[0]
		check(clip == expect[m], "%s plays %s (got %s)" % [m, expect[m], clip])
	for state in ["Idle", "Walk", "Run", "Shield", "Hitstun", "Airborne", "Roll", "LedgeHang"]:
		sword.apply(Vector3.ZERO, snap(0, state, "", frame), 1.0 / 60.0)
		check(sword.blade_pivot.visible, "the blade is there while %s" % state)
	check(not hammer.long_limbs, "the bruiser has the ordinary rig")
	check(not sword.long_limbs and fists.long_limbs, "the brawler uses the long-limbed rig, the sword fighter does not")
	print("fighter view test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
