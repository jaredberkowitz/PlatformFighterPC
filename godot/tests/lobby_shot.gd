extends SceneTree
## Photographs the online lobby over a battle scene: this player's fighter locked in and choosing a stage, the friend ready (needs a
## window: run without --headless).
## Run: Godot --path godot --script res://tests/lobby_shot.gd -- --out=<folder>

const Roster := preload("res://scripts/roster.gd")
const NetLobby := preload("res://ui/net_lobby.gd")


func _initialize() -> void:
	var out := "user://"
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--out="):
			out = a.substr(6)
	Roster.session = {"from_menu": true, "stocks": 3, "time": 0, "skip_countdown": true}
	var main: Node = load("res://main.tscn").instantiate()
	root.add_child(main)
	await create_timer(1.0).timeout
	var entries := Roster.all()
	var lobby: CanvasLayer = NetLobby.new()
	root.add_child(lobby)
	lobby.build(entries, 1, 0)
	lobby.set_friend_name("Sam")
	var theirs: Dictionary = entries[4]
	lobby.show_view({"fighter": Roster.spec_bytes(theirs), "cosmetics": Roster.profile_bytes(theirs), "fighter_locked": true,
		"stage": 2, "stage_locked": true})
	var ev := InputEventKey.new()
	ev.keycode = KEY_ENTER
	ev.pressed = true
	lobby._unhandled_key_input(ev)
	lobby.stage_row.step(1)
	await create_timer(1.5).timeout
	root.get_viewport().get_texture().get_image().save_png(out.path_join("lobby.png"))
	quit()
