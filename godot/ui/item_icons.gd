extends RefCounted
## The run's item icons (R3; placeholders for the art director, `docs/modes/RUNNER_PLAN.md` section 14): a disc in the item's own
## colour with a short glyph, ringed by its rarity (green staple, white common, blue rare, gold prized). The same look is used by
## the HUD's strip, the results screen's build and (as a gem with a label) the shop's pedestals and the finds on routes. Kinds are
## the sim's `ITEM_*` (`sim_core::run::data`).

const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")

const HEAL := 0
const LIFE := 1
const BUBBLE := 16
const BOMB := 17
const KINDS := 18

## Each kind's glyph and colour.
const GLYPHS := ["+", "1UP", "POW", "SPD", "WT", "JMP", "LS", "MAG", "HOT", "IRN", "WNG", "SHD", "2ND", "STD", "RCH", "ORB", "BUB", "BMB"]
const COLOURS := [
	Color(0.35, 0.85, 0.5), Color(0.95, 0.3, 0.35), Color(0.92, 0.36, 0.26), Color(0.3, 0.75, 0.95), Color(0.55, 0.5, 0.62),
	Color(0.55, 0.9, 0.4), Color(0.85, 0.2, 0.42), Color(0.75, 0.75, 0.82), Color(1.0, 0.55, 0.15), Color(0.5, 0.56, 0.66),
	Color(0.7, 0.88, 1.0), Color(0.4, 0.62, 0.95), Color(0.5, 0.95, 0.85), Color(1.0, 0.82, 0.3), Color(0.82, 0.55, 0.95),
	Color(0.38, 0.22, 0.55), Color(0.6, 0.9, 1.0), Color(0.25, 0.25, 0.3),
]
## Rarity rings: staple, common, rare, prized.
const RARITY := [Color(0.35, 0.85, 0.5), Color(0.96, 0.96, 0.92), Color(0.35, 0.6, 1.0), Color(1.0, 0.8, 0.25)]
const RARITY_NAMES := ["STAPLE", "COMMON", "RARE", "PRIZED"]


static func glyph(kind: int) -> String:
	return GLYPHS[kind] if kind >= 0 and kind < GLYPHS.size() else "?"


static func colour(kind: int) -> Color:
	return COLOURS[kind] if kind >= 0 and kind < COLOURS.size() else Color.GRAY


static func rarity_colour(rarity: int) -> Color:
	return RARITY[rarity] if rarity >= 0 and rarity < RARITY.size() else Color.WHITE


## Draws kind `kind`'s icon of radius `r` at `at` on `canvas` (a `CanvasItem`), ringed by `rarity`, with `count` as a badge when above 1.
static func draw(canvas: CanvasItem, at: Vector2, r: float, kind: int, rarity: int, count := 1, alpha := 1.0) -> void:
	canvas.draw_circle(at, r + 3.0, Color(Art.INK, alpha))
	canvas.draw_circle(at, r + 1.0, Color(rarity_colour(rarity), alpha))
	canvas.draw_circle(at, r - 2.0, Color(colour(kind), alpha))
	canvas.draw_circle(at + Vector2(-r * 0.3, -r * 0.35), r * 0.28, Color(1, 1, 1, 0.3 * alpha))
	var text := glyph(kind)
	var size := int(r * (0.95 if text.length() <= 2 else 0.72))
	var font := D.display_font()
	var tw := font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, size).x
	canvas.draw_string(font, at + Vector2(-tw * 0.5 + 1, size * 0.36 + 1), text, HORIZONTAL_ALIGNMENT_LEFT, -1, size, Color(Art.INK, 0.8 * alpha))
	canvas.draw_string(font, at + Vector2(-tw * 0.5, size * 0.36), text, HORIZONTAL_ALIGNMENT_LEFT, -1, size, Color(1, 1, 1, alpha))
	if count > 1:
		var b := at + Vector2(r * 0.75, r * 0.7)
		canvas.draw_circle(b, r * 0.5, Color(Art.INK, alpha))
		var label := "x%d" % count
		var bs := int(r * 0.62)
		var bw := font.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, bs).x
		canvas.draw_string(font, b + Vector2(-bw * 0.5, bs * 0.36), label, HORIZONTAL_ALIGNMENT_LEFT, -1, bs, Color(Art.SUN, alpha))


## A 3D gem for kind `kind` (a ware on its pedestal, a find on a route): a sphere in its colour inside a soft rarity halo, its glyph on
## a label in front. Returns the node (its origin at the gem's middle).
static func gem(kind: int, rarity: int, radius := 0.32) -> Node3D:
	var root := Node3D.new()
	var mi := MeshInstance3D.new()
	var s := SphereMesh.new()
	s.radius = radius
	s.height = radius * 2.0
	mi.mesh = s
	var m := StandardMaterial3D.new()
	m.albedo_color = colour(kind)
	m.emission_enabled = true
	m.emission = colour(kind) * 0.35
	m.roughness = 0.4
	mi.material_override = m
	mi.name = "Gem"
	root.add_child(mi)
	var halo := MeshInstance3D.new()
	var hs := SphereMesh.new()
	hs.radius = radius * 1.45
	hs.height = radius * 2.9
	halo.mesh = hs
	var hm := StandardMaterial3D.new()
	hm.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	hm.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	var rc := rarity_colour(rarity)
	hm.albedo_color = Color(rc.r, rc.g, rc.b, 0.22 if rarity >= 2 else 0.12)
	halo.material_override = hm
	halo.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	halo.name = "Halo"
	root.add_child(halo)
	var label := Label3D.new()
	label.text = glyph(kind)
	label.font_size = 48
	label.pixel_size = 0.006
	label.outline_size = 10
	label.position = Vector3(0, 0, radius + 0.05)
	label.name = "Glyph"
	root.add_child(label)
	return root
