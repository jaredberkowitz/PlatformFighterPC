extends SceneTree
## Foot-slide probe (docs/MOTION_POLISH_PLAN.md phase 0): drives one fighter view through walk, run and dash at several ground speeds,
## the body moving along the stage at that speed, and measures how far a planted foot drifts in the world while it is down. A foot that
## is truly planted does not move (drift 0); one carried along with the body drifts at the ground speed (ratio 1). Headless is fine.
## Run: Godot --headless --path godot --script res://tests/motion_probe.gd [-- --class=0 --check=<max ratio>]
##   Prints one line per case: state, speed (world units a frame), mean drift a frame while planted, drift / speed, and the signed mean
##   (positive: the foot slides forward with the body; negative: it slides back, a skate).

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")

## Cases: [state, speeds]. Walk and run speeds span the roster's walk_speed and run_speed (content/base.pfc); dash its dash_speed.
const CASES := [
	["Walk", [0.08, 0.14, 0.19]],
	["Run", [0.19, 0.24, 0.30]],
	["Dash", [0.24, 0.27, 0.28]],
]
const DASH_LENGTH := 2.4
const WARMUP := 40
const FRAMES := 180
## A foot counts as planted within this height of its lowest point.
const PLANTED := 0.035


func _initialize() -> void:
	var cls := 0
	var check := -1.0
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--class="):
			cls = int(a.substr(8))
		elif a.begins_with("--check="):
			check = float(a.substr(8))
	var worst := 0.0
	for c in CASES:
		for speed in c[1]:
			var r := await _measure(cls, c[0], speed)
			worst = maxf(worst, r[1])
			print("%-5s speed %.3f  drift %.4f/frame  ratio %.2f  signed %+.2f" % [c[0], speed, r[0], r[1], r[2]])
	print("worst ratio %.2f" % worst)
	quit(1 if check >= 0.0 and worst > check else 0)


## [mean absolute drift a frame while planted, that over the speed, signed mean over the speed]
func _measure(cls: int, state: String, speed: float) -> Array:
	var view := FighterView.new()
	root.add_child(view)
	view.build(cls, Loadout.default_for(cls))
	await process_frame
	var feet := [view.skeleton.find_bone("foot.L"), view.skeleton.find_bone("foot.R")]
	var track := [[], []]
	var x := 0.0
	for frame in WARMUP + FRAMES:
		x += speed
		var s := {"state": state, "facing": 1, "vel": Vector2(speed, 0), "platform": 0, "fast_fall": false, "invuln": 0,
			"ledge_invuln": 0, "frame": frame + 1, "shield": 1.0, "percent": 0.0, "move_name": "", "move_tip": Vector3.ZERO,
			"move_timing": PackedInt32Array([1, 0, 0]), "charge": 0, "hitlag": 0, "tumble": false, "launch_pending": false, "char": cls,
			"class": cls, "reach": 3.0, "state_frame": frame, "jumps": 1, "stocks": 3, "dash_length": DASH_LENGTH,
			"dodge": PackedInt32Array()}
		view.apply(Vector3(x, 0, 0), s, 1.0 / 60.0)
		if frame < WARMUP:
			continue
		var to_world := view.skeleton.global_transform
		for k in 2:
			track[k].append(to_world * view.skeleton.get_bone_global_pose(feet[k]).origin)
		if OS.get_environment("PROBE_DEBUG") == state and frame < WARMUP + 40:
			print("  %d L %+.3f %.3f  R %+.3f %.3f" % [frame, track[0][-1].x - x, track[0][-1].y, track[1][-1].x - x, track[1][-1].y])
	view.queue_free()
	var total := 0.0
	var signed := 0.0
	var n := 0
	for k in 2:
		var low := INF
		for p in track[k]:
			low = minf(low, p.y)
		for i in range(1, track[k].size()):
			var a: Vector3 = track[k][i - 1]
			var b: Vector3 = track[k][i]
			if a.y < low + PLANTED and b.y < low + PLANTED:
				total += absf(b.x - a.x)
				signed += b.x - a.x
				n += 1
	if n == 0:
		return [0.0, 0.0, 0.0]
	return [total / n, total / n / speed, signed / n / speed]
