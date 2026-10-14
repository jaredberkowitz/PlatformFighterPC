extends Node3D
## The run's streamed stage, drawn (presentation only; the sim streams it, `sim_core::run`): each room in play is built once, when it
## comes into play, from its footings as the sim laid them (`SimRunner.run_pieces`: its variant, slopes and obstacle pieces), and freed
## when it leaves. Every footing is drawn by its `look` (a street, a roof, an awning, a dumpster...), placeholder shapes until the art
## director's city kit (docs/modes/RUNNER_PLAN.md, the art list). The rule the kit keeps: **footing has a lighter top edge, decoration
## has none** (lamp posts, sign poles, the billboard's board, the water tower's legs are thin and dark and never stand where you could).
## Behind the play plane each room has a building front of its own (brick, concrete or glass), and two parallax layers of skyline
## scroll behind them with lit windows. The coins along the way spin where the sim keeps them. A shop's storefront stands at its door;
## the shop's interior (its own little stage, far off at `INTERIOR_X`) is built here too, with the counter, the wares on their
## pedestals with their prices, and the shopkeeper, who reacts as the sim says. The left wall and an arena's exit gate stand where the
## sim's walls do.

const StageArt := preload("res://scripts/stage_art.gd")
const StageView := preload("res://scripts/stage_view.gd")

const ROOM_SHOP := 2
const ROOM_REST := 3
## How tall the walls are drawn above and below the ground line of the rooms in play.
const WALL_UP := 26.0
const WALL_DOWN := 14.0
## The parallax layers: how much of the camera's movement each shows (0 would be fixed to the sky, 1 the play plane), how far back,
## how wide one repeating tile is.
const FAR_SHIFT := 0.12
const MID_SHIFT := 0.35
const TILE := 64.0
const Z := StageView.FRONT_Z

## Placeholder colours by look (the art director replaces every one; see the art list).
const LOOK_COLOURS := {
	"street": Color(0.45, 0.45, 0.5), "roof": Color(0.58, 0.47, 0.42), "awning": Color(0.86, 0.36, 0.33),
	"balcony": Color(0.4, 0.42, 0.47), "fire_escape": Color(0.3, 0.32, 0.36), "scaffold": Color(0.83, 0.62, 0.28),
	"dumpster": Color(0.25, 0.5, 0.36), "crate": Color(0.72, 0.53, 0.3), "van": Color(0.92, 0.9, 0.84),
	"ac_unit": Color(0.72, 0.75, 0.78), "water_tower": Color(0.55, 0.38, 0.27), "chimney": Color(0.6, 0.3, 0.25),
	"roof_shed": Color(0.5, 0.52, 0.45), "parapet": Color(0.62, 0.55, 0.5), "billboard": Color(0.36, 0.36, 0.4),
	"tin_roof": Color(0.55, 0.62, 0.66), "stairs": Color(0.62, 0.6, 0.58), "ramp": Color(0.5, 0.5, 0.55),
	"planter": Color(0.45, 0.6, 0.32), "bus_shelter": Color(0.4, 0.62, 0.72), "vent_pipe": Color(0.66, 0.68, 0.7),
	"shop_floor": Color(0.62, 0.45, 0.3), "shop_wall": Color(0.95, 0.88, 0.72), "shop_ceiling": Color(0.85, 0.78, 0.64),
	"counter": Color(0.55, 0.36, 0.24), "pedestal": Color(0.85, 0.8, 0.95), "shelf": Color(0.6, 0.42, 0.3),
}
## Pass-through looks drawn as thin slabs; solid looks drawn as boxes standing on their base.
const SLABS := ["awning", "balcony", "fire_escape", "scaffold", "billboard", "water_tower", "bus_shelter", "pedestal", "shelf", "planter"]
const FACADES := [Color(0.62, 0.36, 0.3), Color(0.6, 0.6, 0.62), Color(0.42, 0.56, 0.66)]

