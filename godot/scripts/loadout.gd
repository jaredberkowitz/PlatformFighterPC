extends RefCounted
## A cosmetic loadout: what a fighter looks like. Pure presentation: nothing here reaches the simulation, and the
## checksum never sees it. A loadout is a handful of catalog indices and steps, so it packs into a few bytes that travel in
## the network handshake; an index the receiving game does not know falls back to the default for that slot, so a
## missing or newer asset never blocks a match (plan section 6). Slots are only ever added at the end: a shorter loadout
## (an older one) gives the defaults for the slots it lacks, so old looks and share codes keep working.

const FaceLook := preload("res://scripts/face_look.gd")
const VERSION := 1
## Budget: a loadout is at most this many bytes (was 16 before the face was made of parts; it is about 30 now).
const MAX_BYTES := 48

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

## Moods: the face a fighter wears when it is not doing anything in particular (the `face` slot). Each `name` is an expression in
## FaceLook.EXPRESSIONS, applied to the fighter's own eyes, brows and mouth (see FighterView.set_expression).
const FACES := [{"name": "deadpan"}, {"name": "sleepy"}, {"name": "grumpy"}, {"name": "smug"}]
## What every face turns into while the fighter is being hit (a cosmetic event, never a sim input).
const HURT := {"name": "hurt"}
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
## Gear over the bare body (the concept art's plain body is the default): shorts, then gloves, then socks and shoes, each step adding to the
## last. Looks saved before this slot existed wear the full kit, as they did.
const GEAR := ["none", "shorts", "shorts and gloves", "full kit"]
const GEAR_FULL := 3

## The slots in the order they are written, each with its catalog size. New slots go at the end (see the top).
const SLOTS := ["color", "face", "hat", "glasses", "neck", "accent", "shirt",
	"eyes", "eye_color", "eye_y", "eye_gap", "eye_size", "eye_squash", "eye_turn",
	"brows", "brow_color", "brow_y", "brow_gap", "brow_size", "brow_turn",
	"nose", "nose_y", "nose_size",
	"mouth", "mouth_y", "mouth_size", "mouth_squash",
	"cheeks", "cheek_color", "gear"]

var color := 0
var face := 0
var hat := 1
var glasses := 0
var neck := 1
var accent := 2
var shirt := 1
# The face's parts (FaceLook): the concept art's face (round eyes, no brows or nose, a small smile, round pink cheeks), every
# adjustment in the middle.
var eyes := 0
var eye_color := 0
var eye_y := FaceLook.neutral("eye_y")
var eye_gap := FaceLook.neutral("eye_gap")
var eye_size := FaceLook.neutral("eye_size")
var eye_squash := FaceLook.neutral("eye_squash")
var eye_turn := FaceLook.neutral("eye_turn")
var brows := 0
var brow_color := 0
var brow_y := FaceLook.neutral("brow_y")
var brow_gap := FaceLook.neutral("brow_gap")
var brow_size := FaceLook.neutral("brow_size")
var brow_turn := FaceLook.neutral("brow_turn")
var nose := 0
var nose_y := FaceLook.neutral("nose_y")
var nose_size := FaceLook.neutral("nose_size")
var mouth := 0
var mouth_y := FaceLook.neutral("mouth_y")
var mouth_size := FaceLook.neutral("mouth_size")
var mouth_squash := FaceLook.neutral("mouth_squash")
var cheeks := 1
var cheek_color := 0
var gear := 0


func has_shorts() -> bool:
	return gear >= 1


func has_gloves() -> bool:
	return gear >= 2


func has_shoes() -> bool:
	return gear >= 3


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
		"gear":
			return GEAR.size()
		"eyes":
			return FaceLook.EYES.size()
		"eye_color":
			return FaceLook.EYE_COLORS.size()
		"brows":
			return FaceLook.BROWS.size()
		"brow_color":
			return FaceLook.BROW_COLORS.size()
		"nose":
			return FaceLook.NOSES.size()
		"mouth":
			return FaceLook.MOUTHS.size()
		"cheeks":
			return FaceLook.CHEEKS.size()
		"cheek_color":
			return FaceLook.CHEEK_COLORS.size()
	return FaceLook.steps(slot)


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
		"gear":
			return GEAR
		"eyes":
			return FaceLook.EYES
		"eye_color":
			return FaceLook.EYE_COLOR_NAMES
		"brows":
			return FaceLook.BROWS
		"brow_color":
			return FaceLook.BROW_COLOR_NAMES
		"nose":
			return FaceLook.NOSES
		"mouth":
			return FaceLook.MOUTHS
		"cheeks":
			return FaceLook.CHEEKS
		"cheek_color":
			return FaceLook.CHEEK_COLOR_NAMES
	if FaceLook.ADJUST.has(slot):
		# Adjustments read as steps from the middle: -3 .. 0 .. +3.
		var out := []
		for i in FaceLook.steps(slot):
			var d := i - FaceLook.neutral(slot)
			out.append(("+%d" % d) if d > 0 else str(d))
		return out
	return []


## Each player's starting look: the concept art's bare body in the player's colour (no shirt, neckwear or gear), told apart by a hat and
## glasses as before.
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
	l.neck = 0
	l.face = 0
	l.accent = 2
	l.shirt = 0
	l.gear = 0
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
	# (A look from before the gear slot was dressed in the full kit: it keeps it.)
	if b.size() <= SLOTS.find("gear") + 1:
		l.gear = GEAR_FULL
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
