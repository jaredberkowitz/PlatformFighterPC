extends Control
## The character creator as a cutting mat (the owner's pick of 2026-11-05, the "Paper UI Mocks" canvas, round 4): a bar along the top
## with Exit and Done; under it the categories as named tabs in groups (body; the face's parts and mood; the outfit; how it plays); the
## fighter whole on a green cutting mat at the left, cut out of paper, turned to the camera (it moves in on the head for the face), its
## name on a card at the mat's foot; on the right the category's name and its choices as cards, and under them a card with the part's
## colours and its fit, each fit button named (height, spacing, size, tilt, stretch; each press says what it did beside the name,
## "+1 height"), and Reset. Done saves the fighter and goes to character select.
##
## Looks are cosmetic (loadout.gd, face_look.gd) and never reach the simulation; the stats become fighter parameters in Rust only
## (sim-content recipe.rs). Keys: arrows or WASD move between the categories, the choices and the tools (left and right change a stat
## or a selector), J, Space or Enter picks, Q and E change category, Z and X turn the fighter, C faces it front again, Esc exits.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const Art := preload("res://ui/menu_art.gd")
const Preview := preload("res://ui/preview.gd")
const Loadout := preload("res://scripts/loadout.gd")
const FaceLook := preload("res://scripts/face_look.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")
const FACE_UI := preload("res://shaders/face_ui.gdshader")

## The categories down the left. `slot` is the look's slot the grid chooses; `color` the slot its colour swatches set (with `palette`);
## `adjust` the look's adjustments (FaceLook.ADJUST) the tool buttons move; `face` moves the camera in on the head; `view` frames the
## grid's face pictures (the part's slot for the height, and the magnification); `shot` frames the 3D pictures ("head" or "body").
const CATEGORIES := [
	{"id": "body", "title": "Body", "slot": "color", "kind": "swatch"},
	{"id": "eyes", "title": "Eyes", "slot": "eyes", "color": "eye_color", "adjust": ["eye_y", "eye_gap", "eye_size", "eye_turn", "eye_squash"],
		"face": true, "view": ["eye", 3.4]},
	{"id": "brows", "title": "Brows", "slot": "brows", "color": "brow_color", "adjust": ["brow_y", "brow_gap", "brow_size", "brow_turn"],
		"face": true, "view": ["brow", 1.7]},
	{"id": "nose", "title": "Nose", "slot": "nose", "adjust": ["nose_y", "nose_size"], "face": true, "view": ["nose", 2.6]},
	{"id": "mouth", "title": "Mouth", "slot": "mouth", "adjust": ["mouth_y", "mouth_size", "mouth_squash"], "face": true, "view": ["mouth", 2.4]},
	{"id": "cheeks", "title": "Cheeks", "slot": "cheeks", "color": "cheek_color", "face": true, "view": ["cheek", 1.3]},
	{"id": "freckles", "title": "Freckles", "slot": "freckles", "face": true, "view": ["freckle", 3.2]},
	{"id": "mood", "title": "Mood", "slot": "face", "face": true, "view": ["face", 1.05]},
	{"id": "hat", "title": "Hat", "slot": "hat", "shot": "head"},
	{"id": "glasses", "title": "Glasses", "slot": "glasses", "shot": "head"},
	{"id": "neck", "title": "Neckwear", "slot": "neck", "color": "accent", "shot": "body"},
	{"id": "shirt", "title": "Shirt", "slot": "shirt", "color": "accent", "shot": "body"},
	{"id": "gear", "title": "Gear", "slot": "gear", "color": "accent", "shot": "body"},
	{"id": "play", "title": "How it plays"},
]
## What each adjustment's buttons do: [the button for one step down, the one for a step up, the word for the change, which way is
## "+"] ("up" is a step down in the face's numbers, which grow downward).
const TOOLS := {
	"eye_y": ["down", "up", "height", -1], "eye_gap": ["closer", "apart", "spacing", 1], "eye_size": ["smaller", "bigger", "size", 1],
	"eye_turn": ["turn_in", "turn_out", "tilt", 1], "eye_squash": ["squash", "stretch", "stretch", 1],
	"brow_y": ["down", "up", "height", -1], "brow_gap": ["closer", "apart", "spacing", 1], "brow_size": ["smaller", "bigger", "size", 1],
	"brow_turn": ["turn_in", "turn_out", "tilt", 1],
	"nose_y": ["down", "up", "height", -1], "nose_size": ["smaller", "bigger", "size", 1],
	"mouth_y": ["down", "up", "height", -1], "mouth_size": ["smaller", "bigger", "size", 1], "mouth_squash": ["squash", "stretch", "width", 1],
}
const BAR_KEYS := ["Size", "Run speed", "Jump height", "Weight", "Fall speed"]
## The face shader's drawings for each part (the others are left out of a part's picture).
const PART_TEXTURES := {"eye": ["eye_fill_tex", "eye_line_tex", "eye_extra_tex"], "brow": ["brow_tex"], "nose": ["nose_tex"],
	"mouth": ["mouth_tex"], "cheek": ["cheek_tex", "freckle_tex"], "freckle": ["freckle_tex", "cheek_tex"]}
const ALL_PART_TEXTURES := ["eye_fill_tex", "eye_line_tex", "eye_extra_tex", "brow_tex", "nose_tex", "mouth_tex", "cheek_tex", "freckle_tex"]

# Layout (a 1280x720 screen): the tabs' row, the mat, the name card on its foot, the choices and the fit card under them.
const TABS_Y := 82.0
const STAGE := Rect2(24, 132, 540, 516)
const PLATE := Rect2(40, 596, 340, 66)
const GRID := Rect2(592, 184, 664, 306)
const TOOLS_RECT := Rect2(590, 498, 668, 156)
const CELL := Vector2(102, 94)
const GRID_COLUMNS := 6
## The categories' groups along the tab row: [caption, first category, last category].
const GROUPS := [["BODY", 0, 0], ["FACE", 1, 7], ["OUTFIT", 8, 12], ["PLAY", 13, 13]]
## The cutting mat's green.
const MAT_GREEN := Color("2f8a6a")

var look: RefCounted
var class_index := 0
var casual := false
var editing_slug := ""
var saved_list: Array = []
var category := 0
var focused: Control
var feedback_text := ""
var feedback_time := 0.0

