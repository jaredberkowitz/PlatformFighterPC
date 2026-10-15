extends RefCounted
## Controller settings for the four controllers, kept between runs (`user://pad.json`). The Controls screen's Controller page edits
## them; `input_reader.gd` reads them. The simulation never sees buttons: only the stick and buttons that come out of the reader.
##
## Each action has a list of inputs: a controller button (its Godot `JoyButton` number) or a trigger (`TRIGGER_LEFT` / `TRIGGER_RIGHT`,
## pulled past half way). Options: tap jump (an up flick of the left stick jumps; off by default, owner decision 2026-10-14), what
## the right stick does (smash attacks, tilt attacks, specials or nothing), and rumble.

const PATH := "user://pad.json"
const PADS := 4

const TRIGGER_LEFT := 100
const TRIGGER_RIGHT := 101

## Actions in the order the Controller page lists them, with a label for each.
const ACTIONS := [
	["jump", "Jump"], ["hop", "Short hop"], ["attack", "Attack"], ["strong", "Strong attack"], ["special", "Special"],
	["shield", "Shield"], ["grab", "Grab"],
]

## A fresh controller: A or Y jump, B attack, X special, the triggers shield, the bumpers (and a right-stick click) grab, as the
## reference puts grab on the shoulder buttons and shield on the triggers.
const DEFAULT_MAP := {
	"jump": [JOY_BUTTON_A, JOY_BUTTON_Y], "hop": [], "attack": [JOY_BUTTON_B], "strong": [], "special": [JOY_BUTTON_X],
	"shield": [TRIGGER_LEFT, TRIGGER_RIGHT], "grab": [JOY_BUTTON_LEFT_SHOULDER, JOY_BUTTON_RIGHT_SHOULDER, JOY_BUTTON_RIGHT_STICK],
}

## The options, in the order the Controller page lists them after the actions: [id, label, choices].
const OPTIONS := [
	["tap_jump", "Tap jump", ["Off", "On"]],
	["right_stick", "Right stick", ["Strong attacks", "Tilt attacks", "Specials", "Off"]],
	["rumble", "Rumble", ["On", "Off"]],
]
const RIGHT_SMASH := 0
const RIGHT_TILT := 1
const RIGHT_SPECIAL := 2
const RIGHT_OFF := 3

## Inputs the menus keep for themselves on every controller (Start and Back); they cannot be bound.
const RESERVED := [JOY_BUTTON_START, JOY_BUTTON_BACK, JOY_BUTTON_GUIDE]

static var _table: Array = []
static var _loaded := false


static func _fresh_pad() -> Dictionary:
	var map := {}
	for a in ACTIONS:
		map[a[0]] = (DEFAULT_MAP[a[0]] as Array).duplicate()
	return {"map": map, "tap_jump": 0, "right_stick": RIGHT_SMASH, "rumble": 0}


static func _ensure() -> void:
	if _loaded:
		return
	_loaded = true
	_table = []
	for p in PADS:
		_table.append(_fresh_pad())
	if not FileAccess.file_exists(PATH):
		return
	var json := JSON.new()
	if json.parse(FileAccess.get_file_as_string(PATH)) != OK or not (json.data is Array):
		return
	var parsed: Array = json.data
	for p in mini(parsed.size(), PADS):
		var saved = parsed[p]
		if not (saved is Dictionary):
			continue
		if saved.get("map") is Dictionary:
			for a in ACTIONS:
				var list = saved.map.get(a[0])
				if list is Array:
					var clean := []
					for code in list:
						var c := int(code)
						if _valid(c) and not clean.has(c):
							clean.append(c)
					_table[p].map[a[0]] = clean
		for o in OPTIONS:
			if saved.has(o[0]):
				_table[p][o[0]] = clampi(int(saved[o[0]]), 0, (o[2] as Array).size() - 1)
	# A damaged or hand-edited file must not leave one input on two actions: keep the first.
	for p in PADS:
		var seen := {}
		for a in ACTIONS:
			var kept := []
			for c in _table[p].map[a[0]]:
				if not seen.has(c):
					kept.append(c)
					seen[c] = true
			_table[p].map[a[0]] = kept


