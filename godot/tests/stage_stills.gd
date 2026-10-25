extends SceneTree
## Renders one of the stage select's pictures: a still of stage N as a match shows it, with the fighters and the display hidden, saved
## as art/ui/stages/stage_<n>.webp (1280x720). One stage per run (a second match scene in the same run keeps the first one's scenery).
## Run it for every stage after a stage's art changes (needs a window, so not --headless), then import:
## Run: for n in 0 1 2 3 4; do Godot --path godot --resolution 1280x720 --script res://tests/stage_stills.gd -- --stage=$n; done
##      Godot --headless --path godot --import

const Roster := preload("res://scripts/roster.gd")
const OUT := "res://art/ui/stages"


func _initialize() -> void:
	var n := 0
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--stage="):
			n = int(a.substr(8))
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(OUT))
	Roster.session = {"from_menu": true, "stocks": 3, "time": 0, "skip_countdown": true, "stage": n}
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	await create_timer(2.5).timeout
	# Frozen first: the match shows its fighters again every frame.
	main.process_mode = Node.PROCESS_MODE_DISABLED
	if main.hud != null:
		main.hud.visible = false
	for v in main.views:
		v.visible = false
	await create_timer(0.5).timeout
	var img := root.get_viewport().get_texture().get_image()
	img.convert(Image.FORMAT_RGB8)
	var path := ProjectSettings.globalize_path(OUT.path_join("stage_%d.webp" % n))
	img.save_webp(path, true, 0.86)
	print("saved ", path)
	quit()
