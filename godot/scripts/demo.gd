extends RefCounted
## Scripted input sequences with screenshots, for verifying the build without a human.
## Run:  Godot --path godot -- --demo=wavedash --shots=<output folder>
## Names: wavedash, ledge, tour, shielddrop

var name := ""
var timeline: Array = []  # [frame, stick_x, stick_y, buttons]; holds until the next entry
var events: Array = []    # [frame, "place", player, x, y] or [frame, "helpless", player]
var shots: Array = []     # [frame, label]
var end_frame := 0
var out_dir := ""
var cam_dist := 0.0  # when above zero, overrides the automatic camera distance
var real_keys := false  # drive player 1 through real key events instead of scripted stick values


static func make(demo_name: String, m: Dictionary, dir: String):
	var d = load("res://scripts/demo.gd").new()
	d.name = demo_name
	d.out_dir = dir
	var jump: int = m.jump
	var shield: int = m.shield
	var attack: int = m.attack
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


func events_at(frame: int) -> Array:
	return events.filter(func(e): return e[0] == frame)


func shot_at(frame: int) -> String:
	for s in shots:
		if s[0] == frame:
			return "%s_%s" % [name, s[1]]
	return ""