var preview: Control
var stage_mask: Control
var stage_scene: Control
## The look and stats as they were when the screen opened (or last saved), to warn before leaving with changes.
var opened_with := {}
var exit_armed := false
var name_edit: LineEdit
var status: Control
var header: Control
var tabs: Array = []
var page: Control
var tools: Control
var page_items: Array = []
var tool_items: Array = []
var thumbs: Array = []        # [{mat, slot, value, expr}] face pictures to refresh when the look changes
var frozen: Array = []        # 3D pictures still rendering their first frames
var frozen_frames := 0
var exit_button: Control
var done_button: Control
# The "how it plays" page.
var fighter_select: Control
var rules: Control
var stat_rows := {}
var class_cards: Array = []
## The chosen class's description, under the class grid.
var class_blurb: Label
## The class grid: columns, and each tile's size with its gap.
const CLASS_COLUMNS := 7
const CLASS_TILE := Vector2(94, 46)
var bars: Control
var points_tag: Control
var delete_button: Control
var editor: RefCounted


# ---- Pieces --------------------------------------------------------------------------------------------------------------------

class Pick extends Control:
	## Something to choose or press: a painted card, gold when it is the current choice, ringed when it has the keyboard's focus.
	## Emits `activated` when clicked (or picked with the keys).
	signal activated
	signal hovered
	var chosen := false:
		set(v):
			chosen = v
			queue_redraw()
	var focused := false:
		set(v):
			focused = v
			queue_redraw()
	var fill := Art.CREAM
	var caption := ""
	var seed := 1
	var cut := 12.0

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_STOP
		mouse_entered.connect(func(): hovered.emit())

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			activated.emit()
			accept_event()

	func _card_rect() -> Rect2:
		return Rect2(Vector2(4, 4), size - Vector2(12, 14))

	func _draw() -> void:
		var r := _card_rect()
		# The keyboard's focus: the kit's focus frame round the card's own cut shape (a square coral box before; docs/UI_DESIGN_SYSTEM.md).
		if focused:
			Art.focus_frame(self, r, 1.0, cut)
		Art.card(self, r, Art.SUN if chosen else fill, seed, cut, 3.0 if not chosen else 4.0, Color(0, 0, 0, 0), 0.0, Color(0, 0, 0, 0), 0.0,
			not focused)
		_paint(r)
		if caption != "":
			Art.title_centered(self, caption, Rect2(r.position.x, r.end.y - 26.0, r.size.x, 24.0), 17, Color.WHITE, 0.14)

	func _paint(_r: Rect2) -> void:
		pass


class RailTab extends Pick:
	## A category's tab along the top: its name on a small card (gold when open). (An icon on a rail down the left before the
	## cutting mat: unnamed, the fourteen read as a puzzle.)
	var icon := ""
	var label := ""

	func _paint(r: Rect2) -> void:
		if label != "":
			UI.D.draw_text_centered(self, label, r.grow_individual(0, 0, -2, -1), 18, Art.INK)
			return
		var c := r.get_center()
		var s := minf(r.size.x, r.size.y) * 0.32
		var ink := Art.INK
		var w := 3.0
		match icon:
			"body":
				draw_circle(c + Vector2(0, -s * 0.45), s * 0.5, ink)
				draw_colored_polygon(PackedVector2Array([c + Vector2(-s * 0.8, s), c + Vector2(-s * 0.55, s * 0.1), c + Vector2(s * 0.55, s * 0.1),
					c + Vector2(s * 0.8, s)]), ink)
			"eyes":
				draw_arc(c, s * 0.8, 0, TAU, 24, ink, w, true)
				draw_circle(c + Vector2(s * 0.15, 0), s * 0.42, ink)
			"brows":
				draw_arc(c + Vector2(0, s * 0.9), s * 1.1, PI * 1.25, PI * 1.75, 16, ink, w * 1.6, true)
				draw_circle(c + Vector2(0, s * 0.5), s * 0.28, ink)
			"nose":
				draw_polyline(PackedVector2Array([c + Vector2(0, -s), c + Vector2(-s * 0.4, s * 0.5), c + Vector2(s * 0.3, s * 0.6)]), ink, w, true)
			"mouth":
				draw_arc(c + Vector2(0, -s * 0.5), s, PI * 0.2, PI * 0.8, 16, ink, w * 1.3, true)
			"cheeks":
				draw_circle(c + Vector2(-s * 0.55, 0), s * 0.38, Color(Art.CORAL, 0.8))
				draw_circle(c + Vector2(s * 0.55, 0), s * 0.38, Color(Art.CORAL, 0.8))
			"freckles":
				for d in [Vector2(-0.55, -0.3), Vector2(0.0, -0.45), Vector2(0.5, -0.25), Vector2(-0.3, 0.3), Vector2(0.3, 0.35)]:
					draw_circle(c + d * s, s * 0.17, ink)
			"mood":
				draw_arc(c, s, 0, TAU, 24, ink, w, true)
				draw_circle(c + Vector2(-s * 0.35, -s * 0.2), s * 0.13, ink)
				draw_circle(c + Vector2(s * 0.35, -s * 0.2), s * 0.13, ink)
				draw_arc(c + Vector2(0, s * 0.05), s * 0.45, PI * 0.2, PI * 0.8, 12, ink, w, true)
			"hat":
				draw_colored_polygon(PackedVector2Array([c + Vector2(-s * 0.6, s * 0.3), c + Vector2(-s * 0.5, -s * 0.6), c + Vector2(s * 0.5, -s * 0.6),
					c + Vector2(s * 0.6, s * 0.3)]), ink)
				draw_line(c + Vector2(-s * 1.1, s * 0.4), c + Vector2(s * 1.1, s * 0.4), ink, w * 1.4, true)
			"glasses":
				draw_arc(c + Vector2(-s * 0.55, 0), s * 0.42, 0, TAU, 18, ink, w, true)
				draw_arc(c + Vector2(s * 0.55, 0), s * 0.42, 0, TAU, 18, ink, w, true)
				draw_line(c + Vector2(-s * 0.13, 0), c + Vector2(s * 0.13, 0), ink, w, true)
			"neck":
				draw_colored_polygon(PackedVector2Array([c, c + Vector2(-s, -s * 0.6), c + Vector2(-s, s * 0.6)]), ink)
				draw_colored_polygon(PackedVector2Array([c, c + Vector2(s, -s * 0.6), c + Vector2(s, s * 0.6)]), ink)
			"shirt":
				draw_colored_polygon(PackedVector2Array([c + Vector2(-s * 0.4, -s * 0.8), c + Vector2(-s, -s * 0.4), c + Vector2(-s * 0.75, 0),
					c + Vector2(-s * 0.5, -s * 0.1), c + Vector2(-s * 0.5, s * 0.9), c + Vector2(s * 0.5, s * 0.9), c + Vector2(s * 0.5, -s * 0.1),
					c + Vector2(s * 0.75, 0), c + Vector2(s, -s * 0.4), c + Vector2(s * 0.4, -s * 0.8)]), ink)
			"gear":
				# A pair of shorts.
				draw_colored_polygon(PackedVector2Array([c + Vector2(-s * 0.8, -s * 0.7), c + Vector2(s * 0.8, -s * 0.7), c + Vector2(s * 0.95, s * 0.8),
					c + Vector2(s * 0.15, s * 0.8), c + Vector2(0, -s * 0.05), c + Vector2(-s * 0.15, s * 0.8), c + Vector2(-s * 0.95, s * 0.8)]), ink)
			"play":
				for k in 3:
					var h := s * (0.6 + 0.4 * k)
					draw_rect(Rect2(c + Vector2(-s * 0.9 + k * s * 0.65, s - h), Vector2(s * 0.45, h)), ink)


