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

## (The bright colours lean a little pastel, as the concept art's warm orange does: each keeps its hue with a touch more cream in it, so
## under the stage's warm key and the fighters' saturation lift it reads soft rather than candy-bright.)
## (Muted, 2026-10-18, the owner after the reference cast: mid-value, slightly earthy versions, a warm mustard rather than lemon, a burnt
## orange, a softer blue; the lighter candy colours read as striking where the reference's are calm.)
const PALETTE := [
	Color(0.95, 0.77, 0.22),  # sunny
	Color(0.88, 0.55, 0.3),   # tangerine
	Color(0.36, 0.53, 0.83),  # cobalt
	Color(0.9, 0.58, 0.7),    # bubblegum
	Color(0.48, 0.72, 0.42),  # meadow
	Color(0.6, 0.48, 0.78),   # plum
	Color(0.84, 0.36, 0.34),  # cherry
	Color(0.42, 0.72, 0.72),  # lagoon
	Color(0.9, 0.88, 0.82),   # cloud
	Color(0.35, 0.35, 0.42),  # slate
	Color(0.72, 0.52, 0.34),  # biscuit
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

const HATS := ["none", "sailor cap", "aviator cap", "straw hat", "beanie", "crown", "hood", "plumed helm", "swept spikes", "topknot", "dixie cap",
	"racing helmet", "eared cap", "headband", "top hat", "conical hat"]
const GLASSES := ["none", "shades", "goggles", "round specs"]
const NECKS := ["none", "sash", "neckerchief", "scarf", "cape", "kerchief and sash"]
## Shirts: a plain white one, one in the outfit (accent) colour, stripes and a flower print (godot/art/cloth/).
const SHIRTS := ["none", "white shirt", "outfit shirt", "striped shirt", "flower shirt"]
## Gear over the bare body (the concept art's plain body is the default): shorts, then gloves, then socks and shoes, each step adding to the
## last. Looks saved before this slot existed wear the full kit, as they did.
const GEAR := ["none", "shorts", "shorts and gloves", "full kit"]
const GEAR_FULL := 3
## The belt with its buckle and the shorts' pocket flaps, worn over the shorts (the outfit pass, 2026-10-19: the default looks go without,
## after the owner's T-pose lineup; the belt was one of about ten bands stacked down the body). Looks saved before this slot keep theirs.
const BELTS := ["belt and pockets", "none"]
## How a fighter stands in the menus (character select, previews; never in a match, where the fists stay closed): the clip it holds and
## whether its hands are open (the character judge's round 10: the cast all held one raised-fist pose). Cosmetic only.
const POSES := ["ready", "relaxed", "wave", "guard", "broad"]
const POSE_CLIPS := {"ready": "idle", "relaxed": "idle_tool", "wave": "victory_b", "guard": "idle_look", "broad": "idle_heavy"}
const POSE_OPEN := {"relaxed": true, "broad": true}
## The head's shape: its scale across, up and front to back on the shared head. Every one is near a circle with only small differences
## (the owner, 2026-10-19; the character judge's round 12 had made them clearly different, which read too varied).
## Gloved or bare hands with the kit (the character judge's round 16: in the reference only the aviator wears gloves).
const HANDS := ["gloves", "bare"]
const HEADS := ["round", "bean", "bun", "ball", "onion"]
# (A tenth bigger again, the character judge's round 16: the reference's heads dwarf the body.)
# (Each close to a circle from the front, the owner, 2026-10-19: the varied shapes read too different; the reference's heads are all
# near-round balls. The head mesh is about 1.18 times as wide as it is tall at an even scale, so a circle is about x/1.18 up.)
const HEAD_SCALES := {"round": Vector3(1.45, 1.23, 1.36), "bean": Vector3(1.41, 1.25, 1.33), "bun": Vector3(1.49, 1.22, 1.39),
	"ball": Vector3(1.47, 1.26, 1.40), "onion": Vector3(1.43, 1.24, 1.34)}

## The slots in the order they are written, each with its catalog size. New slots go at the end (see the top).
const SLOTS := ["color", "face", "hat", "glasses", "neck", "accent", "shirt",
	"eyes", "eye_color", "eye_y", "eye_gap", "eye_size", "eye_squash", "eye_turn",
	"brows", "brow_color", "brow_y", "brow_gap", "brow_size", "brow_turn",
	"nose", "nose_y", "nose_size",
	"mouth", "mouth_y", "mouth_size", "mouth_squash",
	"cheeks", "cheek_color", "gear", "pose", "head", "face_lean", "hands", "belt", "freckles"]

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
# (Thick dark bar brows by default since 2026-10-18, after the reference cast: the browless concept face had no attitude.)
var brows := 3
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
var pose := 0
var head := 0
var hands := 0
var belt := 0
var freckles := 0
## How the face is uneven (FaceLook `face_lean` steps; the middle even): one lid lower and the mouth off to that side.
var face_lean := FaceLook.neutral("face_lean")


func has_shorts() -> bool:
	return gear >= 1


func has_gloves() -> bool:
	return gear >= 2 and hands == 0


func has_shoes() -> bool:
	return gear >= 3


func has_belt() -> bool:
	return has_shorts() and belt == 0


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
		"pose":
			return POSES.size()
		"head":
			return HEADS.size()
		"hands":
			return HANDS.size()
		"belt":
			return BELTS.size()
		"freckles":
			return FaceLook.FRECKLES.size()
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
		"pose":
			return POSES
		"head":
			return HEADS
		"hands":
			return HANDS
		"belt":
			return BELTS
		"freckles":
			return FaceLook.FRECKLES
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
		0:
			l.hat = 1
		2:
			# (The little dixie cap, as the reference's blue scout, the character pass, 2026-10-19.)
			l.hat = 10
		1:
			l.hat = 2
		3:
			l.hat = 3
	l.glasses = 1 if player % 4 == 1 or player % 4 == 2 else 0
	l.neck = 0
	# A face of its own for each default fighter (the character pass, 2026-10-19, after the owner's reference cast, where most of each
	# character's personality is its face): sleepy with an open mouth, smug with a cat mouth, grumpy with a frown, deadpan with a blep;
	# no blush (the reference's faces have none).
	# (The ranger smug with droopy sideways eyes and a smirk, the sailor with a curl on its forehead in place of brows, the character judge's
	# round 5 after the reference's ranger and sailor.)
	# (The sailor deadpan with a flat mouth at rest, the owner, 2026-10-19: sleepy with an open mouth read as dazed; that look is its hurt face.)
	# (Since the reference pass, 2026-10-20, the owner's pick of mocked faces: every default the reference's round eye, varied by its mood;
	# the sailor with the salute's toothy grin, the aviator sleepy.)
	var faces := [[0, 6, 0], [1, 4, 0], [2, 7, 0], [3, 5, 0]]
	var f: Array = faces[player % 4]
	l.face = f[0]
	l.mouth = f[1]
	l.eyes = f[2]
	l.cheeks = 0
	# (The aviator scout's hair ginger, as the reference's: the hair takes the brows' colour.)
	l.brow_color = 4 if player % 4 == 1 else 0
	# (No brows on the defaults since the reference pass, 2026-10-20, the owner; they stay in the creator. The sailor's forehead curl and
	# the others' bars before.)
	l.brows = 0
	l.brow_y = FaceLook.neutral("brow_y")
	l.brow_gap = FaceLook.neutral("brow_gap")
	l.brow_size = FaceLook.neutral("brow_size")
	# Dressed, as the reference cast always is (the character pass, 2026-10-19; bare, the defaults read as naked blobs in a match):
	# [shirt, neck, accent, glasses]: a sailor (white shirt, neckerchief, cobalt), an aviator scout (white shirt, sash, meadow green), a
	# holiday shirt (flowers, sash, biscuit), a ranger (outfit shirt, sash, sunny); every one in the full kit (shorts, gloves, socks, shoes).
	# (The sailor in the kerchief and sash, neck 5, as the reference's, the character judge's round 5.)
	# (The holiday shirt on cream (cloud), so its red hibiscus print reads as the reference's, the character judge's round 5.)
	var outfits := [[1, 5, 2, 0], [1, 1, 4, 1], [4, 1, 8, 1], [2, 1, 0, 0]]
	var o: Array = outfits[player % 4]
	l.shirt = o[0]
	l.neck = o[1]
	l.accent = o[2]
	l.glasses = o[3]
	# (Shorts and bare legs and feet in the body's colour since the outfit pass, 2026-10-19, the owner's choice after the T-pose lineup:
	# no socks, boots, belt or pocket flaps; the aviator keeps its gloves (`hands`). The full kit stays in the loadout.)
	l.gear = 2
	l.belt = 1
	# (Each default fighter its own stance in the menus: the sailor relaxed, the aviator waving, the blue on guard, the ranger broad.)
	l.pose = [1, 2, 3, 4][player % 4]
	# (A head and an uneven face of its own each: the sailor a bean drooping left, the aviator a wide bun, the blue a ball frowning right,
	# the ranger an onion with a lazy right lid.)
	l.head = [1, 2, 3, 4][player % 4]
	# (Bare hands in the body's colour but for the aviator's gloves, as the reference's cast.)
	l.hands = 0 if player % 4 == 1 else 1
	# (Even since the reference pass, 2026-10-20, the owner: the mouth off to one side looked odd. The slot stays in the creator.)
	l.face_lean = FaceLook.neutral("face_lean")
	# (Each fighter's signature pushed, the character judge's round 13: the aviator's freckles, the blue's heavy brows and deep frown, the
	# ranger's thick block brows.)
	# (The aviator's freckles their own part since the face pass, 2026-10-19: bold dark dots over a soft round blush, as the reference's.)
	l.cheeks = 1 if player % 4 == 1 else 0
	l.freckles = 1 if player % 4 == 1 else 0
	l.cheek_color = 1
	# (Brows at their own size, the owner, 2026-10-19: enlarged they were far too thick.)
	if player % 4 == 2:
		l.mouth_size = FaceLook.neutral("mouth_size") + 3
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
	# (And one from before the belt slot wore the belt.)
	if b.size() <= SLOTS.find("belt") + 1:
		l.belt = 0
	# (And one from before the freckles slot had none of its own.)
	if b.size() <= SLOTS.find("freckles") + 1:
		l.freckles = 0
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
