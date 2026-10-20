extends RefCounted
## Characters a player can pick: the two built-in fighters and the ones made in the character creator and saved. Also
## the state the menus hand to the match (`session`), and the assembly of a match's content: the base roster plus any
## created fighters in the match, each built from its recipe (stats) by Rust, so the same recipe is the same fighter.

const Loadout := preload("res://scripts/loadout.gd")
const DIR := "user://characters"
const VERSION := 1

## What the menus pass to the match: {"content_text", "chars": [i, j], "entries": [e1, e2], "from_menu": true}.
static var session := {}
## This player's connection-code socket (a bridge `NetCode`), opened the first time the online screen offers a code and kept for as
## long as the game is open, so the code stays the same (`scripts/code_keeper.gd` polls it). Lent to a match while one is on.
static var netcode: RefCounted = null
## A character the creator should open when it starts (a slug), or "".
static var edit_slug := ""

const CLASS_NAMES := ["Longsword", "Claws", "Needles", "Embers", "Gauntlets", "Staff", "Greatsword", "Bracers", "Clutch", "Buckler", "Pockets", "Popgun", "Dagger", "Nodachi"]
const CLASS_BLURBS := [
	"A long blade: great spacing, a tip that hits harder than the hilt, a sharp counter.",
	"Quick claws and a blaster: fast pressure up close and a flame dash to get away.",
	"Quick kicks and hidden needles: fast, light and slippery, strings hits together and drags foes along with a flurry of kicks.",
	"Fast fists and feet: an all-rounder with high jumps, up-air juggles, a spinning down air and a meteor overhand.",
	"Blazing speed and heavy hands: a dashing bruiser with a one-frame flying knee, a stomping meteor and a jab flurry.",
	"Light from a distance: a tall caster with a staff who sets off blasts where it points, locks on and fires, blinks across the stage and answers a strike or a shot in kind.",
	"A huge blade and a rising boost: long reach, a slow wave that crosses the stage, a combo slash that only carries on when it lands, and a boost that turns every special into a finisher.",
	"Huge arms and a heavy frame: a wind-up punch you can save for later, a headbutt that plants foes in the ground, a spinning clothesline, ground slaps, and a throw that carries the catch anywhere, even off the stage.",
	"Light on its feet and hard to launch, with a towering second jump that shrugs off blows: a long tongue that seals foes in shells, a rolling shell, lobbed shells that lift it higher, a crashing drop that scatters stars, and a flurry of kicks.",
	"A sword, a shield and a pack of tools: a blade with a long reach, a shield that stops shots while standing, a bow that leaves arrows to pick up, a boomerang that comes back, a bomb set off from afar, and a whirling spin that lifts it home.",
	"Pockets full of tools: quick punches that pop foes up for more, a slingshot both ways, turnips, a bowling ball, a rocket to fire or to ride, and balloons that fly it home until they pop.",
	"Quick hands and a peel to slip on: fast slaps and flips, a popgun that pops if held too long, a banana anyone can grab and throw, a leap that latches onto a face, and rocket barrels that fly off when hit.",
	"A short knife and a gun, and a partner that comes when the gauge fills: quick slashes and kicks, shots that dodge and leap, a bolt that curses, a guard that soaks up hits to call the partner sooner, and with it every strike lands harder.",
	"A blade of enormous length: slow, sweeping cuts that reach across the stage, a flare charged from a spark to a blast, a dash that slashes eight times, and a single wing that grows when the fight turns against it, bringing speed, power and a third jump.",
]


static func neutral_entry(name: String, class_id: int, look: RefCounted) -> Dictionary:
	return {"name": name, "slug": slug_of(name), "builtin": false, "class": class_id, "size": 5, "speed": 5, "jump": 5, "weight": 5, "look": look, "casual": false}


## The rules the stats are held to, from Rust (so the limit is defined in one place).
static var _rules: RefCounted


