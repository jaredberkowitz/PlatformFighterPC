extends SceneTree
## What a fighter's model shows: the sword fighter's blade is there in every move (it never vanishes), the brawler never has one,
## and all share one body (the base rig: a fighter is never rebuilt for its class).
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
	var needles := FighterView.new()
	root.add_child(sword)
	root.add_child(fists)
	root.add_child(needles)
	sword.build(0, Loadout.default_for(0))
	fists.build(1, Loadout.default_for(1))
	needles.build(2, Loadout.default_for(2))
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
		needles.apply(Vector3.ZERO, snap(2, "Attack", m, frame), 1.0 / 60.0)
		check(not needles.blade_pivot.visible, "the needles fighter never has a blade in %s" % m)
		check(sword.blade_parts[0].visible, "the sword fighter's weapon is a blade in %s" % m)
	# Every move plays a clip the rig has, and the common ones have their own.
	var expect := {"jab": "attack_jab", "jab 2": "attack_jab", "dash attack": "attack_lunge", "side special": "attack_lunge", "utilt": "sword_utilt",
		"up special": "attack_uair", "dtilt": "sword_dtilt", "down special": "attack_low", "neutral special": "attack_smash", "fair": "attack_fair",
		"nair": "attack_nair", "usmash": "sword_usmash", "forward throw": "throw", "pummel": "grab", "ftilt": "sword_ftilt", "fsmash": "sword_fsmash", "dsmash": "sword_dsmash"}
	for m in moves + ["jab 2", "jab 3"]:
		var clip: String = sword._choose_clip(snap(0, "Attack", m, 10))[0]
		check(sword.anim != null and sword.anim.has_animation(clip), "the rig has the clip %s for %s" % [clip, m])
	# The claws fighter: its kicks, its claw slashes and its blaster have their own clips.
	var kicks := {"nair": "kick_double", "bair": "kick_bair", "utilt": "kick_up", "dtilt": "kick_low", "dash attack": "kick_dash",
		"usmash": "kick_uair", "neutral special": "blaster", "ftilt": "attack_claws", "uair": "kick_uair", "dair": "kick_dair",
		"dsmash": "sword_dsmash", "fair": "attack_fair"}
	for m in kicks:
		var clip: String = fists._choose_clip(snap(1, "Attack", m, 10))[0]
		check(clip == kicks[m] and fists.anim.has_animation(clip), "claws %s plays %s (got %s)" % [m, kicks[m], clip])
	# The needles fighter: kicks for its tilts and aerials, the shared chop for its forward air, a landing for its down air's landing hit.
	var needle_kicks := {"ftilt": "kick_up", "utilt": "kick_up", "dtilt": "kick_low", "nair": "kick_nair", "bair": "kick_bair",
		"uair": "kick_uair", "dair": "kick_dair", "fair": "attack_fair", "extra move 0": "land", "fsmash": "kick_side"}
	for m in needle_kicks:
		var clip: String = needles._choose_clip(snap(2, "Attack", m, 10))[0]
		check(clip == needle_kicks[m] and needles.anim.has_animation(clip), "needles %s plays %s (got %s)" % [m, needle_kicks[m], clip])
	for m in expect:
		var clip: String = sword._choose_clip(snap(0, "Attack", m, 10))[0]
		check(clip == expect[m], "%s plays %s (got %s)" % [m, expect[m], clip])
	# Each family runs its own way (round 2: the family runs never played, the run returned the shared clip before the family lookup).
	var running := snap(2, "Run", "", 10)
	running.vel = Vector2(0.24, 0.0)
	check(needles._choose_clip(running)[0] == "run_quick", "the quick family plays its own run (got %s)" % needles._choose_clip(running)[0])
	running = snap(0, "Run", "", 10)
	running.vel = Vector2(0.24, 0.0)
	check(sword._choose_clip(running)[0] == "run", "the sword family keeps the shared run")
	# (Round 3: the brawlers have their own run too, played at the ground's pace.)
	running = snap(1, "Run", "", 10)
	running.vel = Vector2(0.24, 0.0)
	var brawler_run: Array = fists._choose_clip(running)
	check(brawler_run[0] == "run_brawler", "the brawler family plays its own run (got %s)" % brawler_run[0])
	check(is_equal_approx(float(brawler_run[1]), 0.24 / FighterView.RUN_TEMPO), "a run plays at the ground's pace (%.2f)" % float(brawler_run[1]))
	# And walks its own way (round 2).
	var walking := snap(1, "Walk", "", 10)
	walking.vel = Vector2(0.12, 0.0)
	check(fists._choose_clip(walking)[0] == "walk_brawler", "the brawler family plays its own walk (got %s)" % fists._choose_clip(walking)[0])
	for state in ["Idle", "Walk", "Run", "Shield", "Hitstun", "Airborne", "Roll", "LedgeHang"]:
		sword.apply(Vector3.ZERO, snap(0, state, "", frame), 1.0 / 60.0)
		check(sword.blade_pivot.visible, "the blade is there while %s" % state)
	check(not sword.long_limbs and not fists.long_limbs and not needles.long_limbs, "all classes share the base body (no long-limbed rig swap)")
	# The hurt recoil (round 3): away from the hit, bigger for a longer stun, back upright before the stun ends.
	var hurt := func(f: int, total: int, kb_x: float) -> float:
		return FighterView.hurt_recoil({"state_frame": f, "hitstun": total - f, "kb_vel": Vector2(kb_x, 0.05), "facing": 1, "hitlag": 0})
	var light: float = hurt.call(4, 12, -0.1)
	var hard: float = hurt.call(4, 30, -0.1)
	check(light < 0.0 and hard < light, "a hit from in front throws the body back, further for a longer stun (%.1f, %.1f)" % [light, hard])
	check(hurt.call(4, 30, 0.1) > 0.0, "a hit from behind throws it forward")
	check(is_zero_approx(hurt.call(25, 30, -0.1)), "and it is upright again before the stun ends")
	print("fighter view test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
