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
	match demo_name:
		"wavedash":
			d.timeline = [[0, 0, 0, 0], [30, 0, 0, jump], [31, 100, -80, shield], [36, 0, 0, 0]]
			d.shots = [[30, "a_jump_pressed"], [33, "b_squat"], [36, "c_waveland"], [41, "d_slide"], [60, "e_after"]]
			d.end_frame = 70
		"ledge":
			d.events = [[2, "place", 0, -12.0, -0.5]]
			d.timeline = [[0, 0, 0, 0], [60, 0, 127, 0], [61, 0, 0, 0]]
			d.shots = [[25, "a_hang"], [62, "b_getup"], [110, "c_idle"]]
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
		"marth_fair":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, 1.7, 6.0]]
			d.timeline = [[0, 0, 0, 0], [10, 127, 0, attack], [11, 0, 0, 0]]
			d.shots = [[13, "a_windup"], [16, "b_hit"], [19, "c_hitlag"], [30, "d_launched"]]
			d.end_frame = 45
		"marth_bair":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, -5.7, 6.0]]
			d.timeline = [[0, 0, 0, 0], [10, -127, 0, attack], [11, 0, 0, 0]]
			d.shots = [[14, "a_windup"], [17, "b_hit"], [20, "c_hitlag"], [32, "d_launched"], [52, "e_turned"]]
			d.end_frame = 60
		"marth_nair":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 7.0], [1, "place", 1, 1.0, 7.0]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, attack], [11, 0, 0, 0]]
			d.shots = [[13, "a_windup"], [16, "b_hit1"], [25, "c_hit2_window"], [40, "d_late"]]
			d.end_frame = 55
		"marth_dolphin":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 24.0
			d.events = [[1, "stand", 0, -2.0, 1], [1, "stand", 1, -0.2, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 127, special], [11, 0, 0, 0]]
			d.shots = [[12, "a_start"], [15, "b_hit"], [20, "c_rising"], [30, "d_apex"], [55, "e_helpless"]]
			d.end_frame = 75
		"wolf_fair":
			d.chars = [1, 1, 1, 1]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, 0.3, 6.0]]
			d.timeline = [[0, 0, 0, 0], [10, 127, 0, attack], [11, 0, 0, 0]]
			d.shots = [[13, "a_windup"], [17, "b_hit"], [22, "c_hitlag"], [32, "d_launched"]]
			d.end_frame = 45
		"wolf_nair":
			d.chars = [1, 1, 1, 1]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, -0.5, 6.0]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, attack], [11, 0, 0, 0]]
			d.shots = [[13, "a_windup"], [17, "b_hit"], [25, "c_late"], [38, "d_end"]]
			d.end_frame = 55
		"wolf_ftilt":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.8, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 30, 0, attack], [12, 30, 0, 0]]
			d.shots = [[14, "a_swipe"], [18, "b_hit1"], [25, "c_hit2"], [38, "d_after"]]
			d.end_frame = 55
		"wolf_blaster":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 26.0
			d.events = [[1, "stand", 0, -8.0, 1], [1, "stand", 1, 2.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, special], [11, 0, 0, 0]]
			d.shots = [[20, "a_draw"], [27, "b_shot"], [36, "c_flying"], [46, "d_hit"], [62, "e_after"]]
			d.end_frame = 80
		"marth_utilt":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.2, -1]]
			d.timeline = [[0, 0, 0, 0], [4, 0, 66, 0], [12, 0, 66, attack], [14, 0, 66, 0], [22, 0, 0, 0]]
			d.shots = [[15, "a_windup"], [18, "b_hit"], [22, "c_swing"], [32, "d_after"]]
			d.end_frame = 50
		"marth_dtilt":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 1.9, -1]]
			d.timeline = [[0, 0, 0, 0], [4, 0, -66, 0], [12, 0, -66, attack], [14, 0, -66, 0], [22, 0, 0, 0]]
			d.shots = [[16, "a_windup"], [19, "b_hit"], [24, "c_after"]]
			d.end_frame = 40
		"marth_fsmash":
			# Holds the attack for 36 frames of charge, then releases.
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 2.3, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 90, 0, attack | strong], [11, 0, 0, attack], [46, 0, 0, 0]]
			d.shots = [[8, "a_idle"], [30, "b_charging"], [50, "c_swing"], [54, "d_hit"], [62, "e_launch"], [90, "f_after"]]
			d.end_frame = 110
		"marth_usmash":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.2, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 100, attack | strong], [11, 0, 0, 0]]
			d.shots = [[16, "a_windup"], [23, "b_hit"], [28, "c_launch"], [45, "d_after"]]
			d.end_frame = 75
		"marth_dsmash":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, -100, attack | strong], [11, 0, 0, 0]]
			d.shots = [[14, "a_front_swing"], [18, "b_front_hit"], [28, "c_pause"], [32, "d_back_swing"], [40, "e_after"]]
			d.end_frame = 75
		"marth_uair":
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, -1.4, 8.4]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 100, attack], [11, 0, 0, 0]]
			d.shots = [[12, "a_windup"], [15, "b_hit"], [18, "c_hitlag"], [30, "d_launched"]]
			d.end_frame = 45
		"marth_upb_ledge":
			# Up special from below the ledge: it grabs in mid-move.
			d.chars = [0, 0, 0, 0]
			d.cam_dist = 24.0
			d.events = [[2, "place", 0, -12.5, -5.0]]
			d.timeline = [[0, 0, 0, 0], [6, 0, 127, special], [7, 0, 0, 0]]
			d.shots = [[8, "a_start"], [12, "b_rising"], [16, "c_grabbed"], [40, "d_hanging"]]
			d.end_frame = 50
		"wolf_utilt":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.0, -1]]
			d.timeline = [[0, 0, 0, 0], [4, 0, 66, 0], [12, 0, 66, attack], [14, 0, 66, 0], [22, 0, 0, 0]]
			d.shots = [[15, "a_windup"], [19, "b_hit"], [24, "c_late"], [36, "d_after"]]
			d.end_frame = 55
		"wolf_dtilt":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 1.0, -1]]
			d.timeline = [[0, 0, 0, 0], [4, 0, -66, 0], [12, 0, -66, attack], [14, 0, -66, 0], [22, 0, 0, 0]]
			d.shots = [[15, "a_windup"], [17, "b_hit"], [22, "c_after"]]
			d.end_frame = 40
		"wolf_jab":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 20.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.2, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 0, attack], [12, 0, 0, 0], [24, 0, 0, attack], [26, 0, 0, 0], [46, 0, 0, attack], [48, 0, 0, 0]]
			d.shots = [[15, "a_jab1"], [36, "b_jab2"], [58, "c_jab3"], [90, "d_after"]]
			d.end_frame = 100
		"wolf_dashattack":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -9.0, 1], [1, "stand", 1, -1.5, -1]]
			d.timeline = [[0, 0, 0, 0], [2, 127, 0, 0], [22, 127, 0, attack], [24, 127, 0, 0], [28, 0, 0, 0]]
			d.shots = [[20, "a_run"], [27, "b_windup"], [34, "c_hit"], [46, "d_after"]]
			d.end_frame = 70
		"wolf_uair":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, -1.7, 7.6]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 100, attack], [11, 0, 0, 0]]
			d.shots = [[13, "a_windup"], [17, "b_hit"], [20, "c_hitlag"], [32, "d_launched"]]
			d.end_frame = 50
		"wolf_bair":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 6.0], [1, "place", 1, -4.0, 6.0]]
			d.timeline = [[0, 0, 0, 0], [10, -127, 0, attack], [11, 0, 0, 0]]
			d.shots = [[18, "a_windup"], [23, "b_hit"], [26, "c_hitlag"], [38, "d_launched"]]
			d.end_frame = 60
		"wolf_dair":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "place", 0, -2.0, 7.0], [1, "place", 1, -1.6, 5.4]]
			d.timeline = [[0, 0, 0, 0], [6, 0, -70, 0], [12, 0, -70, attack], [14, 0, 0, 0]]
			d.shots = [[22, "a_windup"], [28, "b_hit"], [31, "c_hitlag"], [40, "d_spiked"]]
			d.end_frame = 60
		"wolf_fsmash":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 1.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 90, 0, attack | strong], [11, 0, 0, attack], [40, 0, 0, 0]]
			d.shots = [[8, "a_idle"], [30, "b_charging"], [58, "c_swing"], [64, "d_hit"], [75, "e_launch"]]
			d.end_frame = 100
		"wolf_usmash":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.0, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, 100, attack | strong], [11, 0, 0, 0]]
			d.shots = [[22, "a_hit1"], [32, "b_hit2"], [38, "c_launch"], [55, "d_after"]]
			d.end_frame = 70
		"wolf_dsmash":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 22.0
			d.events = [[1, "stand", 0, -1.2, 1], [1, "stand", 1, 0.8, -1]]
			d.timeline = [[0, 0, 0, 0], [10, 0, -100, attack | strong], [11, 0, 0, 0]]
			d.shots = [[22, "a_front"], [26, "b_hit"], [34, "c_back"], [50, "d_after"]]
			d.end_frame = 65
		"wolf_flash":
			d.chars = [1, 0, 0, 0]
			d.cam_dist = 24.0
			d.events = [[1, "stand", 0, -9.0, 1], [1, "stand", 1, -1.0, -1]]
			d.timeline = [[0, 0, 0, 0], [8, 127, 0, special], [9, 0, 0, 0]]
			d.shots = [[20, "a_windup"], [30, "b_dash"], [40, "c_hit"], [52, "d_end"], [70, "e_after"]]
			d.end_frame = 90
		"wolf_firewolf":
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
