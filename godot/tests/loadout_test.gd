extends SceneTree
## Loadouts: a compact byte format with safe fallbacks, a polygon/mesh budget for every catalog item, and the handshake
## delivering each player's look to the other (over real UDP through the bridge).
## Run: Godot --headless --path godot --script res://tests/loadout_test.gd

const Loadout := preload("res://scripts/loadout.gd")
const FighterView := preload("res://scripts/fighter_view.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func _initialize() -> void:
	_format()
	_budget()
	_net()
	print("loadout test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


func _format() -> void:
	# Every value of every slot round-trips, and a loadout is within the byte budget.
	var l := Loadout.default_for(0)
	for slot in Loadout.SLOTS:
		for v in Loadout.slot_size(slot):
			l.set_slot(slot, v)
			var back := Loadout.from_bytes(l.to_bytes())
			check(back.equals(l), "round trip of %s=%d" % [slot, v])
	check(l.to_bytes().size() <= Loadout.MAX_BYTES, "within the byte budget")
	var rng := RandomNumberGenerator.new()
	rng.seed = 7
	for i in 300:
		var r := Loadout.default_for(0)
		for slot in Loadout.SLOTS:
			r.set_slot(slot, rng.randi_range(0, Loadout.slot_size(slot) - 1))
		check(Loadout.from_code(r.to_code()).equals(r), "share code round trip")

	# Bad input never fails: it gives a valid look.
	var default := Loadout.default_for(1)
	check(Loadout.from_bytes(PackedByteArray(), 1).equals(default), "empty bytes give the default")
	check(Loadout.from_bytes(PackedByteArray([99, 1, 1, 1, 1, 1, 1]), 1).equals(default), "unknown version gives the default")
	var long := PackedByteArray()
	long.resize(Loadout.MAX_BYTES + 1)
	long[0] = Loadout.VERSION
	check(Loadout.from_bytes(long, 1).equals(default), "too long gives the default")
	var newer := Loadout.from_bytes(PackedByteArray([Loadout.VERSION, 200, 200, 200, 200, 200, 200]), 1)
	check(newer.equals(default), "unknown catalog entries fall back slot by slot")
	var partial := Loadout.from_bytes(PackedByteArray([Loadout.VERSION, 5, 99]), 1)
	check(partial.color == 5 and partial.face == default.face, "a known slot is kept, an unknown one falls back")
	check(Loadout.from_code("not hex!", 0).equals(Loadout.default_for(0)), "a bad code gives the default")
	for i in 1000:
		var junk := PackedByteArray()
		junk.resize(rng.randi_range(0, 24))
		for j in junk.size():
			junk[j] = rng.randi_range(0, 255)
		var got := Loadout.from_bytes(junk, 0)
		check(got.to_bytes().size() == Loadout.SLOTS.size() + 1, "junk gives a well-formed loadout")


func _count_meshes(l: RefCounted) -> int:
	var v := FighterView.new()
	root.add_child(v)
	v.build(0, l)
	var n := v.meshes.size()
	v.free()
	return n


func _budget() -> void:
	# Plan 7.3: hard budgets for accessories. Each catalog item adds a handful of meshes at most, and rebuilding does
	# not leak nodes.
	var base := Loadout.default_for(0)
	base.hat = 0
	base.glasses = 0
	base.neck = 0
	var base_count := _count_meshes(base)
	for slot in ["hat", "glasses", "neck"]:
		for v in range(1, Loadout.slot_size(slot)):
			var l := Loadout.default_for(0)
			l.hat = 0
			l.glasses = 0
			l.neck = 0
			l.set_slot(slot, v)
			var added := _count_meshes(l) - base_count
			check(added >= 1 and added <= 12, "%s %d adds %d meshes (budget 1 to 12)" % [slot, v, added])
	var everything := Loadout.default_for(0)
	everything.hat = 5
	everything.glasses = 2
	everything.neck = 3
	check(_count_meshes(everything) <= base_count + 30, "a fully dressed fighter stays in budget")

	var v := FighterView.new()
	root.add_child(v)
	v.build(0, base)
	var children := v.get_child_count()
	for i in 20:
		v.rebuild(everything)
		v.rebuild(base)
	check(v.get_child_count() == children, "rebuilding does not leak nodes")
	check(v.meshes.size() == base_count, "rebuilding does not leak meshes")
	# Every face, the hurt face and the blink have a drawing that loads onto the face shell.
	check(v.face_mesh != null and v.face_mat != null, "the rig has a face shell")
	for f in Loadout.FACES + [Loadout.HURT]:
		v.set_expression(f)
		check(v.face_mat.get_shader_parameter("face_tex") is Texture2D, "face drawing %s loads" % f.name)
	v.blink_left = 0.1
	v.set_expression(Loadout.FACES[0])
	v._show_face()
	check(v.shown_face == "blink" and v.face_mat.get_shader_parameter("face_tex") is Texture2D, "the blink drawing loads")
	# Every shirt dresses the body (prints load) and has sleeves; no shirt has none.
	for s in Loadout.SHIRTS.size():
		var l := Loadout.default_for(0)
		l.shirt = s
		v.rebuild(l)
		# Sleeves are a band painted on the arms (shaders/limb.gdshader): there with a shirt, not without.
		var arms := v.meshes.filter(func(m): return str(m.name).begins_with("Arm"))
		var banded := arms.all(func(m):
			var end = m.material_override.get_shader_parameter("top_end")
			return end != null and float(end) > 0.0)
		check(arms.size() == 2 and banded == (s != 0), "shirt %s sleeves" % Loadout.SHIRTS[s])
		var mat = v._shirt_material()
		check((mat == null) == (s == 0), "shirt %s material" % Loadout.SHIRTS[s])
		if s >= 3:
			check(mat.get_shader_parameter("albedo_tex") is Texture2D, "shirt %s print loads" % Loadout.SHIRTS[s])
	v.free()


func _net() -> void:
	if not ClassDB.class_exists("SimRunner"):
		check(false, "bridge loaded")
		return
	var a = ClassDB.instantiate("SimRunner")
	var b = ClassDB.instantiate("SimRunner")
	root.add_child(a)
	root.add_child(b)
	var la := Loadout.default_for(0)
	la.hat = 5
	la.color = 6
	var lb := Loadout.default_for(1)
	lb.glasses = 3
	lb.neck = 3
	a.set_cosmetics(la.to_bytes())
	b.set_cosmetics(lb.to_bytes())
	check(a.net_host(47131, PackedInt32Array([0, 1, 0, 1]), 2) == "", "host starts")
	check(b.net_join("127.0.0.1:47131") == "", "joiner starts")
	var running := [false, false]
	for t in 600:
		var sa: int = a.net_update(0, 0, 0)
		var sb: int = b.net_update(0, 0, 0)
		running[0] = running[0] or sa == 1
		running[1] = running[1] or sb == 1
		if running[0] and running[1] and a.net_their_cosmetics().size() > 0 and b.net_their_cosmetics().size() > 0:
			break
		OS.delay_msec(2)
	check(Loadout.from_bytes(a.net_their_cosmetics(), 1).equals(lb), "the host receives the joiner's look")
	check(Loadout.from_bytes(b.net_their_cosmetics(), 0).equals(la), "the joiner receives the host's look")
	# The look is cosmetic: both sides still agree on the game.
	check(a.checksum() == b.checksum() or a.frame() != b.frame(), "the checksum does not depend on looks")
