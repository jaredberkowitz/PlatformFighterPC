extends RefCounted
## Turns keyboard and gamepad state into the sim's input: stick (-127..127 each axis) plus a
## button mask. Bindings live here, not in the sim.
##
## Player 1 keyboard: WASD stick, Left Ctrl = gentle tilt (walk), Space jump, J attack, K special,
##   L or Left Shift shield, U grab.
## Player 2 keyboard: Arrows stick, Backslash = gentle tilt, Enter jump, comma attack,
##   period special, slash shield, M grab.
## Gamepads: left stick; A/Y jump, X attack, B special, bumpers/triggers shield, right stick click grab.

const TILT := 0.45  # stick magnitude while the tilt key is held


static func _axis(neg: bool, pos: bool) -> float:
	return (1.0 if pos else 0.0) - (1.0 if neg else 0.0)


static func _key(k: Key) -> bool:
	return Input.is_physical_key_pressed(k)


## Returns {x, y, buttons}. `masks` maps button names to the sim's bit values.
static func read(player: int, masks: Dictionary) -> Dictionary:
	var sx := 0.0
	var sy := 0.0
	var b := 0
	if player == 0:
		sx = _axis(_key(KEY_A), _key(KEY_D))
		sy = _axis(_key(KEY_S), _key(KEY_W))
		if _key(KEY_CTRL):
			sx *= TILT
			sy *= TILT
		if _key(KEY_SPACE): b |= masks.jump
		if _key(KEY_J): b |= masks.attack
		if _key(KEY_K): b |= masks.special
		if _key(KEY_L) or _key(KEY_SHIFT): b |= masks.shield
		if _key(KEY_U): b |= masks.grab
	elif player == 1:
		sx = _axis(_key(KEY_LEFT), _key(KEY_RIGHT))
		sy = _axis(_key(KEY_DOWN), _key(KEY_UP))
		if _key(KEY_BACKSLASH):
			sx *= TILT
			sy *= TILT
		if _key(KEY_ENTER): b |= masks.jump
		if _key(KEY_COMMA): b |= masks.attack
		if _key(KEY_PERIOD): b |= masks.special
		if _key(KEY_SLASH): b |= masks.shield
		if _key(KEY_M): b |= masks.grab

	var pad := player
	if pad in Input.get_connected_joypads():
		var px := Input.get_joy_axis(pad, JOY_AXIS_LEFT_X)
		var py := -Input.get_joy_axis(pad, JOY_AXIS_LEFT_Y)
		if absf(px) > 0.08 or absf(py) > 0.08:
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

	# Digital diagonals would be a longer vector than a physical stick can make; clamp to the unit circle.
	var len := Vector2(sx, sy).length()
	if len > 1.0:
		sx /= len
		sy /= len
	return {"x": roundi(sx * 127.0), "y": roundi(sy * 127.0), "buttons": b}
