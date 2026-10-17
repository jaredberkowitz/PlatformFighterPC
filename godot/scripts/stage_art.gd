extends RefCounted
## Stage art: the backdrop behind a stage and the look of its blocks and platforms, by theme. Presentation only: everything here is
## built from the stage's geometry (which the simulation owns) and its look (`StageLook`: a backdrop name and sky colours, which the
## stage editor changes). Nothing here ever reaches the simulation.
##
## Themes ("backdrops"): meadow (rolling hills, a windmill, a fence and round trees), grove (a floating island under a giant old tree),
## sunset (a canyon at dusk: striped sandstone, mesas, a rock arch and cacti), night (stars and a big moon), ocean (a sandy island on the
## sea with palms, distant islands and a lighthouse) and city (rooftops at night in front of a lit skyline). All original art in the
## project's soft cel style.

const SvgArt := preload("res://scripts/svg_art.gd")
const FxMaterial := preload("res://scripts/fx_material.gd")
const FighterView := preload("res://scripts/fighter_view.gd")
const Materials := preload("res://scripts/materials.gd")
const SUN_SHADER := preload("res://shaders/sun.gdshader")
const CLOUD_SHADER := preload("res://shaders/cloud.gdshader")
const RIDGE_SHADER := preload("res://shaders/painted_ridge.gdshader")
const FOLIAGE_SHADER := preload("res://shaders/toon_foliage.gdshader")
const VisualSettings := preload("res://scripts/visual_settings.gd")
const Toonify := preload("res://scripts/toonify.gd")
const Particles := preload("res://scripts/particles.gd")

## The look of one theme: sky colours (top, horizon), block colours and pattern (`diamonds`, `strata`, `cobble` or `facade`), what tops a
## block (`grass`, `sand` or `roof`), and the platforms (`wood` planks or steel `girder`s) and their colours.
const THEMES := {
	"meadow": {
		"sky_top": Color(0.1, 0.36, 0.9), "sky_mid": Color(0.3, 0.62, 1.0), "sky_horizon": Color(0.78, 0.93, 1.0),
		"sky_glow": [Color(1.0, 0.97, 0.86), 0.25], "sky_wisps": [Color(1.0, 1.0, 1.0), 0.5],
		"soil": Color(0.76, 0.56, 0.36), "soil_dark": Color(0.52, 0.37, 0.25), "grass": Color(0.46, 0.82, 0.16),
		"grass_light": Color(0.66, 0.92, 0.34), "wood": Color(0.86, 0.62, 0.36), "wood_dark": Color(0.62, 0.42, 0.24),
		"pattern": "earth", "cap": "grass", "platform": "wood", "grass_vary": 0.17, "moss": Color(0.4, 0.66, 0.14),
	},
	"grove": {
		"sky_top": Color(0.28, 0.5, 0.98), "sky_mid": Color(0.52, 0.66, 1.0), "sky_horizon": Color(0.98, 0.76, 0.86),
		"sky_glow": [Color(1.0, 0.72, 0.82), 0.55], "sky_wisps": [Color(1.0, 0.9, 0.96), 0.45],
		"soil": Color(0.86, 0.66, 0.38), "soil_dark": Color(0.72, 0.5, 0.27), "grass": Color(0.32, 0.72, 0.26),
		"grass_light": Color(0.6, 0.9, 0.34), "wood": Color(0.93, 0.74, 0.5), "wood_dark": Color(0.7, 0.48, 0.28),
		"pattern": "roots", "cap": "grass", "platform": "wood", "grass_vary": 0.16,
	},
	"sunset": {
		"sky_top": Color(0.36, 0.28, 0.6), "sky_mid": Color(0.84, 0.46, 0.62), "sky_horizon": Color(1.0, 0.64, 0.4),
		"sky_glow": [Color(1.0, 0.56, 0.26), 0.7], "sky_wisps": [Color(1.0, 0.7, 0.62), 0.5],
		"soil": Color(0.86, 0.5, 0.32), "soil_dark": Color(0.68, 0.34, 0.24), "grass": Color(0.93, 0.74, 0.46),
		# (Sun-bleached, in the other stages' range of wood: art review 6 round 8, the canyon's dark maroon planks read as black bars
		# against its bright sky, the one stage whose platforms broke the family.)
		"grass_light": Color(0.98, 0.86, 0.6), "wood": Color(0.82, 0.57, 0.42), "wood_dark": Color(0.58, 0.37, 0.29),
		"pattern": "strata", "cap": "sand", "platform": "wood", "grass_vary": 0.18,
	},
	"night": {
		"sky_top": Color(0.05, 0.07, 0.2), "sky_mid": Color(0.12, 0.13, 0.36), "sky_horizon": Color(0.2, 0.25, 0.48),
		"sky_glow": [Color(0.46, 0.42, 0.86), 0.35],
		"soil": Color(0.36, 0.4, 0.56), "soil_dark": Color(0.27, 0.3, 0.44), "grass": Color(0.38, 0.62, 0.6),
		"grass_light": Color(0.52, 0.78, 0.74), "wood": Color(0.62, 0.66, 0.78), "wood_dark": Color(0.42, 0.45, 0.58),
		"pattern": "diamonds", "cap": "grass", "platform": "wood",
	},
	"ocean": {
		"sky_top": Color(0.06, 0.34, 0.9), "sky_mid": Color(0.2, 0.58, 1.0), "sky_horizon": Color(0.5, 0.8, 1.0),
		"sky_glow": [Color(1.0, 0.96, 0.86), 0.15], "sky_wisps": [Color(1.0, 1.0, 1.0), 0.45],
		"soil": Color(0.82, 0.66, 0.5), "soil_dark": Color(0.6, 0.48, 0.46), "grass": Color(0.96, 0.8, 0.5),
		"grass_light": Color(1.0, 0.95, 0.78), "wood": Color(0.8, 0.62, 0.42), "wood_dark": Color(0.58, 0.42, 0.28),
		"pattern": "seawall", "cap": "sand", "platform": "wood", "grass_vary": 0.14,
	},
	"city": {
		"sky_top": Color(0.08, 0.06, 0.22), "sky_mid": Color(0.22, 0.12, 0.36), "sky_horizon": Color(0.42, 0.22, 0.46),
		"sky_glow": [Color(0.95, 0.38, 0.6), 0.45],
		"soil": Color(0.42, 0.44, 0.54), "soil_dark": Color(0.3, 0.32, 0.4), "grass": Color(0.56, 0.58, 0.66),
		"grass_light": Color(0.78, 0.8, 0.88), "wood": Color(0.84, 0.48, 0.41), "wood_dark": Color(0.5, 0.26, 0.27),
		"pattern": "facade", "cap": "roof", "platform": "girder",
	},
}


static func theme(look: Dictionary) -> Dictionary:
	var name: String = look.get("backdrop", "meadow")
	var t: Dictionary = THEMES.get(name, THEMES.meadow).duplicate()
	t.name = name if THEMES.has(name) else "meadow"
	# The stage editor can override the sky colours.
	var top: String = look.get("sky_top", "")
	var bottom: String = look.get("sky_bottom", "")
	if top.length() == 6:
		t.sky_top = Color.html(top)
	if bottom.length() == 6:
		t.sky_horizon = Color.html(bottom)
	return t


## The sky for a theme: a gradient from the top colour to the horizon (the camera never sees below the horizon much).
## The theme's sky (shaders/sky.gdshader): its top, an optional middle tint (`sky_mid`) and horizon, with an optional warm glow low over
## the horizon behind the stage (`sky_glow`: colour, strength) and painted wisps (`sky_wisps`: colour, amount).
static func sky(t: Dictionary) -> Sky:
	var mat := ShaderMaterial.new()
	mat.shader = SKY_SHADER
	var top: Color = t.sky_top
	var horizon: Color = t.sky_horizon
	mat.set_shader_parameter("top_color", top)
	mat.set_shader_parameter("mid_color", t.get("sky_mid", top.lerp(horizon, 0.5)))
	mat.set_shader_parameter("horizon_color", horizon)
	mat.set_shader_parameter("ground_color", horizon.darkened(0.2))
	if t.has("sky_glow"):
		mat.set_shader_parameter("glow_color", t.sky_glow[0])
		mat.set_shader_parameter("glow_strength", t.sky_glow[1])
	if t.has("sky_wisps"):
		mat.set_shader_parameter("wisp_color", t.sky_wisps[0])
		mat.set_shader_parameter("wisp_amount", t.sky_wisps[1])
	var s := Sky.new()
	s.sky_material = mat
	return s


const SKY_SHADER := preload("res://shaders/sky.gdshader")


# ---- Materials --------------------------------------------------------------------------------------------------------------------

## A flat colour, unlit; one shared material per colour (so `merge_static` can join everything drawn in it). Not to be changed: duplicate it.
static func _unshaded(c: Color) -> StandardMaterial3D:
	var key := c.to_html()
	if _flat_cache.has(key):
		return _flat_cache[key]
	var m := StandardMaterial3D.new()
	m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	m.albedo_color = c
	_flat_cache[key] = m
	return m


static var _flat_cache := {}


## A stage surface in colour `c` with one of the painted tiles in godot/art/stage/ laid over it in world space (`size` world units a repeat).
static func _surface(c: Color, tile: String, size: float, outline := true) -> ShaderMaterial:
	return Materials.world(tile, c, size, outline)


static func _soft(c: Color) -> ShaderMaterial:
	return FighterView.toon(c, false)


## The pattern on a block's front, drawn as vector art (an SVG made here and rasterised once, smooth and anti-aliased, in the hand-drawn
## style of the rest): quilted diamonds, wavy sandstone strata, rounded cobbles, or a building facade with windows (some lit). Each
## tiles seamlessly.
## The city facade tile's window bays across its 512px: [left edge, width] (a width of 0 is a blank pier with a blade sign): a pair, a
## pier, a wide picture window, a narrow one.
## The beach's clouds (`_clouds` look): a little less exposure and a deeper shade, so the lit tops, the soft middle and the cool belly
## all read under the noon sun (flat white shapes, the pain-point review).
const BEACH_CLOUDS := {"exposure": 1.3, "shade_depth": 1.0}
const FACADE_BAYS := [[22, 52], [96, 52], [190, 0], [232, 104], [372, 52], [448, 36]]


static func _soil_material(t: Dictionary) -> ShaderMaterial:
	var a: Color = t.soil
	var b: Color = t.soil_dark
	var pattern: String = t.get("pattern", "diamonds")
	var key := "%s%s%s%s" % [pattern, a.to_html(), b.to_html(), str(t.get("moss", ""))]
	if _soil_cache.has(key):
		return _soil_cache[key]
	var rng := RandomNumberGenerator.new()
	rng.seed = 9
	var ink := b.darkened(0.35).to_html(false)
	var hi := a.lightened(0.18).to_html(false)
	var A := a.to_html(false)
	var B := b.to_html(false)
	var body := ""
	match pattern:
		"strata":
			# Layered sandstone, as a canyon wall is painted: beds of uneven thickness in close warm shades (a pale band now and then), each
			# bed's top edge a gently waving ledge caught by the light with a band of shade under its lip, fine cross-bedding drawn
			# slantwise inside the thicker beds, vertical joints broken bed by bed, dark streaks of desert varnish running down from a few
			# ledges, and a scatter of pebbles. Every wave has a whole number of periods across the tile, and anything crossing an edge is
			# drawn again across it, so it tiles.
			var pale := a.lerp(Color(1.0, 0.9, 0.74), 0.4)
			var beds := [[0.0, 38.0, a], [38.0, 22.0, b], [60.0, 46.0, a.lerp(b, 0.35)], [106.0, 16.0, pale], [122.0, 40.0, a],
				[162.0, 30.0, b.lerp(a, 0.3)], [192.0, 20.0, a.lerp(b, 0.6)], [212.0, 44.0, a.lerp(pale, 0.3)]]
			var wave := func(x: float, k: int) -> float:
				return sin(x / 256.0 * TAU * float(1 + k % 2) + float(k) * 1.7) * 4.0 + sin(x / 256.0 * TAU * 3.0 + float(k)) * 1.5
			for k in beds.size():
				var bed: Array = beds[k]
				var y0: float = bed[0]
				var h: float = bed[1]
				var col: Color = bed[2]
				var top := "M0,%.1f " % (y0 + wave.call(0.0, k))
				for x in range(16, 257, 16):
					top += "L%d,%.1f " % [x, y0 + wave.call(float(x), k)]
				var shape := top + "L256,%.1f L0,%.1f Z" % [y0 + h + 10.0, y0 + h + 10.0]
				body += '<path d="%s" fill="#%s"/>' % [shape, col.to_html(false)]
				# Shade under the lip of the bed above (a soft band at this bed's top), then the lit ledge line.
				body += '<path d="%s" fill="none" stroke="#%s" stroke-width="9" stroke-opacity="0.22" transform="translate(0,5)"/>' % [top, col.darkened(0.45).to_html(false)]
				body += '<path d="%s" fill="none" stroke="#%s" stroke-width="2.6" stroke-opacity="0.5"/>' % [top, ink]
				body += '<path d="%s" fill="none" stroke="#%s" stroke-width="2.2" stroke-opacity="0.55" transform="translate(0,-2.5)"/>' % [top, col.lightened(0.28).to_html(false)]
				# Cross-bedding in the thick beds: short slanting lines in one direction per bed.
				if h >= 30.0:
					var lean := 1.0 if k % 2 == 0 else -1.0
					for j in 6:
						var cx := fmod(float(j) * 47.0 + float(k) * 29.0, 256.0)
						for ox in [-256.0, 0.0, 256.0]:
							var x0: float = cx + ox
							body += '<path d="M%.1f,%.1f q%.1f,%.1f %.1f,%.1f" fill="none" stroke="#%s" stroke-width="1.6" stroke-opacity="0.28"/>' % [x0, y0 + h * 0.3, lean * 14.0, h * 0.25, lean * 30.0, h * 0.55, col.darkened(0.3).to_html(false)]
				# A joint or two through the bed, offset from the beds above and below.
				for j in 2:
					var jx := fmod(float(k) * 83.0 + float(j) * 131.0, 256.0)
					body += '<path d="M%.1f,%.1f l%.1f,%.1f l%.1f,%.1f" fill="none" stroke="#%s" stroke-width="2" stroke-opacity="0.35" stroke-linecap="round"/>' % [jx, y0 + 4.0, sin(k + j) * 3.0, h * 0.5, sin(k * 2 + j) * 3.0, h * 0.45, ink]
			# Desert varnish: dark streaks running down from a few ledges, wider at the top.
			for j in 5:
				var vx := fmod(float(j) * 61.0 + 17.0, 256.0)
				var vy: float = (beds[(j * 3) % beds.size()] as Array)[0]
				body += '<path d="M%.1f,%.1f q-2,%.1f 1,%.1f q2,6 -1,0 Z" fill="#%s" opacity="0.16" stroke="none"/>' % [vx - 4.0, vy + 3.0, 22.0 + j * 6.0, 40.0 + j * 9.0, b.darkened(0.5).to_html(false)]
				body += '<rect x="%.1f" y="%.1f" width="%.1f" height="%.1f" rx="3" fill="#%s" opacity="0.13"/>' % [vx - 3.0, vy + 3.0, 6.0 + j % 3 * 2.0, 34.0 + j * 10.0, b.darkened(0.5).to_html(false)]
			# Pebbles set in the rock.
			for j in 6:
				var px2 := fmod(float(j) * 97.0 + 11.0, 256.0)
				var py := fmod(float(j) * 59.0 + 23.0, 256.0)
				body += '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#%s" stroke="#%s" stroke-width="1.4" stroke-opacity="0.45"/>' % [px2, py, 4.0 + j % 3, 3.0 + j % 2, a.lerp(pale, 0.5).to_html(false), ink]
		"masonry":
			# Big rounded stone blocks laid in rough courses (as a cliff of dressed stone in the reference's style): each a soft-cornered
			# slab a little irregular, its top edge lit, its lower edge and the gaps between in shade, a few cracks and chips. Large, so a
			# wall is a handful of readable shapes, not a busy pattern. Two courses a tile; their joints step, so it tiles.
			body += '<rect width="256" height="256" fill="#%s"/>' % b.darkened(0.18).to_html(false)
			var course := 0
			for y0 in [0.0, 128.0]:
				var x0 := -40.0 if course % 2 == 0 else 24.0
				var x := x0
				while x < 256.0 + 40.0:
					var w := rng.randf_range(78.0, 118.0)
					var h := rng.randf_range(112.0, 122.0)
					var shade := a.lerp(b, rng.randf_range(0.0, 0.45))
					for ox in [-256.0, 0.0, 256.0]:
						var bx: float = x + ox + 4.0
						var by: float = y0 + 4.0
						var r := rng.randf_range(16.0, 26.0)
						body += '<rect x="%.1f" y="%.1f" width="%.1f" height="%.1f" rx="%.1f" fill="#%s" stroke="#%s" stroke-width="3" stroke-opacity="0.45"/>' % [bx, by, w - 8.0, h - 8.0, r, shade.to_html(false), ink]
						# Lit top, shaded foot.
						body += '<rect x="%.1f" y="%.1f" width="%.1f" height="%.1f" rx="%.1f" fill="#%s" opacity="0.28"/>' % [bx + 8.0, by + 6.0, w - 24.0, (h - 8.0) * 0.28, r * 0.6, shade.lightened(0.22).to_html(false)]
						body += '<rect x="%.1f" y="%.1f" width="%.1f" height="%.1f" rx="%.1f" fill="#%s" opacity="0.22"/>' % [bx + 6.0, by + (h - 8.0) * 0.7, w - 20.0, (h - 8.0) * 0.26, r * 0.6, shade.darkened(0.3).to_html(false)]
						if rng.randf() < 0.45:
							var cx: float = bx + rng.randf_range(16.0, w - 30.0)
							body += '<path d="M%.1f,%.1f l%.1f,%.1f l%.1f,%.1f" fill="none" stroke="#%s" stroke-width="2.4" stroke-linecap="round" opacity="0.4"/>' % [cx, by + 14.0, rng.randf_range(-8, 8), rng.randf_range(16, 30), rng.randf_range(-10, 10), rng.randf_range(10, 22), ink]
					x += w
				course += 1
		"seawall":
			# A sea wall of big dressed stones laid irregularly (the beach's own; random bond, each stone its own width and height, so no
			# course runs the length of the wall), deep mortar shade, a lit top on each stone, a crack now and then. Drawn at 512 px, laid
			# every 14. (A tide line was tried in the tile: it repeats with it and could fall mid-wall; the wall's own cool fall toward its
			# foot does that job.)
			var S := 512.0
			body += '<rect width="512" height="512" fill="#%s"/>' % b.darkened(0.28).to_html(false)
			# (Smaller stones than at first, chiselled rather than pillowed: corners only just softened, a lit bevel along the top and left
			# of each and a shaded one along the bottom and right, darker mortar, and fine pits in the stone; the big soft-cornered blocks
			# with a pale inner highlight read as cushions at play size.)
			# (Warm sandstone, art review 6: a cool grey wall was the one cold thing on a warm sand island beside the lighthouse's brown rocks.)
			var stone_grey := Color(0.86, 0.74, 0.6)
			var y := 0.0
			while y < S - 1.0:
				var rh := rng.randf_range(36.0, 74.0)
				if y + rh > S - 30.0:
					rh = S - y
				var widths: Array = []
				var total := 0.0
				while total < S - 40.0:
					var ww := rng.randf_range(40.0, 150.0)
					widths.append(ww)
					total += ww
				var x := -rng.randf_range(0.0, 50.0)
				for wi in widths.size():
					var w: float = float(widths[wi]) * S / total
					# (A wide spread of stone, art review 6 round 2: within a narrow range of shades and sizes the wall read as a regular
					# brick grid at play size, the most placeholder-looking surface on any stage. Now each stone its own shade, a few pale or
					# deep ones, and a wider range of sizes.)
					var shade := a.lerp(b, rng.randf_range(0.0, 0.7)).lerp(stone_grey, 0.2)
					shade = shade.lightened(rng.randf_range(0.02, 0.14)) if rng.randf() < 0.6 else shade.darkened(rng.randf_range(0.0, 0.1))
					var rad := rng.randf_range(3.0, 7.0)
					var lit := shade.lightened(0.2).to_html(false)
					var dark := shade.darkened(0.25).to_html(false)
					var pits := []
					for j in 4:
						pits.append([rng.randf_range(8.0, w - 16.0), rng.randf_range(8.0, rh - 16.0), rng.randf_range(1.4, 3.2)])
					var crack := rng.randf() < 0.25
					var cxr := rng.randf_range(0.2, 0.7)
					var cv := [rng.randf_range(-6, 6), rng.randf_range(10, 18), rng.randf_range(-6, 6), rng.randf_range(8, 14)]
					for ox in [-S, 0.0, S]:
						var bx: float = x + ox + 3.0
						var by: float = y + 3.0
						var bw: float = w - 6.0
						var bh: float = rh - 6.0
						body += '<rect x="%.1f" y="%.1f" width="%.1f" height="%.1f" rx="%.1f" fill="#%s" stroke="#%s" stroke-width="2.5" stroke-opacity="0.5"/>' % [bx, by, bw, bh, rad, shade.to_html(false), ink]
						# The bevels: lit top and left, shaded bottom and right.
						body += '<path d="M%.1f,%.1f L%.1f,%.1f L%.1f,%.1f L%.1f,%.1f L%.1f,%.1f L%.1f,%.1f Z" fill="#%s" opacity="0.55"/>' % [bx + 2, by + bh - 2, bx + 2, by + 2, bx + bw - 2, by + 2, bx + bw - 8, by + 8, bx + 8, by + 8, bx + 8, by + bh - 8, lit]
						body += '<path d="M%.1f,%.1f L%.1f,%.1f L%.1f,%.1f L%.1f,%.1f L%.1f,%.1f L%.1f,%.1f Z" fill="#%s" opacity="0.5"/>' % [bx + bw - 2, by + 2, bx + bw - 2, by + bh - 2, bx + 2, by + bh - 2, bx + 8, by + bh - 8, bx + bw - 8, by + bh - 8, bx + bw - 8, by + 8, dark]
						for pt in pits:
							body += '<circle cx="%.1f" cy="%.1f" r="%.1f" fill="#%s" opacity="0.35"/>' % [bx + float(pt[0]), by + float(pt[1]), float(pt[2]), dark]
						if crack:
							body += '<path d="M%.1f,%.1f l%.1f,%.1f l%.1f,%.1f" fill="none" stroke="#%s" stroke-width="2" stroke-linecap="round" opacity="0.4"/>' % [bx + bw * cxr, by + 6.0, cv[0], cv[1], cv[2], cv[3], ink]
					x += w
				y += rh
		"earth":
			# A bank of farm soil, painted (the meadow's own; the beach keeps the masonry): broad beds of warm earth with soft, gently waving
			# boundaries, stones of very different sizes half buried in it (a few big boulders, more middling ones, many pebbles; each with a
			# lit top, a shade under it and a soft ink), a few fine roots hanging from the turf, and grit. Twice the size of the other
			# patterns and laid every 12 units, so no repeat shows across a stage. Anything crossing an edge is drawn again across it.
			var S := 512.0
			body += '<rect width="512" height="512" fill="#%s"/>' % A
			var bed_cols := [a.lerp(b, 0.22), a.lerp(b, 0.4), a.lerp(b, 0.14), a.lerp(b, 0.5)]
			# Each bed runs from its waving top down under the next bed's (which is drawn over it), the last one wrapped round to the top of
			# the tile and drawn first, so the beds meet only along waving seams. Every wave has whole periods across the tile.
			var bed_top := func(k: int, x: float) -> float:
				return 70.0 + float(k) * 118.0 + sin(x / S * TAU * float(1 + k % 2) + float(k) * 2.1) * 9.0 + sin(x / S * TAU * 3.0 + float(k)) * 4.0
			for kb in [[3, -S], [0, 0.0], [1, 0.0], [2, 0.0], [3, 0.0]]:
				var k: int = kb[0]
				var shift: float = kb[1]
				var d := ""
				for x in range(0, 513, 32):
					d += ("M" if x == 0 else " L") + "%d,%.1f" % [x, bed_top.call(k, float(x)) + shift]
				var bottom: float = 70.0 + float(k) * 118.0 + 190.0 + shift
				body += '<path d="%s L512,%.1f L0,%.1f Z" fill="#%s"/>' % [d, bottom, bottom, (bed_cols[k] as Color).to_html(false)]
				# A soft darker seam where each bed meets the one above.
				body += '<path d="%s" fill="none" stroke="#%s" stroke-width="6" stroke-opacity="0.18"/>' % [d, b.darkened(0.3).to_html(false)]
			var stone_base := Color(0.8, 0.74, 0.66).lerp(a, 0.5)
			var stones := []
			# (Fewer, art review 6: the beds are the big forms, as on the sky islands' banded keels; stones only support them. A scatter of
			# two dozen read as a polka-dot wall.)
			for j in 1:
				stones.append([rng.randf_range(0.0, S), rng.randf_range(60.0, S - 40.0), rng.randf_range(34.0, 46.0)])
			for j in 4:
				stones.append([rng.randf_range(0.0, S), rng.randf_range(0.0, S), rng.randf_range(20.0, 32.0)])
			for j in 8:
				stones.append([rng.randf_range(0.0, S), rng.randf_range(0.0, S), rng.randf_range(6.0, 12.0)])
			for st in stones:
				var rx: float = st[2]
				var ry := rx * rng.randf_range(0.62, 0.8)
				var tone := stone_base.lerp(b, rng.randf_range(0.0, 0.3))
				var tilt := rng.randf_range(-12.0, 12.0)
				for ox in [-S, 0.0, S]:
					for oy in [-S, 0.0, S]:
						var cx: float = st[0] + ox
						var cy: float = st[1] + oy
						var rot := 'transform="rotate(%.1f %.1f %.1f)"' % [tilt, cx, cy]
						# Shade in the soil under the stone, the stone, its lit top.
						body += '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#%s" opacity="0.25" %s/>' % [cx + 2.0, cy + ry * 0.35, rx * 1.05, ry * 0.9, b.darkened(0.35).to_html(false), rot]
						if rx > 32.0:
							# A boulder: a lumpy, flattened stone a step off the soil, its crack running in from the rim (art review 6 round 17:
							# a perfect pale ellipse with a crack from its middle and a pebble under it read as a clock face below the fighters).
							var big_tone := tone.lerp(b, 0.3)
							var pts := ""
							for q in 24:
								var ang := TAU * float(q) / 24.0
								var bump := 1.0 + 0.09 * sin(ang * 2.0 + rx) + 0.05 * cos(ang * 3.0 + 1.3)
								pts += "%.1f,%.1f " % [cx + cos(ang) * rx * bump, cy + sin(ang) * ry * 0.85 * bump]
							body += '<polygon points="%s" fill="#%s" stroke="#%s" stroke-width="3" stroke-opacity="0.4" stroke-linejoin="round" %s/>' % [pts, big_tone.to_html(false), ink, rot]
							body += '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#%s" opacity="0.4" %s/>' % [cx - rx * 0.1, cy - ry * 0.35, rx * 0.6, ry * 0.3, big_tone.lightened(0.18).to_html(false), rot]
							body += '<path d="M%.1f,%.1f l%.1f,%.1f l%.1f,%.1f" fill="none" stroke="#%s" stroke-width="2.2" stroke-linecap="round" opacity="0.3"/>' % [cx + rx * 0.85, cy - ry * 0.1, -rx * 0.25, ry * 0.15, -rx * 0.18, -ry * 0.2, ink]
						else:
							body += '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#%s" stroke="#%s" stroke-width="%.1f" stroke-opacity="0.4" %s/>' % [cx, cy, rx, ry, tone.to_html(false), ink, clampf(rx * 0.08, 1.5, 3.5), rot]
							body += '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#%s" opacity="0.5" %s/>' % [cx - rx * 0.12, cy - ry * 0.32, rx * 0.66, ry * 0.42, tone.lightened(0.25).to_html(false), rot]
			# Fine roots hanging from the turf at the top of the tile (the wall's top is under the grass lip).
			var root_col := Color(0.6, 0.42, 0.28)
			for j in 4:
				var rx0 := 40.0 + float(j) * 131.0
				var len := 50.0 + float(j % 3) * 30.0
				for ox in [-S, 0.0, S]:
					var d := "M%.1f,-4 q%.1f,%.1f %.1f,%.1f" % [rx0 + ox, 14.0 - float(j % 2) * 28.0, len * 0.5, -6.0 + float(j) * 4.0, len]
					body += '<path d="%s" fill="none" stroke="#%s" stroke-width="4" stroke-linecap="round" stroke-opacity="0.45"/>' % [d, ink]
					body += '<path d="%s" fill="none" stroke="#%s" stroke-width="2.4" stroke-linecap="round"/>' % [d, root_col.to_html(false)]
			for j in 60:
				body += '<circle cx="%.1f" cy="%.1f" r="%.1f" fill="#%s" opacity="0.3"/>' % [rng.randf_range(0.0, S), rng.randf_range(0.0, S), rng.randf_range(1.2, 2.6), b.darkened(0.3).to_html(false)]
		"roots":
			# Packed earth under a great tree, painted: soft layers of warm soil, roots winding through it (thick and thin, each a rounded
			# stroke with a lit upper edge and a dark under edge, tapering as they go), a few rounded stones bedded in it and specks of
			# grit. Anything crossing an edge is drawn again across it, so it tiles.
			body += '<rect width="256" height="256" fill="#%s"/>' % B
			for k in 4:
				var y0 := 18.0 + k * 64.0
				var band := a.lerp(b, [0.15, 0.55, 0.3, 0.7][k])
				body += '<path d="M0,%.1f C64,%.1f 128,%.1f 192,%.1f S256,%.1f 256,%.1f L256,%.1f L0,%.1f Z" fill="#%s" opacity="0.7"/>' % [y0, y0 - 8.0,
					y0 + 8.0, y0 - 4.0, y0 + 2.0, y0, y0 + 40.0, y0 + 40.0, band.to_html(false)]
			var root_col := Color(0.56, 0.37, 0.24)
			var roots := [[20.0, -10.0, 90.0, 120.0, 30.0, 270.0, 16.0], [150.0, -10.0, 230.0, 70.0, 120.0, 270.0, 11.0],
				[-20.0, 120.0, 90.0, 170.0, 200.0, 140.0, 8.0], [200.0, 150.0, 140.0, 220.0, 250.0, 270.0, 9.0], [90.0, 30.0, 60.0, 90.0, 110.0, 150.0, 6.0]]
			# (Art review 6 round 7: each root was one even-width stroke with round ends and a lit stripe down it, and the stones pale ovals
			# with a highlight: at play size, pipes and polka dots. Now each root tapers from its thick end in short segments, darker than
			# before, and the stones are fewer, smaller and only a step off the soil, bedded in it.)
			for r in roots:
				var w: float = r[6]
				var p0 := Vector2(r[0], r[1])
				var p1 := Vector2(r[2], r[3])
				var p2 := Vector2(r[4], r[5])
				var steps := 10
				for ox in [-256.0, 0.0, 256.0]:
					for oy in [-256.0, 0.0, 256.0]:
						var off := Vector2(ox, oy)
						for pass_k in 2:
							for i in steps:
								var t0 := float(i) / steps
								var t1 := float(i + 1) / steps
								var q0 := p0.lerp(p1, t0).lerp(p1.lerp(p2, t0), t0) + off
								var q1 := p0.lerp(p1, t1).lerp(p1.lerp(p2, t1), t1) + off
								var wi := w * (1.0 - 0.75 * t0)
								if pass_k == 0:
									body += '<line x1="%.1f" y1="%.1f" x2="%.1f" y2="%.1f" stroke="#%s" stroke-width="%.1f" stroke-linecap="round" stroke-opacity="0.55"/>' % [q0.x, q0.y, q1.x, q1.y, ink, wi + 3.0]
								else:
									body += '<line x1="%.1f" y1="%.1f" x2="%.1f" y2="%.1f" stroke="#%s" stroke-width="%.1f" stroke-linecap="round"/>' % [q0.x, q0.y, q1.x, q1.y, root_col.darkened(0.12).to_html(false), wi]
			for j in 3:
				var cx := fmod(float(j) * 97.0 + 40.0, 256.0)
				var cy := fmod(float(j) * 151.0 + 60.0, 256.0)
				var rx := 6.0 + float(j % 3) * 4.0
				var stone := a.lerp(b, 0.5).lerp(Color(0.7, 0.66, 0.68), 0.3)
				for ox in [-256.0, 0.0, 256.0]:
					body += '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#%s" stroke="#%s" stroke-width="1.6" stroke-opacity="0.4"/>' % [cx + ox, cy, rx, rx * 0.66, stone.to_html(false), ink]
			for j in 24:
				body += '<circle cx="%.1f" cy="%.1f" r="%.1f" fill="#%s" opacity="0.35"/>' % [fmod(float(j) * 53.0, 256.0), fmod(float(j) * 97.0 + 13.0, 256.0), 1.5 + float(j % 2), b.darkened(0.35).to_html(false)]
		"cobble":
			# Rounded stones over dark mortar, each with a lighter top; stones at an edge are drawn again across it, so it tiles.
			# (Mortar only a step darker than the stones and a soft ink, so the wall is one calm mass under the fighters, not a busy pattern
			# that outshouts them.)
			body += '<rect width="256" height="256" fill="#%s"/>' % b.darkened(0.1).to_html(false)
			for k in 16:
				var cx := rng.randf_range(0.0, 256.0)
				var cy := rng.randf_range(0.0, 256.0)
				var rx := rng.randf_range(30.0, 44.0)
				var ry := rx * rng.randf_range(0.7, 0.9)
				# (Stones close in value to their mortar: the wall reads as one mass, the stones as a texture on it.)
				var shade := a.lerp(b, rng.randf_range(0.3, 0.55))
				for ox in [-256.0, 0.0, 256.0]:
					for oy in [-256.0, 0.0, 256.0]:
						body += '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#%s" stroke="#%s" stroke-width="2.5" stroke-opacity="0.55"/>' % [cx + ox, cy + oy, rx, ry, shade.to_html(false), ink]
						body += '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#%s" opacity="0.3"/>' % [cx + ox - rx * 0.15, cy + oy - ry * 0.3, rx * 0.55, ry * 0.4, shade.lightened(0.1).to_html(false)]
		"facade":
			# (Bays of their own widths, FACADE_BAYS: four equal columns read as one window grid across the stage, art review pain points.)
			# A brick apartment front, four storeys a repeat: brick courses in a soft stagger, a stone ledge along each floor, four tall
			# windows a floor each in a pale frame with a cross bar (a third lit warm, with curtains drawn to the sides; a few with a
			# flower pot on the sill), striped awnings over two, an air conditioner hung under one, and a drainpipe down the seam. (It was
			# a flat grid of narrow windows on panels; a wall a fighter stands over the whole match needs a building's details.)
			body += '<rect width="512" height="512" fill="#%s"/>' % A
			var brick_line := a.darkened(0.18).to_html(false)
			for by in range(0, 512, 16):
				body += '<rect x="0" y="%d" width="512" height="2" fill="#%s" opacity="0.45"/>' % [by, brick_line]
				var off := 0 if (by / 16) % 2 == 0 else 20
				for bx in range(off, 512, 40):
					body += '<rect x="%d" y="%d" width="2" height="16" fill="#%s" opacity="0.35"/>' % [bx, by, brick_line]
			var frame_col := Color(0.92, 0.88, 0.86).lerp(a, 0.25).to_html(false)
			var curtain := ["e98a94", "8fb8e8", "e8c070"]
			var awn := ["e2525a", "3f8f8a"]
			for floor_k in 4:
				var fy := floor_k * 128
				# The ledge at the foot of the floor.
				body += '<rect x="0" y="%d" width="512" height="12" fill="#%s"/>' % [fy + 116, hi]
				body += '<rect x="0" y="%d" width="512" height="3" fill="#%s" opacity="0.6"/>' % [fy + 125, ink]
				for bay in FACADE_BAYS:
					var wx: int = bay[0]
					var w: int = bay[1]
					var wy := fy + 18
					if w == 0:
						# The pier: a vertical blade sign across the middle two floors, else a small vent.
						if floor_k == 1:
							body += '<rect x="%d" y="%d" width="30" height="200" rx="5" fill="#%s" stroke="#%s" stroke-width="3"/>' % [wx - 15, fy + 30, awn[0], ink]
							for g in 5:
								body += '<circle cx="%d" cy="%d" r="7" fill="#f4efe6"/>' % [wx, fy + 56 + g * 36]
						elif floor_k != 2:
							body += '<rect x="%d" y="%d" width="22" height="16" fill="#5c5e70" stroke="#%s" stroke-width="2"/>' % [wx - 11, fy + 50, ink]
						continue
					var lit := rng.randf() < 0.34
					body += '<rect x="%d" y="%d" width="%d" height="88" rx="4" fill="#%s" stroke="#%s" stroke-width="3"/>' % [wx - 6, wy - 6, w + 12, frame_col, ink]
					body += '<rect x="%d" y="%d" width="%d" height="76" fill="#%s"/>' % [wx, wy, w, "f2d394" if lit else "2a2f45"]
					if lit:
						var cc: String = curtain[rng.randi_range(0, 2)]
						body += '<path d="M%d,%d l16,0 q-6,38 -16,76 Z" fill="#%s" opacity="0.9"/>' % [wx, wy, cc]
						body += '<path d="M%d,%d l-16,0 q6,38 16,76 Z" fill="#%s" opacity="0.9"/>' % [wx + w, wy, cc]
					else:
						body += '<path d="M%d,%d l14,0 l-14,22 Z" fill="#ffffff" opacity="0.12"/>' % [wx + 6, wy + 4]
					body += '<rect x="%d" y="%d" width="4" height="76" fill="#%s"/>' % [wx + w / 2 - 2, wy, frame_col]
					body += '<rect x="%d" y="%d" width="%d" height="4" fill="#%s"/>' % [wx, wy + 34, w, frame_col]
					body += '<rect x="%d" y="%d" width="%d" height="8" rx="2" fill="#%s" stroke="#%s" stroke-width="2" stroke-opacity="0.5"/>' % [wx - 8, wy + 80, w + 16, hi, ink]
					var extra := rng.randf()
					if extra < 0.2:
						# A flower pot on the sill.
						body += '<rect x="%d" y="%d" width="16" height="12" fill="#c8664c" stroke="#%s" stroke-width="2"/>' % [wx + 8, wy + 68, ink]
						body += '<circle cx="%d" cy="%d" r="9" fill="#5aa04a"/><circle cx="%d" cy="%d" r="4" fill="#f06a8a"/>' % [wx + 16, wy + 62, wx + 18, wy + 58]
					elif extra < 0.32:
						# An air conditioner hung under the window.
						body += '<rect x="%d" y="%d" width="36" height="22" rx="3" fill="#b9bcc8" stroke="#%s" stroke-width="2.5"/>' % [wx + 10, wy + 88, ink]
						for g in 3:
							body += '<rect x="%d" y="%d" width="26" height="2" fill="#%s" opacity="0.6"/>' % [wx + 15, wy + 93 + g * 5, ink]
					elif extra < 0.44:
						# A striped awning over it.
						var ac: String = awn[rng.randi_range(0, 1)]
						var stripes := (w + 20) / 15
						for st_k in stripes:
							body += '<rect x="%d" y="%d" width="15" height="16" fill="#%s"/>' % [wx - 10 + st_k * 15, wy - 22, ac if st_k % 2 == 0 else "f4efe6"]
						for sc in stripes:
							body += '<circle cx="%d" cy="%d" r="7.5" fill="#%s"/>' % [wx - 2 + sc * 15, wy - 6, ac if sc % 2 == 0 else "f4efe6"]
						body += '<rect x="%d" y="%d" width="%d" height="3" fill="#%s" opacity="0.5"/>' % [wx - 10, wy - 23, stripes * 15, ink]
			# A drainpipe down the tile's seam, with brackets.
			body += '<rect x="2" y="0" width="10" height="512" fill="#7d7f92" stroke="#%s" stroke-width="2" stroke-opacity="0.6"/>' % ink
			for py in range(40, 512, 128):
				body += '<rect x="0" y="%d" width="14" height="6" fill="#5c5e70"/>' % py
		_:
			# Quilted diamonds with a soft highlight along their upper edges.
			body += '<rect width="256" height="256" fill="#%s"/>' % B
			for cx in [0, 128, 256]:
				for cy in [0, 128, 256]:
					for c in [[cx, cy], [cx - 64, cy - 64]]:
						var x: float = c[0]
						var y: float = c[1]
						body += '<path d="M%.0f,%.0f L%.0f,%.0f L%.0f,%.0f L%.0f,%.0f Z" fill="#%s" stroke="#%s" stroke-width="3" stroke-opacity="0.5" stroke-linejoin="round"/>' % [x, y - 56, x + 56, y, x, y + 56, x - 56, y, A, ink]
						body += '<path d="M%.0f,%.0f L%.0f,%.0f" stroke="#%s" stroke-width="5" stroke-linecap="round" opacity="0.6"/>' % [x - 40, y - 8, x - 8, y - 40, hi]
	var px := 512 if pattern == "facade" or pattern == "earth" or pattern == "seawall" else 256
	var svg := '<svg xmlns="http://www.w3.org/2000/svg" width="%d" height="%d" viewBox="0 0 %d %d">%s</svg>' % [px, px, px, px, body]
	var img := Image.new()
	img.load_svg_from_string(svg, 1.0)
	img.generate_mipmaps()
	var tex := ImageTexture.create_from_image(img)
	# On the stage's cel shading, laid in world space (one repeat every 3 units, so the stones and bands read large and calm; a facade's every 6.6).
	var m := FighterView.toon(Color.WHITE, true, tex, Vector2.ONE, {"facade": 6.6, "masonry": 7.0, "strata": 6.0, "roots": 8.0, "earth": 12.0, "seawall": 14.0}.get(pattern, 3.0))
	# The front falls further into the cool shade below its lit rim than the default, so the fighting surface and the fighters on it
	# lead and the wall under them settles back (the attention hierarchy in docs/ART_DIRECTION.md).
	m = m.duplicate()
	m.set_shader_parameter("front_fall", 0.52)
	m.set_shader_parameter("front_span", 6.5)
	# (The busiest fronts, the beach's bricks and the desert's strata, fall faster and further: at match scale their strong pattern
	# filled the bottom third of the screen as brightly as the fighting surface. Art review 5.)
	if pattern == "seawall" or pattern == "strata":
		m.set_shader_parameter("front_fall", 0.64)
		m.set_shader_parameter("front_span", 4.5)
	# Moss draping over the top of the wall where the theme has it (`moss`: colour), in a wavy band under the lip.
	if t.has("moss"):
		m.set_shader_parameter("moss_color", t.moss)
		m.set_shader_parameter("moss_amount", 1.0)
	# Painted wear over the pattern, so a block's front reads as one weathered mass rather than a stamped tile.
	match pattern:
		"facade":
			Materials.set_detail(m, "concrete", 0.45, 3.0)
		"cobble":
			Materials.set_detail(m, "stone", 0.4, 3.5)
		"masonry", "seawall":
			Materials.set_detail(m, "stone", 0.3, 4.0)
		_:
			Materials.set_detail(m, "soil", 0.35, 3.0)
	_soil_cache[key] = m
	return m


