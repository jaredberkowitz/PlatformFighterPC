extends Control
## The replay list: every finished match is saved automatically; pick one to watch it. Up/Down choose, Enter watches,
## Delete removes, Esc goes back. While watching: Space pauses, Left/Right jump 5 seconds, Up/Down change the speed, Esc returns here.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const Roster := preload("res://scripts/roster.gd")
const Replays := preload("res://scripts/replays.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")
const Art := preload("res://ui/menu_art.gd")
const D := preload("res://ui/ui_draw.gd")
## The players' colours, as in a match (main.gd), for the winner's ribbon.
const PLAYER_COLORS := [Color(0.92, 0.36, 0.36), Color(0.36, 0.52, 0.95), Color(0.95, 0.78, 0.3), Color(0.4, 0.8, 0.5)]
const MONTHS := ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]

var paths: Array = []
var infos: Array = []
var buttons: Array = []
var index := 0
var scroll: ScrollContainer
var list: VBoxContainer
var note: Control
var sim


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(UI.Backdrop.new(false))
	sim = ClassDB.instantiate("SimRunner")
	add_child(sim)
	add_child(UI.ScreenTitle.new("Replays"))

	scroll = ScrollContainer.new()
	scroll.position = Vector2(60, 100)
	scroll.size = Vector2(1160, 520)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	list = VBoxContainer.new()
	list.add_theme_constant_override("separation", 8)
	scroll.add_child(list)

	note = UI.notice(Vector2(60, 630), 1000, 22)
	add_child(note)
	var keys := UI.KeyBar.new([["W S", "Choose"], ["Enter", "Watch"], ["Del", "Remove"], ["Esc", "Back"]])
	keys.position = Vector2(60, 676)
	keys.size = Vector2(620, 30)
	add_child(keys)
	_fill()
	PadNav.attach(self)
	UI.reveal(self)
	Music.of(self).play("menu")
	Shot.attach(self)


func _fill() -> void:
	for b in buttons:
		b.queue_free()
	buttons.clear()
	infos.clear()
	paths = Replays.files()
	for p in paths:
		var info: Dictionary = sim.replay_peek(Replays.read(p))
		infos.append(info)
		var b := Row.new(Vector2(1120, 76))
		_describe(b, p, info)
		var i := buttons.size()
		b.activated.connect(func(): _watch(i))
		b.mouse_entered.connect(func(): _select(i))
		list.add_child(b)
		buttons.append(b)
	if buttons.is_empty():
		note.set_text("No replays yet. Finish a match and it is saved here.")
	else:
		note.set_text("")
	index = clampi(index, 0, maxi(0, buttons.size() - 1))
	_select(index)


## Fills a row from a replay file's `path` and its summary `info`: the match-up as the headline, when, how long and the rules under it,
## the winner on a ribbon in their player colour, and a tag if this version of the game cannot play it.
func _describe(row: Row, path: String, info: Dictionary) -> void:
	row.detail = _when(path.get_file().get_basename())
	if not info.ok:
		row.headline = "Damaged file"
		row.old = true
		return
	var player_names := []
	for i in int(info.players):
		var n: String = Roster.parse_profile(info["cosmetics%d" % i], i).name
		player_names.append(n if n != "" else "Player %d" % (i + 1))
	row.headline = " vs ".join(player_names.map(func(n): return n.left(10 if player_names.size() > 2 else 12)))
	var winner := int(info.winner)
	if winner >= 0 and winner < player_names.size():
		row.badge = "%s won" % player_names[winner].left(12)
		row.badge_colour = PLAYER_COLORS[winner % PLAYER_COLORS.size()]
	else:
		row.badge = "Draw" if winner == -2 else "Unfinished"
		row.badge_colour = Color(0.55, 0.56, 0.66)
	var secs: int = int(info.frames) / 60
	var rules := "Free play" if int(info.stocks) == 0 else "%d stock%s" % [info.stocks, "" if int(info.stocks) == 1 else "s"]
	row.detail += "   ·   %d:%02d   ·   %s" % [secs / 60, secs % 60, rules]
	row.old = not info.playable