var theme := {}
var looks := PackedStringArray()
## Built rooms by their left edge (a string key): the Node3D holding their pieces.
var pieces := {}
var layout_seen := -1
var backdrop: Node3D
var far_layer: Node3D
var mid_layer: Node3D
## The walls: the left gate and the arena's exit gate.
var wall_node: Node3D
var gate_node: Node3D
## The arena gate's bars (for the opening slide) and where it stands.
var gate_x := 0.0
var gate_shut := false
var gate_open_t := 1.0
## Coins: their spinning discs by position key.
var coins := {}
var coin_mat: StandardMaterial3D
var spin := 0.0
## The shop's interior and its shopkeeper.
var interior: Node3D
var keeper: Node3D
var keeper_body: MeshInstance3D
var keeper_bubble: Label3D
var keeper_stars: Node3D
var ware_nodes: Array = []
var price_labels: Array = []
var shop_t := 0.0
var bonks_seen := 0


func setup(t: Dictionary) -> void:
	theme = t
	for c in get_children():
		c.queue_free()
	pieces.clear()
	coins.clear()
	ware_nodes.clear()
	price_labels.clear()
	interior = null
	layout_seen = -1
	backdrop = Node3D.new()
	backdrop.name = "Backdrop"
	add_child(backdrop)
	var scenery := Node3D.new()
	backdrop.add_child(scenery)
	# (The city's sky and far haze, fixed to the camera. Its own stage is given just below the street, so the props that stand on a
	# stage sit hidden behind the run's buildings: the run's rooms are the stage.)
	StageArt.backdrop(scenery, theme, [-15.0, 15.0, -9.0, -24.0, -15.0, 15.0])
	far_layer = _skyline(-46.0, Color(0.42, 0.46, 0.6), 10.0, 26.0, 7, 0.25)
	mid_layer = _skyline(-22.0, Color(0.33, 0.33, 0.42), 6.0, 15.0, 11, 0.5)
	wall_node = _gate(Color(0.32, 0.3, 0.36), true)
	add_child(wall_node)
	gate_node = _gate(Color(0.62, 0.22, 0.2), false)
	gate_node.visible = false
	add_child(gate_node)
	coin_mat = StandardMaterial3D.new()
	coin_mat.albedo_color = Color(1.0, 0.8, 0.25)
	coin_mat.metallic = 0.6
	coin_mat.roughness = 0.3
	coin_mat.emission_enabled = true
	coin_mat.emission = Color(1.0, 0.7, 0.2)
	coin_mat.emission_energy_multiplier = 0.35


static func _mat(c: Color, glow := 0.0) -> StandardMaterial3D:
	var m := StandardMaterial3D.new()
	m.albedo_color = c
	m.roughness = 0.85
	if glow > 0.0:
		m.emission_enabled = true
		m.emission = c
		m.emission_energy_multiplier = glow
	return m


static func _box(parent: Node3D, size: Vector3, at: Vector3, mat: Material) -> MeshInstance3D:
	var mi := MeshInstance3D.new()
	var b := BoxMesh.new()
	b.size = size
	mi.mesh = b
	mi.material_override = mat
	mi.position = at
	parent.add_child(mi)
	return mi


## A parallax layer of skyline: tiles of building silhouettes with lit windows (cosmetic life), repeating every `TILE`, `z` back.
func _skyline(z: float, colour: Color, low: float, high: float, seed_: int, lit: float) -> Node3D:
	var layer := Node3D.new()
	layer.name = "Skyline"
	add_child(layer)
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_
	var wall := _mat(colour)
	var window := _mat(Color(1.0, 0.86, 0.5), 1.2)
	var dark := _mat(colour.darkened(0.25))
	var blocks := []
	var x := 0.0
	while x < TILE:
		var w := rng.randf_range(4.0, 9.0)
		var h := rng.randf_range(low, high)
		blocks.append([x, minf(w, TILE - x), h])
		x += w + rng.randf_range(0.3, 1.5)
	for k in range(-3, 4):
		var tile := Node3D.new()
		tile.position.x = k * TILE
		layer.add_child(tile)
		for b in blocks:
			var bx: float = b[0]
			var bw: float = b[1]
			var bh: float = b[2]
			_box(tile, Vector3(bw, bh + 30.0, 1.0), Vector3(bx + bw * 0.5, bh * 0.5 - 15.0, z), wall)
			for wy in range(1, int(bh / 2.2)):
				for wx in range(int(bw / 1.6)):
					var on := rng.randf() < lit
					_box(tile, Vector3(0.55, 0.75, 0.05), Vector3(bx + 0.8 + wx * 1.6, wy * 2.2 - 1.0, z + 0.52), window if on else dark)
	return layer


