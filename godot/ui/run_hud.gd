extends CanvasLayer
## The run's display (the single-player run, `scripts/runner.gd`), in the menus' painted style: the distance on an ink brush stroke at
## the top left with the coins under it, the combo and its coin multiplier at the top right (a bar fills toward the next step), and the
## fighter's damage and lives at the bottom left. A banner in the middle for the start, a lost life and the end. At the top in the
## middle, the map: a strip of the rooms behind (dimmed; those dropped behind the wall shown shut), the current one and the next one
## ahead, each with its type's icon (a fork shows its two roads splitting; a branching fork's two rooms stand one over the other, the
## treasure alley above, and the road not taken is crossed out; a rare event is a "?"), and the difficulty tier beside it (R2b). A
## prompt at a shop's door, a hint while an arena's gate is
## shut, and inside the shop the prompt for the ware stood by or the way out (the shop itself is a room you walk around in: see
## `scripts/run_stage.gd`).
## Everything it shows comes from `show_state`; nothing here decides anything.

const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")

var board: Control


class Board extends Control:
	## {metres, coins, kills, combo, best, mult, step, lives, max_lives, percent, name, banner, banner_alpha, combo_flash, map, tier,
	## door, gate, shop, shop_names, decks, deck_names}
	var data := {}

	const ROOM_STREET := 0
	const ROOM_ARENA := 1
	const ROOM_SHOP := 2
	const ROOM_REST := 3
	const ROOM_EVENT := 4
	const ROOM_ALLEY := 5
	const MAP_STRIDE := 5
	## `SimRunner.run_plan`'s numbers a node (R2e).
	const PLAN_STRIDE := 9

	## One room's icon on the map (placeholder shapes for the art director): a street is a little building, an arena crossed
	## swords, a shop a coin, a rest stop a green cross.
	func room_icon(at: Vector2, kind: int, r: float, alpha: float) -> void:
		var ink := Color(Art.INK, alpha)
		match kind:
			ROOM_ARENA:
				draw_line(at + Vector2(-r, -r) * 0.6, at + Vector2(r, r) * 0.6, Color(Art.CORAL, alpha), 4.0)
				draw_line(at + Vector2(r, -r) * 0.6, at + Vector2(-r, r) * 0.6, Color(Art.CORAL, alpha), 4.0)
			ROOM_SHOP:
				draw_circle(at, r * 0.62, ink)
				draw_circle(at, r * 0.5, Color(Art.SUN, alpha))
			ROOM_REST:
				draw_rect(Rect2(at - Vector2(r * 0.18, r * 0.6), Vector2(r * 0.36, r * 1.2)), Color(Art.GREEN, alpha))
				draw_rect(Rect2(at - Vector2(r * 0.6, r * 0.18), Vector2(r * 1.2, r * 0.36)), Color(Art.GREEN, alpha))
			ROOM_EVENT:
				# A rare event: a gold "?".
				draw_circle(at, r * 0.7, Color(Art.SUN, alpha))
				draw_string(D.display_font(), at + Vector2(-r * 0.32, r * 0.42), "?", HORIZONTAL_ALIGNMENT_LEFT, -1, int(r * 1.3), Color(Art.INK, alpha))
			ROOM_ALLEY:
				# A treasure alley: a little chest.
				draw_rect(Rect2(at + Vector2(-r * 0.55, -r * 0.1), Vector2(r * 1.1, r * 0.6)), Color(0.58, 0.36, 0.2, alpha))
				draw_rect(Rect2(at + Vector2(-r * 0.55, -r * 0.45), Vector2(r * 1.1, r * 0.35)), Color(Art.SUN, alpha))
				draw_rect(Rect2(at + Vector2(-r * 0.1, -r * 0.2), Vector2(r * 0.2, r * 0.25)), Color(Art.INK, alpha))
			_:
				draw_rect(Rect2(at + Vector2(-r * 0.45, -r * 0.55), Vector2(r * 0.9, r * 1.1)), Color(Art.BLUE, alpha * 0.85))
				for n in 2:
					draw_rect(Rect2(at + Vector2(-r * 0.3 + n * r * 0.35, -r * 0.35), Vector2(r * 0.22, r * 0.22)), Color(1, 1, 1, alpha))

	## A fork's mark over its icon: the road splitting in two, one branch up (the high road), one along (the low road).
	func fork_mark(at: Vector2, r: float, alpha: float) -> void:
		var c := Color(Art.SUN, alpha)
		var root := at + Vector2(-r * 0.9, -r * 1.25)
		draw_line(root, root + Vector2(r * 0.6, 0), c, 3.0)
		draw_line(root + Vector2(r * 0.6, 0), root + Vector2(r * 1.6, -r * 0.5), c, 3.0)
		draw_line(root + Vector2(r * 0.6, 0), root + Vector2(r * 1.7, 0), c, 3.0)

	## The map: the rooms behind (oldest on the left) and the current one, from `map` (rows [number, type, status (0 shut behind, 1
	## behind, 2 here, 3 ahead, 4 a road not taken), fork (0/1), tier]); then (R2e, F4) the small node map ahead from `plan` (rows of
	## `PLAN_STRIDE`: [how far ahead, line (0 main, 1 a fork's high road, 2 its low road), type, fork, deck change, chest, laid,
	## number, a recovery route]): about three rooms, a branching fork's two roads one above the other with the line after each,
	## each with its icon (a shop, a chest or alley, an arena, a "?" event, an arrow for a deck change up or down). The current room's
	## tier is written at the right end.
	func draw_map(centre_x: float, top: float) -> void:
		var map: PackedInt32Array = data.get("map", PackedInt32Array())
		var plan: PackedInt32Array = data.get("plan", PackedInt32Array())
		var count := map.size() / MAP_STRIDE
		if count == 0:
			return
		# Behind and here (the rooms ahead are the node map's). A road not taken shares the column of the one taken.
		var rows: Array[int] = []
		for n in count:
			if map[n * MAP_STRIDE + 2] != 3:
				rows.append(n)
		var cols: Array[int] = []
		var col := 0
		var here_col := 0
		for i in rows.size():
			var n: int = rows[i]
			var skipped := map[n * MAP_STRIDE + 2] == 4
			if skipped and col > 0:
				cols.append(col - 1)
				continue
			cols.append(col)
			if map[n * MAP_STRIDE + 2] == 2:
				here_col = col
			col += 1
		var ahead := 0
		for k in plan.size() / PLAN_STRIDE:
			ahead = maxi(ahead, plan[k * PLAN_STRIDE])
		var columns := maxi(here_col + 1 + ahead, 1)
		var gap := 54.0
		var start := centre_x - (columns - 1) * gap * 0.5
		var y := top + 42.0
		draw_line(Vector2(start, y), Vector2(start + here_col * gap, y), Color(Art.INK, 0.6), 6.0)
		var placed := {}
		for i in rows.size():
			var o: int = rows[i] * MAP_STRIDE
			var status := map[o + 2]
			var at := Vector2(start + cols[i] * gap, y + (22.0 if status == 4 else 0.0))
			var r := 22.0 if status == 2 else 16.0
			var alpha := 1.0 if status == 2 else 0.55
			draw_circle(at, r + 3.0, Art.INK)
			draw_circle(at, r, Color(Art.CREAM, 0.35 if status == 0 or status == 4 else alpha))
			if status == 2:
				draw_arc(at, r + 6.0, 0.0, TAU, 32, Art.SUN, 4.0)
				placed["0:0"] = at
			if status == 0:
				for k in 3:
					var bx := at.x - r * 0.5 + k * r * 0.5
					draw_line(Vector2(bx, at.y - r * 0.8), Vector2(bx, at.y + r * 0.8), Color(Art.INK, 0.8), 3.0)
			else:
				room_icon(at, map[o + 1], r, alpha * (0.5 if status == 4 else 1.0))
				if map[o + 3] == 1:
					fork_mark(at, r, alpha)
				if status == 4:
					draw_line(at + Vector2(-r, -r) * 0.7, at + Vector2(r, r) * 0.7, Color(Art.INK, 0.85), 3.0)
		# The node map ahead: each node joined to the one before it on its line (or the fork it splits from).
		var here: Vector2 = placed.get("0:0", Vector2(start + here_col * gap, y))
		var line_y := {0: 0.0, 1: -27.0, 2: 27.0}
		var nodes := plan.size() / PLAN_STRIDE
		for k in nodes:
			var o := k * PLAN_STRIDE
			var depth := plan[o]
			var line := plan[o + 1]
			var at := Vector2(here.x + depth * gap, y + float(line_y.get(line, 0.0)))
			var from: Vector2 = placed.get("%d:%d" % [depth - 1, line], placed.get("%d:0" % (depth - 1), here))
			draw_line(from, at, Color(Art.SUN if line > 0 else Art.INK, 0.75), 4.0)
			placed["%d:%d" % [depth, line]] = at
		for k in nodes:
			var o := k * PLAN_STRIDE
			var at: Vector2 = placed["%d:%d" % [plan[o], plan[o + 1]]]
			var r := 16.0
			var alpha := 1.0 if plan[o + 6] == 1 else 0.85
			draw_circle(at, r + 3.0, Art.INK)
			draw_circle(at, r, Color(Art.CREAM, alpha))
			room_icon(at, plan[o + 2], r, alpha)
			if plan[o + 3] == 1:
				fork_mark(at, r, alpha)
			if plan[o + 5] == 1 and plan[o + 2] != ROOM_ALLEY:
				# A chest in it: a little chest at its foot.
				var c := at + Vector2(r * 0.55, r * 0.55)
				draw_rect(Rect2(c - Vector2(6, 3), Vector2(12, 7)), Color(0.58, 0.36, 0.2))
				draw_rect(Rect2(c - Vector2(6, 6), Vector2(12, 4)), Art.SUN)
			var change := plan[o + 4]
			if change != 0:
				# A deck change: an arrow up or down at its shoulder.
				var tip := at + Vector2(r + 4.0, -r * 0.2 if change > 0 else r * 0.2)
				var dy := -10.0 if change > 0 else 10.0
				var c2 := Color(Art.CORAL, 0.95)
				draw_line(tip - Vector2(0, dy), tip, c2, 3.0)
				draw_line(tip, tip + Vector2(-4, -dy * 0.5), c2, 3.0)
				draw_line(tip, tip + Vector2(4, -dy * 0.5), c2, 3.0)
			if plan[o] == 1 and plan[o + 1] == 0:
				draw_string(D.font(), at + Vector2(-14, r + 20), "next", HORIZONTAL_ALIGNMENT_LEFT, -1, 14, Color(1, 1, 1, 0.8))
		draw_decks(start - 44.0, y)
		# The difficulty tier.
		var tier := int(data.get("tier", 0))
		var tx := start + (columns - 1) * gap + 44.0
		draw_string(D.display_font(), Vector2(tx, y + 8.0), "TIER %d" % (tier + 1), HORIZONTAL_ALIGNMENT_LEFT, -1, 20, Color(1, 1, 1, 0.85))
		for k in tier + 1:
			draw_circle(Vector2(tx + 6.0 + k * 12.0, y + 22.0), 4.0, Art.CORAL)

	## R2e (F3): the bonuses called out, each rising and fading for a second and a half under the map ("SKYWARD! +25"), and the
	## juggle's multiplier by the combo while a grunt the player hit is kept in the air ("AIR x2").
	func draw_bonuses(w: float) -> void:
		var popups: Array = data.get("popups", [])
		var k := 0
		for p in popups:
			var t := float(p.age)
			var a := clampf(1.5 - t, 0.0, 1.0)
			var r := Rect2(Vector2(w * 0.5 - 170, 120 + k * 50 - t * 16.0), Vector2(340, 46))
			Art.brush_band(self, r, Color(Art.INK, 0.85 * a), 263, 10.0, true, 1.0)
			Art.title_centered(self, str(p.text), r.grow_individual(-8, 0, -10, 0), 30, Color(Art.SUN, a), 0.0)
			k += 1
		var air := int(data.get("air", 1))
		if air > 1:
			var tag := Rect2(Vector2(w - 300, 150), Vector2(120, 44))
			Art.brush_band(self, tag, Art.CORAL, 271, 10.0, true, 0.95)
			Art.title_centered(self, "AIR x%d" % air, tag.grow_individual(0, 0, -8, 0), 30, Color.WHITE, 0.0)

	## The decks (R2d): a little ladder left of the strip, one rung a deck (the street lowest), the player's deck lit, and an arrow
	## at the next room's deck (two at a branching fork whose roads are on two decks), with the deck's name beside it. `decks`:
	## `SimRunner.run_decks` ([the player's deck, the current room's, the next room's, the other road's or -1, how many]).
	func draw_decks(right_x: float, mid_y: float) -> void:
		var decks: PackedInt32Array = data.get("decks", PackedInt32Array())
		if decks.size() < 5 or decks[4] <= 0:
			return
		var names: PackedStringArray = data.get("deck_names", PackedStringArray())
		var n := decks[4]
		var gap := 15.0
		var x := right_x
		var bottom := mid_y + (n - 1) * gap * 0.5
		draw_line(Vector2(x, bottom + 9.0), Vector2(x, bottom - (n - 1) * gap - 9.0), Color(Art.INK, 0.75), 5.0)
		for d in n:
			var at := Vector2(x, bottom - d * gap)
			draw_circle(at, 7.5, Art.INK)
			draw_circle(at, 5.5, Art.SUN if d == decks[0] else Color(1, 1, 1, 0.35))
		for k in [2, 3]:
			var d := decks[k]
			if d < 0 or d >= n:
				continue
			var tip := Vector2(x + 11.0, bottom - d * gap)
			var c := Color(Art.CORAL, 0.95)
			draw_line(tip + Vector2(13, 0), tip, c, 3.0)
			draw_line(tip, tip + Vector2(6, -5), c, 3.0)
			draw_line(tip, tip + Vector2(6, 5), c, 3.0)
		# The player's deck by name, left of the ladder; the next room's under it when it is another.
		var font := D.font()
		if decks[0] >= 0 and decks[0] < names.size():
			var label := names[decks[0]].to_upper()
			var tw := font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, 15).x
			draw_string(font, Vector2(x - 16.0 - tw, mid_y + 5.0), label, HORIZONTAL_ALIGNMENT_LEFT, -1, 15, Color(1, 1, 1, 0.9))
		var next := decks[2]
		if next >= 0 and next < names.size() and next != decks[1]:
			var word := "UP: " if next > decks[1] else "DOWN: "
			var label := word + names[next].to_upper()
			var tw := font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, 13).x
			draw_string(font, Vector2(x - 16.0 - tw, mid_y + 22.0), label, HORIZONTAL_ALIGNMENT_LEFT, -1, 13, Color(Art.CORAL, 0.95))

	## Inside the shop: what pressing up does where the player stands (`shop`: `SimRunner.run_shop`), or nothing.
	func shop_hint() -> String:
		var shop: PackedInt32Array = data.get("shop", PackedInt32Array())
		if shop.size() < 10 or shop[0] == 0:
			return ""
		var spot := shop[1]
		if spot == -1:
			return "UP: LEAVE THE SHOP"
		if spot < 0 or 10 + spot * 4 + 3 >= shop.size():
			return ""
		var names: PackedStringArray = data.get("shop_names", PackedStringArray())
		var label := names[spot] if spot < names.size() else "?"
		# (The last number is the shopkeeper's grudge: it sells nothing until the player has gone on into a new room, R2c.)
		if shop[shop.size() - 1] > 0 and shop.size() == 10 + names.size() * 4 + 1:
			return "HMPH! CLOSED TILL YOU MOVE ON"
		var o := 10 + spot * 4
		if shop[o + 2] > 0 and shop[o + 3] >= shop[o + 2]:
			return "%s: SOLD OUT" % label.to_upper()
		return "UP: BUY %s (%d)" % [label.to_upper(), shop[o + 1]]

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	## The damage's colour: white, warming through yellow and orange to red as it climbs (as the match display's).
	static func percent_colour(p: float) -> Color:
		var t := clampf(p / 150.0, 0.0, 1.0)
		if t < 0.5:
			return Color.WHITE.lerp(Color(1.0, 0.78, 0.25), t * 2.0)
		return Color(1.0, 0.78, 0.25).lerp(Color(0.9, 0.2, 0.15), (t - 0.5) * 2.0)

	func _draw() -> void:
		if data.is_empty():
			return
		var w := size.x
		var h := size.y
		var big := D.display_font()
		var f := D.font()
		# The distance.
		var band := Rect2(Vector2(22, 18), Vector2(260, 64))
		Art.brush_band(self, band, Art.INK, 211, 16.0, true, 1.0)
		Art.title_centered(self, "%d m" % int(data.get("metres", 0)), band.grow_individual(-10, 0, -18, 0), 46, Color.WHITE, 0.0)
		# The coins: a gold coin and the count.
		var coin_at := Vector2(52, 112)
		draw_circle(coin_at, 17.0, Art.INK)
		draw_circle(coin_at, 14.0, Art.SUN)
		draw_circle(coin_at + Vector2(-3, -3), 5.0, Color(1, 1, 1, 0.55))
		draw_string(big, Vector2(78, 124), str(int(data.get("coins", 0))), HORIZONTAL_ALIGNMENT_LEFT, -1, 34, Color.WHITE)
		draw_string(f, Vector2(80, 150), "%d beaten" % int(data.get("kills", 0)), HORIZONTAL_ALIGNMENT_LEFT, -1, 18, Color(1, 1, 1, 0.8))
		# The combo, when there is one: the count, the multiplier on a gold stroke, a bar toward the next step.
		var combo := int(data.get("combo", 0))
		var mult := int(data.get("mult", 1))
		var right := Rect2(Vector2(w - 300, 18), Vector2(278, 64))
		var flash := float(data.get("combo_flash", 0.0))
		if combo > 0:
			Art.brush_band(self, right, Art.INK, 223, 16.0, true, 0.92)
			draw_string(big, right.position + Vector2(18, 46), "COMBO %d" % combo, HORIZONTAL_ALIGNMENT_LEFT, -1, 34, Color.WHITE.lerp(Art.SUN, flash))
			var tag := Rect2(right.position + Vector2(right.size.x - 92, 6), Vector2(84, 52))
			Art.brush_band(self, tag, Art.SUN, 227, 10.0, true, 0.6)
			Art.title_centered(self, "x%d" % mult, tag.grow_individual(0, 0, -8, 0), 38, Art.INK, 0.0)
			var step := maxi(1, int(data.get("step", 5)))
			var fill := 1.0 if mult >= int(data.get("max_mult", 5)) else float(combo % step) / float(step)
			var bar := Rect2(right.position + Vector2(18, right.size.y + 8), Vector2(right.size.x - 36, 10))
			draw_rect(bar, Color(Art.INK, 0.7))
			draw_rect(Rect2(bar.position, Vector2(bar.size.x * fill, bar.size.y)), Art.SUN)
		draw_string(f, Vector2(w - 290, right.position.y + right.size.y + 46), "best combo %d" % int(data.get("best", 0)), HORIZONTAL_ALIGNMENT_LEFT, -1, 18, Color(1, 1, 1, 0.75))
		# The fighter: name, damage, lives.
		var plate := Rect2(Vector2(22, h - 118), Vector2(300, 96))
		# (The match HUD's dark plate, art review 6 round 15: a cream card here and dark plates in a match read as two different games.)
		Art.card(self, plate, Color(0.11, 0.12, 0.17, 0.92), 241, -1.0, -1.0, Color(0, 0, 0, 0), 0.0, Color(0, 0, 0, 0), 4.0)
		draw_string(big, plate.position + Vector2(16, 30), str(data.get("name", "")).left(16).to_upper(), HORIZONTAL_ALIGNMENT_LEFT, -1, 22, Color(1, 1, 1))
		var p := float(data.get("percent", 0.0))
		var pc := percent_colour(p)
		var pct := "%d%%" % int(p)
		draw_string(big, plate.position + Vector2(18, 84), pct, HORIZONTAL_ALIGNMENT_LEFT, -1, 54, Art.INK)
		draw_string(big, plate.position + Vector2(15, 81), pct, HORIZONTAL_ALIGNMENT_LEFT, -1, 54, pc if p > 0.5 else Color(1, 1, 1))
		var lives := int(data.get("lives", 0))
		for n in int(data.get("max_lives", 3)):
			var at := plate.position + Vector2(plate.size.x - 34 - n * 32, 26)
			draw_circle(at, 12.0, Art.INK)
			draw_circle(at, 9.0, Color(0.95, 0.35, 0.4) if n < lives else Color(1, 1, 1, 0.25))
		# The map, and what this spot offers.
		draw_map(w * 0.5, 14.0)
		draw_bonuses(w)
		var hint := shop_hint()
		if hint == "" and bool(data.get("door", false)):
			hint = "UP: ENTER THE SHOP"
		elif hint == "" and bool(data.get("gate", false)):
			hint = "THE GATE IS SHUT: BEAT THEM ALL"
		if hint != "":
			var hr := Rect2(Vector2(w * 0.5 - 290, h - 104), Vector2(580, 56))
			Art.brush_band(self, hr, Art.INK, 261, 12.0, true, 0.9)
			Art.title_centered(self, hint, hr.grow_individual(-8, 0, -12, 0), 26, Art.SUN if hint.begins_with("UP") else Color.WHITE, 0.0)
		# The banner.
		var banner := str(data.get("banner", ""))
		if banner != "":
			var a := float(data.get("banner_alpha", 1.0))
			var r := Rect2(Vector2(w * 0.5 - 300, h * 0.36), Vector2(600, 110))
			Art.brush_band(self, r, Color(Art.INK, a), 251, 22.0, true, 1.0)
			Art.title_centered(self, banner, r.grow_individual(-12, 0, -26, 0), 72, Color(1, 1, 1, a), 0.0)


