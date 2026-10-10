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
const FighterView := preload("res://scripts/fighter_view.gd")

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
		"grass_light": Color(0.78, 0.8, 0.88), "wood": Color(0.86, 0.36, 0.3), "wood_dark": Color(0.5, 0.2, 0.2),
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

static func _unshaded(c: Color) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	m.albedo_color = c
	return m


## A stage surface in colour `c` with one of the painted tiles in godot/art/stage/ laid over it in world space (`size` world units a repeat).
static func _surface(c: Color, tile: String, size: float, outline := true) -> ShaderMaterial:
	var tex := SvgArt.texture("res://art/stage/%s.svg" % tile, {"#ff00ff": c})
	return FighterView.toon(Color.WHITE, outline, tex, Vector2.ONE, size)


static func _soft(c: Color) -> ShaderMaterial:
	return FighterView.toon(c, false)


## The pattern on a block's front, drawn once into a small texture: quilted diamonds, sandstone strata, rounded cobbles, or a building
## facade with windows (some lit).
static func _soil_material(t: Dictionary) -> ShaderMaterial:
	var size := 64
	var img := Image.create_empty(size, size, false, Image.FORMAT_RGB8)
	var a: Color = t.soil
	var b: Color = t.soil_dark
	var pattern: String = t.get("pattern", "diamonds")
	var rng := RandomNumberGenerator.new()
	rng.seed = 9
	img.fill(a)
	match pattern:
		"strata":
			# Wavy horizontal bands in three shades.
			var shades := [a, b, a.lerp(Color(1, 0.85, 0.6), 0.25), b.darkened(0.12)]
			for y in size:
				for x in size:
					var wave := sin(float(x) / size * TAU) * 2.5
					var band := int(floor((float(y) + wave) / 8.0)) % shades.size()
					img.set_pixel(x, y, shades[(band + shades.size()) % shades.size()])
		"cobble":
			# Rounded stones with dark gaps.
			img.fill(b.darkened(0.25))
			for k in 14:
				var cx := rng.randi_range(0, size - 1)
				var cy := rng.randi_range(0, size - 1)
				var r := rng.randi_range(9, 14)
				var shade := a.lerp(b, rng.randf_range(0.0, 0.5))
				for y in range(cy - r, cy + r):
					for x in range(cx - r, cx + r):
						var dx := x - cx
						var dy := y - cy
						if dx * dx + dy * dy < (r - 1) * (r - 1):
							var lit := shade.lightened(0.08) if dy < -r / 3 else shade
							img.set_pixel(posmod(x, size), posmod(y, size), lit)
		"facade":
			# Concrete panels with a grid of windows, some lit warm yellow.
			for y in size:
				for x in size:
					var seam := x % 32 == 0 or y % 32 == 0
					img.set_pixel(x, y, b if seam else a)
			for wy in [6, 38]:
				for wx in [5, 19, 37, 51]:
					var lit := rng.randf() < 0.55
					var col := Color(1.0, 0.86, 0.45) if lit else Color(0.16, 0.18, 0.26)
					for y in range(wy, wy + 14):
						for x in range(wx, wx + 9):
							img.set_pixel(x, y, col)
		_:
			for y in size:
				for x in size:
					var u := absf(float(x % 32) - 16.0)
					var v := absf(float(y % 32) - 16.0)
					var inside := u + v < 14.0
					var edge := absf(u + v - 14.0) < 1.6
					var c := a if inside else b
					if edge:
						c = b.darkened(0.25)
					img.set_pixel(x, y, c)
	img.generate_mipmaps()
	var tex := ImageTexture.create_from_image(img)
	# On the stage's cel shading, laid in world space (one repeat every 2.2 units, a facade's every 3.3).
	return FighterView.toon(Color.WHITE, true, tex, Vector2.ONE, 2.2 if pattern != "facade" else 3.3)


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
	if not exposed:
		return
	var cx := (left + right) / 2.0
	var w := right - left
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
			var rng := RandomNumberGenerator.new()
			rng.seed = int(left * 13.0 + top * 7.0)
			var pebble := FighterView.toon((t.soil as Color).lightened(0.2))
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
			var x := left + 0.6
			var tuft_mesh := _sphere(0.32, 12)
			while x < right - 0.4:
				_mesh(parent, tuft_mesh, grass, Vector3(x, top - cap_h - 0.12, front_z + 0.2), Vector3(1.2, 0.8, 0.6))
				x += 1.15


