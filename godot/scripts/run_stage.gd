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
const ItemIcons := preload("res://ui/item_icons.gd")

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
## R2e (the owner's playtest note): building masses stand behind the play plane at a safe depth, their fronts here: behind every
## footing's back (blocks reach 8 back from the front), so no building ever crosses or stands in front of the fighters at any zoom
## or height. Only footing, and what you stand on, is at the play depth. (`tests/runner_test.gd` checks every background mesh.)
const BACK_Z := Z - 8.5
## The view's height on the rooftops (where the city's backdrop was made to be seen from).
const BACKDROP_VIEW := 2.0

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
	# R2d (placeholders for the art director: `RUNNER_PLAN.md` 12): the skyline's kit, the climbs' faces, the interiors.
	"crane": Color(0.96, 0.7, 0.3), "girder": Color(0.84, 0.48, 0.41), "wall": Color(0.4, 0.3, 0.38),
	"ledge_block": Color(0.56, 0.58, 0.66), "office_floor": Color(0.62, 0.44, 0.3), "ceiling": Color(0.34, 0.3, 0.42),
	"desk": Color(0.58, 0.42, 0.28), "cubicle": Color(0.5, 0.48, 0.58), "bar_counter": Color(0.58, 0.42, 0.28),
	"table": Color(0.86, 0.66, 0.42), "stairwell": Color(0.3, 0.26, 0.32), "tower": Color(0.36, 0.38, 0.5),
}
## R2d: solid looks drawn as the city's block (a building's face, a tower, a floor, an interior's ceiling).
const BLOCKS := ["street", "roof", "wall", "tower", "office_floor", "ceiling"]
## Pass-through looks drawn as the city's girder platform (with their own dressing).
const GIRDERS := ["balcony", "fire_escape", "scaffold", "billboard", "water_tower", "bus_shelter", "pedestal", "shelf", "planter"]
## How far behind the play plane the city's buildings stand (their front face; the art director's depth, 2026-10-17): the building
## masses no footing hangs on stand here, smaller in the view, behind the fighters (R2e).
const FACADE_Z := -18.0
const AWNING_STRIPES := [[Color(0.92, 0.34, 0.42), Color(0.98, 0.94, 0.9)], [Color(0.3, 0.6, 0.6), Color(0.94, 0.92, 0.84)],
	[Color(0.96, 0.7, 0.3), Color(0.98, 0.94, 0.86)], [Color(0.5, 0.4, 0.8), Color(0.92, 0.9, 0.98)]]

var theme := {}
var looks := PackedStringArray()
## R2e (note B): the decks' names, lowest first, and their ground lines (each deck has its own look: the street's pavement and
## shopfronts, the rooftops' roofs, the skyline's tower tops).
var deck_names := PackedStringArray()
var deck_lines: Array = []
## Built rooms by their left edge (a string key): the Node3D holding their pieces.
var pieces := {}
var layout_seen := -1
## The current room's number last time the stage was built (a lower one is a new run).
var room_seen := 0
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
## R2b: the props (hazards and chests) by their slot, and the runaway van.
var props := {}
var van: Node3D
var van_body: Node3D
var blast_mat: StandardMaterial3D
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
## R3: the pedestals' stock as last built (the wares are rebuilt when it changes: a new shop, the bell), the bell, the speech-bubble
## card over the ware stood by, and the finds on routes (by room number).
var ware_sig := ""
var ware_spots: Array = []
var bell: Node3D
var bell_label: Label3D
var card: Node3D
var card_text: Label3D
var card_spot := -9
var finds := {}
var finds_layout := -1
## R2c: every storefront's "CLOSED" sign (hung on its door while the shopkeeper bears a grudge: it sells nothing until the player
## has gone on into a room further than the shop), and whether they show.
var closed_signs: Array = []
var shops_closed := false


func setup(t: Dictionary) -> void:
	theme = t
	for c in get_children():
		c.queue_free()
	pieces.clear()
	coins.clear()
	ware_nodes.clear()
	price_labels.clear()
	ware_sig = ""
	finds.clear()
	finds_layout = -1
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
	_still_backdrop(backdrop)
	_push_back(backdrop)
	wall_node = _gate(Color(0.32, 0.3, 0.36), true)
	add_child(wall_node)
	gate_node = _gate(Color(0.62, 0.22, 0.2), false)
	gate_node.visible = false
	add_child(gate_node)
	coin_mat = _mat(Color(1.0, 0.8, 0.25), 1.0)


## The backdrop travels with the camera (`follow`), unlike a versus stage's. So nothing in it may light or shade the run's own rooms:
## the city stage's practical lights (its string lights' warm pools and the mast's beacon, built with its hidden stage) would slide
## across the buildings as the camera moves, and its props' cast shadows would crawl over the rooms. Both are turned off here (R2b:
## the owner's report of flickering light and shadows in the run). The backdrop keeps its look: its emissive bulbs and lit windows.
static func _still_backdrop(root: Node3D) -> void:
	for n in root.find_children("*", "", true, false):
		if n is Light3D:
			(n as Light3D).visible = false
		elif n is GeometryInstance3D:
			(n as GeometryInstance3D).cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF


