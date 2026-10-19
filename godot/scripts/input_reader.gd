extends RefCounted

const Bindings := preload("res://scripts/bindings.gd")
const PadBindings := preload("res://scripts/pad_bindings.gd")
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
##   - Down: tap = soft (crouch, down tilt), double tap = hard (fast fall; on a platform, drop through).
## Gamepads: left stick as is (analog); a hard flick down fast falls. Buttons come from `pad_bindings.gd` (the Controls screen's
## Controller page changes them). Defaults: A/Y jump, B attack, X special, triggers shield, bumpers (and a right-stick click) grab.
## The right stick is by default the attack stick (the reference's C-stick): a smash in its direction on the ground, an aerial in the
## air; it only aims the attack, and the left stick keeps moving the fighter (drift, fast fall). It can instead give tilt attacks or
## specials, or do nothing. Tap jump (an up flick of the left stick jumps) is an option, off by default.

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

## Keyboard "down" is a soft press (about half tilt) so it can crouch and down tilt (on a platform too), but it does NOT
## fast fall or drop through. A second tap within DOUBLE_TAP_FRAMES is a hard press (full tilt at once), which is what fast
## fall and, on a platform, the drop through it listen for (sim 144), like flicking a stick.
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


## Returns {x, y, buttons}. `masks` maps button names to the sim's bit values. `keeps_dash`: the fighter is in lag that keeps a dash
## flicked during it (a grab, a throw or a landing; the bridge's `fighter_keeps_dash`), so a direction key pressed now is sent at full
## strength, a flick, and the dash comes out the moment the lag ends. (A key's first press otherwise starts a walk, which the sim never
## reads as a dash.)
static func read(player: int, masks: Dictionary, keeps_dash := false) -> Dictionary:
	var sx := 0.0
	var sy := 0.0
	var b := 0
	if player == 0 or player == 1:
		var id := "p%d" % player
		# With the shield up, a direction press is a flick (roll) and a down press is hard (spot dodge).
		var shielding := _held(player, "shield") or (player == 0 and Input.is_physical_key_pressed(KEY_SHIFT))
		sx = _x_key(id + "x", _held(player, "left"), _held(player, "right"), shielding or keeps_dash)
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
		var buttons := {}
		for c in JOY_BUTTON_MAX:
			if Input.is_joy_button_pressed(pad, c):
				buttons[c] = true
		var raw := {
			"lx": Input.get_joy_axis(pad, JOY_AXIS_LEFT_X), "ly": -Input.get_joy_axis(pad, JOY_AXIS_LEFT_Y),
			"rx": Input.get_joy_axis(pad, JOY_AXIS_RIGHT_X), "ry": -Input.get_joy_axis(pad, JOY_AXIS_RIGHT_Y),
			"buttons": buttons,
			"triggers": Vector2(Input.get_joy_axis(pad, JOY_AXIS_TRIGGER_LEFT), Input.get_joy_axis(pad, JOY_AXIS_TRIGGER_RIGHT)),
		}
		var r := from_pad(pad, raw, masks, Vector2(sx, sy))
		sx = r.x
		sy = r.y
		b |= r.buttons

	# Digital diagonals would be a longer vector than a physical stick can make; clamp to the unit circle.
	var len := Vector2(sx, sy).length()
	if len > 1.0:
		sx /= len
		sy /= len
	return {"x": roundi(sx * 127.0), "y": roundi(sy * 127.0), "buttons": b}


## Per-controller memory for tap jump and the right stick's press.
static var _pad_state := {}
## The left stick counts as up past this; an up push counts as a flick (tap jump) if the stick was below `TAP_FROM` within the last
## `TAP_FRAMES` frames; the jump is held while the stick stays above `TAP_HOLD` (so holding up is a full hop, a quick flick a short one).
const TAP_UP := 0.66
const TAP_FROM := 0.3
const TAP_FRAMES := 3
const TAP_HOLD := 0.5
## The right stick counts as pushed past this.
const RIGHT_PUSH := 0.65


