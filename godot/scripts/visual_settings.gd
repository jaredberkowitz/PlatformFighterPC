extends RefCounted
## Display and comfort settings (presentation only, never the simulation): how much the camera shakes, how strong full-screen flashes
## are, how many particles effects throw, and the rendering quality. Chosen on the Controls screen's Display page and kept in
## user://display.cfg. The match reads them through the helpers below.

const PATH := "user://display.cfg"

## Each setting: its choices (as shown) and the default's index.
const OPTIONS := {
	"shake": {"label": "Screen shake", "choices": ["Off", "Low", "Full"], "default": 2,
		"about": "How much the camera shakes and punches in on heavy hits."},
	"flashes": {"label": "Flashes", "choices": ["Off", "Soft", "Full"], "default": 2,
		"about": "Full-screen flashes on heavy hits and finishing blows. Off or Soft for photosensitivity or comfort."},
	"effects": {"label": "Effects", "choices": ["Low", "Medium", "Full"], "default": 2,
		"about": "How many sparks, dust puffs and confetti hit effects throw."},
	"quality": {"label": "Quality", "choices": ["Fast", "Balanced", "Best"], "default": 2,
		"about": "Fast turns off shadows, ambient occlusion and anti-aliasing for slower machines; Balanced turns off ambient occlusion."},
}
const ORDER := ["shake", "flashes", "effects", "quality"]

const SHAKE_SCALE := [0.0, 0.5, 1.0]
const FLASH_SCALE := [0.0, 0.45, 1.0]
const EFFECTS_SCALE := [0.4, 0.7, 1.0]

static var _values := {}
static var _loaded := false


static func get_index(key: String) -> int:
	_load()
	return int(_values.get(key, OPTIONS[key].default))


static func set_index(key: String, i: int) -> void:
	_load()
	_values[key] = clampi(i, 0, OPTIONS[key].choices.size() - 1)
	var cfg := ConfigFile.new()
	for k in ORDER:
		cfg.set_value("display", k, get_index(k))
	cfg.save(PATH)


static func text(key: String) -> String:
	return OPTIONS[key].choices[get_index(key)]


static func shake() -> float:
	return SHAKE_SCALE[get_index("shake")]


static func flashes() -> float:
	return FLASH_SCALE[get_index("flashes")]


## The finishing blow's two-tone frames are a hard full-screen flash: only with flashes at Full.
static func two_tone() -> bool:
	return get_index("flashes") == 2


static func effects() -> float:
	return EFFECTS_SCALE[get_index("effects")]


## Applies the quality setting to a scene's environment and lights (the key light's shadows) and the viewport's anti-aliasing.
static func apply_quality(env: Environment, root: Node, viewport: Viewport) -> void:
	var q := get_index("quality")
	if q <= 1:
		env.ssao_enabled = false
	if q == 0:
		var key := root.get_node_or_null("KeyLight") as DirectionalLight3D
		if key != null:
			key.shadow_enabled = false
		viewport.msaa_3d = Viewport.MSAA_DISABLED


static func _load() -> void:
	if _loaded:
		return
	_loaded = true
	var cfg := ConfigFile.new()
	if cfg.load(PATH) != OK:
		return
	for k in ORDER:
		var v = cfg.get_value("display", k, OPTIONS[k].default)
		if typeof(v) == TYPE_INT:
			_values[k] = clampi(v, 0, OPTIONS[k].choices.size() - 1)
