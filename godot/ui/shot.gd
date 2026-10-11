extends Node
## Test hook for the menu screens: `--shot=<png>` after `--` saves a screenshot after a short moment and quits.
## `--keys=<names>` first sends those keys (comma separated: DOWN,ENTER,...) so a screen can be photographed after use.

var path := ""
var frames := 0
var keys: Array = []
var wait := 40
var sent := 0
var done := false


static func attach(owner: Node) -> void:
	var shot := ""
	var keys_arg := ""
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--shot="):
			shot = a.substr(7)
		elif a.begins_with("--keys="):
			keys_arg = a.substr(7)
	if shot == "":
		return
	if owner.get_tree().root.has_node("ShotHelper"):
		return
	var n: Node = load("res://ui/shot.gd").new()
	n.name = "ShotHelper"
	n.path = shot
	n.keys = keys_arg.split(",", false)
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--wait="):
			n.wait = int(a.substr(7))
	# On the tree's root, so it survives scene changes (a flow through several screens can be photographed at its end).
	owner.get_tree().root.add_child.call_deferred(n)


func _process(_delta: float) -> void:
	frames += 1
	if frames >= 10 and frames % 3 == 0 and not keys.is_empty():
		var code := OS.find_keycode_from_string(keys.pop_front())
		sent += 1
		for pressed in [true, false]:
			var ev := InputEventKey.new()
			ev.keycode = code
			ev.physical_keycode = code
			ev.pressed = pressed
			Input.parse_input_event(ev)
	# Once every key is sent, a moment for the screen to settle (as long as the keys took), then the picture.
	if not done and keys.is_empty() and frames >= wait + sent * 3:
		done = true
		get_viewport().get_texture().get_image().save_png(path)
		get_tree().quit()