## R2e (the owner's playtest note): the city stage's backdrop was built for a versus stage at the play plane, so its nearest pieces
## (the stage building's underside, its rooftop props, the street) reach toward the fighters. The whole backdrop is set back until
## its front stands behind `BACK_Z`, with the buildings, at every camera height.
static func _push_back(root: Node3D) -> void:
	var front := -INF
	for n in root.find_children("*", "MeshInstance3D", true, false):
		var mi := n as MeshInstance3D
		if mi.mesh == null:
			continue
		var box := mi.transform * mi.get_aabb()
		var p := mi.get_parent()
		while p != root and p is Node3D:
			box = (p as Node3D).transform * box
			p = p.get_parent()
		front = maxf(front, box.position.z + box.size.z)
	if front > BACK_Z - 0.5:
		root.position.z = BACK_Z - 0.5 - front


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
	# (R2e, note B: what holds it up, and the deck it is on: `SimRunner.run_pieces`.)
	var support := int(r[10]) if r.size() > 12 else 0
	var held_at: float = r[11] if r.size() > 12 else base
	var deck := _deck_name(int(r[12])) if r.size() > 12 else ""
	if support == SUPPORT_POSTS and pass_through:
		base = held_at
	var colour: Color = LOOK_COLOURS.get(look, Color(0.5, 0.5, 0.5))
	var w := right - left
	var cx := (left + right) * 0.5
	if absf(rise) > 0.001:
		_slope(node, left, right, top, top + rise, bottom, pass_through, look)
		return
	if look in BLOCKS:
		if (look == "street" or look == "roof") and deck == "street" and bottom < top - 6.0:
			# The street deck is the ground (R2e, note B): a pavement and the road, the earth under them; no windows below your feet.
			_pavement(node, left, right, top, bottom)
			_lamps(node, left, right, top)
			return
		# The city's block: its facade front and roof cap (the cap is its light top edge). On the rooftops a roof with its parapet,
		# the building going down out of view; on the skyline a tower's top.
		StageArt.solid_block(node, theme, left, right, top, bottom, Z, true)
		if bottom < top - 6.0 and look != "ceiling" and look != "office_floor":
			_parapet(node, left, right, top, deck == "skyline" or look == "tower")
		return
	if look == "awning":
		_awning(node, left, right, top)
		_held(node, look, support, left, right, top, held_at)
		return
	if pass_through or look in GIRDERS:
		# The city's girder platform (its lit plate edge is the top edge), then what makes it a balcony, a scaffold...
		StageArt.platform(node, theme, left, right, top, Z)
		_girder_dressing(node, look, left, right, top, base)
		_held(node, look, support, left, right, top, held_at)
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


## `SimRunner.run_pieces`' support numbers (`sim_core::run::support_of`).
const SUPPORT_WALL := 2
const SUPPORT_SPAN := 3
const SUPPORT_POSTS := 4
## Looks whose own dressing already stands on legs down to their base.
const LEGGED := ["scaffold", "water_tower", "bus_shelter"]


## The name of deck `d` ("" when unknown).
func _deck_name(d: int) -> String:
	return deck_names[d] if d >= 0 and d < deck_names.size() else ""


## The deck whose line is nearest height `y` (presentation only: the lines are the sim's, read once).
func _deck_of(y: float) -> int:
	if deck_lines.is_empty():
		return -1
	var best := 0
	for d in deck_lines.size():
		if absf(y - float(deck_lines[d])) < absf(y - float(deck_lines[best])):
			best = d
	return best


## What holds a free footing up (R2e, the owner's note B: nothing floats): a truss under a span between two faces, stilts or a crane's
## mast down to the surface under it, brackets back to the building it hangs on. Decoration: thin, dark, no lip.
func _held(node: Node3D, look: String, support: int, left: float, right: float, top: float, down_to: float) -> void:
	var iron := FighterView.respond(FighterView.toon(Color(0.28, 0.24, 0.3)), "metal")
	var w := right - left
	match support:
		SUPPORT_SPAN:
			# A truss under the deck: a bottom chord and diagonals.
			var under := top - 1.0
			_box(node, Vector3(w, 0.08, 0.08), Vector3((left + right) * 0.5, under, Z - 0.6), iron)
			var n := maxi(int(w / 1.2), 1)
			for k in n:
				var x0 := left + k * w / n
				var x1 := left + (k + 1) * w / n
				var a := Vector2(x0, top - 0.7 if k % 2 == 0 else under)
				var b := Vector2(x1, under if k % 2 == 0 else top - 0.7)
				var d := b - a
				var bar := _box(node, Vector3(d.length(), 0.06, 0.06), Vector3((a.x + b.x) * 0.5, (a.y + b.y) * 0.5, Z - 0.6), iron)
				bar.rotation.z = d.angle()
		SUPPORT_POSTS:
			if look in LEGGED:
				return
			var h := top - down_to
			if h <= 0.2:
				return
			if look == "crane":
				# A crane's lattice mast under the jib's middle, yellow like the jib, with a cab at its top.
				var paint := FighterView.toon(LOOK_COLOURS.crane)
				var mx := (left + right) * 0.5
				for side in [-0.35, 0.35]:
					_box(node, Vector3(0.14, h, 0.14), Vector3(mx + side, down_to + h * 0.5, Z - 1.4), paint)
				var y := down_to + 0.6
				var k := 0
				while y < top - 0.8:
					var brace := _box(node, Vector3(1.0, 0.07, 0.07), Vector3(mx, y, Z - 1.4), paint)
					brace.rotation.z = 0.6 if k % 2 == 0 else -0.6
					y += 1.2
					k += 1
				_box(node, Vector3(1.2, 1.0, 1.2), Vector3(mx, top - 1.2, Z - 1.4), FighterView.toon(Color(0.9, 0.9, 0.94)))
				return
			# Stilts at both ends (and every few units along a long one).
			var xs: Array = [left + 0.25, right - 0.25]
			var x := left + 7.0
			while x < right - 5.0:
				xs.append(x)
				x += 7.0
			for px in xs:
				_box(node, Vector3(0.12, h, 0.12), Vector3(px, down_to + h * 0.5, Z - 0.6), iron)
		SUPPORT_WALL:
			# Beams back to the building's face (behind the play plane); an awning over ground just below it shelters a shop's door and
			# window on that face.
			var reach := (Z - StageArt.PLATFORM_DEPTH) - BACK_Z
			for px in [left + 0.3, right - 0.3]:
				_box(node, Vector3(0.08, 0.08, reach), Vector3(px, top - 0.35, BACK_Z + reach * 0.5), iron)
			if look == "awning" and top - down_to > 2.8 and top - down_to < 4.8:
				_shop_window(node, (left + right) * 0.5, minf(w - 0.6, 5.0), down_to, BACK_Z)