## A pass-through platform in the theme: a rounded wooden plank (a light board on a darker beam), or a steel girder with rivets.
static func platform(parent: Node3D, t: Dictionary, left: float, right: float, top: float, front_z: float) -> void:
	var w := right - left
	var depth := 4.0
	var cx := (left + right) / 2.0
	if t.get("platform", "wood") == "girder":
		var steel := _surface(t.wood, "steel", 3.0)
		_mesh(parent, _box(Vector3(w, 0.18, depth)), steel, Vector3(cx, top - 0.09, front_z - depth / 2.0))
		_mesh(parent, _box(Vector3(w, 0.5, 0.16)), FighterView.toon(t.wood_dark), Vector3(cx, top - 0.4, front_z - 0.3))
		_mesh(parent, _box(Vector3(w, 0.14, depth * 0.7)), steel, Vector3(cx, top - 0.68, front_z - depth / 2.0))
		var rivet := _unshaded(Color(0.95, 0.82, 0.6))
		var x := left + 0.35
		while x < right - 0.2:
			_mesh(parent, _sphere(0.06, 6), rivet, Vector3(x, top - 0.4, front_z - 0.2))
			x += 0.7
		return
	var board := _surface(t.wood, "wood", 2.4)
	_mesh(parent, _box(Vector3(w, 0.26, depth)), board, Vector3(cx, top - 0.13, front_z - depth / 2.0))
	_mesh(parent, _box(Vector3(w - 0.3, 0.22, depth - 0.4)), FighterView.toon(t.wood_dark), Vector3(cx, top - 0.37, front_z - depth / 2.0))
	for side in [-1.0, 1.0]:
		_mesh(parent, _cylinder(0.13, 0.13, depth), board, Vector3(cx + side * w / 2.0, top - 0.13, front_z - depth / 2.0), Vector3.ONE, Vector3(90, 0, 0))
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
	# Fog layers: soft bands of haze in the horizon's colour between the layers of scenery, thickest at the bottom, so each layer sits
	# further back than the one before (a painted depth, at the cost of a few see-through cards).
	var haze: Color = (t.sky_horizon as Color).lerp(Color.WHITE, 0.25)
	for layer in MIST.get(t.name, []):
		_mist(root, haze, layer[0], layer[1], layer[2], layer[3])


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


## Stage extent helpers: [left, right, top, bottom] of the solid ground, with defaults.
static func _ext(stage: Array) -> Array:
	return stage if stage.size() == 4 else [-11.0, 11.0, 0.0, -8.0]


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
	var post := FighterView.toon(Color(0.86, 0.66, 0.42))
	var x: float = e[0] + 0.6
	while x <= e[1] - 0.5:
		_mesh(root, _box(Vector3(0.18, 1.0, 0.18)), post, Vector3(x, e[2] + 0.5, -6.2))
		x += 1.6
	for h in [0.35, 0.75]:
		_mesh(root, _box(Vector3(e[1] - e[0] - 1.0, 0.12, 0.08)), post, Vector3((e[0] + e[1]) / 2.0, e[2] + h, -6.1))
	_flowers(root, e, rng, [Color(1, 0.55, 0.6), Color(1, 1, 1), Color(1, 0.85, 0.3)])


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
		_mesh(arm, _box(Vector3(0.3, 7.0, 0.2)), FighterView.toon(Color(0.55, 0.38, 0.26)), Vector3(0, 3.5, 0))
		_mesh(arm, _box(Vector3(1.6, 5.4, 0.1)), sail, Vector3(0.9, 4.2, 0.1))


## A small round tree: a short trunk under one big ball and a couple of small ones.
static func _round_tree(root: Node3D, at: Vector3, size: float, rng: RandomNumberGenerator) -> void:
	_mesh(root, _cylinder(0.35 * size, 0.5 * size, 3.0 * size, 8), FighterView.toon(Color(0.55, 0.38, 0.24)), at + Vector3(0, 1.5 * size, 0))
	var leaf := FighterView.toon(Color(0.3, 0.62, 0.34).lerp(Color(0.45, 0.72, 0.3), rng.randf()))
	var crown := Sway.new()
	crown.position = at + Vector3(0, 2.8 * size, 0)
	crown.amount = 2.5
	crown.phase = rng.randf() * TAU
	root.add_child(crown)
	_mesh(crown, _sphere(1.0, 16), leaf, Vector3(0, 1.4 * size, 0), Vector3.ONE * 2.4 * size)
	_mesh(crown, _sphere(1.0, 12), leaf, Vector3(1.4, 0.7, 0.5) * size, Vector3.ONE * 1.4 * size)


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
	# The great tree, set back behind the island's middle.
	_tree(root, Vector3(0, top - 2.0, -19.0), 1.0, rng)
	# Two smaller trees further back, either side.
	_tree(root, Vector3(-30, top - 12.0, -55.0), 0.6, rng)
	_tree(root, Vector3(32, top - 13.0, -60.0), 0.55, rng)
	# Round bushes along the back of the island.
	var bush := FighterView.toon(Color(0.3, 0.66, 0.34))
	var bush_light := _soft(Color(0.44, 0.78, 0.4))
	for x in [left + 1.4, left + 3.2, right - 3.2, right - 1.4]:
		var s := rng.randf_range(1.1, 1.5)
		_mesh(root, _sphere(1.0, 16), bush, Vector3(x, top + 0.6, -6.5), Vector3(s * 1.3, s, s))
		_mesh(root, _sphere(0.55, 12), bush_light, Vector3(x - 0.3, top + 1.2, -5.6))
	# Little flowers on the grass behind the fighters.
	var petal := _unshaded(Color(1, 1, 1))
	var heart := _unshaded(Color(1, 0.85, 0.3))
	var x := left + 0.8
	while x < right - 0.8:
		var z := rng.randf_range(-4.5, -2.0)
		_mesh(root, _sphere(0.13, 8), petal, Vector3(x, top + 0.08, z))
		_mesh(root, _sphere(0.06, 6), heart, Vector3(x, top + 0.12, z + 0.08))
		x += rng.randf_range(0.9, 1.8)
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


