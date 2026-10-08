extends Control
## The online screen: host a match or join one, pick your fighter, and connect. The host also chooses the rules (stocks, time,
## ranked); the joiner plays under whatever the host chose. Up/Down choose a row, Left/Right change it, Enter connects, Esc goes back.
##
## Connecting works two ways: direct (the host opens a UDP port, the joiner types the host's address) or through a relay
## server (`pftool net-relay`), where both type the relay's address and a room number.

const UI := preload("res://ui/ui_kit.gd")
const Preview := preload("res://ui/preview.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")

const SETTINGS_PATH := "user://online.json"
const DEFAULT_PORT := 47000
const DELAYS := [0, 1, 2, 3, 4, 5, 6]

const DESCRIPTIONS := {
	"role": "Host starts the match and decides the rules. Join connects to a host. Watch lets you spectate a match a host is playing (type the host's address; the host needs the next port open too).",
	"link": "Direct: the host opens a UDP port (forward it on the router if you are not on the same network) and the joiner types the host's address. Relay: both connect to a relay server with the same room number.",
	"address": "Direct join: the host's address, like 203.0.113.5:47000. Relay: the relay server's address.",
	"port": "The UDP port to host on. Friends outside your network need this port forwarded on the router.",
	"room": "Both players type the same room number to meet on the relay.",
	"fighter": "Your fighter for this match. Made fighters work online: only a few bytes travel, and both sides build the same fighter.",
	"delay": "Input delay in frames. More delay hides lag better but feels heavier. 2 is good for most connections.",
	"stage": "The stage to play on (the host decides).",
	"stocks": "How many lives each fighter has (the host decides).",
	"time": "Time limit; when it runs out the fighter with more stocks, then less damage, wins (the host decides).",
	"rules": "Ranked refuses fighters over the point budget, on either side (the host decides).",
}

var rows: Array = []        # [{id, node}]
var focus := 0
var selectors := {}
var edits := {}
var entries: Array = []
var preview: Control
var bubble: Control
var status: Control
var connect_button: Control
var column: VBoxContainer
var settings := {}


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	Roster.load_match_rules()
	settings = _load_settings()
	entries = Roster.all()
	add_child(UI.Backdrop.new(false))

	var ribbon := UI.Tag.new("Online Match", Vector2(560, 58))
	ribbon.fill = UI.SKY
	ribbon.font_size = 32
	ribbon.edge = UI.INK
	ribbon.position = Vector2(0, 14)
	add_child(ribbon)

	column = VBoxContainer.new()
	column.position = Vector2(40, 92)
	column.add_theme_constant_override("separation", 5)
	add_child(column)

	_add_selector("role", "Role", ["Host", "Join", "Watch"])
	_add_selector("link", "Connect", ["Direct", "Relay"])
	_add_edit("address", "Address", "ip:port")
	_add_edit("port", "Port", "47000")
	_add_edit("room", "Room", "number")
	var names := []
	for e in entries:
		names.append(e.name)
	_add_selector("fighter", "Fighter", names)
	_add_selector("delay", "Delay", DELAYS.map(func(d): return "%d frames" % d))
	_add_selector("stage", "Stage", Roster.stage_names())
	_add_selector("stocks", "Stocks", Roster.STOCK_CHOICES.map(Roster.stocks_text))
	_add_selector("time", "Time", Roster.TIME_CHOICES.map(Roster.time_text))
	_add_selector("rules", "Rules", ["Casual", "Ranked"])

	preview = Preview.new(Vector2i(420, 480), true)
	preview.position = Vector2(640, 150)
	add_child(preview)
	bubble = UI.Bubble.new(Vector2(600, 120))
	bubble.position = Vector2(660, 16)
	add_child(bubble)

	status = UI.Tag.new("", Vector2(700, 40))
	status.fill = Color(1, 1, 1, 0.0)
	status.ink = UI.GOLD
	status.shadow = false
	status.font_size = 22
	status.position = Vector2(60, 612)
	add_child(status)

	connect_button = UI.Btn.new("Connect", Vector2(300, 64))
	connect_button.position = Vector2(960, 640)
	connect_button.activated.connect(_connect)
	add_child(connect_button)
	var back := UI.Btn.new("Back", Vector2(190, 64))
	back.font_size = 30
	back.position = Vector2(740, 640)
	back.activated.connect(_back)
	add_child(back)

	_apply_settings()
	_update_rows()
	_set_focus(0)
	PadNav.attach(self)
	Shot.attach(self)


# ---- Rows -------------------------------------------------------------------------------------------------------------------

func _add_selector(id: String, label: String, options: Array) -> void:
	var s := UI.Selector.new(label, options)
	s.changed.connect(func(_i): _on_changed(id))
	s.focused.connect(func(): _focus_id(id))
	column.add_child(s)
	selectors[id] = s
	rows.append({"id": id, "node": s})