## A shop's ground floor on a building's face: a lit window beside a door, in a dark frame (no awning: awnings are footings).
func _shop_window(node: Node3D, cx: float, w: float, base: float, front: float) -> void:
	_box(node, Vector3(w, 3.2, 0.12), Vector3(cx, base + 1.6, front + 0.06), FighterView.toon(Color(0.3, 0.22, 0.34)))
	_box(node, Vector3(w * 0.55, 2.0, 0.08), Vector3(cx - w * 0.18, base + 1.5, front + 0.13), StageArt._unshaded(Color(0.95, 0.8, 0.55)))
	_box(node, Vector3(w * 0.22, 2.5, 0.08), Vector3(cx + w * 0.32, base + 1.25, front + 0.13), FighterView.toon(Color(0.22, 0.15, 0.2)))


## The street deck's ground (R2e, note B): a pale pavement with a kerb along its front and the gutter under it, and the earth below
## (no windows: it is the ground, not a building).
func _pavement(node: Node3D, left: float, right: float, top: float, bottom: float) -> void:
	var w := right - left
	var cx := (left + right) * 0.5
	var earth := _mat(Color(0.18, 0.16, 0.22))
	_box(node, Vector3(w, top - bottom - 0.4, 8.0), Vector3(cx, (top - 0.4 + bottom) * 0.5, Z - 4.0), earth)
	var slab := _mat(Color(0.62, 0.6, 0.66))
	_box(node, Vector3(w, 0.4, 8.0), Vector3(cx, top - 0.2, Z - 4.0), slab)
	# Paving joints across it.
	var joint := FighterView.toon(Color(0.5, 0.48, 0.55), false)
	var x := ceilf(left / 1.6) * 1.6
	while x < right:
		_box(node, Vector3(0.05, 0.02, 8.0), Vector3(x, top + 0.005, Z - 4.0), joint)
		x += 1.6
	# The kerb: a lighter stone lip at the front, the gutter's dark line under it.
	_box(node, Vector3(w, 0.16, 0.3), Vector3(cx, top - 0.08, Z + 0.05), _mat(Color(0.82, 0.8, 0.84)))
	_box(node, Vector3(w, 0.5, 0.06), Vector3(cx, top - 0.65, Z + 0.02), _mat(Color(0.1, 0.09, 0.13)))


## A parapet along a roof's back edge (the low wall round a roof); a tower's top gets a darker crown with a beacon on a mast.
func _parapet(node: Node3D, left: float, right: float, top: float, tower: bool) -> void:
	var c := Color(0.3, 0.32, 0.44) if tower else Color(0.5, 0.42, 0.5)
	_box(node, Vector3(right - left, 0.7, 0.4), Vector3((left + right) * 0.5, top + 0.35, Z - 7.6), _mat(c))
	if tower and right - left > 6.0:
		_box(node, Vector3(0.1, 2.2, 0.1), Vector3(right - 1.5, top + 1.1, Z - 6.5), FighterView.toon(Color(0.2, 0.18, 0.28)))
		StageArt._mesh(node, StageArt._sphere(0.16, 8), StageArt._unshaded(Color(1.0, 0.3, 0.3)), Vector3(right - 1.5, top + 2.3, Z - 6.5))