class Choice extends Pick:
	## One of the grid's choices: a picture (its child) on a card, with a caption.
	pass


class Swatch extends Pick:
	## A colour to choose: a round blob of it on the card.
	var colour := Color.WHITE

	func _paint(r: Rect2) -> void:
		var c := r.get_center()
		var rad := minf(r.size.x, r.size.y) * 0.32
		draw_circle(c + Vector2(2, 3), rad, Color(Art.INK, 0.35))
		draw_circle(c, rad, colour)
		draw_arc(c, rad, 0, TAU, 32, Art.INK, 3.0, true)
		draw_circle(c + Vector2(-rad * 0.35, -rad * 0.35), rad * 0.22, Color(1, 1, 1, 0.45))


class ToolButton extends Pick:
	## A tool for the chosen part: its icon (two dots and arrows, like the moves it makes), or a word.
	var icon := ""
	var word := ""

	func _paint(r: Rect2) -> void:
		var c := r.get_center()
		var s := minf(r.size.x, r.size.y) * 0.3
		var ink := Art.INK
		if word != "":
			UI.D.draw_text_centered(self, word, r, 22, ink)
			return
		match icon:
			"up", "down":
				var d := -1.0 if icon == "up" else 1.0
				draw_circle(c + Vector2(-s * 0.7, s * 0.35 * -d), s * 0.28, ink)
				draw_circle(c + Vector2(s * 0.7, s * 0.35 * -d), s * 0.28, ink)
				_arrow(c + Vector2(0, s * 0.5 * -d), c + Vector2(0, s * 0.9 * d), ink)
			"closer", "apart":
				var out := icon == "apart"
				var gap := s * (0.95 if out else 0.35)
				draw_circle(c + Vector2(-gap, 0), s * 0.28, ink)
				draw_circle(c + Vector2(gap, 0), s * 0.28, ink)
				if out:
					_arrow(c + Vector2(-gap + s * 0.5, 0), c + Vector2(-gap - s * 0.2, 0), ink, 0.55)
					_arrow(c + Vector2(gap - s * 0.5, 0), c + Vector2(gap + s * 0.2, 0), ink, 0.55)
				else:
					_arrow(c + Vector2(-s * 1.2, 0), c + Vector2(-gap - s * 0.32, 0), ink, 0.55)
					_arrow(c + Vector2(s * 1.2, 0), c + Vector2(gap + s * 0.32, 0), ink, 0.55)
			"smaller", "bigger":
				var big := icon == "bigger"
				draw_circle(c, s * (0.75 if big else 0.35), ink)
				for k in 4:
					var a := PI * 0.25 + k * PI * 0.5
					var dir := Vector2(cos(a), sin(a))
					if big:
						_arrow(c + dir * s * 0.82, c + dir * s * 1.25, ink, 0.4)
					else:
						_arrow(c + dir * s * 1.25, c + dir * s * 0.5, ink, 0.4)
			"turn_in", "turn_out":
				var cw := icon == "turn_out"
				draw_circle(c, s * 0.3, ink)
				var a0 := PI * 1.1 if cw else PI * 1.9
				var a1 := PI * 1.9 if cw else PI * 1.1
				draw_arc(c, s * 0.85, minf(a0, a1), maxf(a0, a1), 16, ink, 3.0, true)
				_arrow(c + Vector2(cos(a1 - (0.2 if cw else -0.2)), sin(a1 - (0.2 if cw else -0.2))) * s * 0.85,
					c + Vector2(cos(a1), sin(a1)) * s * 0.85, ink, 0.5)
			"squash", "stretch":
				var tall := icon == "stretch"
				var e := PackedVector2Array()
				for k in 25:
					var a := TAU * k / 24.0
					e.append(c + Vector2(cos(a) * s * (0.45 if tall else 1.0), sin(a) * s * (1.0 if tall else 0.45)))
				draw_colored_polygon(e, ink)

	func _arrow(from: Vector2, to: Vector2, col: Color, head := 0.45) -> void:
		var d := (to - from).normalized()
		var h := minf(size.x, size.y) * 0.3 * head
		draw_line(from, to - d * h * 0.6, col, 3.0, true)
		draw_colored_polygon(PackedVector2Array([to, to - d * h + d.orthogonal() * h * 0.6, to - d * h - d.orthogonal() * h * 0.6]), col)


class Header extends Control:
	## Printed on the paper: the tab groups' captions, the open category's name with how many choices it has, and the last change beside
	## it ("+1 height").
	var category := ""
	var count_text := ""
	var feedback := ""
	var feedback_alpha := 0.0
	## [x, caption] for each group of tabs.
	var groups: Array = []

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var lf := UI.D.label_font()
		for g in groups:
			draw_string(lf, Vector2(float(g[0]) + 4.0, TABS_Y - 6.0), str(g[1]), HORIZONTAL_ALIGNMENT_LEFT, -1, 13, UI.D.LABEL)
		var df := UI.D.display_font()
		var at := Vector2(GRID.position.x, GRID.position.y - 14.0)
		draw_string(df, at, category.to_upper(), HORIZONTAL_ALIGNMENT_LEFT, -1, 40, Art.INK)
		var w := df.get_string_size(category.to_upper(), HORIZONTAL_ALIGNMENT_LEFT, -1, 40).x
		if count_text != "":
			draw_string(UI.D.font(), at + Vector2(w + 14.0, -4.0), count_text, HORIZONTAL_ALIGNMENT_LEFT, -1, 18, UI.D.LABEL)
		if feedback != "" and feedback_alpha > 0.0:
			var fw := df.get_string_size(feedback, HORIZONTAL_ALIGNMENT_LEFT, -1, 30).x
			draw_string(df, Vector2(GRID.end.x - fw, at.y - 2.0), feedback, HORIZONTAL_ALIGNMENT_LEFT, -1, 30, Color(Art.CORAL, feedback_alpha))