## A controller's state turned into the sim's input. `raw` holds the sticks (`lx`, `ly`, `rx`, `ry`, up positive), `buttons` (button
## number -> held) and `triggers` (left, right). `kb` is the keyboard's stick, which the left stick replaces once pushed past
## `PAD_OVERRIDE`. Returns {x, y (-1..1), buttons}. Pure apart from the per-controller memory, so tests can drive it.
static func from_pad(pad: int, raw: Dictionary, masks: Dictionary, kb := Vector2.ZERO) -> Dictionary:
	var s: Dictionary = _pad_state.get(pad, {"ys": [], "tap": false, "right": false})
	var sx := kb.x
	var sy := kb.y
	var lx: float = raw.get("lx", 0.0)
	var ly: float = raw.get("ly", 0.0)
	if absf(lx) > PAD_OVERRIDE or absf(ly) > PAD_OVERRIDE:
		sx = lx
		sy = ly
	var held: Dictionary = raw.get("buttons", {})
	var trig: Vector2 = raw.get("triggers", Vector2.ZERO)
	var b := 0
	if PadBindings.held_in(pad, "jump", held, trig): b |= masks.jump
	if _short_hop("pad%dh" % pad, PadBindings.held_in(pad, "hop", held, trig)): b |= masks.jump
	if PadBindings.held_in(pad, "attack", held, trig): b |= masks.attack
	if PadBindings.held_in(pad, "strong", held, trig): b |= masks.attack | masks.strong
	if PadBindings.held_in(pad, "special", held, trig): b |= masks.special
	if PadBindings.held_in(pad, "shield", held, trig): b |= masks.shield
	if PadBindings.held_in(pad, "grab", held, trig): b |= masks.grab
	# Tap jump: an up flick of the left stick jumps, held while the stick stays up.
	var ys: Array = s.ys
	if PadBindings.tap_jump(pad):
		if ly >= TAP_UP and not s.tap and ys.any(func(y): return y < TAP_FROM):
			s.tap = true
		if ly < TAP_HOLD:
			s.tap = false
		if s.tap:
			b |= masks.jump
	else:
		s.tap = false
	ys.push_back(ly)
	while ys.size() > TAP_FRAMES:
		ys.pop_front()
	# The right stick: pushed past `RIGHT_PUSH`, toward the nearer of its four directions.
	var rx: float = raw.get("rx", 0.0)
	var ry: float = raw.get("ry", 0.0)
	var pushed := Vector2(rx, ry).length() > RIGHT_PUSH
	var dir := Vector2.ZERO
	if pushed:
		dir = Vector2(signf(rx), 0.0) if absf(rx) >= absf(ry) else Vector2(0.0, signf(ry))
	match PadBindings.right_stick(pad):
		PadBindings.RIGHT_SMASH:
			# Sent as its own bits so the left stick's drift and fast fall carry on through the attack. (It once replaced the left
			# stick while held, which ate a fast-fall flick made during a right-stick aerial.)
			if pushed and masks.has("c_up"):
				b |= masks.attack
				if dir.x != 0.0:
					b |= masks.c_right if dir.x > 0.0 else masks.c_left
				else:
					b |= masks.c_up if dir.y > 0.0 else masks.c_down
		PadBindings.RIGHT_TILT, PadBindings.RIGHT_SPECIAL:
			# On the push's first frame only: the stick points that way (gently for a tilt, fully for a special) with the button.
			if pushed and not s.right:
				var tilt: bool = PadBindings.right_stick(pad) == PadBindings.RIGHT_TILT
				sx = dir.x * (TILT if tilt else 1.0)
				sy = dir.y * (TILT if tilt else 1.0)
				b |= masks.attack if tilt else masks.special
	s.right = pushed
	_pad_state[pad] = s
	return {"x": sx, "y": sy, "buttons": b}


## Forget the controllers' memory (tests).
static func reset_pads() -> void:
	_pad_state = {}
