extends RefCounted
## Drawing helpers and colours for the menu look: slanted (parallelogram) labels and buttons, round arrow buttons, a soft backdrop with light beams, and
## bold italic type. Sleek but cartoony: flat colours, thick edges, a little bounce. Everything here only draws; menu
## screens decide what the pieces do.

const INK := Color(0.11, 0.12, 0.17)
# (The panels' light surfaces: a clean cool white and a light steel, not a warm parchment, which read as dated.)
const CREAM := Color(0.95, 0.96, 0.98)
const CREAM_DARK := Color(0.78, 0.81, 0.88)
const SKY := Color(0.74, 0.88, 0.95)
const GOLD := Color(1.0, 0.82, 0.22)
const TEAL := Color(0.2, 0.62, 0.66)
const SLATE_TOP := Color(0.25, 0.3, 0.36)
const SLATE_BOTTOM := Color(0.12, 0.15, 0.2)
const RED := Color(0.92, 0.28, 0.3)
const BLUE := Color(0.25, 0.5, 0.95)
const SKEW := 0.22

static var _font: Font
static var _display: Font


## The game's type: Barlow Condensed (godot/fonts, SIL Open Font License), a bold condensed italic. `font()` for text, `display_font()`
## (the extra-bold cut) for titles and big labels. Should the files be missing, a similar system face stands in.
static func font() -> Font:
	if _font == null:
		_font = _load("res://fonts/BarlowCondensed-BoldItalic.ttf")
	return _font


static func display_font() -> Font:
	if _display == null:
		_display = _load("res://fonts/BarlowCondensed-ExtraBoldItalic.ttf")
	return _display


static func _load(path: String) -> Font:
	if ResourceLoader.exists(path):
		var f: FontFile = load(path)
		if f != null:
			return f
	var sys := SystemFont.new()
	sys.font_names = PackedStringArray(["Bahnschrift SemiBold Condensed", "Bahnschrift Condensed", "Arial Narrow", "Arial"])
	sys.font_italic = true
	sys.font_weight = 700
	return sys


static func skew_points(rect: Rect2, skew := SKEW) -> PackedVector2Array:
	var s := rect.size.y * skew
	return PackedVector2Array([
		Vector2(rect.position.x + s, rect.position.y),
		Vector2(rect.end.x, rect.position.y),
		Vector2(rect.end.x - s, rect.end.y),
		Vector2(rect.position.x, rect.end.y),
	])


## A slanted panel: a soft two-step shadow, a fill that is lighter at the top (a little gloss), a thin highlight under the top edge,
## and an optional ink edge.
static func draw_slant(c: CanvasItem, rect: Rect2, fill: Color, edge := Color(0, 0, 0, 0), edge_width := 3.0, shadow := true, skew := SKEW) -> void:
	if shadow:
		c.draw_colored_polygon(skew_points(Rect2(rect.position + Vector2(6, 10), rect.size), skew), Color(0, 0, 0, 0.12 * fill.a))
		c.draw_colored_polygon(skew_points(Rect2(rect.position + Vector2(3, 5), rect.size), skew), Color(0, 0, 0, 0.22 * fill.a))
	var pts := skew_points(rect, skew)
	var top := fill.lightened(0.16)
	var bottom := fill.darkened(0.1)
	top.a = fill.a
	bottom.a = fill.a
	c.draw_polygon(pts, PackedColorArray([top, top, bottom, bottom]))
	if fill.a > 0.5 and rect.size.y > 18.0:
		var s := rect.size.y * skew
		var y := rect.position.y + 3.0
		c.draw_line(Vector2(rect.position.x + s + 4.0, y), Vector2(rect.end.x - 6.0, y), Color(1, 1, 1, 0.35), 2.0, true)
	if edge.a > 0.0:
		pts.append(pts[0])
		c.draw_polyline(pts, edge, edge_width, true)


## A soft glow around a slanted panel (selected buttons and tiles).
static func draw_glow(c: CanvasItem, rect: Rect2, color: Color, strength := 1.0, skew := SKEW) -> void:
	for k in 4:
		var g := rect.grow(4.0 + k * 4.0)
		var pts := skew_points(g, skew)
		pts.append(pts[0])
		c.draw_polyline(pts, Color(color.r, color.g, color.b, 0.22 * strength * (1.0 - k / 4.0)), 4.0, true)


static func draw_text_centered(c: CanvasItem, text: String, rect: Rect2, size: int, color: Color) -> void:
	var f := font()
	var w := f.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, size)
	var asc := f.get_ascent(size)
	var desc := f.get_descent(size)
	c.draw_string(f, Vector2(rect.position.x + (rect.size.x - w.x) * 0.5, rect.position.y + (rect.size.y + asc - desc) * 0.5), text, HORIZONTAL_ALIGNMENT_LEFT, -1, size, color)