class StageFrame extends Control:
	## The cutting mat's card: green cardstock on its paper shadows.
	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		Art.card(self, Rect2(Vector2.ZERO, size - Vector2(10, 12)), MAT_GREEN, 61, 16.0)


class Mat extends Control:
	## The cutting mat's face: fine white rules every 36 px, stronger every 180, and the fighter's contact shade (off close up).
	var show_floor := true:
		set(v):
			show_floor = v
			queue_redraw()

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		draw_rect(Rect2(Vector2.ZERO, size), MAT_GREEN)
		var x := 18.0
		while x < size.x:
			draw_line(Vector2(x, 0), Vector2(x, size.y), Color(1, 1, 1, 0.22 if int(x - 18.0) % 180 == 0 else 0.11), 2.0 if int(x - 18.0) % 180 == 0 else 1.0)
			x += 36.0
		var y := 18.0
		while y < size.y:
			draw_line(Vector2(0, y), Vector2(size.x, y), Color(1, 1, 1, 0.22 if int(y - 18.0) % 180 == 0 else 0.11), 2.0 if int(y - 18.0) % 180 == 0 else 1.0)
			y += 36.0
		if show_floor:
			var c := Vector2(size.x * 0.5, size.y * 0.71)
			var pts := PackedVector2Array()
			for k in 33:
				var a := TAU * k / 32.0
				pts.append(c + Vector2(cos(a) * size.x * 0.3, sin(a) * size.y * 0.06))
			draw_colored_polygon(pts, Color(0.04, 0.16, 0.12, 0.3))


class Plate extends Control:
	## The name card on the mat's foot: cream cardstock, a small caption, the name typed on it (gold while typing).
	var focused := false

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		Art.card(self, Rect2(Vector2.ZERO, size - Vector2(6, 6)), Art.SUN if focused else UI.D.SHEET.lightened(0.3), 71, 10.0)
		draw_string(UI.D.label_font(), Vector2(16, 18), "NAME  ·  N TO TYPE", HORIZONTAL_ALIGNMENT_LEFT, -1, 12, UI.D.LABEL)


## The fit card under the choices: a sheet of cream card behind the colours and the fit buttons.
class FitCard extends Control:
	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		Art.card(self, Rect2(Vector2.ZERO, size - Vector2(6, 6)), UI.D.SHEET, 83, 14.0)


# ---- Building the screen ------------------------------------------------------------------------------------------------------

func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	editor = ClassDB.instantiate("ContentEditor")
	editor.new_from_builtin("preview")
	add_child(UI.Backdrop.new(false, 0.38))
	add_child(UI.TopBar.new("CREATE"))
	header = Header.new()
	header.size = Vector2(1280, 720)
	add_child(header)

	# The stage: a painted scene in a card, the fighter standing on its ledge.
	var frame := StageFrame.new()
	frame.position = STAGE.position
	frame.size = STAGE.size + Vector2(10, 12)
	add_child(frame)
	stage_mask = UI.Mask.new()
	stage_mask.position = STAGE.position + Vector2(10, 10)
	stage_mask.size = STAGE.size - Vector2(20, 20)
	var m := stage_mask.size
	stage_mask.set_shape(PackedVector2Array([Vector2(16, 0), Vector2(m.x, 0), Vector2(m.x, m.y - 16), Vector2(m.x - 16, m.y), Vector2(0, m.y),
		Vector2(0, 16)]))
	add_child(stage_mask)
	var scene := Mat.new()
	stage_scene = scene
	scene.size = stage_mask.size
	stage_mask.add_child(scene)
	preview = Preview.new(Vector2i(int(m.x), int(m.y)), false, 8.2)
	preview.spin = false
	preview.facing_bias = 0.0
	preview.framed = true
	# The whole stage is the picture; the camera is aimed so the feet are on the scene's ledge (0.7 of the way down: the point it looks
	# at is 0.4 of the half-height above them, at 30 degrees and this distance).
	preview.body_from = Vector3(0, 1.0, 7.4)
	preview.body_at = Vector3(0, 0.79, 0)
	preview.size = m
	preview.position = Vector2.ZERO
	# (Cut out of paper, as on every menu.)
	UI.die_cut(preview)
	stage_mask.add_child(preview)
	var hint := UI.KeyBar.new([["Z X", "Turn"], ["C", "Front"]])
	hint.band = true
	hint.position = Vector2(STAGE.end.x - 214.0, STAGE.position.y + 16.0)
	hint.size = Vector2(190, 28)
	add_child(hint)

	# The name plate.
	var plate := Plate.new()
	plate.position = PLATE.position
	plate.size = PLATE.size
	add_child(plate)
	name_edit = LineEdit.new()
	name_edit.position = PLATE.position + Vector2(10, 20)
	name_edit.size = Vector2(PLATE.size.x - 30, PLATE.size.y - 26)
	name_edit.max_length = 24
	name_edit.placeholder_text = "Type a name"
	name_edit.alignment = HORIZONTAL_ALIGNMENT_LEFT
	name_edit.add_theme_font_override("font", UI.D.display_font())
	name_edit.add_theme_font_size_override("font_size", 34)
	name_edit.add_theme_color_override("font_color", Art.INK)
	name_edit.add_theme_color_override("font_placeholder_color", Color(UI.D.LABEL, 0.8))
	name_edit.add_theme_color_override("caret_color", Art.INK)
	var clear := StyleBoxEmpty.new()
	name_edit.add_theme_stylebox_override("normal", clear)
	name_edit.add_theme_stylebox_override("focus", clear)
	name_edit.text_submitted.connect(func(_t): name_edit.release_focus())
	name_edit.focus_entered.connect(func():
		plate.focused = true
		plate.queue_redraw())
	name_edit.focus_exited.connect(func():
		plate.focused = false
		plate.queue_redraw())
	add_child(name_edit)

	status = UI.notice(STAGE.position + Vector2(14, 14), STAGE.size.x - 38, 18)
	status.visible = false
	add_child(status)

	# The categories' tabs along the top, in their groups, each named.
	var x := 24.0
	for i in CATEGORIES.size():
		for g in GROUPS:
			if int(g[1]) == i:
				if i > 0:
					x += 18.0
				header.groups.append([x, g[0]])
		var tab := RailTab.new()
		tab.icon = CATEGORIES[i].id
		tab.label = CATEGORIES[i].title
		tab.seed = 200 + i
		tab.cut = 6.0
		var w := UI.D.font().get_string_size(tab.label, HORIZONTAL_ALIGNMENT_LEFT, -1, 18).x + 34.0
		tab.position = Vector2(x, TABS_Y)
		tab.size = Vector2(w, 46)
		x += w - 6.0
		var index := i
		tab.activated.connect(func(): _open(index))
		tab.hovered.connect(func(): _focus(tab))
		add_child(tab)
		tabs.append(tab)

	page = Control.new()
	page.position = GRID.position
	page.size = GRID.size
	page.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(page)
	tools = Control.new()
	tools.position = TOOLS_RECT.position
	tools.size = TOOLS_RECT.size
	tools.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(tools)

	var keys := UI.KeyBar.new([["Q E", "Category"], ["WASD", "Choose"], ["Enter", "Pick"], ["N", "Name"], ["Z X", "Turn"], ["Esc", "Exit"]])
	keys.position = Vector2(24, 674)
	keys.size = Vector2(1232, 30)
	add_child(keys)

	exit_button = _tool_word("Exit", Vector2(1040, 4), Vector2(100, 50), UI.D.SHEET.lightened(0.3))
	exit_button.activated.connect(_back)
	add_child(exit_button)
	done_button = _tool_word("Done", Vector2(1146, 4), Vector2(112, 50), Art.SUN)
	done_button.activated.connect(_finish)
	add_child(done_button)

	saved_list = Roster.saved()
	var start := 0
	if Roster.edit_slug != "":
		for i in saved_list.size():
			if saved_list[i].slug == Roster.edit_slug:
				start = i + 1
		Roster.edit_slug = ""
	_load_fighter(start)
	_open(1)
	PadNav.attach(self)
	UI.reveal(self)
	Music.of(self).play("menu")
	Shot.attach(self)


