extends RefCounted
## Characters a player can pick: the two built-in fighters and the ones made in the character creator and saved. Also
## the state the menus hand to the match (`session`), and the assembly of a match's content: the base roster plus any
## created fighters in the match, each built from its recipe (stats) by Rust, so the same recipe is the same fighter.

const Loadout := preload("res://scripts/loadout.gd")
const DIR := "user://characters"
const VERSION := 1

## What the menus pass to the match: {"content_text", "chars": [i, j], "entries": [e1, e2], "from_menu": true}.
static var session := {}
## A character the creator should open when it starts (a slug), or "".
static var edit_slug := ""

const CLASS_NAMES := ["Longsword", "Claws"]
const CLASS_BLURBS := [
	"A long blade: great spacing, a tip that hits harder than the hilt, a sharp counter.",
	"Quick claws and a blaster: fast pressure up close and a flame dash to get away.",
]


static func neutral_entry(name: String, class_id: int, look: RefCounted) -> Dictionary:
	return {"name": name, "slug": slug_of(name), "builtin": false, "class": class_id, "size": 5, "speed": 5, "jump": 5, "weight": 5, "look": look, "casual": false}


## The rules the stats are held to, from Rust (so the limit is defined in one place).
static var _rules: RefCounted


static func rules() -> RefCounted:
	if _rules == null:
		_rules = ClassDB.instantiate("ContentEditor")
		# The project's own blocked words, on top of the built-in trademark list.
		var list_path := content_path("blocklist.txt")
		if FileAccess.file_exists(list_path):
			_rules.policy_load(FileAccess.get_file_as_string(list_path))
	return _rules


static func budget() -> int:
	return rules().recipe_budget()


static func points(e: Dictionary) -> int:
	return rules().recipe_points(e["class"], e.size, e.speed, e.jump, e.weight)


## Within the point budget, so allowed under ranked rules. A fighter saved as casual can still be within it.
static func ranked_legal(e: Dictionary) -> bool:
	return points(e) <= budget()


## The bytes that stand for this fighter in an online handshake (see `FighterSpec` in Rust).
static func spec_bytes(e: Dictionary) -> PackedByteArray:
	if e.get("builtin", false):
		return rules().builtin_spec(int(e.base_index))
	return rules().fighter_spec(e["class"], e.size, e.speed, e.jump, e.weight)


## What is sent as cosmetics: the look and the name. Never reaches the simulation.
static func profile_bytes(e: Dictionary) -> PackedByteArray:
	var look: PackedByteArray = e.look.to_bytes()
	var out := PackedByteArray([look.size()])
	out.append_array(look)
	out.append_array(str(e.name).left(24).to_utf8_buffer())
	return out


## Reads what `profile_bytes` made (or anything else: a bad profile gives the default look and no name).
static func parse_profile(bytes: PackedByteArray, player := 0) -> Dictionary:
	var look: RefCounted = Loadout.default_for(player)
	var name := ""
	if bytes.size() >= 1 and bytes[0] <= Loadout.MAX_BYTES and bytes.size() >= 1 + bytes[0]:
		look = Loadout.from_bytes(bytes.slice(1, 1 + bytes[0]), player)
		name = bytes.slice(1 + bytes[0]).get_string_from_utf8().left(24)
	# A name from another player is made safe to show: odd characters dropped, cut to length, blocked names replaced.
	return {"look": look, "name": rules().policy_clean_name(name, "")}


## How the next match is won, kept between runs: stocks each (0 = unlimited, free play) and a time limit in seconds (0 = none).
const MATCH_PATH := "user://match_rules.json"
const STOCK_CHOICES := [1, 2, 3, 4, 5, 6, 9, 0]
const TIME_CHOICES := [0, 180, 300, 480, 600]
static var match_stocks := 3
static var match_time := 0
## Index of the stage (see `sim_content::stages`; the names come from the bridge).
static var match_stage := 0
static var _stage_names: Array = []
static var _stage_blurbs: Array = []
static var _match_loaded := false


static func load_match_rules() -> void:
	if _match_loaded:
		return
	_match_loaded = true
	if not FileAccess.file_exists(MATCH_PATH):
		return
	var json := JSON.new()
	if json.parse(FileAccess.get_file_as_string(MATCH_PATH)) != OK:
		return
	var parsed = json.data
	if parsed is Dictionary:
		if STOCK_CHOICES.has(int(parsed.get("stocks", 3))):
			match_stocks = int(parsed.get("stocks", 3))
		if TIME_CHOICES.has(int(parsed.get("time", 0))):
			match_time = int(parsed.get("time", 0))
		match_stage = clampi(int(parsed.get("stage", 0)), 0, maxi(0, stage_names().size() - 1))


