extends RefCounted
## Scripted input sequences with screenshots, for verifying the build without a human.
## Run:  Godot --path godot -- --demo=wavedash --shots=<output folder>
## Names: wavedash, ledge, tour, shielddrop

var name := ""
var timeline: Array = []  # [frame, stick_x, stick_y, buttons]; holds until the next entry
var timeline2: Array = []  # the same for player 2 (empty: player 2 does nothing)
var events: Array = []    # [frame, "place", player, x, y] or [frame, "helpless", player]
var shots: Array = []     # [frame, label]
var end_frame := 0
var out_dir := ""
var cam_dist := 0.0  # when above zero, overrides the automatic camera distance
var cam_focus := -1  # when 0 or more, the camera frames this player alone, without easing (steady shots of one fighter's moves)
var chars: Array = []  # overrides which character each player is (0 sword, 1 claws and blaster)
var real_keys := false  # drive player 1 through real key events instead of scripted stick values


static func make(demo_name: String, m: Dictionary, dir: String):
	var d = load("res://scripts/demo.gd").new()
	d.name = demo_name
	d.out_dir = dir
	var jump: int = m.jump
	var shield: int = m.shield
	var attack: int = m.attack
	var special: int = m.special
	var strong: int = m.strong
	var grab: int = m.grab
	if demo_name.begins_with("fx_"):
		# One move landing on a fighter up close, a picture on every frame, the camera steady on the attacker (for checking trails, hit
		# effects and the animation): fx_<sword|claws|maul>_<jab|ftilt|utilt|dtilt|fsmash|usmash|dsmash|nair|fair|bair|uair|dair>, with
		# _ko on the end for a finishing hit (the other fighter at 170%).
		var parts := demo_name.split("_")
		var cls: int = {"sword": 0, "claws": 1, "maul": 2}.get(parts[1], 0)
		var move: String = parts[2] if parts.size() > 2 else "ftilt"
		d.chars = [cls, cls, cls, cls]
		d.cam_dist = 11.0
		d.cam_focus = 0
		var air := move.ends_with("air")
		if air:
			d.events = [[1, "place", 0, -1.0, 6.0], [1, "place", 1, 0.8, 6.0]]
		else:
			d.events = [[1, "stand", 0, -1.0, 1], [1, "stand", 1, 0.9, -1]]
		# [stick x, stick y, buttons] for each move.
		var inputs: Dictionary = {"jab": [0, 0, attack], "ftilt": [40, 0, attack], "utilt": [0, 70, attack], "dtilt": [0, -70, attack],
			"fsmash": [127, 0, attack], "usmash": [0, 70, attack | strong], "dsmash": [0, -70, attack | strong], "nair": [0, 0, attack],
			"fair": [127, 0, attack], "bair": [-127, 0, attack], "uair": [0, 100, attack], "dair": [0, -70, attack]}
		var input: Array = inputs.get(move, inputs.ftilt)
		d.timeline = [[0, 0, 0, 0], [10, input[0], input[1], input[2]], [11, 0, 0, 0]]
		if demo_name.ends_with("_ko"):
			d.events.append([1, "percent", 1, 170.0])
		# (Long enough for a smash, or a multi-hit move whose hits freeze it.)
		var last := 75
		for f in range(11, last):
			d.shots.append([f, "f%02d" % (f - 10)])
		d.end_frame = last + 1
		return d
	match demo_name:
		"wavedash":
			d.timeline = [[0, 0, 0, 0], [30, 0, 0, jump], [31, 100, -80, shield], [36, 0, 0, 0]]
			d.shots = [[30, "a_jump_pressed"], [33, "b_squat"], [36, "c_waveland"], [41, "d_slide"], [60, "e_after"]]
			d.end_frame = 70
		"ledge":
			d.events = [[2, "place", 0, -12.0, -0.5]]
			d.timeline = [[0, 0, 0, 0], [60, 0, 127, 0], [61, 0, 0, 0]]
			d.shots = [[6, "a_grab_swing"], [25, "b_hang"], [63, "c_climb1"], [67, "d_climb2"], [71, "e_climb3"], [76, "f_climb4"], [110, "g_idle"]]
			d.end_frame = 120
		"portrait":
			d.cam_dist = 9.0
			d.timeline = [[0, 0, 0, 0]]
			d.shots = [[20, "a_idle"]]
			d.end_frame = 25
		"landing":
			d.cam_dist = 17.0
			d.events = [[1, "place", 0, -4.0, 4.0]]
			d.timeline = [[0, 0, 0, 0]]
			d.shots = [[33, "a_before"], [36, "b_impact"], [39, "c_squashed"], [45, "d_rebound"], [70, "e_settled"]]
			d.end_frame = 75
		"dashdance":
			# Dash back and forth in 3-frame bursts, then dash in and jump.
			d.timeline = [[0, 0, 0, 0]]
			var t := 20
			for i in 8:
				d.timeline.append([t, 127 if i % 2 == 0 else -127, 0, 0])
				t += 3
			d.timeline.append([t, 127, 0, 0])
			d.timeline.append([t + 8, 127, 0, jump])
			d.timeline.append([t + 12, 127, 0, 0])
			d.timeline.append([t + 60, 0, 0, 0])
			d.shots = [[26, "a_dance"], [38, "b_dance"], [50, "c_dash_in"], [58, "d_jump"], [68, "e_air"], [95, "f_land"]]
			d.end_frame = 110
		"walkoff":
			d.timeline = [[0, -127, 0, 0]]
			d.shots = [[30, "a_dashing"], [36, "b_off_edge"], [42, "c_falling"], [55, "d_hanging"], [80, "e_later"]]
			d.end_frame = 90
		"fastfall":
			# Real key presses: full hop with Space, then double tap S just after the apex.
			d.real_keys = true
			d.cam_dist = 22.0
			d.timeline = [[0, 0, 0, 0]]
			d.events = [
				[1, "place", 0, 0.0, 0.0],
				[20, "key", KEY_SPACE, true], [43, "key", KEY_SPACE, false],
				[58, "key", KEY_S, true], [61, "key", KEY_S, false],
				[64, "key", KEY_S, true], [75, "key", KEY_S, false]]
			d.shots = [[56, "a_before"], [62, "b_after_first_tap"], [66, "c_second_tap"], [72, "d_falling_fast"], [80, "e_later"]]
			d.end_frame = 100
		"walk":
			# Real key presses: a walk, a double-tap dash into a run, then a short hop with a tap of drift.
			d.real_keys = true
			d.cam_dist = 24.0
			d.timeline = [[0, 0, 0, 0]]
			d.events = [
				[1, "place", 0, -9.0, 0.0],
				[4, "key", KEY_D, true], [34, "key", KEY_D, false],
				[50, "key", KEY_D, true], [53, "key", KEY_D, false],
				[56, "key", KEY_D, true], [86, "key", KEY_D, false],
				[96, "key", KEY_N, true], [106, "key", KEY_N, false],
				[100, "key", KEY_D, true], [104, "key", KEY_D, false]]
			d.shots = [[16, "a_walking"], [30, "b_walk_full_tilt"], [60, "c_dash"], [78, "d_run"], [100, "e_hop_nudge"], [118, "f_drifted"]]
			d.end_frame = 130
		"combat":
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 1.0, -1]]
			d.timeline = [[0, 0, 0, 0], [20, 30, 0, attack], [22, 30, 0, 0], [30, 0, 0, 0]]
			d.shots = [[24, "a_windup"], [28, "b_hit"], [31, "c_hitlag"], [38, "d_launched"], [60, "e_after"]]
			d.end_frame = 80
		"smash":
			d.cam_dist = 24.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 1.2, -1], [1, "percent", 1, 90.0]]
			d.timeline = [[0, 0, 0, 0], [20, 127, 0, attack], [23, 0, 0, 0]]
			d.shots = [[30, "a_windup"], [40, "b_hit"], [48, "c_launch"], [62, "d_flying"], [90, "e_far"]]
			d.end_frame = 120
		"ko":
			# A forward smash on a fighter at 170%: the hit shakes the camera, which closes in because the launch will KO, and the
			# launch leaves a smoke trail.
			d.cam_dist = 24.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 1.2, -1], [1, "percent", 1, 170.0]]
			d.timeline = [[0, 0, 0, 0], [20, 127, 0, attack], [23, 0, 0, 0]]
			d.shots = [[38, "a_hit"], [44, "b_hitlag_zoom"], [56, "c_launch_smoke"], [60, "d_flying"], [64, "e_offscreen"], [68, "f_blast"], [76, "g_blast_later"]]
			d.end_frame = 130
		"stage":
			# A still of the stage with both fighters standing (for checking stage art; add --stage=N).
			d.timeline = [[0, 0, 0, 0]]
			d.shots = [[40, "a_stage"]]
			d.end_frame = 45
		"moves":
			# A tour of ground moves for checking animation: forward tilt, up tilt, down tilt, then the smashes (sword fighter, player 1).
			d.chars = [0, 1, 0, 1]
			d.cam_dist = 16.0
			d.events = [[1, "stand", 0, -1.0, 1], [1, "stand", 1, 6.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 40, 0, attack], [11, 40, 0, 0], [12, 0, 0, 0], [60, 0, 40, attack], [61, 0, 40, 0], [62, 0, 0, 0],
				[110, 0, -40, attack], [111, 0, -40, 0], [112, 0, 0, 0], [160, 0, 0, attack | strong], [161, 0, 0, 0], [230, 0, 40, attack | strong], [231, 0, 0, 0]]
			d.shots = [[18, "a_ftilt"], [68, "b_utilt"], [117, "c_dtilt"], [176, "d_fsmash"], [246, "e_usmash"]]
			d.end_frame = 260
		"kicks":
			# The claws fighter's kicks: up tilt, down tilt, then neutral air and back air in a jump.
			d.chars = [1, 0, 1, 0]
			d.cam_dist = 16.0
			d.events = [[1, "stand", 0, -1.0, 1], [1, "stand", 1, 6.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 40, attack], [11, 0, 40, 0], [12, 0, 0, 0], [60, 0, -40, attack], [61, 0, -40, 0], [62, 0, 0, 0],
				[110, 0, 0, jump], [118, 0, 0, attack], [119, 0, 0, 0], [170, 0, 0, jump], [178, -127, 0, attack], [179, 0, 0, 0]]
			d.shots = [[17, "a_utilt"], [66, "b_dtilt"], [128, "c_nair"], [190, "d_bair"]]
			d.end_frame = 210
		"sword_fair":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, 1.7, 6.0]]
			d.timeline = [[0, 0, 0, 0], [10, 127, 0, attack], [11, 0, 0, 0]]
			d.shots = [[13, "a_windup"], [16, "b_hit"], [19, "c_hitlag"], [30, "d_launched"]]
			d.end_frame = 45
		"sword_fair_close", "sword_dair_close", "claws_nair_close":
			# One move up close, a picture on every frame of its swing, nobody in the way (for checking the animation and the trail).
			var sword: bool = demo_name.begins_with("sword")
			d.chars = [0, 0, 0, 0] if sword else [1, 1, 1, 1]
			# The other fighter hangs back out of reach (the camera frames them both, so not too far).
			d.cam_dist = 13.0
			var away := 6.0 if demo_name.begins_with("sword_dair") else -6.0
			d.events = [[1, "place", 0, 0.0, 8.0], [1, "place", 1, away, 8.0]]
			var stick_y := -70 if demo_name.begins_with("sword_dair") else 0  # down for the down air, short of a fast fall
			var stick_x := 127 if demo_name.begins_with("sword_fair") else 0
			d.timeline = [[0, 0, 0, 0], [10, stick_x, stick_y, attack], [11, 0, 0, 0]]
			var first := 13
			var last := 26 if not demo_name.begins_with("sword_dair") else 30
			for f in range(first, last + 1):
				d.shots.append([f, "f%02d" % (f - 10)])
			d.end_frame = last + 2
		"sword_gallery":
			# Every sword move in turn, photographed at its wind-up, first hit, middle and recovery (for checking the animation).
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 12.0
			d.events = [[1, "stand", 1, 7.0, -1]]  # out of reach in front, so the camera centres the attacker
			# [name, stick x, stick y, buttons, airborne, first active, last active]
			var gallery := [["jab", 0, 0, attack, false, 5, 6], ["ftilt", 40, 0, attack, false, 8, 11], ["utilt", 0, 70, attack, false, 6, 12],
				["dtilt", 0, -70, attack, false, 7, 8], ["fsmash", 60, 0, attack | strong, false, 10, 13],
				["usmash", 0, 70, attack | strong, false, 13, 17], ["dsmash", 0, -70, attack | strong, false, 6, 23],
				["nair", 0, 0, attack, true, 6, 21], ["uair", 0, 100, attack, true, 5, 9], ["bair", -127, 0, attack, true, 7, 11],
				["fair", 127, 0, attack, true, 6, 8], ["dair", 0, -70, attack, true, 9, 13]]
			var t := 20
			var n := 0
			for g in gallery:
				d.events.append([t - 4, "place", 0, 0.0, 6.0] if g[4] else [t - 4, "stand", 0, 0.0, 1])
				d.timeline.append([t, g[1], g[2], g[3]])
				d.timeline.append([t + 1, 0, 0, 0])
				var first: int = g[5]
				var last: int = g[6]
				var mid := (first + last) / 2
				for k in [[first - 3, "a"], [first, "b"], [mid, "c"], [last + 5, "d"]]:
					d.shots.append([t + k[0] - 1, "m%02d_%s_%s" % [n, g[0], k[1]]])
				t += 80
				n += 1
			d.end_frame = t
		"sword_bair":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, -5.7, 6.0]]
			d.timeline = [[0, 0, 0, 0], [10, -127, 0, attack], [11, 0, 0, 0]]
			d.shots = [[14, "a_windup"], [17, "b_hit"], [20, "c_hitlag"], [32, "d_launched"], [52, "e_turned"]]
			d.end_frame = 60
		"sword_nair":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 7.0], [1, "place", 1, 1.0, 7.0]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, attack], [11, 0, 0, 0]]
			d.shots = [[13, "a_windup"], [16, "b_hit1"], [25, "c_hit2_window"], [40, "d_late"]]
			d.end_frame = 55
		"sword_rising_slash":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 24.0
			d.events = [[1, "stand", 0, -2.0, 1], [1, "stand", 1, -0.2, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 127, special], [11, 0, 0, 0]]
			d.shots = [[12, "a_start"], [15, "b_hit"], [20, "c_rising"], [30, "d_apex"], [55, "e_helpless"]]
			d.end_frame = 75
		"claws_fair":
			d.chars = [1, 1, 1, 1]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, 0.3, 6.0]]
			d.timeline = [[0, 0, 0, 0], [10, 127, 0, attack], [11, 0, 0, 0]]
			d.shots = [[13, "a_windup"], [17, "b_hit"], [22, "c_hitlag"], [32, "d_launched"]]
			d.end_frame = 45
		"claws_nair":
			d.chars = [1, 1, 1, 1]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, -0.5, 6.0]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, attack], [11, 0, 0, 0]]
			d.shots = [[13, "a_windup"], [17, "b_hit"], [25, "c_late"], [38, "d_end"]]
			d.end_frame = 55
		"claws_ftilt":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.8, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 30, 0, attack], [12, 30, 0, 0]]
			d.shots = [[14, "a_swipe"], [18, "b_hit1"], [25, "c_hit2"], [38, "d_after"]]
			d.end_frame = 55
		"claws_blaster":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 26.0
			d.events = [[1, "stand", 0, -8.0, 1], [1, "stand", 1, 2.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, special], [11, 0, 0, 0]]
			d.shots = [[20, "a_draw"], [27, "b_shot"], [36, "c_flying"], [46, "d_hit"], [62, "e_after"]]
			d.end_frame = 80
		"sword_utilt":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.2, -1]]
			d.timeline = [[0, 0, 0, 0], [4, 0, 66, 0], [12, 0, 66, attack], [14, 0, 66, 0], [22, 0, 0, 0]]
			d.shots = [[15, "a_windup"], [18, "b_hit"], [22, "c_swing"], [32, "d_after"]]
			d.end_frame = 50
		"sword_dtilt":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 1.9, -1]]
			d.timeline = [[0, 0, 0, 0], [4, 0, -66, 0], [12, 0, -66, attack], [14, 0, -66, 0], [22, 0, 0, 0]]
			d.shots = [[16, "a_windup"], [19, "b_hit"], [24, "c_after"]]
			d.end_frame = 40
		"sword_fsmash":
			# Holds the attack for 36 frames of charge, then releases.
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 2.3, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 90, 0, attack | strong], [11, 0, 0, attack], [46, 0, 0, 0]]
			d.shots = [[8, "a_idle"], [30, "b_charging"], [50, "c_swing"], [54, "d_hit"], [62, "e_launch"], [90, "f_after"]]
			d.end_frame = 110
		"sword_usmash":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.2, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 100, attack | strong], [11, 0, 0, 0]]
			d.shots = [[16, "a_windup"], [23, "b_hit"], [28, "c_launch"], [45, "d_after"]]
			d.end_frame = 75
		"sword_dsmash":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, -100, attack | strong], [11, 0, 0, 0]]
			d.shots = [[14, "a_front_swing"], [18, "b_front_hit"], [28, "c_pause"], [32, "d_back_swing"], [40, "e_after"]]
			d.end_frame = 75
		"sword_uair":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, -1.4, 8.4]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 100, attack], [11, 0, 0, 0]]
			d.shots = [[12, "a_windup"], [15, "b_hit"], [18, "c_hitlag"], [30, "d_launched"]]
			d.end_frame = 45
		"sword_seeker":
			# Neutral special (a scripted placeholder): a bolt that bends toward the enemy's height.
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 26.0
			d.events = [[1, "stand", 0, -9.0, 1], [2, "place", 1, 4.0, 6.0]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, special], [11, 0, 0, 0]]
			d.shots = [[16, "a_windup"], [26, "b_fired"], [38, "c_bending"], [50, "d_rising"], [62, "e_hit"]]
			d.end_frame = 90
		"sword_lunge":
			# Side special (a scripted placeholder): hold still, then thrust forward.
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -7.0, 1], [1, "stand", 1, -2.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 127, 0, special], [11, 0, 0, 0]]
			d.shots = [[16, "a_windup"], [21, "b_thrust"], [25, "c_hit"], [34, "d_recover"], [52, "e_after"]]
			d.end_frame = 70
		"sword_charge_thrust":
			# Neutral special, the charged thrust: hold special to charge, let go to thrust.
			d.chars = [0, 1, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -8.0, 1], [1, "stand", 1, -3.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, special], [70, 0, 0, 0]]
			d.shots = [[20, "a_raised"], [50, "b_charging"], [72, "c_release"], [80, "d_thrust"], [90, "e_hit"]]
			d.end_frame = 110
		"sword_blade_combo":
			# Side special, the blade combo: tap special repeatedly; the later hits follow.
			d.chars = [0, 1, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -8.0, 1], [1, "stand", 1, -4.5, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 127, 0, special], [11, 0, 0, 0]]
			var tt := 14
			for i in 30:
				d.timeline.append([tt, 0, 0, special])
				d.timeline.append([tt + 1, 0, 0, 0])
				tt += 3
			d.shots = [[21, "a_hit1"], [38, "b_hit2"], [60, "c_hit3"], [85, "d_finisher"], [110, "e_after"]]
			d.end_frame = 130
		"sword_counter":
			# Down special, Counter: the brawler swings into the stance and gets hit back harder.
			d.chars = [0, 1, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -8.0, 1], [1, "stand", 1, -5.6, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, -127, special], [11, 0, -127, 0], [14, 0, 0, 0]]
			d.timeline2 = [[0, 0, 0, 0], [12, 0, 0, attack], [13, 0, 0, 0]]
			d.shots = [[16, "a_stance"], [20, "b_caught"], [28, "c_answer"], [40, "d_after"]]
			d.end_frame = 70
		"claws_fire_rush_aim":
			# Up special aimed forward and up with the stick.
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 26.0
			d.events = [[1, "stand", 0, -6.0, 1], [1, "stand", 1, 9.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 100, 127, special], [11, 100, 127, 0]]
			d.shots = [[20, "a_windup"], [32, "b_flying"], [38, "c_flying"], [52, "d_end"]]
			d.end_frame = 70
		"sword_pivot":
			# Dash right, then turn around with grab: the grab comes out behind.
			d.chars = [0, 1, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -8.0, 1], [1, "stand", 1, -10.4, 1]]
			d.timeline = [[0, 0, 0, 0], [10, 127, 0, 0], [26, -127, 0, grab], [27, -127, 0, 0], [28, 0, 0, 0]]
			d.shots = [[25, "a_running"], [30, "b_turning"], [36, "c_grab"], [44, "d_caught"]]
			d.end_frame = 60
		"sword_upb_ledge":
			# Up special from below the ledge: it grabs in mid-move.
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 24.0
			d.events = [[2, "place", 0, -12.5, -5.0]]
			d.timeline = [[0, 0, 0, 0], [6, 0, 127, special], [7, 0, 0, 0]]
			d.shots = [[8, "a_start"], [12, "b_rising"], [16, "c_grabbed"], [40, "d_hanging"]]
			d.end_frame = 50
		"claws_utilt":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.0, -1]]
			d.timeline = [[0, 0, 0, 0], [4, 0, 66, 0], [12, 0, 66, attack], [14, 0, 66, 0], [22, 0, 0, 0]]
			d.shots = [[15, "a_windup"], [19, "b_hit"], [24, "c_late"], [36, "d_after"]]
			d.end_frame = 55
		"claws_dtilt":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 1.0, -1]]
			d.timeline = [[0, 0, 0, 0], [4, 0, -66, 0], [12, 0, -66, attack], [14, 0, -66, 0], [22, 0, 0, 0]]
			d.shots = [[15, "a_windup"], [17, "b_hit"], [22, "c_after"]]
			d.end_frame = 40
		"claws_jab":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.2, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, attack], [12, 0, 0, 0], [24, 0, 0, attack], [26, 0, 0, 0], [46, 0, 0, attack], [48, 0, 0, 0]]
			d.shots = [[15, "a_jab1"], [36, "b_jab2"], [58, "c_jab3"], [90, "d_after"]]
			d.end_frame = 100
		"claws_dashattack":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -9.0, 1], [1, "stand", 1, -1.5, -1]]
			d.timeline = [[0, 0, 0, 0], [2, 127, 0, 0], [22, 127, 0, attack], [24, 127, 0, 0], [28, 0, 0, 0]]
			d.shots = [[20, "a_run"], [27, "b_windup"], [34, "c_hit"], [46, "d_after"]]
			d.end_frame = 70
		"claws_uair":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, -1.7, 7.6]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 100, attack], [11, 0, 0, 0]]
			d.shots = [[13, "a_windup"], [17, "b_hit"], [20, "c_hitlag"], [32, "d_launched"]]
			d.end_frame = 50
		"claws_bair":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, -4.0, 6.0]]
			d.timeline = [[0, 0, 0, 0], [10, -127, 0, attack], [11, 0, 0, 0]]
			d.shots = [[18, "a_windup"], [23, "b_hit"], [26, "c_hitlag"], [38, "d_launched"]]
			d.end_frame = 60
		"claws_dair":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 7.0], [1, "place", 1, -1.6, 5.4]]
			d.timeline = [[0, 0, 0, 0], [6, 0, -70, 0], [12, 0, -70, attack], [14, 0, 0, 0]]
			d.shots = [[22, "a_windup"], [28, "b_hit"], [31, "c_hitlag"], [40, "d_spiked"]]
			d.end_frame = 60
		"claws_fsmash":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 1.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 90, 0, attack | strong], [11, 0, 0, attack], [40, 0, 0, 0]]
			d.shots = [[8, "a_idle"], [30, "b_charging"], [58, "c_swing"], [64, "d_hit"], [75, "e_launch"]]
			d.end_frame = 100
		"claws_usmash":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 100, attack | strong], [11, 0, 0, 0]]
			d.shots = [[22, "a_hit1"], [32, "b_hit2"], [38, "c_launch"], [55, "d_after"]]
			d.end_frame = 70
		"claws_dsmash":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.8, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, -100, attack | strong], [11, 0, 0, 0]]
			d.shots = [[22, "a_front"], [26, "b_hit"], [34, "c_back"], [50, "d_after"]]
			d.end_frame = 65
		"claws_flash_dash":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 24.0
			d.events = [[1, "stand", 0, -9.0, 1], [1, "stand", 1, -1.0, -1]]
			d.timeline = [[0, 0, 0, 0], [8, 127, 0, special], [9, 0, 0, 0]]
			d.shots = [[20, "a_windup"], [30, "b_dash"], [40, "c_hit"], [52, "d_end"], [70, "e_after"]]
			d.end_frame = 90
		"claws_fire_rush":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 24.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, -0.2, -1]]
			d.timeline = [[0, 0, 0, 0], [8, 0, 127, special], [9, 0, 0, 0]]
			d.shots = [[22, "a_windup"], [30, "b_rising"], [42, "c_hits"], [56, "d_final"], [75, "e_helpless"]]
			d.end_frame = 110
		"shield_block":
			# Player 2 shields; player 1 forward-tilts into it. The bubble shrinks and the blocker slides back.
			d.chars = [0, 1, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.6, -1]]
			d.timeline = [[0, 0, 0, 0], [20, 30, 0, attack], [22, 0, 0, 0]]
			d.timeline2 = [[0, 0, 0, 0], [10, 0, 0, shield]]
			d.shots = [[12, "a_shield_up"], [28, "b_block"], [36, "c_stun"], [70, "d_after"]]
			d.end_frame = 90
		"shield_break":
			d.chars = [0, 1, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.6, -1], [2, "shield", 1, 6.0]]
			d.timeline = [[0, 0, 0, 0], [20, 30, 0, attack], [22, 0, 0, 0]]
			d.timeline2 = [[0, 0, 0, 0], [10, 0, 0, shield]]
			d.shots = [[26, "a_hit"], [34, "b_broken_hop"], [60, "c_stunned"], [110, "d_still_stunned"]]
			d.end_frame = 130
		"roll":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -3.0, 1], [1, "stand", 1, 6.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, shield], [20, 127, 0, shield], [21, 0, 0, 0], [60, 0, 0, shield], [66, 0, -127, shield], [67, 0, 0, 0]]
			d.shots = [[18, "a_shield"], [30, "b_rolling"], [40, "c_rolled"], [72, "d_spot_dodge"], [90, "e_after"]]
			d.end_frame = 100
		"grab":
			# Grab, two pummels, then a forward throw.
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.6, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, grab], [11, 0, 0, 0], [30, 0, 0, attack], [32, 0, 0, 0], [58, 0, 0, attack], [60, 0, 0, 0], [86, 127, 0, 0], [90, 0, 0, 0]]
			d.shots = [[14, "a_reaching"], [20, "b_held"], [34, "c_pummel"], [90, "d_throw_windup"], [102, "e_thrown"], [125, "f_after"]]
			d.end_frame = 140
		"shield_grab":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.6, -1]]
			d.timeline = [[0, 0, 0, 0], [8, 0, 0, shield], [20, 0, 0, shield | attack], [21, 0, 0, shield]]
			d.timeline2 = [[0, 0, 0, 0], [10, 0, 0, shield]]
			d.shots = [[16, "a_both_shielding"], [28, "b_shield_grab"], [40, "c_held"]]
			d.end_frame = 60
		"low":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 18.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 1.6, -1]]
			d.timeline = [[0, 0, 0, 0], [8, 0, -127, 0], [20, 0, -127, attack], [22, 0, -127, 0]]
			d.shots = [[15, "a_idle_crouch"], [24, "b_swing"], [27, "c_hit"], [33, "d_after"]]
			d.end_frame = 45
		"shielddrop":
			d.events = [[1, "place", 0, -5.0, 4.0]]
			d.timeline = [[0, 0, 0, 0], [40, 0, 0, shield], [44, 0, -100, shield], [50, 0, 0, 0]]
			d.shots = [[30, "a_on_platform"], [42, "b_shield"], [48, "c_dropping"], [90, "d_below"]]
			d.end_frame = 100
		_:
			d.timeline = [
				[0, 0, 0, 0], [20, 127, 0, 0], [80, 0, 0, 0], [90, 0, 0, jump], [105, 60, 0, 0],
				[150, 0, -100, 0], [170, 0, 0, shield], [200, 0, 0, 0]]
			d.shots = [[26, "a_dash"], [60, "b_run"], [98, "c_hop"], [160, "d_crouch"], [180, "e_shield"]]
			d.end_frame = 215
	return d


func input_at(frame: int) -> Dictionary:
	var cur: Array = timeline[0]
	for e in timeline:
		if e[0] <= frame:
			cur = e
	return {"x": cur[1], "y": cur[2], "buttons": cur[3]}


func input_at2(frame: int) -> Dictionary:
	var cur: Array = timeline2[0]
	for e in timeline2:
		if e[0] <= frame:
			cur = e
	return {"x": cur[1], "y": cur[2], "buttons": cur[3]}


func events_at(frame: int) -> Array:
	return events.filter(func(e): return e[0] == frame)


func shot_at(frame: int) -> String:
	for s in shots:
		if s[0] == frame:
			return "%s_%s" % [name, s[1]]
	return ""