func _tool_word(text: String, at: Vector2, sz: Vector2, colour: Color) -> ToolButton:
	var b := ToolButton.new()
	b.word = text
	b.fill = colour
	b.position = at
	b.size = sz
	b.seed = text.length() * 7
	b.hovered.connect(func(): _focus(b))
	return b


## Starts from a saved fighter (`i` > 0, its place in the saved list plus one) or a new one (0).
func _load_fighter(i: int) -> void:
	if i == 0:
		editing_slug = ""
		look = Loadout.default_for(0)
		class_index = 0
		casual = false
		name_edit.text = ""
		_stats = {"size": 5, "speed": 5, "jump": 5, "weight": 5}
	else:
		var e: Dictionary = saved_list[i - 1]
		editing_slug = e.slug
		look = Loadout.from_bytes(e.look.to_bytes())
		class_index = int(e["class"])
		casual = bool(e.get("casual", false))
		name_edit.text = e.name
		_stats = {"size": e.size, "speed": e.speed, "jump": e.jump, "weight": e.weight}
	_fighter_index = i
	_changed()
	opened_with = _snapshot()
	exit_armed = false


func _snapshot() -> Dictionary:
	return {"look": look.to_code(), "class": class_index, "stats": _stats.duplicate(), "casual": casual, "name": name_edit.text}


var _stats := {"size": 5, "speed": 5, "jump": 5, "weight": 5}
var _fighter_index := 0


# ---- Pages --------------------------------------------------------------------------------------------------------------------

## Opens a category: its choices in the grid, its tools on the right.
func _open(i: int) -> void:
	category = clampi(i, 0, CATEGORIES.size() - 1)
	var cat: Dictionary = CATEGORIES[category]
	for k in tabs.size():
		tabs[k].chosen = k == category
	header.category = cat.title
	header.count_text = ""
	if cat.has("slot") and cat.id != "body":
		header.count_text = "%d to choose from" % Loadout.slot_size(cat.slot)
	header.queue_redraw()
	var close: bool = bool(cat.get("face", false)) or str(cat.get("shot", "")) == "head"
	preview.frame_face(close)
	# Close up, the stage's ground would be a slab across the bottom: just the sky then.
	stage_scene.show_floor = not close
	_build_page()
	_build_tools()
	if focused == null or not is_instance_valid(focused) or not (focused is RailTab):
		_focus(page_items[0] if not page_items.is_empty() else tabs[category])
	else:
		_focus(tabs[category])


func _clear(node: Control) -> void:
	for c in node.get_children():
		node.remove_child(c)
		c.queue_free()


func _build_page() -> void:
	_clear(page)
	page_items.clear()
	thumbs.clear()
	frozen.clear()
	stat_rows.clear()
	class_cards.clear()
	class_blurb = null
	var cat: Dictionary = CATEGORIES[category]
	if cat.id == "play":
		_build_play_page()
		return
	var slot: String = cat.slot
	var count := Loadout.slot_size(slot)
	var names: Array = Loadout.slot_names(slot)
	for v in count:
		var item: Pick = Swatch.new() if cat.get("kind", "") == "swatch" else Choice.new()
		item.seed = 300 + v * 11
		item.position = Vector2((v % GRID_COLUMNS) * (CELL.x + 10), (v / GRID_COLUMNS) * (CELL.y + 8))
		item.size = CELL
		var value := v
		item.activated.connect(func(): _choose(value))
		item.hovered.connect(func(): _focus(item))
		page.add_child(item)
		page_items.append(item)
		if item is Swatch:
			item.colour = Loadout.PALETTE[v]
			item.caption = str(names[v]).capitalize()
		elif cat.has("view"):
			item.add_child(_face_picture(slot, v, cat))
			var n := str(names[v])
			item.caption = "None" if n == "none" else ""
		elif cat.has("shot"):
			item.add_child(_shot_picture(slot, v, cat.shot))
			item.caption = str(names[v]).capitalize()
	_mark_choice()


## A picture of the face with `slot` set to `value`, framed on the part.
func _face_picture(slot: String, value: int, cat: Dictionary) -> Control:
	var pic := ColorRect.new()
	pic.mouse_filter = Control.MOUSE_FILTER_IGNORE
	pic.position = Vector2(14, 12)
	pic.size = CELL - Vector2(36, 34)
	var mat := ShaderMaterial.new()
	mat.shader = FACE_UI
	pic.material = mat
	var expr := "deadpan"
	if slot == "face":
		expr = Loadout.FACES[value].name
	thumbs.append({"mat": mat, "slot": slot, "value": value, "expr": expr, "view": cat.view})
	_paint_thumb(thumbs[-1])
	return pic


