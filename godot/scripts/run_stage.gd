extends Node3D
## The run's streamed stage, drawn (presentation only; the sim streams it, `sim_core::run`): each room in play is built once, when it
## comes into play, from its footings as the sim laid them (`SimRunner.run_pieces`: its variant, slopes and obstacle pieces), and freed
## when it leaves. **It is built from the city stage's own art** (owner, R2a: "re-use as much as possible from the existing city
## stage"): its blocks with their facade fronts and roof caps, its girder platforms, its striped awnings and shopfronts, its crates,
## cars, street lamps, potted plants, the water tower and AC unit of its rooftop, its facade shader for the building fronts behind the
## rooms, and its three skyline layers, here scrolling in parallax. New shapes only where the city has nothing: slopes (made in the
## blocks' own materials), stair flights (in the girders' steel), the dumpster, the coins, the shop's interior and its shopkeeper.
## Every footing is drawn by its `look` (docs/modes/RUNNER_PLAN.md, the art list). The rule the city kit keeps: **footing has a
## lighter top edge, decoration has none** (lamp posts, the billboard's board, the water tower's legs and tank are set back, dark or
## lit from inside, and never stand where you could). The coins spin where the sim keeps them. The left wall and an arena's exit gate
## stand where the sim's walls do.

const StageArt := preload("res://scripts/stage_art.gd")
const StageView := preload("res://scripts/stage_view.gd")
const FighterView := preload("res://scripts/fighter_view.gd")
const Materials := preload("res://scripts/materials.gd")

const ROOM_SHOP := 2
const ROOM_REST := 3
## How tall the walls are drawn above and below the ground line of the rooms in play.
const WALL_UP := 26.0
const WALL_DOWN := 14.0
## The city's skyline layers, near to far, and how much of the camera's movement each shows (0 would be fixed to the sky, 1 the play
## plane); each repeats every `SKYLINE_SPAN` (the city builds a layer from -120 to 120).
const SKYLINE_SHIFT := [0.3, 0.18, 0.08]
const SKYLINE_SPAN := 246.0
const Z := StageView.FRONT_Z

## The city's palette, by look (its awning stripes, girders, crates, its rooftop metal; new shapes in the same colours).
const LOOK_COLOURS := {
	"street": Color(0.42, 0.44, 0.54), "roof": Color(0.42, 0.44, 0.54), "awning": Color(0.92, 0.34, 0.42),
	"balcony": Color(0.5, 0.26, 0.27), "fire_escape": Color(0.3, 0.26, 0.32), "scaffold": Color(0.84, 0.48, 0.41),
	"dumpster": Color(0.3, 0.5, 0.42), "crate": Color(0.86, 0.66, 0.42), "van": Color(0.9, 0.9, 0.94),
	"ac_unit": Color(0.66, 0.68, 0.76), "water_tower": Color(0.55, 0.36, 0.26), "chimney": Color(0.5, 0.26, 0.27),
	"roof_shed": Color(0.5, 0.48, 0.58), "parapet": Color(0.56, 0.58, 0.66), "billboard": Color(0.3, 0.26, 0.32),
	"tin_roof": Color(0.56, 0.58, 0.66), "stairs": Color(0.42, 0.44, 0.54), "ramp": Color(0.42, 0.44, 0.54),
	"planter": Color(0.5, 0.46, 0.6), "bus_shelter": Color(0.2, 0.18, 0.28), "vent_pipe": Color(0.66, 0.68, 0.76),
	"shop_floor": Color(0.62, 0.44, 0.3), "shop_wall": Color(0.98, 0.9, 0.76), "shop_ceiling": Color(0.86, 0.66, 0.42),
	"counter": Color(0.58, 0.42, 0.28), "pedestal": Color(0.84, 0.48, 0.41), "shelf": Color(0.58, 0.42, 0.28),
}
## Pass-through looks drawn as the city's girder platform (with their own dressing).
const GIRDERS := ["balcony", "fire_escape", "scaffold", "billboard", "water_tower", "bus_shelter", "pedestal", "shelf", "planter"]
## The building fronts behind the rooms, in the city skyline's shades (its facade shader paints the windows).
const FACADE_SHADES := [Color(0.36, 0.24, 0.4), Color(0.3, 0.28, 0.42), Color(0.4, 0.3, 0.38)]
const FACADE_SHADER := preload("res://shaders/facade.gdshader")
const AWNING_STRIPES := [[Color(0.92, 0.34, 0.42), Color(0.98, 0.94, 0.9)], [Color(0.3, 0.6, 0.6), Color(0.94, 0.92, 0.84)],
	[Color(0.96, 0.7, 0.3), Color(0.98, 0.94, 0.86)], [Color(0.5, 0.4, 0.8), Color(0.92, 0.9, 0.98)]]

