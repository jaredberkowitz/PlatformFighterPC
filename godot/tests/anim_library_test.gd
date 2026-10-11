extends SceneTree
## The animation library (scripts/anim_library.gd): every clip it names is in the rig, every move of every moveset has a clip of its own
## (none silently falls back), the shared states are the same whatever the moveset, and a multihit move strikes once per hit on the
## move's own frames (fighter_view.gd `strike_progress`).
## Run: Godot --headless --path godot --script res://tests/anim_library_test.gd

const AnimLibrary := preload("res://scripts/anim_library.gd")
const FighterView := preload("res://scripts/fighter_view.gd")
const Roster := preload("res://scripts/roster.gd")

## Every move name the simulation has (sim-core `MoveId::name`), but the script-only slots.
const MOVES := ["jab", "ftilt", "utilt", "dtilt", "dash attack", "fsmash", "usmash", "dsmash", "nair", "fair", "bair", "uair", "dair",
	"neutral special", "side special", "up special", "down special", "jab 2", "jab 3", "grab", "dash grab", "pummel", "forward throw",
	"back throw", "up throw", "down throw", "get-up attack", "pivot grab"]

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func _initialize() -> void:
	_rig_has_every_clip()
	_every_move_mapped()
	_shared_states()
	_multihit()
	print("anim library test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)


func _rig_has_every_clip() -> void:
	var scene: Node = FighterView._model_scene(FighterView.RIG_BASE_PATH)
	check(scene != null, "the base rig loads")
	if scene == null:
		return
	var ap: AnimationPlayer = scene.find_children("*", "AnimationPlayer", true, false)[0]
	for p in AnimLibrary.problems(ap):
		check(false, p)
	# Clips added later have stand-ins for the older rigs, which are clips themselves.
	for c in AnimLibrary.STAND_INS:
		check(ap.has_animation(AnimLibrary.STAND_INS[c]), "stand-in for %s exists" % c)
	scene.free()


func _every_move_mapped() -> void:
	for cls in Roster.CLASS_NAMES.size():
		for m in MOVES:
			var shared: bool = AnimLibrary.SHARED_MOVES.has(m)
			var own: bool = AnimLibrary.CLASS_MOVES.get(cls, {}).has(m)
			check(shared or own, "class %d move '%s' has a clip (not the fallback)" % [cls, m])
	# Overrides are only for moves that exist, and never just repeat the shared clip (that would be a duplicate mapping).
	for cls in AnimLibrary.CLASS_MOVES:
		for m in AnimLibrary.CLASS_MOVES[cls]:
			check(MOVES.has(m), "class %d overrides a real move ('%s')" % [cls, m])
			check(AnimLibrary.CLASS_MOVES[cls][m] != AnimLibrary.SHARED_MOVES.get(m, ""), "class %d's '%s' differs from the shared clip" % [cls, m])
	# The script-only slots take the fallback without a fuss.
	check(AnimLibrary.attack_clip(0, "extra move 3") == AnimLibrary.FALLBACK, "script slots fall back")


func _shared_states() -> void:
	# The shared states name one clip, whatever the moveset (the library has no class in its state table at all), and the new clips stand
	# in for themselves on a rig that has them.
	check(AnimLibrary.state_clip("Landing") == "land", "landing plays land")
	check(AnimLibrary.state_clip("JumpSquat") == "jumpsquat", "the jump's crouch plays jumpsquat")
	check(AnimLibrary.state_clip("Hitstun") == "hurt", "hitstun plays hurt")
	check(AnimLibrary.state_clip("Attack") == "", "attacks are chosen per move")
	var old := AnimationPlayer.new()
	var lib := AnimationLibrary.new()
	lib.add_animation("crouch", Animation.new())
	old.add_animation_library("", lib)
	check(AnimLibrary.state_clip("Landing", old) == "crouch", "a rig without the landing clip plays its stand-in")
	old.free()


func _multihit() -> void:
	var p := FighterView.strike_progress
	# One hit: wind up to the strike on the first active frame, follow through while active, settle by the end.
	var none := PackedInt32Array()
	check(is_equal_approx(p.call(5.0, 30.0, 5.0, 7.0, none), 0.55), "a single hit strikes on its first frame")
	check(is_equal_approx(p.call(7.0, 30.0, 5.0, 7.0, none), 0.63), "and follows through by its last")
	check(is_equal_approx(p.call(30.0, 30.0, 5.0, 7.0, none), 1.0), "and settles by the end")
	# One window is the same as none.
	var one := PackedInt32Array([5, 7])
	for f in range(0, 31):
		check(is_equal_approx(p.call(float(f), 30.0, 5.0, 7.0, one), p.call(float(f), 30.0, 5.0, 7.0, none)), "one window = no windows at %d" % f)
	# Two hits (frames 5-6, then 15-21): a strike on each first frame, drawn back between them, the full follow-through only after the last.
	var two := PackedInt32Array([5, 6, 15, 21])
	check(is_equal_approx(p.call(5.0, 50.0, 5.0, 21.0, two), 0.55), "hit 1 strikes on frame 5")
	check(is_equal_approx(p.call(15.0, 50.0, 5.0, 21.0, two), 0.55), "hit 2 strikes again on frame 15")
	var between: float = p.call(10.0, 50.0, 5.0, 21.0, two)
	check(between < 0.5, "between hits the body draws back (%.2f)" % between)
	check(is_equal_approx(p.call(21.0, 50.0, 5.0, 21.0, two), 0.63), "the last hit follows through by its last frame")
	var settled: float = p.call(50.0, 50.0, 5.0, 21.0, two)
	check(is_equal_approx(settled, 1.0), "and settles by the end")
	# Five hits (the flame kick's rhythm): five strikes, each on its own first frame.
	var five := PackedInt32Array([17, 19, 20, 22, 23, 25, 26, 28, 29, 30])
	var strikes := 0
	var prev := 0.0
	for f in range(0, 56):
		var now: float = p.call(float(f), 55.0, 17.0, 30.0, five)
		if now >= 0.549 and prev < 0.549:
			strikes += 1
		prev = now
	check(strikes == 5, "five hits strike five times (%d)" % strikes)