## A building mass behind the room (R2e, note B; `SimRunner.run_buildings`: [room's left edge, left, right, top, kind, the deck its
## top is on, the ground under its middle]), drawn as the city's block (its brick front with windows, its roof's coping) set behind the
## play plane, reaching down out of view. On the street its ground floor is shopfronts or doors, and the facades rise out of view. An
## interior's building stands a little further back, so the room's own back wall shows in front of it.
func _building(node: Node3D, b: PackedFloat32Array, indoor: bool) -> void:
	var left: float = b[1]
	var right: float = b[2]
	var top: float = b[3]
	var kind := int(b[4])
	var base: float = b[6]
	var on_street := _deck_name(_deck_of(base)) == "street" and top - base > 2.0
	if on_street and not indoor:
		top = maxf(top, base + 22.0)
	# (A building that carries a balcony, a fire escape, an awning or a sill stands just behind the play plane, so what hangs on it
	# meets its face; the rest of the city stands well back, at the art director's `FACADE_Z`, behind the fighters at a smaller scale.)
	var carries := b.size() > 7 and b[7] > 0.5
	var front := BACK_Z if carries or indoor else FACADE_Z
	StageArt.solid_block(node, theme, left, right, top, minf(base, top) - 40.0, front, true)
	if kind == 4:
		# A glass tower: bands of glass across its front.
		var glass := StageArt._unshaded(Color(0.55, 0.7, 0.95))
		var y := top - 1.5
		while y > top - 18.0:
			_box(node, Vector3(right - left - 0.6, 0.5, 0.05), Vector3((left + right) * 0.5, y, front + 0.03), glass)
			y -= 2.4
	if on_street and not indoor:
		# Its ground floor: shopfronts (a row of shops) or doors under a lintel (the others), along its foot.
		var w := right - left
		var n := maxi(int(w / 7.0), 1)
		for k in n:
			var cx := left + (k + 0.5) * w / n
			if kind == 2:
				_shop_window(node, cx, minf(w / n - 0.8, 5.0), base, front)
			else:
				_box(node, Vector3(1.4, 2.4, 0.1), Vector3(cx, base + 1.2, front + 0.06), FighterView.toon(Color(0.25, 0.17, 0.2)))
				_box(node, Vector3(1.8, 0.2, 0.2), Vector3(cx, base + 2.5, front + 0.08), FighterView.toon(Color(0.6, 0.5, 0.45)))


## An interior (R2d, placeholder for the art director): under each ceiling, a warm back wall with a row of lamps, and a window pane at
## either end (the windows you crash in and burst out of).
func _interior(node: Node3D, room: PackedFloat32Array, rows: Array) -> void:
	var wall := _mat(Color(0.62, 0.5, 0.42))
	var glass := StandardMaterial3D.new()
	glass.albedo_color = Color(0.7, 0.86, 0.98, 0.35)
	glass.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	var lamp := _mat(Color(1.0, 0.92, 0.7), 1.0)
	for r in rows:
		var look: String = looks[int(r[8])] if int(r[8]) < looks.size() else ""
		if look != "ceiling":
			continue
		var left: float = r[2]
		var right: float = r[3]
		var under: float = r[5]
		# (The floor under it: the room's lower ground end, or its higher one for a ceiling above both.)
		var floor_y: float = minf(room[6], room[7]) if under < maxf(room[6], room[7]) else maxf(room[6], room[7])
		var h := under - floor_y
		if h <= 0.5:
			continue
		_box(node, Vector3(right - left, h, 0.3), Vector3((left + right) * 0.5, floor_y + h * 0.5, Z - 2.6), wall)
		var k := left + 3.0
		while k < right - 2.0:
			_box(node, Vector3(1.2, 0.15, 0.6), Vector3(k, under - 0.12, Z - 1.6), lamp)
			k += 6.0
		for x in [left, right]:
			_box(node, Vector3(0.12, h, 1.8), Vector3(x, floor_y + h * 0.5, Z - 0.6), glass)


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
	# Its canvas back to the building's face, rising a little (behind the play plane: R2e).
	var back := (Z - 1.6) - BACK_Z
	var sheet := _box(node, Vector3(w, 0.08, back), Vector3((left + right) * 0.5, top + 0.15, BACK_Z + back * 0.5), FighterView.toon((pair[0] as Color).darkened(0.15)))
	sheet.rotation.x = -0.06


## The city's street lamps along a stretch of street (decoration: thin, dark, set back, lit from inside).
func _lamps(node: Node3D, left: float, right: float, top: float) -> void:
	var iron := FighterView.toon(Color(0.2, 0.18, 0.28))
	var lamp := StageArt._unshaded(Color(1.0, 0.9, 0.62))
	var x := ceilf(left / 16.5) * 16.5 + 4.0
	while x < right - 1.0:
		# (Against the buildings' faces, behind the awnings' canvas: R2e.)
		var at := Vector3(x, top, BACK_Z + 0.8)
		# (Short, under the awnings' height.)
		StageArt._mesh(node, StageArt._cylinder(0.12, 0.17, 2.8, 8), iron, at + Vector3(0, 1.4, 0))
		_box(node, Vector3(0.12, 0.12, 1.2), at + Vector3(0, 2.9, 0.5), iron)
		StageArt._mesh(node, StageArt._sphere(0.28, 10), lamp, at + Vector3(0, 2.7, 1.0), Vector3(1.0, 0.7, 1.0))
		# (A small glow round the bulb, art review 6 round 11: at 3.6 across, at the fighters' depth, each lamp bloomed into a white blob as
		# bright as anything in play, as the city stage's shops did before round 7.)
		StageArt._halo(node, at + Vector3(0, 2.6, 1.2), Color(1.0, 0.8, 0.5), 2.0)
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
	# A board hung across the door, tilted, while it is closed to you (placeholder: the art director's turned-round sign replaces it).
	var closed := Label3D.new()
	closed.name = "Closed"
	closed.text = "CLOSED"
	closed.font_size = 72
	closed.pixel_size = 0.01
	closed.modulate = Color(1.0, 0.92, 0.85)
	closed.outline_size = 18
	closed.outline_modulate = Color(0.6, 0.08, 0.1)
	closed.position = Vector3(door_x, door_y + 1.7, -0.7)
	closed.rotation.z = deg_to_rad(-8.0)
	closed.visible = shops_closed
	front.add_child(closed)
	closed_signs.append(closed)


