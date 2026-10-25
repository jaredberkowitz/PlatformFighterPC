extends RefCounted
## A small set of damped springs for secondary motion (docs/MOTION_POLISH_PLAN.md phase 2a): the head and spine lagging the hips, the
## hand's follow-through after a strike, the hat and the neckwear swinging. Each channel is a value pulled back to rest, kicked by what the
## fighter does (a change of velocity, a turn, a landing, a strike ending), with its own stiffness, damping and limits.
##
## Built for a view that only ever sees the latest snapshot: `begin` takes the sim frame each time, and a jump in it (a rollback
## correction, a replay seek, a respawn) puts every channel back to rest instead of turning the jump into a kick. Nothing moves while the
## fighter is frozen by hitlag or the game is paused (the sim frame is not advancing), so a freeze holds the pose with everything else.

## Seconds without a new sim frame after which the game counts as paused.
const PAUSED_AFTER := 0.05
## Seconds per integration sub-step.
const SUB_STEP := 0.008


class Channel:
	var stiffness: float
	var damping: float
	var low: float
	var high: float
	var x := 0.0
	var v := 0.0

	func _init(k: float, d: float, lo: float, hi: float) -> void:
		stiffness = k
		damping = d
		low = lo
		high = hi

	func rest() -> void:
		x = 0.0
		v = 0.0


var channels := {}
var frame := -1
## How many new sim frames this update brings (0 when frozen, paused or after a jump), and whether time flows now.
var new_frames := 0
var flowing := false
## Whether this update's frame jumped (a rollback correction, a replay seek, a respawn): every channel was put back to rest.
var jumped := false
var _still_for := 0.0


## Adds a channel: its stiffness, damping and the range its value is kept in.
func add(name: String, stiffness: float, damping: float, low: float, high: float) -> void:
	channels[name] = Channel.new(stiffness, damping, low, high)


func value(name: String) -> float:
	return (channels[name] as Channel).x


## Call once per rendered frame before kicking: `sim_frame` the snapshot's frame, `hitlag` its hitlag. A backward jump or one of more than
## two frames rests every channel.
func begin(sim_frame: int, hitlag: int, delta: float) -> void:
	new_frames = 0
	jumped = frame >= 0 and (sim_frame < frame or sim_frame - frame > 2)
	if jumped:
		for c in channels.values():
			(c as Channel).rest()
	elif frame >= 0:
		new_frames = sim_frame - frame
	frame = sim_frame
	_still_for = 0.0 if new_frames > 0 else _still_for + delta
	if hitlag > 0:
		new_frames = 0
	flowing = hitlag == 0 and _still_for < PAUSED_AFTER


## Adds `dv` to a channel's speed (only on a frame that moved on, so a kick lands once per sim frame).
func kick(name: String, dv: float) -> void:
	if new_frames > 0:
		(channels[name] as Channel).v += dv


## Moves every channel on by `delta` seconds (nothing while frozen or paused).
func step(delta: float) -> void:
	if not flowing:
		return
	var steps := ceili(delta / SUB_STEP)
	var h := delta / maxi(steps, 1)
	for c in channels.values():
		var ch := c as Channel
		for _i in steps:
			ch.v += (-ch.stiffness * ch.x - ch.damping * ch.v) * h
			ch.x += ch.v * h
		if ch.x < ch.low or ch.x > ch.high:
			ch.x = clampf(ch.x, ch.low, ch.high)
			ch.v = 0.0


## Every channel back to rest (a rebuild, a respawn).
func reset() -> void:
	for c in channels.values():
		(c as Channel).rest()
	frame = -1