func _paint_thumb(t: Dictionary) -> void:
	var variant: RefCounted = Loadout.from_bytes(look.to_bytes())
	variant.set_slot(t.slot, t.value)
	var mat: ShaderMaterial = t.mat
	FaceLook.apply(mat, variant, t.expr)
	mat.set_shader_parameter("skin", look.body_color())
	var y := 0.48
	match str(t.view[0]):
		"eye":
			y = FaceLook.EYE_Y + FaceLook.offset(variant, "eye_y") * 0.011
		"brow":
			y = FaceLook.BROW_Y + FaceLook.offset(variant, "brow_y") * 0.011
		"nose":
			y = FaceLook.NOSE_Y + FaceLook.offset(variant, "nose_y") * 0.01
		"mouth":
			y = FaceLook.MOUTH_Y + FaceLook.offset(variant, "mouth_y") * 0.011
		"cheek", "freckle":
			y = FaceLook.EYE_Y + 0.1
	mat.set_shader_parameter("view", Vector3(0.5, y, float(t.view[1])))
	if str(t.view[0]) == "eye":
		# (Close on one eye, the face's right: framed on the pair, the eight styles looked alike at the tile's size. Art audit
		# 2026-10-29.)
		mat.set_shader_parameter("view", Vector3(0.5 + FaceLook.EYE_GAP + FaceLook.offset(variant, "eye_gap") * 0.012, y, float(t.view[1])))
	if str(t.view[0]) == "freckle":
		# (Close on one cheek: the whole face showed the dots as specks.)
		var cp: Vector3 = mat.get_shader_parameter("cheek_place")
		mat.set_shader_parameter("view", Vector3(0.5 + cp.x, cp.y, float(t.view[1])))
	# A part's picture shows that part alone on the skin (the mood's, the whole face).
	var part := str(t.view[0])
	if PART_TEXTURES.has(part):
		for name in ALL_PART_TEXTURES:
			if not (PART_TEXTURES[part] as Array).has(name):
				mat.set_shader_parameter(name, null)


## A small 3D picture of the fighter wearing `slot` = `value` (it renders a few frames, then stays still).
func _shot_picture(slot: String, value: int, framing: String) -> Control:
	var variant: RefCounted = Loadout.from_bytes(look.to_bytes())
	variant.set_slot(slot, value)
	var pv := Preview.new(Vector2i(int(CELL.x - 20), int(CELL.y - 40)), false, 4.0)
	pv.spin = false
	pv.facing_bias = 0.35
	pv.position = Vector2(6, 6)
	pv.size = CELL - Vector2(20, 40)
	pv.set_loadout(variant)
	pv.set_size_percent(100.0)
	if framing == "head":
		pv.camera.look_at_from_position(Vector3(0, 1.98, 2.7), Vector3(0, 1.86, 0))
	else:
		pv.camera.look_at_from_position(Vector3(0, 1.4, 3.3), Vector3(0, 1.25, 0))
	frozen.append(pv)
	frozen_frames = 0
	return pv


func _build_play_page() -> void:
	# The fighter to edit (saved ones, or a new one), the class, the four stats and the rules.
	var names := ["New fighter"]
	for s in saved_list:
		names.append(s.name)
	fighter_select = UI.Selector.new("Fighter", names)
	fighter_select.set_index(_fighter_index, false)
	fighter_select.changed.connect(func(i):
		_load_fighter(i)
		_open(category))
	fighter_select.position = Vector2(0, 0)
	page.add_child(fighter_select)
	page_items.append(fighter_select)
	# The class (the fighter's weapon and moveset): a grid of named tiles that fits the page (one row of fourteen cards ran off it and
	# over the tools: art audit 2026-10-29), and the chosen class's description under it.
	for c in Roster.CLASS_NAMES.size():
		var card := Choice.new()
		card.seed = 400 + c
		card.caption = Roster.CLASS_NAMES[c]
		card.cut = 8.0
		card.position = Vector2((c % CLASS_COLUMNS) * CLASS_TILE.x, 52 + (c / CLASS_COLUMNS) * CLASS_TILE.y)
		card.size = CLASS_TILE - Vector2(2, 2)
		var ci := c
		card.activated.connect(func():
			class_index = ci
			_mark_choice()
			_changed())
		card.hovered.connect(func(): _focus(card))
		page.add_child(card)
		page_items.append(card)
		class_cards.append(card)
	var rows_used := ceili(float(Roster.CLASS_NAMES.size()) / CLASS_COLUMNS)
	class_blurb = Label.new()
	class_blurb.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	class_blurb.position = Vector2(6, 56 + rows_used * CLASS_TILE.y)
	class_blurb.size = Vector2(GRID.size.x - 10, 44)
	class_blurb.add_theme_font_override("font", UI.font())
	class_blurb.add_theme_font_size_override("font_size", 17)
	class_blurb.add_theme_color_override("font_color", Art.INK)
	class_blurb.mouse_filter = Control.MOUSE_FILTER_IGNORE
	page.add_child(class_blurb)
	var y := 106.0 + rows_used * CLASS_TILE.y
	for stat in ["size", "speed", "jump", "weight"]:
		var r := UI.StatRow.new(stat.capitalize())
		r.set_value(int(_stats[stat]), false)
		r.can_raise = _can_raise
		var id: String = stat
		r.changed.connect(func(v):
			_stats[id] = v
			_changed())
		r.blocked.connect(func(): _say("No points left: lower another stat or choose Casual."))
		r.focused.connect(func(): _focus(r))
		r.position = Vector2(0, y)
		page.add_child(r)
		page_items.append(r)
		stat_rows[stat] = r
		y += 42.0
	rules = UI.Selector.new("Rules", ["Ranked", "Casual"])
	rules.set_index(1 if casual else 0, false)
	rules.changed.connect(func(i):
		casual = i == 1
		_changed())
	rules.focused.connect(func(): _focus(rules))
	rules.position = Vector2(0, y + 6)
	page.add_child(rules)
	page_items.append(rules)
	if editing_slug != "":
		delete_button = _tool_word("Delete fighter", Vector2(GRID.size.x - 210, 0), Vector2(200, 44), Art.CORAL.lerp(Art.CREAM, 0.4))
		delete_button.activated.connect(_delete)
		page.add_child(delete_button)
		page_items.append(delete_button)
	_mark_choice()