func _add_edit(id: String, label: String, placeholder: String) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	var tag := UI.Tag.new(label, Vector2(150, 38))
	tag.fill = UI.CREAM_DARK
	tag.font_size = 22
	row.add_child(tag)
	var edit := LineEdit.new()
	edit.custom_minimum_size = Vector2(330, 38)
	edit.max_length = 64
	edit.placeholder_text = placeholder
	edit.add_theme_font_override("font", UI.font())
	edit.add_theme_font_size_override("font_size", 24)
	edit.add_theme_color_override("font_color", UI.INK)
	edit.add_theme_color_override("font_placeholder_color", Color(0.4, 0.4, 0.45))
	var sb := StyleBoxFlat.new()
	sb.bg_color = UI.CREAM
	sb.border_color = UI.GOLD
	sb.set_border_width_all(4)
	sb.set_corner_radius_all(6)
	sb.content_margin_left = 14
	edit.add_theme_stylebox_override("normal", sb)
	edit.add_theme_stylebox_override("focus", sb)
	edit.text_submitted.connect(func(_t): _connect())
	edit.focus_entered.connect(func(): _focus_id(id, false))
	edit.gui_input.connect(_edit_key)
	row.add_child(edit)
	column.add_child(row)
	edits[id] = edit
	rows.append({"id": id, "node": row, "tag": tag})


func _hosting() -> bool:
	return selectors["role"].index == 0


func _watching() -> bool:
	return selectors["role"].index == 2


func _relay() -> bool:
	return selectors["link"].index == 1 and not _watching()


## Which rows make sense for the chosen role and connection.
func _row_shown(id: String) -> bool:
	match id:
		"link", "fighter", "delay":
			return not _watching()
		"address":
			return not _hosting() or _relay()
		"port":
			return _hosting() and not _relay()
		"room":
			return _relay()
		"stage", "stocks", "time", "rules":
			return _hosting()
	return true


func _update_rows() -> void:
	for r in rows:
		r.node.visible = _row_shown(r.id)
	if edits.has("address"):
		var tag: Control = rows.filter(func(r): return r.id == "address")[0].tag
		tag.set_text("Relay" if _relay() else "Host")
	connect_button.text = "Host!" if _hosting() else ("Watch!" if _watching() else "Join!")
	connect_button.queue_redraw()
	if not _row_shown(rows[focus].id):
		_set_focus(focus + 1)


func _on_changed(id: String) -> void:
	if id == "role" or id == "link":
		_update_rows()
	if id == "fighter":
		_show_fighter()
	_describe()
	status.set_text("")


func _show_fighter() -> void:
	var e: Dictionary = entries[selectors["fighter"].index]
	preview.set_loadout(e.look)
	var ed: RefCounted = ClassDB.instantiate("ContentEditor")
	ed.new_from_builtin("preview")
	var r: Dictionary = ed.recipe_readout(e["class"], e.size, e.speed, e.jump, e.weight)
	preview.set_size_percent(float(r.size_percent))


func _describe() -> void:
	# Deferred: a bubble laid out before it has a size wraps its text one word per line.
	bubble.set_text.call_deferred(DESCRIPTIONS.get(rows[focus].id, ""))


# ---- Focus and keys -----------------------------------------------------------------------------------------------------------

func _focus_id(id: String, release_edit := true) -> void:
	for i in rows.size():
		if rows[i].id == id:
			_set_focus(i, release_edit)


func _set_focus(i: int, release_edit := true) -> void:
	var step := 1 if i >= focus else -1
	var n := rows.size()
	focus = (i + n) % n
	for _k in n:
		if _row_shown(rows[focus].id):
			break
		focus = (focus + step + n) % n
	for k in n:
		var node = rows[k].node
		if node.has_method("set_focus_mark"):
			node.set_focus_mark(k == focus)
	var id: String = rows[focus].id
	if edits.has(id):
		edits[id].grab_focus()
	elif release_edit:
		for e in edits.values():
			e.release_focus()
	_describe()