static var _soil_cache := {}


static func _mesh(parent: Node3D, mesh: Mesh, mat: Material, pos: Vector3, scl := Vector3.ONE, rot := Vector3.ZERO) -> MeshInstance3D:
	var mi := MeshInstance3D.new()
	mi.mesh = mesh
	mi.material_override = mat
	mi.position = pos
	mi.scale = scl
	mi.rotation_degrees = rot
	parent.add_child(mi)
	return mi


static func _sphere(r: float, segments := 24) -> SphereMesh:
	var s := SphereMesh.new()
	s.radius = r
	s.height = r * 2.0
	s.radial_segments = segments
	s.rings = segments / 2
	return s


static func _box(size: Vector3) -> BoxMesh:
	var b := BoxMesh.new()
	b.size = size
	return b


static func _cylinder(top: float, bottom: float, height: float, sides := 20) -> CylinderMesh:
	var c := CylinderMesh.new()
	c.top_radius = top
	c.bottom_radius = bottom
	c.height = height
	c.radial_segments = sides
	return c


# ---- The stage itself --------------------------------------------------------------------------------------------------------------

## A solid block in the theme: its patterned front, topped (if nothing sits on it) by the theme's cap: rolling grass with tufts, smooth sand
## with pebbles, or a rooftop ledge with a metal trim.
static func solid_block(parent: Node3D, t: Dictionary, left: float, right: float, top: float, bottom: float, front_z: float, exposed: bool) -> void:
	var depth := 8.0
	var body := _mesh(parent, _box(Vector3(right - left, top - bottom, depth)), _soil_material(t),
		Vector3((left + right) / 2.0, (top + bottom) / 2.0, front_z - depth / 2.0))
	body.name = "block"
	# Its front falls into shade below the top edge (`front_fall` in shaders/toon_common.gdshaderinc).
	body.set_instance_shader_parameter("front_top", top)
	if not exposed:
		return
	var cx := (left + right) / 2.0
	var w := right - left
	var rng := RandomNumberGenerator.new()
	rng.seed = int(left * 13.0 + top * 7.0)
	# A soft shadow on the front just under the cap, where its overhang keeps the light off: it makes the edge of the fighting surface
	# read at a glance.
	var cap_depth := {"roof": 0.42, "sand": 0.5}.get(t.get("cap", "grass"), 0.62) as float
	_mesh(parent, _shade_band(w + 0.2, 0.9), _band_material(), Vector3(cx, top - cap_depth - 0.45, front_z + 0.012))
	_ledges_and_buttresses(parent, t, left, right, top, bottom, front_z, cap_depth)
	match t.get("cap", "grass"):
		"roof":
			# A concrete ledge with a bright metal trim along the front.
			# (The ledge sits a hair above the block's top so their top faces never fight over the same depth and flicker.)
			# (Art review 6, the owner's rooftop plan: the roof's top, its edge and the facade under it read as three things. The roof is
			# warm tar-grey roofing, a step apart from the violet facade; the edge is a stone coping that stands proud, lit on top with a
			# shaded face and a drip shadow under it; the roofing's seams fall at uneven spacings, faint.)
			# (Art review 6 round 3: the roofing was one flat pale lilac, nearly the coping's value, so the edge of the fighting surface did
			# not read and the biggest surface on screen was blank. Now a step darker than the coping, with broad weathered patches.)
			var roofing := (t.grass as Color).lerp(Color(0.62, 0.56, 0.58), 0.55).darkened(0.1)
			var roof_mat := _surface(roofing, "roof", 4.0).duplicate() as ShaderMaterial
			roof_mat.set_shader_parameter("world_vary", 0.16)
			_mesh(parent, _box(Vector3(w + 0.3, 0.35, depth + 0.3)), roof_mat, Vector3(cx, top - 0.15, front_z - depth / 2.0))
			var coping := _soft((t.grass_light as Color).lerp(Color(0.86, 0.82, 0.8), 0.4))
			var coping_face := _soft((t.grass as Color).darkened(0.18))
			_mesh(parent, _box(Vector3(w + 0.5, 0.12, 0.5)), coping, Vector3(cx, top + 0.02, front_z + 0.02))
			_mesh(parent, _box(Vector3(w + 0.5, 0.2, 0.08)), coping_face, Vector3(cx, top - 0.13, front_z + 0.25))
			_mesh(parent, _box(Vector3(w + 0.3, 0.08, 0.1)), _soft((t.soil_dark as Color).darkened(0.35)), Vector3(cx, top - 0.27, front_z + 0.2))
			_mesh(parent, _box(Vector3(w + 0.3, 0.1, 0.1)), _unshaded(Color(0.95, 0.75, 0.3)), Vector3(cx, top - 0.44, front_z + 0.16))
			var seam := _soft(roofing.darkened(0.08))
			var lap := _soft(roofing.lightened(0.05))
			var sx := left + 1.1
			var j := 0
			while sx < right - 0.6:
				_mesh(parent, _box(Vector3(0.07, 0.02, depth - 0.4)), seam, Vector3(sx, top + 0.031, front_z - depth / 2.0 - 0.1))
				_mesh(parent, _box(Vector3(0.05, 0.02, depth - 0.4)), lap, Vector3(sx + 0.07, top + 0.03, front_z - depth / 2.0 - 0.1))
				sx += 2.1 + 1.3 * absf(sin(float(j) * 2.7 + 0.4))
				j += 1
		"sand":
			var sand := _surface(t.grass, "sand", 5.0)
			# Broad warm and cool drifts across the sand where the theme asks for them (`grass_vary`), as on the lawns: the canyon's top was
			# one flat sheet beside its banded strata front (the owner's outside brief; art review 6 round 5).
			if t.has("grass_vary"):
				sand = sand.duplicate()
				sand.set_shader_parameter("world_vary", t.grass_vary)
			_mesh(parent, _box(Vector3(w + 0.2, 0.45, depth + 0.1)), sand, Vector3(cx, top - 0.2, front_z - depth / 2.0))
			_mesh(parent, _cylinder(0.26, 0.26, w + 0.2), sand, Vector3(cx, top - 0.26, front_z + 0.02), Vector3.ONE, Vector3(0, 0, 90))
			_mesh(parent, _box(Vector3(w + 0.24, 0.1, 0.2)), _soft(t.grass_light), Vector3(cx, top - 0.05, front_z + 0.24))
			# A few small clusters of pebbles and the odd shell on the back of the sand, gathered toward the ends so the middle stays clean.
			# (Art review 6 round 3: one size of pebble every metre or two read as a dotted pattern, a row of dimples across the sand.)
			var pebble := Materials.stone((t.soil as Color).lightened(0.2))
			var pebble_dark := Materials.stone((t.soil as Color).darkened(0.05))
			var shell := _soft(Color(1.0, 0.86, 0.84))
			for u in [0.08, 0.2, 0.34, 0.68, 0.84, 0.94]:
				if rng.randf() < 0.2:
					continue
				var at := Vector3(lerpf(left + 0.6, right - 0.6, u), top + 0.03, front_z - rng.randf_range(2.2, 5.0))
				for j in rng.randi_range(2, 4):
					var r := rng.randf_range(0.06, 0.17) * (1.3 if j == 0 else 1.0)
					var off := Vector3(rng.randf_range(-0.45, 0.45), 0, rng.randf_range(-0.3, 0.3)) * (0.0 if j == 0 else 1.0)
					_mesh(parent, _sphere(r, 8), pebble if rng.randf() < 0.6 else pebble_dark, at + off, Vector3(1.3, 0.6, 1.0),
						Vector3(0, rng.randf_range(0, 180), 0))
				if rng.randf() < 0.45:
					var sp := at + Vector3(rng.randf_range(0.5, 0.9) * (1.0 if rng.randf() < 0.5 else -1.0), 0.01, rng.randf_range(-0.2, 0.2))
					_mesh(parent, _sphere(0.13, 10), shell, sp, Vector3(1.0, 0.35, 0.8), Vector3(0, rng.randf_range(0, 180), 0))
		_:
			# A thick grass cap that rolls over the front edge, with tufts hanging over.
			var grass := _surface(t.grass, "grass", 9.0)
			# Broad sunlit and shaded patches across the lawn where the theme asks for them (`grass_vary`), so the top is not one sheet.
			if t.has("grass_vary"):
				grass = grass.duplicate()
				grass.set_shader_parameter("world_vary", t.grass_vary)
			var cap_h := 0.55
			_mesh(parent, _box(Vector3(w + 0.25, cap_h, depth + 0.1)), grass, Vector3(cx, top - cap_h / 2.0 + 0.04, front_z - depth / 2.0))
			var lip := _mesh(parent, _cylinder(cap_h * 0.6, cap_h * 0.6, w + 0.25), grass, Vector3(cx, top - cap_h * 0.55, front_z + 0.02), Vector3.ONE, Vector3(0, 0, 90))
			lip.name = "lip"
			_mesh(parent, _box(Vector3(w + 0.3, 0.12, 0.2)), _soft(t.grass_light), Vector3(cx, top - 0.06, front_z + 0.3))
			# Tufts hanging over at uneven gaps and sizes, now and then a small stone among them (not where moss drapes the wall instead).
			var x := left + rng.randf_range(0.4, 0.9) if not t.has("moss") else right
			var tuft_mesh := _sphere(0.32, 12)
			var stone := Materials.stone((t.soil as Color).lightened(0.25))
			while x < right - 0.4:
				var k := rng.randf_range(0.75, 1.3)
				# Each tongue its own width and length (from its own place, so the stage's sequence is unchanged), hanging from inside the lip,
				# so the edge reads as one scalloped skirt of turf rather than a row of equal beads.
				var own := RandomNumberGenerator.new()
				own.seed = hash(roundi(x * 100.0))
				# (Every few a long drip of turf, and neighbours overlapping, so the edge is one ragged skirt, not a row of beads.)
				var long_drip := own.randf() < 0.18
				var hang := own.randf_range(1.6, 2.4) if long_drip else own.randf_range(0.5, 1.3)
				var wide := own.randf_range(0.7, 1.0) if long_drip else own.randf_range(1.0, 1.7)
				_mesh(parent, tuft_mesh, grass, Vector3(x, top - cap_h + 0.1 - 0.17 * k * hang, front_z + 0.2 + own.randf_range(-0.04, 0.04)),
					Vector3(1.2 * wide, 0.8 * hang, 0.6) * k)
				if rng.randf() < 0.22:
					_mesh(parent, _sphere(0.16, 6), stone, Vector3(x + 0.5, top - cap_h - 0.3, front_z + 0.12), Vector3(1.3, 0.8, 0.7))
				x += rng.randf_range(0.8, 1.6) * 0.62


## What makes a stage's block a built shape rather than a box with a tiled front: at each top corner (the ledges, gameplay landmarks) a
## chunky corner stone standing proud of the corner, lit on top, under the cap (on a rooftop a metal corner cap instead), and down each
## end of the front a buttress, a column of bigger stones a little proud of the wall, narrowing toward the foot, so the wall is framed and
## the tiled pattern is not the whole face. Deterministic.
static func _ledges_and_buttresses(parent: Node3D, t: Dictionary, left: float, right: float, top: float, bottom: float, front_z: float,
		cap_depth: float) -> void:
	var city: bool = t.get("cap", "grass") == "roof"
	# (Not on the earth islands, the meadow and the grove: their sky islands, the same kind of thing, are earth with no masonry, and a
	# brick buttress at each end made the stage a different object from them. Art review 6, the owner: one cohesive style.)
	# (Nor the desert's strata, carved rock like the mesas round it.)
	if t.get("pattern", "") in ["earth", "roots", "strata"]:
		return
	var stone_c := (t.soil as Color).lightened(0.12)
	var stone := Materials.stone(stone_c, Vector2(1, 1))
	var dark := Materials.stone((t.soil_dark as Color).darkened(0.05), Vector2(1, 2))
	var h := top - bottom
	for side in [-1.0, 1.0]:
		var edge_x: float = right if side > 0.0 else left
		if city:
			# A metal corner cap over the ledge: a bracket down the corner and a bright cap on top.
			var steel := FighterView.respond(FighterView.toon(Color(0.62, 0.64, 0.74)), "metal")
			_mesh(parent, _box(Vector3(0.34, 0.9, 0.34)), steel, Vector3(edge_x - side * 0.08, top - 0.55, front_z + 0.1))
			_mesh(parent, _box(Vector3(0.44, 0.14, 0.44)), steel, Vector3(edge_x - side * 0.08, top - 0.1, front_z + 0.12))
			continue
		# The corner stone: a squat, slightly turned block, a hair proud of the front and the side.
		var cs := _mesh(parent, _box(Vector3(0.95, 0.7, 0.95)), stone, Vector3(edge_x - side * 0.36, top - cap_depth - 0.3, front_z + 0.08),
			Vector3.ONE, Vector3(0, side * 4.0, side * -3.0))
		cs.name = "ledge_stone"
		# The buttress: stacked stones down the front, each a little narrower and darker toward the foot.
		var y := top - cap_depth - 0.7
		var k := 0
		while y > bottom + 0.6 and k < 12:
			var sh := 0.9 + 0.25 * absf(sin(float(k) * 1.9 + side))
			var sw := lerpf(0.9, 0.7, clampf((top - y) / maxf(h, 0.1), 0.0, 1.0))
			_mesh(parent, _box(Vector3(sw, sh - 0.06, 0.4)), stone if k % 3 != 2 else dark,
				Vector3(edge_x - side * (0.4 + 0.03 * sin(float(k) * 2.7)), y - sh / 2.0, front_z + 0.14))
			y -= sh
			k += 1


## A strip `w` wide and `h` tall, dark at its top edge fading to nothing at the bottom (vertex colours), for a soft shadow under an edge.
static func _shade_band(w: float, h: float) -> ArrayMesh:
	var key := "%.2f,%.2f" % [w, h]
	if _band_cache.has(key):
		return _band_cache[key]
	var verts := PackedVector3Array([Vector3(-w / 2.0, h / 2.0, 0), Vector3(w / 2.0, h / 2.0, 0), Vector3(w / 2.0, -h / 2.0, 0),
		Vector3(-w / 2.0, h / 2.0, 0), Vector3(w / 2.0, -h / 2.0, 0), Vector3(-w / 2.0, -h / 2.0, 0)])
	var dark := Color(0.08, 0.04, 0.12, 0.42)
	var clear := Color(0.08, 0.04, 0.12, 0.0)
	var cols := PackedColorArray([dark, dark, clear, dark, clear, clear])
	var arrays := []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = verts
	arrays[Mesh.ARRAY_COLOR] = cols
	var mesh := ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	_band_cache[key] = mesh
	return mesh


static var _band_cache := {}
static var _band_mat: StandardMaterial3D


static func _band_material() -> StandardMaterial3D:
	if _band_mat == null:
		_band_mat = StandardMaterial3D.new()
		_band_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		_band_mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		_band_mat.vertex_color_use_as_albedo = true
		_band_mat.cull_mode = BaseMaterial3D.CULL_DISABLED
	return _band_mat


## How deep a platform is, front to back (world units, back from its front).
const PLATFORM_DEPTH := 4.0


## A pass-through platform in the theme: a rounded wooden plank (a light board on a darker beam), or a steel girder with rivets.
static func platform(parent: Node3D, t: Dictionary, left: float, right: float, top: float, front_z: float) -> void:
	var w := right - left
	var depth := PLATFORM_DEPTH
	var cx := (left + right) / 2.0
	if t.get("platform", "wood") == "girder":
		var steel := _surface(t.wood, "steel", 3.0)
		_mesh(parent, _box(Vector3(w, 0.18, depth)), steel, Vector3(cx, top - 0.09, front_z - depth / 2.0))
		_mesh(parent, _box(Vector3(w, 0.5, 0.16)), Materials.wood(t.wood_dark, Vector2(4, 1)), Vector3(cx, top - 0.4, front_z - 0.3))
		# The flange underneath in a darker steel, so top, web and underside read as three parts of one beam.
		_mesh(parent, _box(Vector3(w, 0.14, depth * 0.7)), _surface((t.wood_dark as Color).darkened(0.15), "steel", 3.0), Vector3(cx, top - 0.68, front_z - depth / 2.0))
		# A bright edge where the top plate meets its front (it catches the light and marks where the platform ends), and the plate's
		# shade falling on the web below it.
		_mesh(parent, _box(Vector3(w, 0.05, 0.06)), _soft((t.wood as Color).lightened(0.3)), Vector3(cx, top - 0.02, front_z + 0.005))
		_mesh(parent, _shade_band(w, 0.28), _band_material(), Vector3(cx, top - 0.3, front_z - 0.21))
		var rivet := _unshaded(Color(0.95, 0.82, 0.6))
		var x := left + 0.35
		while x < right - 0.2:
			_mesh(parent, _sphere(0.06, 6), rivet, Vector3(x, top - 0.4, front_z - 0.2))
			x += 0.7
		return
	var board := _surface(t.wood, "wood", 2.4)
	_mesh(parent, _box(Vector3(w, 0.26, depth)), board, Vector3(cx, top - 0.13, front_z - depth / 2.0))
	_mesh(parent, _box(Vector3(w - 0.3, 0.22, depth - 0.4)), Materials.wood(t.wood_dark, Vector2(4, 1)), Vector3(cx, top - 0.37, front_z - depth / 2.0))
	for side in [-1.0, 1.0]:
		_mesh(parent, _cylinder(0.13, 0.13, depth), board, Vector3(cx + side * w / 2.0, top - 0.13, front_z - depth / 2.0), Vector3.ONE, Vector3(90, 0, 0))
	# A light edge where the top meets the front (it catches the light, and says where the platform ends), and a soft shade on the beam
	# under the board.
	_mesh(parent, _box(Vector3(w, 0.05, 0.06)), _soft((t.wood as Color).lightened(0.22)), Vector3(cx, top - 0.02, front_z + 0.005))
	_mesh(parent, _shade_band(w - 0.3, 0.3), _band_material(), Vector3(cx, top - 0.4, front_z - 0.19))
	var seam := _unshaded((t.wood_dark as Color).darkened(0.2))
	var n := maxi(1, int(w / 1.6))
	for k in range(1, n):
		_mesh(parent, _box(Vector3(0.05, 0.26, 0.05)), seam, Vector3(left + w * k / n, top - 0.13, front_z + 0.01))
	# How it is built (art review 6 round 32: one slab with a seam or two read as a prototype block): a pair of nail heads either side of
	# every seam and at both ends, and a cross-batten under each end, the joists the boards are nailed to.
	var nail := _unshaded((t.wood_dark as Color).darkened(0.1))
	for k in range(0, n + 1):
		var sx := left + w * k / n
		for side in [-1.0, 1.0]:
			var nx: float = sx + side * 0.16
			if nx < left + 0.08 or nx > right - 0.08:
				continue
			for ny in [-0.07, -0.19]:
				_mesh(parent, _sphere(0.018, 6), nail, Vector3(nx, top + ny, front_z + 0.015))
	var batten := Materials.wood((t.wood_dark as Color).darkened(0.12), Vector2(1, 1))
	for bx in [left + 0.55, right - 0.55]:
		# (Inside the beam's height and set back from the front, round 36: lower and flush it hung below the platform as a dark block.)
		_mesh(parent, _box(Vector3(0.3, 0.16, depth - 0.8)), batten, Vector3(bx, top - 0.52, front_z - depth / 2.0 - 0.1))
	_platform_dressing(parent, t, left, right, top, front_z, depth)


## Dresses a wooden platform for its stage, so it belongs to the place rather than being the same plank everywhere: rope lashings near
## each end on the beach and in the canyon; moss over the front edge and a vine or two hanging below in the grove; the meadow's
## plain plank. Only at the ends and below the top, so the play surface and fighters' feet stay clear.
## Deterministic (from the platform's place).
static func _platform_dressing(parent: Node3D, t: Dictionary, left: float, right: float, top: float, front_z: float, depth: float) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = hash(Vector2i(roundi(left * 10.0), roundi(top * 10.0)))
	var cx := (left + right) / 2.0
	var w := right - left
	match t.get("name", ""):
		"ocean", "sunset":
			var rope := Materials.wood(Color(0.86, 0.74, 0.52) if t.name == "ocean" else Color(0.78, 0.62, 0.42), Vector2(1, 6))
			# (Bands of rope wrapped round the board, round 36: squashed rings seen edge-on read as pink fins.)
			for ex in [left + 0.6, right - 0.6]:
				for j in 3:
					_mesh(parent, _box(Vector3(0.09, 0.3, 0.08)), rope, Vector3(ex + (j - 1) * 0.13, top - 0.16, front_z + 0.03))
		"grove":
			var moss := Materials.foliage(Color(0.36, 0.64, 0.36), Vector2(2, 2), true)
			# (Overlapping clumps of very different sizes, gaps now and then: an even row read as beads.)
			var x := left + 0.3
			while x < right - 0.2:
				var r := rng.randf_range(0.1, 0.3)
				if rng.randf() > 0.2:
					_mesh(parent, _sphere(1.0, 8), moss, Vector3(x, top - 0.26 - r * rng.randf_range(0.2, 1.0), front_z + 0.02), Vector3(r * 1.8, r * 1.2, r))
				x += r * rng.randf_range(1.2, 2.2)
			var vine := Materials.foliage(Color(0.3, 0.58, 0.32), Vector2(1, 3), true)
			for k in 2:
				var vx := lerpf(left + 0.8, right - 0.8, rng.randf())
				var vl := rng.randf_range(0.8, 1.6)
				_mesh(parent, _cylinder(0.05, 0.07, vl, 6), vine, Vector3(vx, top - 0.45 - vl * 0.5, front_z - 0.1), Vector3.ONE, Vector3(0, 0, rng.randf_range(-8, 8)))
				_mesh(parent, _sphere(0.13, 8), vine, Vector3(vx, top - 0.45 - vl, front_z - 0.1), Vector3(1.2, 0.8, 1.0))
		"meadow":
			# (Nothing: hay tufts under the ends read as bread rolls, a flower below as a dangling berry. The meadow's plank is its own.)
			pass


# ---- Backdrops ------------------------------------------------------------------------------------------------------------------------

## Builds the scenery behind the stage for theme `t` under `root`. `stage` is the stage's extent: [left, right, top, bottom] of its
## solid blocks, so decorations sit on and around it.
static func backdrop(root: Node3D, t: Dictionary, stage: Array) -> void:
	match t.name:
		"grove":
			_grove(root, t, stage)
		"sunset":
			_sunset(root, t, stage)
		"night":
			_night(root, t)
		"ocean":
			_ocean(root, t, stage)
		"city":
			_city(root, t, stage)
		_:
			_meadow(root, t, stage)
	# Painted ridges far behind everything (shaders/painted_ridge.gdshader), coloured from the theme's own sky so the backdrop, the
	# stage and the fighters' light share one scheme.
	for spec in RIDGES.get(t.name, []):
		ridge(root, t, spec)
	# Fog layers: soft bands of haze in the horizon's colour between the layers of scenery, thickest at the bottom, so each layer sits
	# further back than the one before (a painted depth, at the cost of a few see-through cards).
	var haze: Color = (t.sky_horizon as Color).lerp(Color.WHITE, 0.1)
	for layer in MIST.get(t.name, []):
		_mist(root, haze, layer[0], layer[1], layer[2], layer[3])
	painted_layers(root, t.name)
	# What hangs under the stage, fitted to its lowest block (the extent's last two numbers) where it has one.
	var e := _ext(stage)
	underside(root, t, [stage[4], stage[5], e[2], e[3]] if stage.size() >= 6 else e)
	_props(root, t.name, _ext(stage))
	merge_static(root)


## Merges the still scenery that shares a material into one mesh each (a theme is dozens of small shapes: stars, fence posts, windows),
## so the whole backdrop costs a few draw calls rather than hundreds. Only `root`'s own children are merged: anything that moves (sails,
## crowns, the sea) sits under a node of its own and is left alone, as is anything see-through. Returns how many meshes it saved.
static func merge_static(root: Node3D) -> int:
	var groups := {}
	var order: Array = []
	for c in root.get_children():
		var mi := c as MeshInstance3D
		# (Meshes built by hand, like the great tree's trunk, are left as they are: merged in with the built-in shapes they were lost.
		# Authored props' parts, placed by `_place_prop`, are merged.)
		if mi == null or mi.mesh == null or (mi.mesh is ArrayMesh and not mi.has_meta("prop")) or mi.material_override == null or mi.transparency > 0.0 				or mi.get_child_count() > 0:
			continue
		var key: int = mi.material_override.get_instance_id()
		if not groups.has(key):
			groups[key] = []
			order.append(key)
		groups[key].append(mi)
	var saved := 0
	for key in order:
		var parts: Array = groups[key]
		if parts.size() < 2:
			continue
		var st := SurfaceTool.new()
		for mi: MeshInstance3D in parts:
			for surface in mi.mesh.get_surface_count():
				st.append_from(mi.mesh, surface, mi.transform)
		var one := MeshInstance3D.new()
		one.mesh = st.commit()
		one.material_override = parts[0].material_override
		one.cast_shadow = parts[0].cast_shadow
		root.add_child(one)
		for mi: MeshInstance3D in parts:
			root.remove_child(mi)
			mi.free()
		saved += parts.size() - 1
	return saved