## Lets go of the Rust-backed helper (the game does this as it shuts down, before the extension unloads).
static func release() -> void:
	_rules = null


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
## The default is the reference's standard 1v1: 3 stocks and 7 minutes, a tie on stocks at time going to sudden death (owner,
## 14 October 2026). A rules file saved before then (no "v") is moved to the 7-minute default once.
const MATCH_PATH := "user://match_rules.json"
const STOCK_CHOICES := [1, 2, 3, 4, 5, 6, 9, 0]
const TIME_CHOICES := [0, 180, 300, 420, 480, 600]
const DEFAULT_TIME := 420
const RULES_VERSION := 2
static var match_stocks := 3
static var match_time := DEFAULT_TIME
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
		if int(parsed.get("v", 1)) < RULES_VERSION:
			match_time = DEFAULT_TIME
		elif TIME_CHOICES.has(int(parsed.get("time", DEFAULT_TIME))):
			match_time = int(parsed.get("time", DEFAULT_TIME))
		match_stage = clampi(int(parsed.get("stage", 0)), 0, maxi(0, stage_names().size() - 1))


static func save_match_rules() -> void:
	var f := FileAccess.open(MATCH_PATH, FileAccess.WRITE)
	if f != null:
		f.store_string(JSON.stringify({"v": RULES_VERSION, "stocks": match_stocks, "time": match_time, "stage": match_stage}))


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
	# (Each built-in fighter has a silhouette of its own, not just a colour: the Duelist a plumed helm and a cape, the Brawler its aviator
	# cap and a streaming scarf, the Rogue a hood.)
	var knight: RefCounted = Loadout.default_for(0)
	knight.hat = 7
	knight.neck = 4
	knight.shirt = 2
	knight.gear = 1
	# (Each fighter's outfit in its own colour, art review 10: ten of the fourteen wore the same blue shirt and read alike at play size.)
	knight.accent = 6
	var duelist := neutral_entry("Duelist", 0, knight)
	duelist.builtin = true
	duelist.base_index = 0
	var scrapper: RefCounted = Loadout.default_for(1)
	scrapper.neck = 3
	scrapper.shirt = 3
	scrapper.gear = 2
	scrapper.accent = 2
	var brawler := neutral_entry("Brawler", 1, scrapper)
	brawler.builtin = true
	brawler.base_index = 1
	var hooded: RefCounted = Loadout.default_for(2)
	hooded.hat = 6
	hooded.glasses = 0
	hooded.shirt = 2
	hooded.gear = 1
	hooded.accent = 11
	var rogue := neutral_entry("Rogue", 2, hooded)
	rogue.builtin = true
	rogue.base_index = 2
	var plumber: RefCounted = Loadout.default_for(3)
	plumber.hat = 1
	plumber.shirt = 1
	plumber.gear = 3
	var tumbler := neutral_entry("Tumbler", 3, plumber)
	tumbler.builtin = true
	tumbler.base_index = 3
	# (Its own look, not player one's: a cherry racer in an aviator cap and goggles, a scarf to trail behind it, gloves for the fists.)
	var racer: RefCounted = Loadout.default_for(4)
	racer.color = 6
	racer.hat = 2
	racer.glasses = 2
	racer.neck = 3
	racer.gear = 2
	racer.shirt = 2
	racer.accent = 0
	var sprinter := neutral_entry("Sprinter", 4, racer)
	sprinter.builtin = true
	sprinter.base_index = 4
	# (Its own look: a plum seer in a crown with a sash.)
	var oracle: RefCounted = Loadout.default_for(5)
	oracle.color = 5
	oracle.hat = 5
	oracle.glasses = 0
	oracle.neck = 4
	oracle.gear = 1
	oracle.shirt = 2
	oracle.accent = 8
	var seer := neutral_entry("Seer", 5, oracle)
	seer.builtin = true
	seer.base_index = 5
	# (Its own look: a stern slate swordsman, bareheaded, a scarf to stream behind it, the full kit for the heavy blade.)
	var hero: RefCounted = Loadout.default_for(6)
	hero.color = 9
	hero.hat = 8
	hero.brow_color = 4
	hero.glasses = 0
	hero.neck = 3
	hero.gear = 3
	hero.shirt = 2
	hero.face = 2
	hero.accent = 4
	var champion := neutral_entry("Champion", 6, hero)
	champion.builtin = true
	champion.base_index = 6
	# (Its own look: a big biscuit-brown bruiser, bareheaded, a neckerchief, shorts and gloves for the fists.)
	var bruiser: RefCounted = Loadout.default_for(7)
	bruiser.color = 10
	bruiser.hat = 9
	bruiser.glasses = 0
	bruiser.neck = 2
	bruiser.gear = 2
	bruiser.face = 2
	var hauler := neutral_entry("Hauler", 7, bruiser)
	hauler.builtin = true
	hauler.base_index = 7
	# (Its own look: a meadow-green hopper in a beanie with a neckerchief, gloves and boots for its kicks.)
	var hopper: RefCounted = Loadout.default_for(8)
	hopper.color = 4
	hopper.hat = 4
	hopper.glasses = 0
	hopper.neck = 2
	hopper.gear = 3
	var bounder := neutral_entry("Bounder", 8, hopper)
	bounder.builtin = true
	bounder.base_index = 8
	# (Its own look: a lagoon-blue traveller in a straw hat and a streaming scarf, sword in hand and a round shield on its arm.)
	var traveller: RefCounted = Loadout.default_for(9)
	traveller.color = 7
	traveller.hat = 3
	traveller.glasses = 0
	traveller.neck = 3
	traveller.gear = 3
	traveller.shirt = 2
	traveller.accent = 10
	var wanderer := neutral_entry("Wanderer", 9, traveller)
	wanderer.builtin = true
	wanderer.base_index = 9
	# (The shared default look until the art agent gives it its own.)
	var villager: RefCounted = Loadout.default_for(10)
	villager.hat = 3
	villager.shirt = 4
	villager.gear = 1
	villager.accent = 1
	var settler := neutral_entry("Settler", 10, villager)
	settler.builtin = true
	settler.base_index = 10
	# (The shared default look until the art agent gives it its own.)
	var monkey: RefCounted = Loadout.default_for(11)
	monkey.hat = 2
	monkey.shirt = 3
	monkey.gear = 1
	monkey.accent = 7
	var scamp := neutral_entry("Scamp", 11, monkey)
	scamp.builtin = true
	scamp.base_index = 11
	# The Phantom: a gentleman thief at night: a midnight body, shades, a cape and the full kit (art review 6).
	var thief: RefCounted = Loadout.default_for(12)
	thief.color = 11
	thief.accent = 6
	thief.hat = 0
	thief.glasses = 1
	thief.neck = 4
	thief.shirt = 2
	thief.gear = 3
	var phantom := neutral_entry("Phantom", 12, thief)
	phantom.builtin = true
	phantom.base_index = 12
	# The Exile: a pale wanderer with a long scarf, dark clothes and the full kit (art review 6).
	var wanderer_look: RefCounted = Loadout.default_for(13)
	wanderer_look.color = 8
	wanderer_look.accent = 9
	wanderer_look.hat = 0
	wanderer_look.neck = 3
	wanderer_look.shirt = 2
	wanderer_look.gear = 3
	var exile := neutral_entry("Exile", 13, wanderer_look)
	exile.builtin = true
	exile.base_index = 13
	# (The built-in fighters keep their belts: the outfit pass took them off the default looks only.)
	for l in [knight, scrapper, hooded, plumber, racer, oracle, hero, bruiser, hopper, traveller, villager, monkey, thief, wanderer_look]:
		l.belt = 0
	return [duelist, brawler, rogue, tumbler, sprinter, seer, champion, hauler, bounder, wanderer, settler, scamp, phantom, exile]


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


## The most fighters a player can save (the select screen fits them all; the creator stops offering a new one at this many).
const MAX_FIGHTERS := 100


## Saves `e`; returns "" or what went wrong. A fighter that is new (no file under its name, and not a rename of `replacing`, an existing
## saved fighter) is refused once MAX_FIGHTERS are saved.
static func save(e: Dictionary, replacing := "") -> String:
	var adds := not FileAccess.file_exists(path_of(e.slug)) and (replacing == "" or not FileAccess.file_exists(path_of(replacing)))
	if adds and saved().size() >= MAX_FIGHTERS:
		return "The roster is full (%d fighters): delete one to make room." % MAX_FIGHTERS
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
