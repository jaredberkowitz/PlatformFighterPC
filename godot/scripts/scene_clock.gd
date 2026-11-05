class_name SceneClock
extends Node
## Scenery on twos (the style pass, docs/art/stages/STYLE_PASS.md): the scenery's clock. With `steps` pictures a second the scenery moves
## only on those pictures, all on the same frames, as a stop-motion set is shot; with 0 it moves every frame. The fighters, the camera and
## the effects (on twos of their own, scripts/particles.gd) never read it. StageArt sets `steps` for each stage (StageArt.SCENE_STEPS_TRY)
## and adds one of these to the stage, so the shaders' clock (shaders/scene_time.gdshaderinc) ticks even where nothing else moves.
## Cosmetic: nothing here reaches the sim.

## The scenery's pictures a second; 0 for one every frame.
static var steps := 0.0
static var _clock := 0.0
static var _frame := -1


## The scenery's picture this frame, the same for everything that moves in it: with `steps` 0 every frame is a new one. The clock runs
## on the game's frame time (a recording's too), advanced once a frame by the first thing to ask; the shaders take the same picture's
## time (`pf_scene_t`).
static func picture(delta: float) -> int:
	var f := Engine.get_process_frames()
	if f != _frame:
		_frame = f
		_clock = fmod(_clock + delta, 3600.0)
		if steps > 0.0:
			RenderingServer.global_shader_parameter_set("pf_scene_t", floorf(_clock * steps) / steps)
	if steps <= 0.0:
		return f
	return int(floorf(_clock * steps))


func _process(delta: float) -> void:
	RenderingServer.global_shader_parameter_set("pf_scene_steps", steps)
	picture(delta)


## The shaders back on TIME when the stage goes.
func _exit_tree() -> void:
	RenderingServer.global_shader_parameter_set("pf_scene_steps", 0.0)