## A gate: a dark pillar of brick with iron bars across the street (placeholder).
func _gate(colour: Color, solid: bool) -> Node3D:
	var root := Node3D.new()
	root.name = "Gate"
	var mat := _mat(colour)
	if solid:
		_box(root, Vector3(1.0, WALL_UP + WALL_DOWN, 3.4), Vector3(-0.5, (WALL_UP - WALL_DOWN) * 0.5, -0.6), mat)
	var bar_mat := StandardMaterial3D.new()
	bar_mat.albedo_color = Color(0.14, 0.13, 0.16)
	bar_mat.metallic = 0.6
	bar_mat.roughness = 0.4
	var bars := Node3D.new()
	bars.name = "Bars"
	root.add_child(bars)
	for n in 5:
		var bar := MeshInstance3D.new()
		var c := CylinderMesh.new()
		c.top_radius = 0.09
		c.bottom_radius = 0.09
		c.height = 9.0
		bar.mesh = c
		bar.material_override = bar_mat
		bar.position = Vector3(-0.5 if solid else 0.5, 4.5, 1.1 - n * 0.55)
		bars.add_child(bar)
	for y in [1.0, 4.5, 8.0]:
		_box(bars, Vector3(0.14, 0.18, 2.4), Vector3(-0.5 if solid else 0.5, y, 0.0), bar_mat)
	if not solid:
		# (An arena's gate: a red frame over the bars, so it reads as the way out.)
		_box(root, Vector3(0.5, 0.5, 3.0), Vector3(0.5, 9.2, 0.0), mat)
	return root


## The lighter top edge every footing has (the art direction's rule: what you can stand on shows a bright lip; decoration never).
func _top_edge(parent: Node3D, left: float, right: float, top_l: float, top_r: float, colour: Color, depth := 1.6) -> void:
	var lip := _mat(colour.lightened(0.5), 0.25)
	var w := right - left
	var rise := top_r - top_l
	var mi := _box(parent, Vector3(sqrt(w * w + rise * rise), 0.1, depth), Vector3((left + right) * 0.5, (top_l + top_r) * 0.5 - 0.04, Z - depth * 0.5 + 0.02), lip)
	mi.rotation.z = atan2(rise, w)


## A sloped footing: a prism whose top runs from `top_l` to `top_r`, down to `bottom` (a solid ramp or tin roof), or a thin sloped
## slab (stairs or a fire escape's flight).
func _slope(parent: Node3D, left: float, right: float, top_l: float, top_r: float, bottom: float, slab: bool, colour: Color) -> void:
	var depth := 1.6 if slab else 8.0
	var thick := 0.3
	var lo_l := top_l - thick if slab else bottom
	var lo_r := top_r - thick if slab else bottom
	var zf := Z
	var zb := Z - depth
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var quads := [
		[Vector3(left, lo_l, zf), Vector3(right, lo_r, zf), Vector3(right, top_r, zf), Vector3(left, top_l, zf)],
		[Vector3(left, top_l, zf), Vector3(right, top_r, zf), Vector3(right, top_r, zb), Vector3(left, top_l, zb)],
		[Vector3(right, lo_r, zf), Vector3(right, lo_r, zb), Vector3(right, top_r, zb), Vector3(right, top_r, zf)],
		[Vector3(left, lo_l, zb), Vector3(left, lo_l, zf), Vector3(left, top_l, zf), Vector3(left, top_l, zb)],
	]
	for q in quads:
		for i in [0, 1, 2, 0, 2, 3]:
			st.add_vertex(q[i])
	st.generate_normals()
	var mi := MeshInstance3D.new()
	mi.mesh = st.commit()
	var mat := _mat(colour)
	mat.cull_mode = BaseMaterial3D.CULL_DISABLED
	mi.material_override = mat
	parent.add_child(mi)
	if slab:
		# (Stairs read as steps: dark treads across the flight.)
		var steps := int((right - left) / 0.6)
		var tread := _mat(colour.darkened(0.35))
		for k in steps:
			var x := left + (k + 0.5) * (right - left) / steps
			var y := top_l + (top_r - top_l) * (x - left) / (right - left)
			_box(parent, Vector3(0.06, 0.32, depth), Vector3(x, y - 0.16, Z - depth * 0.5), tread)
	_top_edge(parent, left, right, top_l, top_r, colour, depth)


