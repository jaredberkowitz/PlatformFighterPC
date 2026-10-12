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
const RIDGE_SHADER := preload("res://shaders/painted_ridge.gdshader")
const FOLIAGE_SHADER := preload("res://shaders/toon_foliage.gdshader")
const VisualSettings := preload("res://scripts/visual_settings.gd")

## The look of one theme: sky colours (top, horizon), block colours and pattern (`diamonds`, `strata`, `cobble` or `facade`), what tops a
## block (`grass`, `sand` or `roof`), and the platforms (`wood` planks or steel `girder`s) and their colours.
const THEMES := {
	"meadow": {
		"sky_top": Color(0.32, 0.55, 0.9), "sky_horizon": Color(0.8, 0.92, 0.98),
		"soil": Color(0.62, 0.46, 0.32), "soil_dark": Color(0.52, 0.37, 0.25), "grass": Color(0.42, 0.76, 0.36),
		"grass_light": Color(0.62, 0.88, 0.46), "wood": Color(0.86, 0.62, 0.36), "wood_dark": Color(0.62, 0.42, 0.24),
		"pattern": "cobble", "cap": "grass", "platform": "wood",
	},
	"grove": {
		"sky_top": Color(0.42, 0.6, 0.96), "sky_horizon": Color(0.99, 0.86, 0.93),
		"soil": Color(0.86, 0.66, 0.38), "soil_dark": Color(0.72, 0.5, 0.27), "grass": Color(0.36, 0.76, 0.36),
		"grass_light": Color(0.58, 0.9, 0.45), "wood": Color(0.93, 0.74, 0.5), "wood_dark": Color(0.7, 0.48, 0.28),
		"pattern": "diamonds", "cap": "grass", "platform": "wood",
	},
	"sunset": {
		"sky_top": Color(0.36, 0.28, 0.6), "sky_horizon": Color(1.0, 0.64, 0.4),
		"soil": Color(0.86, 0.5, 0.32), "soil_dark": Color(0.68, 0.34, 0.24), "grass": Color(0.93, 0.74, 0.46),
		"grass_light": Color(0.98, 0.86, 0.6), "wood": Color(0.56, 0.37, 0.28), "wood_dark": Color(0.38, 0.24, 0.2),
		"pattern": "strata", "cap": "sand", "platform": "wood",
	},
	"night": {
		"sky_top": Color(0.05, 0.07, 0.2), "sky_horizon": Color(0.2, 0.25, 0.48),
		"soil": Color(0.36, 0.4, 0.56), "soil_dark": Color(0.27, 0.3, 0.44), "grass": Color(0.38, 0.62, 0.6),
		"grass_light": Color(0.52, 0.78, 0.74), "wood": Color(0.62, 0.66, 0.78), "wood_dark": Color(0.42, 0.45, 0.58),
		"pattern": "diamonds", "cap": "grass", "platform": "wood",
	},
	"ocean": {
		"sky_top": Color(0.2, 0.52, 0.92), "sky_horizon": Color(0.74, 0.92, 1.0),
		"soil": Color(0.62, 0.58, 0.54), "soil_dark": Color(0.5, 0.46, 0.44), "grass": Color(0.98, 0.88, 0.62),
		"grass_light": Color(1.0, 0.95, 0.78), "wood": Color(0.8, 0.62, 0.42), "wood_dark": Color(0.58, 0.42, 0.28),
		"pattern": "cobble", "cap": "sand", "platform": "wood",
	},
	"city": {
		"sky_top": Color(0.08, 0.06, 0.22), "sky_horizon": Color(0.42, 0.22, 0.46),
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
static func sky(t: Dictionary) -> Sky:
	var mat := ProceduralSkyMaterial.new()
	mat.sky_top_color = t.sky_top
	mat.sky_horizon_color = t.sky_horizon
	mat.ground_horizon_color = t.sky_horizon
	mat.ground_bottom_color = (t.sky_horizon as Color).darkened(0.2)
	mat.sun_angle_max = 0.0
	mat.sky_curve = 0.12
	var s := Sky.new()
	s.sky_material = mat
	return s


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
static func _soil_material(t: Dictionary) -> ShaderMaterial:
	var a: Color = t.soil
	var b: Color = t.soil_dark
	var pattern: String = t.get("pattern", "diamonds")
	var key := "%s%s%s" % [pattern, a.to_html(), b.to_html()]
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
			# Wavy bands in four shades, each edge a gentle wave with a soft ink line.
			var shades := [A, B, a.lerp(Color(1, 0.88, 0.7), 0.3).to_html(false), b.darkened(0.1).to_html(false)]
			for k in 4:
				var y0 := k * 64.0
				var d := "M0,%.1f " % y0
				for x in range(0, 257, 32):
					d += "L%d,%.1f " % [x, y0 + sin(float(x) / 256.0 * TAU + k) * 9.0]
				d += "L256,%.1f L0,%.1f Z" % [y0 + 64.0 + 12.0, y0 + 64.0 + 12.0]
				body += '<path d="%s" fill="#%s" stroke="#%s" stroke-width="2.5" stroke-linejoin="round"/>' % [d, shades[k], ink]
		"cobble":
			# Rounded stones over dark mortar, each with a lighter top; stones at an edge are drawn again across it, so it tiles.
			body += '<rect width="256" height="256" fill="#%s"/>' % b.darkened(0.25).to_html(false)
			for k in 16:
				var cx := rng.randf_range(0.0, 256.0)
				var cy := rng.randf_range(0.0, 256.0)
				var rx := rng.randf_range(30.0, 44.0)
				var ry := rx * rng.randf_range(0.7, 0.9)
				var shade := a.lerp(b, rng.randf_range(0.0, 0.5))
				for ox in [-256.0, 0.0, 256.0]:
					for oy in [-256.0, 0.0, 256.0]:
						body += '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#%s" stroke="#%s" stroke-width="3"/>' % [cx + ox, cy + oy, rx, ry, shade.to_html(false), ink]
						body += '<ellipse cx="%.1f" cy="%.1f" rx="%.1f" ry="%.1f" fill="#%s" opacity="0.55"/>' % [cx + ox - rx * 0.15, cy + oy - ry * 0.3, rx * 0.55, ry * 0.4, shade.lightened(0.15).to_html(false)]
		"facade":
			# Concrete panels with rounded windows, some lit warm, each with a sill. Twice the size of the other patterns (sixteen windows
			# a repeat, about a third lit, in a softer warm), so the wall under the fighters is not a busy grid stamped every few units.
			body += '<rect width="512" height="512" fill="#%s"/>' % A
			for x in [0, 128, 256, 384]:
				for y in [0, 128, 256, 384]:
					body += '<rect x="%d" y="%d" width="128" height="128" fill="none" stroke="#%s" stroke-width="4"/>' % [x, y, B]
			for wy in [22, 150, 278, 406]:
				for wx in [18, 74, 146, 202, 274, 330, 402, 458]:
					var lit := rng.randf() < 0.32
					var col := "f2d394" if lit else "2a2f45"
					body += '<rect x="%d" y="%d" width="36" height="56" rx="6" fill="#%s" stroke="#%s" stroke-width="3"/>' % [wx, wy, col, ink]
					body += '<rect x="%d" y="%d" width="44" height="6" rx="3" fill="#%s"/>' % [wx - 4, wy + 58, hi]
		_:
			# Quilted diamonds with a soft highlight along their upper edges.
			body += '<rect width="256" height="256" fill="#%s"/>' % B
			for cx in [0, 128, 256]:
				for cy in [0, 128, 256]:
					for c in [[cx, cy], [cx - 64, cy - 64]]:
						var x: float = c[0]
						var y: float = c[1]
						body += '<path d="M%.0f,%.0f L%.0f,%.0f L%.0f,%.0f L%.0f,%.0f Z" fill="#%s" stroke="#%s" stroke-width="3" stroke-linejoin="round"/>' % [x, y - 56, x + 56, y, x, y + 56, x - 56, y, A, ink]
						body += '<path d="M%.0f,%.0f L%.0f,%.0f" stroke="#%s" stroke-width="5" stroke-linecap="round" opacity="0.6"/>' % [x - 40, y - 8, x - 8, y - 40, hi]
	var px := 512 if pattern == "facade" else 256
	var svg := '<svg xmlns="http://www.w3.org/2000/svg" width="%d" height="%d" viewBox="0 0 %d %d">%s</svg>' % [px, px, px, px, body]
	var img := Image.new()
	img.load_svg_from_string(svg, 1.0)
	img.generate_mipmaps()
	var tex := ImageTexture.create_from_image(img)
	# On the stage's cel shading, laid in world space (one repeat every 2.2 units, a facade's every 6.6: twice the pixels, same scale).
	var m := FighterView.toon(Color.WHITE, true, tex, Vector2.ONE, 2.2 if pattern != "facade" else 6.6)
	# Painted wear over the pattern, so a block's front reads as one weathered mass rather than a stamped tile.
	match pattern:
		"facade":
			Materials.set_detail(m, "concrete", 0.45, 3.0)
		"cobble":
			Materials.set_detail(m, "stone", 0.4, 3.5)
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
	match t.get("cap", "grass"):
		"roof":
			# A concrete ledge with a bright metal trim along the front.
			# (The ledge sits a hair above the block's top so their top faces never fight over the same depth and flicker.)
			_mesh(parent, _box(Vector3(w + 0.3, 0.35, depth + 0.3)), _surface(t.grass, "roof", 4.0), Vector3(cx, top - 0.15, front_z - depth / 2.0))
			_mesh(parent, _box(Vector3(w + 0.34, 0.1, 0.12)), _soft(t.grass_light), Vector3(cx, top - 0.05, front_z + 0.18))
			_mesh(parent, _box(Vector3(w + 0.3, 0.12, 0.12)), _unshaded(Color(0.95, 0.75, 0.3)), Vector3(cx, top - 0.42, front_z + 0.16))
		"sand":
			var sand := _surface(t.grass, "sand", 5.0)
			_mesh(parent, _box(Vector3(w + 0.2, 0.45, depth + 0.1)), sand, Vector3(cx, top - 0.2, front_z - depth / 2.0))
			_mesh(parent, _cylinder(0.26, 0.26, w + 0.2), sand, Vector3(cx, top - 0.26, front_z + 0.02), Vector3.ONE, Vector3(0, 0, 90))
			_mesh(parent, _box(Vector3(w + 0.24, 0.1, 0.2)), _soft(t.grass_light), Vector3(cx, top - 0.05, front_z + 0.24))
			# A few pebbles and shells on the back of the sand.
			var pebble := Materials.stone((t.soil as Color).lightened(0.2))
			var x := left + 0.7
			while x < right - 0.5:
				_mesh(parent, _sphere(0.14, 8), pebble, Vector3(x, top + 0.04, front_z - rng.randf_range(2.0, 5.0)), Vector3(1.3, 0.6, 1.0))
				x += rng.randf_range(1.2, 2.6)
		_:
			# A thick grass cap that rolls over the front edge, with tufts hanging over.
			var grass := _surface(t.grass, "grass", 3.5)
			var cap_h := 0.55
			_mesh(parent, _box(Vector3(w + 0.25, cap_h, depth + 0.1)), grass, Vector3(cx, top - cap_h / 2.0 + 0.04, front_z - depth / 2.0))
			var lip := _mesh(parent, _cylinder(cap_h * 0.6, cap_h * 0.6, w + 0.25), grass, Vector3(cx, top - cap_h * 0.55, front_z + 0.02), Vector3.ONE, Vector3(0, 0, 90))
			lip.name = "lip"
			_mesh(parent, _box(Vector3(w + 0.3, 0.12, 0.2)), _soft(t.grass_light), Vector3(cx, top - 0.06, front_z + 0.3))
			# Tufts hanging over at uneven gaps and sizes, now and then a small stone among them.
			var x := left + rng.randf_range(0.4, 0.9)
			var tuft_mesh := _sphere(0.32, 12)
			var stone := Materials.stone((t.soil as Color).lightened(0.25))
			while x < right - 0.4:
				var k := rng.randf_range(0.75, 1.3)
				_mesh(parent, tuft_mesh, grass, Vector3(x, top - cap_h - 0.12 * k, front_z + 0.2), Vector3(1.2, 0.8, 0.6) * k)
				if rng.randf() < 0.22:
					_mesh(parent, _sphere(0.16, 6), stone, Vector3(x + 0.5, top - cap_h - 0.3, front_z + 0.12), Vector3(1.3, 0.8, 0.7))
				x += rng.randf_range(0.8, 1.6)


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


## A pass-through platform in the theme: a rounded wooden plank (a light board on a darker beam), or a steel girder with rivets.
static func platform(parent: Node3D, t: Dictionary, left: float, right: float, top: float, front_z: float) -> void:
	var w := right - left
	var depth := 4.0
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
	var haze: Color = (t.sky_horizon as Color).lerp(Color.WHITE, 0.25)
	for layer in MIST.get(t.name, []):
		_mist(root, haze, layer[0], layer[1], layer[2], layer[3])
	painted_layers(root, t.name)
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
		# (Meshes built by hand, like the great tree's trunk, are left as they are: merged in with the built-in shapes they were lost.)
		if mi == null or mi.mesh == null or mi.mesh is ArrayMesh or mi.material_override == null or mi.transparency > 0.0 				or mi.get_child_count() > 0:
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
			_rock(root, Vector3(l + 3.2, y, -4.2), 0.8, rng)
			_rock(root, Vector3(l + 4.2, y, -3.8), 0.45, rng)
			_crate(root, Vector3(r - 4.6, y, -4.4), 0.9, 12.0)
			_barrel(root, Vector3(r - 3.4, y, -4.0), 0.75)
			_lantern_post(root, Vector3(l + 8.0, y, -4.8), Color(1.0, 0.75, 0.4))
			_rock(root, Vector3(r - 7.5, y, -3.6), 0.35, rng)
		"ocean":
			patches = [[l + 4.5, -4.6, 0.25, 0.2], [r - 4.4, -4.8, 0.25, 0.2], [l + 6.4, -3.6, 0.38, 0.3], [r - 6.5, -4.0, 0.9, 0.55]]
			_umbrella(root, Vector3(l + 4.5, y, -4.6), Color(0.98, 0.55, 0.55))
			_life_ring(root, Vector3(r - 4.4, y, -4.8))
			_bucket(root, Vector3(l + 6.4, y, -3.6), Color(0.45, 0.7, 1.0))
			_rock(root, Vector3(r - 6.8, y, -3.8), 0.4, rng)
			_rock(root, Vector3(r - 6.2, y, -4.2), 0.3, rng)
		"city":
			patches = [[l + 2.2, -4.5, 1.6, 0.75], [r - 4.0, -4.0, 0.5, 0.4]]
			_crate(root, Vector3(l + 1.6, y, -4.6), 1.0, 0.0)
			_crate(root, Vector3(l + 2.7, y, -4.4), 0.8, 8.0)
			_crate(root, Vector3(l + 2.0, y + 1.0, -4.6), 0.7, -10.0)
			_potted_plant(root, Vector3(r - 4.0, y, -4.0))
			_string_lights(root, Vector3(l + 3.0, y + 4.6, -6.0), Vector3(r - 2.5, y + 5.6, -6.5))
	for p in patches:
		_contact(root, Vector3(p[0], y, p[1]), Vector2(p[2], p[3]))


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
	var centre := at + Vector3(0, 0.55, 0)
	_mesh(root, _cylinder(0.55, 0.55, 1.3, 9), FighterView.toon(Color(0.98, 0.84, 0.5)), centre, Vector3.ONE, Vector3(90, turn, 0))
	var band := FighterView.toon(Color(0.8, 0.56, 0.32), false)
	for dz in [-0.32, 0.32]:
		_mesh(root, _cylinder(0.57, 0.57, 0.08, 9), band, centre + Vector3(0, 0, dz).rotated(Vector3.UP, deg_to_rad(turn)), Vector3.ONE, Vector3(90, turn, 0))


static func _barrel(root: Node3D, at: Vector3, size: float) -> void:
	_mesh(root, _cylinder(0.42 * size, 0.38 * size, 1.1 * size, 10), Materials.wood(Color(0.78, 0.52, 0.34), Vector2(2, 1)), at + Vector3(0, 0.55 * size, 0))
	var hoop := FighterView.toon(Color(0.42, 0.4, 0.5), false)
	for h in [0.22, 0.88]:
		_mesh(root, _cylinder(0.43 * size, 0.43 * size, 0.07 * size, 10), hoop, at + Vector3(0, h * size, 0))


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
	for diag in [45.0, -45.0]:
		_mesh(root, _box(Vector3(size * 1.15, b * 0.9, 0.035)), plank, face + basis * Vector3(0, 0, 0.005), Vector3.ONE, Vector3(0, turn, diag))


static func _signpost(root: Node3D, at: Vector3, points: float) -> void:
	_mesh(root, _box(Vector3(0.14, 1.7, 0.14)), Materials.wood(Color(0.74, 0.54, 0.34)), at + Vector3(0, 0.85, 0))
	var board := Materials.wood(Color(0.9, 0.72, 0.48))
	_mesh(root, _box(Vector3(1.1, 0.34, 0.08)), board, at + Vector3(points * 0.3, 1.4, 0.08), Vector3.ONE, Vector3(0, 0, -4))
	_mesh(root, _box(Vector3(0.9, 0.3, 0.08)), board, at + Vector3(-points * 0.25, 1.0, 0.08), Vector3.ONE, Vector3(0, 0, 5))


## A rock: a ball with few sides, squashed and turned, so it reads as a faceted stone.
static func _rock(root: Node3D, at: Vector3, size: float, rng: RandomNumberGenerator) -> void:
	var s := Vector3(rng.randf_range(1.1, 1.4), rng.randf_range(0.6, 0.8), 1.0) * size
	_mesh(root, _sphere(1.0, 6), Materials.stone(Color(0.74, 0.72, 0.8)), at + Vector3(0, s.y * 0.7, 0), s, Vector3(0, rng.randf_range(0, 60), rng.randf_range(-8, 8)))


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
	_mesh(root, _cylinder(0.0, 1.5, 0.5, 8), FighterView.toon(colour), top, Vector3.ONE, Vector3(0, 0, 8))
	# A towel on the sand under it.
	_mesh(root, _box(Vector3(1.6, 0.03, 0.8)), FighterView.toon(Color(0.55, 0.8, 0.95), false), at + Vector3(0.6, 0.02, 0.4))


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
	var total := 0
	for m in masses:
		total += int(m[2])
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
			cores.append_from(ball, 0, Transform3D(Basis.IDENTITY.scaled(Vector3.ONE * radius * 0.82), centre))
		for k in int(m[2]):
			# A direction out from the middle, mostly toward the front (the back of a crown is never seen), placed about its surface.
			var dir := Vector3(rng.randf_range(-1, 1), rng.randf_range(-0.8, 1), rng.randf_range(-0.2, 1)).normalized()
			var pos := centre + dir * radius * rng.randf_range(0.72, 1.02)
			var s := card * rng.randf_range(0.8, 1.2)
			var b := Basis.from_euler(Vector3(rng.randf_range(-0.5, 0.5), rng.randf_range(-0.6, 0.6), rng.randf_range(0, TAU))).scaled(Vector3.ONE * s)
			if rooted:
				# A tuft stands upright on the ground, facing the camera give or take.
				b = Basis.from_euler(Vector3(0, rng.randf_range(-0.5, 0.5), rng.randf_range(-0.12, 0.12))).scaled(Vector3.ONE * s)
				pos = centre + Vector3(0, s * 0.5, 0)
			mm.set_instance_transform(i, Transform3D(b, pos))
			var mid := b.inverse() * ((centre - Vector3(0, s, s * 0.5) if rooted else centre) - pos)
			mm.set_instance_custom_data(i, Color(mid.x, mid.y, mid.z, rng.randf()))
			var light := clampf(0.45 + dir.y * 0.3 + dir.z * 0.3 + rng.randf_range(-0.15, 0.15), 0.0, 0.999)
			mm.set_instance_color(i, greens[int(light * greens.size())] if not rooted else greens[rng.randi_range(0, greens.size() - 1)])
			i += 1
	if cores != null:
		var c := MeshInstance3D.new()
		c.mesh = cores.commit()
		c.material_override = core
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
		greens: Array) -> void:
	if VisualSettings.get_index("quality") == 0:
		return
	var spots := []
	# (Its own random numbers, so the stage's layout stays as it was.)
	var r := RandomNumberGenerator.new()
	r.seed = int(left * 31.0 + top * 7.0) + 5
	var x := left + 0.4
	while x < right - 0.4:
		spots.append([Vector3(x, top, r.randf_range(z_back, z_front)), 0.0, 1])
		x += r.randf_range(0.5, 1.1)
	_leaf_cards(root, spots, greens, 1.0, rng, "grass_tuft", true)


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
		"tone": Color(0.42, 0.52, 0.74), "mix": 0.45, "edge": 0.45}],
	"grove": [{"z": -165.0, "y": -34.0, "height": 52.0, "shape": 0, "seed": 8.0, "base": 0.5, "amplitude": 0.32, "frequency": 3.5,
		"tone": Color(0.62, 0.52, 0.78), "mix": 0.4, "edge": 0.5}],
	"sunset": [{"z": -175.0, "y": -36.0, "height": 58.0, "shape": 2, "seed": 5.0, "base": 0.4, "amplitude": 0.4, "frequency": 3.0,
		"tone": Color(0.42, 0.26, 0.5), "mix": 0.55, "edge": 0.4},
		{"z": -132.0, "y": -34.0, "height": 46.0, "shape": 2, "seed": 11.0, "base": 0.36, "amplitude": 0.36, "frequency": 2.6,
		"tone": Color(0.55, 0.26, 0.3), "mix": 0.62, "edge": 0.5}],
	"night": [{"z": -160.0, "y": -42.0, "height": 56.0, "shape": 1, "seed": 2.0, "base": 0.4, "amplitude": 0.45, "frequency": 4.0,
		"tone": Color(0.08, 0.1, 0.24), "mix": 0.55, "edge": 0.35}],
	"ocean": [{"z": -175.0, "y": -3.0, "height": 14.0, "shape": 0, "seed": 4.0, "base": -0.25, "amplitude": 0.75, "frequency": 6.0,
		"tone": Color(0.3, 0.5, 0.62), "mix": 0.45, "edge": 0.4}],
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
	m.set_shader_parameter("edge_colour", horizon.lerp(Color.WHITE, 0.5))
	m.set_shader_parameter("edge_amount", float(spec.edge))
	m.set_shader_parameter("shape", int(spec.shape))
	m.set_shader_parameter("seed", float(spec.seed))
	m.set_shader_parameter("base", float(spec.base))
	m.set_shader_parameter("amplitude", float(spec.amplitude))
	m.set_shader_parameter("frequency", float(spec.frequency))
	m.set_shader_parameter("haze_colour", horizon)
	m.set_shader_parameter("haze", float(spec.get("haze", 0.3)))
	var q := QuadMesh.new()
	q.size = Vector2(520.0, float(spec.height))
	var mi := _mesh(root, q, m, Vector3(0, float(spec.y) + float(spec.height) * 0.5, float(spec.z)))
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	mi.name = "Ridge"
	return mi


