extends Control
## The online screen: host a match or join one, pick your fighter, and connect. The host also chooses the rules (stocks, time,
## ranked); the joiner plays under whatever the host chose. Up/Down choose a row, Left/Right change it, Enter connects, Esc goes back.
##
## Connecting works three ways. Code (the default): each player sees a code, the two swap codes over any chat and both press
## Connect; the game finds a way through both routers (UDP hole punching, `transport::punch`) and the lower code hosts. Direct: the
## host opens a UDP port and the joiner types the host's address. Relay: both type a relay server's address (`pftool net-relay`) and a
## room number.

const Music := preload("res://scripts/music.gd")
const UI := preload("res://ui/ui_kit.gd")
const Preview := preload("res://ui/preview.gd")
const Roster := preload("res://scripts/roster.gd")
const Shot := preload("res://ui/shot.gd")
const PadNav := preload("res://scripts/pad_nav.gd")
const CodeKeeper := preload("res://scripts/code_keeper.gd")

const SETTINGS_PATH := "user://online.json"
const DEFAULT_PORT := 47000
const DELAYS := [0, 1, 2, 3, 4, 5, 6]

const DESCRIPTIONS := {
	"mode": "A duel is two players. A group match has a lobby for up to four: the host starts it when everyone is in. Everyone must choose the same mode, and a group match needs a direct connection (no relay).",
	"link": "Code: swap codes with your friend (Discord, any chat) and both press Connect. No port forwarding for most routers. If Windows asks about network access, press Allow. Direct and Relay are the older ways.",
	"mycode": "Your code. Copy it (Enter, or the Copy button) and send it to your friend. If Windows asks whether to allow the game on the network, press Allow.",
	"friend": "Paste your friend's code here (Ctrl+V or Paste), then both press Connect. The player whose code comes first hosts, and their rules are used.",
	"role": "Host starts the match and decides the rules. Join connects to a host. Watch spectates a hosted match (the host needs the next port open).",
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

## What to try when a code connection fails (also in How to play.txt).
const CODE_FAILED := "Could not reach your friend. Check you both pressed Connect with each other's codes. Some routers block this (phone hotspots, some internet providers). Then: 1) one player forwards UDP port 47000 and you use Direct; 2) both install a free virtual-LAN app (Tailscale, ZeroTier or Radmin VPN) and use Direct with the address it shows; 3) on the same network, use Direct with the host's local address."

## `NetCode.poll()` states.
const CODE_FINDING := 0
const CODE_READY := 1
const CODE_WAITING := 2
const CODE_CONNECTED := 3
const CODE_FAILED_STATE := 4

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
## Playing by code: the socket and its state (bridge `NetCode`), the code shown, and the last state seen.
var netcode: RefCounted = null
var code_state := -1
var code_label: Control
## Tests set these before `_ready` to skip the internet (STUN) and use loopback.
var code_stun := PackedStringArray()
var code_lan := ""


func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	Roster.load_match_rules()
	settings = _load_settings()
	entries = Roster.all()
	add_child(UI.Backdrop.new(false, 0.38))

	add_child(UI.ScreenTitle.new("Online match"))

	column = VBoxContainer.new()
	column.position = Vector2(40, 92)
	column.add_theme_constant_override("separation", 5)
	add_child(column)

	_add_selector("link", "Connect", ["Code", "Direct", "Relay"])
	_add_code_row()
	_add_edit("friend", "Friend", "paste your friend's code")
	_add_paste_button()
	_add_selector("role", "Role", ["Host", "Join", "Watch", "Quick match"])
	_add_selector("mode", "Mode", ["Duel (2)", "Group (3-4)"])
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

	# The fighter on its own stage, and what the chosen row means above it.
	var stage := UI.Stage.new(Vector2(560, 470), 0.25, 47)
	stage.position = Vector2(690, 150)
	add_child(stage)
	preview = Preview.new(Vector2i(530, 438), false)
	stage.add_fighter(preview)
	bubble = UI.Bubble.new(Vector2(600, 124))
	bubble.position = Vector2(660, 16)
	add_child(bubble)

	status = UI.notice(Vector2(40, 600), 620)
	add_child(status)

	connect_button = UI.Btn.new("Connect", Vector2(300, 64))
	connect_button.position = Vector2(960, 638)
	connect_button.activated.connect(_connect)
	add_child(connect_button)
	var back := UI.Btn.new("Back", Vector2(190, 64))
	back.font_size = 30
	back.position = Vector2(750, 638)
	var keys := UI.KeyBar.new([["W S", "Choose"], ["A D", "Change"], ["Enter", "Go"], ["Esc", "Back"]])
	keys.position = Vector2(40, 672)
	keys.size = Vector2(620, 30)
	add_child(keys)
	back.activated.connect(_back)
	add_child(back)

	_apply_settings()
	_update_rows()
	_set_focus(0)
	_sync_code()
	PadNav.attach(self)
	UI.reveal(self)
	Music.of(self).play("menu")
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


## The "Your code" row: the code on a card, and a Copy button.
func _add_code_row() -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 6)
	var tag := UI.Tag.new("Your code", Vector2(150, 38))
	tag.fill = UI.CREAM_DARK
	tag.font_size = 22
	row.add_child(tag)
	code_label = UI.Tag.new("...", Vector2(330, 38))
	code_label.font_size = 24
	row.add_child(code_label)
	var copy := UI.Btn.new("Copy", Vector2(110, 38))
	copy.font_size = 22
	copy.activated.connect(_copy_code)
	row.add_child(copy)
	column.add_child(row)
	rows.append({"id": "mycode", "node": row, "tag": tag})


