extends CanvasLayer
## The pause screen (battles and runs), after the reference's: Esc (or a controller's Start) freezes the match and shows a column of
## choices over the softened stage. Resume; Restart; Camera, a free look (move and zoom with the stick or the arrow keys and W/S
## zoom, Esc or Start to come back); Hide display; Quit, which asks once more before leaving. Online the match cannot stop, so
## the same screen only offers Resume, Hide display and Leave, over the running match.
## Up and down choose, Enter (or J, Space) picks, Esc resumes. The scene acts on the choice through `chosen`.

const UI := preload("res://ui/ui_kit.gd")
const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")

signal chosen(action: String)

var buttons: Array = []
var actions: Array = []
var labels: Array = []
var index := 0
var online := false
var title := "PAUSED"
## Set by the scene while the free camera is on: the menu hides and only Esc / Enter bring it back.
var camera_mode := false
var confirm_quit := false
var board: Control
var column: VBoxContainer
var hint: Control


class Board extends Control:
	var owner_screen

	func _init() -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var w := size.x
		draw_rect(Rect2(Vector2.ZERO, size), Color(Art.INK, 0.45))
		var banner := Rect2(Vector2(w * 0.5 - 220, 70), Vector2(440, 84))
		Art.brush_band(self, banner, Art.INK, 503, 22.0, true, 1.0)
		Art.title_centered(self, owner_screen.title, banner.grow_individual(-12, 0, -26, 0), 58, Color.WHITE, 0.0)
		if owner_screen.online:
			draw_string(D.font(), Vector2(w * 0.5 - 200, 182), "Online: the match keeps going while this is open.",
				HORIZONTAL_ALIGNMENT_LEFT, 400, 18, Color(1, 1, 1, 0.85))


func _init() -> void:
	layer = 12


## `choices`: [[label, action], ...] (the first is selected). `is_online` hides the choices a networked match cannot make.
func build(choices: Array, is_online: bool, heading := "PAUSED") -> void:
	online = is_online
	title = heading
	board = Board.new()
	board.owner_screen = self
	add_child(board)
	column = VBoxContainer.new()
	column.add_theme_constant_override("separation", 12)
	column.position = Vector2(640 - 150, 210)
	add_child(column)
	for c in choices:
		var b := UI.Btn.new(c[0], Vector2(300, 58))
		b.font_size = 26
		var action: String = c[1]
		b.activated.connect(func(): _pick(action))
		column.add_child(b)
		buttons.append(b)
		actions.append(action)
		labels.append(c[0])
	hint = UI.KeyBar.new([["Up/Down", "Choose"], ["Enter", "Pick"], ["Esc", "Resume"]])
	hint.position = Vector2(340, 664)
	hint.size = Vector2(600, 36)
	add_child(hint)
	_select(0)


func _select(i: int) -> void:
	index = posmod(i, buttons.size())
	for n in buttons.size():
		buttons[n].selected = n == index
	if confirm_quit and actions[index] != "quit":
		_reset_quit()


func _reset_quit() -> void:
	confirm_quit = false
	var q := actions.find("quit")
	if q >= 0:
		buttons[q].text = labels[q]


func _pick(action: String) -> void:
	if action == "quit" and not confirm_quit:
		# Quitting asks once more: the same choice again leaves.
		confirm_quit = true
		buttons[actions.find("quit")].text = "Sure? Pick again"
		return
	chosen.emit(action)


## Shows or hides the menu around the free camera.
func set_camera_mode(on: bool) -> void:
	camera_mode = on
	board.visible = not on
	column.visible = not on
	hint.visible = true
	remove_child(hint)
	hint = UI.KeyBar.new([["Arrows", "Move"], ["W/S", "Zoom"], ["Esc", "Back"]] if on
		else [["Up/Down", "Choose"], ["Enter", "Pick"], ["Esc", "Resume"]])
	hint.position = Vector2(340, 664)
	hint.size = Vector2(600, 36)
	add_child(hint)


## A key press while the menu is open. Returns true when it was used.
func handle_key(event: InputEventKey) -> bool:
	if camera_mode:
		if event.keycode in [KEY_ESCAPE, KEY_ENTER, KEY_KP_ENTER, KEY_SPACE]:
			chosen.emit("camera_done")
			return true
		return false
	match event.keycode:
		KEY_UP, KEY_W:
			_select(index - 1)
		KEY_DOWN, KEY_S:
			_select(index + 1)
		KEY_ENTER, KEY_KP_ENTER, KEY_SPACE, KEY_J:
			_pick(actions[index])
		KEY_ESCAPE:
			chosen.emit("resume")
		_:
			return false
	return true