## "2026-10-12_21-47-57" (a replay's file name) as "Oct 12, 21:47"; anything else as it is.
static func _when(stem: String) -> String:
	var parts := stem.split("_")
	if parts.size() < 2:
		return stem
	var day := parts[0].split("-")
	var clock := parts[1].split("-")
	if day.size() < 3 or clock.size() < 2:
		return stem
	var month := clampi(int(day[1]), 1, 12)
	return "%s %d, %s:%s" % [MONTHS[month - 1], int(day[2]), clock[0], clock[1]]


func _select(i: int) -> void:
	if buttons.is_empty():
		return
	index = i
	for k in buttons.size():
		buttons[k].selected = k == index
	scroll.ensure_control_visible(buttons[index])


func _watch(i: int) -> void:
	var info: Dictionary = infos[i]
	if not info.ok:
		note.set_text("That replay file is damaged: " + str(info.error))
		return
	if not info.playable:
		note.set_text("That replay was recorded by another version of the game and cannot be played.")
		return
	Roster.session = {"from_menu": true, "replay": Replays.read(paths[i])}
	get_tree().change_scene_to_file("res://main.tscn")


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	match event.keycode:
		KEY_UP, KEY_W:
			_select(maxi(0, index - 1))
		KEY_DOWN, KEY_S:
			_select(mini(buttons.size() - 1, index + 1))
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE:
			if not buttons.is_empty():
				_watch(index)
		KEY_DELETE:
			if not buttons.is_empty():
				Replays.delete(paths[index])
				_fill()
		KEY_ESCAPE:
			get_tree().change_scene_to_file("res://menu.tscn")


## One replay in the list: a painted card (UI.Btn's, lifting when chosen) with the match-up in large type, a quieter line under it, the
## result on a ribbon in the winner's colour at the right, and, for a replay this version cannot play, the card dimmed and tagged.
class Row extends UI.Btn:
	var headline := ""
	var detail := ""
	var badge := ""
	var badge_colour := Color(0.55, 0.56, 0.66)
	var old := false

	func _init(min_size: Vector2) -> void:
		super("", min_size)

	func _draw() -> void:
		var keep := fill
		if old:
			fill = Color(0.8, 0.79, 0.76)
		super._draw()
		fill = keep
		var r := Rect2(Vector2.ZERO, size).grow(-6.0)
		r.size -= Vector2(4, 6)
		r.position.y -= pop * 4.0
		var text_ink := ink if not old else Color(0.38, 0.38, 0.44)
		var f := D.font()
		draw_string(f, r.position + Vector2(34, 31), headline, HORIZONTAL_ALIGNMENT_LEFT, r.size.x * 0.55, 27, text_ink)
		draw_string(f, r.position + Vector2(36, 52), detail, HORIZONTAL_ALIGNMENT_LEFT, r.size.x * 0.55, 17, Color(text_ink, 0.72))
		# The result on a ribbon at the right, in the winner's colour.
		if badge != "":
			var band := Rect2(Vector2(r.end.x - 300, r.position.y + 12), Vector2(270, r.size.y - 24))
			var c := badge_colour if not old else badge_colour.lerp(Color(0.6, 0.6, 0.62), 0.6)
			Art.brush_band(self, band, c, seed, 12.0, true, 0.6)
			D.draw_text_centered(self, badge, band, 22, Color.WHITE)
		if old:
			var tag := Rect2(Vector2(r.end.x - 470, r.position.y + 18), Vector2(150, r.size.y - 36))
			D.draw_slant(self, tag, Color(0.32, 0.33, 0.42), Color(0, 0, 0, 0), 0.0, false)
			D.draw_text_centered(self, "OLD VERSION", tag, 15, Color(0.92, 0.92, 0.95))
