extends RefCounted

const Bindings := preload("res://scripts/bindings.gd")
## Turns keyboard and gamepad state into the sim's input: stick (-127..127 each axis) plus a
## button mask. Bindings live here, not in the sim.
##
## Keyboard keys come from `bindings.gd` (the Controls screen changes them). Defaults:
## Player 1: WASD stick, Left Ctrl = gentle tilt (slow walk), Space jump, N short hop,
##   J attack, I strong (smash) attack, K special, L (or Left Shift) shield, U grab.
## Player 2: Arrows stick, Backslash = gentle tilt, Enter jump, apostrophe short hop,
##   comma attack, semicolon strong attack, period special, slash shield, M grab.
##
## A keyboard direction is digital, so it is shaped to behave like a thumb on a stick:
##   - A first press WALKS: it starts at a light tilt and ramps up over a few frames. Holding keeps
##     walking (at full tilt in the air, so drift builds to full speed). Tapping briefly in the air
##     gives a small nudge, not a full drift.
##   - A second press within DASH_TAP_FRAMES of the previous press (double tap, or tapping the other
##     direction while dash dancing) is full strength at once, which is a flick: it dashes.
##   - Down: tap = soft (crouch, drop through), double tap = hard (fast fall).
## Gamepads: left stick as is (analog); a hard flick down fast falls; A/Y jump, X attack, B special,
## bumpers/triggers shield, right stick click grab, right stick = strong (smash) attack in that direction.

const TILT := 0.45  # stick magnitude while the tilt key is held
## A connected controller only overrides the keyboard once its stick is pushed past this, so a
## drifting stick cannot hijack the keyboard axes.
const PAD_OVERRIDE := 0.25

## Walking keyboard press: starts here and ramps linearly to full tilt over RAMP_FRAMES. It must start
## above the sim's flick start (0.3) so the ramp can never register as a dash flick.
const WALK_START := 0.34
const RAMP_FRAMES := 14
## A press this soon after the previous direction press is a double tap (full strength, dashes).
const DASH_TAP_FRAMES := 18
## The short hop key holds jump for this many frames (the sim's jump squat is 3).
const SHORT_HOP_FRAMES := 2

static var _x_state := {}
static var _hop_state := {}
static var _down_state := {}

## Keyboard "down" is a soft press (about half tilt) so it can crouch, drop through platforms and
## shield-drop, but it does NOT fast fall. A second tap within DOUBLE_TAP_FRAMES is a hard press
## (full tilt at once), which is what fast fall listens for, like flicking or hard-pressing a stick.
## Holding down ramps to full tilt after SOFT_HOLD_FRAMES. That is a slow roll, not a hard press.
const SOFT_DOWN := 0.55
const SOFT_HOLD_FRAMES := 4
const DOUBLE_TAP_FRAMES := 26  # about 0.43 s between the two presses


static func _axis(neg: bool, pos: bool) -> float:
	return (1.0 if pos else 0.0) - (1.0 if neg else 0.0)


## Returns the down amount (0..1) for a held/released key. See the notes above.
static func _down_key(id: String, down: bool, force_hard := false) -> float:
	var s: Dictionary = _down_state.get(id, {"was": false, "held": 0, "since": 1000, "double": false})
	s.since = mini(s.since + 1, 1000)
	if down and not s.was:
		s.double = force_hard or s.since <= DOUBLE_TAP_FRAMES
		s.since = 0
		s.held = 0
	if down:
		s.held += 1
	s.was = down
	_down_state[id] = s
	if not down:
		return 0.0
	if s.double or s.held > SOFT_HOLD_FRAMES:
		return 1.0
	return SOFT_DOWN


## Horizontal keyboard axis, shaped as described at the top. Call once per frame per player. The most
## recently pressed direction wins when both are held (so dash dancing never stalls at neutral).
static func _x_key(id: String, neg: bool, pos: bool, force_hard := false) -> float:
	var s: Dictionary = _x_state.get(id, {"dir": 0, "held": 0, "since": 1000, "hard": false, "pn": false, "pp": false, "last": 0})
	s.since = mini(s.since + 1, 1000)
	var pressed := 0
	if neg and not s.pn:
		pressed = -1
	if pos and not s.pp:
		pressed = 1 if pressed == 0 else 0  # both keys landing on one frame: ignore
	if pressed != 0:
		s.last = pressed
	s.pn = neg
	s.pp = pos
	var dir := 0
	if neg and pos:
		dir = s.last
	elif neg:
		dir = -1
	elif pos:
		dir = 1
	if dir == 0:
		s.dir = 0
		s.held = 0
		s.hard = false
	elif pressed != 0 and pressed == dir:
		# A new press: full strength if it follows another press closely (double tap / reversal).
		s.hard = force_hard or s.since <= DASH_TAP_FRAMES
		s.since = 0
		s.held = 1
		s.dir = dir
	elif dir != s.dir:
		# The direction changed without a new press (the other key was let go): walk on from the start.
		s.hard = false
		s.held = 1
		s.dir = dir
	else:
		s.held += 1
	_x_state[id] = s
	if dir == 0:
		return 0.0
	if s.hard:
		return float(dir)
	var t := clampf(float(s.held - 1) / float(RAMP_FRAMES), 0.0, 1.0)
	return float(dir) * lerpf(WALK_START, 1.0, t)


