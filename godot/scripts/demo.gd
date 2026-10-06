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


static func make(demo_name: String, m: Dictionary, dir: String):
	var d = load("res://scripts/demo.gd").new()
	d.name = demo_name
	d.out_dir = dir
	var jump: int = m.jump
	var shield: int = m.shield
	match demo_name:
		"wavedash":
			d.timeline = [[0, 0, 0, 0], [30, 0, 0, jump], [31, 100, -80, shield], [36, 0, 0, 0]]
			d.shots = [[30, "a_jump_pressed"], [33, "b_squat"], [36, "c_waveland"], [41, "d_slide"], [60, "e_after"]]
			d.end_frame = 70
		"ledge":
			d.events = [[2, "place", 0, -21.0, -0.5]]
			d.timeline = [[0, 0, 0, 0], [60, 0, 127, 0], [61, 0, 0, 0]]
			d.shots = [[25, "a_hang"], [62, "b_getup"], [110, "c_idle"]]
			d.end_frame = 120
		"portrait":
			d.cam_dist = 9.0
			d.timeline = [[0, 0, 0, 0]]
			d.shots = [[20, "a_idle"]]
			d.end_frame = 25
		"shielddrop":
			d.events = [[1, "place", 0, -8.0, 6.4]]
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