var theme := {}
var looks := PackedStringArray()
## Built rooms by their left edge (a string key): the Node3D holding their pieces.
var pieces := {}
var layout_seen := -1
var backdrop: Node3D
## The city's skyline layers (near to far), each in a holder that scrolls at its parallax, with a copy either side.
var skylines: Array = []
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
var coin_mat: Material
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


func setup(t: Dictionary) -> void:
	theme = t
	for c in get_children():
		c.queue_free()
	pieces.clear()
	coins.clear()
	ware_nodes.clear()
	price_labels.clear()
	skylines.clear()
	interior = null
	layout_seen = -1
	backdrop = Node3D.new()
	backdrop.name = "Backdrop"
	add_child(backdrop)
	var scenery := Node3D.new()
	backdrop.add_child(scenery)
	# (The city stage's whole backdrop: its moon and stars, its three skyline layers, the street below with its cars and shops. Its own
	# stage is given just below the street, so the props that stand on a stage sit hidden behind the run's buildings.)
	StageArt.backdrop(scenery, theme, [-15.0, 15.0, -9.0, -24.0, -15.0, 15.0])
	_parallax_skylines(scenery)
	wall_node = _gate(Color(0.32, 0.3, 0.36), true)
	add_child(wall_node)
	gate_node = _gate(Color(0.62, 0.22, 0.2), false)
	gate_node.visible = false
	add_child(gate_node)
	coin_mat = _mat(Color(1.0, 0.8, 0.25), 1.0)


## The city's skyline layers (`skyline_0` near to `skyline_2` far), moved into holders that scroll at their own parallax: each gets a
## copy either side, so the row of buildings carries on however far the run goes.
func _parallax_skylines(scenery: Node3D) -> void:
	for k in 3:
		var mesh := scenery.find_child("skyline_%d" % k, true, false) as MeshInstance3D
		if mesh == null:
			skylines.append(null)
			continue
		var holder := Node3D.new()
		holder.name = "Parallax%d" % k
		backdrop.add_child(holder)
		mesh.get_parent().remove_child(mesh)
		holder.add_child(mesh)
		for side in [-1.0, 1.0]:
			var copy := mesh.duplicate() as MeshInstance3D
			copy.position.x += side * SKYLINE_SPAN
			holder.add_child(copy)
		skylines.append(holder)
	mid_layer = skylines[1] if skylines.size() > 1 else null
	far_layer = skylines[2] if skylines.size() > 2 else null


## The city's toon material in colour `c` (or, glowing, its unshaded lit colour).
static func _mat(c: Color, glow := 0.0) -> Material:
	if glow > 0.0:
		return StageArt._unshaded(c)
	return FighterView.toon(c)


static func _box(parent: Node3D, size: Vector3, at: Vector3, mat: Material) -> MeshInstance3D:
	return StageArt._mesh(parent, StageArt._box(size), mat, at)