func _edit_key(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed:
		match event.keycode:
			KEY_UP:
				_set_focus(focus - 1)
				get_viewport().set_input_as_handled()
			KEY_DOWN:
				_set_focus(focus + 1)
				get_viewport().set_input_as_handled()
			KEY_ESCAPE:
				for e in edits.values():
					e.release_focus()
				get_viewport().set_input_as_handled()


func _unhandled_key_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.pressed or event.echo:
		return
	var id: String = rows[focus].id
	match event.keycode:
		KEY_UP, KEY_W:
			_set_focus(focus - 1)
		KEY_DOWN, KEY_S:
			_set_focus(focus + 1)
		KEY_LEFT, KEY_A:
			if selectors.has(id):
				selectors[id].step(-1)
		KEY_RIGHT, KEY_D:
			if selectors.has(id):
				selectors[id].step(1)
		KEY_ENTER, KEY_KP_ENTER:
			_connect()
		KEY_ESCAPE:
			_back()


func _back() -> void:
	_save_settings()
	get_tree().change_scene_to_file("res://menu.tscn")


# ---- Settings kept between runs ---------------------------------------------------------------------------------------------

func _load_settings() -> Dictionary:
	var out := {"role": 0, "link": 0, "address": "", "port": str(DEFAULT_PORT), "room": "1", "delay": 2, "fighter": ""}
	if FileAccess.file_exists(SETTINGS_PATH):
		var json := JSON.new()
		var parsed = json.data if json.parse(FileAccess.get_file_as_string(SETTINGS_PATH)) == OK else null
		if parsed is Dictionary:
			for k in out:
				if parsed.has(k):
					out[k] = parsed[k]
	return out


func _apply_settings() -> void:
	selectors["role"].set_index(int(settings.role), false)
	selectors["link"].set_index(int(settings.link), false)
	edits["address"].text = str(settings.address)
	edits["port"].text = str(settings.port)
	edits["room"].text = str(settings.room)
	selectors["delay"].set_index(maxi(0, DELAYS.find(int(settings.delay))), false)
	var fighter_index := 0
	var wanted: String = str(settings.fighter) if settings.fighter != "" else Roster.net_entry().slug
	for i in entries.size():
		if entries[i].slug == wanted:
			fighter_index = i
	selectors["fighter"].set_index(fighter_index, false)
	selectors["stage"].set_index(Roster.match_stage, false)
	selectors["stocks"].set_index(maxi(0, Roster.STOCK_CHOICES.find(Roster.match_stocks)), false)
	selectors["time"].set_index(maxi(0, Roster.TIME_CHOICES.find(Roster.match_time)), false)
	selectors["rules"].set_index(0, false)
	_show_fighter()


func _save_settings() -> void:
	var f := FileAccess.open(SETTINGS_PATH, FileAccess.WRITE)
	if f != null:
		f.store_string(JSON.stringify({
			"role": selectors["role"].index, "link": selectors["link"].index, "address": edits["address"].text.strip_edges(),
			"port": edits["port"].text.strip_edges(), "room": edits["room"].text.strip_edges(),
			"delay": DELAYS[selectors["delay"].index], "fighter": entries[selectors["fighter"].index].slug,
		}))
	Roster.match_stage = selectors["stage"].index
	Roster.match_stocks = Roster.STOCK_CHOICES[selectors["stocks"].index]
	Roster.match_time = Roster.TIME_CHOICES[selectors["time"].index]
	Roster.save_match_rules()


# ---- Connecting ----------------------------------------------------------------------------------------------------------------

## The connection the screen describes: {"host", "relay", "addr", "port", "room", "delay", "entry", ...} or {"error": text}.
func connection() -> Dictionary:
	var c := {
		"host": _hosting(), "watch": false, "relay": _relay(), "addr": edits["address"].text.strip_edges(),
		"port": DEFAULT_PORT, "room": 0, "delay": DELAYS[selectors["delay"].index],
		"entry": entries[selectors["fighter"].index],
		"stage": selectors["stage"].index, "stocks": Roster.STOCK_CHOICES[selectors["stocks"].index], "time": Roster.TIME_CHOICES[selectors["time"].index],
		"ranked": selectors["rules"].index == 1,
	}
	if _relay():
		if c.addr == "":
			return {"error": "Type the relay server's address (like 203.0.113.5:47001)."}
		if not c.addr.contains(":"):
			return {"error": "The relay address needs a port, like 203.0.113.5:47001."}
		var room: String = edits["room"].text.strip_edges()
		if not room.is_valid_int() or int(room) < 1:
			return {"error": "Type a room number (1 or more); both players use the same one."}
		c.room = int(room)
	elif _watching():
		c.watch = true
		if c.addr == "":
			return {"error": "Type the host's address (like 203.0.113.5:47000). The host must also have the next port open."}
		if not c.addr.contains(":"):
			c.addr += ":%d" % DEFAULT_PORT
		return c
	elif _hosting():
		var port: String = edits["port"].text.strip_edges()
		if not port.is_valid_int() or int(port) < 1024 or int(port) > 65535:
			return {"error": "The port must be a number from 1024 to 65535."}
		c.port = int(port)
	else:
		if c.addr == "":
			return {"error": "Type the host's address (like 203.0.113.5:47000)."}
		if not c.addr.contains(":"):
			c.addr += ":%d" % DEFAULT_PORT
	if c.ranked and not Roster.ranked_legal(c.entry):
		return {"error": "%s is over the %d point budget: not allowed under ranked rules." % [c.entry.name, Roster.budget()]}
	return c


func _connect() -> void:
	var c := connection()
	if c.has("error"):
		status.set_text(c.error)
		return
	_save_settings()
	Roster.save_last(c.entry.slug)
	Roster.session = {"from_menu": true, "online": c, "stocks": c.stocks, "time": c.time, "stage": c.stage}
	get_tree().change_scene_to_file("res://main.tscn")