## Hangs (or takes down) the "CLOSED" board on every storefront: `closed` is the shop's grudge (`SimRunner.run_shop`'s last number).
func show_closed(closed: bool) -> void:
	shops_closed = closed
	var kept: Array = []
	for c in closed_signs:
		if is_instance_valid(c):
			(c as Label3D).visible = closed
			kept.append(c)
	closed_signs = kept


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
	if deck_names.is_empty():
		deck_names = sim.run_deck_names()
		deck_lines = Array(sim.run_deck_lines())
	# (Every frame: a coin picked up changes no layout. R2a only redrew the coins when the stage changed, so a collected coin stayed on
	# screen until the next room came: the owner's "collected coins don't disappear".)
	sync_coins(sim.run_coins())
	if layout == layout_seen:
		return
	layout_seen = layout
	# (A fresh run, a debug restart included: the rooms of the last one go, though a new one may stand where they stood.)
	var placed: int = sim.run_info()[8] if sim.run_info().size() > 8 else 0
	if placed < room_seen:
		for key in pieces.keys():
			pieces[key].queue_free()
		pieces.clear()
	room_seen = placed
	var wanted := {}
	var rows: Array = sim.run_pieces()
	for r in rows:
		var key := "%.3f" % r[0]
		if not wanted.has(key):
			wanted[key] = []
		wanted[key].append(r)
	var rooms := {}
	# (R2d: a room's key carries its lid, so a climb is redrawn with its lid once it closes.)
	var lidded := {}
	for r in sim.run_rooms():
		var k := "%.3f" % r[0]
		if r.size() > 14 and r[14] > 0.5:
			lidded[k] = true
			k += "L"
		rooms[k] = r
	var buildings := {}
	for b in sim.run_buildings():
		var bk := "%.3f" % b[0]
		if not buildings.has(bk):
			buildings[bk] = []
		buildings[bk].append(b)
	var keyed := {}
	for key in wanted.keys():
		keyed[key + ("L" if lidded.has(key) else "")] = wanted[key]
	wanted = keyed
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
			# (R2e, note B: the city the room stands in, its building masses, instead of R2a's one front per room.)
			var masses := Node3D.new()
			masses.name = "Buildings"
			node.add_child(masses)
			for b in buildings.get(key.trim_suffix("L"), []):
				_building(masses, b, room.size() > 11 and room[11] > 0.5)
			if int(room[2]) == ROOM_SHOP:
				_storefront(node, room[3], room[4])
			elif int(room[2]) == ROOM_REST:
				_rest_sign(node, (room[0] + room[1]) * 0.5, room[6])
			if room.size() > 11 and room[11] > 0.5:
				_interior(node, room, wanted[key])
			if room.size() > 14 and room[14] > 0.5:
				# The lid over a climb left behind (R2d): the building's roof closing the way back down; over a drop (R2e) a ceiling a
				# little over its floor, closing the way back up.
				var lid_top: float = room[16] if room.size() > 16 else room[7]
				StageArt.solid_block(node, theme, room[0], room[1], lid_top, lid_top - 1.0, Z, true)
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
		# (R2e, note B: the backdrop follows the view's height too, so the city stage's own rooftop props, built for a stage at the
		# rooftops' height, always stay below the view instead of floating mid-facade on the street deck; above the rooftops the far
		# city sinks a little, so the skyline deck looks out over it.)
		backdrop.position.y = view_y - BACKDROP_VIEW - maxf(view_y - BACKDROP_VIEW, 0.0) * 0.5
	for k in skylines.size():
		var holder: Node3D = skylines[k]
		if holder == null:
			continue
		var shift: float = SKYLINE_SHIFT[k]
		holder.position.x = -fposmod(view_centre_x * shift, SKYLINE_SPAN)


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
	# The wares' stands (R3: two staples' stands, then four pedestals), filled by `sync_wares`; the bell at the counter's left end.
	ware_spots.clear()
	for i in (last.size() - 5) / 2:
		ware_spots.append(Vector2(last[5 + i * 2], last[6 + i * 2]))
	var counter_top := 1.3
	var bell_x: float = last[0]
	for k in rows.size() - 1:
		var r: PackedFloat32Array = rows[k]
		if int(r[6]) < looks.size() and looks[int(r[6])] == "counter":
			counter_top = r[2]
			bell_x = r[0] + 0.35
	bell = Node3D.new()
	bell.name = "Bell"
	bell.position = Vector3(bell_x, counter_top, 0.1)
	interior.add_child(bell)
	var dome := MeshInstance3D.new()
	var ds := SphereMesh.new()
	ds.radius = 0.22
	ds.height = 0.3
	ds.is_hemisphere = true
	dome.mesh = ds
	dome.material_override = _mat(Color(1.0, 0.8, 0.3), 0.5)
	bell.add_child(dome)
	_box(bell, Vector3(0.5, 0.05, 0.4), Vector3(0, 0.0, 0), _mat(Color(0.4, 0.26, 0.18)))
	_box(bell, Vector3(0.05, 0.08, 0.05), Vector3(0, 0.3, 0), _mat(Color(0.9, 0.7, 0.3)))
	bell_label = Label3D.new()
	bell_label.font_size = 52
	bell_label.pixel_size = 0.008
	bell_label.outline_size = 12
	bell_label.position = Vector3(0, 1.1, 0.3)
	bell.add_child(bell_label)
	# The speech-bubble card over the ware stood by (R3).
	card = Node3D.new()
	card.name = "Card"
	card.visible = false
	interior.add_child(card)
	var paper := _box(card, Vector3(4.4, 1.7, 0.05), Vector3(0, 0, 0), _mat(Color(0.99, 0.96, 0.88), 1.0))
	paper.name = "Paper"
	_box(card, Vector3(4.55, 1.85, 0.04), Vector3(0, 0, -0.02), _mat(Color(0.12, 0.1, 0.14)))
	var tail := _box(card, Vector3(0.3, 0.3, 0.05), Vector3(0, -0.95, 0), _mat(Color(0.99, 0.96, 0.88)))
	tail.rotation.z = PI / 4.0
	card_text = Label3D.new()
	card_text.font_size = 40
	card_text.pixel_size = 0.0085
	card_text.width = 480.0
	card_text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	card_text.modulate = Color(0.12, 0.1, 0.14)
	card_text.outline_size = 0
	card_text.position = Vector3(0, 0, 0.05)
	card.add_child(card_text)
	sync_wares(sim)
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