# ---- Props: a few small low-poly things behind the fighters per theme (few sides, flat colours, ink outlines) ------------------------

static func _props(root: Node3D, theme_name: String, e: Array) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 77
	var l: float = e[0]
	var r: float = e[1]
	var y: float = e[2]
	# Each prop sits in a soft contact patch (an ellipse, `[x, z, half width, half depth]`), so it rests on the ground rather than floating
	# over it: screen-space ambient occlusion does not show under this light, and the baked shading (`StageLight`) only knows the blocks.
	var patches: Array = []
	match theme_name:
		"meadow":
			patches = [[l + 3.0, -4.6, 0.95, 0.75], [l + 4.3, -5.0, 0.95, 0.75], [r - 4.0, -4.4, 0.55, 0.45], [r - 2.4, -4.0, 0.3, 0.25],
				[l + 7.5, -3.8, 0.6, 0.45]]
			_hay_bale(root, Vector3(l + 3.0, y, -4.6), 0.0)
			_hay_bale(root, Vector3(l + 4.3, y, -5.0), 25.0)
			_barrel(root, Vector3(r - 4.0, y, -4.4), 0.8)
			_signpost(root, Vector3(r - 2.4, y, -4.0), -1.0)
			_rock(root, Vector3(l + 7.5, y, -3.8), 0.45, rng)
		"grove":
			patches = [[l + 3.15, -4.1, 1.1, 0.6], [r - 5.0, -4.0, 0.4, 0.3], [r - 6.5, -4.4, 0.85, 0.65], [l + 6.0, -3.6, 0.55, 0.42]]
			for k in 3:
				_mushroom(root, Vector3(l + 2.6 + k * 0.55, y, -3.8 - k * 0.3), 0.5 + k * 0.15, Color(0.95, 0.5, 0.6))
			_mushroom(root, Vector3(r - 5.0, y, -4.0), 0.55, Color(0.7, 0.6, 1.0))
			_stump(root, Vector3(r - 6.5, y, -4.4))
			_rock(root, Vector3(l + 6.0, y, -3.6), 0.4, rng)
		"sunset":
			patches = [[l + 3.2, -4.2, 1.15, 0.85], [l + 4.2, -3.8, 0.6, 0.45], [r - 4.6, -4.4, 0.75, 0.6], [r - 3.4, -4.0, 0.5, 0.4],
				[l + 8.0, -4.8, 0.3, 0.25], [r - 7.5, -3.6, 0.48, 0.36]]
			_rock(root, Vector3(l + 3.2, y, -4.2), 0.8, rng, CANYON_ROCK)
			_rock(root, Vector3(l + 4.2, y, -3.8), 0.45, rng, CANYON_ROCK)
			_crate(root, Vector3(r - 4.6, y, -4.4), 0.9, 12.0)
			_barrel(root, Vector3(r - 3.4, y, -4.0), 0.75)
			_lantern_post(root, Vector3(l + 8.0, y, -4.8), Color(1.0, 0.75, 0.4))
			_rock(root, Vector3(r - 7.5, y, -3.6), 0.35, rng, CANYON_ROCK)
		"ocean":
			patches = [[l + 4.5, -4.6, 0.25, 0.2], [r - 4.4, -4.8, 0.25, 0.2], [l + 6.4, -3.6, 0.38, 0.3], [r - 6.5, -4.0, 0.9, 0.55]]
			_umbrella(root, Vector3(l + 4.5, y, -4.6), Color(0.98, 0.55, 0.55))
			_life_ring(root, Vector3(r - 4.4, y, -4.8))
			_bucket(root, Vector3(l + 6.4, y, -3.6), Color(0.45, 0.7, 1.0))
			_rock(root, Vector3(r - 6.8, y, -3.8), 0.4, rng, BEACH_ROCK)
			_rock(root, Vector3(r - 6.2, y, -4.2), 0.3, rng, BEACH_ROCK)
		"city":
			patches = [[l + 2.2, -4.5, 1.6, 0.75], [r - 4.0, -4.0, 0.5, 0.4]]
			_crate(root, Vector3(l + 1.6, y, -4.6), 1.0, 0.0)
			_crate(root, Vector3(l + 2.7, y, -4.4), 0.8, 8.0)
			_crate(root, Vector3(l + 2.0, y + 1.0, -4.6), 0.7, -10.0)
			_potted_plant(root, Vector3(r - 4.0, y, -4.0))
			_string_lights(root, Vector3(l + 3.0, y + 4.6, -6.0), Vector3(r - 2.5, y + 5.6, -6.5))
			# The bulbs light what is under them: three soft warm and rose pools along the string (no shadows, short range), so the roof,
			# the crates and the fighters take a little of their colour, and a red one from the mast's beacon. A few practical lights
			# rather than one per bulb.
			for k in 3:
				var f := (float(k) + 0.5) / 3.0
				var at := Vector3(l + 3.0, y + 4.6, -6.0).lerp(Vector3(r - 2.5, y + 5.6, -6.5), f) + Vector3(0, -1.6, 1.5)
				_practical(root, at, [Color(1.0, 0.78, 0.5), Color(1.0, 0.6, 0.72), Color(1.0, 0.78, 0.5)][k], 1.1, 7.0)
			_practical(root, Vector3(r - 2.5, y + 6.1, -6.0), Color(1.0, 0.3, 0.3), 0.8, 4.0)
	for p in patches:
		_contact(root, Vector3(p[0], y, p[1]), Vector2(p[2], p[3]))


## A practical light: a small warm pool from a lamp or a bulb in the scenery (`energy`, `reach` in units), no shadows, so it costs little.
## Emission alone makes a thing look lit; this lets it light what is round it.
static func _practical(root: Node3D, at: Vector3, colour: Color, energy: float, reach: float) -> OmniLight3D:
	var o := OmniLight3D.new()
	o.light_color = colour
	o.light_energy = energy
	o.omni_range = reach
	o.omni_attenuation = 1.6
	o.shadow_enabled = false
	o.light_specular = 0.0
	o.position = at
	root.add_child(o)
	return o


## A soft dark ellipse lying on the ground at `at` (`half` its half width and half depth), darkest in the middle: the shade a thing casts
## on the ground right where it stands. One shared see-through material, in the stage's shadow hue family.
static func _contact(root: Node3D, at: Vector3, half: Vector2) -> void:
	if _contact_mat == null:
		var g := Gradient.new()
		var ink := Color(0.1, 0.05, 0.16)
		g.set_color(0, Color(ink, CONTACT_STRENGTH))
		g.set_color(1, Color(ink, 0.0))
		g.add_point(0.55, Color(ink, CONTACT_STRENGTH * 0.85))
		var tex := GradientTexture2D.new()
		tex.gradient = g
		tex.fill = GradientTexture2D.FILL_RADIAL
		tex.fill_from = Vector2(0.5, 0.5)
		tex.fill_to = Vector2(1.0, 0.5)
		tex.width = 64
		tex.height = 64
		_contact_mat = StandardMaterial3D.new()
		_contact_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		_contact_mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		_contact_mat.albedo_texture = tex
		_contact_mat.disable_fog = true
	# The match camera looks along the ground from low down, so most of the patch is hidden behind its prop: it reaches wider than the
	# footprint and further toward the camera, where it shows.
	var q := QuadMesh.new()
	q.size = half * Vector2(2.6, 3.2)
	q.orientation = PlaneMesh.FACE_Y
	at.z += half.y * 0.35
	# (Just above the ground, the grass cap included, which stands 0.04 proud of the block, so it never fights the top face for depth.)
	var mi := _mesh(root, q, _contact_mat, at + Vector3(0, 0.06, 0))
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	mi.name = "Contact"


static var _contact_mat: StandardMaterial3D
## How dark a contact patch is at its middle (alpha).
const CONTACT_STRENGTH := 0.55


static func _hay_bale(root: Node3D, at: Vector3, turn: float) -> void:
	# A round bale lying on its side: a drum whose rim rounds over into flat ends (a lathe, so its strands can run round it), the straw
	# drawn as strands wrapping the body and as a rolled spiral on each end, tied with two bands of twine. Warm gold, lit and shaded by the
	# stage's light like any surface, not a flat decal.
	var centre := at + Vector3(0, 0.55, 0)
	var rot := Vector3(90, turn, 0)
	var body_mat := Materials.as_family(FighterView.toon(Color.WHITE, true, _straw_texture(false), Vector2.ONE), "grass")
	_mesh(root, _lathe(HAY_PROFILE, 22, 3.0), body_mat, centre, Vector3.ONE, rot)
	var end_mat := Materials.as_family(FighterView.toon(Color.WHITE, false, _straw_texture(true), Vector2.ONE), "grass")
	var axis := Basis.from_euler(rot * PI / 180.0) * Vector3.UP
	for side in [-1.0, 1.0]:
		_mesh(root, _disc(0.47, 22), end_mat, centre + axis * side * 0.645, Vector3.ONE, rot + Vector3(0.0 if side > 0.0 else 180.0, 0, 0))
	var twine := FighterView.toon(Color(0.66, 0.44, 0.26), false)
	for dz in [-0.3, 0.3]:
		var ring := TorusMesh.new()
		ring.inner_radius = 0.545
		ring.outer_radius = 0.585
		ring.rings = 22
		ring.ring_segments = 6
		_mesh(root, ring, twine, centre + axis * dz, Vector3.ONE, rot)


## A round bale's outline along its axis (y, from end to end) as [radius, y]: the body a little fuller in the middle, the rim rounding
## over into the flat ends.
const HAY_PROFILE := [[0.44, -0.65], [0.5, -0.635], [0.535, -0.6], [0.552, -0.54], [0.56, -0.42], [0.565, -0.2], [0.567, 0.0],
	[0.565, 0.2], [0.56, 0.42], [0.552, 0.54], [0.535, 0.6], [0.5, 0.635], [0.44, 0.65]]


## A surface of revolution round the y axis through `profile` ([radius, y] pairs, in order), `sides` round, its UV u running round it
## `u_repeat` times and v along the profile; smooth normals. Open at both ends.
static func _lathe(profile: Array, sides: int, u_repeat: float) -> ArrayMesh:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var n := profile.size()
	for i in n - 1:
		for k in sides:
			var quad := []
			for c in [[i, k], [i, k + 1], [i + 1, k + 1], [i + 1, k]]:
				var pr: Array = profile[c[0]]
				var a := TAU * float(c[1]) / float(sides)
				quad.append([Vector3(cos(a) * float(pr[0]), float(pr[1]), sin(a) * float(pr[0])), Vector2(float(c[1]) / float(sides) * u_repeat, float(c[0]) / float(n - 1))])
			for t in [0, 1, 2, 0, 2, 3]:
				st.set_uv(quad[t][1])
				st.add_vertex(quad[t][0])
	st.generate_normals()
	return st.commit()


## A flat disc of radius `r` facing up (+y), its UV laid flat across it (the middle at 0.5, 0.5).
static func _disc(r: float, sides: int) -> ArrayMesh:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	st.set_normal(Vector3.UP)
	for k in sides:
		var a0 := TAU * float(k) / float(sides)
		var a1 := TAU * float(k + 1) / float(sides)
		for v in [Vector2.ZERO, Vector2(cos(a0), sin(a0)), Vector2(cos(a1), sin(a1))]:
			st.set_uv(Vector2(0.5, 0.5) + v * 0.5)
			st.add_vertex(Vector3(v.x * r, 0.0, v.y * r))
	return st.commit()


## The bale's straw, drawn once: `ends` the rolled spiral seen on an end (fine strands round a darker heart), else strands of straw
## running round the body in close golden shades, with a few loose wisps.
static func _straw_texture(ends: bool) -> Texture2D:
	var key := "ends" if ends else "body"
	if _straw_cache.has(key):
		return _straw_cache[key]
	var gold := Color(0.95, 0.78, 0.44)
	var deep := Color(0.78, 0.56, 0.28)
	var pale := Color(1.0, 0.9, 0.62)
	var rng := RandomNumberGenerator.new()
	rng.seed = 21
	var body := '<rect width="256" height="256" fill="#%s"/>' % gold.to_html(false)
	if ends:
		body += '<circle cx="128" cy="128" r="128" fill="#%s"/>' % gold.lerp(deep, 0.15).to_html(false)
		# The roll: a spiral from the heart out, drawn as many short arcs in alternating shades so it reads as layers of straw.
		var turns := 7.0
		var steps := 300
		var prev := Vector2(128, 128)
		for i in range(1, steps + 1):
			var t := float(i) / float(steps)
			var a := t * turns * TAU
			var rr := 8.0 + t * 112.0
			var p := Vector2(128, 128) + Vector2(cos(a), sin(a)) * rr
			var c := deep if (i / 6) % 2 == 0 else pale
			body += '<line x1="%.1f" y1="%.1f" x2="%.1f" y2="%.1f" stroke="#%s" stroke-width="%.1f" stroke-linecap="round" opacity="0.7"/>' % [prev.x, prev.y, p.x, p.y, c.to_html(false), 3.5 + t * 2.0]
			prev = p
		body += '<circle cx="128" cy="128" r="12" fill="#%s"/>' % deep.darkened(0.2).to_html(false)
		# A darker ring at the rim, where the end turns into the body.
		body += '<circle cx="128" cy="128" r="122" fill="none" stroke="#%s" stroke-width="10" opacity="0.45"/>' % deep.to_html(false)
	else:
		# Strands across the tile (round the bale), each a long gentle wave; they tile across, and every band is drawn again below the edge.
		for j in 70:
			var y := rng.randf_range(0.0, 256.0)
			var c := [deep, pale, gold.lerp(deep, 0.4), gold.lerp(pale, 0.5)][j % 4] as Color
			var amp := rng.randf_range(1.0, 3.0)
			var ph := rng.randf_range(0.0, TAU)
			var w := rng.randf_range(1.6, 3.4)
			for oy in [-256.0, 0.0, 256.0]:
				var d := "M0,%.1f" % (y + oy + sin(ph) * amp)
				for x in range(16, 257, 16):
					d += " L%d,%.1f" % [x, y + oy + sin(ph + float(x) / 256.0 * TAU * 2.0) * amp]
				body += '<path d="%s" fill="none" stroke="#%s" stroke-width="%.1f" stroke-linecap="round" opacity="0.65"/>' % [d, c.to_html(false), w]
	var svg := '<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">%s</svg>' % body
	var img := Image.new()
	img.load_svg_from_string(svg, 1.0)
	img.generate_mipmaps()
	var tex := ImageTexture.create_from_image(img)
	_straw_cache[key] = tex
	return tex


static var _straw_cache := {}


static func _barrel(root: Node3D, at: Vector3, size: float) -> void:
	# A cooper's barrel: a bulging body of staves (a lathe, the staves painted on so they run up it), two iron hoops that sit on the bulge,
	# a rim where the staves stand above the lid, and the lid inset below it. Each part a different value, so it reads as built.
	var body := Materials.as_family(FighterView.toon(Color.WHITE, true, _stave_texture(), Vector2.ONE), "wood")
	var prof := []
	for p in BARREL_PROFILE:
		prof.append([float(p[0]) * size, float(p[1]) * size])
	_mesh(root, _lathe(prof, 18, 1.0), body, at)
	var hoop := FighterView.respond(FighterView.toon(Color(0.36, 0.34, 0.42), false), "metal")
	for h in [0.2, 0.84]:
		var ring := TorusMesh.new()
		var r := _barrel_radius(h) * size
		ring.inner_radius = r - 0.01 * size
		ring.outer_radius = r + 0.045 * size
		ring.rings = 18
		ring.ring_segments = 6
		_mesh(root, ring, hoop, at + Vector3(0, h * 1.1 * size, 0), Vector3(1, 0.9, 1))
	# The lid, a step down inside the rim, darker (it is in the rim's shade), with the rim's inner wall round it.
	_mesh(root, _cylinder(0.35 * size, 0.35 * size, 0.06 * size, 18), FighterView.toon(Color(0.5, 0.32, 0.22), false), at + Vector3(0, 1.04 * size, 0))
	_mesh(root, _disc(0.35 * size, 18), Materials.as_family(FighterView.toon(Color.WHITE, false, _stave_texture(true), Vector2.ONE), "wood"),
		at + Vector3(0, 1.071 * size, 0))


## A barrel's outline from foot to top as [radius, height] (size 1): narrow at the ends, bulging in the middle, the top a rim of staves
## standing a little above the lid.
const BARREL_PROFILE := [[0.33, 0.0], [0.37, 0.06], [0.405, 0.22], [0.425, 0.4], [0.43, 0.55], [0.425, 0.7], [0.405, 0.88],
	[0.37, 1.04], [0.36, 1.1], [0.34, 1.1]]


static func _barrel_radius(h: float) -> float:
	var y := h * 1.1
	for i in BARREL_PROFILE.size() - 1:
		var a: Array = BARREL_PROFILE[i]
		var b: Array = BARREL_PROFILE[i + 1]
		if y >= float(a[1]) and y <= float(b[1]) and float(b[1]) > float(a[1]):
			return lerpf(float(a[0]), float(b[0]), (y - float(a[1])) / (float(b[1]) - float(a[1])))
	return 0.4


## Staves, drawn once: twelve boards up the body in close warm browns with dark seams, a lit edge on each and a little grain; `lid` the
## lid's boards across instead, in a darker brown.
static func _stave_texture(lid := false) -> Texture2D:
	var key := "lid" if lid else "body"
	if _stave_cache.has(key):
		return _stave_cache[key]
	var rng := RandomNumberGenerator.new()
	rng.seed = 5
	var base := Color(0.62, 0.4, 0.26) if lid else Color(0.78, 0.52, 0.33)
	var seam := base.darkened(0.5)
	var body := '<rect width="256" height="256" fill="#%s"/>' % base.to_html(false)
	var n := 5 if lid else 12
	var w := 256.0 / float(n)
	for k in n:
		var c := base.lerp(base.darkened(0.2), rng.randf_range(0.0, 1.0))
		var x := float(k) * w
		if lid:
			body += '<rect x="0" y="%.1f" width="256" height="%.1f" fill="#%s"/>' % [x, w, c.to_html(false)]
			body += '<line x1="0" y1="%.1f" x2="256" y2="%.1f" stroke="#%s" stroke-width="4" opacity="0.7"/>' % [x, x, seam.to_html(false)]
		else:
			body += '<rect x="%.1f" y="0" width="%.1f" height="256" fill="#%s"/>' % [x, w, c.to_html(false)]
			body += '<line x1="%.1f" y1="0" x2="%.1f" y2="256" stroke="#%s" stroke-width="3" opacity="0.7"/>' % [x, x, seam.to_html(false)]
			body += '<line x1="%.1f" y1="0" x2="%.1f" y2="256" stroke="#%s" stroke-width="2" opacity="0.35"/>' % [x + 3.0, x + 3.0, c.lightened(0.3).to_html(false)]
			for g in 3:
				var gx := x + rng.randf_range(6.0, w - 4.0)
				body += '<path d="M%.1f,0 q%.1f,128 0,256" fill="none" stroke="#%s" stroke-width="1.2" opacity="0.3"/>' % [gx, rng.randf_range(-4.0, 4.0), c.darkened(0.3).to_html(false)]
	var svg := '<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">%s</svg>' % body
	var img := Image.new()
	img.load_svg_from_string(svg, 1.0)
	img.generate_mipmaps()
	var tex := ImageTexture.create_from_image(img)
	_stave_cache[key] = tex
	return tex


static var _stave_cache := {}


static func _crate(root: Node3D, at: Vector3, size: float, turn: float) -> void:
	_mesh(root, _box(Vector3.ONE * size), Materials.wood(Color(0.86, 0.66, 0.42)), at + Vector3(0, size / 2.0, 0), Vector3.ONE, Vector3(0, turn, 0))
	# The front: a frame of darker boards round its edge and an X brace across it (a crate is built, not a block).
	var plank := FighterView.toon(Color(0.68, 0.48, 0.3), false)
	var basis := Basis(Vector3.UP, deg_to_rad(turn))
	var face := at + basis * Vector3(0, size / 2.0, size / 2.0 + 0.01)
	var b := size * 0.12
	for e2 in [[Vector3(0, size / 2.0 - b / 2.0, 0), Vector3(size, b, 0.04)], [Vector3(0, -size / 2.0 + b / 2.0, 0), Vector3(size, b, 0.04)],
			[Vector3(size / 2.0 - b / 2.0, 0, 0), Vector3(b, size, 0.04)], [Vector3(-size / 2.0 + b / 2.0, 0, 0), Vector3(b, size, 0.04)]]:
		_mesh(root, _box(e2[1]), plank, face + basis * (e2[0] as Vector3), Vector3.ONE, Vector3(0, turn, 0))
	# Slats behind the frame (two board seams) and one diagonal brace (art review 6 round 34: a full X across a blank face is the
	# placeholder-crate symbol; a built crate is slats braced once).
	var seam := FighterView.toon(Color(0.6, 0.42, 0.26), false)
	for sy in [-size / 6.0, size / 6.0]:
		_mesh(root, _box(Vector3(size - b * 2.0, size * 0.025, 0.02)), seam, face + basis * Vector3(0, sy, -0.005), Vector3.ONE, Vector3(0, turn, 0))
	_mesh(root, _box(Vector3(size * 1.1, b * 0.9, 0.035)), plank, face + basis * Vector3(0, 0, 0.005), Vector3.ONE, Vector3(0, turn, 45.0))
	# Corner battens up the four vertical edges, standing proud of the sides: they break the block's hard edges into planes that catch
	# the light (a lit batten on the key side, a shaded one opposite), as a real crate's corner posts do.
	var post := Materials.wood(Color(0.74, 0.54, 0.34), Vector2(1, 2))
	for cx in [-1.0, 1.0]:
		for cz in [-1.0, 1.0]:
			_mesh(root, _box(Vector3(b * 0.9, size * 1.02, b * 0.9)), post, at + basis * Vector3(cx * (size / 2.0 - b * 0.3), size / 2.0, cz * (size / 2.0 - b * 0.3)),
				Vector3.ONE, Vector3(0, turn, 0))


static func _signpost(root: Node3D, at: Vector3, points: float) -> void:
	_mesh(root, _box(Vector3(0.14, 1.7, 0.14)), Materials.wood(Color(0.74, 0.54, 0.34)), at + Vector3(0, 0.85, 0))
	var board := Materials.wood(Color(0.9, 0.72, 0.48))
	_mesh(root, _box(Vector3(1.1, 0.34, 0.08)), board, at + Vector3(points * 0.3, 1.4, 0.08), Vector3.ONE, Vector3(0, 0, -4))
	_mesh(root, _box(Vector3(0.9, 0.3, 0.08)), board, at + Vector3(-points * 0.25, 1.0, 0.08), Vector3.ONE, Vector3(0, 0, 5))


const CANYON_ROCK := Color(0.82, 0.6, 0.5)
const BEACH_ROCK := Color(0.82, 0.76, 0.68)


## A rock: a ball with few sides, squashed and turned, so it reads as a faceted stone.
## `colour`: the stone, in the stage's own rock (art review 6 round 30: one cool lavender on every stage sat outside the canyon's and the
## beach's warm palettes).
static func _rock(root: Node3D, at: Vector3, size: float, rng: RandomNumberGenerator, colour := Color(0.74, 0.72, 0.8)) -> void:
	var s := Vector3(rng.randf_range(1.1, 1.4), rng.randf_range(0.6, 0.8), 1.0) * size
	_mesh(root, _sphere(1.0, 6), Materials.stone(colour), at + Vector3(0, s.y * 0.7, 0), s, Vector3(0, rng.randf_range(0, 60), rng.randf_range(-8, 8)))


## A mushroom: a pale stem under a round cap with white spots, with a tiny idle squash and stretch (see `Squish`).
static func _mushroom(parent: Node3D, at: Vector3, size: float, cap: Color) -> void:
	var root := Squish.new()
	root.position = at
	root.phase = at.x * 1.7
	parent.add_child(root)
	at = Vector3.ZERO
	_mesh(root, _cylinder(0.12 * size, 0.18 * size, 0.7 * size, 8), FighterView.toon(Color(0.98, 0.94, 0.86)), at + Vector3(0, 0.35 * size, 0))
	var top := _sphere(0.45 * size, 10)
	top.is_hemisphere = true
	top.height = 0.4 * size
	_mesh(root, top, FighterView.toon(cap), at + Vector3(0, 0.68 * size, 0))
	var spot := _unshaded(Color(1, 0.97, 0.95))
	for k in 3:
		var a := TAU * k / 3.0 + 0.4
		_mesh(root, _sphere(0.07 * size, 6), spot, at + Vector3(cos(a) * 0.22, 0.95, sin(a) * 0.22 + 0.1) * size)


static func _stump(root: Node3D, at: Vector3) -> void:
	_mesh(root, _cylinder(0.5, 0.62, 0.7, 9), Materials.bark(Color(0.6, 0.42, 0.28), Vector2(2, 1)), at + Vector3(0, 0.35, 0))
	_mesh(root, _cylinder(0.44, 0.44, 0.03, 9), FighterView.toon(Color(0.94, 0.8, 0.58), false), at + Vector3(0, 0.71, 0))


static func _lantern_post(root: Node3D, at: Vector3, glow: Color) -> void:
	var wood := Materials.wood(Color(0.5, 0.36, 0.3))
	_mesh(root, _box(Vector3(0.14, 2.2, 0.14)), wood, at + Vector3(0, 1.1, 0))
	_mesh(root, _box(Vector3(0.7, 0.1, 0.1)), wood, at + Vector3(0.3, 2.15, 0))
	_mesh(root, _cylinder(0.14, 0.18, 0.36, 6), _unshaded(glow), at + Vector3(0.58, 1.86, 0))
	_halo(root, at + Vector3(0.58, 1.86, 0.25), glow, 1.3)
	_mesh(root, _cylinder(0.0, 0.22, 0.14, 6), FighterView.toon(Color(0.3, 0.26, 0.3), false), at + Vector3(0.58, 2.1, 0))


static func _umbrella(root: Node3D, at: Vector3, colour: Color) -> void:
	_mesh(root, _cylinder(0.05, 0.05, 2.4, 6), FighterView.toon(Color(0.95, 0.95, 0.95)), at + Vector3(0, 1.2, 0), Vector3.ONE, Vector3(0, 0, 8))
	var top := at + Vector3(-0.17, 2.4, 0)
	# A striped canopy: the cloth in eight panels alternating its colour and white, a rib along each seam, a scalloped hem and a knob.
	var canopy := Node3D.new()
	canopy.position = top
	canopy.rotation_degrees = Vector3(0, 0, 8)
	root.add_child(canopy)
	var white := FighterView.toon(Color(0.98, 0.96, 0.92))
	var cloth := FighterView.toon(colour)
	_mesh(canopy, _canopy_mesh(1.5, 0.5, 8, 0), cloth, Vector3.ZERO)
	_mesh(canopy, _canopy_mesh(1.5, 0.5, 8, 1), white, Vector3.ZERO)
	for k in 8:
		var a := TAU * (float(k) + 0.5) / 8.0
		_mesh(canopy, _sphere(0.2, 8), cloth if k % 2 == 0 else white, Vector3(cos(a) * 1.4, -0.26, sin(a) * 1.4), Vector3(1.0, 0.55, 1.0))
	_mesh(canopy, _sphere(0.1, 8), white, Vector3(0, 0.28, 0))
	# A towel on the sand under it.
	_mesh(root, _box(Vector3(1.6, 0.03, 0.8)), FighterView.toon(Color(0.55, 0.8, 0.95), false), at + Vector3(0.6, 0.02, 0.4))


## Every other panel (`parity` 0 or 1) of a cone `radius` wide and `height` tall, its tip at `height` / 2 and its rim at -`height` / 2,
## cut into `panels` panels round it (a beach umbrella's striped canopy, one mesh per colour).
static func _canopy_mesh(radius: float, height: float, panels: int, parity: int) -> ArrayMesh:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var tip := Vector3(0, height / 2.0, 0)
	for k in panels:
		if k % 2 != parity:
			continue
		for j in 4:
			var a0 := TAU * (float(k) + float(j) / 4.0) / float(panels)
			var a1 := TAU * (float(k) + float(j + 1) / 4.0) / float(panels)
			var p0 := Vector3(cos(a0) * radius, -height / 2.0, sin(a0) * radius)
			var p1 := Vector3(cos(a1) * radius, -height / 2.0, sin(a1) * radius)
			for v in [tip, p0, p1]:
				st.set_uv(Vector2(v.x, v.z))
				st.add_vertex(v)
	st.generate_normals()
	return st.commit()


static func _life_ring(root: Node3D, at: Vector3) -> void:
	_mesh(root, _box(Vector3(0.12, 1.6, 0.12)), Materials.wood(Color(0.86, 0.7, 0.5)), at + Vector3(0, 0.8, -0.1))
	var ring := TorusMesh.new()
	ring.inner_radius = 0.28
	ring.outer_radius = 0.5
	ring.rings = 12
	ring.ring_segments = 6
	_mesh(root, ring, FighterView.toon(Color(1.0, 0.45, 0.42)), at + Vector3(0, 1.2, 0.05), Vector3.ONE, Vector3(90, 0, 0))


static func _bucket(root: Node3D, at: Vector3, colour: Color) -> void:
	_mesh(root, _cylinder(0.26, 0.2, 0.4, 8), FighterView.toon(colour), at + Vector3(0, 0.2, 0))
	_mesh(root, _cylinder(0.04, 0.1, 0.5, 5), FighterView.toon(Color(1.0, 0.82, 0.3)), at + Vector3(0.4, 0.12, 0.1), Vector3.ONE, Vector3(0, 0, 70))


static func _potted_plant(root: Node3D, at: Vector3) -> void:
	_mesh(root, _cylinder(0.36, 0.26, 0.6, 8), FighterView.toon(Color(0.86, 0.5, 0.42)), at + Vector3(0, 0.3, 0))
	var leaf := Materials.foliage(Color(0.4, 0.72, 0.5), Vector2(1, 1))
	for k in 5:
		var a := TAU * k / 5.0
		_mesh(root, _sphere(1.0, 6), leaf, at + Vector3(cos(a) * 0.25, 0.85, sin(a) * 0.2), Vector3(0.18, 0.55, 0.1), Vector3(0, -rad_to_deg(a), 30))


## A string of little lit bulbs sagging between two points.
static func _string_lights(root: Node3D, a: Vector3, b: Vector3) -> void:
	var cord := FighterView.toon(Color(0.25, 0.22, 0.3), false)
	var bulbs := [_unshaded(Color(1.0, 0.86, 0.5)), _unshaded(Color(1.0, 0.6, 0.75)), _unshaded(Color(0.6, 0.9, 1.0))]
	var n := 18
	var prev := a
	for k in range(1, n + 1):
		var f := float(k) / n
		var p := a.lerp(b, f) + Vector3(0, -1.2 * sin(f * PI), 0)
		var seg := _mesh(root, _box(Vector3(0.04, 0.04, prev.distance_to(p))), cord, (prev + p) / 2.0)
		seg.basis = Basis.looking_at(p - prev, Vector3.UP)
		if k < n:
			_mesh(root, _sphere(0.11, 6), bulbs[k % 3], p + Vector3(0, -0.12, 0))
			_halo(root, p + Vector3(0, -0.12, 0.15), (bulbs[k % 3] as StandardMaterial3D).albedo_color, 0.55)
		prev = p


## Leaf cards for a crown: for each mass [centre, radius, cards] (in `parent`'s space), that many cards about `card` units across scattered
## through it (more toward its front and its outside), each facing the camera within a few tens of degrees and turned at random, coloured
## from `greens` (deep shade to sunlit tips, by how high and how far forward the card sits). One MultiMesh per crown, drawn by
## shaders/toon_foliage.gdshader: each card knows its mass's middle, so its normals bend away from it and the mass shades as one round
## volume; it flutters in the wind.
## A crown's leaf cards: what share of the listed count is made, and how much larger each is (see `_leaf_cards`).
const CROWN_CARD_SHARE := 0.0
const CROWN_CARD_GROWTH := 1.35