## A gate: a dark pillar of brick with iron bars across the street (placeholder).
func _gate(colour: Color, solid: bool) -> Node3D:
	var root := Node3D.new()
	root.name = "Gate"
	var mat := _mat(colour)
	if solid:
		_box(root, Vector3(1.0, WALL_UP + WALL_DOWN, 3.4), Vector3(-0.5, (WALL_UP - WALL_DOWN) * 0.5, -0.6), mat)
	var bar_mat := FighterView.respond(FighterView.toon(Color(0.2, 0.18, 0.28)), "metal")
	var bars := Node3D.new()
	bars.name = "Bars"
	root.add_child(bars)
	for n in 5:
		StageArt._mesh(bars, StageArt._cylinder(0.09, 0.09, 9.0, 8), bar_mat, Vector3(-0.5 if solid else 0.5, 4.5, 1.1 - n * 0.55))
	for y in [1.0, 4.5, 8.0]:
		_box(bars, Vector3(0.14, 0.18, 2.4), Vector3(-0.5 if solid else 0.5, y, 0.0), bar_mat)
	if not solid:
		# (An arena's gate: a red frame over the bars, so it reads as the way out.)
		_box(root, Vector3(0.5, 0.5, 3.0), Vector3(0.5, 9.2, 0.0), mat)
	return root


## The lighter top edge every footing has (the art direction's rule: what you can stand on shows a bright lip; decoration never), in
## the city blocks' cap colour.
func _top_edge(parent: Node3D, left: float, right: float, top_l: float, top_r: float, colour: Color, depth := 1.6) -> void:
	var lip := FighterView.toon(colour.lightened(0.45), false)
	var w := right - left
	var rise := top_r - top_l
	var mi := _box(parent, Vector3(sqrt(w * w + rise * rise), 0.1, depth), Vector3((left + right) * 0.5, (top_l + top_r) * 0.5 - 0.04, Z - depth * 0.5 + 0.02), lip)
	mi.rotation.z = atan2(rise, w)


## A sloped footing: a prism whose top runs from `top_l` to `top_r`, down to `bottom`, in the city blocks' own material and cap (a
## ramp, a tin roof, stairs in the street); or a thin flight in the girders' steel with treads across it (a fire escape's stairs).
func _slope(parent: Node3D, left: float, right: float, top_l: float, top_r: float, bottom: float, flight: bool, look: String) -> void:
	var depth := StageArt.PLATFORM_DEPTH if flight else 8.0
	var lo_l := top_l - 0.3 if flight else bottom
	var lo_r := top_r - 0.3 if flight else bottom
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
		var uvs := [Vector2(0, 0), Vector2(1, 0), Vector2(1, 1), Vector2(0, 1)]
		for i in [0, 2, 1, 0, 3, 2]:
			st.set_uv(uvs[i])
			st.add_vertex(q[i])
	st.generate_normals()
	var mat: Material = StageArt._surface(theme.wood, "steel", 3.0) if flight else StageArt._soil_material(theme)
	var mi := StageArt._mesh(parent, st.commit(), mat, Vector3.ZERO)
	mi.set_instance_shader_parameter("front_top", maxf(top_l, top_r))
	var cap: Color = theme.get("grass", Color(0.56, 0.58, 0.66))
	if look == "tin_roof":
		cap = LOOK_COLOURS.tin_roof
	if flight or look == "stairs":
		# Treads across it: it reads as steps.
		var steps := int((right - left) / 0.6)
		var tread := FighterView.toon((theme.wood_dark as Color).darkened(0.2), false)
		for k in steps:
			var x := left + (k + 0.5) * (right - left) / steps
			var y := top_l + (top_r - top_l) * (x - left) / (right - left)
			_box(parent, Vector3(0.06, 0.3, depth), Vector3(x, y - 0.15, Z - depth * 0.5), tread)
	if flight:
		# A railing along it (decoration: thin, dark, no lip).
		var rail := FighterView.toon(Color(0.2, 0.18, 0.28), false)
		var w := right - left
		var rise := top_r - top_l
		var r := _box(parent, Vector3(sqrt(w * w + rise * rise), 0.06, 0.06), Vector3((left + right) * 0.5, (top_l + top_r) * 0.5 + 0.9, Z - 0.05), rail)
		r.rotation.z = atan2(rise, w)
	_top_edge(parent, left, right, top_l, top_r, cap, depth)


