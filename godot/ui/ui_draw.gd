extends RefCounted
## Drawing helpers and colours for the menu look: slanted (parallelogram) labels and buttons, round arrow buttons, a soft backdrop with light beams, and
## bold italic type. Sleek but cartoony: flat colours, thick edges, a little bounce. Everything here only draws; menu
## screens decide what the pieces do.

const INK := Color(0.11, 0.12, 0.17)
const CREAM := Color(0.96, 0.95, 0.88)
const CREAM_DARK := Color(0.82, 0.8, 0.68)
const SKY := Color(0.74, 0.88, 0.95)
const GOLD := Color(1.0, 0.82, 0.22)
const TEAL := Color(0.2, 0.62, 0.66)
const SLATE_TOP := Color(0.25, 0.3, 0.36)
const SLATE_BOTTOM := Color(0.12, 0.15, 0.2)
const RED := Color(0.92, 0.28, 0.3)
const BLUE := Color(0.25, 0.5, 0.95)
const SKEW := 0.22

static var _font: Font


## Bold, italic, condensed-looking type: a system display face if there is one, the built-in font otherwise.
static func font() -> Font:
	if _font == null:
		var sys := SystemFont.new()
		sys.font_names = PackedStringArray(["Bahnschrift SemiBold Condensed", "Bahnschrift Condensed", "Bahnschrift", "Arial Narrow", "Impact", "Arial"])
		sys.font_italic = true
		sys.font_weight = 700
		var v := FontVariation.new()
		v.base_font = sys
		v.variation_embolden = 0.25
		_font = v
	return _font


static func skew_points(rect: Rect2, skew := SKEW) -> PackedVector2Array:
	var s := rect.size.y * skew
	return PackedVector2Array([
		Vector2(rect.position.x + s, rect.position.y),
		Vector2(rect.end.x, rect.position.y),
		Vector2(rect.end.x - s, rect.end.y),
		Vector2(rect.position.x, rect.end.y),
	])


static func draw_slant(c: CanvasItem, rect: Rect2, fill: Color, edge := Color(0, 0, 0, 0), edge_width := 3.0, shadow := true) -> void:
	if shadow:
		c.draw_colored_polygon(skew_points(Rect2(rect.position + Vector2(3, 5), rect.size)), Color(0, 0, 0, 0.28))
	c.draw_colored_polygon(skew_points(rect), fill)
	if edge.a > 0.0:
		var pts := skew_points(rect)
		pts.append(pts[0])
		c.draw_polyline(pts, edge, edge_width, true)


static func draw_text_centered(c: CanvasItem, text: String, rect: Rect2, size: int, color: Color) -> void:
	var f := font()
	var w := f.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, size)
	var asc := f.get_ascent(size)
	var desc := f.get_descent(size)
	c.draw_string(f, Vector2(rect.position.x + (rect.size.x - w.x) * 0.5, rect.position.y + (rect.size.y + asc - desc) * 0.5), text, HORIZONTAL_ALIGNMENT_LEFT, -1, size, color)