## One footing, drawn by its look. `r`: [room left, room, left, right, top, bottom, pass_through, rise, look, base].
func _footing(node: Node3D, r: PackedFloat32Array) -> void:
	var left: float = r[2]
	var right: float = r[3]
	var top: float = r[4]
	var bottom: float = r[5]
	var pass_through: bool = r[6] > 0.5
	var rise: float = r[7]
	var look: String = looks[int(r[8])] if int(r[8]) < looks.size() else "roof"
	var base: float = r[9]
	var colour: Color = LOOK_COLOURS.get(look, Color(0.5, 0.5, 0.5))
	var w := right - left
	var cx := (left + right) * 0.5
	if absf(rise) > 0.001:
		_slope(node, left, right, top, top + rise, bottom, pass_through, colour)
		return
	if look == "street" or look == "roof":
		StageArt.solid_block(node, theme, left, right, top, bottom, Z, true)
		if look == "street":
			# Decoration: lamp posts along the kerb, thin and dark, behind the play plane (never footing).
			var post := _mat(Color(0.12, 0.12, 0.15))
			var lamp := _mat(Color(1.0, 0.9, 0.6), 1.5)
			var x := ceilf(left / 12.0) * 12.0 + 3.0
			while x < right - 1.0:
				_box(node, Vector3(0.12, 4.2, 0.12), Vector3(x, top + 2.1, -2.2), post)
				_box(node, Vector3(0.5, 0.18, 0.3), Vector3(x + 0.2, top + 4.2, -2.2), lamp)
				x += 12.0
		return
	if pass_through or look in SLABS:
		var thick := 0.22
		_box(node, Vector3(w, thick, 1.6), Vector3(cx, top - thick * 0.5, Z - 0.8), _mat(colour))
		_top_edge(node, left, right, top, top, colour)
		_slab_dressing(node, look, left, right, top, base, colour)
		return
	# A solid object standing on its base: a box from its base (or the object's own height for a crate on a lid) to its top.
	var from := base if base > bottom else bottom
	if look == "crate":
		from = maxf(from, top - w)
	var h := maxf(top - from, 0.2)
	_box(node, Vector3(w, h, 2.0), Vector3(cx, from + h * 0.5, Z - 1.0), _mat(colour))
	_top_edge(node, left, right, top, top, colour, 2.0)
	match look:
		"van":
			var glass := _mat(Color(0.4, 0.55, 0.7), 0.2)
			_box(node, Vector3(w * 0.25, h * 0.4, 0.05), Vector3(right - w * 0.18, from + h * 0.62, Z + 0.03), glass)
			var tyre := _mat(Color(0.1, 0.1, 0.12))
			for tx in [left + 0.8, right - 0.8]:
				_box(node, Vector3(0.7, 0.7, 0.3), Vector3(tx, from + 0.2, Z + 0.1), tyre)
		"dumpster":
			_box(node, Vector3(w + 0.1, 0.12, 2.1), Vector3(cx, top - 0.06, Z - 1.0), _mat(colour.darkened(0.3)))
		"ac_unit":
			_box(node, Vector3(h * 0.6, h * 0.6, 0.05), Vector3(cx, from + h * 0.5, Z + 0.03), _mat(Color(0.3, 0.32, 0.35)))
		"counter":
			_box(node, Vector3(w + 0.3, 0.12, 2.2), Vector3(cx, top - 0.06, Z - 1.0), _mat(colour.lightened(0.2)))


