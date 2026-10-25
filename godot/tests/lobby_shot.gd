extends SceneTree
## Photographs the online lobby over a battle scene, the friend locked in on the third stage: the fighter step (lobby_fighters.png),
## then this player's fighter locked in and choosing a stage in both views (lobby.png in the preview view, lobby_full.png in the full
## one). The player's own remembered view is put back afterwards (needs a window: run without --headless).
## Run: Godot --path godot --script res://tests/lobby_shot.gd -- --out=<folder>

const Roster := preload("res://scripts/roster.gd")
const NetLobby := preload("res://ui/net_lobby.gd")

var out := "user://"


func shot(name: String) -> void:
	root.get_viewport().get_texture().get_image().save_png(out.path_join(name))
	print("shot ", name)


func _initialize() -> void:
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
	var view_file := "user://stage_view.txt"
	var had_view := FileAccess.file_exists(view_file)
	var old_view := FileAccess.get_file_as_string(view_file) if had_view else ""
	if had_view:
		DirAccess.remove_absolute(ProjectSettings.globalize_path(view_file))
	Roster.session = {"from_menu": true, "stocks": 3, "time": 0, "skip_countdown": true}
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	await create_timer(1.0).timeout
	var entries := Roster.all()
	var lobby: CanvasLayer = NetLobby.new()
	root.add_child(lobby)
	# As main.gd's _open_lobby does: the match's display hides under the lobby.
	main.hud.visible = false
	lobby.build(entries, 1, 0)
	lobby.set_friend_name("Sam")
	var theirs: Dictionary = entries[4]
	lobby.show_view({"fighter": Roster.spec_bytes(theirs), "cosmetics": Roster.profile_bytes(theirs), "fighter_locked": true,
		"stage": 2, "stage_locked": true})
	await create_timer(1.5).timeout
	shot("lobby_fighters.png")
	var ev := InputEventKey.new()
	ev.keycode = KEY_ENTER
	ev.pressed = true
	lobby._unhandled_key_input(ev)
	lobby._step(1)
	await create_timer(1.0).timeout
	shot("lobby.png")
	ev.keycode = KEY_V
	lobby._unhandled_key_input(ev)
	await create_timer(1.0).timeout
	shot("lobby_full.png")
	if had_view:
		var f := FileAccess.open(view_file, FileAccess.WRITE)
		f.store_string(old_view)
	else:
		DirAccess.remove_absolute(ProjectSettings.globalize_path(view_file))
	quit()
