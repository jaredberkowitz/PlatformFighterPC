extends SceneTree
## Loadouts: a compact byte format with safe fallbacks, a polygon/mesh budget for every catalog item, and the handshake
## delivering each player's look to the other (over real UDP through the bridge).
## Run: Godot --headless --path godot --script res://tests/loadout_test.gd

const Loadout := preload("res://scripts/loadout.gd")
const FighterView := preload("res://scripts/fighter_view.gd")
const FaceLook := preload("res://scripts/face_look.gd")

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
	# (A look this short was saved before the gear slot: it keeps the full kit it wore then.)
	var dressed := Loadout.default_for(1)
	dressed.gear = Loadout.GEAR_FULL
	check(newer.equals(dressed), "unknown catalog entries fall back slot by slot")
	# New players start as the concept's bare body; a look saved before the gear slot keeps its kit.
	check(default.gear == 0 and default.shirt == 0 and default.neck == 0, "the default look is the bare body")
	var old_look: PackedByteArray = default.to_bytes().slice(0, Loadout.SLOTS.find("gear") + 1)
	check(Loadout.from_bytes(old_look, 1).gear == Loadout.GEAR_FULL, "an old look keeps its gear")
	check(Loadout.from_bytes(default.to_bytes(), 1).gear == 0, "a new bare look stays bare")
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
	# The face is built from parts: every part of every catalog has its drawings, and every expression (moods, the action faces,
	# the blink) puts the eyes and mouth on the face shell, shut eyes where the expression shuts them.
	check(v.face_mesh != null and v.face_mat != null, "the rig has a face shell")
	for expr in FaceLook.EXPRESSIONS:
		v.set_expression({"name": expr})
		var ok: bool = v.face_mat.get_shader_parameter("eye_fill_tex") is Texture2D and v.face_mat.get_shader_parameter("mouth_tex") is Texture2D
		if FaceLook.SHUT.has(expr):
			ok = ok and v.face_mat.get_shader_parameter("eye_extra_tex") is Texture2D
		check(ok, "expression %s has its drawings" % expr)
	v.blink_left = 0.1
	v.set_expression(Loadout.FACES[0])
	v._show_face()
	check(v.shown_face == "blink" and v.face_mat.get_shader_parameter("eye_mode") == 1, "the blink shuts the eyes")
	for slot in ["eyes", "brows", "nose", "mouth", "cheeks"]:
		for i in Loadout.slot_size(slot):
			var l := Loadout.default_for(0)
			l.set_slot(slot, i)
			var mat := ShaderMaterial.new()
			mat.shader = preload("res://shaders/face.gdshader")
			FaceLook.apply(mat, l)
			var tex_name: String = {"eyes": "eye_fill_tex", "brows": "brow_tex", "nose": "nose_tex", "mouth": "mouth_tex", "cheeks": "cheek_tex"}[slot]
			var none: bool = Loadout.slot_names(slot)[i] == "none"
			check((mat.get_shader_parameter(tex_name) == null) == none, "%s %s has its drawing" % [slot, Loadout.slot_names(slot)[i]])
	# Old looks (before the face had parts: seven slots) still read, with the standard face.
	var old := Loadout.from_bytes(PackedByteArray([Loadout.VERSION, 3, 1, 2, 1, 2, 4, 3]), 0)
	check(old.color == 3 and old.face == 1 and old.eyes == 0 and old.cheeks == 1 and old.eye_y == FaceLook.neutral("eye_y"), "an old look keeps the standard face")
	# Glasses follow the eyes.
	var spread := Loadout.default_for(0)
	spread.eye_gap = Loadout.slot_size("eye_gap") - 1
	check(FaceLook.eye_out(spread) > FaceLook.eye_out(Loadout.default_for(0)), "eyes set apart move the glasses' lenses apart")
	# Every shirt dresses the body (prints load) and has sleeves; no shirt has none.
	for s in Loadout.SHIRTS.size():
		var l := Loadout.default_for(0)
		l.shirt = s
		v.rebuild(l)
		if FighterView.use_base_body:
			# The base body wears the shirt painted on its skin (shaders/skin.gdshader), sleeves and print included.
			var skins := v.meshes.filter(func(m): return str(m.name).begins_with("Skin"))
			var skin_mat: ShaderMaterial = skins[0].material_override if not skins.is_empty() else null
			var worn: bool = skin_mat != null and (skin_mat.get_shader_parameter("shirt") as Color).a > 0.5
			check(skins.size() == 1 and worn == (s != 0), "shirt %s on the skin" % Loadout.SHIRTS[s])
			if s >= 3:
				check(skin_mat.get_shader_parameter("shirt_tex") is Texture2D, "shirt %s print on the skin" % Loadout.SHIRTS[s])
		else:
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