func _add_paste_button() -> void:
	var row: HBoxContainer = rows[rows.size() - 1].node
	var paste := UI.Btn.new("Paste", Vector2(110, 38))
	paste.font_size = 22
	paste.activated.connect(_paste_code)
	row.add_child(paste)
	edits["friend"].max_length = 80
	edits["friend"].text_changed.connect(func(_t): _show_role())


func _code() -> bool:
	return selectors["link"].index == 0


func _hosting() -> bool:
	if _code():
		return netcode != null and netcode.hosts_against(edits["friend"].text)
	return selectors["role"].index == 0


func _watching() -> bool:
	return not _code() and selectors["role"].index == 2


func _quick() -> bool:
	return not _code() and selectors["role"].index == 3


func _relay() -> bool:
	return not _code() and ((selectors["link"].index == 2 and not _watching()) or _quick())


## Which rows make sense for the chosen role and connection.
func _row_shown(id: String) -> bool:
	match id:
		"mycode", "friend":
			return _code()
		"role":
			return not _code()
		"mode":
			return not _code() and not _watching() and not _quick()
		"fighter":
			# A duel's fighter is chosen in the lobby once connected; a group match still brings the one chosen here.
			return not _watching() and not _code() and selectors["mode"].index == 1
		"delay":
			return not _watching()
		"address":
			return not _code() and (not _hosting() or _relay())
		"port":
			return not _code() and _hosting() and not _relay()
		"room":
			return not _code() and _relay() and not _quick()
		"stage":
			# A duel's stage is chosen in the lobby; a group match's host still sets it here.
			return not _code() and _hosting() and selectors["mode"].index == 1
		"stocks", "time", "rules":
			# By code either player may end up hosting, so both set rules (the host's are used).
			return _code() or _hosting() or _quick()
	return true


func _update_rows() -> void:
	for r in rows:
		r.node.visible = _row_shown(r.id)
	if edits.has("address"):
		var tag: Control = rows.filter(func(r): return r.id == "address")[0].tag
		tag.set_text("Relay" if _relay() else "Host")
	if _code():
		connect_button.text = "Connect!"
	else:
		connect_button.text = "Host!" if _hosting() else ("Watch!" if _watching() else ("Find!" if _quick() else "Join!"))
	connect_button.queue_redraw()
	if not _row_shown(rows[focus].id):
		_set_focus(focus + 1)


func _on_changed(id: String) -> void:
	if id == "role" or id == "link":
		_update_rows()
	if id == "link":
		_sync_code()
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
			if id == "mycode":
				_copy_code()
			else:
				_connect()
		KEY_ESCAPE:
			_back()


func _back() -> void:
	if code_state == CODE_WAITING:
		# Esc while waiting stops waiting; the code stays the same.
		netcode.cancel()
		status.set_text("Stopped waiting. Press Connect to try again.")
		return
	_save_settings()
	# (The code's socket stays open, so the code is the same next time: `Roster.netcode`.)
	netcode = null
	get_tree().change_scene_to_file("res://menu.tscn")


# ---- Settings kept between runs ---------------------------------------------------------------------------------------------