## One footing, drawn by its look from the city's pieces. `r`: [room left, room, left, right, top, bottom, pass_through, rise, look,
## base].
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
		_slope(node, left, right, top, top + rise, bottom, pass_through, look)
		return
	if look in ["street", "roof"]:
		# The city's block: its facade front and roof cap (the cap is its light top edge).
		StageArt.solid_block(node, theme, left, right, top, bottom, Z, true)
		if look == "street":
			_lamps(node, left, right, top)
		return
	if look == "awning":
		_awning(node, left, right, top)
		return
	if pass_through or look in GIRDERS:
		# The city's girder platform (its lit plate edge is the top edge), then what makes it a balcony, a scaffold...
		StageArt.platform(node, theme, left, right, top, Z)
		_girder_dressing(node, look, left, right, top, base)
		return
	# A solid object standing on its base (for a crate on a lid, its own size).
	var from := base if base > bottom else bottom
	var h := maxf(top - from, 0.2)
	var at := Vector3(cx, from, Z - 1.0)
	match look:
		"crate":
			StageArt._crate(node, Vector3(cx, top - w, Z - w * 0.5), w, 0.0)
		"van":
			# The city's car, stretched into a delivery van.
			var van := Node3D.new()
			van.position = at
			van.scale = Vector3(w / 4.6, h / 2.08, 1.0)
			node.add_child(van)
			StageArt._car(van, colour, 1.0)
			_top_edge(node, left, right, top, top, colour, 1.9)
		"ac_unit":
			_ac_unit(node, at, w, h)
		"vent_pipe":
			var metal := FighterView.respond(FighterView.toon(LOOK_COLOURS.vent_pipe), "paint")
			StageArt._mesh(node, StageArt._cylinder(w * 0.45, w * 0.45, h, 10), metal, at + Vector3(0, h * 0.5, 0))
			StageArt._mesh(node, StageArt._cylinder(w * 0.6, w * 0.6, 0.1, 12), FighterView.toon(Color(0.42, 0.42, 0.52), false), at + Vector3(0, h, 0))
			_top_edge(node, left, right, top, top, colour, w)
		"planter":
			_box(node, Vector3(w, h, 1.6), at + Vector3(0, h * 0.5, 0), FighterView.toon(colour))
			_top_edge(node, left, right, top, top, colour)
			for k in int(w / 1.2):
				StageArt._potted_plant(node, Vector3(left + 0.6 + k * 1.2, top, Z - 1.2))
		"dumpster":
			_box(node, Vector3(w, h, 2.0), at + Vector3(0, h * 0.5, 0), FighterView.toon(colour))
			_box(node, Vector3(w + 0.12, 0.14, 2.1), at + Vector3(0, h - 0.07, 0), FighterView.toon(colour.darkened(0.3)))
			_top_edge(node, left, right, top, top, colour, 2.0)
		_:
			# A chimney, a roof shed, a parapet, the shop's counter: a box in the city's colours with its lip.
			_box(node, Vector3(w, h, 2.0), at + Vector3(0, h * 0.5, 0), FighterView.toon(colour))
			_top_edge(node, left, right, top, top, colour, 2.0)


## The city's striped awning (its shops' colours), laid flat enough to stand on, with a lighter lip.
func _awning(node: Node3D, left: float, right: float, top: float) -> void:
	var w := right - left
	var pair: Array = AWNING_STRIPES[int(absf(left)) % AWNING_STRIPES.size()]
	var n := maxi(int(w / 0.8), 1)
	for j in n:
		_box(node, Vector3(w / n, 0.14, 1.6), Vector3(left + (j + 0.5) * w / n, top - 0.07, Z - 0.8), FighterView.toon(pair[j % 2]))
		# Its valance: the stripes hanging over the front.
		_box(node, Vector3(w / n, 0.4, 0.06), Vector3(left + (j + 0.5) * w / n, top - 0.32, Z + 0.02), FighterView.toon(pair[j % 2]))
	_top_edge(node, left, right, top, top, pair[1])