## The decoration around a slab: an awning's stripes, a water tower's legs and tank, a scaffold's poles, a billboard's board, a fire
## escape's railing. All thin, dark or set back, with no light edge: none of it holds you.
func _slab_dressing(node: Node3D, look: String, left: float, right: float, top: float, base: float, colour: Color) -> void:
	var cx := (left + right) * 0.5
	var w := right - left
	var dark := _mat(colour.darkened(0.45))
	match look:
		"awning":
			var stripe := _mat(Color(0.96, 0.92, 0.85))
			var n := int(w / 0.8)
			for k in n:
				if k % 2 == 0:
					_box(node, Vector3(w / n, 0.4, 0.05), Vector3(left + (k + 0.5) * w / n, top - 0.35, Z + 0.02), stripe)
		"water_tower":
			if top - base > 4.0:
				# The deck's legs, and the tank between the deck and its lid.
				for lx in [left + 0.3, right - 0.3]:
					_box(node, Vector3(0.14, top - base, 0.14), Vector3(lx, (top + base) * 0.5, -1.2), dark)
			else:
				var tank := MeshInstance3D.new()
				var c := CylinderMesh.new()
				c.top_radius = w * 0.5
				c.bottom_radius = w * 0.5
				c.height = 3.0
				tank.mesh = c
				tank.material_override = _mat(colour.darkened(0.1))
				tank.position = Vector3(cx, top - 1.5, -1.6)
				node.add_child(tank)
		"scaffold":
			for lx in [left + 0.15, right - 0.15]:
				_box(node, Vector3(0.1, top - base, 0.1), Vector3(lx, (top + base) * 0.5, -1.2), dark)
		"billboard":
			# The board: big, set back and unlit at its top, so it never reads as a ledge.
			_box(node, Vector3(w - 0.4, 2.6, 0.2), Vector3(cx, top + 1.6, -1.9), _mat(Color(0.2, 0.45, 0.75)))
			_box(node, Vector3(w * 0.5, 0.5, 0.05), Vector3(cx, top + 1.8, -1.75), _mat(Color(1.0, 0.85, 0.3), 0.6))
		"fire_escape", "balcony":
			_box(node, Vector3(w, 0.06, 0.06), Vector3(cx, top + 0.9, Z - 0.05), dark)
			var k := left
			while k <= right:
				_box(node, Vector3(0.05, 0.9, 0.05), Vector3(k, top + 0.45, Z - 0.05), dark)
				k += 0.7
		"pedestal":
			_box(node, Vector3(0.5, top + 0.0, 0.5), Vector3(cx, top * 0.5, -0.3), dark)


## A building front behind a room (brick, concrete or glass, by the room's number) with windows, some lit.
func _facade(node: Node3D, room: PackedFloat32Array) -> void:
	var left: float = room[0]
	var right: float = room[1]
	var ground: float = minf(room[6], room[7])
	var seq := int(room[5])
	var colour: Color = FACADES[seq % FACADES.size()]
	_box(node, Vector3(right - left, 18.0, 0.5), Vector3((left + right) * 0.5, ground + 5.0, -6.0), _mat(colour))
	var rng := RandomNumberGenerator.new()
	rng.seed = seq * 7919 + 13
	var lit := _mat(Color(1.0, 0.85, 0.5), 1.0)
	var unlit := _mat(colour.darkened(0.45))
	var x := left + 1.2
	while x < right - 1.0:
		for wy in [3.0, 6.0, 9.0, 12.0]:
			_box(node, Vector3(1.0, 1.3, 0.05), Vector3(x, ground + wy, -5.72), lit if rng.randf() < 0.3 else unlit)
		x += 2.6


## A storefront at a shop's door (placeholder): a lit window box, a door and a sign.
func _storefront(parent: Node3D, door_x: float, door_y: float) -> void:
	var front := Node3D.new()
	front.name = "Storefront"
	front.position = Vector3(door_x, door_y, -0.9)
	parent.add_child(front)
	_box(front, Vector3(7.0, 4.6, 0.6), Vector3(0, 2.3, -0.4), _mat(Color(0.86, 0.74, 0.55)))
	_box(front, Vector3(1.4, 2.6, 0.2), Vector3(0, 1.3, 0.0), _mat(Color(0.36, 0.2, 0.14)))
	var glow := _mat(Color(1.0, 0.86, 0.45), 0.8)
	for side in [-1.0, 1.0]:
		_box(front, Vector3(1.8, 1.4, 0.2), Vector3(side * 2.4, 1.8, 0.0), glow)
	var label := Label3D.new()
	label.text = "SHOP"
	label.font_size = 96
	label.pixel_size = 0.01
	label.modulate = Color(0.25, 0.12, 0.08)
	label.outline_size = 0
	label.position = Vector3(0, 3.7, 0.0)
	front.add_child(label)


