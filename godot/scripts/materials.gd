extends RefCounted
## The material library: one cel material per surface family, so wood, grass, bark, foliage and stone read as different stuff while
## sharing one palette and one lighting model (shaders/toon.gdshader). A family is the cel material with a hand-drawn tile from
## godot/art/stage/ (drawn on the placeholder colour #ff00ff, recoloured to the surface's own colour, so one tile serves every hue) and
## its own response to the light (`FAMILIES`): wood and grass matte with a firm step into shadow, foliage softer and rounder with a
## stronger rim (it reads as a puffy mass), stone crisp with a faint sheen, bark soft and matte.
##
## Use: `Materials.<family>(colour, ...)`. Stage surfaces are laid in world space (`world`: one repeat every `size` units, so a surface of
## any size tiles evenly); props and trees by their own UVs (`tile` repeats across the mesh). Materials are cached per family, colour and
## tiling, so a stage uses a handful and `StageArt.merge_static` can still join everything drawn in one. Each is its own copy of the cel
## material (the family's settings must not reach other users of a shared one); the ink outline pass is shared.
##
## What belongs where: the shapes (bevels, curvature, roots) are modelled in code in stage_art.gd (or in Blender for imported props, whose
## plain materials `toonify.gd` turns into cel ones); the colours come from the stage theme (`StageArt.THEMES`); the surface character
## (the tiles and these settings) lives here.

const FighterView := preload("res://scripts/fighter_view.gd")
const SvgArt := preload("res://scripts/svg_art.gd")

## How each family takes the light: ramp softness (how gradual the step into shadow is), sheen (a small soft highlight) and rim strength.
const FAMILIES := {
	"wood": {"ramp_softness": 0.13, "sheen": 0.0, "rim_strength": 0.25, "world_vary": 0.13},
	"grass": {"ramp_softness": 0.18, "sheen": 0.0, "rim_strength": 0.2},
	"bark": {"ramp_softness": 0.2, "sheen": 0.0, "rim_strength": 0.25},
	"foliage": {"ramp_softness": 0.26, "sheen": 0.0, "rim_strength": 0.35},
	"stone": {"ramp_softness": 0.1, "sheen": 0.03, "rim_strength": 0.3},
	"plain": {"ramp_softness": 0.16, "sheen": 0.0, "rim_strength": 0.3},
	# (Steel: a crisp step and the metal response's highlights, so it reads apart from wood and stone, 2026-10-17.)
	"metal": {"ramp_softness": 0.08, "sheen": 0.0, "rim_strength": 0.5, "metal": 0.6},
}

## Which family each stage tile is.
const TILE_FAMILY := {"paving": "stone", "wood": "wood", "grass": "grass", "field": "grass", "asphalt": "stone", "sand": "grass", "roof": "stone", "steel": "metal", "bark": "bark",
	"foliage": "foliage", "stone": "stone"}

## The painted detail laid over each world-laid stage tile (shaders/toon_common.gdshaderinc `detail_*`): [detail tile, amount, world units
## per fine repeat]. Tiles not listed have none.
const TILE_DETAIL := {"paving": ["concrete", 0.3, 3.0], "roof": ["concrete", 0.5, 3.0], "asphalt": ["concrete", 0.35, 4.0], "steel": ["concrete", 0.3, 2.5], "grass": ["soil", 0.35, 2.5],
	"sand": ["soil", 0.3, 3.0], "stone": ["stone", 0.4, 3.5], "wood": ["stone", 0.18, 4.0]}

static var _cache := {}


## The painted tile `name` (art/stage/png/<name>.png, made from a Material Maker export by tools/art/mm_tile.gd), or null when there is
## none and the SVG tile serves.
static func _png_tile(name: String) -> Texture2D:
	var path := "res://art/stage/png/%s.png" % name
	return load(path) as Texture2D if ResourceLoader.exists(path) else null


## Sets a world-laid material's painted detail: the tile `detail` (concrete, soil, stone), `amount`, `scale` world units per fine repeat.
static func set_detail(m: ShaderMaterial, detail: String, amount: float, scale: float) -> void:
	var tex := _png_tile("detail_" + detail)
	if tex == null:
		tex = SvgArt.texture("res://art/stage/detail_%s.svg" % detail)
	if tex == null:
		return
	m.set_shader_parameter("detail_tex", tex)
	m.set_shader_parameter("detail_amount", amount)
	m.set_shader_parameter("detail_scale", scale)