## The interior's moment: the shopkeeper's mood (waving, cheering, grumbling, dizzy, sulking), where it looks, the wares' prices (sold out),
## the ware stood by bobbing. `shop` is `SimRunner.run_shop`.
func animate_interior(shop: PackedInt32Array, delta: float, names := PackedStringArray(), blurbs := PackedStringArray()) -> void:
	if interior == null or keeper == null or shop.size() < SHOP_HEAD:
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
		5:
			# Sulking after the bonk (R2b): arms crossed, turned half away, and it will not sell.
			text = "Hmph!"
			tilt = -0.18
			squash = 1.04
		6:
			# The bell rung (R3): fresh stock.
			text = "Fresh stock!"
			bob = absf(sin(shop_t * 14.0)) * 0.3
			tilt = sin(shop_t * 12.0) * 0.15
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
		var o := SHOP_HEAD + i * WARE_STRIDE
		if shop.size() <= o + 6:
			break
		var ware: Node3D = ware_nodes[i]
		var label: Label3D = price_labels[i]
		if ware == null:
			label.text = "EMPTY"
			label.modulate = Color(1, 1, 1, 0.35)
			continue
		var out := shop[o + 4] > 0 and shop[o + 5] >= shop[o + 4]
		ware.visible = not out
		ware.rotation.y = shop_t * 1.5
		ware.scale = Vector3.ONE * (1.25 if i == look else 1.0)
		label.text = "SOLD" if out else str(shop[o + 3])
		label.modulate = Color(1, 1, 1, 0.5) if out else Color(1.0, 0.9, 0.4)
	# The bell's price, and the card over the ware stood by: its name, what it does, its price and how many are owned.
	if bell_label != null:
		bell_label.text = "BELL %d" % shop[11]
		bell.scale = Vector3.ONE * (1.25 if shop[1] == BELL_SPOT else 1.0)
		if mood == 6:
			bell.rotation.z = sin(shop_t * 40.0) * 0.25
		else:
			bell.rotation.z = 0.0
	var spot := shop[1]
	var o2 := SHOP_HEAD + spot * WARE_STRIDE
	var show := spot >= 0 and spot < ware_spots.size() and shop.size() > o2 + 6 and shop[o2] >= 0
	card.visible = show
	if show:
		var at: Vector2 = ware_spots[spot]
		card.position = Vector3(clampf(at.x, ware_spots[0].x + 0.4, ware_spots[ware_spots.size() - 1].x), at.y + 3.4, 0.6)
		var out := shop[o2 + 4] > 0 and shop[o2 + 5] >= shop[o2 + 4]
		var name := names[spot] if spot < names.size() else "?"
		var blurb := blurbs[spot] if spot < blurbs.size() else ""
		var rarity := shop[o2 + 2]
		var tag: String = ItemIcons.RARITY_NAMES[rarity] if rarity >= 0 and rarity < 4 else ""
		var price_line := "SOLD" if out else "%d coins" % shop[o2 + 3]
		card_text.text = "%s  (%s)\n%s\n%s    You own %d" % [name.to_upper(), tag, blurb, price_line, shop[o2 + 6]]
		var paper: MeshInstance3D = card.get_node("Paper")
		var rc := ItemIcons.rarity_colour(rarity)
		var pm := paper.material_override as StandardMaterial3D
		if pm != null:
			pm.albedo_color = Color(0.99, 0.96, 0.88).lerp(rc, 0.12)


## The shop's numbers (`SimRunner.run_shop`): the header's length, each ware's, and the bell's spot.
const SHOP_HEAD := 14
const WARE_STRIDE := 7
const BELL_SPOT := 6


