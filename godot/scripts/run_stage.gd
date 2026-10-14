extends Node3D
## The run's streamed stage, drawn (presentation only; the sim streams it, `sim_core::run`): each chunk in play is built once, from the
## stage art's blocks and platforms in the city's look, when it comes into play and freed when it leaves; the city's backdrop rides
## along behind the view. Placeholder until the art director's city kit (see docs/modes/RUNNER_PLAN.md, R1 progress).

const StageArt := preload("res://scripts/stage_art.gd")
const StageView := preload("res://scripts/stage_view.gd")

var theme := {}
## Built chunks by their left edge (a string key): the Node3D holding their pieces.
var pieces := {}
var layout_seen := -1
var backdrop: Node3D


func setup(t: Dictionary) -> void:
	theme = t
	for c in get_children():
		c.queue_free()
	pieces.clear()
	layout_seen = -1
	backdrop = Node3D.new()
	backdrop.name = "Backdrop"
	add_child(backdrop)
	var scenery := Node3D.new()
	backdrop.add_child(scenery)
	# (The city's skyline, following the camera across. Its own stage is given just below the street, so the props that stand on a
	# stage sit hidden behind the run's buildings: the run's chunks are the stage.)
	StageArt.backdrop(scenery, theme, [-15.0, 15.0, -9.0, -24.0, -15.0, 15.0])


## Builds what has come into play and frees what has gone, when the sim's stage has changed (`layout`).
func sync(sim, layout: int) -> void:
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
	for key in pieces.keys():
		if not wanted.has(key):
			pieces[key].queue_free()
			pieces.erase(key)
	for key in wanted.keys():
		if pieces.has(key):
			continue
		var node := Node3D.new()
		node.name = "Chunk"
		add_child(node)
		pieces[key] = node
		var list: Array = wanted[key]
		for r in list:
			var left: float = r[2]
			var right: float = r[3]
			var top: float = r[4]
			var bottom: float = r[5]
			if r[6] > 0.5:
				StageArt.platform(node, theme, left, right, top, StageView.FRONT_Z)
			else:
				StageArt.solid_block(node, theme, left, right, top, bottom, StageView.FRONT_Z, true)


## Keeps the backdrop behind the view.
func follow(view_centre_x: float) -> void:
	if backdrop != null:
		backdrop.position.x = view_centre_x