static func _make(family: String, colour: Color, tile_name: String, tile: Vector2, world_tile: float, outline: bool) -> ShaderMaterial:
	var key := "%s|%s|%s|%s|%s|%s" % [family, colour.to_html(), tile_name, tile, world_tile, outline]
	if _cache.has(key):
		return _cache[key]
	var tex: Texture2D = null
	# A painted value tile (art/stage/png/, from tools/art/mm_tile.gd) is greyscale and takes the surface's colour from the material;
	# an SVG tile is recoloured itself.
	var painted := _png_tile(tile_name) if tile_name != "" else null
	if painted != null:
		tex = painted
	elif tile_name != "":
		tex = SvgArt.texture("res://art/stage/%s.svg" % tile_name, {"#ff00ff": colour})
	var base := FighterView.toon(colour if painted != null or tex == null else Color.WHITE, outline, tex, tile, world_tile)
	var m := base.duplicate() as ShaderMaterial
	for p in FAMILIES[family]:
		m.set_shader_parameter(p, FAMILIES[family][p])
	if world_tile > 0.0 and TILE_DETAIL.has(tile_name):
		var d: Array = TILE_DETAIL[tile_name]
		set_detail(m, d[0], d[1], d[2])
	_cache[key] = m
	return m


## A copy of cel material `m` (one with its own texture, drawn in code) taking family `family`'s response to the light, so a prop with a
## painted surface of its own (a barrel's staves, a bale's straw) still reads as wood or straw like the rest of the stage. Cached.
static func as_family(m: ShaderMaterial, family: String) -> ShaderMaterial:
	var key := "as|%d|%s" % [m.get_instance_id(), family]
	if _cache.has(key):
		return _cache[key]
	var c := m.duplicate() as ShaderMaterial
	for p in FAMILIES[family]:
		if p != "world_vary":
			c.set_shader_parameter(p, FAMILIES[family][p])
	_cache[key] = c
	return c


## A stage surface laid in world space: the tile `tile_name` (grass, wood, sand, roof, steel...) in colour `c`, one repeat every `size`
## world units.
static func world(tile_name: String, c: Color, size: float, outline := true) -> ShaderMaterial:
	return _make(TILE_FAMILY.get(tile_name, "plain"), c, tile_name, Vector2.ONE, size, outline)


## Planks or carved wood on a prop (a barrel, a crate, a post, a sign), the grain repeated `tile` times across its UVs.
static func wood(c: Color, tile := Vector2.ONE, outline := true) -> ShaderMaterial:
	return _make("wood", c, "wood", tile, 0.0, outline)


## Whitewash or plaster on a wall (a cottage, a windmill's tower): soft patches, a few hairline cracks and chips, repeated `tile` times.
static func plaster(c: Color, tile := Vector2(2, 2), outline := true) -> ShaderMaterial:
	return _make("plain", c, "plaster", tile, 0.0, outline)


## A roof of rounded shingles in overlapping rows, a dark gap and a lit edge on each, repeated `tile` times.
static func shingles(c: Color, tile := Vector2(3, 2), outline := true) -> ShaderMaterial:
	return _make("wood", c, "shingles", tile, 0.0, outline)


## Bark: vertical grooves and a few dark knots, repeated `tile` times round and up a trunk.
static func bark(c: Color, tile := Vector2(4, 3), outline := true) -> ShaderMaterial:
	return _make("bark", c, "bark", tile, 0.0, outline)


## Bark in soft plates for the small trees (art/stage/png/bark_plates.png from tools/art/noise_tile.gd; the owner, 2026-10-15: the bark
## tile's sharp streaks on the small trees read as nothing real). The great trunks keep `bark`, whose bold furrows read at their size.
static func bark_plates(c: Color, tile := Vector2(2, 1), outline := true) -> ShaderMaterial:
	return _make("bark", c, "bark_plates" if _png_tile("bark_plates") != null else "bark", tile, 0.0, outline)


## A palm trunk: soft horizontal ring scars (the painted tile only; with none, the trunk is plain), repeated `tile` times.
static func palm(c: Color, tile := Vector2(2, 1), outline := true) -> ShaderMaterial:
	return _make("bark", c, "palm" if _png_tile("palm") != null else "", tile, 0.0, outline)


## Leaves: soft clumps with a shaded underside, for crowns and bushes.
static func foliage(c: Color, tile := Vector2(3, 2), outline := true) -> ShaderMaterial:
	return _make("foliage", c, "foliage", tile, 0.0, outline)


## Stone: faint facets, a hairline crack or two and speckles.
static func stone(c: Color, tile := Vector2(2, 1), outline := true) -> ShaderMaterial:
	return _make("stone", c, "stone", tile, 0.0, outline)


## Four coordinated greens from one: [deep shade, body, light, sunlit tips], each a step along the hue toward warmer and lighter, so
## the masses of a crown or bush read as one plant in light and shade rather than random greens.
static func greens(base: Color) -> Array:
	return [
		Color.from_hsv(base.h + 0.02, minf(1.0, base.s * 1.08), base.v * 0.82),
		base,
		Color.from_hsv(base.h - 0.03, base.s * 0.92, minf(1.0, base.v * 1.14)),
		Color.from_hsv(base.h - 0.06, base.s * 0.8, minf(1.0, base.v * 1.26)),
	]