static func save_match_rules() -> void:
	var f := FileAccess.open(MATCH_PATH, FileAccess.WRITE)
	if f != null:
		f.store_string(JSON.stringify({"stocks": match_stocks, "time": match_time, "stage": match_stage}))


## The stages a match can be played on, from Rust (asked once).
static func stage_names() -> Array:
	if _stage_names.is_empty():
		var sim = ClassDB.instantiate("SimRunner")
		for i in sim.stage_count():
			_stage_names.append(sim.stage_name(i))
			_stage_blurbs.append(sim.stage_blurb(i))
		sim.free()
	return _stage_names


static func stage_blurb(i: int) -> String:
	stage_names()
	return _stage_blurbs[clampi(i, 0, _stage_blurbs.size() - 1)]


static func stocks_text(stocks: int) -> String:
	return "Free play" if stocks == 0 else ("%d stock%s" % [stocks, "" if stocks == 1 else "s"])


static func time_text(seconds: int) -> String:
	return "No limit" if seconds == 0 else "%d:%02d" % [seconds / 60, seconds % 60]


const LAST_PATH := "user://last_fighter.txt"


static func save_last(slug: String) -> void:
	var f := FileAccess.open(LAST_PATH, FileAccess.WRITE)
	if f != null:
		f.store_string(slug)


## The fighter to bring to an online match: `--fighter=<slug>` after `--`, else the one player 1 last played with.
static func net_entry() -> Dictionary:
	# The online screen's choice wins; then `--fighter=<slug>`; then the one played last.
	var wanted := ""
	if session.has("online"):
		return session.online.entry
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--fighter="):
			wanted = slug_of(a.substr(10))
	if wanted == "" and FileAccess.file_exists(LAST_PATH):
		wanted = FileAccess.get_file_as_string(LAST_PATH).strip_edges()
	var everyone := all()
	for e in everyone:
		if e.slug == wanted:
			return e
	return everyone[0]


static func builtins() -> Array:
	var duelist := neutral_entry("Duelist", 0, Loadout.default_for(0))
	duelist.builtin = true
	duelist.base_index = 0
	var brawler := neutral_entry("Brawler", 1, Loadout.default_for(1))
	brawler.builtin = true
	brawler.base_index = 1
	return [duelist, brawler]


static func slug_of(name: String) -> String:
	var out := ""
	for ch in name.strip_edges().to_lower():
		if (ch >= "a" and ch <= "z") or (ch >= "0" and ch <= "9") or ch == "-" or ch == "_":
			out += ch
		elif ch == " ":
			out += "_"
	return out.substr(0, 24)


## A name a creator can save under: 1 to 24 letters, digits, spaces, `_` or `-`, and not one of the built-in fighters.
static func name_problem(name: String, editing_slug := "") -> String:
	var n := name.strip_edges()
	if n == "":
		return "Give your fighter a name."
	if n.length() > 24:
		return "Names can be at most 24 characters."
	for ch in n:
		var ok: bool = (ch >= "a" and ch <= "z") or (ch >= "A" and ch <= "Z") or (ch >= "0" and ch <= "9") or ch == " " or ch == "_" or ch == "-"
		if not ok:
			return "Names can use letters, numbers, spaces, - and _."
	var blocked: String = rules().policy_check_name(n)
	if blocked != "":
		return blocked
	var slug := slug_of(n)
	if slug == "":
		return "Give your fighter a name."
	for b in builtins():
		if b.slug == slug:
			return "That name belongs to a built-in fighter."
	if slug != editing_slug:
		for s in saved():
			if s.slug == slug:
				return "You already have a fighter with that name."
	return ""


static func path_of(slug: String) -> String:
	return "%s/%s.json" % [DIR, slug]


static func to_json(e: Dictionary) -> String:
	return JSON.stringify({
		"version": VERSION, "name": e.name, "class": e["class"], "size": e.size, "speed": e.speed, "jump": e.jump,
		"weight": e.weight, "look": e.look.to_code(), "casual": e.get("casual", false),
	}, "  ")


## Reads a character file's text. Anything wrong with it gives an empty dictionary rather than an error.
static func from_json(text: String) -> Dictionary:
	var json := JSON.new()
	if json.parse(text) != OK or not (json.data is Dictionary):
		return {}
	var parsed: Dictionary = json.data
	var name := str(parsed.get("name", ""))
	if name_problem(name, slug_of(name)) != "" and slug_of(name) == "":
		return {}
	var stat := func(key: String) -> int:
		return clampi(int(parsed.get(key, 5)), 1, 9)
	var e := neutral_entry(name, clampi(int(parsed.get("class", 0)), 0, CLASS_NAMES.size() - 1), Loadout.from_code(str(parsed.get("look", ""))))
	e.size = stat.call("size")
	e.speed = stat.call("speed")
	e.jump = stat.call("jump")
	e.weight = stat.call("weight")
	e.casual = bool(parsed.get("casual", false))
	return e