static func _leaf_cards(parent: Node3D, masses: Array, greens: Array, card: float, _stage_rng: RandomNumberGenerator,
		tile := "leaf_cluster", rooted := false, core: Material = null) -> MultiMeshInstance3D:
	# Its own random numbers, seeded from where it is, so adding the cards leaves the rest of the stage's layout as it was.
	var rng := RandomNumberGenerator.new()
	rng.seed = hash(str(masses[0][0]) + tile)
	var mm := MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.use_colors = true
	mm.use_custom_data = true
	var quad := QuadMesh.new()
	quad.size = Vector2.ONE
	mm.mesh = quad
	# (A crown is its solid lobes, painted with round leaf clumps (the `foliage` tile), as a painted tree's masses are: an artistic choice
	# over pointed leaf cards, which read as noise at play distance. `CROWN_CARD_SHARE` brings cards back if a stage wants them. A tuft is
	# all cards.)
	var share := 1.0 if rooted else CROWN_CARD_SHARE
	var total := 0
	for m in masses:
		total += int(float(m[2]) * share) if not rooted else maxi(1, int(m[2]))
	mm.instance_count = total
	var i := 0
	# A solid core inside each mass (a little smaller than it), so the cards over its surface give the crown a leafy edge and texture
	# while the volume stays whole: no sky through the middle of a tree. All of a crown's cores are one mesh (one draw call).
	var cores: SurfaceTool = null
	var ball := _sphere(1.0, 14)
	if core != null:
		cores = SurfaceTool.new()
	for m in masses:
		var centre: Vector3 = m[0]
		var radius: float = m[1]
		if cores != null:
			cores.append_from(ball, 0, Transform3D(Basis.IDENTITY.scaled(Vector3.ONE * radius * (0.82 if rooted else 1.0)), centre))
		for k in (int(float(m[2]) * share) if not rooted else maxi(1, int(m[2]))):
			# A direction out from the middle, mostly toward the front (the back of a crown is never seen), placed about its surface.
			var dir := Vector3(rng.randf_range(-1, 1), rng.randf_range(-0.8, 1), rng.randf_range(-0.2, 1)).normalized()
			if not rooted:
				# Round the edge and over the top, where a scallop shows against the sky.
				dir = Vector3(rng.randf_range(-1, 1), rng.randf_range(-0.2, 1), rng.randf_range(-0.6, 0.4)).normalized()
			var pos := centre + dir * radius * (rng.randf_range(0.72, 1.02) if rooted else rng.randf_range(0.9, 1.0))
			var s := card * rng.randf_range(0.8, 1.2) * (1.0 if rooted else CROWN_CARD_GROWTH)
			var b := Basis.from_euler(Vector3(rng.randf_range(-0.5, 0.5), rng.randf_range(-0.6, 0.6), rng.randf_range(0, TAU))).scaled(Vector3.ONE * s)
			if rooted:
				# A tuft stands upright on the ground, facing the camera give or take.
				b = Basis.from_euler(Vector3(0, rng.randf_range(-0.5, 0.5), rng.randf_range(-0.12, 0.12))).scaled(Vector3.ONE * s)
				pos = centre + Vector3(0, s * 0.5, 0)
			mm.set_instance_transform(i, Transform3D(b, pos))
			var mid := b.inverse() * ((centre - Vector3(0, s, s * 0.5) if rooted else centre) - pos)
			mm.set_instance_custom_data(i, Color(mid.x, mid.y, mid.z, rng.randf()))
			var light := clampf(0.45 + dir.y * 0.3 + dir.z * 0.3 + rng.randf_range(-0.15, 0.15), 0.0, 0.999)
			mm.set_instance_color(i, greens[1 + int(light * 1.999)] if not rooted else greens[rng.randi_range(0, greens.size() - 1)])
			i += 1
	if cores != null:
		var c := MeshInstance3D.new()
		c.mesh = cores.commit()
		# A crown takes the light the way a painted tree does: a crisp step from its lit top into a deep, cool underside, a warm rim.
		var crisp := core
		if not rooted and core is ShaderMaterial:
			crisp = (core as ShaderMaterial).duplicate()
			crisp.set_shader_parameter("ramp_softness", 0.08)
			crisp.set_shader_parameter("shadow_amount", 0.78)
			crisp.set_shader_parameter("rim_strength", 0.5)
		c.material_override = crisp
		parent.add_child(c)
	var mi := MultiMeshInstance3D.new()
	mi.multimesh = mm
	mi.material_override = _foliage_material(tile, rooted)
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	parent.add_child(mi)
	return mi


static var _foliage_cache := {}


static func _foliage_material(tile: String, rooted: bool) -> ShaderMaterial:
	var key := "%s|%s" % [tile, rooted]
	if _foliage_cache.has(key):
		return _foliage_cache[key]
	var m := ShaderMaterial.new()
	m.shader = FOLIAGE_SHADER
	m.set_shader_parameter("leaf_tex", SvgArt.texture("res://art/stage/%s.svg" % tile))
	m.set_shader_parameter("rooted", rooted)
	if rooted:
		m.set_shader_parameter("flutter", 0.08)
	_foliage_cache[key] = m
	return m


## Grass and flower tufts along the back of a grassy stage top (from `left` to `right` at height `top`, between `z_front` and
## `z_back`), stirring in the wind: on the Balanced and Best tiers only.
static func _tufts(root: Node3D, left: float, right: float, top: float, z_front: float, z_back: float, rng: RandomNumberGenerator,
		greens: Array, gap := Vector2(0.5, 1.1), size := 1.0) -> void:
	if VisualSettings.get_index("quality") == 0:
		return
	var spots := []
	# (Its own random numbers, so the stage's layout stays as it was.)
	var r := RandomNumberGenerator.new()
	r.seed = int(left * 31.0 + top * 7.0 + z_front * 13.0) + 5
	var x := left + 0.4
	while x < right - 0.4:
		spots.append([Vector3(x, top, r.randf_range(z_back, z_front)), 0.0, 1])
		x += r.randf_range(gap.x, gap.y)
	_leaf_cards(root, spots, greens, size, rng, "grass_tuft", true)


## A lawn's grass in depth (the meadow's and the grove's): a thick band of tall tufts behind the fighters, a lighter scatter of shorter
## ones across the middle of the lawn behind them, and a fringe of small tufts along the front edge, low enough that feet stay in view.
static func _lawn(root: Node3D, e: Array, rng: RandomNumberGenerator, greens: Array) -> void:
	_tufts(root, float(e[0]), float(e[1]), float(e[2]), -2.6, -5.8, rng, greens, Vector2(0.14, 0.32), 1.55)
	_tufts(root, float(e[0]), float(e[1]), float(e[2]), -3.4, -5.8, rng, greens, Vector2(0.2, 0.4), 1.2)
	_tufts(root, float(e[0]), float(e[1]), float(e[2]), -0.9, -2.6, rng, greens, Vector2(0.25, 0.55), 0.85)
	_tufts(root, float(e[0]), float(e[1]), float(e[2]), 0.35, 0.05, rng, greens, Vector2(0.3, 0.7), 0.42)


## A soft glow round a light (a lamp, a bulb): a see-through card facing the camera, its colour fading out from a near-white middle
## (shaders/fx.gdshader), so it blooms with the effects and is drawn after the finishing pass. `size` is its width in world units.
static func _halo(parent: Node3D, at: Vector3, colour: Color, size: float) -> MeshInstance3D:
	var quad := QuadMesh.new()
	quad.size = Vector2(size, size)
	var mi := MeshInstance3D.new()
	mi.mesh = quad
	mi.material_override = FxMaterial.get_material(false, 0, _halo_texture(colour), 0.6)
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	mi.position = at
	parent.add_child(mi)
	return mi


static var _halo_cache := {}


static func _halo_texture(colour: Color) -> Texture2D:
	var key := colour.to_html()
	if _halo_cache.has(key):
		return _halo_cache[key]
	var g := Gradient.new()
	g.set_color(0, Color(1.0, 1.0, 1.0, 0.95).lerp(colour, 0.25))
	g.set_color(1, Color(colour.r, colour.g, colour.b, 0.0))
	g.add_point(0.18, Color(colour.r, colour.g, colour.b, 0.75).lerp(Color.WHITE, 0.35))
	g.add_point(0.5, Color(colour.r, colour.g, colour.b, 0.22))
	var tex := GradientTexture2D.new()
	tex.gradient = g
	tex.fill = GradientTexture2D.FILL_RADIAL
	tex.fill_from = Vector2(0.5, 0.5)
	tex.fill_to = Vector2(1.0, 0.5)
	tex.width = 64
	tex.height = 64
	_halo_cache[key] = tex
	return tex


## Painted backdrop layers for a theme, if it has any: godot/art/backdrops/<theme>/layers.json lists pictures (PNG with transparency) to
## stand behind the stage as flat cards, each [{"image": "far_hills.png", "z": -110, "y": -8, "width": 260}, ...] (the card's bottom at
## `y`, its height from the picture's shape). The perspective camera gives the layers their parallax. See docs/ART_WORKFLOW.md.
static func painted_layers(root: Node3D, theme_name: String) -> int:
	var dir := "res://art/backdrops/%s/" % theme_name
	if not FileAccess.file_exists(dir + "layers.json"):
		return 0
	var layers = JSON.parse_string(FileAccess.get_file_as_string(dir + "layers.json"))
	if not (layers is Array):
		push_warning("%slayers.json is not a list of layers" % dir)
		return 0
	var placed := 0
	for layer in layers:
		if not (layer is Dictionary) or not ResourceLoader.exists(dir + str(layer.get("image", ""))):
			continue
		var tex := load(dir + str(layer.image)) as Texture2D
		if tex == null:
			continue
		var width := float(layer.get("width", 200.0))
		var height := width * float(tex.get_height()) / maxf(1.0, float(tex.get_width()))
		var m := StandardMaterial3D.new()
		m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA_SCISSOR if layer.get("cutout", false) else BaseMaterial3D.TRANSPARENCY_ALPHA
		m.albedo_texture = tex
		var q := QuadMesh.new()
		q.size = Vector2(width, height)
		var mi := _mesh(root, q, m, Vector3(float(layer.get("x", 0.0)), float(layer.get("y", -8.0)) + height * 0.5, float(layer.get("z", -100.0))))
		mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		mi.name = "Painted"
		placed += 1
	return placed


## The painted ridges of each theme, furthest first: {z, y (the card's foot), height, shape (0 hills, 1 mountains, 2 mesas, 3 cloud), seed,
## base, amplitude, frequency, tone (a colour the sky's horizon is mixed toward for the ridge), mix (how far), edge (the lit top)}.
const RIDGES := {
	"meadow": [{"z": -170.0, "y": -44.0, "height": 60.0, "shape": 1, "seed": 3.0, "base": 0.42, "amplitude": 0.5, "frequency": 4.5,
		# (More of its own blue and less haze, art review 6 round 31: at mix 0.55 the ridge was mostly horizon colour, a formless pale band
		# behind the crisp painted hills; found with `--pick`.)
		"tone": Color(0.36, 0.56, 0.84), "mix": 0.8, "edge": 0.7, "haze": 0.06}],
	"grove": [{"z": -165.0, "y": -34.0, "height": 52.0, "shape": 0, "seed": 8.0, "base": 0.5, "amplitude": 0.32, "frequency": 3.5,
		"tone": Color(0.5, 0.5, 0.84), "mix": 0.55, "edge": 0.5}],
	"sunset": [{"z": -175.0, "y": -36.0, "height": 58.0, "shape": 2, "seed": 5.0, "base": 0.4, "amplitude": 0.4, "frequency": 3.0,
		"tone": Color(0.42, 0.26, 0.5), "mix": 0.55, "edge": 0.4},
		{"z": -132.0, "y": -34.0, "height": 46.0, "shape": 2, "seed": 11.0, "base": 0.36, "amplitude": 0.36, "frequency": 2.6,
		"tone": Color(0.55, 0.26, 0.3), "mix": 0.62, "edge": 0.5}],
	"night": [{"z": -160.0, "y": -42.0, "height": 56.0, "shape": 1, "seed": 2.0, "base": 0.4, "amplitude": 0.45, "frequency": 4.0,
		"tone": Color(0.08, 0.1, 0.24), "mix": 0.55, "edge": 0.35}],
	# (The sea's horizon is the bank of piled cumulus modelled in `_ocean`, with the distant islands in front of it. A far coast drawn
	# here had its foot under the sea, so only a thin faded hump showed, a see-through lens across the clouds: removed, art review 5.)
	"ocean": [],
}


## One painted ridge behind the stage, from a RIDGES entry.
static func ridge(root: Node3D, t: Dictionary, spec: Dictionary) -> MeshInstance3D:
	var horizon: Color = t.sky_horizon
	var tone: Color = spec.tone
	var m := ShaderMaterial.new()
	m.shader = RIDGE_SHADER
	var top := horizon.lerp(tone, float(spec.mix))
	m.set_shader_parameter("top_colour", top)
	m.set_shader_parameter("foot_colour", top.lerp(horizon, 0.35))
	m.set_shader_parameter("edge_colour", horizon.lerp(Color.WHITE, 0.3))
	m.set_shader_parameter("edge_amount", float(spec.edge))
	m.set_shader_parameter("shape", int(spec.shape))
	m.set_shader_parameter("seed", float(spec.seed))
	m.set_shader_parameter("base", float(spec.base))
	m.set_shader_parameter("amplitude", float(spec.amplitude))
	m.set_shader_parameter("frequency", float(spec.frequency))
	m.set_shader_parameter("haze_colour", horizon)
	m.set_shader_parameter("haze", float(spec.get("haze", 0.14)))
	m.set_shader_parameter("flank", float(spec.get("flank", 0.8 if int(spec.shape) == 2 else 0.0)))
	m.set_shader_parameter("aspect", float(spec.height) / 520.0)
	var q := QuadMesh.new()
	q.size = Vector2(520.0, float(spec.height))
	var mi := _mesh(root, q, m, Vector3(0, float(spec.y) + float(spec.height) * 0.5, float(spec.z)))
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	mi.name = "Ridge"
	return mi


## The fog layers of each theme: [z, bottom y, height, strength].
const MIST := {
	"meadow": [[-56.0, -24.0, 12.0, 0.16], [-86.0, -26.0, 16.0, 0.24]],
	"grove": [[-60.0, -24.0, 12.0, 0.08], [-90.0, -26.0, 16.0, 0.14]],
	"sunset": [[-60.0, -26.0, 10.0, 0.07], [-95.0, -24.0, 16.0, 0.13]],
	"ocean": [[-130.0, -10.0, 6.0, 0.08]],
	"night": [[-70.0, -26.0, 18.0, 0.4]],
	"city": [[-80.0, -30.0, 24.0, 0.35]],
}


static func _mist(root: Node3D, colour: Color, z: float, bottom: float, height: float, strength: float) -> void:
	var g := Gradient.new()
	g.set_color(0, Color(colour, strength))
	g.set_color(1, Color(colour, 0.0))
	g.add_point(0.35, Color(colour, strength * 0.75))
	var tex := GradientTexture2D.new()
	tex.gradient = g
	tex.fill_from = Vector2(0.5, 1.0)
	tex.fill_to = Vector2(0.5, 0.0)
	tex.width = 4
	tex.height = 64
	var m := StandardMaterial3D.new()
	m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	m.albedo_texture = tex
	m.disable_fog = true
	var q := QuadMesh.new()
	q.size = Vector2(500.0, height)
	var mi := _mesh(root, q, m, Vector3(0, bottom + height * 0.5, z))
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	mi.name = "Mist"


## Motes: soft glowing specks (pollen, seeds, petals of light) drifting slowly over the stage `e` ([left, right, top, bottom]), mostly
## behind the fighters, in `colour`; they bloom a little (FxMaterial). Presentation only: fewer with the Effects setting, none on Fast.
static func _motes(root: Node3D, colour: Color, e: Array) -> void:
	if VisualSettings.get_index("quality") == 0:
		return
	var p := GPUParticles3D.new()
	p.name = "motes"
	p.amount = maxi(8, int(MOTE_COUNT * VisualSettings.effects()))
	p.lifetime = 9.0
	p.preprocess = 9.0
	p.randomness = 1.0
	var w := float(e[1]) - float(e[0]) + 16.0
	var pm := ParticleProcessMaterial.new()
	pm.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_BOX
	pm.emission_box_extents = Vector3(w / 2.0, 5.0, 7.0)
	pm.direction = Vector3(0.3, 1.0, 0.0)
	pm.spread = 60.0
	pm.initial_velocity_min = 0.08
	pm.initial_velocity_max = 0.3
	pm.gravity = Vector3(0.0, 0.03, 0.0)
	pm.turbulence_enabled = true
	pm.turbulence_noise_strength = 0.6
	pm.turbulence_noise_scale = 6.0
	pm.turbulence_noise_speed_random = 0.4
	pm.scale_min = 0.5
	pm.scale_max = 1.3
	# Fade in and out over each mote's life, so none pops.
	var fade := Gradient.new()
	fade.set_color(0, Color(colour, 0.0))
	fade.set_color(1, Color(colour, 0.0))
	fade.add_point(0.2, Color(colour, 0.85))
	fade.add_point(0.8, Color(colour, 0.85))
	var ramp := GradientTexture1D.new()
	ramp.gradient = fade
	pm.color_ramp = ramp
	p.process_material = pm
	var quad := QuadMesh.new()
	quad.size = Vector2(0.1, 0.1)
	p.draw_pass_1 = quad
	p.material_override = FxMaterial.get_material(false, 0, _soft_dot(), 0.6)
	p.position = Vector3((float(e[0]) + float(e[1])) / 2.0, float(e[2]) + 4.0, -5.0)
	p.visibility_aabb = AABB(Vector3(-w, -12, -14), Vector3(w * 2.0, 24, 28))
	p.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	root.add_child(p)


const MOTE_COUNT := 48


## Light shafts: a few long soft beams slanting down across the far hills behind the stage `e`, added faintly onto what is behind them
## (they sit behind the stage, so the fighters and the playing surface are never washed by them).
static func _light_shafts(root: Node3D, colour: Color, e: Array) -> void:
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.blend_mode = BaseMaterial3D.BLEND_MODE_ADD
	mat.cull_mode = BaseMaterial3D.CULL_DISABLED
	mat.no_depth_test = false
	mat.albedo_color = Color(colour, SHAFT_STRENGTH)
	mat.albedo_texture = _soft_dot()
	var cx := (float(e[0]) + float(e[1])) / 2.0
	var y := float(e[2])
	for k in 5:
		var quad := QuadMesh.new()
		quad.size = Vector2(3.0 + 1.6 * (k % 3), 46.0)
		var mi := MeshInstance3D.new()
		mi.mesh = quad
		mi.material_override = mat
		mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		mi.position = Vector3(cx - 30.0 + k * 14.0, y + 6.0, -22.0 - k * 4.0)
		mi.rotation_degrees = Vector3(0, 0, 28.0)
		root.add_child(mi)


## (0.05, art review 6 round 24: at 0.1 the shafts laid a pink veil over the grove's middle distance, the palest stage left.)
const SHAFT_STRENGTH := 0.05


static var _dot: GradientTexture2D = null


## A soft round spot, white in the middle fading to nothing at the edge.
static func _soft_dot() -> GradientTexture2D:
	if _dot == null:
		var g := Gradient.new()
		g.set_color(0, Color(1, 1, 1, 1))
		g.set_color(1, Color(1, 1, 1, 0))
		_dot = GradientTexture2D.new()
		_dot.gradient = g
		_dot.fill = GradientTexture2D.FILL_RADIAL
		_dot.fill_from = Vector2(0.5, 0.5)
		_dot.fill_to = Vector2(0.5, 0.0)
		_dot.width = 64
		_dot.height = 64
	return _dot


## Painted cumulus (shaders/cloud.gdshader, lit from the stage's sun): each cloud a heap of lumpy puffs on a flat base, of three kinds
## so the sky is not one stamp repeated: a domed heap with smaller puffs piled on its crown, a long low bank, and a tall tower. No ink
## outline: they are far, and the haze softens them.
static func _clouds(root: Node3D, rng: RandomNumberGenerator, count: int, color: Color, y_range: Vector2, z_range := Vector2(-120, -85),
		kinds := [0, 1, 2], scale := 1.0, x_range := Vector2(-90, 90), look := {}) -> void:
	var puff := _sphere(1.0, 32)
	for k in count:
		var cloud := Drift.new()
		cloud.speed = rng.randf_range(0.6, 1.6)
		cloud.position = Vector3(rng.randf_range(x_range.x, x_range.y), rng.randf_range(y_range.x, y_range.y), rng.randf_range(z_range.x, z_range.y))
		root.add_child(cloud)
		var mat := ShaderMaterial.new()
		mat.shader = CLOUD_SHADER
		mat.set_shader_parameter("colour", color)
		mat.set_shader_parameter("base_y", cloud.position.y)
		# (A stage's own exposure and shade depth, `look`: under the beach's noon light the default washed every tone to white.)
		for key in look:
			mat.set_shader_parameter(key, look[key])
		var kind: int = kinds[k % kinds.size()]
		var size := rng.randf_range(0.7, 1.15) * scale
		mat.set_shader_parameter("belly_height", 2.2 * size)
		# [puffs, how wide each, how tall the middle, how much the middle rises over the ends]
		var shape: Array = [[rng.randi_range(5, 7), 2.7, 4.8, 2.4], [rng.randi_range(7, 9), 2.9, 3.2, 1.0], [rng.randi_range(4, 5), 2.3, 6.2, 3.4]][kind]
		var puffs: int = shape[0]
		var span := float(puffs) * float(shape[1]) * size
		var crowns := []
		for b in puffs:
			var u := (float(b) + 0.5) / puffs * 2.0 - 1.0
			# Taller toward the middle, so the heap domes; every puff sits on the same base line.
			var r := (float(shape[2]) - absf(u) * float(shape[3])) * size * rng.randf_range(0.85, 1.15)
			var at := Vector3(u * span * 0.5 + rng.randf_range(-0.4, 0.4), r * 0.55, rng.randf_range(-1.5, 1.5))
			_mesh(cloud, puff, mat, at, Vector3(r, r * 0.85, r * 0.8))
			crowns.append([at, r])
		# Smaller puffs piled on the crown of a heap or a tower (the cauliflower top of a summer cumulus).
		if kind != 1:
			for c in crowns.slice(1, crowns.size() - 1):
				var r2: float = float(c[1]) * rng.randf_range(0.45, 0.6)
				var at2: Vector3 = c[0] + Vector3(rng.randf_range(-0.6, 0.6), float(c[1]) * 0.62, rng.randf_range(0.2, 1.0))
				_mesh(cloud, puff, mat, at2, Vector3(r2, r2 * 0.9, r2 * 0.85))
		# The flat base: a wide squashed puff under them all.
		_mesh(cloud, puff, mat, Vector3(0, 0.6 * size, 0.3), Vector3(span * 0.6, 1.3 * size, 3.4 * size))


static func _birds(root: Node3D, rng: RandomNumberGenerator, colour: Color) -> void:
	for k in 2:
		var flock := Flock.new()
		flock.position = Vector3(rng.randf_range(-100, 60), rng.randf_range(16, 30), rng.randf_range(-75, -60))
		flock.speed = rng.randf_range(3.0, 5.5)
		root.add_child(flock)
		flock.build(rng.randi_range(3, 5), colour, rng)


## Rolling hills in two rows (a nearer one at z -72 and a farther one at -100), each one continuous piece of land: round hilltops (each
## the top of an ellipse, as wide as `w` and as tall as `h`, over a base at y -22) melted together where they meet, over a lower valley
## floor between them, and the land running on down out of sight below, so however far the camera pulls back the hills are ground, not
## shapes hanging in the sky. `hill_top` gives a row's height at any x (to stand things on it).
static func _hills(root: Node3D, rng: RandomNumberGenerator, near: Color, far: Color) -> void:
	_hill_bumps = [[], []]
	for k in 8:
		var is_far := k % 2 == 1
		var w := rng.randf_range(26.0, 44.0)
		var x := -105.0 + k * 30.0 + rng.randf_range(-8, 8)
		var h := rng.randf_range(10.0, 18.0)
		(_hill_bumps[1 if is_far else 0] as Array).append([x, w, h, -22.0])
	_land_band(root, _hill_bumps[1], -17.0, -100.0, far)
	_land_band(root, _hill_bumps[0], -17.0, -72.0, near)


## The bumps of the two rows of hills ([centre x, half width, height, base y]; near row first), set by `_hills`.
static var _hill_bumps: Array = [[], []]


## The height of the land in hill row `row` (0 near, 1 far) at `x`.
static func hill_top(x: float, row := 0) -> float:
	return _land_top(_hill_bumps[row], -17.0, x)


## The top of a band of land with `bumps` ([centre x, half width, height, base y]: an elliptical hilltop each) over a floor at `floor_y`,
## each bump blended into the floor and its neighbours with a smooth maximum (so the valleys are rounded, not creased).
static func _land_top(bumps: Array, floor_y: float, x: float) -> float:
	var y := floor_y
	for b in bumps:
		var dx := (x - float(b[0])) / float(b[1])
		if absf(dx) < 1.0:
			var top := float(b[3]) + float(b[2]) * sqrt(1.0 - dx * dx)
			var k := 3.0
			var m := clampf(0.5 + 0.5 * (top - y) / k, 0.0, 1.0)
			y = lerpf(y, top, m) + k * m * (1.0 - m)
	return y


## A band of land facing the camera at depth `z`: its top follows `_land_top` across x from -240 to 240, and it runs down 180 units below
## that, so it is ground all the way out of view. Its normals tip up toward the top edge (lit like the round of a hill) and face the camera
## further down, where the haze takes it; one mesh, toon-shaded in `colour`.
static func _land_band(root: Node3D, bumps: Array, floor_y: float, z: float, colour: Color, step := 2.0, roll := 1.0, profile: Array = ROLL_PROFILE) -> void:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	# The cross-section, back to front: [drop below the crest, depth from z]. With `roll` the land is a rounded ridge: it rolls away
	# behind the crest and over toward the camera in front, then falls away (`ROLL_PROFILE`, scaled), so the key light gives it a lit
	# top, a cel-shaded front and a real silhouette in depth; without, it is a flat front (the canopy's shade band).
	var rows: Array = []
	if roll > 0.0:
		for r in profile:
			rows.append([float(r[0]), float(r[1]) * roll])
	else:
		for d in [0.0, 0.4, 1.2, 2.6, 5.0, 9.0, 18.0, 40.0, 90.0, 180.0]:
			rows.append([d, 0.0])
	var cols: Array = []
	var x := -240.0
	while x <= 240.0:
		var top := _land_top(bumps, floor_y, x)
		var col: Array = []
		for r in rows:
			col.append(Vector3(x, top - float(r[0]), z + float(r[1])))
		cols.append(col)
		x += step
	for i in cols.size() - 1:
		for j in rows.size() - 1:
			for c in [[i, j], [i + 1, j], [i + 1, j + 1], [i, j], [i + 1, j + 1], [i, j + 1]]:
				st.add_vertex(cols[c[0]][c[1]])
	st.generate_normals()
	_mesh(root, st.commit(), _soft(colour), Vector3.ZERO)


## A rolled band's cross-section (see `_land_band`): [drop below the crest, depth from the band's z], from behind the crest to the foot.
const ROLL_PROFILE := [[3.0, -6.0], [1.0, -3.6], [0.2, -1.6], [0.0, 0.0], [0.25, 1.3], [0.9, 2.5], [1.9, 3.4], [3.3, 4.0], [5.2, 4.35],
	[9.0, 4.5], [18.0, 4.55], [40.0, 4.6], [90.0, 4.6], [180.0, 4.6]]


## The near land's cross-section: the same roll over the crest, then a valley floor sloping down toward the camera (as a hillside seen
## from above), flattening out far below, so what is laid on it (fields, hedges, a farm) recedes in real perspective.
const VALLEY_PROFILE := [[3.0, -6.0], [1.0, -3.6], [0.2, -1.6], [0.0, 0.0], [0.25, 1.3], [0.9, 2.5], [1.9, 3.6], [3.3, 5.0], [5.2, 7.2],
	[9.0, 11.5], [18.0, 21.0], [32.0, 33.0], [50.0, 40.0], [90.0, 42.0], [180.0, 42.0]]


## How far in front of a rolled band's z its face is, `d` below the crest (to lay things on the face), for `profile`.
static func roll_z(d: float, profile: Array = ROLL_PROFILE) -> float:
	for k in range(1, profile.size()):
		var a: Array = profile[k - 1]
		var b: Array = profile[k]
		if d <= float(b[0]) and float(b[0]) > float(a[0]):
			return lerpf(float(a[1]), float(b[1]), (d - float(a[0])) / (float(b[0]) - float(a[0])))
	return float(profile[-1][1])


## A low band of land nearer the stage (at depth `z`, its top rolling gently round `y`), that the nearer trees stand on; deterministic
## (no random draws), so adding it leaves the rest of a theme's layout where it was. `fore_top` gives its height at x.
static func _fore_band(root: Node3D, z: float, y: float, colour: Color, profile: Array = VALLEY_PROFILE) -> void:
	var bumps: Array = []
	var x := -230.0
	var k := 0
	while x <= 230.0:
		bumps.append([x, 16.0 + 6.0 * absf(sin(k * 1.7)), 2.2 + 1.6 * absf(sin(k * 2.3 + 0.5)), y - 1.0])
		x += 22.0 + 8.0 * absf(sin(k * 0.9))
		k += 1
	_fore_bumps = bumps
	_land_band(root, bumps, y - 1.0, z, colour, 2.0, 1.0, profile)


static var _fore_bumps: Array = []


## A forest seen from above, filling the low land in front of `z` from about `y` down: rows of round crowns (balls of painted leaf
## clumps, every third a little lighter) over a dark band of shade between them, each row a little lower, nearer and larger than the one
## behind it so the rows overlap like a canopy falling away down a valley, in `colours` (one per row, cycling). Deterministic.
static func _canopy(root: Node3D, z: float, y: float, colours: Array) -> void:
	var rows := 6
	var fill := (colours[0] as Color).darkened(0.35)
	for r in rows:
		var row_y := y - 1.0 - float(r * r) * 0.9 - float(r) * 2.6
		var size := 1.8 + float(r) * 0.7
		var rz := z + float(r) * 1.4
		# The shade between the crowns: a band behind them, from just under their tops on down.
		_land_band(root, [], row_y + size * 0.2, rz - size * 0.6, fill, 2.0, 0.0)
		# (The two nearest rows inked like the trees by the stage; further rows without, as distance takes the line.)
		var inked := r < 2
		var leaf := Materials.foliage(colours[r % colours.size()], Vector2(2, 2), inked)
		var leaf_lit := Materials.foliage((colours[r % colours.size()] as Color).lightened(0.12), Vector2(2, 2), inked)
		var x := -150.0 + float(r) * 1.3
		var k := 0
		# Crowns, not equal puffs: each a big mass with two or three smaller lobes round its shoulders, of very different sizes, now and
		# then a tall one standing above the row, so the canopy reads as trees grouped into a forest.
		while x <= 150.0:
			var n := absf(sin(k * 2.3 + r)) * 0.6 + absf(sin(k * 0.53 + r * 1.9)) * 0.4
			var w := size * (0.75 + 0.9 * n)
			var tall := 1.35 if absf(sin(k * 1.13 + r * 3.7)) > 0.93 else 1.0
			var h := w * (0.85 + 0.25 * absf(sin(k * 1.7 + r * 0.6))) * tall
			var jy := size * 0.35 * sin(k * 1.37 + r * 2.1) + (h - w) * 0.5
			var jz := 0.8 * sin(k * 3.1 + r)
			var main := leaf_lit if (k + r) % 3 == 0 else leaf
			var c := Vector3(x, row_y + jy, rz + jz)
			_mesh(root, _sphere(1.0, 12), main, c, Vector3(w, h, w * 0.7))
			var lobes := 2 + int(absf(sin(k * 4.1 + r)) * 1.99)
			for li in lobes:
				var side := -1.0 if li % 2 == 0 else 1.0
				var lw := w * (0.45 + 0.15 * absf(sin(k + li * 2.7)))
				_mesh(root, _sphere(1.0, 10), leaf if main == leaf_lit else leaf_lit,
					c + Vector3(side * w * (0.7 + 0.1 * li), -h * 0.15 + h * 0.35 * float(li / 2), 0.25), Vector3(lw, lw * 0.9, lw * 0.7))
			x += w * (1.6 + 0.5 * absf(sin(k * 0.71 + r * 1.3)))
			k += 1


