extends RefCounted
## A cosmetic loadout: what a fighter looks like. Pure presentation: nothing here reaches the simulation, and the
## checksum never sees it. A loadout is a handful of catalog indices, so it packs into a few bytes that travel in
## the network handshake; an index the receiving game does not know falls back to the default for that slot, so a
## missing or newer asset never blocks a match (plan section 6).

const VERSION := 1
## Hard budget (plan 7.3): a loadout is at most this many bytes.
const MAX_BYTES := 16

const PALETTE := [
	Color(1.0, 0.85, 0.2),    # sunny
	Color(0.98, 0.58, 0.18),  # tangerine
	Color(0.25, 0.45, 0.92),  # cobalt
	Color(0.95, 0.5, 0.72),   # bubblegum
	Color(0.4, 0.78, 0.4),    # meadow
	Color(0.62, 0.42, 0.88),  # plum
	Color(0.9, 0.3, 0.3),     # cherry
	Color(0.35, 0.8, 0.82),   # lagoon
	Color(0.92, 0.92, 0.88),  # cloud
	Color(0.35, 0.35, 0.42),  # slate
	Color(0.75, 0.55, 0.35),  # biscuit
	Color(0.2, 0.2, 0.28),    # midnight
]
const PALETTE_NAMES := ["sunny", "tangerine", "cobalt", "bubblegum", "meadow", "plum", "cherry", "lagoon", "cloud", "slate", "biscuit", "midnight"]

## Faces. Each `name` is a drawing, godot/art/faces/<name>.svg, shown on the head (see FighterView.set_expression). The numbers
## (lids, mouth, brow) describe the drawing and are kept for older tools.
const FACES := [
	{"name": "deadpan", "lid": 0.5, "mouth_w": 0.14, "mouth_h": 0.05, "mouth_tilt": 0.0, "brow": 0.0},
	{"name": "sleepy", "lid": 0.82, "mouth_w": 0.1, "mouth_h": 0.04, "mouth_tilt": -4.0, "brow": -8.0},
	{"name": "grumpy", "lid": 0.55, "mouth_w": 0.16, "mouth_h": 0.045, "mouth_tilt": -14.0, "brow": 24.0},
	{"name": "smug", "lid": 0.62, "mouth_w": 0.18, "mouth_h": 0.05, "mouth_tilt": 14.0, "brow": -10.0},
]
## What every face turns into while the fighter is being hit (a cosmetic event, never a sim input).
const HURT := {"name": "hurt", "lid": 0.0, "mouth_w": 0.12, "mouth_h": 0.16, "mouth_tilt": 0.0, "brow": -16.0}
## Faces every fighter pulls in action, whatever its own face (cosmetic, like the hurt face): a yell while attacking, gritted teeth while
## straining (shielding, hanging on, charging a smash), a focused look while running and jumping, a grin when it wins.
const ATTACK_FACE := {"name": "attack"}
const EFFORT_FACE := {"name": "effort"}
const FOCUS_FACE := {"name": "focus"}
## Spiral eyes: sent flying by a big hit, or a broken shield. Wide eyes and a little round mouth: caught in a grab.
const DAZED_FACE := {"name": "dazed"}
const SHOCK_FACE := {"name": "shock"}
const HAPPY_FACE := {"name": "happy"}

const HATS := ["none", "sailor cap", "aviator cap", "straw hat", "beanie", "crown"]
const GLASSES := ["none", "shades", "goggles", "round specs"]
const NECKS := ["none", "sash", "neckerchief", "scarf"]
## Shirts: a plain white one, one in the outfit (accent) colour, stripes and a flower print (godot/art/cloth/).
const SHIRTS := ["none", "white shirt", "outfit shirt", "striped shirt", "flower shirt"]

## The slots in the order they are written, each with its catalog size.
const SLOTS := ["color", "face", "hat", "glasses", "neck", "accent", "shirt"]

var color := 0
var face := 0
var hat := 1
var glasses := 0
var neck := 1
var accent := 2
var shirt := 1


static func slot_size(slot: String) -> int:
	match slot:
		"color", "accent":
			return PALETTE.size()
		"face":
			return FACES.size()
		"hat":
			return HATS.size()
		"glasses":
			return GLASSES.size()
		"neck":
			return NECKS.size()
		"shirt":
			return SHIRTS.size()
	return 1


static func slot_names(slot: String) -> Array:
	match slot:
		"color", "accent":
			return PALETTE_NAMES
		"face":
			return FACES.map(func(f): return f.name)
		"hat":
			return HATS
		"glasses":
			return GLASSES
		"neck":
			return NECKS
		"shirt":
			return SHIRTS
	return []


## The look each player had before loadouts existed, so nothing changes until someone edits theirs.
static func default_for(player: int) -> RefCounted:
	var l: RefCounted = load("res://scripts/loadout.gd").new()
	l.color = player % 4
	match player % 4:
		0, 2:
			l.hat = 1
		1:
			l.hat = 2
		3:
			l.hat = 3
	l.glasses = 1 if player % 4 == 1 or player % 4 == 2 else 0
	l.neck = 1
	l.face = 0
	l.accent = 2
	l.shirt = [1, 4, 1, 3][player % 4]
	return l


func get_slot(slot: String) -> int:
	return get(slot)


func set_slot(slot: String, value: int) -> void:
	set(slot, clampi(value, 0, slot_size(slot) - 1))


func to_bytes() -> PackedByteArray:
	var b := PackedByteArray([VERSION])
	for s in SLOTS:
		b.append(get(s))
	return b


## Reads a loadout from bytes of any origin. Anything missing, too long or out of range becomes the default for
## that slot, so a bad or newer loadout still gives a valid fighter.
static func from_bytes(b: PackedByteArray, player := 0) -> RefCounted:
	var l: RefCounted = default_for(player)
	if b.size() < 1 or b.size() > MAX_BYTES or b[0] != VERSION:
		return l
	for i in SLOTS.size():
		if i + 1 < b.size():
			var v := int(b[i + 1])
			if v < slot_size(SLOTS[i]):
				l.set(SLOTS[i], v)
	return l


## A short text a player can paste to a friend: the bytes in hex.
func to_code() -> String:
	return to_bytes().hex_encode()


static func from_code(code: String, player := 0) -> RefCounted:
	var text := code.strip_edges()
	if text.length() % 2 != 0 or not text.is_valid_hex_number(false) and text != "":
		return default_for(player)
	return from_bytes(text.hex_decode(), player)


func accent_color() -> Color:
	return PALETTE[accent]


func body_color() -> Color:
	return PALETTE[color]


static func path_for(slot_number: int) -> String:
	return "user://loadout_p%d.dat" % (slot_number + 1)


func save(slot_number: int) -> bool:
	var f := FileAccess.open(path_for(slot_number), FileAccess.WRITE)
	if f == null:
		return false
	f.store_buffer(to_bytes())
	return true


## The saved loadout for a player slot, or that player's default if there is none (or it cannot be read).
static func load_saved(slot_number: int) -> RefCounted:
	var f := FileAccess.open(path_for(slot_number), FileAccess.READ)
	if f == null:
		return default_for(slot_number)
	return from_bytes(f.get_buffer(MAX_BYTES + 1), slot_number)


func equals(other: RefCounted) -> bool:
	return to_bytes() == other.to_bytes()
