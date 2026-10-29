extends SceneTree
## Nothing the stages or the fighters build outlives them: once every stage and a fighter with every weapon are built, shown and freed,
## and the model caches are let go as the game does when it shuts down (music.gd `_exit_tree`), no node is left outside the tree. The
## packaged game (the release export template) crashed on quit with such nodes alive: a weapon model made by `_model_scene` and only
## copied, a prop loaded only to see whether it was there (toonify.gd `has_prop` answers that). Needs a debug build (the orphan list).
## Run: Godot --headless --path godot --script res://tests/orphan_test.gd

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
const StageView := preload("res://scripts/stage_view.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func _initialize() -> void:
	if not ClassDB.class_exists("SimRunner"):
		print("FAIL the game bridge is not loaded")
		quit(1)
		return
	var before := {}
	for id in Node.get_orphan_node_ids():
		before[id] = true
	var sim: Node = ClassDB.instantiate("SimRunner")
	root.add_child(sim)
	var stages := 0
	for i in 32:
		if not sim.use_stage(i):
			break
		var stage := StageView.new()
		root.add_child(stage)
		stage.build(sim)
		await process_frame
		stage.free()
		stages += 1
	check(stages >= 5, "the sim's stages were built: %d" % stages)
	for p in 4:
		var view := FighterView.new()
		root.add_child(view)
		view.build(p, Loadout.default_for(p))
		# (The round shield is made the first time its class is shown.)
		view._place_buckler({"class": FighterView.BUCKLER_CLASS, "char": FighterView.BUCKLER_CLASS, "facing": 1})
		await process_frame
		view.free()
	sim.free()
	FighterView.release_caches()
	var left := {}
	for id in Node.get_orphan_node_ids():
		if before.has(id):
			continue
		var n := instance_from_id(id) as Node
		if n != null and n.get_parent() == null:
			var label := "%s (%s)" % [n.name, n.get_class()]
			left[label] = int(left.get(label, 0)) + 1
	for label in left:
		print("  left over: %d x %s" % [left[label], label])
	check(left.is_empty(), "no node outlives the stages and fighters (%d kinds left over)" % left.size())
	print("orphan_test: ", "FAILED" if failed else "ok")
	quit(1 if failed else 0)