## Patchwork farmland painted down the face of the near band (`_fore_band`, at depth `z` with its top round `y`), as the low land below a
## hill looks from up on it: rows of fields in a few greens and a ripe wheat gold, divided by dark rounded hedgerows that follow the land's
## roll near the top and flatten further down (further away), a few round bushes along the hedges, and a pale dirt lane winding down
## through them. Deterministic (no random draws). One mesh per colour, each a hair in front of the band.
static func _fields(root: Node3D, z: float, y: float, colours: Array, hedge_colour: Color) -> void:
	var hedges := [2.2, 5.4, 9.6, 15.0, 22.0, 31.0, 43.0]
	var y_ref := y
	var mean := y + 0.5
	# Each hedge follows the land's roll near the top and a slower swell of its own further down; and the fields fan out from a point
	# above the middle as they come down (`at`: from across `u` and down `d` to a point on the face), as rows of fields do in perspective.
	var line := func(x: float, d: float) -> float:
		var k := exp(-d / 12.0)
		return lerpf(mean, fore_top(x, y), k) - d + sin(x * 0.045 + d * 0.7) * d * 0.07
	var at := func(u: float, d: float) -> Vector3:
		var x := u * (1.0 + d * 0.03)
		var ly: float = line.call(x, d)
		return Vector3(x, ly, z + roll_z(fore_top(x, y_ref) - ly, VALLEY_PROFILE) + 0.12)
	var tools: Array = []
	for c in colours.size() + 2:
		var st := SurfaceTool.new()
		st.begin(Mesh.PRIMITIVE_TRIANGLES)
		st.set_normal(Vector3(0.0, 0.25, 1.0).normalized())
		tools.append(st)
	var hedge_st: SurfaceTool = tools[colours.size()]
	var lane_st: SurfaceTool = tools[colours.size() + 1]
	# Furrows: rows of crop drawn across some fields, a shade darker than the field (green crops, then the wheat).
	var furrow_green := SurfaceTool.new()
	furrow_green.begin(Mesh.PRIMITIVE_TRIANGLES)
	var furrow_gold := SurfaceTool.new()
	furrow_gold.begin(Mesh.PRIMITIVE_TRIANGLES)
	var quad := func(st: SurfaceTool, a: Vector3, b: Vector3, c: Vector3, d: Vector3) -> void:
		# (Wound to face the camera: given bottom-left, bottom-right, top-right, top-left.)
		for v in [a, c, b, a, d, c]:
			st.set_normal(Vector3(0.0, 0.25, 1.0).normalized())
			st.add_vertex(v)
	var side_hedges: Array = []
	# The fields: between each pair of hedges, patches of a few to a few tens of units, their colours picked by a hash.
	for row in hedges.size() - 1:
		var d0: float = hedges[row]
		var d1: float = hedges[row + 1]
		var x := -240.0 + 9.0 * row
		var k := 0
		while x < 240.0:
			var w := 12.0 + 18.0 * absf(sin(row * 3.1 + k * 1.93)) + d0 * 0.4
			var pick := int(absf(sin(row * 7.7 + k * 4.1)) * 1000.0) % colours.size()
			var st: SurfaceTool = tools[pick]
			var step := 3.0
			var front := Vector3(0, 0, 0.05)
			var xa := x
			while xa < x + w:
				var xb := minf(xa + step, x + w)
				quad.call(st, at.call(xa, d1) + front, at.call(xb, d1) + front, at.call(xb, d0) + front, at.call(xa, d0) + front)
				xa = xb
			# (Crop rows are in the field tile now.)
			if false:
				var fst := furrow_gold if pick == colours.size() - 1 else furrow_green
				var gap := 1.3 + 0.05 * d0
				var dr := d0 + gap
				while dr < d1 - gap * 0.5:
					var xa2 := x + 0.6
					while xa2 < x + w - 0.6:
						var xb2 := minf(xa2 + 3.0, x + w - 0.6)
						var lift := Vector3(0, 0.16, 0.2)
						var drop := Vector3(0, -0.16, 0.2)
						quad.call(fst, at.call(xa2, dr) + drop, at.call(xb2, dr) + drop, at.call(xb2, dr) + lift, at.call(xa2, dr) + lift)
						xa2 = xb2
					dr += gap
			# A hedge down the side between this field and the next: a thin shade line under a few bushes along it.
			var side := Vector3(0.18, 0, 0)
			var hz := Vector3(0, 0, 0.08)
			quad.call(hedge_st, at.call(x + w, d1) - side + hz, at.call(x + w, d1) + side + hz, at.call(x + w, d0) + side + hz, at.call(x + w, d0) - side + hz)
			side_hedges.append([x + w, d0, d1])
			x += w
			k += 1
	# The hedgerows across: a dark band along each line, thicker nearer (higher up the face).
	for d in hedges:
		var thick := lerpf(0.9, 0.35, float(d) / 43.0)
		var xa := -240.0
		while xa < 240.0:
			var xb := xa + 3.0
			var up := Vector3(0, thick * 0.5, 0.09)
			var down := Vector3(0, -thick * 0.5, 0.09)
			quad.call(hedge_st, at.call(xa, d) + down, at.call(xb, d) + down, at.call(xb, d) + up, at.call(xa, d) + up)
			xa = xb
	# A lane winding down from the top through the fields, narrowing as it goes.
	var prev := Vector2.INF
	var dd := 0.0
	while dd < 43.0:
		var cx := 30.0 + 7.0 * sin(dd * 0.16) + 3.0 * sin(dd * 0.41)
		var half := lerpf(1.3, 0.35, dd / 43.0)
		var p3: Vector3 = at.call(cx, dd)
		cx = p3.x
		var here := Vector2(cx, p3.y)
		if prev != Vector2.INF:
			var ph := lerpf(1.3, 0.35, (dd - 1.0) / 43.0)
			quad.call(lane_st, Vector3(cx - half, here.y, z + 0.12), Vector3(cx + half, here.y, z + 0.12), Vector3(prev.x + ph, prev.y, z + 0.12),
				Vector3(prev.x - ph, prev.y, z + 0.12))
		prev = here
		dd += 1.0
	# The fields painted as the stage's own ground is (the grass tile with its detail, laid in world space), not flat fills, so the valley
	# reads at the stage's finish; the hedgerow bands under the bushes stay plain, as their shade.
	var materials: Array = []
	for c in colours:
		# (Laid big and light: the tile only breaks up the fill, each field keeps its own colour.)
		# (A painted field of crop rows (godot/art/stage/field.svg) in the field's own colour, laid in world space at the size of the farm,
		# with the soil detail over it.)
		var fm := Materials.world("field", c, 6.0, false).duplicate() as ShaderMaterial
		fm.set_shader_parameter("world_vary", 0.05)
		Materials.set_detail(fm, "soil", 0.25, 5.0)
		materials.append(fm)
	materials.append(_soft(hedge_colour.darkened(0.12)))
	materials.append(_soft(Color(0.9, 0.8, 0.58)))
	for i in tools.size():
		_mesh(root, (tools[i] as SurfaceTool).commit(), materials[i], Vector3.ZERO)
	_mesh(root, furrow_green.commit(), _soft((colours[0] as Color).darkened(0.16)), Vector3.ZERO)
	_mesh(root, furrow_gold.commit(), _soft((colours[colours.size() - 1] as Color).darkened(0.14)), Vector3.ZERO)
	# A farm beside the lane: a red barn and a white farmhouse, a little in front of the face.
	# (Smaller than the cottages on the hills: they are nearer, but the farm should read as far below the stage.)
	var farm := Node3D.new()
	farm.position = at.call(-30.0, 5.0) + Vector3(0, 0, 0.6)
	farm.scale = Vector3.ONE * 0.6
	root.add_child(farm)
	_barn(farm, Vector3.ZERO)
	_cottage(farm, Vector3(8.0, 0.4, -0.3), Color(0.4, 0.46, 0.6), 1.0)
	merge_static(farm)
	# The hedgerows across: a row of round, leafy bushes along each line (smaller and closer set further down), on the dark band as their
	# shade, so a hedge has volume like the trees by the stage rather than being a drawn line.
	var leaf := Materials.foliage(hedge_colour, Vector2(2, 2), true)
	var leaf_lit := Materials.foliage(hedge_colour.lightened(0.1), Vector2(2, 2), true)
	for hi in hedges.size():
		var d: float = hedges[hi]
		var hr := lerpf(0.95, 0.4, d / 43.0)
		var bx := -130.0 + float(hi) * 1.7
		var j := 0
		while bx < 130.0:
			# Bushes of very different sizes, overlapping into one lumpy row, with now and then a gap or a tree standing out of it.
			var n := absf(sin(bx * 0.37 + d * 1.3)) * absf(sin(bx * 0.11 + d))
			var rr := hr * (0.6 + 0.9 * n)
			var b3: Vector3 = at.call(bx, d)
			if n > 0.08:
				_mesh(root, _sphere(1.0, 8), leaf if j % 4 != 0 else leaf_lit, b3 + Vector3(0, rr * 0.4, 0.25), Vector3(rr * 1.5, rr, 0.6))
			bx += rr * (1.1 + 0.6 * absf(sin(bx * 2.3)))
			j += 1
	for sh in side_hedges:
		var sd: float = sh[1]
		while sd < float(sh[2]):
			var rr := lerpf(0.75, 0.35, sd / 43.0) * (0.8 + 0.4 * absf(sin(sd * 1.9 + float(sh[0]))))
			_mesh(root, _sphere(1.0, 8), leaf, at.call(float(sh[0]), sd) + Vector3(0, rr * 0.4, 0.25), Vector3(rr * 1.2, rr, 0.6))
			sd += rr * 2.4
	# Round bushes dotted along the top hedges.
	var bush := Materials.foliage(hedge_colour.lightened(0.08), Vector2(2, 2), true)
	for j in 14:
		var bx := -150.0 + j * 23.0 + 6.0 * sin(j * 2.7)
		var bd: float = hedges[j % 3]
		var r := lerpf(1.3, 0.8, bd / 10.0)
		var b3: Vector3 = at.call(bx, bd)
		_mesh(root, _sphere(1.0, 10), bush, b3 + Vector3(0, r * 0.4, 0.3), Vector3(r * 1.3, r, 0.6))


static func fore_top(x: float, y: float) -> float:
	return _land_top(_fore_bumps, y - 1.0, x)


## A trunk as one mesh: rings up a gentle bow (`bend` sideways at most), tapering from `r_bottom` to `r_top` and flaring by `flare` at the
## foot; bark UVs run round (u) and up (v). One mesh, so its ink outline has no seams.
static func trunk_mesh(height: float, r_bottom: float, r_top: float, bend: float, flare: float) -> ArrayMesh:
	var key := "%.2f,%.2f,%.2f,%.2f,%.2f" % [height, r_bottom, r_top, bend, flare]
	if _trunk_cache.has(key):
		return _trunk_cache[key]
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var sides := 22
	var rings := 12
	var rows: Array = []
	for j in rings + 1:
		var v := float(j) / rings
		var r := lerpf(r_bottom, r_top, v) + flare * exp(-v * 9.0)
		var centre := Vector3(sin(v * PI * 0.9) * bend, v * height, 0)
		var row: Array = []
		for i in sides + 1:
			var a := TAU * i / sides
			var n := Vector3(sin(a), 0, cos(a))
			row.append([centre + n * r, n, Vector2(float(i) / sides, 1.0 - v)])
		rows.append(row)
	for j in rings:
		for i in sides:
			for c in [[j, i], [j + 1, i], [j + 1, i + 1], [j, i], [j + 1, i + 1], [j, i + 1]]:
				var vtx: Array = rows[c[0]][c[1]]
				st.set_normal(vtx[1])
				st.set_uv(vtx[2])
				st.add_vertex(vtx[0])
	var mesh := st.commit()
	_trunk_cache[key] = mesh
	return mesh


static var _trunk_cache := {}


## A row of soft tree shapes far back on the hills: a short trunk under a ball or two, all in one hazy colour, so they read as a
## distant wood rather than as things to look at.
static func _tree_line(root: Node3D, rng: RandomNumberGenerator, y: float, z: float, colour: Color) -> void:
	var leaf := _soft(colour)
	var trunk := _soft(colour.darkened(0.25))
	var x := -110.0
	while x < 110.0:
		var s := rng.randf_range(3.0, 5.0)
		var at := Vector3(x, y + rng.randf_range(-1.5, 1.5), z + rng.randf_range(-6, 6))
		_mesh(root, _cylinder(0.35 * s * 0.3, 0.45 * s * 0.3, s * 0.9, 6), trunk, at + Vector3(0, s * 0.45, 0))
		_mesh(root, _sphere(1.0, 10), leaf, at + Vector3(0, s * 1.3, 0), Vector3(s, s * 1.1, s))
		if rng.randf() < 0.5:
			_mesh(root, _sphere(1.0, 10), leaf, at + Vector3(s * 0.7, s * 0.95, -0.5), Vector3.ONE * s * 0.7)
		x += rng.randf_range(7.0, 12.0)


## Flowers and grass tufts in a few clusters near each end of the stage's top `e` = [left, right, top] (and none in the middle third),
## behind the fighters.
static func _flower_clusters(root: Node3D, e: Array, rng: RandomNumberGenerator, petals: Array, tuft_colour: Color) -> void:
	var heart := _unshaded(Color(1, 0.85, 0.3))
	var tuft := Materials.foliage(tuft_colour, Vector2(1, 1), false)
	var l: float = e[0]
	var r: float = e[1]
	var top: float = e[2]
	var third := (r - l) / 3.0
	for side in [-1.0, 1.0]:
		for c in 2:
			var cx: float = (l + rng.randf_range(0.8, third - 0.6)) if side < 0.0 else (r - rng.randf_range(0.8, third - 0.6))
			var cz := rng.randf_range(-4.6, -2.4)
			for k in rng.randi_range(3, 5):
				var p := Vector3(cx + rng.randf_range(-0.7, 0.7), top + 0.08, cz + rng.randf_range(-0.6, 0.6))
				_mesh(root, _sphere(0.13, 8), _unshaded(petals[k % petals.size()]), p)
				_mesh(root, _sphere(0.06, 6), heart, p + Vector3(0, 0.04, 0.08))
			for k in 3:
				var p := Vector3(cx + rng.randf_range(-0.9, 0.9), top + 0.1, cz + rng.randf_range(-0.7, 0.4))
				_mesh(root, _sphere(1.0, 6), tuft, p, Vector3(0.22, 0.3, 0.18) * rng.randf_range(0.8, 1.3))


## What hangs under the stage, below its lowest block `e` = [left, right, top, bottom], so the stage reads as a place (an island, a mesa,
## a rock in the sea, a building) rather than a brick floating in the air. All of it is set back `UNDER_RECESS` behind the blocks' front:
## a fighter under the stage passes in front of it, it never reads as ground to stand on or bump into, and the step in depth draws an
## ink line along the block's bottom, which stays the true edge of the solid ground. Its lower part fades into a bank of haze in the sky's
## horizon colour (`_under_haze`), so it sinks into the air below the stage the way far things do.
const UNDER_RECESS := 0.9
const STAGE_FRONT := 2.0


static func underside(root: Node3D, t: Dictionary, e: Array) -> void:
	var l: float = e[0]
	var r: float = e[1]
	var bottom: float = e[3]
	var front := STAGE_FRONT - UNDER_RECESS
	var rng := RandomNumberGenerator.new()
	rng.seed = 13
	match t.name:
		"sunset":
			# The stage is the top of a mesa: the column carries on down in the same strata, a little wider at its foot, into the haze.
			# (Narrower than the block at first, so the block reads as the mesa's cap; in shade, below the light.)
			var mesa := _hanging_mass(bottom, 34.0, l, r, front, 3.6, 0.06, 7, func(s: float) -> float: return 0.9 + 0.14 * s,
				func(_s: float) -> float: return 0.0)
			var col := _mesh(root, mesa, _under_material(_soil_material(t), 0.62), Vector3.ZERO)
			col.name = "Underside"
			col.set_instance_shader_parameter("front_top", e[2])
			_under_haze(root, t, bottom - 1.0, 6.0, 0.5)
			# The desert floor the mesa stands in (level with the far buttes' feet) and fallen rock heaped round its foot.
			var floor_y := DESERT_FLOOR
			# (Painted sand laid in world space with soil detail, not a flat fill, and a scatter of scrub and stones over it so the floor
			# reads as ground going off into the distance.)
			var sand_floor := Materials.world("sand", Color(0.9, 0.62, 0.44), 9.0, false)
			_mesh(root, _box(Vector3(400.0, 1.0, 180.0)), sand_floor, Vector3(0, floor_y - 0.5, -80.0))
			var scrub := Materials.foliage(Color(0.56, 0.6, 0.36), Vector2(2, 2), true)
			var pebble := Materials.stone(Color(0.78, 0.5, 0.4))
			for k in 40:
				var sx := rng.randf_range(-120.0, 120.0)
				var sz := rng.randf_range(-130.0, -14.0)
				if absf(sx) < r - l and sz > -20.0:
					continue
				var sc := rng.randf_range(0.5, 1.3) * lerpf(1.0, 1.8, -sz / 130.0)
				if k % 3 == 0:
					_mesh(root, _sphere(1.0, 7), pebble, Vector3(sx, floor_y + sc * 0.25, sz), Vector3(sc * 1.3, sc * 0.7, sc), Vector3(0, rng.randf_range(0, 90), 0))
				else:
					_mesh(root, _sphere(1.0, 8), scrub, Vector3(sx, floor_y + sc * 0.35, sz), Vector3(sc * 1.2, sc * 0.7, sc))
			var rubble := Materials.stone((t.soil_dark as Color).lerp(Color(0.9, 0.62, 0.44), 0.45))
			for k in 9:
				var x := lerpf(l - 2.5, r + 2.5, (k + rng.randf_range(0.2, 0.8)) / 9.0)
				var size := rng.randf_range(0.6, 1.2) * (1.5 if k == 0 or k == 8 else 1.0)
				_mesh(root, _sphere(1.0, 8), rubble, Vector3(x, floor_y + size * 0.2, front - 2.0 + rng.randf_range(-1.5, 0.5)),
					Vector3(size * 1.3, size, size), Vector3(0, rng.randf_range(0, 90), rng.randf_range(-12, 12)))
		"ocean":
			# A rock shelf the island stands on, a little wider at the waterline, with boulders where the sea meets it.
			var shelf := _hanging_mass(bottom, 3.0, l, r, front, 3.4, 0.14, 3, func(s: float) -> float: return 1.0 + 0.06 * s,
				func(_s: float) -> float: return 0.0)
			var rock := Materials.stone((t.soil_dark as Color).darkened(0.15), Vector2(5, 1))
			_mesh(root, shelf, rock, Vector3.ZERO).name = "Underside"
			# (The sea is a unit below the block: the boulders rise from it to the block's foot, so it stands on rock, foam round them.)
			# Sand banked up against each end of the wall, spilling from about half its height down into the sea: the ends slope rather
			# than stop in square corners, so the island's outline is a shore, not a box.
			var sand_bank := _surface(t.grass, "sand", 5.0)
			var bh := (e[2] as float) - bottom
			for side in [-1.0, 1.0]:
				var ex: float = r if side > 0.0 else l
				_mesh(root, _sphere(1.0, 16), sand_bank, Vector3(ex + side * 0.4, bottom - 0.6, front - 1.0), Vector3(2.6, bh * 0.55, 2.4))
				_mesh(root, _sphere(1.0, 12), sand_bank, Vector3(ex + side * 1.8, bottom - 0.9, front - 0.6), Vector3(1.6, bh * 0.25, 1.8))
			var boulder := Materials.stone((t.soil as Color).darkened(0.12))
			for k in 8:
				var x := lerpf(l - 0.4, r + 0.4, (k + rng.randf_range(0.25, 0.75)) / 8.0)
				_mesh(root, _sphere(1.0, 9), boulder, Vector3(x, bottom - 1.0, front - 0.6),
					Vector3(rng.randf_range(1.5, 2.1), rng.randf_range(1.0, 1.35), 1.2), Vector3(0, rng.randf_range(0, 60), rng.randf_range(-10, 10)))
		"city":
			# The rooftop is a building: its facade carries on down to the street, lost in the glow of the city below. A cornice marks
			# the floor the roof sits on.
			var cx := (l + r) / 2.0
			var street := bottom - CITY_STREET_DROP
			var tall := bottom + 0.3 - street
			var tower := _mesh(root, _box(Vector3(r - l - 0.4, tall, 7.0)), _under_material(_soil_material(t), 0.55), Vector3(cx, street + tall / 2.0, front - 3.5))
			tower.name = "Underside"
			tower.set_instance_shader_parameter("front_top", e[2])
			_mesh(root, _box(Vector3(r - l + 0.1, 0.32, 0.5)), _soft((t.soil as Color).lightened(0.1)), Vector3(cx, bottom - 0.12, front + 0.2))
			_mesh(root, _shade_band(r - l - 0.4, 1.2), _band_material(), Vector3(cx, bottom - 0.9, front + 0.01))
			_shopfront(root, cx, r - l - 0.4, street, front + 0.02)
			_street(root, street, front)
		_:
			# A floating island: a keel of earth tapering to a point a little off-centre, rocks set in it, roots hanging from the rim.
			var length := clampf((r - l) * 0.42, 4.0, 8.0)
			var keel := _hanging_mass(bottom, length, l, r, front, 3.2, 0.12, 9,
				func(s: float) -> float: return (1.0 - 0.18 * s) * (1.0 - pow(s, 2.4)),
				func(s: float) -> float: return 0.09 * s * s)
			# (The keel is the wall's own earth carried on down, falling into shade below the light, as the canyon's mesa and the city's
			# tower are, so the platform reads as one body of earth, not a wall set on a separate rock.)
			var under := _mesh(root, keel, _under_material(_soil_material(t), 0.85), Vector3.ZERO)
			under.name = "Underside"
			under.set_instance_shader_parameter("front_top", e[2])
			# A ragged skirt of rounded rocks along the foot of the wall, half above and half below its bottom edge, so the built wall
			# grows out of the island's earth instead of sitting on it along a ruled line. Deterministic.
			# (In the earth's own darker colour, of very different sizes, in clusters with gaps, sunk into the earth rather than standing
			# proud in a row: a ring of equal light stones read as a separate layer, a necklace under the wall.)
			var skirt := Materials.stone((t.soil_dark as Color).lerp(t.soil as Color, 0.15), Vector2(1, 1))
			var skirt_lit := Materials.stone((t.soil_dark as Color).lerp(t.soil as Color, 0.4), Vector2(1, 1))
			var sx := l - 0.2
			var j := 0
			# (Not under the earth islands: their wall's earth carries straight on into the keel, as the sky islands' do; the row of
			# rocks read as a separate rubble layer. Art review 6.)
			if t.get("pattern", "") in ["earth", "roots"]:
				sx = r + 1.0
			while sx < r + 0.2:
				var n := absf(sin(float(j) * 2.3 + 0.7)) * absf(sin(float(j) * 0.9 + 1.3))
				var sr := 0.35 + 1.0 * n
				if n > 0.3:
					_mesh(root, _sphere(1.0, 9), skirt if j % 3 != 0 else skirt_lit, Vector3(sx, bottom - 0.25 - sr * 0.3 + 0.35 * sin(float(j) * 2.9),
						0.05 + sr * 0.15), Vector3(sr * 1.35, sr, sr * 0.8), Vector3(0, float(j) * 37.0, float(j) * 11.0))
				sx += sr * (1.2 + 0.7 * absf(sin(float(j) * 1.7)))
				j += 1
			var w := r - l
			var cx := (l + r) / 2.0
			# (Stones bedded in the keel, in the earth's own colours a step lighter, half sunk: pale lavender stones read as stickers.)
			var rock := Materials.stone((t.soil as Color).lerp(t.soil_dark as Color, 0.35))
			for k in 6:
				var u := rng.randf_range(-0.36, 0.36)
				var down := (1.0 - absf(u) * 2.0) * rng.randf_range(0.25, 0.6) * length
				_mesh(root, _sphere(1.0, 7), rock, Vector3(cx + u * w, bottom - 0.9 - down, front - 0.45),
					Vector3(1.3, 0.9, 1.0) * rng.randf_range(0.45, 0.85),
					Vector3(rng.randf_range(-20, 20), rng.randf_range(0, 60), rng.randf_range(-15, 15)))
			var root_mat := Materials.bark(Color(0.5, 0.33, 0.22), Vector2(1, 2))
			for k in 7:
				var x := l + 0.8 + rng.randf_range(0.0, w - 1.6)
				var len := rng.randf_range(1.2, 3.2) * (1.0 - absf(x - cx) / w)
				_mesh(root, _cylinder(0.05, 0.12, len, 5), root_mat, Vector3(x, bottom - len * 0.5 + 0.1, front + 0.15), Vector3.ONE,
					Vector3(rng.randf_range(-8, 8), 0, rng.randf_range(-14, 14)))
			_under_haze(root, t, bottom - length * 0.65, 9.0, 0.45)


## A copy of a block's world-laid material `m` for what carries on below it, out of the light: its front falls `fall` of the way into
## the cool front shade at once (`front_fall` in shaders/toon_common.gdshaderinc).
static func _under_material(m: ShaderMaterial, fall: float) -> ShaderMaterial:
	var key := "%d,%.2f" % [m.get_instance_id(), fall]
	if not _under_cache.has(key):
		var u := m.duplicate() as ShaderMaterial
		u.set_shader_parameter("front_fall", fall)
		_under_cache[key] = u
	return _under_cache[key]


static var _under_cache := {}


## A rough mass hanging from a block's bottom `top` for `length` units: `rings` rings of 14 points each, from the block's width
## [`l`, `r`] (a little inside it) down, `width.call(s)` times that half-width at each step `s` (0 at the top, 1 at the bottom),
## shifted sideways by `shift.call(s)` of the width, `half_depth` deep front to back with its front at `front`, every point pushed in or
## out by up to `rough` (the same every time). The top ring is tucked inside the block; the bottom is closed. Smooth normals, so the cel
## ramp shades it as one mass; UVs run round (u, in units) and down (v).
static func _hanging_mass(top: float, length: float, l: float, r: float, front: float, half_depth: float, rough: float, rings: int,
		width: Callable, shift: Callable) -> ArrayMesh:
	const SIDES := 14
	var rng := RandomNumberGenerator.new()
	rng.seed = int(l * 31.0 + r * 17.0 + length * 7.0)
	var hw0 := (r - l) * 0.5 - 0.2
	var cx0 := (l + r) * 0.5
	var zc := front - half_depth * (1.0 + rough)
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	for k in rings:
		var s := float(k) / float(rings - 1)
		var y := top + 0.3 - (length + 0.3) * s
		var hw := maxf(hw0 * float(width.call(s)), 0.02)
		var hd := half_depth if k == 0 else minf(half_depth, maxf(hw, 0.02))
		var cx := cx0 + float(shift.call(s)) * (r - l)
		var row := []
		for i in SIDES:
			row.append(rng.randf_range(-rough, rough) if k > 0 else 0.0)
		for i in SIDES + 1:
			# The seam (i = 0 and SIDES) at the back, where nobody sees it.
			var a := -PI * 0.5 + TAU * float(i) / SIDES
			var j: float = row[i % SIDES]
			st.set_uv(Vector2(float(i) / SIDES * (hw + hd) * 1.6, (top - y) * 0.5))
			st.add_vertex(Vector3(cx + cos(a) * hw * (1.0 + j), y, zc + sin(a) * hd * (1.0 + j)))
	for k in rings - 1:
		for i in SIDES:
			var a0 := k * (SIDES + 1) + i
			var b0 := a0 + SIDES + 1
			for v in [a0, b0, a0 + 1, a0 + 1, b0, b0 + 1]:
				st.add_index(v)
	# Close the bottom on one point under the last ring.
	var last := (rings - 1) * (SIDES + 1)
	var tip := rings * (SIDES + 1)
	st.set_uv(Vector2(0.0, (length + 0.3) * 0.5 + 0.2))
	st.add_vertex(Vector3(cx0 + float(shift.call(1.0)) * (r - l), top + 0.3 - (length + 0.3) - 0.05, zc))
	for i in SIDES:
		for v in [last + i, tip, last + i + 1]:
			st.add_index(v)
	st.generate_normals()
	return st.commit()


## A bank of haze under the stage, in front of what hangs there but behind the fighters: clear at `from_y`, thickening downward over
## `depth` units to `strength`, in the sky's horizon colour; wide enough to cover the view, so the scenery below the stage sinks into it
## too.
static func _under_haze(root: Node3D, t: Dictionary, from_y: float, depth: float, strength: float) -> void:
	var colour: Color = (t.sky_horizon as Color).lerp(Color.WHITE, 0.2)
	var g := Gradient.new()
	g.set_color(0, Color(colour, strength))
	g.set_color(1, Color(colour, 0.0))
	g.add_point(0.45, Color(colour, strength * 0.55))
	var tex := GradientTexture2D.new()
	tex.gradient = g
	tex.fill_from = Vector2(0.5, 1.0)
	tex.fill_to = Vector2(0.5, 0.0)
	tex.width = 4
	tex.height = 128
	var m := StandardMaterial3D.new()
	m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	m.albedo_texture = tex
	# (Clamped, or the bank's thick bottom bleeds into its clear top edge as a line.)
	m.texture_repeat = false
	m.disable_fog = true
	var q := QuadMesh.new()
	q.size = Vector2(400.0, depth)
	var z := 0.3 - UNDER_RECESS + 0.4
	var mi := _mesh(root, q, m, Vector3(0, from_y - depth * 0.5, z))
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	mi.name = "UnderHaze"
	# Below the bank the haze stays at its full strength, so a far zoom never shows where it ends.
	var solid := m.duplicate() as StandardMaterial3D
	solid.albedo_texture = null
	solid.albedo_color = Color(colour, strength)
	var fill := QuadMesh.new()
	fill.size = Vector2(400.0, 80.0)
	var below := _mesh(root, fill, solid, Vector3(0, from_y - depth - 40.0 + 0.02, z))
	below.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF


## An authored prop (godot/models/props/<name>.glb, from art/blender/make_props.py) at `at`, `size` times its modelled size, turned `turn`
## degrees about the vertical. Its parts go straight under `root` with their cel materials as overrides, so `merge_static` joins them
## with everything else drawn in the same colours (a few draw calls for a whole row of mesas). False if the prop is missing.
static func _place_prop(root: Node3D, prop_name: String, at: Vector3, size: float, turn := 0.0, surface: Material = null) -> bool:
	var prop := Toonify.load_prop(prop_name)
	if prop == null:
		push_warning("stage prop %s is missing (run art/blender/make_props.py)" % prop_name)
		return false
	var place := Transform3D(Basis(Vector3.UP, deg_to_rad(turn)).scaled(Vector3.ONE * size), at)
	for part: MeshInstance3D in prop.find_children("*", "MeshInstance3D", true, false):
		var xf := part.transform
		var up := part.get_parent() as Node3D
		while up != null and up != prop:
			xf = up.transform * xf
			up = up.get_parent() as Node3D
		var mi := MeshInstance3D.new()
		mi.mesh = part.mesh
		# (`surface` puts every part in one material instead, e.g. the stage's own painted strata on a butte.)
		mi.material_override = surface if surface != null else part.get_surface_override_material(0)
		mi.transform = place * prop.transform * xf
		mi.name = prop_name
		mi.set_meta("prop", true)
		root.add_child(mi)
	prop.free()
	return true


## Stage extent helpers: [left, right, top, bottom] of the solid ground, with defaults.
static func _ext(stage: Array) -> Array:
	return stage.slice(0, 4) if stage.size() >= 4 else [-11.0, 11.0, 0.0, -8.0]


static func _meadow(root: Node3D, t: Dictionary, stage: Array) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 42
	var e := _ext(stage)
	_hills(root, rng, Color(0.36, 0.72, 0.34), Color(0.42, 0.66, 0.6))
	_clouds(root, rng, 11, Color(1.0, 1.0, 1.0), Vector2(14, 34))
	# Islands floating in the sky beyond the stage, at several depths (smaller and paler with distance).
	# (Six, round 40: nine crowded the top of the frame, several clipped by its edge.)
	_sky_islands(root, t, rng, 6)
	_birds(root, rng, Color(0.25, 0.22, 0.3))
	# A windmill on the crest of a near hill (just behind the land's edge, so it stands in it), its sails turning slowly.
	_windmill(root, Vector3(-34, hill_top(-34.0) - 0.8, -73))
	# A few cottages on the near hills (just behind the land's edge), smoke rising from their chimneys: someone lives here.
	for c in [[18.0, Color(0.78, 0.34, 0.3)], [44.0, Color(0.36, 0.5, 0.72)], [-66.0, Color(0.74, 0.5, 0.3)]]:
		var cx: float = c[0]
		_cottage(root, Vector3(cx, hill_top(cx) - 0.6, -74.0), c[1], 1.0 if cx > 0.0 else -1.0)
	# A low rise of meadow in front of the hills, and round trees standing along it (their feet just behind its edge).
	_fore_band(root, -46.0, -10.0, Color(0.4, 0.76, 0.32))
	_fields(root, -46.0, -10.0, [Color(0.42, 0.78, 0.3), Color(0.52, 0.8, 0.3), Color(0.38, 0.7, 0.3), Color(0.44, 0.76, 0.34), Color(0.78, 0.8, 0.42)],
		Color(0.28, 0.56, 0.24))
	for k in 6:
		var tx := rng.randf_range(-70, 70)
		rng.randf_range(-12, -6)
		rng.randf_range(-62, -48)
		_round_tree(root, Vector3(tx, fore_top(tx, -10.0) - 0.5, -47.0), rng.randf_range(0.7, 1.2), rng)
	# A wooden fence along the back of the stage, and flowers: pale posts with rounded caps, each in its own contact shade, and darker
	# rails, so the fence reads as built (posts and rails) under the key light rather than one flat colour.
	var post := Materials.wood(Color(0.9, 0.7, 0.46))
	var rail := Materials.wood(Color(0.72, 0.5, 0.32), Vector2(6, 1))
	var cap := FighterView.toon(Color(0.78, 0.56, 0.36))
	# Built by hand: sturdier posts of slightly different heights, each leaning a touch, and the rails set bay by bay between them, each
	# board a little off level, its ends passing into the posts (not two perfect rails the length of the stage).
	var posts: Array = []
	var x: float = e[0] + 0.6
	while x <= e[1] - 0.5:
		var tall := 0.94 + rng.randf_range(-0.05, 0.05)
		var lean := rng.randf_range(-2.5, 2.5)
		_mesh(root, _box(Vector3(0.22, tall, 0.22)), post, Vector3(x, e[2] + tall / 2.0, -6.2), Vector3.ONE, Vector3(0, 0, lean))
		_mesh(root, _sphere(0.145, 8), cap, Vector3(x - sin(deg_to_rad(lean)) * tall, e[2] + tall + 0.02, -6.2), Vector3(1.0, 0.55, 1.0))
		_contact(root, Vector3(x, e[2], -6.2), Vector2(0.22, 0.17))
		posts.append(x)
		x += 1.6
	for i in posts.size() - 1:
		var a0: float = posts[i]
		var a1: float = posts[i + 1]
		for h in [0.32, 0.7]:
			var tilt := rng.randf_range(-1.6, 1.6)
			_mesh(root, _box(Vector3(a1 - a0 + 0.12, 0.15, 0.09)), rail, Vector3((a0 + a1) / 2.0, e[2] + h + rng.randf_range(-0.02, 0.02), -6.06),
				Vector3.ONE, Vector3(0, 0, tilt))
	_flowers(root, e, rng, [Color(1, 0.55, 0.6), Color(1, 1, 1), Color(1, 0.85, 0.3)])
	# A bright day: a few motes of pollen in the air, no shafts of light (golden, small and few: pale ones on the blue sky read as snow).
	_motes(root, Color(1.0, 0.82, 0.4), e)