## A palm: a curving trunk of stacked rings under a fan of long leaves and a few coconuts.
static func _palm(root: Node3D, base: Vector3, size: float, rng: RandomNumberGenerator) -> void:
	var bark := FighterView.toon(Color(0.66, 0.5, 0.32))
	var lean := rng.randf_range(-0.12, 0.12)
	var top := base
	for k in 8:
		var seg := base + Vector3(lean * k * k * 0.12 * size, k * 0.75 * size, 0)
		_mesh(root, _cylinder(0.24 * size, 0.3 * size, 0.8 * size, 10), bark, seg + Vector3(0, 0.4 * size, 0))
		top = seg + Vector3(0, 0.8 * size, 0)
	var leaf := FighterView.toon(Color(0.24, 0.6, 0.3))
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
	# Three layers of buildings, darker and smaller the further back, with lit windows.
	for layer in 3:
		var z := -40.0 - layer * 28.0
		var shade := Color(0.16, 0.14, 0.3).lerp(Color(0.28, 0.2, 0.42), layer / 2.0)
		var x := -120.0
		while x < 120.0:
			var w := rng.randf_range(6.0, 13.0)
			var h := rng.randf_range(10.0, 34.0) * (1.0 - layer * 0.15)
			_building(root, Vector3(x + w / 2.0, float(e[3]) - 6.0, z), Vector3(w, h, 5.0), shade, rng, layer)
			x += w + rng.randf_range(0.5, 3.0)
	# A water tower and an antenna on the roof behind the stage.
	var roof: float = e[2]
	var tower := Vector3(float(e[0]) + 3.0, roof, -6.0)
	var wood := FighterView.toon(Color(0.55, 0.36, 0.26))
	for leg in [Vector3(-0.9, 0, -0.9), Vector3(0.9, 0, -0.9), Vector3(-0.9, 0, 0.9), Vector3(0.9, 0, 0.9)]:
		_mesh(root, _box(Vector3(0.18, 2.4, 0.18)), wood, tower + leg + Vector3(0, 1.2, 0))
	_mesh(root, _cylinder(1.3, 1.3, 2.2, 14), wood, tower + Vector3(0, 3.5, 0))
	_mesh(root, _cylinder(0.0, 1.5, 1.0, 14), FighterView.toon(Color(0.4, 0.28, 0.22)), tower + Vector3(0, 5.1, 0))
	var mast := Vector3(float(e[1]) - 2.5, roof, -6.5)
	_mesh(root, _cylinder(0.08, 0.15, 6.0, 6), FighterView.toon(Color(0.6, 0.62, 0.7)), mast + Vector3(0, 3, 0))
	var blink := Blinker.new()
	blink.position = mast + Vector3(0, 6.1, 0)
	root.add_child(blink)
	blink.build(Color(1.0, 0.25, 0.25))
	# Vents and an air conditioner.
	var metal := FighterView.toon(Color(0.66, 0.68, 0.76))
	_mesh(root, _box(Vector3(1.6, 1.0, 1.2)), metal, Vector3(float(e[0]) + 6.5, roof + 0.5, -4.5))
	_mesh(root, _cylinder(0.3, 0.3, 0.9, 10), metal, Vector3(float(e[1]) - 6.0, roof + 0.45, -4.0))


static func _building(root: Node3D, base: Vector3, size: Vector3, shade: Color, rng: RandomNumberGenerator, layer: int) -> void:
	_mesh(root, _box(size), _unshaded(shade), base + Vector3(0, size.y / 2.0, 0))
	# Windows: a grid of small squares, some lit.
	var lit := _unshaded(Color(1.0, 0.86, 0.48).darkened(layer * 0.15))
	var cols := maxi(1, int(size.x / 2.0))
	var rows := maxi(1, int(size.y / 2.6))
	var pane := _box(Vector3(0.7, 0.9, 0.05))
	for r in rows:
		for c in cols:
			if rng.randf() < 0.42:
				_mesh(root, pane, lit, base + Vector3(-size.x / 2.0 + (c + 0.5) * size.x / cols, 1.5 + r * 2.6, size.z / 2.0 + 0.03))


