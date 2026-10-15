extends Node3D
## A grunt of the run (R2b, docs/modes/RUNNER_WORLD.md section 4), drawn with the roster's own fighter rig, body, clips and cosmetic
## loadouts (`fighter_view.gd`, `loadout.gd`): each kind is a base-game body in a grunt outfit of its own colour (`OUTFITS`), so it reads
## as an enemy at a glance, with a red marker over its head. Its tell is a flash of the marker into a big "!"; it comes out of a dark
## doorway (R2a), and shrinks away when beaten. Everything it shows is the sim's (the bridge's `run_mobs` and the grunt's own fighter
## snapshot, `fighter_*(100 + n)`); nothing here decides anything. The art director's grunt outfits replace these (docs/modes/
## RUNNER_PLAN.md, the R2b art list).

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")

const MODE_WINDUP := 2
const MODE_BURST := 5
const MODE_EMERGE := 6
const EMERGE_FRAMES := 24.0
## The enemy marker's red, and the tell's warning yellow.
const ENEMY_RED := Color(0.95, 0.24, 0.24)
const TELL_YELLOW := Color(1.0, 0.86, 0.2)

## Each role's grunt outfit, from the existing catalog only (loadout slot values): a team colour, a hat, glasses, neckwear, a shirt,
## gear and a mood. Brawler, leaper, shieldbearer, spearman, lobber, bruiser.
const OUTFITS := [
	{"color": 1, "hat": 4, "glasses": 0, "neck": 2, "shirt": 3, "gear": 2, "face": 2, "accent": 6},
	{"color": 5, "hat": 6, "glasses": 2, "neck": 3, "shirt": 2, "gear": 1, "face": 3, "accent": 11},
	{"color": 9, "hat": 7, "glasses": 0, "neck": 1, "shirt": 2, "gear": 3, "face": 0, "accent": 6},
	{"color": 2, "hat": 1, "glasses": 0, "neck": 4, "shirt": 1, "gear": 3, "face": 2, "accent": 6},
	{"color": 4, "hat": 3, "glasses": 3, "neck": 2, "shirt": 4, "gear": 1, "face": 3, "accent": 10},
	{"color": 11, "hat": 8, "glasses": 1, "neck": 0, "shirt": 0, "gear": 3, "face": 2, "accent": 6},
]
## What each role is called (the marker's text while it is not telling, empty: the arrow alone).
const ROLE_NAMES := ["brawler", "leaper", "shieldbearer", "spearman", "lobber", "bruiser"]

var fighter: Node3D
var kind := -1
var role := -1
var door: MeshInstance3D
var tell_label: Label3D
var t := 0.0
## Its body's drawn size (as the match draws that fighter: its size, and broader bodies bigger).
var base_scale := 1.0


## The grunt outfit for role `r`.
static func outfit(r: int) -> RefCounted:
	var l: RefCounted = Loadout.default_for(1)
	var o: Dictionary = OUTFITS[clampi(r, 0, OUTFITS.size() - 1)]
	for slot in o:
		l.set_slot(slot, int(o[slot]))
	return l


## Builds (or rebuilds) it as grunt kind `k` of role `r`, `height` tall.
func build(k: int, r: int, height: float) -> void:
	kind = k
	role = r
	if fighter == null:
		fighter = FighterView.new()
		add_child(fighter)
		fighter.build(1, outfit(r))
		fighter.set_overhead(false, "", ENEMY_RED)
		door = MeshInstance3D.new()
		var q := QuadMesh.new()
		q.size = Vector2(1.6, 2.6)
		door.mesh = q
		var dm := StandardMaterial3D.new()
		dm.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		dm.albedo_color = Color(0.06, 0.04, 0.1)
		door.material_override = dm
		door.position = Vector3(0, 1.3, -0.9)
		add_child(door)
		tell_label = Label3D.new()
		tell_label.text = "!"
		tell_label.font_size = 160
		tell_label.outline_size = 36
		tell_label.modulate = TELL_YELLOW
		tell_label.outline_modulate = FighterView.INK
		tell_label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
		tell_label.no_depth_test = true
		tell_label.fixed_size = true
		tell_label.pixel_size = 0.0009
		tell_label.position = Vector3(0, height + 1.0, 0)
		add_child(tell_label)
	else:
		fighter.rebuild(outfit(r))
	tell_label.position.y = height + 1.0


## Draws it at `pos` from its row of `run_mobs` (`row`) and its fighter snapshot `snap`.
func apply(pos: Vector2, row: Array, snap: Dictionary, delta: float) -> void:
	t += delta
	var mode := int(row[6])
	position = Vector3.ZERO
	var grow := 1.0
	if mode == MODE_EMERGE:
		grow = clampf(1.0 - float(row[8]) / EMERGE_FRAMES, 0.0, 1.0)
	elif mode == MODE_BURST:
		grow = maxf(0.0, fighter.scale.x / maxf(base_scale, 0.01) - delta * 4.0)
	door.visible = mode == MODE_EMERGE
	door.position = Vector3(pos.x, pos.y + 1.3, -0.9)
	fighter.scale = Vector3.ONE * base_scale * (lerpf(0.35, 1.0, grow) if mode != MODE_BURST else grow)
	fighter.apply(Vector3(pos.x, pos.y, 0), snap, delta)
	# The tell: the marker becomes a big bobbing "!" (yellow, flashing red as the move nears).
	var telling := mode == MODE_WINDUP
	tell_label.visible = telling
	if telling:
		var left := float(row[16])
		tell_label.position = Vector3(pos.x, pos.y + float(row[12]) + 1.1 + sin(t * 18.0) * 0.08, 0.3)
		tell_label.modulate = TELL_YELLOW if left > 6.0 or int(t * 20.0) % 2 == 0 else ENEMY_RED
	if fighter.name_label != null:
		fighter.name_label.visible = not telling and mode != MODE_EMERGE and mode != MODE_BURST
	if fighter.tag_arrow != null:
		fighter.tag_arrow.visible = not telling and mode != MODE_EMERGE and mode != MODE_BURST
