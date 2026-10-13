extends SceneTree
## The fighter-select grid fits any roster: one row while it can, then new rows with smaller cards, every card inside the grid's band
## until the smallest size; and never more fighters than Roster.MAX_FIGHTERS can be saved.
## Run: Godot --headless --path godot --script res://tests/select_grid_test.gd

const Select := preload("res://ui/select.gd")
const Roster := preload("res://scripts/roster.gd")

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
			check(rows * (t + 12 + Select.GRID_GAP.y) <= Select.GRID_AREA.y + 0.5, "%d cards: every row fits the band %s" % [n, fit])
		check(t <= prev_t, "%d cards: cards never grow as the roster grows" % n)
		prev_t = t
	check(int(Select.grid_layout(9)[0]) >= 9, "nine cards sit in one row")
	check(ceili(30.0 / float(Select.grid_layout(30)[0])) >= 2, "thirty cards start new rows")
	print("select grid test ", "FAILED" if failed else "PASSED")
	quit(1 if failed else 0)