## A rest stop's sign (placeholder).
func _rest_sign(parent: Node3D, x: float, y: float) -> void:
	var label := Label3D.new()
	label.text = "REST"
	label.font_size = 80
	label.pixel_size = 0.01
	label.modulate = Color(0.25, 0.55, 0.3)
	label.position = Vector3(x, y + 5.2, -1.2)
	parent.add_child(label)


## Builds what has come into play and frees what has gone, and places the walls, when the sim's stage has changed (`layout`).
func sync(sim, layout: int) -> void:
	if looks.is_empty():
		looks = sim.run_looks()
	if layout == layout_seen:
		return
	layout_seen = layout
	var wanted := {}
	var rows: Array = sim.run_pieces()
	for r in rows:
		var key := "%.3f" % r[0]
		if not wanted.has(key):
			wanted[key] = []
		wanted[key].append(r)
	var rooms := {}
	for r in sim.run_rooms():
		rooms["%.3f" % r[0]] = r
	for key in pieces.keys():
		if not wanted.has(key):
			pieces[key].queue_free()
			pieces.erase(key)
	for key in wanted.keys():
		if pieces.has(key):
			continue
		var node := Node3D.new()
		node.name = "Room"
		add_child(node)
		pieces[key] = node
		for r in wanted[key]:
			_footing(node, r)
		if rooms.has(key):
			var room: PackedFloat32Array = rooms[key]
			_facade(node, room)
			if int(room[2]) == ROOM_SHOP:
				_storefront(node, room[3], room[4])
			elif int(room[2]) == ROOM_REST:
				_rest_sign(node, (room[0] + room[1]) * 0.5, room[6])
	var view: PackedFloat32Array = sim.run_view()
	var info: PackedInt32Array = sim.run_info()
	if view.size() >= 12 and wall_node != null:
		var ground: float = view[3] + 14.0
		wall_node.position = Vector3(view[6], ground, 0.0)
		var shut := info.size() >= 12 and info[11] == 1
		if shut:
			gate_x = view[7]
			gate_node.position = Vector3(gate_x, ground, 0.0)
			gate_node.visible = true
			gate_open_t = 0.0
		elif gate_shut:
			# Just cleared: the bars slide up and away (`animate`).
			gate_open_t = 0.0001
		gate_shut = shut
	sync_coins(sim.run_coins())


## The coins: a spinning disc for each coin not yet taken.
func sync_coins(data: PackedFloat32Array) -> void:
	var seen := {}
	for n in data.size() / 4:
		var o := n * 4
		var key := "%d:%d" % [roundi(data[o] * 10.0), roundi(data[o + 1] * 10.0)]
		seen[key] = true
		var taken := data[o + 2] > 0.5
		if not coins.has(key):
			if taken:
				continue
			var mi := MeshInstance3D.new()
			var c := CylinderMesh.new()
			c.top_radius = 0.32
			c.bottom_radius = 0.32
			c.height = 0.08
			c.radial_segments = 16
			mi.mesh = c
			mi.material_override = coin_mat
			mi.position = Vector3(data[o], data[o + 1], 0.2)
			mi.rotation.x = PI * 0.5
			add_child(mi)
			coins[key] = mi
		(coins[key] as MeshInstance3D).visible = not taken
	for key in coins.keys():
		if not seen.has(key):
			coins[key].queue_free()
			coins.erase(key)


## How many coins are drawn now (for the tests).
func coins_shown() -> int:
	var n := 0
	for c in coins.values():
		if (c as Node3D).visible:
			n += 1
	return n


## The arena gate's opening, over about half a second, and the coins' spin.
func animate(delta: float) -> void:
	spin += delta * 3.0
	for c in coins.values():
		(c as Node3D).rotation.y = spin
	if gate_node == null or gate_shut or not gate_node.visible:
		return
	gate_open_t = minf(gate_open_t + delta * 2.0, 1.0)
	var bars: Node3D = gate_node.get_node("Bars")
	bars.position.y = gate_open_t * 9.5
	if gate_open_t >= 1.0:
		gate_node.visible = false
		bars.position.y = 0.0