## A big round cartoon tree: a tapered trunk with bark grooves and roots, under a canopy of soft green balls.
static func _tree(root: Node3D, base: Vector3, size: float, rng: RandomNumberGenerator) -> void:
	var trunk_h := 18.0 * size
	var bark := FighterView.toon(Color(0.62, 0.42, 0.26))
	var groove := _soft(Color(0.48, 0.31, 0.19))
	_mesh(root, _cylinder(2.0 * size, 2.9 * size, trunk_h, 24), bark, base + Vector3(0, trunk_h / 2.0, 0))
	for k in 6:
		var a := -0.9 + k * 0.36
		_mesh(root, _box(Vector3(0.22 * size, trunk_h * rng.randf_range(0.5, 0.85), 0.2 * size)), groove,
			base + Vector3(sin(a) * 2.55 * size, trunk_h * 0.38, cos(a) * 2.55 * size), Vector3.ONE, Vector3(0, rad_to_deg(a), 0))
	for k in 5:
		var a := -1.2 + k * 0.6
		_mesh(root, _cylinder(0.25 * size, 1.1 * size, 3.2 * size, 10), bark,
			base + Vector3(sin(a) * 3.0 * size, 0.6 * size, cos(a) * 2.2 * size), Vector3.ONE, Vector3(0, 0, -rad_to_deg(a) * 0.6))
	var leaf := FighterView.toon(Color(0.2, 0.55, 0.3))
	var leaf_mid := _soft(Color(0.28, 0.66, 0.34))
	var leaf_light := _soft(Color(0.45, 0.8, 0.42))
	var sway := Sway.new()
	sway.position = base + Vector3(0, trunk_h, 0)
	sway.amount = 0.8
	sway.speed = 0.5
	root.add_child(sway)
	var crown := Vector3(0, 3.0 * size, 0)
	for k in 15:
		var a := TAU * k / 15.0
		var r := rng.randf_range(4.0, 9.0) * size
		var off := Vector3(cos(a) * r * 1.3, sin(a) * r * 0.55 + rng.randf_range(-1.0, 2.0) * size, rng.randf_range(-2.0, 1.5) * size)
		var s := rng.randf_range(4.0, 6.5) * size
		_mesh(sway, _sphere(1.0, 20), leaf if k % 3 == 0 else leaf_mid, crown + off, Vector3.ONE * s)
	for k in 8:
		var off := Vector3(rng.randf_range(-8.0, 8.0), rng.randf_range(1.5, 6.0), rng.randf_range(1.0, 3.0)) * size
		_mesh(sway, _sphere(1.0, 16), leaf_light, crown + off, Vector3.ONE * rng.randf_range(2.2, 3.6) * size)


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

	func _process(_delta: float) -> void:
		if lamp != null:
			lamp.visible = int(Time.get_ticks_msec() / 700) % 2 == 0


## The sea: a wide plane at `level` that swells gently, with strips of foam drifting on it. Cosmetic.
class Waves extends Node3D:
	var level := -9.0
	var foam: Array = []

	func build() -> void:
		var water := MeshInstance3D.new()
		var plane := PlaneMesh.new()
		plane.size = Vector2(500, 260)
		water.mesh = plane
		var m := StandardMaterial3D.new()
		m.albedo_color = Color(0.2, 0.56, 0.82)
		m.diffuse_mode = BaseMaterial3D.DIFFUSE_TOON
		m.specular_mode = BaseMaterial3D.SPECULAR_TOON
		m.roughness = 0.4
		water.material_override = m
		water.position = Vector3(0, level, -100)
		add_child(water)
		var fm := StandardMaterial3D.new()
		fm.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		fm.albedo_color = Color(0.9, 0.97, 1.0)
		var rng := RandomNumberGenerator.new()
		rng.seed = 4
		for k in 40:
			var f := MeshInstance3D.new()
			var b := BoxMesh.new()
			b.size = Vector3(rng.randf_range(2.0, 6.0), 0.05, 0.25)
			f.mesh = b
			f.material_override = fm
			f.position = Vector3(rng.randf_range(-120, 120), level + 0.05, rng.randf_range(-150, -8))
			add_child(f)
			foam.append([f, rng.randf() * TAU, f.position.x])

	func _process(_delta: float) -> void:
		var t := Time.get_ticks_msec() / 1000.0
		position.y = sin(t * 0.8) * 0.12
		for f in foam:
			f[0].position.x = f[2] + sin(t * 0.4 + f[1]) * 1.5