func _init() -> void:
	layer = 5


func build() -> void:
	board = Board.new()
	add_child(board)


## R2e: the callouts shown now (each {text, age}), and the run's callout count last seen.
var popups: Array = []
var callouts_seen := -1
const CALL_TEXT := ["", "AIR JUGGLE!", "SKYWARD!", "GROUNDED!", "BOWLED!"]


## Turns the sim's last callout (`bonus`: `SimRunner.run_bonus`) into a popup when it is new, and ages the others.
func _callouts(bonus: PackedFloat32Array, delta: float) -> void:
	for p in popups:
		p.age += delta
	popups = popups.filter(func(p): return p.age < 1.5)
	if bonus.size() < 6:
		return
	var n := int(bonus[1])
	if callouts_seen >= 0 and n != callouts_seen:
		var kind := int(bonus[0])
		if kind > 0 and kind < CALL_TEXT.size():
			var text: String = CALL_TEXT[kind]
			if bonus[4] > 0.5:
				text += " +%d" % int(bonus[4])
			popups.append({"text": text, "age": 0.0})
			if popups.size() > 3:
				popups.pop_front()
	callouts_seen = n


func show_state(d: Dictionary) -> void:
	if board == null:
		build()
	if d.has("bonus"):
		var bonus: PackedFloat32Array = d["bonus"]
		_callouts(bonus, float(d.get("delta", 0.016)))
		d["popups"] = popups.duplicate(true)
		d["air"] = int(bonus[5]) if bonus.size() > 5 else 1
	if d != board.data:
		board.data = d
		board.queue_redraw()
