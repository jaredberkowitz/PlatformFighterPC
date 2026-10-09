extends RefCounted
## Stage art: the backdrop behind a stage and the look of its blocks and platforms, by theme. Presentation only: everything here is
## built from the stage's geometry (which the simulation owns) and its look (`StageLook`: a backdrop name and sky colours, which the
## stage editor changes). Nothing here ever reaches the simulation.
##
## Themes ("backdrops"): meadow (rolling hills and clouds), grove (a floating island under a giant old tree, with wooden platforms),
## sunset (warm sky and distant mesas), night (stars and a big moon). All original art in the project's soft cel style.

const FighterView := preload("res://scripts/fighter_view.gd")

## The look of one theme: sky colours (top, horizon), the ground under the sky, block soil colours, the grass on top, platform wood.
const THEMES := {
	"meadow": {
		"sky_top": Color(0.32, 0.55, 0.9), "sky_horizon": Color(0.78, 0.9, 0.98),
		"soil": Color(0.5, 0.42, 0.55), "soil_dark": Color(0.43, 0.35, 0.48), "grass": Color(0.5, 0.78, 0.4),
		"grass_light": Color(0.66, 0.88, 0.5), "wood": Color(0.84, 0.6, 0.34), "wood_dark": Color(0.62, 0.42, 0.24),
	},
	"grove": {
		"sky_top": Color(0.42, 0.6, 0.96), "sky_horizon": Color(0.99, 0.86, 0.93),
		"soil": Color(0.86, 0.66, 0.38), "soil_dark": Color(0.72, 0.5, 0.27), "grass": Color(0.36, 0.76, 0.36),
		"grass_light": Color(0.58, 0.9, 0.45), "wood": Color(0.93, 0.74, 0.5), "wood_dark": Color(0.7, 0.48, 0.28),
	},
	"sunset": {
		"sky_top": Color(0.38, 0.3, 0.62), "sky_horizon": Color(1.0, 0.66, 0.42),
		"soil": Color(0.8, 0.52, 0.36), "soil_dark": Color(0.64, 0.38, 0.27), "grass": Color(0.86, 0.72, 0.42),
		"grass_light": Color(0.95, 0.84, 0.55), "wood": Color(0.55, 0.36, 0.28), "wood_dark": Color(0.38, 0.24, 0.2),
	},
	"night": {
		"sky_top": Color(0.05, 0.07, 0.2), "sky_horizon": Color(0.2, 0.25, 0.48),
		"soil": Color(0.36, 0.4, 0.56), "soil_dark": Color(0.27, 0.3, 0.44), "grass": Color(0.38, 0.62, 0.6),
		"grass_light": Color(0.52, 0.78, 0.74), "wood": Color(0.62, 0.66, 0.78), "wood_dark": Color(0.42, 0.45, 0.58),
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


static func _soft(c: Color) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.albedo_color = c
	m.diffuse_mode = BaseMaterial3D.DIFFUSE_TOON
	m.specular_mode = BaseMaterial3D.SPECULAR_DISABLED
	m.roughness = 1.0
	return m


## The soil pattern under a stage: rows of diamonds in two shades (a quilted look), drawn once into a small texture.
static func _soil_material(t: Dictionary) -> StandardMaterial3D:
	var size := 64
	var img := Image.create_empty(size, size, false, Image.FORMAT_RGB8)
	var a: Color = t.soil
	var b: Color = t.soil_dark
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
	var tex := ImageTexture.create_from_image(img)
	var m := StandardMaterial3D.new()
	m.albedo_texture = tex
	m.texture_filter = BaseMaterial3D.TEXTURE_FILTER_LINEAR_WITH_MIPMAPS
	m.diffuse_mode = BaseMaterial3D.DIFFUSE_TOON
	m.specular_mode = BaseMaterial3D.SPECULAR_DISABLED
	m.roughness = 1.0
	m.uv1_triplanar = true
	m.uv1_scale = Vector3(0.45, 0.45, 0.45)
	return m


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

## A solid block in the theme: patterned soil, with grass on top if nothing sits on it (a top surface fighters stand on).
static func solid_block(parent: Node3D, t: Dictionary, left: float, right: float, top: float, bottom: float, front_z: float, exposed: bool) -> void:
	var depth := 8.0
	var body := _mesh(parent, _box(Vector3(right - left, top - bottom, depth)), _soil_material(t),
		Vector3((left + right) / 2.0, (top + bottom) / 2.0, front_z - depth / 2.0))
	body.name = "block"
	if not exposed:
		return
	# A thick grass cap that rolls over the front edge.
	var grass := FighterView.toon(t.grass)
	var cap_h := 0.55
	_mesh(parent, _box(Vector3(right - left + 0.25, cap_h, depth + 0.1)), grass,
		Vector3((left + right) / 2.0, top - cap_h / 2.0 + 0.04, front_z - depth / 2.0))
	var lip := _mesh(parent, _cylinder(cap_h * 0.6, cap_h * 0.6, right - left + 0.25), grass,
		Vector3((left + right) / 2.0, top - cap_h * 0.55, front_z + 0.02), Vector3.ONE, Vector3(0, 0, 90))
	lip.name = "lip"
	# A lighter stripe along the front of the grass.
	_mesh(parent, _box(Vector3(right - left + 0.3, 0.12, 0.2)), _soft(t.grass_light),
		Vector3((left + right) / 2.0, top - 0.06, front_z + 0.3))
	# Tufts hanging over the edge, every metre or so.
	var x := left + 0.6
	var tuft_mesh := _sphere(0.32, 12)
	while x < right - 0.4:
		_mesh(parent, tuft_mesh, grass, Vector3(x, top - cap_h - 0.12, front_z + 0.2), Vector3(1.2, 0.8, 0.6))
		x += 1.15


## A pass-through platform in the theme: a rounded wooden plank (a light top board on a darker beam).
static func platform(parent: Node3D, t: Dictionary, left: float, right: float, top: float, front_z: float) -> void:
	var w := right - left
	var depth := 4.0
	var cx := (left + right) / 2.0
	var board := FighterView.toon(t.wood)
	_mesh(parent, _box(Vector3(w, 0.26, depth)), board, Vector3(cx, top - 0.13, front_z - depth / 2.0))
	_mesh(parent, _box(Vector3(w - 0.3, 0.22, depth - 0.4)), FighterView.toon(t.wood_dark), Vector3(cx, top - 0.37, front_z - depth / 2.0))
	# Rounded ends.
	for side in [-1.0, 1.0]:
		_mesh(parent, _cylinder(0.13, 0.13, depth), board, Vector3(cx + side * w / 2.0, top - 0.13, front_z - depth / 2.0), Vector3.ONE, Vector3(90, 0, 0))
	# Plank seams on the front.
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
			_sunset(root, t)
		"night":
			_night(root, t)
		_:
			_meadow(root, t)


static func _clouds(root: Node3D, rng: RandomNumberGenerator, count: int, color: Color, y_range: Vector2) -> void:
	var mat := _unshaded(color)
	for k in count:
		var cloud := Node3D.new()
		cloud.position = Vector3(rng.randf_range(-90, 90), rng.randf_range(y_range.x, y_range.y), rng.randf_range(-120, -85))
		root.add_child(cloud)
		var puffs := rng.randi_range(3, 5)
		for b in puffs:
			var s := rng.randf_range(2.4, 4.4)
			_mesh(cloud, _sphere(1.0, 16), mat, Vector3(b * 3.0 - puffs * 1.5, rng.randf_range(-0.6, 1.4), 0), Vector3(s, s * 0.8, s))


static func _hills(root: Node3D, rng: RandomNumberGenerator, near: Color, far: Color) -> void:
	for k in 8:
		var is_far := k % 2 == 1
		var w := rng.randf_range(26.0, 44.0)
		_mesh(root, _sphere(1.0, 24), _soft(far if is_far else near),
			Vector3(-105.0 + k * 30.0 + rng.randf_range(-8, 8), -22.0, -100.0 if is_far else -72.0),
			Vector3(w, rng.randf_range(10.0, 18.0), 6.0))


static func _meadow(root: Node3D, t: Dictionary) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 42
	_hills(root, rng, Color(0.55, 0.78, 0.6), Color(0.62, 0.78, 0.82))
	_clouds(root, rng, 9, Color(1, 1, 1), Vector2(14, 34))


static func _sunset(root: Node3D, t: Dictionary) -> void:
	var rng := RandomNumberGenerator.new()
	rng.seed = 7
	# A big low sun and flat-topped mesas in silhouette.
	_mesh(root, _sphere(14.0, 32), _unshaded(Color(1.0, 0.86, 0.55)), Vector3(18, 4, -140))
	for k in 7:
		var w := rng.randf_range(10.0, 22.0)
		var h := rng.randf_range(10.0, 20.0)
		var far := k % 2 == 1
		var c := Color(0.42, 0.24, 0.36) if far else Color(0.3, 0.16, 0.28)
		_mesh(root, _cylinder(w * 0.42, w * 0.55, h, 8), _unshaded(c), Vector3(-95.0 + k * 30.0, -14.0 + h / 2.0 - 8.0, -105.0 if far else -80.0))
	_clouds(root, rng, 6, Color(1.0, 0.78, 0.7), Vector2(18, 30))


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
	var crown := base + Vector3(0, trunk_h + 3.0 * size, 0)
	for k in 15:
		var a := TAU * k / 15.0
		var r := rng.randf_range(4.0, 9.0) * size
		var off := Vector3(cos(a) * r * 1.3, sin(a) * r * 0.55 + rng.randf_range(-1.0, 2.0) * size, rng.randf_range(-2.0, 1.5) * size)
		var s := rng.randf_range(4.0, 6.5) * size
		_mesh(root, _sphere(1.0, 20), leaf if k % 3 == 0 else leaf_mid, crown + off, Vector3.ONE * s)
	for k in 8:
		var off := Vector3(rng.randf_range(-8.0, 8.0), rng.randf_range(1.5, 6.0), rng.randf_range(1.0, 3.0)) * size
		_mesh(root, _sphere(1.0, 16), leaf_light, crown + off, Vector3.ONE * rng.randf_range(2.2, 3.6) * size)


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
