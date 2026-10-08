extends RefCounted
## Keyboard bindings for the two local players, kept between runs (`user://controls.json`). The Controls screen edits them;
## `input_reader.gd` reads them. The simulation never sees keys: only the stick and buttons that come out of the reader.

const PATH := "user://controls.json"

## Actions in the order the Controls screen lists them, with a label for each.
const ACTIONS := [
	["left", "Left"], ["right", "Right"], ["up", "Up"], ["down", "Down"],
	["tilt", "Gentle tilt (hold)"], ["jump", "Jump"], ["hop", "Short hop"],
	["attack", "Attack"], ["strong", "Strong attack"], ["special", "Special"], ["shield", "Shield"], ["grab", "Grab"],
]

## What a fresh install uses (player 1 on the left hand, player 2 on the arrows).
const DEFAULTS := [
	{
		"left": KEY_A, "right": KEY_D, "up": KEY_W, "down": KEY_S, "tilt": KEY_CTRL, "jump": KEY_SPACE, "hop": KEY_N,
		"attack": KEY_J, "strong": KEY_I, "special": KEY_K, "shield": KEY_L, "grab": KEY_U,
	},
	{
		"left": KEY_LEFT, "right": KEY_RIGHT, "up": KEY_UP, "down": KEY_DOWN, "tilt": KEY_BACKSLASH, "jump": KEY_ENTER,
		"hop": KEY_APOSTROPHE, "attack": KEY_COMMA, "strong": KEY_SEMICOLON, "special": KEY_PERIOD, "shield": KEY_SLASH, "grab": KEY_M,
	},
]

## Keys that stay free for the game and its menus; they cannot be bound.
const RESERVED := [KEY_ESCAPE, KEY_F1, KEY_F2, KEY_F3, KEY_F6, KEY_F7, KEY_F8, KEY_F9]

static var _table: Array = []
static var _loaded := false


static func _fresh() -> Array:
	return [DEFAULTS[0].duplicate(), DEFAULTS[1].duplicate()]


static func _ensure() -> void:
	if _loaded:
		return
	_loaded = true
	_table = _fresh()
	if not FileAccess.file_exists(PATH):
		return
	var json := JSON.new()
	if json.parse(FileAccess.get_file_as_string(PATH)) != OK:
		return
	var parsed = json.data
	if not (parsed is Array) or parsed.size() != 2:
		return
	for p in 2:
		if not (parsed[p] is Dictionary):
			continue
		for a in ACTIONS:
			var id: String = a[0]
			if parsed[p].has(id):
				var code := int(parsed[p][id])
				if code > 0 and not RESERVED.has(code):
					_table[p][id] = code
	# A damaged or hand-edited file must not leave one key on two actions of a player: keep the first, restore the rest.
	for p in 2:
		var seen := {}
		for a in ACTIONS:
			var id: String = a[0]
			var code: int = _table[p][id]
			if seen.has(code):
				_table[p][id] = DEFAULTS[p][id]
			seen[code] = true


static func key(player: int, action: String) -> int:
	_ensure()
	return _table[clampi(player, 0, 1)][action]


## Binds `code` to the action. If the same player already uses that key for another action, the two swap, so no key is ever
## on two actions. Returns false (and changes nothing) for a reserved key.
static func set_key(player: int, action: String, code: int) -> bool:
	_ensure()
	if RESERVED.has(code) or code <= 0:
		return false
	var t: Dictionary = _table[player]
	var old: int = t[action]
	for a in ACTIONS:
		if a[0] != action and t[a[0]] == code:
			t[a[0]] = old
	t[action] = code
	save()
	return true


static func reset(player := -1) -> void:
	_ensure()
	for p in 2:
		if player < 0 or player == p:
			_table[p] = DEFAULTS[p].duplicate()
	save()


static func save() -> void:
	_ensure()
	var f := FileAccess.open(PATH, FileAccess.WRITE)
	if f != null:
		f.store_string(JSON.stringify(_table))


static func key_name(code: int) -> String:
	return OS.get_keycode_string(code) if code > 0 else "?"


## Forget what was loaded, so the next read starts from the file again (tests).
static func unload() -> void:
	_loaded = false
	_table = []
