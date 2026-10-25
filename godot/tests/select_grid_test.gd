extends SceneTree
## The fighter-select grid fits any roster: one row while it can, then new rows with smaller cards, every card inside the grid's band
## until the smallest size; and never more fighters than Roster.MAX_FIGHTERS can be saved.
## Run: Godot --headless --path godot --script res://tests/select_grid_test.gd

const Select := preload("res://ui/select.gd")
const Roster := preload("res://scripts/roster.gd")
const Loadout := preload("res://scripts/loadout.gd")

var failed := false


func check(ok: bool, what: String) -> void:
	if not ok:
		print("FAIL ", what)
		failed = true


func _initialize() -> void:
	var prev_t := 1000
	for n in range(1, Roster.MAX_FIGHTERS + 8):
		var fit: Array = Select.grid_layout(n)
		var cols: int = fit[0]
		var t: int = fit[1]
		var rows := ceili(float(n) / float(cols))
		check(cols >= 1 and t >= Select.TOKEN_MIN and t <= Select.TOKEN_MAX, "%d cards: a sane layout %s" % [n, fit])
		check(cols * (t + Select.GRID_GAP.x) <= Select.GRID_AREA.x, "%d cards: the row fits the width %s" % [n, fit])
		if t > Select.TOKEN_MIN:
			check(rows * (Select.card_height(t) + 12 + Select.GRID_GAP.y) <= Select.GRID_AREA.y + 0.5, "%d cards: every row fits the band %s" % [n, fit])
		check(t <= prev_t, "%d cards: cards never grow as the roster grows" % n)
		prev_t = t
	check(int(Select.grid_layout(9)[0]) >= 9, "nine cards sit in one row")
	check(ceili(30.0 / float(Select.grid_layout(30)[0])) >= 2, "thirty cards start new rows")
	# The roster refuses a fighter past the cap (a rename of a saved one still saves). Test fighters are named zz_cap and removed after.
	var made: Array = []
	var start := Roster.saved().size()
	for k in maxi(0, Roster.MAX_FIGHTERS - start):
		var e := Roster.neutral_entry("zz cap %d" % k, 0, Loadout.default_for(0))
		if Roster.save(e) == "":
			made.append(e.slug)
	check(Roster.saved().size() == Roster.MAX_FIGHTERS, "the roster fills to the cap (%d)" % Roster.saved().size())
	var extra := Roster.neutral_entry("zz cap extra", 0, Loadout.default_for(0))
	check(Roster.save(extra) != "", "a fighter past the cap is refused")
	if not made.is_empty():
		var renamed := Roster.neutral_entry("zz cap renamed", 0, Loadout.default_for(0))
		check(Roster.save(renamed, made[0]) == "", "renaming a saved fighter at the cap still saves")
		made.append(renamed.slug)
	for slug in made + [extra.slug]:
		Roster.delete(slug)
	check(Roster.saved().size() == start, "the test leaves the roster as it found it")
	print("select grid test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