static func save(e: Dictionary) -> String:
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(DIR))
	var f := FileAccess.open(path_of(e.slug), FileAccess.WRITE)
	if f == null:
		return "Could not save the fighter."
	f.store_string(to_json(e))
	return ""


static func delete(slug: String) -> void:
	var p := ProjectSettings.globalize_path(path_of(slug))
	if FileAccess.file_exists(path_of(slug)):
		DirAccess.remove_absolute(p)


static func saved() -> Array:
	var out := []
	var dir := DirAccess.open(DIR)
	if dir == null:
		return out
	var names := []
	for f in dir.get_files():
		if f.ends_with(".json"):
			names.append(f)
	names.sort()
	for f in names:
		var file := FileAccess.open("%s/%s" % [DIR, f], FileAccess.READ)
		if file == null:
			continue
		var e := from_json(file.get_as_text())
		if not e.is_empty():
			out.append(e)
	return out


static func all() -> Array:
	return builtins() + saved()


static func base_content_path() -> String:
	return content_path("base.pfc")


## Where a file of the shipped content lives: the `content` folder next to the project (a checkout), next to the game's executable (a packaged
## build), or inside the project. The first that exists wins; the checkout path is returned when none does.
static func content_path(file: String) -> String:
	var checkout := ProjectSettings.globalize_path("res://").path_join("../content").path_join(file).simplify_path()
	var candidates := [
		checkout,
		OS.get_executable_path().get_base_dir().path_join("content").path_join(file),
		ProjectSettings.globalize_path("res://content").path_join(file),
	]
	for c in candidates:
		if FileAccess.file_exists(c):
			return c
	return checkout


## Builds the content for a match between `entries` (one per player). Returns {"text", "chars", "error"}: the bundle text to
## load, which fighter index each entry is in it, and an error message (empty if all is well).
static func build_content(entries: Array) -> Dictionary:
	var ed = ClassDB.instantiate("ContentEditor")
	var path := base_content_path()
	if FileAccess.file_exists(path):
		var err: String = ed.open_file(path)
		if err != "":
			return {"error": err, "text": "", "chars": []}
	else:
		ed.new_from_builtin("Match")
	var added := {}
	for e in entries:
		if e.get("builtin", false):
			continue
		if not added.has(e.slug):
			var section: Dictionary = ed.derive_fighter(e.slug, e["class"], e.size, e.speed, e.jump, e.weight)
			var problems: PackedStringArray = ed.put_section(section)
			if not problems.is_empty():
				return {"error": problems[0], "text": "", "chars": []}
			added[e.slug] = true
	var index_of := {}
	var n := 0
	for s in ed.sections():
		if s.kind == "fighter":
			index_of[s.name] = n
			n += 1
	# One fighter index per entry, in the order the entries were given.
	var ordered := []
	for e in entries:
		ordered.append(int(e.base_index) if e.get("builtin", false) else int(index_of[e.slug]))
	return {"text": ed.text(false), "chars": ordered, "error": ""}


## What the creator's stat bars show, normalised against the extremes a recipe can reach: 0 is the least, 1 the most.
static func readout_bars(ed: RefCounted, e: Dictionary) -> Dictionary:
	var r: Dictionary = ed.recipe_readout(e["class"], e.size, e.speed, e.jump, e.weight)
	var lo: Dictionary = ed.recipe_readout(e["class"], 1, 1, 1, 1)
	var hi: Dictionary = ed.recipe_readout(e["class"], 9, 9, 9, 9)
	var bars := {}
	# Run speed and jump are best at small size and high stats: take the extremes from the full range.
	var fast: Dictionary = ed.recipe_readout(e["class"], 1, 9, 9, 1)
	var slow: Dictionary = ed.recipe_readout(e["class"], 9, 1, 1, 9)
	bars["Run speed"] = clampf((r.run_speed - slow.run_speed) / maxf(fast.run_speed - slow.run_speed, 0.0001), 0.0, 1.0)
	bars["Jump height"] = clampf((r.jump_height - slow.jump_height) / maxf(fast.jump_height - slow.jump_height, 0.0001), 0.0, 1.0)
	bars["Weight"] = clampf((r.weight - lo.weight) / maxf(hi.weight - lo.weight, 0.0001), 0.0, 1.0)
	bars["Fall speed"] = clampf((r.fall_speed - lo.fall_speed) / maxf(hi.fall_speed - lo.fall_speed, 0.0001), 0.0, 1.0)
	bars["Size"] = clampf((r.size_percent - 68.0) / 64.0, 0.0, 1.0)
	return bars