func _load_settings() -> Dictionary:
	var out := {"role": 0, "connect": 0, "friend_code": "", "address": "", "port": str(DEFAULT_PORT), "room": "1", "delay": 2, "fighter": ""}
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
	selectors["link"].set_index(clampi(int(settings.connect), 0, 2), false)
	edits["friend"].text = str(settings.friend_code)
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
			"role": selectors["role"].index, "connect": selectors["link"].index, "friend_code": edits["friend"].text.strip_edges(), "address": edits["address"].text.strip_edges(),
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
		"host": _hosting(), "quick": _quick(), "watch": false, "group": selectors["mode"].index == 1 and not _watching(), "relay": _relay(), "addr": edits["address"].text.strip_edges(),
		"port": DEFAULT_PORT, "room": 0, "delay": DELAYS[selectors["delay"].index],
		"entry": entries[selectors["fighter"].index],
		"stage": selectors["stage"].index, "stocks": Roster.STOCK_CHOICES[selectors["stocks"].index], "time": Roster.TIME_CHOICES[selectors["time"].index],
		"ranked": selectors["rules"].index == 1,
	}
	if _code():
		c.group = false
		c.code = true
		if netcode == null or netcode.my_code() == "":
			return {"error": "Still finding your address: wait a moment."}
		var bad: String = netcode.check_code(edits["friend"].text)
		if bad != "":
			return {"error": bad}
		if c.ranked and c.host and not Roster.ranked_legal(c.entry):
			return {"error": "%s is over the %d point budget: not allowed under ranked rules." % [c.entry.name, Roster.budget()]}
		return c
	if c.group and _relay():
		return {"error": "Group matches need a direct connection for now (choose Direct)."}
	if _quick():
		if c.addr == "":
			return {"error": "Type the relay server's address (like 203.0.113.5:47001): it pairs players who are looking."}
		if not c.addr.contains(":"):
			return {"error": "The relay address needs a port, like 203.0.113.5:47001."}
		if c.ranked and not Roster.ranked_legal(c.entry):
			return {"error": "%s is over the %d point budget: not allowed under ranked rules." % [c.entry.name, Roster.budget()]}
		return c
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
	if _code():
		if code_state == CODE_WAITING:
			return
		var err: String = netcode.connect_to(edits["friend"].text)
		if err != "":
			status.set_text(err)
			return
		bubble.label.remove_theme_font_size_override("font_size")
		_describe()
		return
	_stop_code()
	Roster.save_last(c.entry.slug)
	Roster.session = {"from_menu": true, "online": c, "stocks": c.stocks, "time": c.time, "stage": c.stage}
	get_tree().change_scene_to_file("res://main.tscn")


# ---- Playing by code ------------------------------------------------------------------------------------------------------------

## Opens the code socket when Code is chosen, or takes up the one already open (`Roster.netcode`: one code per launch), and closes
## it when another way is chosen (so Direct can host on the same port).
func _sync_code() -> void:
	if _code() and netcode == null:
		# (A socket a match never gave back, a crash or an old scene, reads -1: open a fresh one.)
		if Roster.netcode != null and Roster.netcode.poll() != -1 and code_stun.is_empty():
			netcode = Roster.netcode
			Roster.netcode.cancel()
		else:
			netcode = ClassDB.instantiate("NetCode")
			var err: String = netcode.start(DEFAULT_PORT, code_stun, code_lan)
			if err != "":
				status.set_text(err)
				netcode = null
				return
			if code_stun.is_empty():
				Roster.netcode = netcode
		code_state = -1
		code_label.set_text("...")
		CodeKeeper.attach(self)
	elif not _code():
		_stop_code()


func _stop_code() -> void:
	if netcode != null:
		netcode.stop()
		if Roster.netcode == netcode:
			Roster.netcode = null
		netcode = null
	code_state = -1


func _copy_code() -> void:
	if netcode == null or netcode.my_code() == "":
		return
	DisplayServer.clipboard_set(netcode.my_code())
	status.set_text("Copied! Send it to your friend in Discord or any chat.")


func _paste_code() -> void:
	edits["friend"].text = DisplayServer.clipboard_get().strip_edges()
	_show_role()


## Who will host, once the friend's code is in.
func _show_role() -> void:
	if netcode == null or netcode.check_code(edits["friend"].text) != "":
		return
	status.set_text("You will host: your rules are used." if _hosting() else "Your friend will host: their rules are used.")


func _process(_delta: float) -> void:
	if netcode == null:
		return
	var st: int = netcode.poll()
	if st == code_state and st != CODE_WAITING:
		return
	var was := code_state
	code_state = st
	match st:
		CODE_FINDING:
			code_label.set_text("...")
			status.set_text("Finding your address...")
		CODE_READY:
			code_label.set_text(netcode.my_code())
			if was == CODE_WAITING:
				return
			if not netcode.public_known():
				status.set_text("No internet address found: this code works on the same network only.")
			elif netcode.strict_nat():
				status.set_text("Your router looks strict: codes may not get through.")
			else:
				status.set_text("Copy your code to your friend, paste theirs, both press Connect.")
		CODE_WAITING:
			status.set_text("Waiting for your friend... %d s (both press Connect)" % netcode.seconds_left())
		CODE_CONNECTED:
			status.set_text("Connected!")
			_go_code()
		CODE_FAILED_STATE:
			status.set_text("Could not connect: see the box above for what to try.")
			bubble.label.add_theme_font_size_override("font_size", 17)
			bubble.set_text.call_deferred(CODE_FAILED)


## The friend answered: on to the match, which plays over this socket.
func _go_code() -> void:
	var c := connection()
	if c.has("error"):
		status.set_text(c.error)
		return
	c.host = netcode.i_host()
	c.netcode = netcode
	netcode = null  # lent to the match scene, which gives the socket back afterwards (`Roster.netcode`)
	Roster.save_last(c.entry.slug)
	Roster.session = {"from_menu": true, "online": c, "stocks": c.stocks, "time": c.time, "stage": c.stage}
	get_tree().change_scene_to_file("res://main.tscn")
