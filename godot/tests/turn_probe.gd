extends SceneTree
## Prints how far the hips, chest and face point away from the camera (degrees about the vertical; 0 faces the camera, 90 is side-on
## toward the facing direction) in the idle and the run, for the sword and claws classes, both facings. For judging the fighters' turn
## against the camera. Run: Godot --headless --path godot --script res://tests/turn_probe.gd

const FighterView := preload("res://scripts/fighter_view.gd")
const Loadout := preload("res://scripts/loadout.gd")
## A face turned further than this from the camera (degrees) loses its far eye round the head's edge (judged in the turn mocks).
const EYES_BOTH_AT := 47.0


func snap(cls: int, state: String, frame: int, facing: int, vx: float) -> Dictionary:
	return {"state": state, "facing": facing, "vel": Vector2(vx, 0), "kb_vel": Vector2.ZERO, "platform": 0, "fast_fall": false,
		"invuln": 0, "ledge_invuln": 0, "frame": frame, "shield": 1.0, "percent": 0.0, "move_name": "", "move_tip": Vector3.ZERO,
		"move_timing": PackedInt32Array([0, 0, 0]), "charge": 0, "hitlag": 0, "tumble": false, "launch_pending": false, "char": cls,
		"class": cls, "reach": 3.0, "state_frame": frame, "jumps": 1, "stocks": 3, "dodge": PackedInt32Array(), "dash_length": 2.4}


func yaw(view, bone: String, facing: int) -> float:
	var sk: Skeleton3D = view.skeleton
	var i := sk.find_bone(bone)
	var g: Transform3D = sk.global_transform * sk.get_bone_global_pose(i) * sk.get_bone_global_rest(i).affine_inverse()
	var ahead: Vector3 = g.basis * Vector3(0, 0, 1)
	return rad_to_deg(atan2(ahead.x * facing, ahead.z))


func _initialize() -> void:
	for cls in [0, 1, 2, 3, 8, 10]:
		for facing in [1, -1]:
			for st in [["Idle", 0.0], ["Run", 0.24]]:
				var view := FighterView.new()
				root.add_child(view)
				view.build(cls, Loadout.default_for(cls))
				await process_frame
				var h := 0.0
				var c := 0.0
				var f := 0.0
				var eyes := 0
				var hand := 0.0
				var width := 0.0
				for fr in 40:
					view.apply(Vector3.ZERO, snap(cls, st[0], fr + 1, facing, float(st[1]) * facing), 1.0 / 60.0)
					if fr >= 20:
						h += yaw(view, "hips", facing) / 20.0
						c += yaw(view, "spine", facing) / 20.0
						var fy := yaw(view, "head", facing)
						f += fy / 20.0
						if fy <= EYES_BOTH_AT:
							eyes += 1
						var sk: Skeleton3D = view.skeleton
						var far := "R" if facing > 0 else "L"
						var xs := []
						for b in ["hips", "spine", "head", "armU.L", "armL.L", "hand.L", "armU.R", "armL.R", "hand.R", "foot.L", "foot.R", "shin.L", "shin.R"]:
							xs.append((sk.global_transform * sk.get_bone_global_pose(sk.find_bone(b)).origin).x * facing)
						var chest_x: float = (sk.global_transform * sk.get_bone_global_pose(sk.find_bone("spine")).origin).x * facing
						var hx: float = (sk.global_transform * sk.get_bone_global_pose(sk.find_bone("hand." + far)).origin).x * facing
						hand += (hx - chest_x) / 20.0
						width += (xs.max() - xs.min()) / 20.0
				print("TURN class %d facing %+d %-4s hips %5.1f chest %5.1f face %5.1f eyes %3d%% farhand %+.2f width %.2f" % [cls, facing, st[0], h, c, f, eyes * 5, hand, width])
				view.queue_free()
	quit(0)