## Islands floating in the sky behind the stage (the way the stage itself floats): an earth keel tapering to a point, a thick grassy cap
## with a lip of turf over its edge, and on some a round tree or a bush. `count` of them over the sky at depths from 55 to 130 units,
## higher and smaller the further they are, so they layer the sky into distances. Each bobs gently (`Drift` keeps them in place).
static func _sky_islands(root: Node3D, t: Dictionary, rng: RandomNumberGenerator, count: int) -> void:
	for k in count:
		var depth := rng.randf_range(0.0, 1.0)
		# (Each island takes on the sky's colour with its distance, as the windmill and houses at the same depth do through the haze:
		# full-strength earth and grass on the islands made them the sharpest things in the sky, pulling the eye from the fight. Art
		# review 6, background cohesion.)
		var air := t.sky_horizon as Color
		var fade := 0.15 + 0.22 * depth
		var grass: Color = (t.grass as Color).lerp(air, fade)
		# (The stage's own painted soil, faded with distance, art review 6 round 25: a generic grey stone made the islands read as a lower
		# grade of art than the block the fighters stand on; one material ties the background to the foreground.)
		var air_t := t.duplicate()
		air_t.soil = (t.soil as Color).lerp(air, fade)
		air_t.soil_dark = (t.soil_dark as Color).lerp(air, fade)
		var earth := _under_material(_soil_material(air_t), 0.45)
		var turf := FighterView.toon(grass.lightened(0.05))
		var lip := FighterView.toon(grass.darkened(0.12))
		var z := lerpf(-70.0, -140.0, depth)
		var size := lerpf(2.4, 4.4, rng.randf()) * lerpf(1.0, 1.4, depth)
		# (Spread across the sky, kept off the middle above the stage where the fighters fly.)
		var side := -1.0 if k % 2 == 0 else 1.0
		var x := side * (rng.randf_range(12.0, 34.0) + 10.0 * float(k / 2)) * lerpf(1.0, 1.5, depth)
		var y := rng.randf_range(3.0, 9.0) + depth * 6.0
		var island := Node3D.new()
		island.position = Vector3(x, y, z)
		root.add_child(island)
		var hw := size
		# (A rocky tuber rather than a cone: full under the turf, bulging and pinching as it narrows to a point, in more rings so its outline
		# reads as rounded rock, not a faceted pyramid.)
		var lump := float(k) * 1.7
		var keel := _hanging_mass(0.0, size * rng.randf_range(1.0, 1.5), -hw, hw, size * 0.6, size * 0.6, 0.1, 13,
			func(u: float) -> float: return (1.0 - 0.12 * u) * (1.0 - pow(u, 2.8)) * (1.0 + 0.1 * sin(u * 9.0 + lump)),
			func(u: float) -> float: return 0.06 * u * u + 0.03 * sin(u * 5.0 + lump))
		_mesh(island, keel, earth, Vector3.ZERO)
		# The grassy top: a thick squashed disc, its rim rolling down over the earth.
		_mesh(island, _cylinder(hw * 0.98, hw * 1.02, 0.7, 20), turf, Vector3(0, 0.35, 0), Vector3(1.0, 1.0, 0.62))
		_mesh(island, _cylinder(hw * 1.04, hw * 0.96, 0.4, 20), lip, Vector3(0, -0.05, 0), Vector3(1.0, 1.0, 0.64))
		var faded := t.duplicate()
		for key in ["soil", "soil_dark", "grass"]:
			faded[key] = (t[key] as Color).lerp(air, fade)
		_island_detail(island, hw, size, faded, k)
		var on_top := rng.randf()
		if on_top < 0.45:
			_round_tree(island, Vector3(rng.randf_range(-0.4, 0.4) * hw, 0.6, 0), clampf(size / 6.0, 0.7, 1.4), rng)
		elif on_top < 0.75:
			var bush := Materials.foliage(Materials.greens(grass)[1], Vector2(2, 2), false)
			_mesh(island, _sphere(1.0, 12), bush, Vector3(rng.randf_range(-0.5, 0.5) * hw, 0.9, 0), Vector3(1.6, 1.1, 1.2) * size / 6.0)
		# (Its still parts merged by material, as the root's are: an island is a couple of dozen small shapes.)
		merge_static(island)


## The finer parts of a sky island `hw` across (in `island`'s space; its own numbers, from `k`, so the sky's layout is unchanged): a pale
## band of strata under the turf, a few stones set in the keel, turf hanging over the lip in rounded drips at the front, roots trailing
## below, and a bright crisp line where the grass top meets the light.
static func _island_detail(island: Node3D, hw: float, size: float, t: Dictionary, k: int) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 900 + k
	var grass: Color = t.grass
	var band := Materials.stone((t.soil as Color).lightened(0.12), Vector2(6, 1))
	_mesh(island, _cylinder(hw * 0.94, hw * 0.86, size * 0.18, 18), band, Vector3(0, -0.45, 0), Vector3(1.0, 1.0, 0.6))
	var stone := Materials.stone((t.soil_dark as Color).lerp(Color(0.6, 0.58, 0.62), 0.5))
	# A second, darker band lower down, so the keel reads as layered rock.
	_mesh(island, _cylinder(hw * 0.72, hw * 0.6, size * 0.1, 16), Materials.stone((t.soil_dark as Color).darkened(0.12), Vector2(6, 1)),
		Vector3(0, -0.5 - size * 0.32, 0), Vector3(1.0, 1.0, 0.6))
	# Vines hanging over the lip at the front, ending in a leafy tuft.
	var vine := FighterView.toon((t.grass as Color).darkened(0.25), false)
	for j in 3:
		var vx := rng.randf_range(-0.7, 0.7) * hw
		var vl := rng.randf_range(0.5, 1.2) * size * 0.5
		var vz := sqrt(maxf(0.0, 1.0 - pow(vx / hw, 2.0))) * hw * 0.6
		_mesh(island, _cylinder(0.03 * size, 0.02 * size, vl, 5), vine, Vector3(vx, -0.2 - vl / 2.0, vz))
		_mesh(island, _sphere(0.09 * size, 8), vine, Vector3(vx, -0.2 - vl, vz), Vector3(1.0, 1.3, 1.0))
	for j in 5:
		var a := rng.randf_range(-0.7, 0.7)
		var down := rng.randf_range(0.25, 0.55) * size
		_mesh(island, _sphere(1.0, 8), stone, Vector3(sin(a) * hw * 0.6 * (1.0 - down / size), -0.6 - down, cos(a) * hw * 0.32),
			Vector3.ONE * rng.randf_range(0.1, 0.18) * size, Vector3(rng.randf_range(0, 60), rng.randf_range(0, 90), 0))
	# Turf rolling over the front edge: tongues of different widths and lengths, overlapping and sunk into the lip so they read as one
	# scalloped skirt of grass, not a row of equal beads.
	var drip := FighterView.toon(grass.darkened(0.12))
	var n := 9
	for j in n:
		var u := (float(j) + rng.randf_range(0.0, 1.0)) / float(n) * 2.0 - 1.0
		var r := rng.randf_range(0.1, 0.3) * size * 0.5
		var hang := rng.randf_range(1.0, 2.6)
		_mesh(island, _sphere(1.0, 8), drip, Vector3(u * hw * 0.92, -0.12 - r * hang * 0.55, sqrt(maxf(0.0, 1.0 - u * u)) * hw * 0.6),
			Vector3(r * rng.randf_range(1.6, 2.4), r * hang, r))
	# Roots trailing from the keel: thin, tapering to a point, in two bent lengths each, close to the earth's own colour (dark sticks read
	# as posts).
	var root_m := FighterView.toon((t.soil_dark as Color).darkened(0.28), false)
	for j in 4:
		var u := rng.randf_range(-0.6, 0.6)
		var len := rng.randf_range(0.5, 1.1) * size * 0.6
		var bend := rng.randf_range(-25.0, 25.0)
		var top := Vector3(u * hw * 0.7, -0.5 - size * 0.2, hw * 0.3)
		_mesh(island, _cylinder(0.035 * size, 0.022 * size, len * 0.55, 5), root_m, top + Vector3(0, -len * 0.27, 0), Vector3.ONE, Vector3(0, 0, bend * 0.4))
		var mid := top + Vector3(sin(deg_to_rad(bend * 0.4)) * len * 0.55, -len * 0.53, 0)
		_mesh(island, _cylinder(0.022 * size, 0.004 * size, len * 0.5, 5), root_m, mid + Vector3(-sin(deg_to_rad(bend)) * len * 0.25, -len * 0.24, 0), Vector3.ONE, Vector3(0, 0, bend))
	_mesh(island, _cylinder(hw * 1.0, hw * 1.0, 0.06, 24), FighterView.toon(grass.lightened(0.25), false), Vector3(0, 0.72, 0), Vector3(1.0, 1.0, 0.63))


## A windmill (`at` is the foot of its tower): a footing of rough stones, a tapered whitewashed tower with a plank door, two small
## shuttered windows and a wooden gallery round it, a shingled cap with a trim ring and a finial, and four lattice sails, each a stock
## carrying a frame of rails and slats with canvas over most of it, that turn.
static func _windmill(root: Node3D, at: Vector3) -> void:
	var plaster := Materials.plaster(Color(0.96, 0.94, 0.88), Vector2(3, 3))
	var wood := Materials.wood(Color(0.55, 0.38, 0.26))
	var wood_dark := Materials.wood(Color(0.42, 0.28, 0.2), Vector2(2, 1))
	_mesh(root, _cylinder(3.3, 3.6, 2.0, 14), Materials.stone(Color(0.66, 0.62, 0.58), Vector2(4, 1)), at + Vector3(0, 1.0, 0))
	# (A grassy mound it stands in, and its shade, so it is planted on the hill and does not hover over it.)
	_mesh(root, _sphere(1.0, 16), Materials.foliage(Color(0.4, 0.74, 0.3), Vector2(3, 2), false), at + Vector3(0, 0.1, 0), Vector3(5.6, 1.3, 4.6))
	_contact(root, at + Vector3(0, 0.4, 0), Vector2(4.2, 3.4))
	_mesh(root, _cylinder(1.8, 3.0, 12.0, 14), plaster, at + Vector3(0, 7.0, 0))
	# The door at the foot, and two windows up the front, each a dark opening in a wooden frame with a shutter.
	_mesh(root, _box(Vector3(1.6, 2.6, 0.3)), wood_dark, at + Vector3(0, 3.2, 2.85))
	_mesh(root, _cylinder(0.8, 0.8, 0.3, 12), wood_dark, at + Vector3(0, 4.5, 2.85), Vector3.ONE, Vector3(90, 0, 0))
	for wy in [7.2, 10.2]:
		var r := lerpf(3.0, 1.8, (wy - 1.0) / 12.0)
		_mesh(root, _box(Vector3(1.0, 1.1, 0.2)), wood, at + Vector3(0, wy, r + 0.05))
		_mesh(root, _box(Vector3(0.7, 0.8, 0.22)), _unshaded(Color(0.2, 0.16, 0.22)), at + Vector3(0, wy, r + 0.08))
		_mesh(root, _box(Vector3(0.5, 1.1, 0.12)), Materials.wood(Color(0.36, 0.56, 0.62)), at + Vector3(0.8, wy, r + 0.1), Vector3.ONE, Vector3(0, -25, 0))
	# The gallery: a wooden deck round the tower and a rail over it.
	_mesh(root, _cylinder(3.0, 3.0, 0.25, 18), wood, at + Vector3(0, 8.6, 0))
	_mesh(root, _cylinder(3.0, 3.0, 0.12, 18), wood_dark, at + Vector3(0, 9.5, 0))
	for k in 12:
		var ang := TAU * k / 12.0
		_mesh(root, _box(Vector3(0.12, 0.9, 0.12)), wood_dark, at + Vector3(sin(ang) * 2.9, 9.05, cos(ang) * 2.9))
	# The cap: a trim ring, a shingled cone and a knob on top.
	_mesh(root, _cylinder(2.1, 2.1, 0.4, 16), wood_dark, at + Vector3(0, 13.2, 0))
	_mesh(root, _cylinder(0.0, 2.6, 3.2, 16), Materials.shingles(Color(0.74, 0.34, 0.28), Vector2(6, 2)), at + Vector3(0, 15.0, 0))
	_mesh(root, _sphere(0.3, 10), wood_dark, at + Vector3(0, 16.7, 0))
	# The windshaft comes out of the cap through a wooden housing on the tower's face, tilted up a little as a real one is, into the hub.
	_mesh(root, _box(Vector3(1.3, 1.1, 1.0)), wood_dark, at + Vector3(0, 12.5, 1.75))
	_mesh(root, _cylinder(0.26, 0.26, 1.2, 10), wood, at + Vector3(0, 12.55, 2.6), Vector3.ONE, Vector3(84, 0, 0))
	var hub := Spinner.new()
	hub.speed = 0.6
	hub.position = at + Vector3(0, 12.6, 3.2)
	root.add_child(hub)
	# The hub: a drum the stocks pass through, an iron cap on its nose.
	_mesh(hub, _cylinder(0.62, 0.62, 0.7, 14), wood_dark, Vector3.ZERO, Vector3.ONE, Vector3(90, 0, 0))
	_mesh(hub, _cylinder(0.32, 0.4, 0.3, 12), FighterView.respond(FighterView.toon(Color(0.36, 0.34, 0.42)), "metal"), Vector3(0, 0, 0.45), Vector3.ONE, Vector3(90, 0, 0))
	var sail := FighterView.toon(Color(0.95, 0.9, 0.8))
	for k in 4:
		var arm := Node3D.new()
		arm.rotation.z = TAU * k / 4.0
		hub.add_child(arm)
		# The stock; on one side of it a lattice (an outer rail and slats across) with the canvas stretched behind it, so the frame reads
		# over the cloth; on the other a narrow leading board.
		_mesh(arm, _box(Vector3(0.32, 7.6, 0.26)), wood, Vector3(0, 3.8, 0))
		_mesh(arm, _box(Vector3(0.14, 5.8, 0.16)), wood, Vector3(1.85, 4.6, 0.06))
		for j in 7:
			_mesh(arm, _box(Vector3(1.9, 0.1, 0.12)), wood, Vector3(0.95, 1.9 + j * 0.9, 0.08))
		_mesh(arm, _box(Vector3(1.66, 5.2, 0.08)), sail, Vector3(0.95, 4.75, -0.04))
		_mesh(arm, _box(Vector3(0.42, 5.6, 0.08)), wood_dark, Vector3(-0.36, 4.7, -0.02))


## A red barn (`at` its foot): plank walls under a steep roof, white trim round the big doors with a cross brace, and a hay loft door.
static func _barn(root: Node3D, at: Vector3) -> void:
	var red := Materials.wood(Color(0.78, 0.26, 0.22), Vector2(4, 2))
	var trim := FighterView.toon(Color(0.98, 0.95, 0.9))
	var roof := Materials.shingles(Color(0.4, 0.34, 0.36), Vector2(4, 2))
	_mesh(root, _box(Vector3(5.6, 3.6, 4.0)), red, at + Vector3(0, 1.8, 0))
	# A stone footing, and white boards up the corners (built, not a red box).
	_mesh(root, _box(Vector3(5.8, 0.4, 4.2)), Materials.stone(Color(0.6, 0.57, 0.56), Vector2(4, 1)), at + Vector3(0, 0.2, 0))
	for cx in [-2.75, 2.75]:
		_mesh(root, _box(Vector3(0.22, 3.4, 0.22)), trim, at + Vector3(cx, 1.9, 1.95))
	for sgn in [-1.0, 1.0]:
		_mesh(root, _box(Vector3(6.2, 0.3, 3.0)), roof, at + Vector3(0, 4.45, sgn * 1.05), Vector3.ONE, Vector3(sgn * 45.0, 0, 0))
	_mesh(root, _box(Vector3(2.4, 2.6, 0.1)), trim, at + Vector3(0, 1.3, 2.02))
	_mesh(root, _box(Vector3(2.0, 2.3, 0.12)), red, at + Vector3(0, 1.2, 2.04))
	for sgn in [-1.0, 1.0]:
		_mesh(root, _box(Vector3(0.16, 2.9, 0.06)), trim, at + Vector3(0, 1.2, 2.12), Vector3.ONE, Vector3(0, 0, sgn * 40.0))
	_mesh(root, _box(Vector3(1.0, 0.9, 0.1)), trim, at + Vector3(0, 3.2, 2.02))
	_mesh(root, _box(Vector3(0.8, 0.7, 0.12)), _unshaded(Color(0.24, 0.16, 0.14)), at + Vector3(0, 3.2, 2.04))


## A cottage (`at` its foot), in the manner of a cel-shaded adventure's village houses: whitewashed walls framed in dark timber (corner
## posts, a beam at the floor line, braces in the end bays), a deep shingled roof in `roof` overhanging the gables and the eaves with a
## ridge beam along its top, gable ends filled to the roof with a round attic window, a plank door on a stone step with a lantern, a lit
## window with shutters and a flower box, shrubs at its foot, and a stone chimney at one end (`side`) with smoke curling up from it.
static func _cottage(root: Node3D, at: Vector3, roof: Color, side: float) -> void:
	var wall := Materials.plaster(Color(0.97, 0.93, 0.84), Vector2(2, 2))
	var roof_m := Materials.shingles(roof, Vector2(3, 2))
	var timber := Materials.wood(Color(0.42, 0.28, 0.2), Vector2(1, 3))
	var frame := Materials.wood(Color(0.56, 0.38, 0.26))
	var stone := Materials.stone(Color(0.64, 0.6, 0.58), Vector2(4, 1))
	var W := 5.0
	var D := 3.6
	var H := 3.2
	_mesh(root, _box(Vector3(W, H, D)), wall, at + Vector3(0, H / 2.0, 0))
	_mesh(root, _box(Vector3(W + 0.3, 0.5, D + 0.3)), stone, at + Vector3(0, 0.25, 0))
	# The timber frame on the front and the ends.
	for cx in [-W / 2.0, W / 2.0]:
		for cz in [-D / 2.0, D / 2.0]:
			_mesh(root, _box(Vector3(0.26, H, 0.26)), timber, at + Vector3(cx, H / 2.0 + 0.25, cz))
	_mesh(root, _box(Vector3(W + 0.1, 0.2, 0.1)), timber, at + Vector3(0, 2.25, D / 2.0 + 0.04))
	_mesh(root, _box(Vector3(W + 0.1, 0.24, 0.1)), timber, at + Vector3(0, H, D / 2.0 + 0.04))
	for sx in [-1.0, 1.0]:
		_mesh(root, _box(Vector3(0.14, 1.6, 0.08)), timber, at + Vector3(sx * (W / 2.0 - 0.55), 1.35, D / 2.0 + 0.05), Vector3.ONE,
			Vector3(0, 0, sx * 32.0))
	# The roof: two deep slabs overhanging every edge, a fascia board under each eave, and a round ridge beam.
	var pitch := 40.0
	var run := D / 2.0 + 0.7
	var slab := run / cos(deg_to_rad(pitch))
	var rise := run * tan(deg_to_rad(pitch))
	for sgn in [-1.0, 1.0]:
		var mid := Vector3(0, H + rise / 2.0 + 0.05, sgn * run / 2.0)
		_mesh(root, _box(Vector3(W + 1.2, 0.34, slab)), roof_m, at + mid, Vector3.ONE, Vector3(sgn * pitch, 0, 0))
		_mesh(root, _box(Vector3(W + 1.25, 0.2, 0.16)), frame, at + Vector3(0, H - 0.08, sgn * (run - 0.05)))
	_mesh(root, _cylinder(0.2, 0.2, W + 1.3, 10), timber, at + Vector3(0, H + rise + 0.12, 0), Vector3.ONE, Vector3(0, 0, 90))
	# The gable ends, filled to the underside of the roof, each with a round attic window.
	for ex in [-W / 2.0, W / 2.0]:
		_mesh(root, _gable(D, rise - 0.05, 0.16), wall, at + Vector3(ex, H, 0))
		_mesh(root, _cylinder(0.36, 0.36, 0.1, 14), frame, at + Vector3(ex + signf(ex) * 0.06, H + rise * 0.38, 0), Vector3.ONE, Vector3(0, 0, 90))
		_mesh(root, _cylinder(0.26, 0.26, 0.12, 14), _unshaded(Color(1.0, 0.84, 0.54)), at + Vector3(ex + signf(ex) * 0.08, H + rise * 0.38, 0),
			Vector3.ONE, Vector3(0, 0, 90))
	# The door on its step, with a lantern beside it.
	var dx := -0.95 * side
	_mesh(root, _box(Vector3(1.2, 2.05, 0.1)), frame, at + Vector3(dx, 1.27, D / 2.0 + 0.02))
	_mesh(root, _box(Vector3(0.96, 1.85, 0.12)), Materials.wood(Color(0.5, 0.32, 0.22), Vector2(3, 1)), at + Vector3(dx, 1.2, D / 2.0 + 0.05))
	_mesh(root, _sphere(0.07, 6), Materials.wood(Color(0.86, 0.7, 0.3)), at + Vector3(dx + 0.3 * side, 1.2, D / 2.0 + 0.13))
	_mesh(root, _box(Vector3(1.5, 0.22, 0.7)), stone, at + Vector3(dx, 0.11, D / 2.0 + 0.4))
	_mesh(root, _box(Vector3(0.22, 0.3, 0.22)), _unshaded(Color(1.0, 0.82, 0.46)), at + Vector3(dx + 0.85 * side, 2.0, D / 2.0 + 0.16))
	_mesh(root, _box(Vector3(0.3, 0.06, 0.3)), timber, at + Vector3(dx + 0.85 * side, 2.18, D / 2.0 + 0.16))
	# The window: framed, lit, shuttered, a flower box under it.
	var wx := 1.0 * side
	_mesh(root, _box(Vector3(1.15, 1.05, 0.08)), frame, at + Vector3(wx, 1.6, D / 2.0 + 0.02))
	_mesh(root, _box(Vector3(0.9, 0.8, 0.1)), _unshaded(Color(1.0, 0.86, 0.56)), at + Vector3(wx, 1.6, D / 2.0 + 0.05))
	_mesh(root, _box(Vector3(0.06, 0.8, 0.12)), frame, at + Vector3(wx, 1.6, D / 2.0 + 0.07))
	_mesh(root, _box(Vector3(0.9, 0.06, 0.12)), frame, at + Vector3(wx, 1.6, D / 2.0 + 0.07))
	for sx in [-1.0, 1.0]:
		_mesh(root, _box(Vector3(0.44, 1.0, 0.08)), Materials.wood(roof.lightened(0.1), Vector2(1, 3)), at + Vector3(wx + sx * 0.82, 1.6, D / 2.0 + 0.06))
	_mesh(root, _box(Vector3(1.2, 0.28, 0.34)), frame, at + Vector3(wx, 1.0, D / 2.0 + 0.2))
	var petals := [Color(0.96, 0.36, 0.42), Color(1.0, 0.84, 0.3), Color(0.98, 0.6, 0.78)]
	for j in 5:
		_mesh(root, _sphere(0.13, 8), FighterView.toon(Color(0.36, 0.62, 0.22)), at + Vector3(wx - 0.44 + j * 0.22, 1.2, D / 2.0 + 0.22))
		_mesh(root, _sphere(0.08, 6), _unshaded(petals[j % 3]), at + Vector3(wx - 0.44 + j * 0.22, 1.3, D / 2.0 + 0.3))
	# Shrubs at the corners of its foot.
	var bush := Materials.foliage(Color(0.34, 0.6, 0.24), Vector2(2, 2), false)
	for sx in [-1.0, 1.0]:
		_mesh(root, _sphere(1.0, 12), bush, at + Vector3(sx * (W / 2.0 + 0.2), 0.45, D / 2.0 + 0.1), Vector3(0.85, 0.7, 0.7))
	_contact(root, at, Vector2(W / 2.0 + 0.5, D / 2.0 + 0.4))
	var chimney := at + Vector3(1.5 * side, H + rise * 0.7, -0.7)
	_mesh(root, _box(Vector3(0.75, 2.2, 0.75)), Materials.stone(Color(0.62, 0.58, 0.56), Vector2(1, 2)), chimney)
	_mesh(root, _box(Vector3(0.95, 0.18, 0.95)), stone, chimney + Vector3(0, 1.1, 0))
	var top := Node3D.new()
	top.position = chimney + Vector3(0, 1.3, 0)
	root.add_child(top)
	# (Under the chimney top, not the root: the root's own meshes are merged when the backdrop is built.)
	Particles.chimney_smoke(top, top)


## A gable end: a flat triangle `width` across (along z) and `height` tall, `thick` through (along x), its base at the origin.
static func _gable(width: float, height: float, thick: float) -> ArrayMesh:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var h := thick / 2.0
	var w := width / 2.0
	var a := [Vector3(-h, 0, -w), Vector3(-h, 0, w), Vector3(-h, height, 0)]
	var b := [Vector3(h, 0, -w), Vector3(h, 0, w), Vector3(h, height, 0)]
	for tri in [[b[0], b[2], b[1]], [a[0], a[1], a[2]], [a[0], b[0], b[1]], [a[0], b[1], a[1]], [a[1], b[1], b[2]], [a[1], b[2], a[2]],
			[a[2], b[2], b[0]], [a[2], b[0], a[0]]]:
		for v in tri:
			st.set_uv(Vector2(v.z / width + 0.5, v.y / height))
			st.add_vertex(v)
	st.generate_normals()
	return st.commit()


## A small round tree: a short trunk under one big ball and a couple of small ones.
static func _round_tree(root: Node3D, at: Vector3, size: float, rng: RandomNumberGenerator) -> void:
	# A tapering trunk with a slight bow and roots flaring at its foot (one mesh, `trunk_mesh`), in warm painted bark.
	_mesh(root, trunk_mesh(3.2 * size, 0.42 * size, 0.26 * size, 0.18 * size * (1.0 if fmod(absf(at.x), 2.0) < 1.0 else -1.0), 0.35 * size),
		Materials.bark_plates(Color(0.6, 0.4, 0.24), Vector2(2, 1)), at)
	var greens := Materials.greens(Materials.greens(Color(0.3, 0.62, 0.16))[rng.randi_range(0, 2)])
	var crown := Sway.new()
	crown.position = at + Vector3(0, 2.8 * size, 0)
	crown.amount = 2.5
	crown.phase = rng.randf() * TAU
	root.add_child(crown)
	# Masses of leaf cards shading as rounded volumes (`_leaf_cards`), in one of three crown shapes so a row of trees is not one tree
	# repeated: a big mass with a smaller one to one side, a tall crown of a mass stacked on a mass, or a wide one with two shoulders.
	# (The shape comes from the tree's own position, not the stage's sequence, so nothing else in the layout moves.)
	var own := RandomNumberGenerator.new()
	own.seed = hash(Vector2i(roundi(at.x * 10.0), roundi(at.z * 10.0)))
	var big := own.randf_range(2.15, 2.55)
	var masses: Array
	match own.randi_range(0, 2):
		0:
			var lean := 1.0 if own.randf() < 0.5 else -1.0
			masses = [[Vector3(0, 1.4, 0) * size, big * size, 64], [Vector3(1.4 * lean, 0.7, 0.5) * size, 1.4 * size, 26]]
		1:
			masses = [[Vector3(0, 1.1, 0) * size, big * 0.92 * size, 54], [Vector3(0.25, 2.6, 0.2) * size, 1.45 * size, 28]]
		_:
			masses = [[Vector3(0, 1.5, 0) * size, big * 0.95 * size, 56], [Vector3(1.5, 0.6, 0.3) * size, 1.15 * size, 18],
				[Vector3(-1.4, 0.75, 0.3) * size, 1.05 * size, 16]]
	# Smaller clumps round the crown's edge (a crown of clustered leaf masses, as a cel-shaded adventure's trees are, rather than one
	# round blob), and two branches forking from the top of the trunk up into it, seen in the gaps between the clumps.
	for j in 4:
		var ang := own.randf_range(0.0, TAU)
		var reach := big * own.randf_range(0.75, 0.95)
		masses.append([Vector3(cos(ang) * reach, 1.4 + sin(ang) * reach * 0.7, own.randf_range(0.2, 0.7)) * size,
			own.randf_range(0.7, 0.95) * size, 12])
	var bark := Materials.bark_plates(Color(0.56, 0.37, 0.22), Vector2(1, 1))
	for sx in [-1.0, 1.0]:
		var tilt: float = sx * own.randf_range(28.0, 40.0)
		var length := own.randf_range(1.1, 1.5) * size
		_mesh(crown, _cylinder(0.08 * size, 0.15 * size, length, 6), bark,
			Vector3(-sin(deg_to_rad(tilt)) * length * 0.5, length * 0.4, 0.3 * size), Vector3.ONE, Vector3(0, 0, tilt))
	_leaf_cards(crown, masses, greens, 1.6 * size, rng, "leaf_cluster", false, Materials.foliage(greens[1], Vector2(3, 2), true))


static func _flowers(root: Node3D, e: Array, rng: RandomNumberGenerator, colors: Array) -> void:
	var heart := _unshaded(Color(1, 0.85, 0.3))
	var x: float = e[0] + 0.8
	var k := 0
	while x < e[1] - 0.8:
		var z := rng.randf_range(-4.5, -2.0)
		_mesh(root, _sphere(0.13, 8), _unshaded(colors[k % colors.size()]), Vector3(x, e[2] + 0.08, z))
		_mesh(root, _sphere(0.06, 6), heart, Vector3(x, e[2] + 0.12, z + 0.08))
		x += rng.randf_range(0.9, 1.8)
		k += 1


## A canyon at dusk: a big low sun, striped mesas and buttes, a rock arch, and cacti on the rim behind the stage.
## The canyon's desert floor: the height the far buttes stand at, the stage's mesa and the floor itself.
const DESERT_FLOOR := -16.0


static func _sunset(root: Node3D, t: Dictionary, stage: Array) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 7
	var e := _ext(stage)
	# (Smaller, lower and off to the left, art review 6 round 39: a huge disc right behind the stage outshone the fight.)
	_painted_sun(root, Vector3(-46, -4, -140), 10.0)
	# (Each row's rock takes on the dusk sky's colour with its distance, as the meadow's islands do: at full strength the far buttes were
	# as dark and saturated as the mesa the fighters stand on, so nothing receded. Art review 6, background cohesion.)
	var rows := []
	var haze := (t.sky_horizon as Color).lerp(Color.WHITE, 0.35)
	for fade in [0.35, 0.55]:
		var air := t.duplicate()
		air.soil = (t.soil as Color).lerp(haze, fade)
		air.soil_dark = (t.soil_dark as Color).lerp(haze, fade)
		rows.append(_under_material(_soil_material(air), 0.35 * (1.0 - fade)))
	# Buttes and mesas in strata on rubble slopes (art/blender/make_props.py), a nearer row and a farther one.
	for k in 8:
		var far := k % 2 == 1
		var mesa_rock: ShaderMaterial = rows[1 if far else 0]
		var at := Vector3(-110.0 + k * 30.0 + rng.randf_range(-6, 6), -16.0, -110.0 if far else -78.0)
		# (In the stage's own painted strata, laid in world space and falling into shade below the light, so the buttes are the same rock
		# as the mesa the fighters stand on.)
		_place_prop(root, "mesa_a" if k % 3 != 1 else "mesa_b", at, rng.randf_range(0.7, 1.05), rng.randf_range(0.0, 360.0), mesa_rock)
	# A rock arch on the left, standing on the desert floor.
	_rock_arch(root, Vector3(-34, DESERT_FLOOR, -48), 12.0, 18.5)
	# Cacti behind the stage.
	for x in [e[0] + 1.5, e[1] - 2.0]:
		_cactus(root, Vector3(x, e[2], -6.5), rng.randf_range(0.65, 0.85))
		_contact(root, Vector3(x, e[2], -6.5), Vector2(0.45, 0.38))
	_clouds(root, rng, 6, Color(1.0, 0.78, 0.7), Vector2(18, 30))
	# A line of telegraph poles running off across the desert floor, wires sagging between them: a road out there somewhere.
	var pole := Materials.wood(Color(0.5, 0.34, 0.26))
	var wire := FighterView.toon(Color(0.3, 0.2, 0.22), false)
	var prev := Vector3.INF
	for k in 9:
		var at := Vector3(-120.0 + k * 30.0, DESERT_FLOOR, -40.0 - 4.0 * absf(sin(k * 1.3)))
		# (None behind the stage itself: from the match camera the pole by the stage's end stood exactly on its corner, a stray stick
		# with its wire across the arch. Art review 6 round 18. The line breaks off there and carries on beyond.)
		if absf(at.x) < 36.0:
			prev = Vector3.INF
			continue
		_mesh(root, _cylinder(0.22, 0.28, 9.0, 6), pole, at + Vector3(0, 4.5, 0))
		_mesh(root, _box(Vector3(2.6, 0.22, 0.22)), pole, at + Vector3(0, 8.4, 0))
		var hook := at + Vector3(0, 8.6, 0)
		if prev != Vector3.INF:
			var last := prev
			for j in range(1, 9):
				var f := float(j) / 8.0
				var q := prev.lerp(hook, f) + Vector3(0, -1.4 * sin(f * PI), 0)
				var seg := _mesh(root, _box(Vector3(0.06, 0.06, last.distance_to(q))), wire, (last + q) / 2.0)
				seg.basis = Basis.looking_at(q - last, Vector3.UP)
				last = q
		prev = hook


## A natural arch of layered sandstone (`at` the middle of its foot on the ground, legs `span` apart, `height` to the top of the span):
## two legs of stacked, slightly uneven slabs in the canyon's strata colours, wider and rubbly at the foot, joined by a thick span that
## bows up between them, banded the same way, and a few fallen blocks at its feet. Deterministic (no random draws).
static func _rock_arch(root: Node3D, at: Vector3, span: float, height: float) -> void:
	# Close reds with a pale band now and then, as the mesas' strata (not stripes).
	# (Painted stone, inked, as the props by the stage are: flat fills read as a different, unfinished style.)
	var strata := [Materials.stone(Color(0.84, 0.46, 0.34), Vector2(2, 1)), Materials.stone(Color(0.8, 0.42, 0.32), Vector2(2, 1)),
		Materials.stone(Color(0.88, 0.52, 0.38), Vector2(2, 1)), Materials.stone(Color(0.94, 0.68, 0.5), Vector2(2, 1))]
	var slab := 2.4
	for side in [-1.0, 1.0]:
		var y := 0.0
		var k := 0
		while y < height - 2.0:
			var u := y / height
			var r := lerpf(3.2, 2.1, u) * (1.0 + 0.1 * sin(k * 2.7 + side))
			var h := slab * (0.8 + 0.4 * absf(sin(k * 1.9)))
			_mesh(root, _cylinder(r * 0.94, r, h, 9), strata[3 if k % 4 == 3 else k % 3], at + Vector3(side * span * 0.5 + 0.2 * sin(k * 1.3), y + h * 0.5, 0),
				Vector3(1.0, 1.0, 0.8), Vector3(0, k * 23.0, 0))
			y += h
			k += 1
	# The span: thick bands following a shallow bow from leg to leg, each band a run of boxes turned along it.
	var bands := [[0.0, 2.2, strata[1]], [2.2, 0.8, strata[3]], [3.0, 1.4, strata[0]]]
	for band in bands:
		var lift: float = band[0]
		var thick: float = band[1]
		var n := 10
		for j in n:
			var t0 := float(j) / n
			var t1 := float(j + 1) / n
			var x0 := lerpf(-span * 0.5 - 1.8, span * 0.5 + 1.8, t0)
			var x1 := lerpf(-span * 0.5 - 1.8, span * 0.5 + 1.8, t1)
			var y0 := height - 2.0 + lift + 1.6 * sin(t0 * PI)
			var y1 := height - 2.0 + lift + 1.6 * sin(t1 * PI)
			var mid := Vector3((x0 + x1) * 0.5, (y0 + y1) * 0.5 + thick * 0.5, 0)
			var len := Vector2(x1 - x0, y1 - y0).length()
			_mesh(root, _box(Vector3(len + 0.3, thick, 3.4 - lift * 0.3)), band[2], at + mid, Vector3.ONE, Vector3(0, 0, rad_to_deg(atan2(y1 - y0, x1 - x0))))
	# Fallen blocks at its feet.
	for b in [[-span * 0.5 - 3.2, 1.1], [-span * 0.5 + 2.6, 0.7], [span * 0.5 + 3.0, 1.3], [span * 0.5 - 2.4, 0.6]]:
		_mesh(root, _box(Vector3(1.6, 1.2, 1.4) * float(b[1])), strata[2], at + Vector3(float(b[0]), 0.5 * float(b[1]), 1.0), Vector3.ONE,
			Vector3(0, float(b[0]) * 17.0, float(b[0]) * 3.0))