## Keeps the backdrop behind the view, and scrolls the two skyline layers at their parallax.
func follow(view_centre_x: float, view_y := 0.0) -> void:
	if backdrop != null:
		backdrop.position.x = view_centre_x
	for pair in [[far_layer, FAR_SHIFT], [mid_layer, MID_SHIFT]]:
		var layer: Node3D = pair[0]
		if layer != null:
			var shift: float = pair[1]
			layer.position.x = view_centre_x - fposmod(view_centre_x * shift, TILE)
			layer.position.y = view_y * (1.0 - shift)


# ---- The shop's interior -------------------------------------------------------------------------------------------------------

## Builds the interior once (`SimRunner.run_interior`), with the wares' names and prices.
func build_interior(sim) -> void:
	if interior != null:
		return
	if looks.is_empty():
		looks = sim.run_looks()
	interior = Node3D.new()
	interior.name = "Interior"
	add_child(interior)
	var rows: Array = sim.run_interior()
	if rows.size() < 2:
		return
	for k in rows.size() - 1:
		var r: PackedFloat32Array = rows[k]
		# (As a placed footing row: [room, room, left, right, top, bottom, pass_through, rise, look, base].)
		var look: String = looks[int(r[6])] if int(r[6]) < looks.size() else ""
		var shell := look in ["shop_floor", "shop_wall", "shop_ceiling"]
		var row := PackedFloat32Array([r[0], 0, r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[3] if shell else 0.0])
		_footing(interior, row)
	var last: PackedFloat32Array = rows[rows.size() - 1]
	var left: float = rows[0][0]
	var right: float = rows[0][1]
	# The back wall with a window, a rug, the door you came in by.
	_box(interior, Vector3(right - left + 30.0, 40.0, 0.4), Vector3((left + right) * 0.5, 4.0, -3.5), _mat(Color(0.98, 0.9, 0.76)))
	_box(interior, Vector3(4.0, 2.6, 0.1), Vector3((left + right) * 0.5, 6.4, -3.25), _mat(Color(0.65, 0.82, 0.95), 0.6))
	_box(interior, Vector3(1.6, 2.8, 0.2), Vector3(last[0], 1.4, -3.2), _mat(Color(0.42, 0.24, 0.16)))
	var sign := Label3D.new()
	sign.text = "EXIT"
	sign.font_size = 56
	sign.pixel_size = 0.01
	sign.modulate = Color(0.3, 0.6, 0.35)
	sign.position = Vector3(last[0], 3.2, -3.1)
	interior.add_child(sign)
	# The wares on their pedestals, a price over each.
	var names: PackedStringArray = sim.run_shop_names()
	var shop: PackedInt32Array = sim.run_shop()
	var wares := shop[5] if shop.size() >= 6 else 0
	for i in wares:
		var px: float = last[5 + i * 2]
		var py: float = last[6 + i * 2]
		var kind := shop[10 + i * 4]
		var ware := Node3D.new()
		ware.position = Vector3(px, py + 0.45, 0.0)
		interior.add_child(ware)
		var mi := MeshInstance3D.new()
		var s := SphereMesh.new()
		s.radius = 0.32
		s.height = 0.64
		mi.mesh = s
		mi.material_override = _mat(Color(0.95, 0.3, 0.35) if kind == 1 else Color(0.35, 0.85, 0.5), 0.3)
		ware.add_child(mi)
		if kind == 0:
			_box(ware, Vector3(0.12, 0.4, 0.05), Vector3(0, 0, 0.33), _mat(Color.WHITE, 0.4))
			_box(ware, Vector3(0.4, 0.12, 0.05), Vector3(0, 0, 0.33), _mat(Color.WHITE, 0.4))
		ware_nodes.append(ware)
		var price := Label3D.new()
		price.font_size = 64
		price.pixel_size = 0.008
		price.outline_size = 12
		price.position = Vector3(px, py + 1.4, 0.3)
		price.text = "%s\n%d" % [names[i] if i < names.size() else "?", shop[11 + i * 4]]
		interior.add_child(price)
		price_labels.append(price)
	# The shopkeeper (placeholder for the art director's character): a round blob in an apron with a 2D face, behind the counter.
	keeper = Node3D.new()
	keeper.name = "Shopkeeper"
	keeper.position = Vector3(last[2], last[3], -0.4)
	interior.add_child(keeper)
	var r: float = last[4]
	keeper_body = MeshInstance3D.new()
	var body := SphereMesh.new()
	body.radius = r
	body.height = r * 2.0
	keeper_body.mesh = body
	keeper_body.material_override = _mat(Color(0.98, 0.78, 0.55))
	keeper_body.position = Vector3(0, r, 0)
	keeper.add_child(keeper_body)
	_box(keeper_body, Vector3(r * 1.3, r * 0.9, 0.05), Vector3(0, -r * 0.25, r * 0.92), _mat(Color(0.45, 0.7, 0.55)))
	for side in [-1.0, 1.0]:
		var eye := MeshInstance3D.new()
		var es := SphereMesh.new()
		es.radius = r * 0.13
		es.height = r * 0.26
		eye.mesh = es
		eye.material_override = _mat(Color(0.1, 0.08, 0.12))
		eye.position = Vector3(side * r * 0.3, r * 0.25, r * 0.88)
		eye.name = "Eye"
		keeper_body.add_child(eye)
	keeper_bubble = Label3D.new()
	keeper_bubble.font_size = 64
	keeper_bubble.pixel_size = 0.01
	keeper_bubble.outline_size = 14
	keeper_bubble.position = Vector3(0, r * 2.0 + 0.9, 0.4)
	keeper.add_child(keeper_bubble)
	keeper_stars = Node3D.new()
	keeper.add_child(keeper_stars)
	for k in 3:
		_box(keeper_stars, Vector3(0.18, 0.18, 0.18), Vector3(cos(k * TAU / 3.0) * r, r * 2.1, sin(k * TAU / 3.0) * r), _mat(Color(1.0, 0.9, 0.3), 1.0))


