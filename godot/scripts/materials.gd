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
	"wood": {"ramp_softness": 0.13, "sheen": 0.0, "rim_strength": 0.25},
	"grass": {"ramp_softness": 0.18, "sheen": 0.0, "rim_strength": 0.2},
	"bark": {"ramp_softness": 0.2, "sheen": 0.0, "rim_strength": 0.25},
	"foliage": {"ramp_softness": 0.26, "sheen": 0.0, "rim_strength": 0.5},
	"stone": {"ramp_softness": 0.1, "sheen": 0.03, "rim_strength": 0.3},
	"plain": {"ramp_softness": 0.16, "sheen": 0.0, "rim_strength": 0.3},
}

## Which family each stage tile is.
const TILE_FAMILY := {"wood": "wood", "grass": "grass", "sand": "grass", "roof": "stone", "steel": "plain", "bark": "bark",
	"foliage": "foliage", "stone": "stone"}

static var _cache := {}


static func _make(family: String, colour: Color, tile_name: String, tile: Vector2, world_tile: float, outline: bool) -> ShaderMaterial:
	var key := "%s|%s|%s|%s|%s|%s" % [family, colour.to_html(), tile_name, tile, world_tile, outline]
	if _cache.has(key):
		return _cache[key]
	var tex: Texture2D = null
	if tile_name != "":
		tex = SvgArt.texture("res://art/stage/%s.svg" % tile_name, {"#ff00ff": colour})
	var base := FighterView.toon(Color.WHITE if tex != null else colour, outline, tex, tile, world_tile)
	var m := base.duplicate() as ShaderMaterial
	for p in FAMILIES[family]:
		m.set_shader_parameter(p, FAMILIES[family][p])
	_cache[key] = m
	return m


## A stage surface laid in world space: the tile `tile_name` (grass, wood, sand, roof, steel...) in colour `c`, one repeat every `size`
## world units.
static func world(tile_name: String, c: Color, size: float, outline := true) -> ShaderMaterial:
	return _make(TILE_FAMILY.get(tile_name, "plain"), c, tile_name, Vector2.ONE, size, outline)


## Planks or carved wood on a prop (a barrel, a crate, a post, a sign), the grain repeated `tile` times across its UVs.
static func wood(c: Color, tile := Vector2.ONE, outline := true) -> ShaderMaterial:
	return _make("wood", c, "wood", tile, 0.0, outline)


## Bark: vertical grooves and a few dark knots, repeated `tile` times round and up a trunk.
static func bark(c: Color, tile := Vector2(4, 3), outline := true) -> ShaderMaterial:
	return _make("bark", c, "bark", tile, 0.0, outline)


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