## The city's street lamps along a stretch of street (decoration: thin, dark, set back, lit from inside).
func _lamps(node: Node3D, left: float, right: float, top: float) -> void:
	var iron := FighterView.toon(Color(0.2, 0.18, 0.28))
	var lamp := StageArt._unshaded(Color(1.0, 0.9, 0.62))
	var x := ceilf(left / 16.5) * 16.5 + 4.0
	while x < right - 1.0:
		var at := Vector3(x, top, -2.4)
		StageArt._mesh(node, StageArt._cylinder(0.12, 0.17, 5.0, 8), iron, at + Vector3(0, 2.5, 0))
		_box(node, Vector3(0.12, 0.12, 1.2), at + Vector3(0, 4.9, 0.5), iron)
		StageArt._mesh(node, StageArt._sphere(0.28, 10), lamp, at + Vector3(0, 4.7, 1.0), Vector3(1.0, 0.7, 1.0))
		StageArt._halo(node, at + Vector3(0, 4.6, 1.2), Color(1.0, 0.8, 0.5), 3.6)
		x += 16.5


## The city rooftop's AC unit (its recipe: box, lip, plinth, grille, fan), sized to the footing.
func _ac_unit(node: Node3D, at: Vector3, w: float, h: float) -> void:
	var metal := FighterView.respond(FighterView.toon(LOOK_COLOURS.ac_unit), "paint")
	var trim := FighterView.respond(FighterView.toon(Color(0.56, 0.58, 0.66)), "metal")
	var slat := FighterView.toon(Color(0.42, 0.42, 0.52), false)
	_box(node, Vector3(w, h, 1.2), at + Vector3(0, h * 0.5, 0), metal)
	_box(node, Vector3(w + 0.1, 0.08, 1.3), at + Vector3(0, h, 0), trim)
	_box(node, Vector3(w + 0.2, 0.12, 1.4), at + Vector3(0, 0.06, 0), FighterView.toon(Color(0.5, 0.48, 0.58)))
	for k in 5:
		_box(node, Vector3(w * 0.8, 0.05, 0.03), at + Vector3(0, h * (0.22 + k * 0.14), 0.61), slat)
	StageArt._mesh(node, StageArt._cylinder(0.42, 0.42, 0.06, 16), slat, at + Vector3(0, h + 0.03, 0))
	StageArt._contact(node, at, Vector2(w * 0.6, 0.7))


## What turns a girder into a balcony, a fire escape, a scaffold, a billboard's catwalk, a water tower's deck or a bus shelter's roof:
## railings, poles, the board, the tower's legs and tank (the city rooftop's recipe), the shelter's posts. All decoration: thin,
## dark or lit from inside, set back, with no lip.
func _girder_dressing(node: Node3D, look: String, left: float, right: float, top: float, base: float) -> void:
	var cx := (left + right) * 0.5
	var w := right - left
	var iron := FighterView.respond(FighterView.toon(Color(0.3, 0.26, 0.32)), "metal")
	match look:
		"fire_escape", "balcony":
			_box(node, Vector3(w, 0.06, 0.06), Vector3(cx, top + 0.9, Z - 0.05), iron)
			var k := left
			while k <= right + 0.01:
				_box(node, Vector3(0.05, 0.9, 0.05), Vector3(k, top + 0.45, Z - 0.05), iron)
				k += 0.7
		"scaffold":
			for lx in [left + 0.15, right - 0.15]:
				_box(node, Vector3(0.1, top - base + 0.6, 0.1), Vector3(lx, (top + base) * 0.5 + 0.3, Z - 0.1), iron)
		"billboard":
			# The board on its legs, set back, lit like the city's rooftop billboards.
			for lx in [left + w * 0.2, left + w * 0.8]:
				_box(node, Vector3(0.14, 1.0, 0.14), Vector3(lx, top + 0.5, -3.2), iron)
			_box(node, Vector3(w - 0.4, 2.6, 0.2), Vector3(cx, top + 2.3, -3.2), StageArt._unshaded(Color(0.98, 0.6, 0.8)))
			StageArt._halo(node, Vector3(cx, top + 2.3, -2.9), Color(1.0, 0.6, 0.85), w)
		"water_tower":
			var wood := Materials.wood(LOOK_COLOURS.water_tower)
			if top - base > 4.5:
				# The lid, over the tank (the city's: staves, iron hoops, a cone roof).
				StageArt._mesh(node, StageArt._cylinder(0.0, w * 0.6, 1.0, 14), FighterView.toon(Color(0.4, 0.28, 0.22)), Vector3(cx, top + 0.5, -1.6))
			else:
				for lx in [left + 0.3, right - 0.3]:
					_box(node, Vector3(0.18, top - base, 0.18), Vector3(lx, (top + base) * 0.5, -1.4), wood)
				var r := w * 0.32
				StageArt._mesh(node, StageArt._cylinder(r, r, 2.6, 14), wood, Vector3(cx, top + 1.3, -1.6))
				for hy in [0.5, 1.3, 2.1]:
					StageArt._mesh(node, StageArt._cylinder(r + 0.04, r + 0.04, 0.08, 14), iron, Vector3(cx, top + hy, -1.6))
		"bus_shelter":
			for lx in [left + 0.2, right - 0.2]:
				_box(node, Vector3(0.1, top - base, 0.1), Vector3(lx, (top + base) * 0.5, Z - 0.6), iron)
			_box(node, Vector3(1.0, 1.8, 0.1), Vector3(right - 0.6, base + 1.3, -1.2), StageArt._unshaded(Color(0.98, 0.6, 0.8)))
		"pedestal":
			_box(node, Vector3(0.5, top, 0.5), Vector3(cx, top * 0.5, -0.8), FighterView.toon(Color(0.5, 0.26, 0.27)))