## The fog layers of each theme: [z, bottom y, height, strength].
const MIST := {
	"meadow": [[-56.0, -24.0, 17.0, 0.55], [-86.0, -26.0, 20.0, 0.6]],
	"grove": [[-60.0, -24.0, 18.0, 0.5], [-90.0, -26.0, 22.0, 0.6]],
	"sunset": [[-60.0, -26.0, 20.0, 0.45], [-95.0, -24.0, 26.0, 0.55]],
	"ocean": [[-130.0, -10.0, 13.0, 0.55]],
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


static func _clouds(root: Node3D, rng: RandomNumberGenerator, count: int, color: Color, y_range: Vector2) -> void:
	var mat := _unshaded(color)
	for k in count:
		var cloud := Drift.new()
		cloud.speed = rng.randf_range(0.6, 1.6)
		cloud.position = Vector3(rng.randf_range(-90, 90), rng.randf_range(y_range.x, y_range.y), rng.randf_range(-120, -85))
		root.add_child(cloud)
		var puffs := rng.randi_range(3, 5)
		for b in puffs:
			var s := rng.randf_range(2.4, 4.4)
			_mesh(cloud, _sphere(1.0, 16), mat, Vector3(b * 3.0 - puffs * 1.5, rng.randf_range(-0.6, 1.4), 0), Vector3(s, s * 0.8, s))


static func _birds(root: Node3D, rng: RandomNumberGenerator, colour: Color) -> void:
	for k in 2:
		var flock := Flock.new()
		flock.position = Vector3(rng.randf_range(-100, 60), rng.randf_range(16, 30), rng.randf_range(-75, -60))
		flock.speed = rng.randf_range(3.0, 5.5)
		root.add_child(flock)
		flock.build(rng.randi_range(3, 5), colour, rng)


static func _hills(root: Node3D, rng: RandomNumberGenerator, near: Color, far: Color) -> void:
	for k in 8:
		var is_far := k % 2 == 1
		var w := rng.randf_range(26.0, 44.0)
		_mesh(root, _sphere(1.0, 24), _soft(far if is_far else near),
			Vector3(-105.0 + k * 30.0 + rng.randf_range(-8, 8), -22.0, -100.0 if is_far else -72.0),
			Vector3(w, rng.randf_range(10.0, 18.0), 6.0))


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


## The underside of a floating island below its lowest block `e` = [left, right, top, bottom]: a tapering mass of earth
## (its face in the theme's soil, the stone tile drawn on), a few rocks jutting from it and roots hanging from its rim.
static func _island_underside(root: Node3D, t: Dictionary, e: Array) -> void:
	var l: float = e[0]
	var r: float = e[1]
	var bottom: float = e[3]
	var w := r - l
	var cx := (l + r) / 2.0
	var earth := Materials.stone((t.soil_dark as Color).darkened(0.1), Vector2(4, 2))
	var rock := Materials.stone((t.soil as Color).lerp(Color(0.74, 0.72, 0.8), 0.5))
	# The mass: a cone hanging under the block, no wider than it (so nothing sticks out past it), squashed front to back to the block's
	# depth, and a smaller tip below it, off-centre.
	_mesh(root, _cylinder(w * 0.48, w * 0.1, 6.0, 10), earth, Vector3(cx, bottom - 3.0, -3.7), Vector3(1.0, 1.0, 7.4 / w))
	_mesh(root, _cylinder(w * 0.14, 0.0, 3.5, 8), earth, Vector3(cx + w * 0.06, bottom - 7.6, -3.7), Vector3(1.0, 1.0, 0.6))
	var rng := RandomNumberGenerator.new()
	rng.seed = 13
	for k in 5:
		var x := cx + rng.randf_range(-w * 0.35, w * 0.35)
		var depth := absf(x - cx) / (w * 0.5)
		_mesh(root, _sphere(1.0, 6), rock, Vector3(x * 0.85 + cx * 0.15, bottom - 1.2 - (1.0 - depth) * rng.randf_range(1.0, 3.5), -0.6),
			Vector3(1.4, 1.0, 1.1) * rng.randf_range(0.6, 1.1), Vector3(0, rng.randf_range(0, 60), rng.randf_range(-15, 15)))
	# Roots hanging from the rim, thin and few.
	var root_mat := Materials.bark(Color(0.55, 0.36, 0.24), Vector2(1, 2))
	for k in 6:
		var x := l + 1.0 + rng.randf_range(0.0, w - 2.0)
		var len := rng.randf_range(1.5, 3.5)
		_mesh(root, _cylinder(0.07, 0.14, len, 5), root_mat, Vector3(x, bottom - len * 0.5, 0.1), Vector3.ONE,
			Vector3(0, 0, rng.randf_range(-12, 12)))


## Stage extent helpers: [left, right, top, bottom] of the solid ground, with defaults.
static func _ext(stage: Array) -> Array:
	return stage.slice(0, 4) if stage.size() >= 4 else [-11.0, 11.0, 0.0, -8.0]


static func _meadow(root: Node3D, t: Dictionary, stage: Array) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 42
	var e := _ext(stage)
	_hills(root, rng, Color(0.5, 0.78, 0.5), Color(0.62, 0.8, 0.86))
	_clouds(root, rng, 9, Color(1, 1, 1), Vector2(14, 34))
	_birds(root, rng, Color(0.25, 0.22, 0.3))
	# A windmill on a far hill, its sails turning slowly.
	_windmill(root, Vector3(-34, -6, -70))
	# Round trees dotted over the nearer hills.
	for k in 6:
		_round_tree(root, Vector3(rng.randf_range(-70, 70), rng.randf_range(-12, -6), rng.randf_range(-62, -48)), rng.randf_range(0.7, 1.2), rng)
	# A wooden fence along the back of the stage, and flowers.
	var post := Materials.wood(Color(0.86, 0.66, 0.42))
	var x: float = e[0] + 0.6
	while x <= e[1] - 0.5:
		_mesh(root, _box(Vector3(0.18, 1.0, 0.18)), post, Vector3(x, e[2] + 0.5, -6.2))
		x += 1.6
	for h in [0.35, 0.75]:
		_mesh(root, _box(Vector3(e[1] - e[0] - 1.0, 0.12, 0.08)), post, Vector3((e[0] + e[1]) / 2.0, e[2] + h, -6.1))
	_flowers(root, e, rng, [Color(1, 0.55, 0.6), Color(1, 1, 1), Color(1, 0.85, 0.3)])
	_tufts(root, float(e[0]), float(e[1]), float(e[2]), -2.4, -5.8, rng, Materials.greens(Color(0.42, 0.74, 0.38)))


## A windmill: a tapered white tower with a cone roof, and four sails that turn.
static func _windmill(root: Node3D, at: Vector3) -> void:
	_mesh(root, _cylinder(1.8, 3.0, 12.0, 12), FighterView.toon(Color(0.96, 0.94, 0.9)), at + Vector3(0, 6, 0))
	_mesh(root, _cylinder(0.0, 2.6, 3.0, 12), FighterView.toon(Color(0.78, 0.36, 0.3)), at + Vector3(0, 13.5, 0))
	var hub := Spinner.new()
	hub.speed = 0.6
	hub.position = at + Vector3(0, 11.5, 2.2)
	root.add_child(hub)
	var sail := FighterView.toon(Color(0.92, 0.88, 0.8))
	for k in 4:
		var arm := Node3D.new()
		arm.rotation.z = TAU * k / 4.0
		hub.add_child(arm)
		_mesh(arm, _box(Vector3(0.3, 7.0, 0.2)), Materials.wood(Color(0.55, 0.38, 0.26)), Vector3(0, 3.5, 0))
		_mesh(arm, _box(Vector3(1.6, 5.4, 0.1)), sail, Vector3(0.9, 4.2, 0.1))


## A small round tree: a short trunk under one big ball and a couple of small ones.
static func _round_tree(root: Node3D, at: Vector3, size: float, rng: RandomNumberGenerator) -> void:
	_mesh(root, _cylinder(0.35 * size, 0.5 * size, 3.0 * size, 8), Materials.bark(Color(0.55, 0.38, 0.24), Vector2(2, 1)), at + Vector3(0, 1.5 * size, 0))
	var greens := Materials.greens(Materials.greens(Color(0.32, 0.64, 0.34))[rng.randi_range(0, 2)])
	var crown := Sway.new()
	crown.position = at + Vector3(0, 2.8 * size, 0)
	crown.amount = 2.5
	crown.phase = rng.randf() * TAU
	root.add_child(crown)
	# Two masses of leaf cards (a big one and a smaller one to its side), shading as rounded volumes (`_leaf_cards`).
	_leaf_cards(crown, [[Vector3(0, 1.4, 0) * size, 2.4 * size, 64], [Vector3(1.4, 0.7, 0.5) * size, 1.4 * size, 26]], greens,
		1.6 * size, rng, "leaf_cluster", false, Materials.foliage(greens[0], Vector2(3, 2), false))


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
static func _sunset(root: Node3D, t: Dictionary, stage: Array) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 7
	var e := _ext(stage)
	_mesh(root, _sphere(14.0, 32), _unshaded(Color(1.0, 0.86, 0.55)), Vector3(18, 2, -140))
	_mesh(root, _sphere(18.0, 32), _unshaded(Color(1.0, 0.75, 0.5, 1.0)).duplicate(), Vector3(18, 2, -142)).transparency = 0.7
	for k in 8:
		var w := rng.randf_range(10.0, 24.0)
		var h := rng.randf_range(12.0, 24.0)
		var far := k % 2 == 1
		var base := Color(0.55, 0.3, 0.36) if far else Color(0.72, 0.36, 0.3)
		var mesa := Node3D.new()
		mesa.position = Vector3(-110.0 + k * 30.0 + rng.randf_range(-6, 6), -16.0, -110.0 if far else -78.0)
		root.add_child(mesa)
		# Each mesa is a few stacked slabs in alternating shades (strata).
		var y := 0.0
		var r := w * 0.55
		for band in 4:
			var bh := h / 4.0
			_mesh(mesa, _cylinder(r * 0.94, r, bh, 10), _soft(base.lightened(0.08 * (band % 2))), Vector3(0, y + bh / 2.0, 0))
			y += bh
			r *= 0.93
	# A rock arch on the left.
	var arch := _soft(Color(0.78, 0.42, 0.3))
	_mesh(root, _cylinder(1.6, 2.2, 14.0, 10), arch, Vector3(-40, -6, -48))
	_mesh(root, _cylinder(1.6, 2.2, 14.0, 10), arch, Vector3(-28, -6, -48))
	_mesh(root, _box(Vector3(16.0, 3.0, 4.0)), arch, Vector3(-34, 2.5, -48))
	# Cacti behind the stage.
	for x in [e[0] + 1.5, e[1] - 2.0]:
		_cactus(root, Vector3(x, e[2], -6.5), rng.randf_range(0.65, 0.85))
		_contact(root, Vector3(x, e[2], -6.5), Vector2(0.45, 0.38))
	_clouds(root, rng, 6, Color(1.0, 0.78, 0.7), Vector2(18, 30))


static func _cactus(root: Node3D, at: Vector3, size: float) -> void:
	var green := FighterView.toon(Color(0.36, 0.62, 0.38))
	_mesh(root, _cylinder(0.45 * size, 0.5 * size, 3.6 * size, 12), green, at + Vector3(0, 1.8 * size, 0))
	_mesh(root, _sphere(0.45 * size, 12), green, at + Vector3(0, 3.6 * size, 0))
	for side in [-1.0, 1.0]:
		var h := (1.6 if side < 0 else 2.2) * size
		_mesh(root, _cylinder(0.28 * size, 0.28 * size, 1.0 * size, 10), green, at + Vector3(side * 0.7 * size, h, 0), Vector3.ONE, Vector3(0, 0, 90))
		_mesh(root, _cylinder(0.28 * size, 0.28 * size, 1.4 * size, 10), green, at + Vector3(side * 1.15 * size, h + 0.6 * size, 0))
		_mesh(root, _sphere(0.28 * size, 10), green, at + Vector3(side * 1.15 * size, h + 1.3 * size, 0))


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
	_hills(root, rng, Color(0.62, 0.84, 0.66), Color(0.76, 0.84, 0.95))
	_clouds(root, rng, 8, Color(1, 0.97, 1), Vector2(10, 36))
	_birds(root, rng, Color(0.25, 0.22, 0.3))
	var top: float = stage[2] if stage.size() == 4 else 0.0
	var left: float = stage[0] if stage.size() == 4 else -11.0
	var right: float = stage[1] if stage.size() == 4 else 11.0
	# A far line of soft tree shapes on the hills, between them and the trees, for one more layer of depth (the haze fades it).
	_tree_line(root, rng, top - 16.0, -78.0, Color(0.5, 0.72, 0.56))
	# The great tree, set back behind the island's middle.
	_tree(root, Vector3(0, top - 2.0, -19.0), 1.0, rng)
	# Two smaller trees further back, either side.
	_tree(root, Vector3(-30, top - 12.0, -55.0), 0.6, rng)
	_tree(root, Vector3(32, top - 13.0, -60.0), 0.55, rng)
	# Round bushes along the back of the island.
	var bush_greens := Materials.greens(Color(0.3, 0.66, 0.34))
	var bush := Materials.foliage(bush_greens[1])
	var bush_light := Materials.foliage(bush_greens[3], Vector2(2, 1), false)
	for x in [left + 1.4, left + 3.2, right - 3.2, right - 1.4]:
		var s := rng.randf_range(1.1, 1.5)
		_mesh(root, _sphere(1.0, 16), bush, Vector3(x, top + 0.6, -6.5), Vector3(s * 1.3, s, s))
		_mesh(root, _sphere(0.55, 12), bush_light, Vector3(x - 0.3, top + 1.2, -5.6))
		_contact(root, Vector3(x, top, -6.5), Vector2(s * 1.15, s * 0.8))
	# Little flowers and tufts on the grass behind the fighters, gathered toward the ends so the middle of the stage stays clean.
	_flower_clusters(root, [left, right, top], rng, [Color(1, 1, 1), Color(1, 0.8, 0.9)], Color(0.46, 0.84, 0.42))
	_tufts(root, left, right, top, -2.4, -5.8, rng, Materials.greens(Color(0.46, 0.8, 0.42)))
	# Under the island: tapered earth and rock with roots hanging from it, so it reads as a floating island.
	if stage.size() >= 6:
		_island_underside(root, t, [stage[4], stage[5], top, stage[3]])
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
	_clouds(root, rng, 8, Color(1, 1, 1), Vector2(16, 34))
	_birds(root, rng, Color(0.97, 0.97, 1.0))
	# Distant islands.
	for k in 3:
		var isle := Vector3(-70.0 + k * 65.0 + rng.randf_range(-10, 10), float(e[3]) - 3.0, -110.0 + rng.randf_range(-10, 10))
		_mesh(root, _sphere(1.0, 16), _soft(Color(0.98, 0.88, 0.62)), isle, Vector3(rng.randf_range(10, 16), 3.5, 6.0))
		_mesh(root, _sphere(1.0, 16), _soft(Color(0.3, 0.62, 0.42)), isle + Vector3(0, 2.0, 0), Vector3(rng.randf_range(6, 9), 3.5, 4.0))
		_palm(root, isle + Vector3(rng.randf_range(-4, 4), 4.0, 2.0), 1.4, rng)
	# A lighthouse on the right.
	var lh := Vector3(52, float(e[3]) - 2.0, -80)
	_mesh(root, _cylinder(2.0, 3.0, 20.0, 16), FighterView.toon(Color(0.96, 0.95, 0.92)), lh + Vector3(0, 10, 0))
	for k in 3:
		_mesh(root, _cylinder(2.6 - k * 0.3, 2.8 - k * 0.3, 2.2, 16), FighterView.toon(Color(0.86, 0.3, 0.28)), lh + Vector3(0, 3.0 + k * 6.0, 0))
	_mesh(root, _sphere(1.6, 16), _unshaded(Color(1.0, 0.95, 0.7)), lh + Vector3(0, 21.5, 0))
	_mesh(root, _cylinder(0.0, 2.2, 2.0, 16), FighterView.toon(Color(0.3, 0.32, 0.4)), lh + Vector3(0, 23.5, 0))
	# A sailboat.
	var boat := Bob.new()
	boat.position = Vector3(-30, float(e[3]) - 0.6, -60)
	root.add_child(boat)
	_mesh(boat, _box(Vector3(6.0, 1.2, 2.0)), FighterView.toon(Color(0.86, 0.36, 0.3)), Vector3.ZERO)
	_mesh(boat, _cylinder(0.0, 2.6, 6.0, 3), FighterView.toon(Color(1, 1, 1)), Vector3(0.6, 3.6, 0), Vector3(1, 1, 0.15))
	# Palms at the ends of the island, behind the fighters.
	_palm(root, Vector3(float(e[0]) + 1.8, float(e[2]), -5.5), 1.0, rng)
	_palm(root, Vector3(float(e[1]) - 1.6, float(e[2]), -6.0), 1.15, rng)
	_contact(root, Vector3(float(e[0]) + 1.8, float(e[2]), -5.5), Vector2(0.4, 0.35))
	_contact(root, Vector3(float(e[1]) - 1.6, float(e[2]), -6.0), Vector2(0.45, 0.38))


## A palm: a curving trunk of stacked rings under a fan of long leaves and a few coconuts.
static func _palm(root: Node3D, base: Vector3, size: float, rng: RandomNumberGenerator) -> void:
	var bark := Materials.bark(Color(0.66, 0.5, 0.32), Vector2(1, 1))
	var lean := rng.randf_range(-0.12, 0.12)
	var top := base
	for k in 8:
		var seg := base + Vector3(lean * k * k * 0.12 * size, k * 0.75 * size, 0)
		_mesh(root, _cylinder(0.24 * size, 0.3 * size, 0.8 * size, 10), bark, seg + Vector3(0, 0.4 * size, 0))
		top = seg + Vector3(0, 0.8 * size, 0)
	var leaf := Materials.foliage(Color(0.24, 0.6, 0.3), Vector2(2, 1))
	var head := Sway.new()
	head.position = top
	head.amount = 5.0
	head.speed = 1.3
	head.phase = rng.randf() * TAU
	root.add_child(head)
	for k in 7:
		var a := TAU * k / 7.0
		var frond := Node3D.new()
		frond.rotation = Vector3(0, a, 0)
		head.add_child(frond)
		_mesh(frond, _sphere(1.0, 10), leaf, Vector3(1.6 * size, -0.3 * size, 0), Vector3(1.9 * size, 0.18 * size, 0.55 * size), Vector3(0, 0, -18))
	for k in 3:
		_mesh(head, _sphere(0.22 * size, 8), FighterView.toon(Color(0.45, 0.32, 0.2)), Vector3(cos(k * 2.1) * 0.35, -0.3, sin(k * 2.1) * 0.35) * size)


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
		var z := -40.0 - layer * 28.0
		var shade := Color(0.2, 0.17, 0.33).lerp(Color(0.29, 0.22, 0.42), layer / 2.0)
		_skyline_layer(root, rng, layer, z, shade, float(e[3]) - 6.0)
	# A water tower and an antenna on the roof behind the stage.
	var roof: float = e[2]
	var tower := Vector3(float(e[0]) + 3.0, roof, -6.0)
	var wood := Materials.wood(Color(0.55, 0.36, 0.26))
	for leg in [Vector3(-0.9, 0, -0.9), Vector3(0.9, 0, -0.9), Vector3(-0.9, 0, 0.9), Vector3(0.9, 0, 0.9)]:
		_mesh(root, _box(Vector3(0.18, 2.4, 0.18)), wood, tower + leg + Vector3(0, 1.2, 0))
	_mesh(root, _cylinder(1.3, 1.3, 2.2, 14), wood, tower + Vector3(0, 3.5, 0))
	_mesh(root, _cylinder(0.0, 1.5, 1.0, 14), FighterView.toon(Color(0.4, 0.28, 0.22)), tower + Vector3(0, 5.1, 0))
	# Iron hoops round the staves, a hatch on the roof and a ladder up the side: they say what it is made of and how big it is.
	var iron := FighterView.toon(Color(0.3, 0.26, 0.32))
	for hy in [2.6, 3.5, 4.4]:
		_mesh(root, _cylinder(1.34, 1.34, 0.08, 14), iron, tower + Vector3(0, hy, 0))
	_mesh(root, _box(Vector3(0.5, 0.12, 0.5)), iron, tower + Vector3(0.3, 5.45, 0.55), Vector3.ONE, Vector3(-30, 0, 0))
	for lx in [-0.18, 0.18]:
		_mesh(root, _box(Vector3(0.05, 3.4, 0.05)), iron, tower + Vector3(1.0 + lx, 2.2, 0.95))
	for ly in range(5):
		_mesh(root, _box(Vector3(0.4, 0.04, 0.04)), iron, tower + Vector3(1.0, 0.8 + ly * 0.7, 0.95))
	var mast := Vector3(float(e[1]) - 2.5, roof, -6.5)
	_mesh(root, _cylinder(0.08, 0.15, 6.0, 6), FighterView.toon(Color(0.6, 0.62, 0.7)), mast + Vector3(0, 3, 0))
	var blink := Blinker.new()
	blink.position = mast + Vector3(0, 6.1, 0)
	root.add_child(blink)
	blink.build(Color(1.0, 0.25, 0.25))
	# Vents and an air conditioner.
	var metal := FighterView.toon(Color(0.66, 0.68, 0.76))
	var unit := Vector3(float(e[0]) + 6.5, roof, -4.5)
	_mesh(root, _box(Vector3(1.6, 1.0, 1.2)), metal, unit + Vector3(0, 0.5, 0))
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
	for spot in spots:
		var d := Decal.new()
		d.texture_albedo = marks[int(spot[2])]
		var k := rng.randf_range(2.2, 3.4)
		d.size = Vector3(k, 0.3, k * 0.8)
		d.position = Vector3(lerpf(left, right, float(spot[0])), roof + 0.05, float(spot[1]))
		d.rotation.y = rng.randf_range(-0.5, 0.5)
		d.upper_fade = 0.3
		d.lower_fade = 0.3
		root.add_child(d)


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
const FACADE_LIT := [3.0, 2.3, 1.7]


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
	var bark := Materials.bark(Color(0.62, 0.42, 0.26), Vector2(5, 3))
	_mesh(root, trunk_mesh(trunk_h, 2.9 * size, 2.0 * size, 0.7 * size, 1.4 * size), bark, base)
	for k in 6:
		var a := -1.25 + k * 0.5
		var reach := rng.randf_range(0.85, 1.15)
		_mesh(root, _cylinder(0.25 * size, 1.15 * size * reach, 3.4 * size * reach, 10), bark,
			base + Vector3(sin(a) * 3.2 * size, 0.6 * size, cos(a) * 2.3 * size), Vector3.ONE, Vector3(0, 0, -rad_to_deg(a) * 0.6))
	# Four coordinated greens: deep shade, body, light, and sunlit tips.
	var g := Materials.greens(Color(0.27, 0.62, 0.34))
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
	_leaf_cards(sway, masses, g, 3.6 * size, rng, "leaf_cluster", false, leaf)
	var _unused := [leaf_mid, leaf_light, leaf_sun]


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
