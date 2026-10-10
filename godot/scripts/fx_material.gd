extends RefCounted
## The materials every effect draws with (hit bursts, trails, particles, rings): unshaded vertex colours whose bright cores glow
## (shaders/fx.gdshader). One shared material per kind, so each kind compiles once and can be warmed up before a match (`all`).

const FX_SHADER := preload("res://shaders/fx.gdshader")
const FX_TOP_SHADER := preload("res://shaders/fx_top.gdshader")

static var _cache := {}

## Effects run on match time when a match is on (they stop with a pause, slow with the knock-out slow motion, and line up with demo
## screenshots); `tick` is called by the match every frame with its clock, and `delta` turns a frame's real time into match time.
static var clocked := false
static var _last := -1.0
static var _dt := 0.0


static func tick(match_seconds: float) -> void:
	_dt = clampf(match_seconds - _last, 0.0, 0.1) if _last >= 0.0 else 0.0
	_last = match_seconds
	clocked = true


static func stop_clock() -> void:
	clocked = false
	_last = -1.0


## How far effects move on this frame: the match's time if it runs one, otherwise `real` (capped, so a hitch does not skip an effect).
static func delta(real: float) -> float:
	return _dt if clocked else minf(real, 1.0 / 20.0)


## `on_top` draws over everything; `texture` is laid on each quad (a soft disc for particles); `priority` orders see-through effects;
## `glow` scales how much the material's bright cores bloom (0: never).
static func get_material(on_top: bool, priority := 0, texture: Texture2D = null, glow := 1.0) -> ShaderMaterial:
	var key := "%s|%d|%s|%.2f" % [on_top, priority, texture.get_rid() if texture != null else 0, glow]
	if _cache.has(key):
		return _cache[key]
	var m := ShaderMaterial.new()
	m.shader = FX_TOP_SHADER if on_top else FX_SHADER
	m.render_priority = priority
	if texture != null:
		m.set_shader_parameter("tex", texture)
		m.set_shader_parameter("use_tex", true)
	m.set_shader_parameter("glow_scale", glow)
	_cache[key] = m
	return m


## Every material made so far (for warming them up).
static func all() -> Array:
	return _cache.values()


static func release() -> void:
	_cache.clear()