## A building front behind a room, built with the city skyline's own facade shader (windows, frames, some lit), its height and shade
## from the room's number: each room reads as another building on the same street.
func _facade(node: Node3D, room: PackedFloat32Array) -> void:
	var left: float = room[0]
	var right: float = room[1]
	var ground: float = minf(room[6], room[7])
	var seq := int(room[5])
	var tall := 9.0 + float((seq * 37) % 4) * 1.5
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	StageArt._facade_box(st, Vector3(left, ground - 4.0, -8.5), Vector3(right, ground + tall, -6.0), fmod(float(seq) * 0.37, 1.0), false, tall + 4.0)
	var m := ShaderMaterial.new()
	m.shader = FACADE_SHADER
	var shade: Color = FACADE_SHADES[seq % FACADE_SHADES.size()]
	m.set_shader_parameter("base_shade", shade)
	m.set_shader_parameter("roof_shade", shade.lightened(0.12))
	m.set_shader_parameter("lit_energy", 1.9)
	m.set_shader_parameter("lit_share", 0.3)
	m.set_shader_parameter("detail", 1.0)
	m.set_shader_parameter("sign_energy", 2.0)
	var mi := StageArt._mesh(node, st.commit(), m, Vector3.ZERO)
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF


## A storefront at a shop's door: the city's shopfront (lit windows either side of a door, a striped awning, a sign).
func _storefront(parent: Node3D, door_x: float, door_y: float) -> void:
	var front := Node3D.new()
	front.name = "Storefront"
	parent.add_child(front)
	StageArt._shopfront(front, door_x, 7.0, door_y, -1.2)
	var label := Label3D.new()
	label.text = "SHOP"
	label.font_size = 96
	label.pixel_size = 0.01
	label.modulate = Color(1.0, 0.95, 0.98)
	label.outline_size = 16
	label.outline_modulate = Color(0.5, 0.1, 0.3)
	label.position = Vector3(door_x, door_y + 4.4, -1.0)
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


## Keeps the backdrop behind the view, and scrolls the city's skyline layers at their parallax (each wraps round its copies).
func follow(view_centre_x: float, view_y := 0.0) -> void:
	if backdrop != null:
		backdrop.position.x = view_centre_x
	for k in skylines.size():
		var holder: Node3D = skylines[k]
		if holder == null:
			continue
		var shift: float = SKYLINE_SHIFT[k]
		holder.position.x = -fposmod(view_centre_x * shift, SKYLINE_SPAN)
		holder.position.y = view_y * shift * 0.5


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
