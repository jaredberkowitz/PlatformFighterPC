extends SceneTree
## Which state clips put a hand over the drawn face at the match camera: for each, the share of sampled frames in which a hand is in
## front of the head (nearer the camera than the head's middle) and within the face's radius of its middle on screen. Classes 0 and 1,
## facing right (the clips are mirrored facing left). Run: Godot --headless --path godot --script res://tests/face_cover_probe.gd

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
const STATES := [["Idle", 0.0, 0.0, 0], ["Walk", 0.12, 0.0, 0], ["Run", 0.24, 0.0, 0], ["Dash", 0.27, 0.0, 0], ["JumpSquat", 0.0, 0.0, 0],
	["Airborne", 0.0, 0.32, -1], ["Airborne", 0.0, 0.12, -1], ["Airborne", 0.0, -0.05, -1], ["Airborne", 0.0, -0.12, -1],
	["Airborne", 0.0, -0.25, -1], ["Landing", 0.0, 0.0, 0], ["Shield", 0.0, 0.0, 0], ["Crouch", 0.0, 0.0, 0], ["Hitstun", 0.0, 0.0, 0],
	["LedgeHang", 0.0, 0.0, 0], ["Turn", 0.0, 0.0, 0], ["Helpless", 0.0, -0.1, -1], ["ItemPickup", 0.0, 0.0, 0], ["ItemThrow", 0.0, 0.0, 0],
	["ShieldBreak", 0.0, 0.0, 0], ["Tether", 0.0, 0.0, -1]]
const FACE_R := 0.4


func snap(cls: int, st: Array, frame: int) -> Dictionary:
	return {"state": st[0], "facing": 1, "vel": Vector2(st[1], st[2]), "kb_vel": Vector2.ZERO, "platform": st[3], "fast_fall": false,
		"invuln": 0, "ledge_invuln": 0, "frame": frame, "shield": 1.0, "percent": 0.0, "move_name": "", "move_tip": Vector3.ZERO,
		"move_timing": PackedInt32Array([0, 0, 0]), "charge": 0, "hitlag": 0, "tumble": false, "launch_pending": false, "char": cls,
		"class": cls, "reach": 3.0, "state_frame": frame, "jumps": 1, "stocks": 3, "dodge": PackedInt32Array(), "dash_length": 2.4,
		"lag": 8}


func _initialize() -> void:
	for cls in [0, 1, 2, 3, 8, 10]:
		var line := "COVER class %d:" % cls
		for st in STATES:
			var view := FighterView.new()
			root.add_child(view)
			view.build(cls, Loadout.default_for(cls))
			await process_frame
			var sk: Skeleton3D = view.skeleton
			var covered := 0
			var n := 0
			for fr in 60:
				view.apply(Vector3.ZERO, snap(cls, st, fr + 1), 1.0 / 60.0)
				if fr < 10:
					continue
				var hb := sk.find_bone("head")
				var hp: Transform3D = sk.global_transform * sk.get_bone_global_pose(hb)
				var centre: Vector3 = hp.origin + hp.basis.y.normalized() * 0.3
				var hit := false
				for side in ["L", "R"]:
					var h: Vector3 = sk.global_transform * sk.get_bone_global_pose(sk.find_bone("hand." + side)).origin
					if h.z > centre.z and Vector2(h.x - centre.x, h.y - centre.y).length() < FACE_R:
						hit = true
				if view.blade_pivot.visible:
					var tipish: Vector3 = view.blade_pivot.global_position
					if tipish.z > centre.z and Vector2(tipish.x - centre.x, tipish.y - centre.y).length() < FACE_R:
						hit = true
				covered += 1 if hit else 0
				n += 1
			line += " %s%s %d%%" % [st[0], ("(%.2f)" % st[2]) if st[0] == "Airborne" else "", 100 * covered / n]
			view.queue_free()
		print(line)
	quit(0)