## A setting sun (shaders/sun.gdshader, `at` its middle, `r` its radius): a hot core that blooms, its disc deepening to orange at the
## rim, a soft corona and faint slow rays on one card; and long thin bands of lilac and rose cloud drawn across its lower half, the way
## evening cloud lies over a low sun.
static func _painted_sun(root: Node3D, at: Vector3, r: float) -> void:
	var m := ShaderMaterial.new()
	m.shader = SUN_SHADER
	var q := QuadMesh.new()
	q.size = Vector2(r * 10.0, r * 10.0)
	m.set_shader_parameter("disc", 0.2)
	var sun := _mesh(root, q, m, at)
	sun.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	sun.name = "Sun"
	# Two real banks of cumulus drifting across the lower half of the disc, lit warm from the sun, in place of the flat squashed shapes
	# (the owner, 2026-10-15: the sun's clouds looked cheap). In front of it, so the sun sinks behind them.
	var rng := RandomNumberGenerator.new()
	rng.seed = 41
	_clouds(root, rng, 2, Color(1.0, 0.6, 0.56), Vector2(at.y - r * 0.55, at.y - r * 0.15), Vector2(at.z + 6.0, at.z + 8.0), [1], 1.15,
		Vector2(at.x - r * 0.6, at.x + r * 0.6))


## A ribbed saguaro with curving arms and a flower on top (art/blender/make_props.py), `size` about 4 units tall at 1.
static func _cactus(root: Node3D, at: Vector3, size: float) -> void:
	_place_prop(root, "cactus", at, size * 0.88)


static func _night(root: Node3D, t: Dictionary) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 11
	_mesh(root, _sphere(9.0, 32), _unshaded(Color(0.95, 0.95, 0.85)), Vector3(-30, 30, -130))
	var star := _unshaded(Color(1, 1, 0.9))
	var dot := _sphere(0.35, 6)
	for k in 90:
		_mesh(root, dot, star, Vector3(rng.randf_range(-130, 130), rng.randf_range(-5, 70), rng.randf_range(-150, -120)))
	_hills(root, rng, Color(0.14, 0.18, 0.32), Color(0.2, 0.24, 0.4))


## The grove: a giant old tree behind a floating island, a few smaller trees, bushes on the island, pastel hills and clouds, and
## sparkles drifting in the canopy.
static func _grove(root: Node3D, t: Dictionary, stage: Array) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 3
	_hills(root, rng, Color(0.3, 0.66, 0.4), Color(0.4, 0.54, 0.84))
	_clouds(root, rng, 8, Color(1, 0.97, 1), Vector2(10, 36))
	_birds(root, rng, Color(0.25, 0.22, 0.3))
	var top: float = stage[2] if stage.size() == 4 else 0.0
	var left: float = stage[0] if stage.size() == 4 else -11.0
	var right: float = stage[1] if stage.size() == 4 else 11.0
	# A far line of soft tree shapes on the hills, between them and the trees, for one more layer of depth (the haze fades it).
	_tree_line(root, rng, top - 16.0, -78.0, Color(0.5, 0.72, 0.56))
	# The great tree, set back behind the island's middle.
	_tree(root, Vector3(0, top - 2.0, -19.0), 1.0, rng)
	# A low rise of land in front of the hills, and two smaller trees standing on it either side (feet just behind its edge).
	# (A rounded ridge, not the meadow's valley floor: the forest canopy fills the valley below it.)
	_fore_band(root, -52.0, top - 11.0, Color(0.34, 0.66, 0.42), ROLL_PROFILE)
	_tree(root, Vector3(-30, fore_top(-30.0, top - 11.0) - 0.6, -55.0), 0.6, rng)
	_tree(root, Vector3(32, fore_top(32.0, top - 11.0) - 0.6, -55.0), 0.55, rng)
	# Below it, the treetops of a forest falling away down the valley.
	# (Deep, saturated forest greens: in the old pale mints, washed further by the air at that depth, the crowns either side of the stage
	# read as shapeless fog blobs, the weakest thing on the stage. Art review 6, refinement round 1.)
	_canopy(root, -49.0, top - 12.5, [Color(0.16, 0.44, 0.3), Color(0.26, 0.56, 0.32), Color(0.14, 0.4, 0.32), Color(0.3, 0.6, 0.34)])
	# Round bushes along the back of the island.
	var bush_greens := Materials.greens(Color(0.3, 0.66, 0.34))
	var bush := Materials.foliage(bush_greens[1])
	# (A clump of overlapping lobes in two greens, inked and lit like the crowns: a pale ball stuck on the front read as a flat sticker.)
	var bush_light := Materials.foliage(bush_greens[2])
	for x in [left + 1.4, left + 3.2, right - 3.2, right - 1.4]:
		var s := rng.randf_range(1.1, 1.5)
		_mesh(root, _sphere(1.0, 16), bush, Vector3(x, top + 0.6, -6.5), Vector3(s * 1.3, s, s))
		for j in 3:
			var side := float(j - 1)
			var lr := s * rng.randf_range(0.5, 0.65)
			_mesh(root, _sphere(1.0, 12), bush_light if j != 1 else bush, Vector3(x + side * s * 0.75, top + 0.4 + lr * 0.6 + (0.35 * s if j == 1 else 0.0),
				-6.2 + absf(side) * 0.2), Vector3(lr * 1.15, lr, lr))
		_contact(root, Vector3(x, top, -6.5), Vector2(s * 1.15, s * 0.8))
	# Little flowers and tufts on the grass behind the fighters, gathered toward the ends so the middle of the stage stays clean.
	_flower_clusters(root, [left, right, top], rng, [Color(1, 1, 1), Color(1, 0.8, 0.9)], Color(0.46, 0.84, 0.42))
	# Strings of lanterns hung from the great tree out to the smaller ones, under the canopy: the grove is looked after.
	_string_lights(root, Vector3(-4.0, top + 12.0, -17.0), Vector3(-22.0, top + 8.0, -30.0))
	_string_lights(root, Vector3(4.0, top + 12.0, -17.0), Vector3(22.0, top + 8.0, -30.0))
	# Pale pink motes drifting under the great tree, and light falling through its canopy.
	_light_shafts(root, Color(1.0, 0.84, 0.9), [left, right, top, top - 10.0])
	_motes(root, Color(1.0, 0.86, 0.94), [left, right, top, top - 10.0])
	# Sparkles in the canopy: four-pointed glints that twinkle (see `Twinkle`).
	var tw := Twinkle.new()
	root.add_child(tw)
	for k in 14:
		tw.add(Vector3(rng.randf_range(-11, 11), top + rng.randf_range(9, 20), -15.0 + rng.randf_range(-1, 1)), rng.randf_range(0.5, 0.9), rng.randf() * TAU,
			[Color(1, 0.95, 0.6), Color(1, 0.8, 0.95), Color(0.8, 0.95, 1)][k % 3])


## The ocean: the island sits on a gently moving sea that runs to the horizon; distant islands with palms, a lighthouse with a turning
## beam, a sailboat, gulls in the clouds, and palms at the ends of the stage.
static func _ocean(root: Node3D, t: Dictionary, stage: Array) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 21
	var e := _ext(stage)
	var sea := Waves.new()
	sea.level = float(e[3]) - 1.0
	root.add_child(sea)
	sea.build()
	_clouds(root, rng, 8, Color(1, 1, 1), Vector2(16, 34), Vector2(-120, -85), [0, 1, 2], 1.0, Vector2(-90, 90), BEACH_CLOUDS)
	# A bank of cumulus piled along the horizon, behind the far coast (its own random numbers, so the rest of the layout stays put).
	var bank_rng := RandomNumberGenerator.new()
	bank_rng.seed = 41
	_clouds(root, bank_rng, 16, Color(1, 1, 1), Vector2(-10.0, -5.0), Vector2(-215, -195), [1, 0, 1], 1.9, Vector2(-90, 90), BEACH_CLOUDS)
	_birds(root, rng, Color(0.97, 0.97, 1.0))
	# Distant islands.
	for k in 3:
		var isle := Vector3(-70.0 + k * 65.0 + rng.randf_range(-10, 10), float(e[3]) - 3.0, -110.0 + rng.randf_range(-10, 10))
		# (The sand painted as the stage's own beach is, the green a leafy foliage mound, both inked: the same finish as the props by the stage.)
		_mesh(root, _sphere(1.0, 16), Materials.world("sand", Color(0.98, 0.88, 0.62), 6.0), isle, Vector3(rng.randf_range(10, 16), 3.5, 6.0))
		_mesh(root, _sphere(1.0, 16), Materials.foliage(Color(0.3, 0.62, 0.42), Vector2(4, 2)), isle + Vector3(0, 2.0, 0), Vector3(rng.randf_range(6, 9), 3.5, 4.0))
		_palm(root, isle + Vector3(rng.randf_range(-4, 4), 4.0, 2.0), 1.4, rng)
		_isle_detail(root, isle, k)
	# A lighthouse on its rocks on the right (art/blender/make_props.py), its lantern glowing.
	# (Its rocks stand out of the sea, with foam round them; set 2.4 down, they were under it and the tower rose from bare water.)
	var lh := Vector3(52, float(e[3]) - 1.5, -80)
	_place_prop(root, "lighthouse", lh, 2.3, -20.0)
	_halo(root, lh + Vector3(0, 8.3 * 2.3, 0), Color(1.0, 0.9, 0.6), 7.0)
	# A sailboat.
	var boat := Bob.new()
	boat.position = Vector3(-30, float(e[3]) - 0.6, -60)
	root.add_child(boat)
	var dinghy := Toonify.load_prop("sailboat")
	if dinghy != null:
		dinghy.scale = Vector3.ONE * 1.6
		dinghy.position = Vector3(0, -0.9, 0)
		dinghy.rotation_degrees = Vector3(0, 160, 0)
		boat.add_child(dinghy)
	# A fisherman's hut on stilts out over the water, a jetty of planks running to it, and floats bobbing off the end: people live here.
	_stilt_hut(root, Vector3(30.0, float(e[3]) - 1.0, -46.0))
	# Palms at the ends of the island, behind the fighters.
	_palm(root, Vector3(float(e[0]) + 1.8, float(e[2]), -5.5), 1.0, rng)
	_palm(root, Vector3(float(e[1]) - 1.6, float(e[2]), -6.0), 1.15, rng)
	_contact(root, Vector3(float(e[0]) + 1.8, float(e[2]), -5.5), Vector2(0.4, 0.35))
	_contact(root, Vector3(float(e[1]) - 1.6, float(e[2]), -6.0), Vector2(0.45, 0.38))


## The finer parts of a distant island at `isle` (its own numbers, from `k`, so the sea's layout is unchanged): a ring of white surf
## where the beach meets the water, a second, smaller palm leaning the other way, grey rocks at one end with surf round them, and on one
## island a little hut with a red roof among the greenery.
static func _isle_detail(root: Node3D, isle: Vector3, k: int) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 500 + k
	var foam := _unshaded(Color(0.96, 0.99, 1.0))
	_mesh(root, _cylinder(1.0, 1.0, 0.08, 28), foam, isle + Vector3(0, 0.15, 0), Vector3(17.5, 1.0, 7.0))
	_mesh(root, _sphere(1.0, 12), Materials.foliage(Color(0.36, 0.68, 0.46), Vector2(3, 2)), isle + Vector3(rng.randf_range(-6.0, -3.0), 1.8, 1.0), Vector3(4.0, 2.6, 3.0))
	_palm(root, isle + Vector3(rng.randf_range(3.0, 6.0), 2.8, 2.6), 1.0, rng)
	var rock := Materials.stone(Color(0.62, 0.6, 0.66))
	var end := -1.0 if k % 2 == 0 else 1.0
	for j in 3:
		var r := rng.randf_range(1.0, 1.8)
		_mesh(root, _sphere(1.0, 8), rock, isle + Vector3(end * (12.0 + j * 1.8), 0.4, 2.0 - j), Vector3(r * 1.2, r, r), Vector3(0, rng.randf_range(0, 90), 0))
		_mesh(root, _cylinder(1.0, 1.0, 0.06, 14), foam, isle + Vector3(end * (12.0 + j * 1.8), 0.12, 2.0 - j), Vector3(r * 1.6, 1.0, r * 1.3))
	if k == 1:
		var hut := isle + Vector3(-1.5, 3.2, 2.2)
		_mesh(root, _box(Vector3(2.4, 1.6, 1.8)), Materials.plaster(Color(0.96, 0.92, 0.84)), hut + Vector3(0, 0.8, 0))
		_mesh(root, _cylinder(0.0, 2.0, 1.2, 4), Materials.shingles(Color(0.86, 0.36, 0.3), Vector2(4, 2)), hut + Vector3(0, 2.2, 0), Vector3.ONE, Vector3(0, 45, 0))


## A hut on stilts over the sea (`at` on the water under its middle): posts down into the water, a plank deck, walls of weathered boards
## with a lit window, a shaggy thatched roof, a jetty running off to the right (toward the lighthouse) on its own posts, a net hung over the rail and two red and
## white floats bobbing beside it.
static func _stilt_hut(root: Node3D, at: Vector3) -> void:
	var post := Materials.wood(Color(0.46, 0.34, 0.26))
	var plank := Materials.wood(Color(0.74, 0.58, 0.4), Vector2(6, 1))
	var boards := Materials.wood(Color(0.62, 0.5, 0.4), Vector2(3, 2))
	var thatch := Materials.wood(Color(0.86, 0.7, 0.42), Vector2(5, 3))
	for px in [-2.6, 2.6]:
		for pz in [-1.8, 1.8]:
			_mesh(root, _cylinder(0.2, 0.24, 4.6, 6), post, at + Vector3(px, 1.8, pz))
	_mesh(root, _box(Vector3(6.4, 0.3, 4.6)), plank, at + Vector3(0, 4.1, 0))
	_mesh(root, _box(Vector3(4.6, 3.0, 3.4)), boards, at + Vector3(0.4, 5.75, -0.2))
	_mesh(root, _box(Vector3(1.0, 0.9, 0.1)), _unshaded(Color(1.0, 0.86, 0.56)), at + Vector3(1.4, 6.0, 1.52))
	_mesh(root, _box(Vector3(0.9, 1.9, 0.1)), post, at + Vector3(-0.8, 5.2, 1.52))
	# The roof: a low cone of thatch with a ragged lower lip.
	_mesh(root, _cylinder(0.0, 4.4, 2.6, 10), thatch, at + Vector3(0.4, 8.5, -0.2), Vector3(1.0, 1.0, 0.85))
	_mesh(root, _cylinder(4.4, 4.6, 0.35, 10), Materials.wood(Color(0.76, 0.6, 0.34), Vector2(8, 1)), at + Vector3(0.4, 7.1, -0.2), Vector3(1.0, 1.0, 0.85))
	# The jetty: a walk of planks off to the right on pairs of posts.
	_mesh(root, _box(Vector3(22.0, 0.22, 1.6)), plank, at + Vector3(14.0, 4.0, 1.2))
	var x := 5.0
	while x < 25.0:
		for pz in [0.5, 1.9]:
			_mesh(root, _cylinder(0.14, 0.16, 4.4, 6), post, at + Vector3(x, 1.9, pz))
		x += 4.0
	# A rail along the deck's front with a net slung over it.
	_mesh(root, _box(Vector3(6.4, 0.12, 0.12)), post, at + Vector3(0, 5.0, 2.25))
	_mesh(root, _box(Vector3(2.4, 0.9, 0.06)), FighterView.toon(Color(0.36, 0.52, 0.5)), at + Vector3(-1.6, 4.6, 2.3))
	for k in 2:
		var buoy := Bob.new()
		buoy.position = at + Vector3(-4.2 - k * 2.2, 0.1, 2.6 - k * 1.4)
		root.add_child(buoy)
		_mesh(buoy, _sphere(0.5, 10), FighterView.toon(Color(0.95, 0.3, 0.3)), Vector3.ZERO, Vector3(1.0, 0.8, 1.0))
		_mesh(buoy, _cylinder(0.52, 0.52, 0.22, 10), FighterView.toon(Color(0.98, 0.96, 0.92)), Vector3(0, 0.05, 0))


## A palm: a curving trunk of stacked rings under a fan of long leaves and a few coconuts.
static func _palm(root: Node3D, base: Vector3, size: float, rng: RandomNumberGenerator) -> void:
	# (Its own painted ring tile, art/stage/png/palm.png from tools/art/noise_tile.gd, about two rings a scale: the bark tile's vertical
	# plates ran down the narrow trunk as a brown streak, art review 6 round 37; a palm's texture is its rings.)
	var bark := Materials.palm(Color(0.66, 0.5, 0.32), Vector2(2, 0.34))
	var lean := rng.randf_range(-0.12, 0.12)
	var top := base
	# (A trunk of overlapping scales, each a ring wider at its top than its foot, turned along the trunk's curve and narrowing toward
	# the crown, so it reads as one curving palm trunk rather than a stack of drums; a darker ring marks each joint.)
	# (Rings without their own ink and a gentler flare, art review 6 round 35: each scale outlined and flaring 26% read as a stack of cups.)
	var joint := Materials.bark(Color(0.5, 0.36, 0.24), Vector2(1, 1), false)
	var n := 10
	for k in n:
		var u := float(k) / float(n)
		var seg := base + Vector3(lean * k * k * 0.08 * size, k * 0.6 * size, 0)
		var slope := lean * 2.0 * float(k) * 0.08 / 0.6
		var r := lerpf(0.32, 0.22, u) * size
		var tilt := Vector3(0, 0, -rad_to_deg(atan(slope)))
		_mesh(root, _cylinder(r * 1.06, r * 0.94, 0.7 * size, 12), bark, seg + Vector3(0, 0.35 * size, 0), Vector3.ONE, tilt)
		_mesh(root, _cylinder(r * 1.08, r * 1.08, 0.06 * size, 12), joint, seg + Vector3(slope * 0.66 * size, 0.68 * size, 0), Vector3.ONE, tilt)
		top = seg + Vector3(slope * 0.7 * size, 0.7 * size, 0)
	# A flared, rooted foot in the sand.
	_mesh(root, _cylinder(0.3 * size, 0.5 * size, 0.4 * size, 12), bark, base + Vector3(0, 0.1 * size, 0))
	_contact(root, base, Vector2(0.7, 0.55) * size)
	var head := Sway.new()
	head.position = top
	head.amount = 5.0
	head.speed = 1.3
	head.phase = rng.randf() * TAU
	root.add_child(head)
	# The crown of arching, leafleted fronds and coconuts (art/blender/make_props.py), swaying on the trunk.
	var crown := Toonify.load_prop("palm_crown")
	if crown != null:
		crown.scale = Vector3.ONE * size * 1.4
		crown.rotation_degrees = Vector3(0, rng.randf_range(0.0, 360.0), 0)
		head.add_child(crown)


## The city at night: rooftops in the foreground, a lit skyline in layers behind, a big moon, stars, a water tower and an antenna with a
## blinking light, and props on the roof behind the fighters.
static func _city(root: Node3D, t: Dictionary, stage: Array) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 33
	var e := _ext(stage)
	_mesh(root, _sphere(8.0, 32), _unshaded(Color(0.98, 0.94, 0.82)), Vector3(30, 34, -140))
	var star := _unshaded(Color(1, 1, 0.9))
	var dot := _sphere(0.3, 6)
	for k in 70:
		_mesh(root, dot, star, Vector3(rng.randf_range(-140, 140), rng.randf_range(20, 75), rng.randf_range(-160, -130)))
	# Three layers of buildings, darker and smaller the further back, their windows painted on by one facade shader per layer
	# (shaders/facade.gdshader): the nearest layer's lit windows glow, the far ones less.
	for layer in 3:
		# (The nearest layer 52 back, not 40: nearer, its towers stood right behind the fighters. The owner, 2026-10-17.)
		var z := -52.0 - layer * 28.0
		var shade := Color(0.2, 0.17, 0.33).lerp(Color(0.29, 0.22, 0.42), layer / 2.0)
		# (The further layers drift toward the sky's own horizon colour, so distance reads as air, not as three equal cut-outs: the owner's
		# rooftop plan, art review 6.)
		shade = shade.lerp(t.sky_horizon as Color, [0.0, 0.16, 0.34][layer])
		_skyline_layer(root, rng, layer, z, shade, float(e[3]) - CITY_STREET_DROP)
	# A water tower and an antenna on the roof behind the stage.
	var roof: float = e[2]
	var tower := Vector3(float(e[0]) + 3.0, roof, -6.0)
	var wood := Materials.wood(Color(0.55, 0.36, 0.26))
	for leg in [Vector3(-0.9, 0, -0.9), Vector3(0.9, 0, -0.9), Vector3(-0.9, 0, 0.9), Vector3(0.9, 0, 0.9)]:
		_mesh(root, _box(Vector3(0.18, 2.4, 0.18)), wood, tower + leg + Vector3(0, 1.2, 0))
	_mesh(root, _cylinder(1.3, 1.3, 2.2, 14), wood, tower + Vector3(0, 3.5, 0))
	_mesh(root, _cylinder(0.0, 1.5, 1.0, 14), FighterView.toon(Color(0.4, 0.28, 0.22)), tower + Vector3(0, 5.1, 0))
	# Iron hoops round the staves, a hatch on the roof and a ladder up the side: they say what it is made of and how big it is.
	var iron := FighterView.respond(FighterView.toon(Color(0.3, 0.26, 0.32)), "metal")
	for hy in [2.6, 3.5, 4.4]:
		_mesh(root, _cylinder(1.34, 1.34, 0.08, 14), iron, tower + Vector3(0, hy, 0))
	_mesh(root, _box(Vector3(0.5, 0.12, 0.5)), iron, tower + Vector3(0.3, 5.45, 0.55), Vector3.ONE, Vector3(-30, 0, 0))
	for lx in [-0.18, 0.18]:
		_mesh(root, _box(Vector3(0.05, 3.4, 0.05)), iron, tower + Vector3(1.0 + lx, 2.2, 0.95))
	for ly in range(5):
		_mesh(root, _box(Vector3(0.4, 0.04, 0.04)), iron, tower + Vector3(1.0, 0.8 + ly * 0.7, 0.95))
	var mast := Vector3(float(e[1]) - 2.5, roof, -6.5)
	_mesh(root, _cylinder(0.08, 0.15, 6.0, 6), FighterView.respond(FighterView.toon(Color(0.6, 0.62, 0.7)), "metal"), mast + Vector3(0, 3, 0))
	var blink := Blinker.new()
	blink.position = mast + Vector3(0, 6.1, 0)
	root.add_child(blink)
	blink.build(Color(1.0, 0.25, 0.25))
	# Vents and an air conditioner.
	var metal := FighterView.respond(FighterView.toon(Color(0.66, 0.68, 0.76)), "paint")
	var unit := Vector3(float(e[0]) + 6.5, roof, -4.5)
	_mesh(root, _box(Vector3(1.6, 1.0, 1.2)), metal, unit + Vector3(0, 0.5, 0))
	# A lip round its top and a plinth under it: the box gets a lit top edge and sits on something, not straight on the roof.
	var trim := FighterView.respond(FighterView.toon(Color(0.56, 0.58, 0.66)), "metal")
	_mesh(root, _box(Vector3(1.7, 0.08, 1.3)), trim, unit + Vector3(0, 1.0, 0))
	_mesh(root, _box(Vector3(1.8, 0.12, 1.4)), FighterView.toon(Color(0.5, 0.48, 0.58)), unit + Vector3(0, 0.06, 0))
	# Its grille (slats across the front) and a fan on top.
	var slat := FighterView.toon(Color(0.42, 0.42, 0.52), false)
	for k in 5:
		_mesh(root, _box(Vector3(1.3, 0.05, 0.03)), slat, unit + Vector3(0, 0.22 + k * 0.14, 0.61))
	_mesh(root, _cylinder(0.42, 0.42, 0.06, 16), slat, unit + Vector3(0, 1.03, 0))
	_mesh(root, _cylinder(0.12, 0.12, 0.08, 10), metal, unit + Vector3(0, 1.06, 0))
	var vent := Vector3(float(e[1]) - 6.0, roof, -4.0)
	_mesh(root, _cylinder(0.3, 0.3, 0.9, 10), metal, vent + Vector3(0, 0.45, 0))
	# A wider cap on the vent.
	_mesh(root, _cylinder(0.42, 0.42, 0.1, 12), slat, vent + Vector3(0, 0.95, 0))
	_mesh(root, _box(Vector3(0.5, 0.06, 0.5)), slat, mast + Vector3(0, 0.03, 0))
	_contact(root, Vector3(float(e[0]) + 6.5, roof, -4.5), Vector2(0.95, 0.7))
	_contact(root, Vector3(float(e[1]) - 6.0, roof, -4.0), Vector2(0.38, 0.32))
	_contact(root, tower, Vector2(1.2, 1.1))
	_contact(root, mast, Vector2(0.2, 0.18))
	_roof_marks(root, e, rng)


## How far below the stage's lowest block the street is (the buildings of the skyline stand on it too).
const CITY_STREET_DROP := 6.0


## The street the city stands on, at height `y` (the stage building's front at `front`): a pavement along the foot of the stage building,
## the road in front of the nearest row of buildings with a dashed line down it, a pavement under that row with a kerb, street lamps along
## it with warm pools of light, and a few cars driving both ways with their lights on. One flat stretch from far left to far right, so a
## camera pulled right back sees a city that stands on the ground.
static func _street(root: Node3D, y: float, front: float) -> void:
	# (Painted asphalt (godot/art/stage/asphalt.svg: grit, a sealed crack, a patch) laid in world space, and inked stone kerbs: flat fills read as a
	# different, unfinished style beside the painted rooftop.)
	var asphalt := Materials.world("asphalt", Color(0.3, 0.27, 0.4), 6.0, false)
	var pave := Materials.stone(Color(0.46, 0.42, 0.56), Vector2(40, 2))
	var kerb := Materials.stone(Color(0.6, 0.56, 0.7), Vector2(60, 1))
	# The pavement round the stage building, the road, the far pavement and its kerb.
	_mesh(root, _box(Vector3(300.0, 0.5, 6.0)), pave, Vector3(0, y, front + 1.0))
	_mesh(root, _box(Vector3(300.0, 0.5, 22.0)), asphalt, Vector3(0, y - 0.25, front - 18.0))
	_mesh(root, _box(Vector3(300.0, 0.5, 7.0)), pave, Vector3(0, y, front - 32.5))
	for kz in [front + 4.0, front - 7.0, front - 29.0]:
		_mesh(root, _box(Vector3(300.0, 0.18, 0.4)), kerb, Vector3(0, y + 0.2, kz))
	# The line down the middle of the road.
	var paint := _unshaded(Color(0.95, 0.82, 0.46))
	var x := -150.0
	while x < 150.0:
		_mesh(root, _box(Vector3(2.4, 0.04, 0.3)), paint, Vector3(x, y + 0.01, front - 18.0))
		x += 5.0
	# Zebra crossings either side of the stage building, worn a little.
	var zebra := _unshaded(Color(0.86, 0.84, 0.9))
	for cx in [-34.0, 36.0]:
		for j in 7:
			_mesh(root, _box(Vector3(1.0, 0.04, 2.4)), zebra, Vector3(cx + j * 1.7, y + 0.02, front - 11.0 - float(j % 2) * 0.1))
			_mesh(root, _box(Vector3(1.0, 0.04, 2.4)), zebra, Vector3(cx + j * 1.7, y + 0.02, front - 24.8))
	# Street lamps along the far pavement: a post, an arm out over the road, a lamp and a warm halo.
	var iron := FighterView.toon(Color(0.2, 0.18, 0.28))
	var lamp := _unshaded(Color(1.0, 0.9, 0.62))
	x = -132.0
	while x <= 132.0:
		var at := Vector3(x, y + 0.25, front - 29.6)
		_mesh(root, _cylinder(0.14, 0.2, 6.0, 8), iron, at + Vector3(0, 3.0, 0))
		_mesh(root, _box(Vector3(0.12, 0.12, 1.6)), iron, at + Vector3(0, 5.9, 0.7))
		_mesh(root, _sphere(0.32, 10), lamp, at + Vector3(0, 5.7, 1.4), Vector3(1.0, 0.7, 1.0))
		_halo(root, at + Vector3(0, 5.6, 1.6), Color(1.0, 0.8, 0.5), 4.2)
		x += 16.5
	# Cars: two lanes, one each way, a cab among them.
	var colours := [Color(0.92, 0.36, 0.4), Color(0.36, 0.7, 0.86), Color(0.98, 0.8, 0.3), Color(0.62, 0.48, 0.9), Color(0.9, 0.9, 0.94),
		Color(0.4, 0.78, 0.6)]
	for k in 6:
		var dir := 1.0 if k % 2 == 0 else -1.0
		var car := Drift.new()
		car.speed = dir * (7.0 + 2.0 * float(k % 3))
		car.span = 140.0
		car.position = Vector3(-130.0 + k * 47.0, y, front - (13.5 if dir > 0.0 else 22.5))
		root.add_child(car)
		_car(car, colours[k % colours.size()], dir, k == 2)
	_street_furniture(root, y, front)


## A little car pointing along `dir` (+1 right, -1 left), `taxi` for a cab: a body with a bonnet and a boot stepping down from the cabin,
## dark wheel arches over chunky tyres with pale hubs, a windscreen and side windows split by pillars, a door line, grey bumpers,
## headlights with a glow ahead of them and red tail lights; a cab has a lit sign on its roof and a chequer stripe.
static func _car(parent: Node3D, colour: Color, dir: float, taxi := false) -> void:
	var body := FighterView.respond(FighterView.toon(colour), "paint")
	var dark := FighterView.toon(colour.darkened(0.45))
	var glass := _unshaded(Color(0.3, 0.34, 0.52))
	var glint := _unshaded(Color(0.62, 0.7, 0.9))
	var tyre := FighterView.toon(Color(0.12, 0.1, 0.16))
	var hub := FighterView.toon(Color(0.8, 0.8, 0.86))
	var chrome := FighterView.toon(Color(0.66, 0.66, 0.74))
	# Body: the lower shell, the bonnet and boot (a little lower, rounded off at the ends) and the cabin.
	_mesh(parent, _box(Vector3(3.6, 0.8, 1.9)), body, Vector3(0, 0.85, 0))
	for e in [1.0, -1.0]:
		_mesh(parent, _sphere(1.0, 12), body, Vector3(1.75 * e * dir, 0.95, 0), Vector3(0.5, 0.45, 0.95))
	_mesh(parent, _box(Vector3(2.1, 0.85, 1.72)), body, Vector3(-0.2 * dir, 1.65, 0))
	# Glass: the windscreen and rear window as dark panels on the cabin's ends, the side windows split by a pillar, a glint on each.
	_mesh(parent, _box(Vector3(0.08, 0.62, 1.5)), glass, Vector3(0.87 * dir, 1.68, 0))
	_mesh(parent, _box(Vector3(0.08, 0.55, 1.5)), glass, Vector3(-1.27 * dir, 1.68, 0))
	for wx in [-0.62, 0.32]:
		_mesh(parent, _box(Vector3(0.82, 0.55, 0.04)), glass, Vector3((wx - 0.2) * dir, 1.7, 0.87))
		_mesh(parent, _box(Vector3(0.18, 0.4, 0.05)), glint, Vector3((wx - 0.42) * dir, 1.74, 0.88), Vector3.ONE, Vector3(0, 0, 30))
	_mesh(parent, _box(Vector3(0.04, 0.7, 0.03)), dark, Vector3(-0.2 * dir, 0.9, 0.96))
	# Wheel arches, tyres and hubs.
	for wx in [-1.25, 1.25]:
		_mesh(parent, _cylinder(0.56, 0.56, 1.96, 14), dark, Vector3(wx, 0.55, 0), Vector3(1.0, 1.0, 1.0), Vector3(90, 0, 0))
		for wz in [-0.85, 0.85]:
			_mesh(parent, _cylinder(0.44, 0.44, 0.34, 14), tyre, Vector3(wx, 0.44, wz), Vector3.ONE, Vector3(90, 0, 0))
			_mesh(parent, _cylinder(0.2, 0.2, 0.36, 10), hub, Vector3(wx, 0.44, wz), Vector3.ONE, Vector3(90, 0, 0))
	# Bumpers, lights.
	for e in [1.0, -1.0]:
		_mesh(parent, _box(Vector3(0.22, 0.24, 1.96)), chrome, Vector3(2.15 * e * dir, 0.62, 0))
	for lz in [-0.62, 0.62]:
		_mesh(parent, _sphere(0.17, 8), _unshaded(Color(1.0, 0.96, 0.78)), Vector3(2.12 * dir, 0.98, lz), Vector3(0.5, 1.0, 1.0))
		_mesh(parent, _box(Vector3(0.1, 0.2, 0.34)), _unshaded(Color(1.0, 0.24, 0.3)), Vector3(-2.18 * dir, 0.98, lz))
	_halo(parent, Vector3(2.7 * dir, 1.0, 0.95), Color(1.0, 0.92, 0.7), 2.4)
	_halo(parent, Vector3(-2.4 * dir, 1.0, 0.95), Color(1.0, 0.3, 0.3), 1.0)
	if taxi:
		_mesh(parent, _box(Vector3(0.9, 0.32, 0.5)), _unshaded(Color(1.0, 0.94, 0.7)), Vector3(-0.2 * dir, 2.25, 0))
		for k in 8:
			_mesh(parent, _box(Vector3(0.2, 0.12, 0.03)), dark if k % 2 == 0 else _unshaded(Color(0.98, 0.96, 0.9)), Vector3((-1.3 + k * 0.32) * dir, 1.05, 0.96))