func _build_tools() -> void:
	_clear(tools)
	tool_items.clear()
	bars = null
	points_tag = null
	var cat: Dictionary = CATEGORIES[category]
	if cat.id == "play":
		# How it plays: the bars and the points on a card beside the stats.
		tools.position = Vector2(1040, GRID.position.y + 196)
		var sheet := FitCard.new()
		sheet.size = Vector2(222, 278)
		tools.add_child(sheet)
		var how := UI.Tag.new("How it plays", Vector2(180, 30))
		how.printed = "label"
		how.position = Vector2(4, 6)
		tools.add_child(how)
		bars = UI.Bars.new(Vector2(204, 180))
		bars.position = Vector2(8, 38)
		tools.add_child(bars)
		points_tag = UI.Tag.new("", Vector2(206, 44))
		points_tag.font_size = 22
		points_tag.position = Vector2(4, 222)
		tools.add_child(points_tag)
		_changed()
		return
	tools.position = TOOLS_RECT.position
	var adjust: Array = cat.get("adjust", [])
	if not cat.has("color") and adjust.is_empty():
		return
	var sheet := FitCard.new()
	sheet.size = TOOLS_RECT.size
	tools.add_child(sheet)
	var y := 36.0
	if cat.has("color"):
		var slot: String = cat.color
		var colours: Array = _palette(slot)
		var label := UI.Tag.new("Outfit colour" if slot == "accent" else "Colour", Vector2(200, 28))
		label.printed = "label"
		label.position = Vector2(8, 6)
		tools.add_child(label)
		for v in colours.size():
			var sw := Swatch.new()
			sw.colour = colours[v]
			sw.seed = 500 + v
			sw.cut = 6.0
			sw.position = Vector2(14 + (v % 6) * 46, y + (v / 6) * 52)
			sw.size = Vector2(48, 50)
			var value := v
			sw.activated.connect(func(): _set_slot(slot, value, str(Loadout.slot_names(slot)[value]).capitalize()))
			sw.hovered.connect(func(): _focus(sw))
			sw.set_meta("slot", slot)
			sw.set_meta("value", v)
			tools.add_child(sw)
			tool_items.append(sw)
	# The fit: a pair of buttons for each adjustment, named, in two columns of three (right of the colours when there are colours).
	var fit_x := 300.0 if cat.has("color") else 8.0
	if not adjust.is_empty():
		var fit_label := UI.Tag.new("Fit", Vector2(120, 28))
		fit_label.printed = "label"
		fit_label.position = Vector2(fit_x, 6)
		tools.add_child(fit_label)
	for i in adjust.size():
		var a: String = adjust[i]
		var t: Array = TOOLS[a]
		var at := Vector2(fit_x + 6.0 + (i / 3) * 180.0, 34.0 + (i % 3) * 38.0)
		var word := UI.Tag.new(str(t[2]).capitalize(), Vector2(84, 34))
		word.printed = "label"
		word.font_size = 15
		word.position = at + Vector2(98, 0)
		tools.add_child(word)
		for k in 2:
			var b := ToolButton.new()
			b.icon = t[k]
			b.seed = 600 + tool_items.size()
			b.position = at + Vector2(k * 48, 0)
			b.size = Vector2(52, 40)
			b.tooltip_text = "%s %s" % [t[2].capitalize(), "-" if k == 0 else "+"]
			var slot: String = a
			var step := -1 if k == 0 else 1
			b.activated.connect(func(): _nudge(slot, step))
			b.hovered.connect(func(): _focus(b))
			tools.add_child(b)
			tool_items.append(b)
	if not adjust.is_empty() or cat.has("color"):
		var reset := _tool_word("Reset", Vector2(TOOLS_RECT.size.x - 112, 6), Vector2(100, 36), UI.D.SHEET.lightened(0.3))
		reset.activated.connect(_reset_part)
		tools.add_child(reset)
		tool_items.append(reset)
	_mark_choice()


func _palette(slot: String) -> Array:
	match slot:
		"eye_color":
			return FaceLook.EYE_COLORS
		"brow_color":
			return FaceLook.BROW_COLORS
		"cheek_color":
			return FaceLook.CHEEK_COLORS
	return Loadout.PALETTE


## Marks the current choice in the grid, the class cards and the swatches.
func _mark_choice() -> void:
	var cat: Dictionary = CATEGORIES[category]
	if cat.id == "play":
		for c in class_cards.size():
			class_cards[c].chosen = c == class_index
		if class_blurb != null and class_index >= 0 and class_index < Roster.CLASS_BLURBS.size():
			class_blurb.text = Roster.CLASS_BLURBS[class_index]
	else:
		var v: int = look.get_slot(cat.slot)
		for k in page_items.size():
			page_items[k].chosen = k == v
	for t in tool_items:
		if t is Swatch and t.has_meta("slot"):
			t.chosen = look.get_slot(t.get_meta("slot")) == t.get_meta("value")


# ---- Changing the fighter -----------------------------------------------------------------------------------------------------

func _choose(value: int) -> void:
	var cat: Dictionary = CATEGORIES[category]
	if cat.id == "play":
		return
	_set_slot(cat.slot, value, str(Loadout.slot_names(cat.slot)[value]).capitalize())


func _set_slot(slot: String, value: int, said: String) -> void:
	look.set_slot(slot, value)
	_tell(said)
	_mark_choice()
	_changed()
	# The 3D pictures show the outfit colour: redo them when it changes.
	if slot == "accent" or slot == "color":
		if CATEGORIES[category].has("shot"):
			var keep := page_items.find(focused)
			_build_page()
			if keep >= 0 and keep < page_items.size():
				_focus(page_items[keep])


## One step of an adjustment ("+1 height").
func _nudge(slot: String, step: int) -> void:
	var before: int = look.get_slot(slot)
	look.set_slot(slot, before + step)
	var t: Array = TOOLS[slot]
	if look.get_slot(slot) == before:
		_tell("That's the limit")
		return
	var shown := FaceLook.offset(look, slot) * int(t[3])
	_tell("%s%d %s" % ["+" if shown > 0 else "", shown, t[2]])
	_changed()


## Puts the open category's part back in its own place (the adjustments in the middle).
func _reset_part() -> void:
	for a in CATEGORIES[category].get("adjust", []):
		look.set_slot(a, FaceLook.neutral(a))
	_tell("Reset")
	_changed()


