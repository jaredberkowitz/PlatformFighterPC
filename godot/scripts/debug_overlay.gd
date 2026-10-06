extends CanvasLayer
## Training-style readout: per-player state, frame counters, flags, stick display, plus match info.

const StickDisplay := preload("res://scripts/stick_display.gd")
const FighterView := preload("res://scripts/fighter_view.gd")

var panels: Array = []
var labels: Array = []
var sticks: Array = []
var header: Label
var help: Label
var hint_visible := true
var warm_labels: Array = []


func _font() -> SystemFont:
	var f := SystemFont.new()
	f.font_names = PackedStringArray(["Consolas", "Cascadia Mono", "Courier New", "monospace"])
	return f


func _label(font: Font, size := 14) -> Label:
	var l := Label.new()
	l.add_theme_font_override("font", font)
	l.add_theme_font_size_override("font_size", size)
	l.add_theme_color_override("font_outline_color", Color(0, 0, 0, 0.9))
	l.add_theme_constant_override("outline_size", 5)
	return l


func build(player_count: int, masks: Dictionary) -> void:
	var font := _font()
	header = _label(font, 14)
	header.position = Vector2(12, 8)
	add_child(header)

	for i in player_count:
		var box := VBoxContainer.new()
		var right := i % 2 == 1
		box.anchor_top = 1.0
		box.anchor_bottom = 1.0
		box.anchor_left = 1.0 if right else 0.0
		box.anchor_right = 1.0 if right else 0.0
		box.offset_left = -320.0 if right else 12.0
		box.offset_right = -12.0 if right else 320.0
		box.offset_top = -230.0
		box.offset_bottom = -10.0
		add_child(box)
		var label := _label(font, 14)
		label.add_theme_color_override("font_color", FighterView.COLORS[i])
		box.add_child(label)
		var sd := StickDisplay.new()
		sd.masks = masks
		sd.color = FighterView.COLORS[i]
		box.add_child(sd)
		panels.append(box)
		labels.append(label)
		sticks.append(sd)

	help = _label(font, 12)
	help.anchor_left = 0.5
	help.anchor_right = 0.5
	help.anchor_top = 1.0
	help.anchor_bottom = 1.0
	help.offset_left = -330.0
	help.offset_right = 330.0
	help.offset_top = -64.0
	help.offset_bottom = -8.0
	help.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	help.text = "P1: WASD  Space jump  J atk  K spc  L/Shift shield  Ctrl = tilt (walk)    P2: Arrows  Enter jump  \\ tilt\n" \
		+ "S/Down: tap = crouch, double tap = fast fall   F1 overlay   F2 ECB   P pause   . step   , back   R restart"
	add_child(help)


func update(snaps: Array, inputs: Array, info: Dictionary) -> void:
	header.text = "frame %d   checksum %s   sim v%d   content %s%s" % [
		info.frame, info.checksum, info.version, info.content_hash,
		"   [PAUSED  history %d]" % info.history if info.paused else ""]
	for i in labels.size():
		var s: Dictionary = snaps[i]
		var inp: Dictionary = inputs[i]
		var flags: Array[String] = []
		if s.dodged: flags.append("dodged")
		if s.fast_fall: flags.append("fastfall")
		if s.ledge >= 0: flags.append("ledge%d" % s.ledge)
		labels[i].text = "P%d  %s  f%d\nface %s  plat %d  jumps %d  lag %d\npos %6.2f %6.2f   vel %6.3f %6.3f\nledge inv %d  grabs %d  cd %d\n%s" % [
			i + 1, s.state, s.state_frame, "R" if s.facing > 0 else "L", s.platform, s.jumps, s.lag,
			s.pos.x, s.pos.y, s.vel.x, s.vel.y, s.ledge_invuln, s.grabs, s.cooldown, " ".join(flags)]
		sticks[i].stick = Vector2(inp.x, inp.y) / 127.0
		sticks[i].buttons = inp.buttons
		sticks[i].min_down = info.min_down[i]
		sticks[i].queue_redraw()


func set_overlay_visible(v: bool) -> void:
	header.visible = v
	help.visible = v
	for p in panels:
		p.visible = v


## Draws every printable character at both overlay font sizes so the glyph cache is filled
## before play. Without this, the first time a new digit appears causes a hitch.
func prewarm_text(on: bool) -> void:
	if warm_labels.is_empty():
		var chars := ""
		for c in range(32, 127):
			chars += char(c)
		var font := _font()
		for size in [12, 14]:
			var l := _label(font, size)
			l.text = chars + "\n" + chars
			l.position = Vector2(0, 100 + (size - 12) * 60)
			add_child(l)
			warm_labels.append(l)
	for l in warm_labels:
		l.visible = on