## What makes the street a street (all at street height `y`, the stage building's front at `front`; deterministic): zebra crossings
## at two corners with traffic lights on the far kerb, manhole covers and drains in the road, and along the far pavement benches, bins,
## fire hydrants, a bus stop with a lit advert, and small trees in square planters; then the shops along the foot of the nearest row of
## buildings, lit windows under striped awnings with a band of sign over each.
static func _street_furniture(root: Node3D, y: float, front: float) -> void:
	var white := _unshaded(Color(0.92, 0.9, 0.96))
	var iron := FighterView.toon(Color(0.2, 0.18, 0.28))
	var walk := front - 31.5
	for cx in [-46.0, 46.0]:
		for k in 7:
			_mesh(root, _box(Vector3(0.9, 0.04, 18.0)), white, Vector3(cx - 4.5 + k * 1.5, y + 0.01, front - 18.0))
		# A traffic light on the far kerb: a post, a housing, three lamps (green lit).
		var post := Vector3(cx + 6.0, y + 0.25, front - 29.5)
		_mesh(root, _cylinder(0.12, 0.14, 4.4, 8), iron, post + Vector3(0, 2.2, 0))
		_mesh(root, _box(Vector3(0.7, 1.9, 0.6)), iron, post + Vector3(0, 5.0, 0.2))
		for j in 3:
			var lit: Color = [Color(0.4, 0.14, 0.16), Color(0.4, 0.32, 0.12), Color(0.4, 1.0, 0.6)][j]
			_mesh(root, _sphere(0.2, 8), _unshaded(lit), post + Vector3(0, 5.6 - j * 0.6, 0.52))
		_halo(root, post + Vector3(0, 4.4, 0.8), Color(0.4, 1.0, 0.6), 1.4)
	var cover := FighterView.toon(Color(0.16, 0.14, 0.22))
	for mx in [-80.0, -22.0, 18.0, 70.0]:
		_mesh(root, _cylinder(0.7, 0.7, 0.04, 14), cover, Vector3(mx, y + 0.01, front - 15.0 - fmod(absf(mx), 7.0)))
	# Along the far pavement, spaced by a fixed pattern.
	var wood := Materials.wood(Color(0.62, 0.44, 0.3), Vector2(3, 1))
	var leaf := _soft(Color(0.34, 0.62, 0.4))
	var x := -120.0
	var k := 0
	while x < 120.0:
		var at := Vector3(x, y + 0.25, walk)
		match k % 5:
			0:
				# A tree in a planter.
				_mesh(root, _box(Vector3(1.6, 0.6, 1.6)), FighterView.toon(Color(0.5, 0.46, 0.6)), at + Vector3(0, 0.3, 0))
				_mesh(root, _cylinder(0.14, 0.2, 2.6, 6), Materials.bark(Color(0.5, 0.36, 0.28)), at + Vector3(0, 1.6, 0))
				_mesh(root, _sphere(1.0, 12), leaf, at + Vector3(0, 3.4, 0), Vector3(1.4, 1.2, 1.3))
			1:
				# A bench.
				_mesh(root, _box(Vector3(2.4, 0.14, 0.7)), wood, at + Vector3(0, 0.75, 0))
				_mesh(root, _box(Vector3(2.4, 0.6, 0.12)), wood, at + Vector3(0, 1.15, -0.32))
				for lx in [-1.0, 1.0]:
					_mesh(root, _box(Vector3(0.1, 0.75, 0.6)), iron, at + Vector3(lx, 0.38, 0))
			2:
				# A bin and a hydrant.
				_mesh(root, _cylinder(0.45, 0.4, 1.1, 10), FighterView.toon(Color(0.3, 0.5, 0.42)), at + Vector3(-0.8, 0.55, 0))
				_mesh(root, _cylinder(0.5, 0.5, 0.1, 10), iron, at + Vector3(-0.8, 1.12, 0))
				_mesh(root, _cylinder(0.22, 0.26, 0.9, 8), FighterView.toon(Color(0.9, 0.24, 0.26)), at + Vector3(0.9, 0.45, 0.6))
				_mesh(root, _sphere(0.24, 8), FighterView.toon(Color(0.9, 0.24, 0.26)), at + Vector3(0.9, 0.92, 0.6))
				_mesh(root, _cylinder(0.08, 0.08, 0.7, 6), FighterView.toon(Color(0.9, 0.24, 0.26)), at + Vector3(0.9, 0.6, 0.6), Vector3.ONE, Vector3(0, 0, 90))
			3:
				# A bus stop: a roof on two posts, a lit advert at one end.
				for lx in [-1.6, 1.6]:
					_mesh(root, _box(Vector3(0.1, 2.8, 0.1)), iron, at + Vector3(lx, 1.4, -0.4))
				_mesh(root, _box(Vector3(3.6, 0.12, 1.4)), iron, at + Vector3(0, 2.85, 0))
				_mesh(root, _box(Vector3(1.0, 1.8, 0.1)), _unshaded(Color(0.98, 0.6, 0.8)), at + Vector3(1.6, 1.3, -0.4))
				_halo(root, at + Vector3(1.6, 1.3, 0.0), Color(1.0, 0.6, 0.85), 2.2)
				_mesh(root, _box(Vector3(2.4, 0.12, 0.5)), wood, at + Vector3(-0.3, 0.7, -0.3))
		x += 13.0 + 3.0 * float(k % 3)
		k += 1
	# Shops along the foot of the nearest row of buildings (its front is 37.5 back).
	var shop_z := -37.4
	var awnings := [[Color(0.92, 0.34, 0.42), Color(0.98, 0.94, 0.9)], [Color(0.3, 0.6, 0.6), Color(0.94, 0.92, 0.84)],
		[Color(0.96, 0.7, 0.3), Color(0.98, 0.94, 0.86)], [Color(0.5, 0.4, 0.8), Color(0.92, 0.9, 0.98)]]
	# (A step below full white, and the glows round them half the size: art review 6 round 7, the lit shops bloomed into the brightest
	# blobs on screen, down in the corners where the eye lands.)
	var glows := [Color(0.9, 0.76, 0.5), Color(0.6, 0.78, 0.9), Color(0.9, 0.62, 0.74), Color(0.74, 0.88, 0.64)]
	x = -126.0
	k = 0
	while x < 126.0:
		var w := 7.0 + 2.0 * float(k % 3)
		var cx := x + w * 0.5
		_mesh(root, _box(Vector3(w - 0.4, 3.2, 0.2)), FighterView.toon(Color(0.26, 0.2, 0.34)), Vector3(cx, y + 1.85, shop_z))
		# The shop window: lit from inside, warmer at the floor, divided into panes by a frame (art review 6 round 16: one flat pale fill
		# read as a blank sign, not a window).
		var pane: Color = glows[k % glows.size()]
		_mesh(root, _box(Vector3(w * 0.62, 2.0, 0.1)), _unshaded(pane), Vector3(cx - w * 0.12, y + 1.6, shop_z + 0.12))
		_mesh(root, _box(Vector3(w * 0.62, 0.7, 0.1)), _unshaded(pane.lerp(Color(1.0, 0.7, 0.45), 0.35).darkened(0.12)), Vector3(cx - w * 0.12, y + 0.95, shop_z + 0.13))
		var mullion := _unshaded(Color(0.26, 0.2, 0.34))
		for mi_k in 3:
			var mx := cx - w * 0.12 - w * 0.31 + w * 0.62 * float(mi_k + 1) / 4.0
			_mesh(root, _box(Vector3(0.12, 2.0, 0.1)), mullion, Vector3(mx, y + 1.6, shop_z + 0.15))
		_mesh(root, _box(Vector3(w * 0.62, 0.12, 0.1)), mullion, Vector3(cx - w * 0.12, y + 2.1, shop_z + 0.15))
		_mesh(root, _box(Vector3(1.1, 2.4, 0.1)), _unshaded(Color(0.5, 0.36, 0.3)), Vector3(cx + w * 0.32, y + 1.45, shop_z + 0.12))
		var pair: Array = awnings[k % awnings.size()]
		var n := int(w / 0.8)
		for j in n:
			_mesh(root, _box(Vector3((w - 0.4) / n, 0.1, 1.3)), FighterView.toon(pair[j % 2]), Vector3(x + 0.2 + (j + 0.5) * (w - 0.4) / n, y + 3.5, shop_z + 0.55),
				Vector3.ONE, Vector3(24, 0, 0))
		_mesh(root, _box(Vector3(w * 0.5, 0.6, 0.1)), _unshaded((glows[(k + 2) % glows.size()] as Color).lerp(Color.WHITE, 0.2)), Vector3(cx, y + 4.3, shop_z + 0.12))
		x += w
		k += 1


## The ground floor of the stage building where it meets the street (`width` wide, centred on `cx`, its front at `front`): a lit shop
## window either side of a door, under a striped awning, and a sign over it.
static func _shopfront(root: Node3D, cx: float, width: float, y: float, front: float) -> void:
	var frame := FighterView.toon(Color(0.3, 0.22, 0.34))
	var lit := _unshaded(Color(0.9, 0.76, 0.52))
	_mesh(root, _box(Vector3(width, 3.4, 0.2)), frame, Vector3(cx, y + 1.95, front))
	for side in [-1.0, 1.0]:
		_mesh(root, _box(Vector3(width * 0.32, 2.2, 0.1)), lit, Vector3(cx + side * width * 0.27, y + 1.75, front + 0.12))
	_mesh(root, _box(Vector3(1.4, 2.6, 0.1)), _unshaded(Color(0.62, 0.42, 0.3)), Vector3(cx, y + 1.55, front + 0.12))
	# The awning: alternating stripes, tipped out over the pavement.
	var stripes := [FighterView.toon(Color(0.92, 0.34, 0.42)), FighterView.toon(Color(0.98, 0.94, 0.9))]
	var n := int(width / 0.9)
	for k in n:
		var sx := cx - width / 2.0 + (k + 0.5) * width / n
		_mesh(root, _box(Vector3(width / n, 0.12, 1.4)), stripes[k % 2], Vector3(sx, y + 3.55, front + 0.6), Vector3.ONE, Vector3(22, 0, 0))
	_mesh(root, _box(Vector3(width * 0.4, 0.8, 0.12)), _unshaded(Color(0.98, 0.5, 0.74)), Vector3(cx, y + 4.4, front + 0.14))
	_halo(root, Vector3(cx, y + 1.8, front + 0.8), Color(1.0, 0.8, 0.5), width * 0.35)


## Weathering on the rooftop behind the fighters: a water stain, a puddle with the sky in it, a tar patch and some grime
## (godot/art/stage/roof_detail.svg, four marks side by side), each a decal laid on the roof, turned and sized a little differently.
static func _roof_marks(root: Node3D, e: Array, rng: RandomNumberGenerator) -> void:
	var marks := _roof_mark_textures()
	if marks.is_empty():
		return
	var roof: float = e[2]
	var left: float = e[0]
	var right: float = e[1]
	var spots := [[0.18, -2.2, 0], [0.42, -4.6, 1], [0.66, -1.8, 2], [0.86, -3.8, 3], [0.3, -5.6, 3]]
	# (Flat quads that multiply the roof by the mark, not decals: a decal's box showed as a pale rectangle under the practical lights, and
	# a multiplied stain takes whatever light falls on the roof under it.)
	for spot in spots:
		var k := rng.randf_range(2.2, 3.4)
		var q := QuadMesh.new()
		q.size = Vector2(k, k * 0.8)
		q.orientation = PlaneMesh.FACE_Y
		var m := ShaderMaterial.new()
		m.shader = _stain_shader()
		m.set_shader_parameter("mark", marks[int(spot[2])])
		var mi := _mesh(root, q, m, Vector3(lerpf(left, right, float(spot[0])), roof + 0.035, float(spot[1])), Vector3.ONE,
			Vector3(0, rad_to_deg(rng.randf_range(-0.5, 0.5)), 0))
		mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF


static var _stain: Shader


## A stain laid on a surface: multiplies what is under it by the mark's colour where the mark is opaque, and leaves it alone elsewhere.
static func _stain_shader() -> Shader:
	if _stain == null:
		_stain = Shader.new()
		_stain.code = """shader_type spatial;
render_mode unshaded, blend_mul, depth_draw_never, cull_disabled, shadows_disabled, fog_disabled;
uniform sampler2D mark : source_color, filter_linear_mipmap;
void fragment() {
	vec4 c = texture(mark, UV);
	ALBEDO = mix(vec3(1.0), c.rgb, c.a);
}
"""
	return _stain


static var _roof_marks_cache: Array = []


static func _roof_mark_textures() -> Array:
	if not _roof_marks_cache.is_empty():
		return _roof_marks_cache
	var sheet := SvgArt.texture("res://art/stage/roof_detail.svg")
	if sheet == null:
		return []
	var img := sheet.get_image()
	img.clear_mipmaps()
	var w := img.get_width() / 4
	for k in 4:
		var part := img.get_region(Rect2i(k * w, 0, w, img.get_height()))
		part.generate_mipmaps()
		_roof_marks_cache.append(ImageTexture.create_from_image(part))
	return _roof_marks_cache


const FACADE_SHADER := preload("res://shaders/facade.gdshader")
## How bright each skyline layer's lit windows are (times white; the glow threshold is 2.3): the near layer blooms softly, the others do not
## (emissive, not bloomed: the background should not outshine the fighters).
## (Art review 6, refinement round 1: all three layers were near-white and the near one bloomed right behind the fighters, a field of
## pale rectangles that was the noisiest backdrop in the game. Now none blooms, and each layer back is dimmer.)
const FACADE_LIT := [1.15, 0.8, 0.55]
## The share of a skyline layer's windows that are lit (nearest first): a scattered glow behind the fighters, not a field of lights that
## outshines them (the near layers stand right behind the play area).
## (Fewer lit the further back, as distance would hide them: the far layer was the most lit, 36%.)
const FACADE_LIT_SHARE := [0.18, 0.13, 0.09]


## One layer of the city's backdrop at depth `z`, its streets at `street`: a row of buildings (boxes with a parapet, some with rooftop
## vents, tanks and an antenna) built as one mesh, windows and all painted by `facade.gdshader`. Faces carry their own coordinates (see
## the shader). One mesh and one material per layer, so it costs a draw call or two however many windows it shows.
static func _skyline_layer(root: Node3D, rng: RandomNumberGenerator, layer: int, z: float, shade: Color, street: float) -> void:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var depth := 5.0
	# Signs on some of the nearer buildings, from their own random numbers (so the skyline's layout is the same with or without them).
	var signs := RandomNumberGenerator.new()
	signs.seed = 77 + layer
	# Two rooftop billboards on the middle layer, on the buildings nearest these places (in view, above the stage's roofline).
	var billboard_at := [-20.0, 24.0] if layer == 1 else []
	var billboard_best := {}
	var x := -120.0
	while x < 120.0:
		var w := rng.randf_range(6.0, 13.0)
		var h := rng.randf_range(10.0, 34.0) * (1.0 - layer * 0.15)
		var seed := rng.randf()
		var lo := Vector3(x, street, z - depth / 2.0)
		var hi := Vector3(x + w, street + h, z + depth / 2.0)
		_facade_box(st, lo, hi, seed, false, h)
		# The roofline: a cornice lip a little proud all round, and on some taller buildings a narrower setback storey or two above it,
		# so the skyline's silhouette steps instead of being a row of plain boxes. (Its own random numbers, so the layout is unchanged.)
		var shape := RandomNumberGenerator.new()
		shape.seed = hash(Vector2i(roundi(x * 10.0), layer))
		_facade_box(st, Vector3(lo.x - 0.25, hi.y - 0.35, lo.z - 0.25), Vector3(hi.x + 0.25, hi.y + 0.1, hi.z + 0.25), seed, true, 0.45)
		var set_top := hi.y + 0.1
		# (More of them, and a second, narrower step on the tallest: a skyline of stepped crowns, not flat-topped boxes.)
		if h > 14.0 and shape.randf() < 0.68:
			var inset := w * shape.randf_range(0.18, 0.3)
			var sh := shape.randf_range(2.6, 5.5)
			_facade_box(st, Vector3(lo.x + inset, set_top, lo.z + 0.6), Vector3(hi.x - inset, set_top + sh, hi.z - 0.6), seed, false, sh)
			_facade_box(st, Vector3(lo.x + inset - 0.2, set_top + sh - 0.3, lo.z + 0.4), Vector3(hi.x - inset + 0.2, set_top + sh + 0.1, hi.z - 0.4), seed, true, 0.4)
			if h > 24.0 and shape.randf() < 0.55:
				var in2 := inset + w * 0.14
				var t2 := set_top + sh + 0.1
				var sh2 := shape.randf_range(1.8, 3.5)
				_facade_box(st, Vector3(lo.x + in2, t2, lo.z + 1.0), Vector3(hi.x - in2, t2 + sh2, hi.z - 1.0), seed, false, sh2)
				_facade_box(st, Vector3(lo.x + in2 - 0.15, t2 + sh2 - 0.25, lo.z + 0.85), Vector3(hi.x - in2 + 0.15, t2 + sh2 + 0.1, hi.z - 0.85), seed, true, 0.35)
		if layer < 2 and signs.randf() < 0.34:
			# A lit sign on the front: a tall blade sign near one side, or a wide one across the middle.
			var front := hi.z + 0.08
			var tall := signs.randf() < 0.5
			var sw := 1.7 if tall else minf(w * 0.6, 6.0)
			var sh := minf(h * 0.4, 6.0) if tall else 1.8
			var sx := lo.x + (0.8 if signs.randf() < 0.5 else w - 0.8 - sw) if tall else lo.x + (w - sw) * 0.5
			var sy := street + h * signs.randf_range(0.35, 0.7)
			_facade_quad(st, [Vector3(sx, sy, front), Vector3(sx + sw, sy, front), Vector3(sx + sw, sy + sh, front), Vector3(sx, sy + sh, front)],
				Vector3(0, 0, 1), Vector2(sw, sh), 0.0, Color(signs.randf(), 0.0, 1.0))
		for b in billboard_at:
			var d := absf(x + w / 2.0 - float(b))
			if not billboard_best.has(b) or d < float(billboard_best[b][0]):
				billboard_best[b] = [d, x, w, street + h, seed]
		# Something on the roof now and then: a vent box, a round tank on legs (a box here, at this distance) or an antenna.
		var top := street + h
		var n := rng.randi_range(0, 2)
		for k in n:
			var kind := rng.randi_range(0, 2)
			var cx := rng.randf_range(x + 1.0, x + w - 1.0)
			match kind:
				0:
					var s := Vector3(rng.randf_range(1.0, 2.2), rng.randf_range(0.6, 1.2), rng.randf_range(1.0, 2.0))
					_facade_box(st, Vector3(cx - s.x / 2.0, top, z - s.z / 2.0), Vector3(cx + s.x / 2.0, top + s.y, z + s.z / 2.0), seed, true, s.y)
				1:
					_facade_box(st, Vector3(cx - 1.0, top + 1.2, z - 1.0), Vector3(cx + 1.0, top + 3.0, z + 1.0), seed, true, 1.8)
					_facade_box(st, Vector3(cx - 0.8, top, z - 0.1), Vector3(cx - 0.6, top + 1.2, z + 0.1), seed, true, 1.2)
					_facade_box(st, Vector3(cx + 0.6, top, z - 0.1), Vector3(cx + 0.8, top + 1.2, z + 0.1), seed, true, 1.2)
				_:
					_facade_box(st, Vector3(cx - 0.08, top, z - 0.08), Vector3(cx + 0.08, top + rng.randf_range(2.5, 5.0), z + 0.08), seed, true, 4.0)
		x += w + rng.randf_range(0.5, 3.0)
	for b in billboard_best.values():
		var bx: float = b[1]
		var bw := minf(float(b[2]) * 0.75, 7.0)
		var left: float = bx + (float(b[2]) - bw) * 0.5
		var top: float = b[3]
		# Two legs and the sign standing on them, facing the stage.
		for lx in [left + bw * 0.2, left + bw * 0.8]:
			_facade_box(st, Vector3(lx - 0.1, top, z - 0.1), Vector3(lx + 0.1, top + 1.2, z + 0.1), float(b[4]), true, 1.2)
		_facade_quad(st, [Vector3(left, top + 1.2, z + 0.12), Vector3(left + bw, top + 1.2, z + 0.12), Vector3(left + bw, top + 3.4, z + 0.12),
			Vector3(left, top + 3.4, z + 0.12)], Vector3(0, 0, 1), Vector2(bw, 2.2), 0.0, Color(signs.randf(), 0.0, 1.0))
	var m := ShaderMaterial.new()
	m.shader = FACADE_SHADER
	m.set_shader_parameter("base_shade", shade)
	m.set_shader_parameter("roof_shade", shade.lightened(0.12))
	m.set_shader_parameter("lit_energy", FACADE_LIT[layer])
	m.set_shader_parameter("lit_share", FACADE_LIT_SHARE[layer])
	# The far layer: windows only (frames, sills and floor bands would be noise at that distance).
	m.set_shader_parameter("detail", 0.0 if layer == 2 else 1.0)
	m.set_shader_parameter("sign_energy", 3.0 - layer * 0.8)
	var mi := MeshInstance3D.new()
	mi.mesh = st.commit()
	mi.material_override = m
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	mi.name = "skyline_%d" % layer
	root.add_child(mi)


## A box from `lo` to `hi` into `st` for the facade shader: its front and sides as walls (windows unless `plain`), its top plain. Each
## face's UV runs in world units across it and up from the street; UV2 holds the face's width and the building's height `h`.
static func _facade_box(st: SurfaceTool, lo: Vector3, hi: Vector3, seed: float, plain: bool, h: float) -> void:
	var col := Color(seed, 1.0 if plain else 0.0, 0.0)
	var w := hi.x - lo.x
	var d := hi.z - lo.z
	var y0 := 0.0 if not plain else 0.0
	# Front (+z), left (-x), right (+x), top: corners bottom-left, bottom-right, top-right, top-left seen from outside.
	_facade_quad(st, [Vector3(lo.x, lo.y, hi.z), Vector3(hi.x, lo.y, hi.z), Vector3(hi.x, hi.y, hi.z), Vector3(lo.x, hi.y, hi.z)], Vector3(0, 0, 1),
		Vector2(w, h), y0, col)
	_facade_quad(st, [Vector3(lo.x, lo.y, lo.z), Vector3(lo.x, lo.y, hi.z), Vector3(lo.x, hi.y, hi.z), Vector3(lo.x, hi.y, lo.z)], Vector3(-1, 0, 0),
		Vector2(d, h), y0, col)
	_facade_quad(st, [Vector3(hi.x, lo.y, hi.z), Vector3(hi.x, lo.y, lo.z), Vector3(hi.x, hi.y, lo.z), Vector3(hi.x, hi.y, hi.z)], Vector3(1, 0, 0),
		Vector2(d, h), y0, col)
	_facade_quad(st, [Vector3(lo.x, hi.y, hi.z), Vector3(hi.x, hi.y, hi.z), Vector3(hi.x, hi.y, lo.z), Vector3(lo.x, hi.y, lo.z)], Vector3(0, 1, 0),
		Vector2(w, d), y0, Color(seed, 1.0, 0.0))


static func _facade_quad(st: SurfaceTool, c: Array, normal: Vector3, size: Vector2, _y0: float, col: Color) -> void:
	var uvs := [Vector2(0, 0), Vector2(size.x, 0), Vector2(size.x, size.y), Vector2(0, size.y)]
	for i in [0, 2, 1, 0, 3, 2]:
		st.set_color(col)
		st.set_normal(normal)
		st.set_uv(uvs[i])
		st.set_uv2(size)
		st.add_vertex(c[i])


## A big round cartoon tree: a tapered trunk with bark grooves and roots, under a canopy of soft green balls.
static func _tree(root: Node3D, base: Vector3, size: float, rng: RandomNumberGenerator) -> void:
	var trunk_h := 18.0 * size
	# Bark drawn on (grooves and knots, `Materials.bark`) rather than built of boxes, on one trunk that bows gently as it rises, tapers,
	# and flares into broad roots at its foot.
	# (Broad plates, art review 6 round 28: repeated five times round the trunk the furrows were a busy fine pinstripe beside the
	# foreground's few broad soft forms.)
	var bark := Materials.bark(Color(0.62, 0.42, 0.26), Vector2(2.5, 2))
	_mesh(root, trunk_mesh(trunk_h, 2.9 * size, 2.0 * size, 0.7 * size, 1.4 * size), bark, base)
	for k in 6:
		var a := -1.25 + k * 0.5
		var reach := rng.randf_range(0.85, 1.15)
		_mesh(root, _cylinder(0.25 * size, 1.15 * size * reach, 3.4 * size * reach, 10), bark,
			base + Vector3(sin(a) * 3.2 * size, 0.6 * size, cos(a) * 2.3 * size), Vector3.ONE, Vector3(0, 0, -rad_to_deg(a) * 0.6))
	# Four coordinated greens: deep shade, body, light, and sunlit tips.
	# (Deeper, art review 6 round 27: from the lighter base the crown's lit masses stepped up to near-white greens and the tree read chalky.)
	var g := Materials.greens(Color(0.18, 0.46, 0.22))
	var leaf := Materials.foliage(g[0])
	var leaf_mid := Materials.foliage(g[1], Vector2(3, 2), false)
	var leaf_light := Materials.foliage(g[2], Vector2(2, 1), false)
	var leaf_sun := Materials.foliage(g[3], Vector2(2, 1), false)
	var sway := Sway.new()
	sway.position = base + Vector3(0, trunk_h, 0)
	sway.amount = 0.8
	sway.speed = 0.5
	root.add_child(sway)
	var crown := Vector3(0, 3.0 * size, 0)
	# The crown: fifteen big masses round the top of the trunk and eight smaller sunlit ones in front, each a cluster of leaf cards that
	# shades as one rounded volume (`_leaf_cards`). (The sphere materials above stay for the cards' colours.)
	var masses := []
	for k in 15:
		var a := TAU * k / 15.0
		var r := rng.randf_range(4.0, 9.0) * size
		var off := Vector3(cos(a) * r * 1.3, sin(a) * r * 0.55 + rng.randf_range(-1.0, 2.0) * size, rng.randf_range(-2.0, 1.5) * size)
		masses.append([crown + off, rng.randf_range(4.0, 6.5) * size, 30])
	for k in 8:
		var off := Vector3(rng.randf_range(-8.0, 8.0), rng.randf_range(1.5, 6.0), rng.randf_range(1.0, 3.0)) * size
		masses.append([crown + off, rng.randf_range(2.2, 3.6) * size, 14])
	_leaf_cards(sway, masses, g, 3.6 * size, rng, "leaf_cluster", false, leaf_mid)
	var _unused := [leaf, leaf_light, leaf_sun]


## Four-pointed glints that slowly turn and pulse. Cosmetic; they run on their own clock.
class Twinkle extends Node3D:
	var glints: Array = []  # [node, phase]

	func add(at: Vector3, size: float, phase: float, color: Color) -> void:
		var mi := MeshInstance3D.new()
		mi.mesh = _star()
		var m := StandardMaterial3D.new()
		m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		m.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		m.cull_mode = BaseMaterial3D.CULL_DISABLED
		m.albedo_color = color
		mi.material_override = m
		mi.position = at
		mi.scale = Vector3.ONE * size
		add_child(mi)
		glints.append([mi, phase, size])

	func _process(delta: float) -> void:
		var t := Time.get_ticks_msec() / 1000.0
		for g in glints:
			var k := 0.6 + 0.4 * sin(t * 1.7 + g[1])
			g[0].scale = Vector3.ONE * g[2] * k
			g[0].rotation.z = t * 0.4 + g[1]

	static func _star() -> ArrayMesh:
		var v := PackedVector3Array()
		for k in 8:
			var a0 := TAU * k / 8.0
			var a1 := TAU * (k + 1) / 8.0
			var r0 := 1.0 if k % 2 == 0 else 0.28
			var r1 := 1.0 if (k + 1) % 2 == 0 else 0.28
			v.append(Vector3.ZERO)
			v.append(Vector3(cos(a0) * r0, sin(a0) * r0, 0))
			v.append(Vector3(cos(a1) * r1, sin(a1) * r1, 0))
		var arrays := []
		arrays.resize(Mesh.ARRAY_MAX)
		arrays[Mesh.ARRAY_VERTEX] = v
		var m := ArrayMesh.new()
		m.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
		return m



## Turns slowly around its z axis (a windmill's sails). Cosmetic.
## Scenery that moves (cosmetic, on its own clock).
## Drifts slowly sideways and wraps round (clouds).
class Drift extends Node3D:
	var speed := 1.0
	var span := 100.0

	func _process(delta: float) -> void:
		position.x += speed * delta
		if position.x > span:
			position.x -= span * 2.0
		elif position.x < -span:
			position.x += span * 2.0


## Rocks gently to and fro about its own place (tree crowns, palm heads). `amount` in degrees.
class Sway extends Node3D:
	var amount := 2.0
	var speed := 0.9
	var phase := 0.0
	var t := 0.0

	func _process(delta: float) -> void:
		t += delta
		rotation.z = deg_to_rad(amount) * sin(t * speed + phase)
		rotation.x = deg_to_rad(amount * 0.4) * sin(t * speed * 0.7 + phase * 1.3)


## A gentle squash and stretch about the node's base, as if breathing (mushrooms).
class Squish extends Node3D:
	var phase := 0.0
	var t := 0.0

	func _process(delta: float) -> void:
		t += delta
		var k := sin(t * 2.2 + phase) * 0.035
		scale = Vector3(1.0 - k * 0.5, 1.0 + k, 1.0 - k * 0.5)


## Bobs on the water and rolls a little (a boat).
class Bob extends Node3D:
	var t := 0.0
	var base_y := INF

	func _process(delta: float) -> void:
		if base_y == INF:
			base_y = position.y
		t += delta
		position.y = base_y + 0.35 * sin(t * 1.1)
		rotation.z = deg_to_rad(4.0) * sin(t * 0.9 + 0.6)


## A small flock of birds crossing the sky, wings flapping, coming round again after a while.
class Flock extends Node3D:
	var birds: Array = []   # [node, wing L, wing R, phase]
	var speed := 4.0
	var t := 0.0

	func build(count: int, colour: Color, rng: RandomNumberGenerator) -> void:
		var mat := StandardMaterial3D.new()
		mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		mat.albedo_color = colour
		mat.cull_mode = BaseMaterial3D.CULL_DISABLED
		var wing := BoxMesh.new()
		wing.size = Vector3(1.0, 0.16, 0.3)
		for k in count:
			var bird := Node3D.new()
			bird.position = Vector3(-k * 2.2 + rng.randf_range(-0.5, 0.5), absf(k - count / 2.0) * -1.0 + rng.randf_range(-0.4, 0.4), 0)
			add_child(bird)
			var parts := []
			for side in [-1.0, 1.0]:
				var hinge := Node3D.new()
				bird.add_child(hinge)
				var w := MeshInstance3D.new()
				w.mesh = wing
				w.material_override = mat
				w.position = Vector3(side * 0.5, 0, 0)
				hinge.add_child(w)
				parts.append(hinge)
			birds.append([bird, parts[0], parts[1], rng.randf() * TAU])

	func _process(delta: float) -> void:
		t += delta
		position.x += speed * delta
		if position.x > 110.0:
			position.x = -110.0
		for b in birds:
			var flap := sin(t * 9.0 + float(b[3])) * 0.6
			(b[1] as Node3D).rotation.z = -flap
			(b[2] as Node3D).rotation.z = flap


class Spinner extends Node3D:
	var speed := 0.5

	func _process(delta: float) -> void:
		rotation.z += speed * delta


## A small light that blinks (an antenna's warning light). Cosmetic.
class Blinker extends Node3D:
	var lamp: MeshInstance3D

	func build(color: Color) -> void:
		lamp = MeshInstance3D.new()
		var s := SphereMesh.new()
		s.radius = 0.25
		s.height = 0.5
		lamp.mesh = s
		var m := StandardMaterial3D.new()
		m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		m.albedo_color = color
		lamp.material_override = m
		add_child(lamp)
		# Its glow blinks with it.
		load("res://scripts/stage_art.gd")._halo(lamp, Vector3(0, 0, 0.3), color, 1.6)

	func _process(_delta: float) -> void:
		if lamp != null:
			lamp.visible = int(Time.get_ticks_msec() / 700) % 2 == 0


## The sea: a wide plane at `level` that swells gently, with strips of foam drifting on it. Cosmetic.
class Waves extends Node3D:
	var level := -9.0
	var foam: Array = []

	## The sea (shaders/water.gdshader): banded colour, drifting streaks, a swell, glints and foam where things stand in it; the Fast
	## tier gets the bands only.
	func build() -> void:
		var water := MeshInstance3D.new()
		var plane := PlaneMesh.new()
		plane.size = Vector2(500, 260)
		plane.subdivide_width = 80
		plane.subdivide_depth = 40
		water.mesh = plane
		var m := ShaderMaterial.new()
		m.shader = load("res://shaders/water.gdshader")
		m.set_shader_parameter("cheap", load("res://scripts/visual_settings.gd").get_index("quality") == 0)
		# After the finishing pass (-100) but before every other see-through thing (name tags, effects, halos at 0), which it would
		# otherwise paint over.
		m.render_priority = -50
		water.material_override = m
		water.position = Vector3(0, level, -100)
		water.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		add_child(water)

	func _process(_delta: float) -> void:
		var t := Time.get_ticks_msec() / 1000.0
		position.y = sin(t * 0.8) * 0.12
