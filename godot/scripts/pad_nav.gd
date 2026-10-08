extends Node
## Lets a controller drive the menus. It watches the first two controllers and turns D-pad and left stick presses (with
## key-repeat when held), A, B, Start and Back into the key presses the menu screens already understand, so every screen works
## with a controller without knowing about it.
##
## A screen can define `pad_scheme(pad: int) -> Dictionary` to choose the keys for a controller (character select gives controller
## 1 the player 1 keys and controller 2 the player 2 keys), or return an empty dictionary to switch controller menu navigation off
## (during a match the controller plays). Without it the arrows, Enter and Escape are used.

const DEFAULT := {
	"up": KEY_UP, "down": KEY_DOWN, "left": KEY_LEFT, "right": KEY_RIGHT,
	"confirm": KEY_ENTER, "back": KEY_ESCAPE, "start": KEY_ENTER, "menu": KEY_ESCAPE,
}
## Seconds before a held direction repeats, and between repeats.
const FIRST_REPEAT := 0.4
const REPEAT := 0.11
const STICK := 0.6

var held := {}


## Adds the helper to the tree's root once (it survives scene changes).
static func attach(owner: Node) -> void:
	if owner.get_tree().root.has_node("PadNav"):
		return
	var n: Node = load("res://scripts/pad_nav.gd").new()
	n.name = "PadNav"
	owner.get_tree().root.add_child.call_deferred(n)


func _process(delta: float) -> void:
	var scene := get_tree().current_scene
	var pads: Array = Input.get_connected_joypads()
	for pad in pads.slice(0, 2):
		var scheme: Dictionary = DEFAULT
		if scene != null and scene.has_method("pad_scheme"):
			scheme = scene.pad_scheme(pad)
		if scheme.is_empty():
			continue
		var x := Input.get_joy_axis(pad, JOY_AXIS_LEFT_X)
		var y := Input.get_joy_axis(pad, JOY_AXIS_LEFT_Y)
		var state := {
			"up": Input.is_joy_button_pressed(pad, JOY_BUTTON_DPAD_UP) or y < -STICK,
			"down": Input.is_joy_button_pressed(pad, JOY_BUTTON_DPAD_DOWN) or y > STICK,
			"left": Input.is_joy_button_pressed(pad, JOY_BUTTON_DPAD_LEFT) or x < -STICK,
			"right": Input.is_joy_button_pressed(pad, JOY_BUTTON_DPAD_RIGHT) or x > STICK,
			"confirm": Input.is_joy_button_pressed(pad, JOY_BUTTON_A),
			"back": Input.is_joy_button_pressed(pad, JOY_BUTTON_B),
			"start": Input.is_joy_button_pressed(pad, JOY_BUTTON_START),
			"menu": Input.is_joy_button_pressed(pad, JOY_BUTTON_BACK),
		}
		for action in state:
			var id := "%d:%s" % [pad, action]
			if not state[action]:
				held.erase(id)
				continue
			var t: float = held.get(id, -1.0)
			var repeats: bool = action in ["up", "down", "left", "right"]
			if t < 0.0:
				held[id] = 0.0
				_press(scheme.get(action, 0))
			else:
				t += delta
				if repeats and t >= FIRST_REPEAT:
					t -= REPEAT
					_press(scheme.get(action, 0))
				held[id] = t


func _press(code: int) -> void:
	if code <= 0:
		return
	for down in [true, false]:
		var ev := InputEventKey.new()
		ev.keycode = code
		ev.physical_keycode = code
		ev.pressed = down
		Input.parse_input_event(ev)