## R3: the wares on their stands as the sim has them now (`SimRunner.run_shop`): rebuilt when the stock changes (a new shop, the
## bell). Each is its item's gem in its rarity's halo, with its price under it.
func sync_wares(sim) -> void:
	if interior == null:
		return
	var shop: PackedInt32Array = sim.run_shop()
	if shop.size() < SHOP_HEAD:
		return
	var spots := shop[5]
	var sig := ""
	for i in spots:
		sig += "%d," % shop[SHOP_HEAD + i * WARE_STRIDE]
	if sig == ware_sig:
		return
	ware_sig = sig
	for n in ware_nodes:
		if n != null:
			(n as Node3D).queue_free()
	for l in price_labels:
		(l as Node3D).queue_free()
	ware_nodes.clear()
	price_labels.clear()
	for i in mini(spots, ware_spots.size()):
		var o := SHOP_HEAD + i * WARE_STRIDE
		var at: Vector2 = ware_spots[i]
		var ware: Node3D = null
		if shop[o] >= 0:
			ware = ItemIcons.gem(shop[o + 1], shop[o + 2])
			ware.position = Vector3(at.x, at.y + 0.45, 0.0)
			interior.add_child(ware)
		ware_nodes.append(ware)
		var price := Label3D.new()
		price.font_size = 64
		price.pixel_size = 0.008
		price.outline_size = 12
		price.position = Vector3(at.x, at.y + 1.25, 0.3)
		interior.add_child(price)
		price_labels.append(price)


## How many wares stand on their stands now (for the tests).
func wares_shown() -> int:
	var n := 0
	for w in ware_nodes:
		if w != null:
			n += 1
	return n


# ---- R3 (F8): finds on routes ---------------------------------------------------------------------------------------------------

## The finds in the rooms in play (`SimRunner.run_finds`): an item's gem on a little stand, food (an apple), a 1-up (a green bottle
## with "1UP"); a taken one is gone. Rebuilt when the stage's layout changes; they bob.
func sync_finds(sim, layout: int) -> void:
	if layout != finds_layout:
		finds_layout = layout
		var rows: Array = sim.run_finds()
		var seen := {}
		for r in rows:
			var row: PackedFloat32Array = r
			var key := int(row[6])
			seen[key] = true
			if row[5] > 0.5:
				if finds.has(key):
					finds[key].queue_free()
					finds.erase(key)
				continue
			if finds.has(key):
				continue
			var node := Node3D.new()
			node.name = "Find"
			node.position = Vector3(row[0], row[1], Z - 0.2)
			add_child(node)
			_box(node, Vector3(0.7, 0.2, 0.7), Vector3(0, 0.1, 0), _mat(Color(0.84, 0.48, 0.41)))
			var what := int(row[2])
			var top: Node3D
			match what:
				1:
					top = ItemIcons.gem(int(row[3]), int(row[4]), 0.36)
				2:
					top = Node3D.new()
					var apple := MeshInstance3D.new()
					var sm := SphereMesh.new()
					sm.radius = 0.3
					sm.height = 0.56
					apple.mesh = sm
					apple.material_override = _mat(Color(0.9, 0.18, 0.18), 0.2)
					top.add_child(apple)
					_box(top, Vector3(0.05, 0.18, 0.05), Vector3(0, 0.32, 0), _mat(Color(0.4, 0.25, 0.1)))
					_box(top, Vector3(0.18, 0.06, 0.05), Vector3(0.1, 0.36, 0), _mat(Color(0.3, 0.75, 0.3)))
				_:
					top = ItemIcons.gem(ItemIcons.LIFE, 3, 0.36)
					(top.get_node("Gem") as MeshInstance3D).material_override = _mat(Color(0.3, 0.85, 0.4), 0.4)
			top.name = "Top"
			top.position = Vector3(0, 0.75, 0)
			node.add_child(top)
			finds[key] = node
		for key in finds.keys():
			if not seen.has(key):
				finds[key].queue_free()
				finds.erase(key)
	var t := Time.get_ticks_msec() * 0.001
	for key in finds:
		var top: Node3D = finds[key].get_node("Top")
		top.position.y = 0.75 + sin(t * 3.0 + float(key)) * 0.12
		top.rotation.y = t * 1.5


## How many finds are drawn now (for the tests).
func finds_shown() -> int:
	return finds.size()


# ---- R2b: props and the runaway van (placeholders for the art director: see RUNNER_PLAN.md's R2b art list) ------------------------

const PROP_GAS := 0
const PROP_SIGN := 1
const PROP_CHEST := 2
const PROP_LIT := 1
const PROP_BURST := 2
const PROP_FALLING := 3
const PROP_BROKEN := 4
const PROP_OPEN := 5


