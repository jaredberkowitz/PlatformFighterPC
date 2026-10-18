extends CanvasLayer
## The end of a run, in the results screen's painted style over the softened stage: "RUN OVER" on a brush stroke, a card with how far
## the run went (distance, coins, mobs beaten, the best combo, rooms reached, arenas cleared, the seed), and what to do next.
## Left and right choose, Enter (or J, Space) picks, the mouse works too. It reports the choice through `chosen`.

const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")
const ItemIcons := preload("res://ui/item_icons.gd")

signal chosen(action: String)

var buttons: Array = []
var actions: Array = []
var index := 0
## {metres, coins, kills, best, rooms, cleared, seed, name, build, item_names} (R3: the build, `SimRunner.run_build`, and the item
## names by kind, `SimRunner.run_item_names`)
var stats := {}
var board: Control


class Board extends Control:
	var owner_screen

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var s: Dictionary = owner_screen.stats
		var w := size.x
		draw_rect(Rect2(Vector2.ZERO, size), Color(Art.INK, 0.45))
		Art.brush_band(self, Rect2(Vector2(w * 0.5 - 250, 132), Vector2(500, 34)), Art.SUN, 301, 14.0, false, 0.8)
		var banner := Rect2(Vector2(w * 0.5 - 300, 60), Vector2(600, 90))
		Art.brush_band(self, banner, Art.INK, 303, 22.0, true, 1.0)
		Art.title_centered(self, "RUN OVER", banner.grow_individual(-12, 0, -26, 0), 62, Color.WHITE, 0.0)
		var card := Rect2(Vector2(w * 0.5 - 330, 200), Vector2(660, 330))
		Art.card(self, card, Art.CREAM, 307, -1.0, -1.0, Art.SUN, 0.0, Color(0, 0, 0, 0), 7.0)
		var big := D.display_font()
		var f := D.font()
		draw_string(big, card.position + Vector2(36, 54), str(s.get("name", "")).left(20).to_upper(), HORIZONTAL_ALIGNMENT_LEFT, -1, 30, Art.INK)
		var rows := [["Distance", "%d m" % int(s.get("metres", 0))], ["Coins", str(int(s.get("coins", 0)))],
			["Beaten", str(int(s.get("kills", 0)))], ["Best combo", str(int(s.get("best", 0)))]]
		for n in rows.size():
			var at := card.position + Vector2(36 + (n % 2) * 320, 120 + (n / 2) * 96)
			draw_string(f, at, rows[n][0], HORIZONTAL_ALIGNMENT_LEFT, -1, 20, Color(Art.INK, 0.6))
			draw_string(big, at + Vector2(0, 50), rows[n][1], HORIZONTAL_ALIGNMENT_LEFT, -1, 48, Art.INK)
		draw_string(f, card.position + Vector2(36, card.size.y - 22), "Rooms %d    Arenas cleared %d    Seed %d" % [int(s.get("rooms", 0)), int(s.get("cleared", 0)), int(s.get("seed", 0))],
			HORIZONTAL_ALIGNMENT_LEFT, -1, 18, Color(Art.INK, 0.55))
		# R3: the build the run carried, each item with its icon and stacks.
		var names: PackedStringArray = s.get("item_names", PackedStringArray())
		var shown: Array = owner_screen.build_items()
		var bcard := Rect2(Vector2(card.end.x + 20, 200), Vector2(300, 330))
		if shown.is_empty():
			return
		Art.card(self, bcard, Art.CREAM, 311, -1.0, -1.0, Art.SUN, 0.0, Color(0, 0, 0, 0), 7.0)
		draw_string(big, bcard.position + Vector2(24, 40), "BUILD", HORIZONTAL_ALIGNMENT_LEFT, -1, 26, Art.INK)
		for k in shown.size():
			var e: Array = shown[k]
			var col: int = k % 2
			var rowy: int = k / 2
			var at := bcard.position + Vector2(36 + col * 140, 80 + rowy * 34)
			ItemIcons.draw(self, at, 13.0, e[0], e[1], e[2])
			var label: String = names[e[0] * 2] if e[0] * 2 < names.size() else ItemIcons.glyph(e[0])
			draw_string(f, at + Vector2(20, 6), label.left(12), HORIZONTAL_ALIGNMENT_LEFT, -1, 15, Art.INK)


func _init() -> void:
	layer = 10


## The build's items to list: `[kind, rarity, count]` for every kind the run owned (the staples and the slot's consumables aside).
func build_items() -> Array:
	var b: PackedInt32Array = stats.get("build", PackedInt32Array())
	var out := []
	if b.size() < 9 + ItemIcons.KINDS * 2:
		return out
	for kind in ItemIcons.KINDS:
		if kind in [ItemIcons.HEAL, ItemIcons.LIFE, ItemIcons.BUBBLE, ItemIcons.BOMB]:
			continue
		if b[9 + kind] > 0:
			out.append([kind, b[9 + ItemIcons.KINDS + kind], b[9 + kind]])
	return out


## `choices`: [[label, action], ...] (the first is selected).
func build(run_stats: Dictionary, choices: Array) -> void:
	stats = run_stats
	board = Board.new()
	board.owner_screen = self
	add_child(board)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 16)
	row.position = Vector2(640 - choices.size() * 123, 580)
	add_child(row)
	for c in choices:
		var b := UI.Btn.new(c[0], Vector2(230, 64))
		b.font_size = 26
		var action: String = c[1]
		b.activated.connect(func(): chosen.emit(action))
		row.add_child(b)
		buttons.append(b)
		actions.append(action)
	_select(0)
	add_child(UI.Reveal.new())


func _select(i: int) -> void:
	index = posmod(i, buttons.size())
	for n in buttons.size():
		buttons[n].selected = n == index


func handle_key(event: InputEventKey) -> bool:
	match event.keycode:
		KEY_LEFT, KEY_A:
			_select(index - 1)
		KEY_RIGHT, KEY_D:
			_select(index + 1)
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE, KEY_J:
			chosen.emit(actions[index])
		_:
			return false
	return true