static func _valid(code: int) -> bool:
	return (code == TRIGGER_LEFT or code == TRIGGER_RIGHT or (code >= 0 and code < JOY_BUTTON_MAX)) and not RESERVED.has(code)


static func _pad(pad: int) -> Dictionary:
	_ensure()
	return _table[clampi(pad, 0, PADS - 1)]


## The inputs bound to `action` on controller `pad`.
static func inputs(pad: int, action: String) -> Array:
	return _pad(pad).map.get(action, [])


## An option's chosen index (see `OPTIONS`).
static func option(pad: int, id: String) -> int:
	return int(_pad(pad).get(id, 0))


static func set_option(pad: int, id: String, value: int) -> void:
	for o in OPTIONS:
		if o[0] == id:
			_pad(pad)[id] = posmod(value, (o[2] as Array).size())
			save()
			return


static func tap_jump(pad: int) -> bool:
	return option(pad, "tap_jump") == 1


static func right_stick(pad: int) -> int:
	return option(pad, "right_stick")


static func rumble(pad: int) -> bool:
	return option(pad, "rumble") == 0


## Binds `code` to `action` alone (its other inputs are let go), taking it off any other action of the same controller. Returns false
## (and changes nothing) for a reserved or unknown input.
static func set_input(pad: int, action: String, code: int) -> bool:
	if not _valid(code):
		return false
	var map: Dictionary = _pad(pad).map
	for a in ACTIONS:
		(map[a[0]] as Array).erase(code)
	map[action] = [code]
	save()
	return true


static func reset(pad := -1) -> void:
	_ensure()
	for p in PADS:
		if pad < 0 or pad == p:
			_table[p] = _fresh_pad()
	save()


static func save() -> void:
	_ensure()
	var f := FileAccess.open(PATH, FileAccess.WRITE)
	if f != null:
		f.store_string(JSON.stringify(_table))


## Whether any input bound to `action` is held, given the controller's buttons and triggers (see `input_reader.gd`'s raw state).
static func held_in(pad: int, action: String, buttons: Dictionary, triggers: Vector2) -> bool:
	for c in inputs(pad, action):
		if c == TRIGGER_LEFT:
			if triggers.x > 0.5:
				return true
		elif c == TRIGGER_RIGHT:
			if triggers.y > 0.5:
				return true
		elif buttons.get(c, false):
			return true
	return false


## A short name for an input, as the Controller page shows it (Xbox names; other controllers' buttons sit in the same places).
static func input_name(code: int) -> String:
	match code:
		TRIGGER_LEFT: return "LT"
		TRIGGER_RIGHT: return "RT"
		JOY_BUTTON_A: return "A"
		JOY_BUTTON_B: return "B"
		JOY_BUTTON_X: return "X"
		JOY_BUTTON_Y: return "Y"
		JOY_BUTTON_LEFT_SHOULDER: return "LB"
		JOY_BUTTON_RIGHT_SHOULDER: return "RB"
		JOY_BUTTON_LEFT_STICK: return "L stick click"
		JOY_BUTTON_RIGHT_STICK: return "R stick click"
		JOY_BUTTON_DPAD_UP: return "D-pad up"
		JOY_BUTTON_DPAD_DOWN: return "D-pad down"
		JOY_BUTTON_DPAD_LEFT: return "D-pad left"
		JOY_BUTTON_DPAD_RIGHT: return "D-pad right"
	return "Button %d" % code


static func inputs_text(pad: int, action: String) -> String:
	var names := []
	for c in inputs(pad, action):
		names.append(input_name(c))
	return " / ".join(names) if not names.is_empty() else "-"


## Forget what was loaded, so the next read starts from the file again (tests).
static func unload() -> void:
	_loaded = false
	_table = []
