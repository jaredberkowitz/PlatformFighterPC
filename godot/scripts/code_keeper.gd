extends Node
## Keeps this player's connection code alive for as long as the game is open (the owner, 19 October 2026: one code per launch, not
## a new one each visit to the online screen). The code's socket (`Roster.netcode`, a bridge `NetCode`) is polled every frame from
## here, whichever screen is showing, so its keepalive holds the router's mapping and the code stays the same. During a match the
## socket is lent to the match and comes back after it (`SimRunner.net_leave_to_code`).

const Roster := preload("res://scripts/roster.gd")


## Adds the keeper to the tree's root once (it survives scene changes, like `PadNav`).
static func attach(owner: Node) -> void:
	if owner.get_tree().root.has_node("CodeKeeper"):
		return
	var n: Node = load("res://scripts/code_keeper.gd").new()
	n.name = "CodeKeeper"
	owner.get_tree().root.add_child.call_deferred(n)


func _process(_delta: float) -> void:
	if Roster.netcode != null:
		Roster.netcode.poll()