## A prop's placeholder: a red gas canister with a yellow band and a valve; a loose sign (a board on two wires, a "!" hazard stripe); a
## chest (a wooden box with a gold lid and lock).
func _prop_node(role: int, half_w: float, height: float) -> Node3D:
	var root := Node3D.new()
	match role:
		PROP_GAS:
			var red := FighterView.respond(FighterView.toon(Color(0.86, 0.22, 0.2)), "paint")
			StageArt._mesh(root, StageArt._cylinder(half_w, half_w, height, 14), red, Vector3(0, height * 0.5, 0))
			StageArt._mesh(root, StageArt._cylinder(half_w * 1.02, half_w * 1.02, height * 0.16, 14), FighterView.toon(Color(1.0, 0.82, 0.2)), Vector3(0, height * 0.55, 0))
			StageArt._mesh(root, StageArt._cylinder(half_w * 0.3, half_w * 0.3, 0.2, 8), FighterView.toon(Color(0.3, 0.3, 0.36)), Vector3(0, height + 0.1, 0))
		PROP_SIGN:
			var board := _box(root, Vector3(half_w * 2.0, height, 0.12), Vector3(0, -height * 0.5, 0.1), FighterView.toon(Color(0.96, 0.84, 0.3)))
			board.name = "Board"
			_box(root, Vector3(half_w * 1.6, height * 0.25, 0.14), Vector3(0, -height * 0.5, 0.12), FighterView.toon(Color(0.2, 0.16, 0.24), false))
			for sx in [-1.0, 1.0]:
				var wire := _box(root, Vector3(0.04, 1.2, 0.04), Vector3(sx * half_w * 0.8, 0.6, 0.1), FighterView.toon(Color(0.2, 0.18, 0.28), false))
				wire.name = "Wire"
		_:
			var wood := FighterView.toon(Color(0.58, 0.36, 0.2))
			_box(root, Vector3(half_w * 2.0, height * 0.65, 1.0), Vector3(0, height * 0.325, 0), wood)
			var lid := Node3D.new()
			lid.name = "Lid"
			lid.position = Vector3(0, height * 0.65, -0.5)
			root.add_child(lid)
			_box(lid, Vector3(half_w * 2.0, height * 0.35, 1.0), Vector3(0, height * 0.175, 0.5), FighterView.toon(Color(1.0, 0.78, 0.25)))
			_box(root, Vector3(0.22, 0.26, 0.06), Vector3(0, height * 0.6, 0.52), FighterView.toon(Color(0.3, 0.24, 0.2), false))
	var blast := MeshInstance3D.new()
	blast.name = "Blast"
	var sm := SphereMesh.new()
	sm.radius = 1.0
	sm.height = 2.0
	blast.mesh = sm
	if blast_mat == null:
		blast_mat = StandardMaterial3D.new()
		blast_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		blast_mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		blast_mat.albedo_color = Color(1.0, 0.62, 0.2, 0.55)
	blast.material_override = blast_mat
	blast.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	blast.visible = false
	root.add_child(blast)
	add_child(root)
	return root


## The props where the sim keeps them (`run_props`), and the van (`run_van`). Called every frame.
func sync_props(sim, delta: float) -> void:
	var data: PackedFloat32Array = sim.run_props()
	var seen := {}
	for n in data.size() / 9:
		var o := n * 9
		var key := "%d:%d" % [int(data[o]), int(data[o + 1])]
		seen[key] = true
		var role := int(data[o + 2])
		if not props.has(key):
			props[key] = _prop_node(role, data[o + 7], data[o + 8])
		var node: Node3D = props[key]
		var mode := int(data[o + 5])
		node.position = Vector3(data[o + 3], data[o + 4], Z - 0.6)
		var blast: MeshInstance3D = node.get_node("Blast")
		blast.visible = mode == PROP_BURST
		match role:
			PROP_GAS:
				# Burning: it shakes and flashes; bursting: a ball of fire.
				node.rotation.z = sin(Time.get_ticks_msec() * 0.06) * 0.12 if mode == PROP_LIT else 0.0
				if mode == PROP_BURST:
					blast.position = Vector3(0, 0.6, 0.6)
					blast.scale = Vector3.ONE * lerpf(2.6, 1.2, clampf(data[o + 6] / 4.0, 0.0, 1.0))
				for c in node.get_children():
					if c is MeshInstance3D and c != blast:
						(c as MeshInstance3D).visible = mode != PROP_BURST
			PROP_SIGN:
				node.rotation.z = move_toward(node.rotation.z, 0.5 if mode >= PROP_FALLING else 0.0, delta * 6.0)
				for c in node.get_children():
					if c.name.begins_with("Wire"):
						(c as Node3D).visible = mode < PROP_FALLING
			_:
				var lid: Node3D = node.get_node("Lid")
				lid.rotation.x = move_toward(lid.rotation.x, -1.9 if mode == PROP_OPEN else 0.0, delta * 8.0)
	for key in props.keys():
		if not seen.has(key):
			props[key].queue_free()
			props.erase(key)
	var v: PackedFloat32Array = sim.run_van()
	if v.size() < 8:
		return
	if van == null:
		van = Node3D.new()
		van.name = "Van"
		add_child(van)
		# The city's car, stretched into a delivery van (as the parked van piece), with its roof's light top edge.
		van_body = Node3D.new()
		van.add_child(van_body)
		van_body.scale = Vector3(v[3] * 2.0 / 4.6, v[4] / 2.08, 1.0)
		StageArt._car(van_body, LOOK_COLOURS.get("van", Color(0.9, 0.9, 0.86)), 1.0)
		_top_edge(van, -v[3], v[3], v[4], v[4], LOOK_COLOURS.get("van", Color(0.9, 0.9, 0.86)), 1.9)
	van.visible = int(v[0]) != 0
	van.position = Vector3(v[1], v[2], Z - 1.0)
	# Driving: it bounces a little on its springs.
	van_body.position.y = absf(sin(Time.get_ticks_msec() * 0.03)) * 0.06 if int(v[0]) == 2 else 0.0


## How many props are drawn now (for the tests).
func props_shown() -> int:
	return props.size()
