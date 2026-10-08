extends Control
## The replay list: every finished match is saved automatically; pick one to watch it. Up/Down choose, Enter watches,
## Delete removes, Esc goes back. While watching: Space pauses, Left/Right jump 5 seconds, Up/Down change the speed, Esc returns here.

const UI := preload("res://ui/ui_kit.gd")
const Roster := preload("res://scripts/roster.gd")
const Replays := preload("res://scripts/replays.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")

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
	var ribbon := UI.Tag.new("Replays", Vector2(520, 58))
	ribbon.fill = UI.SKY
	ribbon.font_size = 32
	ribbon.edge = UI.INK
	ribbon.position = Vector2(0, 14)
	add_child(ribbon)

	scroll = ScrollContainer.new()
	scroll.position = Vector2(60, 100)
	scroll.size = Vector2(1160, 520)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)
	list = VBoxContainer.new()
	list.add_theme_constant_override("separation", 8)
	scroll.add_child(list)

	note = UI.Tag.new("", Vector2(1000, 44))
	note.fill = Color(1, 1, 1, 0)
	note.ink = UI.GOLD
	note.shadow = false
	note.font_size = 24
	note.position = Vector2(60, 630)
	add_child(note)
	var hint := UI.Tag.new("Up / Down choose     Enter watch     Delete remove     Esc back", Vector2(760, 36))
	hint.fill = Color(1, 1, 1, 0.12)
	hint.ink = Color(1, 1, 1, 0.8)
	hint.font_size = 20
	hint.shadow = false
	hint.position = Vector2(60, 676)
	add_child(hint)
	_fill()
	PadNav.attach(self)
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
		var b := UI.Btn.new(_label(p, info), Vector2(1120, 58))
		b.font_size = 26
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


func _label(path: String, info: Dictionary) -> String:
	var stamp: String = path.get_file().get_basename().replace("_", "  ").substr(0, 21)
	if not info.ok:
		return "%s    (damaged file)" % stamp
	var player_names := []
	for i in int(info.players):
		var n: String = Roster.parse_profile(info["cosmetics%d" % i], i).name
		player_names.append(n if n != "" else "Player %d" % (i + 1))
	var versus := " vs ".join(player_names.map(func(n): return n.left(10 if player_names.size() > 2 else 12)))
	var result := "unfinished"
	if int(info.winner) >= 0 and int(info.winner) < player_names.size():
		result = "%s won" % player_names[int(info.winner)].left(12)
	elif int(info.winner) == -2:
		result = "draw"
	var secs: int = int(info.frames) / 60
	var rules := "free play" if int(info.stocks) == 0 else "%d stock%s" % [info.stocks, "" if int(info.stocks) == 1 else "s"]
	var old := "" if info.playable else "   (older version)"
	return "%s    %s    %s    %d:%02d    %s%s" % [stamp, versus, result, secs / 60, secs % 60, rules, old]


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