## The interior's moment: the shopkeeper's mood (waving, cheering, grumbling, dizzy), where it looks, the wares' prices (sold out),
## the ware stood by bobbing. `shop` is `SimRunner.run_shop`.
func animate_interior(shop: PackedInt32Array, delta: float) -> void:
	if interior == null or keeper == null or shop.size() < 10:
		return
	shop_t += delta
	var mood := shop[6]
	var look := shop[8]
	var r: float = (keeper_body.mesh as SphereMesh).radius
	var bob := 0.0
	var tilt := 0.0
	var squash := 1.0
	var text := ""
	match mood:
		1:
			text = "Welcome!"
			tilt = sin(shop_t * 9.0) * 0.25
		2:
			text = "Thank you!"
			bob = absf(sin(shop_t * 10.0)) * 0.5
		3:
			text = "Hmm, can't do that..."
			tilt = sin(shop_t * 30.0) * 0.08
			squash = 0.92
		4:
			text = "Hey!"
			tilt = sin(shop_t * 14.0) * 0.4
	keeper_body.position = Vector3(0, r * squash + bob, 0)
	keeper_body.scale = Vector3(1.0 / squash, squash, 1.0)
	keeper_body.rotation.z = tilt
	keeper_bubble.text = text
	keeper_bubble.visible = text != ""
	keeper_stars.visible = mood == 4
	keeper_stars.rotation.y = shop_t * 6.0
	# It looks at the ware the player stands by (its eyes turn that way).
	var gaze := 0.0
	if look >= 0 and look < ware_nodes.size():
		gaze = signf((ware_nodes[look] as Node3D).position.x - keeper.position.x)
	for e in keeper_body.get_children():
		if e.name.begins_with("Eye"):
			var side := signf((e as Node3D).position.x)
			(e as Node3D).position.x = side * r * 0.3 + gaze * r * 0.15
	for i in ware_nodes.size():
		var o := 10 + i * 4
		if shop.size() <= o + 3:
			break
		var limit := shop[o + 2]
		var sold := shop[o + 3]
		var out := limit > 0 and sold >= limit
		var ware: Node3D = ware_nodes[i]
		ware.visible = not out
		ware.rotation.y = shop_t * 1.5
		ware.scale = Vector3.ONE * (1.25 if i == look else 1.0)
		var label: Label3D = price_labels[i]
		var name := label.text.split("\n")[0]
		label.text = "%s\n%s" % [name, "SOLD OUT" if out else str(shop[o + 1])]
		label.modulate = Color(1, 1, 1, 0.5) if out else Color(1.0, 0.9, 0.4)