func _tell(text: String) -> void:
	exit_armed = false
	header.feedback = text
	feedback_time = 1.6


func _say(text: String) -> void:
	status.set_text(text)
	status.visible = text != ""


func current_entry() -> Dictionary:
	var e := Roster.neutral_entry(name_edit.text.strip_edges(), class_index, Loadout.from_bytes(look.to_bytes()))
	for stat in _stats:
		e[stat] = int(_stats[stat])
	e.casual = casual
	return e


## May a stat go up? Only while there are points left, unless the rules are casual.
func _can_raise() -> bool:
	return casual or Roster.points(current_entry()) < Roster.budget()


func _changed() -> void:
	var e := current_entry()
	preview.set_loadout(e.look)
	preview.set_weapon(int(e["class"]))
	if CATEGORIES[category].id == "mood":
		preview.set_expression(Loadout.FACES[look.face])
	var r: Dictionary = editor.recipe_readout(e["class"], e.size, e.speed, e.jump, e.weight)
	preview.set_size_percent(float(r.size_percent))
	for t in thumbs:
		_paint_thumb(t)
	if bars != null:
		bars.set_values(Roster.readout_bars(editor, e), BAR_KEYS)
	if points_tag != null:
		var spent := Roster.points(e)
		var over: bool = spent > Roster.budget()
		points_tag.set_text("Points %d / %d%s" % [spent, Roster.budget(), "  CASUAL" if e.casual else ""])
		points_tag.fill = UI.RED.lerp(UI.CREAM, 0.35) if (over and not e.casual) else (UI.SKY if e.casual else UI.GOLD)
	_say("")


func _process(delta: float) -> void:
	if feedback_time > 0.0:
		feedback_time -= delta
		header.feedback_alpha = clampf(feedback_time / 0.4, 0.0, 1.0)
		header.queue_redraw()
	# The 3D pictures render a few frames, then are kept as stills (they cost nothing after).
	if not frozen.is_empty():
		frozen_frames += 1
		if frozen_frames > 4:
			for pv in frozen:
				if is_instance_valid(pv):
					pv.viewport.render_target_update_mode = SubViewport.UPDATE_DISABLED
					pv.set_process(false)
			frozen.clear()


# ---- Focus and keys ------------------------------------------------------------------------------------------------------------

func _all_focusable() -> Array:
	var out: Array = tabs.duplicate()
	out.append_array(page_items)
	out.append_array(tool_items)
	out.append(exit_button)
	out.append(done_button)
	return out.filter(func(c): return is_instance_valid(c) and c.is_visible_in_tree())


func _focus(c: Control) -> void:
	if c == null:
		return
	if focused != null and is_instance_valid(focused):
		_mark_focus(focused, false)
	focused = c
	_mark_focus(c, true)


func _mark_focus(c: Control, on: bool) -> void:
	if c is Pick:
		c.focused = on
	elif c.has_method("set_focus_mark"):
		c.set_focus_mark(on)


## Moves the focus to the nearest thing in direction `dir`. Stepping onto the rail lands on the open category's tab.
func _move(dir: Vector2) -> void:
	if focused == null or not is_instance_valid(focused):
		_focus(tabs[category])
		return
	var from := focused.get_global_rect().get_center()
	var best: Control = null
	var best_score := INF
	for c in _all_focusable():
		if c == focused:
			continue
		var d: Vector2 = c.get_global_rect().get_center() - from
		var along := d.dot(dir)
		if along <= 4.0:
			continue
		var score := along + absf(d.cross(dir)) * 2.5
		if score < best_score:
			best_score = score
			best = c
	if best == null:
		return
	if best is RailTab and not (focused is RailTab):
		best = tabs[category]
	_focus(best)
	if best is RailTab:
		_open(tabs.find(best))


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed:
		return
	if name_edit.has_focus():
		# Typing the name: Esc (or Enter, handled by the field) just stops typing.
		if event.keycode == KEY_ESCAPE:
			name_edit.release_focus()
		return
	var slider := focused != null and is_instance_valid(focused) and (focused is UI.StatRow or focused is UI.Selector)
	match event.keycode:
		KEY_UP, KEY_W:
			_move(Vector2.UP)
		KEY_DOWN, KEY_S:
			_move(Vector2.DOWN)
		KEY_LEFT, KEY_A:
			if slider:
				focused.step(-1)
			else:
				_move(Vector2.LEFT)
		KEY_RIGHT, KEY_D:
			if slider:
				focused.step(1)
			else:
				_move(Vector2.RIGHT)
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE, KEY_J:
			if event.echo:
				return
			if focused is Pick:
				focused.activated.emit()
		KEY_Q:
			if not event.echo:
				_open((category - 1 + CATEGORIES.size()) % CATEGORIES.size())
		KEY_E:
			if not event.echo:
				_open((category + 1) % CATEGORIES.size())
		KEY_Z:
			preview.facing_bias = clampf(preview.facing_bias - 0.15, -1.6, 1.6)
		KEY_X:
			preview.facing_bias = clampf(preview.facing_bias + 0.15, -1.6, 1.6)
		KEY_C:
			preview.facing_bias = 0.0
		KEY_N:
			if not event.echo:
				name_edit.grab_focus()
		KEY_ESCAPE:
			if not event.echo:
				_back()


## Leaves for the main menu; with unsaved changes, only on a second press.
func _back() -> void:
	if _snapshot() != opened_with and not exit_armed:
		exit_armed = true
		_say("Unsaved changes: press Exit again to leave without saving.")
		return
	get_tree().change_scene_to_file("res://menu.tscn")


func _delete() -> void:
	if editing_slug == "":
		return
	Roster.delete(editing_slug)
	get_tree().reload_current_scene()


func _finish() -> void:
	var e := current_entry()
	var problem := Roster.name_problem(e.name, editing_slug)
	if problem == "" and not e.casual and not Roster.ranked_legal(e):
		problem = "Over the budget (%d of %d): lower a stat or choose Casual." % [Roster.points(e), Roster.budget()]
	if problem != "":
		_say(problem)
		if e.name == "" or Roster.name_problem(e.name, editing_slug) != "":
			name_edit.grab_focus()
		return
	var err := Roster.save(e, editing_slug)
	if err != "":
		_say(err)
		return
	# Renaming saves a new fighter and removes the old file.
	if editing_slug != "" and editing_slug != e.slug:
		Roster.delete(editing_slug)
	Roster.session = {"preselect": e.slug}
	get_tree().change_scene_to_file("res://select.tscn")