## Short hop key: jump held for SHORT_HOP_FRAMES frames from each press, whatever the key does after.
static func _short_hop(id: String, down: bool) -> bool:
	var s: Dictionary = _hop_state.get(id, {"was": false, "left": 0})
	if down and not s.was:
		s.left = SHORT_HOP_FRAMES
	s.was = down
	var on: bool = s.left > 0
	if on:
		s.left -= 1
	_hop_state[id] = s
	return on


static func _key(k: Key) -> bool:
	return Input.is_physical_key_pressed(k)


## Is the key bound to `action` for this player held?
static func _held(player: int, action: String) -> bool:
	return Input.is_physical_key_pressed(Bindings.key(player, action) as Key)


## Returns {x, y, buttons}. `masks` maps button names to the sim's bit values.
static func read(player: int, masks: Dictionary) -> Dictionary:
	var sx := 0.0
	var sy := 0.0
	var b := 0
	if player == 0 or player == 1:
		var id := "p%d" % player
		# With the shield up, a direction press is a flick (roll) and a down press is hard (spot dodge).
		var shielding := _held(player, "shield") or (player == 0 and Input.is_physical_key_pressed(KEY_SHIFT))
		sx = _x_key(id + "x", _held(player, "left"), _held(player, "right"), shielding)
		sy = 1.0 if _held(player, "up") else -_down_key(id, _held(player, "down"), shielding)
		if _held(player, "tilt"):
			sx = signf(sx) * TILT
			sy *= TILT
		if _held(player, "jump") or _short_hop(id + "h", _held(player, "hop")): b |= masks.jump
		if _held(player, "attack"): b |= masks.attack
		if _held(player, "strong"): b |= masks.attack | masks.strong
		if _held(player, "special"): b |= masks.special
		if shielding: b |= masks.shield
		if _held(player, "grab"): b |= masks.grab

	var pad := player
	if pad in Input.get_connected_joypads():
		var px := Input.get_joy_axis(pad, JOY_AXIS_LEFT_X)
		var py := -Input.get_joy_axis(pad, JOY_AXIS_LEFT_Y)
		if absf(px) > PAD_OVERRIDE or absf(py) > PAD_OVERRIDE:
			sx = px
			sy = py
		if Input.is_joy_button_pressed(pad, JOY_BUTTON_A) or Input.is_joy_button_pressed(pad, JOY_BUTTON_Y):
			b |= masks.jump
		if Input.is_joy_button_pressed(pad, JOY_BUTTON_X): b |= masks.attack
		if Input.is_joy_button_pressed(pad, JOY_BUTTON_B): b |= masks.special
		if Input.is_joy_button_pressed(pad, JOY_BUTTON_LEFT_SHOULDER) or Input.is_joy_button_pressed(pad, JOY_BUTTON_RIGHT_SHOULDER):
			b |= masks.shield
		if Input.get_joy_axis(pad, JOY_AXIS_TRIGGER_LEFT) > 0.5 or Input.get_joy_axis(pad, JOY_AXIS_TRIGGER_RIGHT) > 0.5:
			b |= masks.shield
		if Input.is_joy_button_pressed(pad, JOY_BUTTON_RIGHT_STICK): b |= masks.grab
		# The right stick is the smash stick: pushed, it attacks hard in that direction.
		var rx := Input.get_joy_axis(pad, JOY_AXIS_RIGHT_X)
		var ry := -Input.get_joy_axis(pad, JOY_AXIS_RIGHT_Y)
		if Vector2(rx, ry).length() > 0.65:
			b |= masks.attack | masks.strong
			sx = rx
			sy = ry

	# Digital diagonals would be a longer vector than a physical stick can make; clamp to the unit circle.
	var len := Vector2(sx, sy).length()
	if len > 1.0:
		sx /= len
		sy /= len
	return {"x": roundi(sx * 127.0), "y": roundi(sy * 127.0), "buttons": b}
