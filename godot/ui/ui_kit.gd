extends RefCounted
## Menu pieces: slanted tags and buttons, round arrows, selector rows, stat rows, portraits and the backdrop.
## Colours and drawing helpers live in ui_draw.gd; the same names are repeated here for the screens.

const D := preload("res://ui/ui_draw.gd")
const Art := preload("res://ui/menu_art.gd")
const INK := D.INK
const CREAM_DARK := D.CREAM_DARK
const CREAM := D.CREAM
const SKY := D.SKY
const GOLD := D.GOLD
const TEAL := D.TEAL
const SLATE_TOP := D.SLATE_TOP
const SLATE_BOTTOM := D.SLATE_BOTTOM
const RED := D.RED
const BLUE := D.BLUE
const SKEW := D.SKEW


static func font() -> Font:
	return D.font()


# ---- Backdrop -------------------------------------------------------------------------------------------------

class Backdrop extends Control:
	## The menus' world behind everything: the meadow stage's land and sky, live in 3D (the windmill and fields, the floating islands,
	## lit cumulus drifting, the sun's light and the match's finishing pass), the camera drifting slowly past it, so the menus show the
	## game's own world rather than a drawing of a sky. On the Fast tier, the painted sky (menu_art.gd) instead. Never takes the mouse.
	const StageArt := preload("res://scripts/stage_art.gd")
	const Lighting := preload("res://scripts/lighting.gd")
	const VisualSettings := preload("res://scripts/visual_settings.gd")
	const POST := preload("res://shaders/post.gdshader")
	var t := 0.0
	var cam: Camera3D
	var env: Environment
	var root: Node3D
	var world_look := {}
	var post_mat: ShaderMaterial
	var lit_frames := 0

	## `dim`: a cool shade laid over the world (0 none), for screens dense with text, so it sits calmly over a busy picture.
	func _init(_warm := false, dim := 0.0) -> void:
		set_anchors_preset(Control.PRESET_FULL_RECT)
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		if dim > 0.0:
			# (Drawn after the world, as the backdrop's last child, so it shades only the world.)
			var shade := ColorRect.new()
			shade.set_anchors_preset(Control.PRESET_FULL_RECT)
			shade.mouse_filter = Control.MOUSE_FILTER_IGNORE
			shade.color = Color(0.08, 0.1, 0.2, dim)
			add_child.call_deferred(shade)
		if VisualSettings.get_index("quality") == 0:
			var sky := Art.SkyScene.new()
			sky.foliage = true
			sky.set_anchors_preset(Control.PRESET_FULL_RECT)
			add_child(sky)
			return
		var box := SubViewportContainer.new()
		box.stretch = true
		box.set_anchors_preset(Control.PRESET_FULL_RECT)
		box.mouse_filter = Control.MOUSE_FILTER_IGNORE
		add_child(box)
		var vp := SubViewport.new()
		vp.own_world_3d = true
		vp.msaa_3d = Viewport.MSAA_4X
		box.add_child(vp)
		root = Node3D.new()
		vp.add_child(root)
		env = Environment.new()
		var we := WorldEnvironment.new()
		we.environment = env
		root.add_child(we)
		Lighting.apply(env, root)
		world_look = StageArt.theme({})
		env.background_mode = Environment.BG_SKY
		env.sky = StageArt.sky(world_look)
		# The meadow's land and sky without its fighting island (its empty platforms showed through the gaps between the cards as clutter):
		# the island's place is given far below the view, so what stands on it and hangs under it is out of sight.
		var scenery := Node3D.new()
		root.add_child(scenery)
		StageArt.backdrop(scenery, world_look, [-14.0, 14.0, -60.0, -66.0, -14.0, 14.0])
		cam = Camera3D.new()
		cam.fov = 30.0
		root.add_child(cam)
		var post := MeshInstance3D.new()
		var quad := QuadMesh.new()
		quad.size = Vector2(2, 2)
		post.mesh = quad
		post.extra_cull_margin = 16384.0
		post.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		post_mat = ShaderMaterial.new()
		post_mat.shader = POST
		post_mat.render_priority = -100
		post_mat.set_shader_parameter("stage_dist", 62.0)
		post.material_override = post_mat
		post.position = Vector3(0, 0, -1)
		cam.add_child(post)
		_place_camera()

	func _place_camera() -> void:
		# A slow sway across the island, a little above it, so the world is never still and never busy.
		var x := sin(t * 0.06) * 9.0
		cam.look_at_from_position(Vector3(x, 9.0 + sin(t * 0.045) * 1.5, 62.0), Vector3(x * 0.4, 5.0, 0.0))

	func _process(delta: float) -> void:
		if cam == null:
			return
		t += delta
		_place_camera()
		# (The stage's light is set after the screen's own previews, which light themselves with the menus' neutral mood through the same
		# shared shader values: a frame later, so the world and the fighters in front of it share the meadow's light.)
		if lit_frames < 2:
			lit_frames += 1
			if lit_frames == 2:
				Lighting.stage_mood(env, root, world_look, post_mat)


# ---- Scenery ---------------------------------------------------------------------------------------------------

class Scene extends Control:
	## A painted sky (menu_art.gd) with a grassy ledge across the lower middle for a fighter to stand on: the scene behind the fighters in
	## the menus (the Play tile, the players' panels), warm or cool by `mood`. Put it in a `Mask` to give it a shape.
	var mood := 0.0:
		set(v):
			mood = v
			if sky != null:
				sky.mood = v
	var seed := 3:
		set(v):
			seed = v
			if sky != null:
				sky.seed = v
	var sky: Control
	var ledge: Control

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		sky = Art.SkyScene.new()
		sky.detail = 0.6
		add_child(sky)
		ledge = Art.Piece.new(_paint_ledge)
		add_child(ledge)

	func _notification(what: int) -> void:
		if what == NOTIFICATION_RESIZED or what == NOTIFICATION_READY:
			sky.size = size
			ledge.size = size
			ledge.queue_redraw()

	func _paint_ledge(c: CanvasItem) -> void:
		# A layered platform of grass over rock (the islands' drawing, without trees), lit from the upper left like everything else, its
		# top at 0.7 of the way down where the fighter's feet are, and a soft contact shadow under them so the fighter stands on it.
		var w := size.x
		var h := size.y
		var pal: Dictionary = Art.palette(mood)
		var span := minf(w * 0.8, h * 1.3)
		var top_y := h * 0.7
		Art.island(c, Vector2(w * 0.5, top_y + span * 0.08), span, Color("5cc46a").lerp(pal.grass, 0.4), pal.rock, pal.rock_light, seed + 5,
			false, 1.0, true, true)
		var shadow := PackedVector2Array()
		for k in 24:
			var a := TAU * k / 24.0
			shadow.append(Vector2(w * 0.5 + 6.0, top_y + 2.0) + Vector2(cos(a) * span * 0.13, sin(a) * span * 0.022))
		c.draw_colored_polygon(shadow, Color(Art.INK, 0.28))
		var core := PackedVector2Array()
		for q in shadow:
			core.append(Vector2(w * 0.5 + 6.0, top_y + 2.0) + (q - Vector2(w * 0.5 + 6.0, top_y + 2.0)) * 0.6)
		c.draw_colored_polygon(core, Color(Art.INK, 0.18))


class Spotlight extends Control:
	## A card's backdrop for fighters, as a fighting game frames its character art: the card's colour deepening from a lit top to a dark
	## foot, a soft pool of light behind the fighters, faint rays fanning out from it, a floor of light they stand on with a contact
	## shadow under each, and a fine diagonal hatch, so the fighters read as the subject of the card. `feet`: [x, y] shares of the size.
	var colour := Color(0.3, 0.5, 0.9)
	var feet: Array = []
	var seed := 3
	## The floor and contact shadows (off for a close-up on the face).
	var show_floor := true:
		set(v):
			show_floor = v
			queue_redraw()

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _notification(what: int) -> void:
		if what == NOTIFICATION_RESIZED:
			queue_redraw()

	func _draw() -> void:
		var w := size.x
		var h := size.y
		if w < 4.0 or h < 4.0:
			return
		var top := colour.lightened(0.28)
		var bottom := colour.darkened(0.45)
		draw_polygon(PackedVector2Array([Vector2.ZERO, Vector2(w, 0), Vector2(w, h), Vector2(0, h)]),
			PackedColorArray([top, top, bottom, bottom]))
		var centre := Vector2(w * 0.5, h * 0.5)
		if not feet.is_empty():
			var sum := Vector2.ZERO
			for f in feet:
				sum += Vector2(float(f[0]) * w, float(f[1]) * h)
			centre = sum / feet.size() - Vector2(0, h * 0.3)
		# Rays fanning from behind the fighters.
		var reach := maxf(w, h) * 1.2
		for k in 14:
			var a := TAU * (float(k) + Art.noise(seed + k) * 0.3) / 14.0
			var spread := 0.07 + 0.04 * absf(Art.noise(seed * 3 + k))
			draw_colored_polygon(PackedVector2Array([centre, centre + Vector2(cos(a - spread), sin(a - spread)) * reach,
				centre + Vector2(cos(a + spread), sin(a + spread)) * reach]), Color(1, 1, 1, 0.05))
		# The pool of light.
		for k in 8:
			draw_circle(centre, minf(w, h) * (0.62 - k * 0.07), Color(colour.lightened(0.6), 0.06))
		# A fine hatch over it all.
		var step := 10.0
		var x := -h
		while x < w:
			draw_line(Vector2(x, h), Vector2(x + h, 0), Color(1, 1, 1, 0.035), 2.0)
			x += step
		# The floor: a broad band of light under the fighters, and a contact shadow under each.
		if show_floor and not feet.is_empty():
			var fy := float(feet[0][1]) * h
			var floor_poly := PackedVector2Array()
			for k in 33:
				var a := TAU * k / 32.0
				floor_poly.append(Vector2(w * 0.5, fy + h * 0.02) + Vector2(cos(a) * w * 0.62, sin(a) * h * 0.11))
			draw_colored_polygon(floor_poly, Color(colour.lightened(0.45), 0.35))
			for f in feet:
				var at := Vector2(float(f[0]) * w + 4.0, float(f[1]) * h + 2.0)
				var sh := PackedVector2Array()
				for k in 21:
					var a := TAU * k / 20.0
					sh.append(at + Vector2(cos(a) * h * 0.12, sin(a) * h * 0.022))
				draw_colored_polygon(sh, Color(D.INK, 0.3))


class Mask extends Control:
	## A shape used only as a mask: its children (a `Scene`, say) show inside `shape` (in its own coordinates) and nowhere else.
	var shape := PackedVector2Array()

	func _init() -> void:
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		clip_children = CanvasItem.CLIP_CHILDREN_ONLY

	func set_shape(points: PackedVector2Array) -> void:
		shape = points
		queue_redraw()

	func _draw() -> void:
		if shape.size() >= 3:
			draw_colored_polygon(shape, Color.WHITE)


# ---- Key glyphs --------------------------------------------------------------------------------------------------

class KeyBar extends Control:
	## The controls along the bottom of a screen: a dark translucent ribbon of key glyphs and what they do. `items` is a list of
	## [key, what it does].
	var items: Array = []

	func _init(list: Array) -> void:
		items = list
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		# A slim slanted backing in the menus' deep indigo (the utility tiles' colour), with a lit top edge, tying the hints to the cards.
		var band := PackedVector2Array([Vector2(12, 0), Vector2(size.x, 0), Vector2(size.x - 12, size.y), Vector2(0, size.y)])
		draw_colored_polygon(band, Color(0.21, 0.22, 0.38, 0.88))
		draw_line(Vector2(13, 1.5), Vector2(size.x - 1, 1.5), Color(1, 1, 1, 0.18), 2.0, true)
		var f := D.font()
		var widths := []
		var total := 0.0
		for item in items:
			var kw := maxf(28.0, f.get_string_size(item[0], HORIZONTAL_ALIGNMENT_LEFT, -1, 15).x + 12.0)
			var tw := f.get_string_size(item[1], HORIZONTAL_ALIGNMENT_LEFT, -1, 16).x
			widths.append([kw, tw])
			total += kw + 6.0 + tw + 18.0
		var x := (size.x - total) * 0.5
		for k in items.size():
			var kw: float = widths[k][0]
			Art.key_cap(self, Rect2(x, size.y * 0.5 - 12.0, kw, 24.0), items[k][0])
			x += kw + 6.0
			draw_string(f, Vector2(x, size.y * 0.5 + 6.0), items[k][1], HORIZONTAL_ALIGNMENT_LEFT, -1, 16, Color(1, 1, 1, 0.9))
			x += float(widths[k][1]) + 18.0


# ---- Screen pieces -------------------------------------------------------------------------------------------------

class ScreenTitle extends Control:
	## A screen's name on an ink brush stroke at the top left (as on the creator and character select).
	var text := ""

	func _init(label: String, width := 470.0) -> void:
		text = label
		position = Vector2(12, 8)
		size = Vector2(width, 66)
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func _draw() -> void:
		var band := Rect2(Vector2.ZERO, Vector2(size.x, 62))
		Art.brush_band(self, band, Art.INK, 41 + text.length(), 20.0, true, 0.9)
		Art.title_centered(self, text.to_upper(), band.grow_individual(-10, 0, -22, 0), 44, Color.WHITE, 0.0)


## A line of news for the player (a status, a warning) on a dark band in gold, so it reads over the sky; nothing while it is empty.
static func notice(at: Vector2, width: float, font_size := 22) -> Tag:
	var t := Tag.new("", Vector2(width, 38))
	t.fill = Color(Art.INK, 0.78)
	t.ink = Art.SUN
	t.shadow = false
	t.font_size = font_size
	t.position = at
	return t


class Stage extends Control:
	## A fighter's own little stage in a card: a spotlit backdrop (`Spotlight`) with the fighter standing in its light.
	## `add_fighter` puts a preview (preview.gd, made without its floor) in, framed so its feet are on the platform.
	var mask: Control
	var scene: Control
	## The frame's inner rim (a player's colour on the results screen; pale by default).
	var rim := Color(1, 1, 1, 0.5)

	func _init(sz: Vector2, mood := 0.25, seed := 31) -> void:
		size = sz
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		mask = Mask.new()
		mask.position = Vector2(10, 10)
		mask.size = sz - Vector2(30, 32)
		var m := mask.size
		mask.set_shape(PackedVector2Array([Vector2(16, 0), Vector2(m.x, 0), Vector2(m.x, m.y - 16), Vector2(m.x - 16, m.y), Vector2(0, m.y),
			Vector2(0, 16)]))
		add_child(mask)
		# (A spotlit card, warmer with `mood`, as the main menu's tiles: the painted 2D sky that stood here no longer matched the live world
		# behind the screens.)
		var spot := Spotlight.new()
		spot.colour = Color.from_hsv(lerpf(0.62, 0.98, clampf(mood, 0.0, 1.0)), 0.6, 0.82)
		spot.feet = [[0.5, 0.7]]
		spot.seed = seed
		spot.size = m
		scene = spot
		mask.add_child(scene)

	func add_fighter(preview: Control) -> void:
		preview.spin = false
		preview.facing_bias = 0.35
		preview.framed = true
		# The camera aims so the feet are on the platform (0.7 of the way down; see the creator's stage).
		preview.body_from = Vector3(0, 1.0, 7.4)
		preview.body_at = Vector3(0, 0.79, 0)
		preview.position = Vector2.ZERO
		preview.size = mask.size
		mask.add_child(preview)

	func _draw() -> void:
		Art.card(self, Rect2(Vector2.ZERO, size - Vector2(10, 12)), Art.INDIGO, 61, 20.0, 5.0, rim)


## A wipe that opens a new screen: two slanted bands slide away. `UI.reveal(screen)` from a screen's `_ready`.
class Reveal extends CanvasLayer:
	var t := 0.0
	var board: Control

	func _init() -> void:
		layer = 100
		board = Control.new()
		board.set_anchors_preset(Control.PRESET_FULL_RECT)
		board.mouse_filter = Control.MOUSE_FILTER_IGNORE
		board.draw.connect(_paint)
		add_child(board)

	func _process(delta: float) -> void:
		t += delta
		board.queue_redraw()
		if t > 0.45:
			queue_free()

	func _paint() -> void:
		var w := board.size.x
		var h := board.size.y
		var k := clampf(t / 0.38, 0.0, 1.0)
		var ease := 1.0 - pow(1.0 - k, 3.0)
		var lean := h * 0.35
		var x1 := lerpf(-lean, w + lean, ease)
		board.draw_colored_polygon(PackedVector2Array([Vector2(x1, 0), Vector2(w + lean, 0), Vector2(w + lean, h), Vector2(x1 - lean, h)]), Color(0.06, 0.07, 0.14))
		var x2 := lerpf(-lean - 40.0, w + lean, clampf(ease * 1.12, 0.0, 1.0))
		board.draw_colored_polygon(PackedVector2Array([Vector2(x2, 0), Vector2(x2 + 26.0, 0), Vector2(x2 + 26.0 - lean, h), Vector2(x2 - lean, h)]), D.GOLD)


static func reveal(screen: Node) -> void:
	screen.add_child(Reveal.new())


# ---- Slanted label ----------------------------------------------------------------------------------------------

class Tag extends Control:
	## A label: text on a painted card (menu_art.gd), or on a plain slanted strip when its colour is see-through.
	var text := ""
	var seed := 1
	var fill := D.CREAM
	var ink := Color(0.11, 0.12, 0.17)
	var edge := Color(0, 0, 0, 0)
	var font_size := 24
	var shadow := true
	var highlight := false

	func _init(label := "", min_size := Vector2(120, 40)) -> void:
		text = label
		custom_minimum_size = min_size
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		seed = absi(label.hash()) % 997

	func set_text(t: String) -> void:
		text = t
		queue_redraw()

	func _draw() -> void:
		var rect := Rect2(Vector2.ZERO, size).grow(-4)
		if fill.a < 0.9 and text == "":
			return
		if fill.a < 0.9:
			var skew_pad := rect.size.y * D.SKEW
			D.draw_slant(self, rect, fill, D.GOLD if highlight else edge, 4.0 if highlight else 3.0, shadow)
			D.draw_text_centered(self, text, Rect2(rect.position.x + skew_pad * 0.5, rect.position.y, rect.size.x - skew_pad, rect.size.y), font_size, ink)
			return
		rect.size -= Vector2(3, 3)
		Art.card(self, rect, fill, seed, -1.0, -1.0, D.GOLD if highlight else Color(0, 0, 0, 0))
		D.draw_text_centered(self, text, rect, font_size, ink)




# ---- Slanted button ---------------------------------------------------------------------------------------------

class Btn extends Control:
	## A big button on a painted card. It lifts a little when hovered or selected with the keyboard, and emits `activated`. It repaints
	## only when something changes (and while selected, for its shine).
	signal activated
	var text := "":
		set(v):
			text = v
			queue_redraw()
	var fill := D.CREAM:
		set(v):
			fill = v
			queue_redraw()
	var ink := Color(0.11, 0.12, 0.17):
		set(v):
			ink = v
			queue_redraw()
	var accent := Color(1.0, 0.82, 0.22)
	var font_size := 34:
		set(v):
			font_size = v
			queue_redraw()
	var selected := false:
		set(v):
			selected = v
			queue_redraw()
	var disabled := false:
		set(v):
			disabled = v
			queue_redraw()
	var hover := false
	var pop := 0.0
	var seed := 1
	var shine := 0.0
	## A setting ("Label: value"): a dark card, the label in a quiet white and the value in the accent, as the menu's utility tiles are
	## (art review 6: the select screen's settings were plain pale bars unlike every other panel).
	var setting := false:
		set(v):
			setting = v
			if v:
				fill = Color(0.2, 0.22, 0.34)
				ink = Color(1, 1, 1)
			queue_redraw()

	func _init(label := "", min_size := Vector2(320, 64)) -> void:
		text = label
		custom_minimum_size = min_size
		mouse_filter = Control.MOUSE_FILTER_STOP
		seed = absi(label.hash()) % 997
		mouse_entered.connect(func():
			hover = true)
		mouse_exited.connect(func():
			hover = false)

	func _process(delta: float) -> void:
		var target := 1.0 if (selected or hover) and not disabled else 0.0
		var was := pop
		pop = move_toward(pop, target, delta * 7.0)
		# The shine crosses once when the button is selected (brief focus feedback), then the button rests.
		if selected and shine < 1.0:
			shine = minf(1.0, shine + delta / 0.5)
			queue_redraw()
		elif not selected:
			shine = 0.0
		if pop != was:
			queue_redraw()

	func _gui_input(event: InputEvent) -> void:
		if disabled:
			return
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			activated.emit()

	func _draw() -> void:
		# A painted card (menu_art.gd) that lifts and warms toward the accent colour when selected, with a shine sweeping across it.
		var rect := Rect2(Vector2.ZERO, size).grow(-6.0)
		rect.size -= Vector2(4, 6)
		var base := fill if not disabled else Color(0.6, 0.6, 0.6)
		var shown := base.lerp(accent, pop * 0.55)
		var lift := pop * 4.0
		var r := Rect2(rect.position - Vector2(0, lift), rect.size)
		Art.card_glow(self, r, accent, pop)
		var shape := Art.card(self, rect, shown, seed, -1.0, -1.0, Color(0, 0, 0, 0), lift)
		if selected and shine < 1.0:
			# The shine, crossing once.
			var sx := lerpf(r.position.x - 40.0, r.end.x + 40.0, shine)
			var lean := r.size.y * D.SKEW
			var band := PackedVector2Array([Vector2(sx, r.position.y + 2), Vector2(sx + 22, r.position.y + 2), Vector2(sx + 22 - lean, r.end.y - 2), Vector2(sx - lean, r.end.y - 2)])
			for poly in Geometry2D.intersect_polygons(band, shape):
				draw_colored_polygon(poly, Color(1, 1, 1, 0.28))
		# Long labels shrink to fit the button.
		var fs := font_size
		while fs > 12 and D.font().get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x > r.size.x - 28.0:
			fs -= 2
		if setting and text.contains(":"):
			var cut := text.find(":")
			var label := text.substr(0, cut + 1) + " "
			var value := text.substr(cut + 1).strip_edges()
			var f := D.font()
			var lw := f.get_string_size(label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
			var vw := f.get_string_size(value, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
			var x := r.position.x + (r.size.x - lw - vw) * 0.5
			var y := r.position.y + (r.size.y + f.get_ascent(fs) - f.get_descent(fs)) * 0.5
			draw_string(f, Vector2(x, y), label, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color(0.82, 0.84, 0.92))
			draw_string(f, Vector2(x + lw, y), value, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, Color(1.0, 0.82, 0.3))
			return
		D.draw_text_centered(self, text, r, fs, ink if not disabled else Color(0.35, 0.35, 0.35))



# ---- Menu tile ---------------------------------------------------------------------------------------------------------

class Tile extends Control:
	## A big menu tile: a painted card (menu_art.gd) in its own colour with a large line-drawn icon, a title and a subtitle. Selected, it
	## lifts, glows and brightens. `show_scene` paints a scene into it instead (the Play tile). Emits `activated`.
	signal activated
	var title := ""
	var subtitle := ""
	var icon := ""
	var color := Color(0.9, 0.3, 0.3)
	var selected := false:
		set(v):
			selected = v
			queue_redraw()
	var hover := false
	var pop := 0.0
	## 0 primary (a destination: bigger type, a deeper body under the card), 1 utility (quieter: a thinner line, a shallower body, smaller
	## type).
	var tier := 0
	## A press: the card sinks into its body for a moment (1 at the press, easing to 0).
	var press := 0.0
	const TILE_SKEW := 0.08

	func _init(label: String, sub: String, tint: Color, glyph: String) -> void:
		title = label
		subtitle = sub
		color = tint
		icon = glyph
		mouse_filter = Control.MOUSE_FILTER_STOP
		mouse_entered.connect(func(): hover = true)
		mouse_exited.connect(func(): hover = false)

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			activated.emit()

	## Shows the card being pressed (it sinks into its body and springs back).
	func pressed() -> void:
		press = 1.0
		queue_redraw()

	func _process(delta: float) -> void:
		var was := pop
		pop = move_toward(pop, 1.0 if (selected or hover) else 0.0, delta * 8.0)
		var was_press := press
		press = move_toward(press, 0.0, delta * 7.0)
		if pop == was and press == was_press and _laid == size:
			return
		_laid = size
		if mask != null:
			var r := _card_rect()
			mask.position = r.position + Vector2(10, 10 - _lift())
			mask.size = r.size - Vector2(20, 20)
			scene.size = mask.size
			var c := CUT - 6.0
			mask.set_shape(PackedVector2Array([Vector2(c, 0), Vector2(mask.size.x, 0), Vector2(mask.size.x, mask.size.y - c),
				Vector2(mask.size.x - c, mask.size.y), Vector2(0, mask.size.y), Vector2(0, c)]))
		for ch in get_children():
			if ch is Top:
				ch.queue_redraw()
		queue_redraw()

	const CUT := 20.0
	var _laid := Vector2(-1, -1)
	## The tile that leads (Play): it alone gets the painted rim.
	var hero := false
	## A painted scene inside the card (the Play tile's), made by `show_scene`.
	var mask: Control
	var scene: Control
	var top: Control

	class Top extends Control:
		## The tile's text, drawn above anything placed in the tile (its scene, a fighter).
		var tile: Control

		func _init(t: Control) -> void:
			tile = t
			set_anchors_preset(Control.PRESET_FULL_RECT)
			mouse_filter = Control.MOUSE_FILTER_IGNORE

		func _draw() -> void:
			tile.paint_text(self)

	## Fills the card with a painted scene (sky, islands, a ledge), under whatever is added with `add_layer`.
	func show_scene(mood := 0.0, seed := 5) -> void:
		mask = Mask.new()
		add_child(mask)
		scene = Scene.new()
		scene.mood = mood
		scene.seed = seed
		mask.add_child(scene)
		_ensure_top()

	## Fills the card with a spotlit backdrop in its own colour for fighters to stand in (see `Spotlight`): `feet` the fighters' feet,
	## as shares of the card's width and height.
	func show_spotlight(feet: Array) -> void:
		mask = Mask.new()
		add_child(mask)
		var spot := Spotlight.new()
		spot.colour = color
		spot.feet = feet
		spot.seed = absi(title.hash()) % 97
		scene = spot
		mask.add_child(spot)
		_ensure_top()

	## Adds a child above the tile's scene but below its text.
	func add_layer(node: Node) -> void:
		add_child(node)
		_ensure_top()

	func _ensure_top() -> void:
		if top == null:
			top = Top.new(self)
			add_child(top)
		move_child(top, -1)

	func _lift() -> float:
		return pop * 7.0 - press * (_slab() + 3.0)

	func _slab() -> float:
		return 7.0 if tier == 0 else 4.0

	func _card_rect() -> Rect2:
		return Rect2(Vector2(6, 6), size - Vector2(16, 18 + _slab()))

	func _draw() -> void:
		var r := _card_rect()
		Art.card_glow(self, Rect2(r.position - Vector2(0, _lift()), r.size), Color(1, 0.92, 0.6), pop, CUT)
		var fill := color.lerp(color.lightened(0.1), pop)
		# The line in a deep shade of the card's own colour (the hero keeps the ink, with its gold rim), thinner for a utility card.
		var line := Art.INK if hero else color.darkened(0.62)
		var weight := 4.0 if tier == 0 else 3.0
		Art.card(self, r, fill, absi(title.hash()) % 997, CUT, weight, Color(Art.SUN, 0.85) if hero else Color(0, 0, 0, 0), _lift(), line, _slab())
		# Focused: a bright accent along the top and left edges (it reads without the colour, as a lit edge with the lift).
		if pop > 0.01:
			var lr := Rect2(r.position - Vector2(0, _lift()), r.size)
			var a := Color(1, 0.97, 0.85, 0.95 * pop)
			draw_polyline(PackedVector2Array([Vector2(lr.position.x + 3.0, lr.end.y - 6.0), Vector2(lr.position.x + 3.0, lr.position.y + CUT + 1.0),
				Vector2(lr.position.x + CUT + 1.0, lr.position.y + 3.0), Vector2(lr.end.x - 6.0, lr.position.y + 3.0)]), a, 4.0, true)
		if top == null:
			paint_text(self)

	## The icon, the title and the subtitle (on `c`, the tile itself or its top layer).
	func paint_text(c: CanvasItem) -> void:
		var rect := _card_rect()
		rect.position.y -= _lift()
		# The icon: one size rule and one line weight for every tile, pale, with a shadow the way the light falls (down and right).
		var short := minf(rect.size.x, rect.size.y)
		var r := clampf(short * (0.27 if short > 200.0 else (0.21 if tier == 0 else 0.16)), 18.0, 64.0)
		var ic := Vector2(rect.end.x - r * 1.25 - 10.0, rect.position.y + maxf(r * 1.2 + 8.0, rect.size.y * 0.4))
		if rect.size.y < 110.0:
			ic.y = rect.get_center().y
		elif short <= 200.0:
			ic = Vector2(rect.end.x - r - 16.0, rect.position.y + r + 12.0)
		if mask != null:
			# Over a painted scene the text gets a band of shade to sit on, inside the card, fading up into the picture.
			var top_y := rect.end.y - rect.size.y * 0.3
			var cut := CUT
			var scrim := PackedVector2Array([Vector2(rect.position.x + 8.0, top_y), Vector2(rect.end.x - 8.0, top_y), Vector2(rect.end.x - 8.0, rect.end.y - cut - 2.0),
				Vector2(rect.end.x - cut - 2.0, rect.end.y - 8.0), Vector2(rect.position.x + 8.0, rect.end.y - 8.0)])
			var clear := Color(D.INK, 0.0)
			var dark := Color(D.INK, 0.55)
			c.draw_polygon(scrim, PackedColorArray([clear, clear, dark, dark, dark]))
		if icon != "":
			_icon(c, ic + Art.SHADOW * 0.5, r, Color(D.INK, 0.3))
			_icon(c, ic, r, Color(1, 1, 1, 0.82 + 0.18 * pop))
		# The title in the display face with a thin outline; the subtitle in the text face with only a soft shadow (outlines muddy small
		# type). Both at the bottom left, shrunk to fit.
		var f := D.display_font()
		var big := clampi(int(rect.size.y * 0.2), 24, 64) if tier == 0 else clampi(int(rect.size.y * 0.2), 20, 30)
		var room := rect.size.x - 44.0
		while big > 16 and f.get_string_size(title, HORIZONTAL_ALIGNMENT_LEFT, -1, big).x > room:
			big -= 2
		var body := D.font()
		var sub_size := 18 if rect.size.y > 200.0 else (16 if tier == 0 else 14)
		var has_sub := subtitle != "" and body.get_string_size(subtitle, HORIZONTAL_ALIGNMENT_LEFT, -1, sub_size).x <= room
		var pad := 22.0 if rect.size.y > 200.0 else 16.0
		var base := rect.end.y - (pad + sub_size + 8.0 if has_sub else pad)
		Art.title(c, title, Vector2(rect.position.x + pad, base), big, Color.WHITE, 0.09)
		if has_sub:
			var at := Vector2(rect.position.x + pad + 2.0, rect.end.y - pad + 2.0)
			c.draw_string(body, at + Vector2(1, 2), subtitle, HORIZONTAL_ALIGNMENT_LEFT, -1, sub_size, Color(D.INK, 0.45))
			c.draw_string(body, at, subtitle, HORIZONTAL_ALIGNMENT_LEFT, -1, sub_size, Color(1, 1, 1, 0.92))

	## Simple line icons drawn in code (original, no fonts or images), all with the same line weight.
	const ICON_LINE := 5.5

	func _icon(canvas: CanvasItem, c: Vector2, r: float, col: Color) -> void:
		# (The weight scales with the icon, so a small one keeps the same optical stroke as a big one.)
		var wdt := clampf(r * 0.1, 3.0, ICON_LINE)
		match icon:
			"online":
				# A globe: the outline, the equator, two lines of latitude and a meridian.
				canvas.draw_arc(c, r * 0.8, 0, TAU, 48, col, wdt, true)
				canvas.draw_line(c + Vector2(-r * 0.8, 0), c + Vector2(r * 0.8, 0), col, wdt, true)
				for y in [-0.45, 0.45]:
					var half := sqrt(0.64 - y * y) * r
					canvas.draw_line(c + Vector2(-half, y * r), c + Vector2(half, y * r), col, wdt, true)
				var ell := PackedVector2Array()
				for k in 33:
					var a := TAU * k / 32.0
					ell.append(c + Vector2(cos(a) * r * 0.34, sin(a) * r * 0.8))
				canvas.draw_polyline(ell, col, wdt, true)
			"replays":
				# A frame of film with a play mark.
				var rr := Rect2(c - Vector2(r * 0.9, r * 0.6), Vector2(r * 1.8, r * 1.2))
				canvas.draw_rect(rr, col, false, wdt)
				for k in 5:
					var x := rr.position.x + r * 0.16 + k * r * 0.33
					canvas.draw_rect(Rect2(Vector2(x, rr.position.y + wdt), Vector2(r * 0.13, r * 0.12)), col)
					canvas.draw_rect(Rect2(Vector2(x, rr.end.y - wdt - r * 0.12), Vector2(r * 0.13, r * 0.12)), col)
				canvas.draw_colored_polygon(PackedVector2Array([c + Vector2(-r * 0.18, -r * 0.24), c + Vector2(r * 0.3, 0), c + Vector2(-r * 0.18, r * 0.24)]), col)
			"creator":
				# A star.
				var pts := PackedVector2Array()
				for k in 10:
					var a := -PI / 2 + TAU * k / 10.0
					pts.append(c + Vector2(cos(a), sin(a)) * r * (0.82 if k % 2 == 0 else 0.36))
				pts.append(pts[0])
				canvas.draw_polyline(pts, col, wdt, true)
			"controls":
				# A pad: its body with two grips, a cross and two buttons.
				var outline := PackedVector2Array()
				for q in [Vector2(-0.55, -0.38), Vector2(0.55, -0.38), Vector2(0.78, -0.25), Vector2(0.92, 0.3), Vector2(0.8, 0.5), Vector2(0.58, 0.48),
						Vector2(0.38, 0.22), Vector2(-0.38, 0.22), Vector2(-0.58, 0.48), Vector2(-0.8, 0.5), Vector2(-0.92, 0.3), Vector2(-0.78, -0.25),
						Vector2(-0.55, -0.38)]:
					outline.append(c + q * r)
				canvas.draw_polyline(outline, col, wdt, true)
				canvas.draw_line(c + Vector2(-r * 0.6, -r * 0.06), c + Vector2(-r * 0.26, -r * 0.06), col, wdt, true)
				canvas.draw_line(c + Vector2(-r * 0.43, -r * 0.23), c + Vector2(-r * 0.43, r * 0.11), col, wdt, true)
				canvas.draw_circle(c + Vector2(r * 0.32, -r * 0.14), wdt * 0.85, col)
				canvas.draw_circle(c + Vector2(r * 0.52, r * 0.04), wdt * 0.85, col)
			"editors":
				# A wrench.
				canvas.draw_line(c + Vector2(-r * 0.6, r * 0.6), c + Vector2(r * 0.12, -r * 0.12), col, wdt * 1.4, true)
				canvas.draw_arc(c + Vector2(r * 0.34, -r * 0.34), r * 0.32, PI * 0.95, PI * 2.55, 24, col, wdt, true)
			"run":
				# A figure mid-stride with speed lines behind it (art review 6 UI round 1: the Run tile had no icon and read unfinished).
				canvas.draw_circle(c + Vector2(r * 0.3, -r * 0.62), r * 0.17, col)
				canvas.draw_line(c + Vector2(r * 0.2, -r * 0.38), c + Vector2(-r * 0.05, r * 0.15), col, wdt * 1.2, true)
				canvas.draw_polyline(PackedVector2Array([c + Vector2(-r * 0.05, r * 0.15), c + Vector2(r * 0.25, r * 0.4), c + Vector2(r * 0.15, r * 0.8)]), col, wdt, true)
				canvas.draw_polyline(PackedVector2Array([c + Vector2(-r * 0.05, r * 0.15), c + Vector2(-r * 0.35, r * 0.45), c + Vector2(-r * 0.65, r * 0.4)]), col, wdt, true)
				canvas.draw_polyline(PackedVector2Array([c + Vector2(-r * 0.4, -r * 0.3), c + Vector2(r * 0.12, -r * 0.22), c + Vector2(r * 0.45, -r * 0.02)]), col, wdt, true)
				for k in 3:
					var y := -r * 0.35 + k * r * 0.3
					canvas.draw_line(c + Vector2(-r * 1.0, y), c + Vector2(-r * (0.62 - 0.08 * k), y), col, wdt * 0.8, true)
			"quit":
				# A door and the way out.
				var door := Rect2(c - Vector2(r * 0.65, r * 0.75), Vector2(r * 0.75, r * 1.5))
				canvas.draw_rect(door, col, false, wdt)
				canvas.draw_line(c + Vector2(-r * 0.15, 0), c + Vector2(r * 0.7, 0), col, wdt, true)
				canvas.draw_colored_polygon(PackedVector2Array([c + Vector2(r * 0.62, -r * 0.28), c + Vector2(r * 1.0, 0), c + Vector2(r * 0.62, r * 0.28)]), col)


# ---- Round arrow button ------------------------------------------------------------------------------------------

class Arrow extends Control:
	signal pressed
	var direction := 1
	var hover := false
	var fill := D.CREAM

	func _init(dir := 1) -> void:
		direction = dir
		custom_minimum_size = Vector2(38, 38)
		mouse_filter = Control.MOUSE_FILTER_STOP
		mouse_entered.connect(func():
			hover = true
			queue_redraw())
		mouse_exited.connect(func():
			hover = false
			queue_redraw())

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			pressed.emit()

	func _draw() -> void:
		var c := size * 0.5
		var r := minf(size.x, size.y) * 0.5 - 3.0
		draw_circle(c + Vector2(2, 3), r, Color(0, 0, 0, 0.3))
		draw_circle(c, r, Color(1.0, 0.85, 0.3) if hover else fill)
		draw_arc(c, r, 0, TAU, 32, Color(0.11, 0.12, 0.17), 3.0, true)
		var d := float(direction)
		var tri := PackedVector2Array([c + Vector2(-5 * d, -9), c + Vector2(7 * d, 0), c + Vector2(-5 * d, 9)])
		draw_colored_polygon(tri, Color(0.82, 0.45, 0.2))
		tri.append(tri[0])
		draw_polyline(tri, Color(0.11, 0.12, 0.17), 2.0, true)


# ---- Selector row: [label] <  value  > ----------------------------------------------------------------------------

class Selector extends HBoxContainer:
	## A label tab, a left arrow, a value tab and a right arrow. `changed(index)` fires on every change.
	signal changed(index: int)
	signal focused
	var options: Array = []
	var index := 0
	var tag: Control
	var value_tag: Control
	var swatches := false
	var focus_mark := false

	func _init(label: String, choices: Array) -> void:
		options = choices
		add_theme_constant_override("separation", 6)
		tag = Tag.new(label, Vector2(150, 38))
		tag.fill = D.CREAM_DARK
		tag.font_size = 22
		add_child(tag)
		var left := Arrow.new(-1)
		left.pressed.connect(func(): step(-1))
		add_child(left)
		value_tag = Tag.new("", Vector2(210, 38))
		value_tag.font_size = 24
		add_child(value_tag)
		var right := Arrow.new(1)
		right.pressed.connect(func(): step(1))
		add_child(right)
		set_index(0, false)

	func step(delta: int) -> void:
		set_index((index + delta + options.size()) % options.size())

	func set_index(i: int, emit := true) -> void:
		index = clampi(i, 0, options.size() - 1)
		value_tag.set_text(str(options[index]).capitalize() if options[index] is String else str(options[index]))
		if emit:
			changed.emit(index)

	func set_focus_mark(on: bool) -> void:
		focus_mark = on
		tag.highlight = on
		tag.fill = D.GOLD.lerp(D.CREAM, 0.35) if on else D.CREAM_DARK
		tag.queue_redraw()

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed:
			focused.emit()




# ---- Stat row: label, pips, arrows ----------------------------------------------------------------------------------

class StatRow extends HBoxContainer:
	signal changed(value: int)
	signal focused
	## Emitted when a rise was refused because `can_raise` said there is nothing to spend.
	signal blocked
	var can_raise: Callable
	var value := 5
	var tag: Control
	var pips: Control
	var focus_mark := false

	class Pips extends Control:
		var value := 5
		func _init() -> void:
			custom_minimum_size = Vector2(250, 38)
			mouse_filter = Control.MOUSE_FILTER_IGNORE
		func _draw() -> void:
			for i in 9:
				var x := 6.0 + i * 27.0
				var on := i < value
				var rect := Rect2(x, 5.0, 22.0, 28.0)
				D.draw_slant(self, rect, D.GOLD if on else Color(0.2, 0.24, 0.3), Color(0.11, 0.12, 0.17), 2.0, false)

	func _init(label: String, start := 5) -> void:
		add_theme_constant_override("separation", 6)
		tag = Tag.new(label, Vector2(150, 38))
		tag.fill = D.CREAM_DARK
		tag.font_size = 22
		add_child(tag)
		var left := Arrow.new(-1)
		left.pressed.connect(func(): step(-1))
		add_child(left)
		pips = Pips.new()
		add_child(pips)
		var right := Arrow.new(1)
		right.pressed.connect(func(): step(1))
		add_child(right)
		set_value(start, false)

	func step(delta: int) -> void:
		if delta > 0 and value < 9 and can_raise.is_valid() and not can_raise.call():
			blocked.emit()
			return
		set_value(clampi(value + delta, 1, 9))

	func set_value(v: int, emit := true) -> void:
		value = clampi(v, 1, 9)
		pips.value = value
		pips.queue_redraw()
		if emit:
			changed.emit(value)

	func set_focus_mark(on: bool) -> void:
		focus_mark = on
		tag.highlight = on
		tag.fill = D.GOLD.lerp(D.CREAM, 0.35) if on else D.CREAM_DARK
		tag.queue_redraw()

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed:
			focused.emit()


# ---- Round portrait -----------------------------------------------------------------------------------------------------

class Portrait extends Control:
	## A round token showing a fighter's colour, face and hat colour: the character-select grid and the saved list use it.
	var body := Color(1.0, 0.85, 0.2)
	var accent := Color(0.25, 0.45, 0.92)
	var hat := 0
	var glasses := 0
	var face := 0
	var label := ""
	var ring := Color(0.11, 0.12, 0.17)
	var ring_width := 4.0
	var hover := false
	var glyph := ""      # "?" for random, "+" for new
	var scale_hint := 1.0
	signal clicked(button: int)

	func _init(diameter := 76.0) -> void:
		custom_minimum_size = Vector2(diameter, diameter)
		mouse_filter = Control.MOUSE_FILTER_STOP
		mouse_entered.connect(func():
			hover = true
			queue_redraw())
		mouse_exited.connect(func():
			hover = false
			queue_redraw())

	func set_look(l: RefCounted) -> void:
		body = l.body_color()
		accent = l.accent_color()
		hat = l.hat
		glasses = l.glasses
		face = l.face
		queue_redraw()

	func _gui_input(event: InputEvent) -> void:
		if event is InputEventMouseButton and event.pressed:
			clicked.emit(event.button_index)

	func _draw() -> void:
		var c := size * 0.5
		var r := minf(size.x, size.y) * 0.5 - ring_width
		draw_circle(c + Vector2(2, 4), r + 2, Color(0, 0, 0, 0.3))
		draw_circle(c, r, Color(1, 0.96, 0.86) if glyph == "" else Color(0.95, 0.9, 0.7))
		if glyph != "":
			D.draw_text_centered(self, glyph, Rect2(Vector2.ZERO, size), int(r * 1.3), Color(0.82, 0.45, 0.2))
		else:
			# Body blob, hat, eyes and mouth: a tiny version of the 3D fighter.
			var s := r / 38.0
			draw_circle(c + Vector2(0, 6 * s), 27 * s * scale_hint, body)
			draw_arc(c + Vector2(0, 6 * s), 27 * s * scale_hint, 0, TAU, 28, Color(0.11, 0.12, 0.17), 2.0, true)
			for sx in [-1.0, 1.0]:
				draw_circle(c + Vector2(sx * 10 * s, 4 * s), 5 * s, Color(1, 1, 1))
				draw_circle(c + Vector2(sx * 10 * s + sx * -0.5, 5 * s), 2.2 * s, Color(0.11, 0.12, 0.17))
			draw_line(c + Vector2(-5 * s, 16 * s), c + Vector2(5 * s, 16 * s), Color(0.11, 0.12, 0.17), 2.0)
			if hat != 0:
				var col := accent if hat in [1, 3, 4] else Color(0.85, 0.7, 0.3)
				draw_rect(Rect2(c + Vector2(-17 * s, -26 * s), Vector2(34 * s, 11 * s)), col)
				draw_rect(Rect2(c + Vector2(-17 * s, -26 * s), Vector2(34 * s, 11 * s)), Color(0.11, 0.12, 0.17), false, 2.0)
			if glasses != 0:
				draw_line(c + Vector2(-17 * s, 3 * s), c + Vector2(17 * s, 3 * s), Color(0.11, 0.12, 0.17), 3.0)
		draw_arc(c, r, 0, TAU, 40, ring if not hover else D.GOLD, ring_width, true)
		if label != "":
			draw_string(D.font(), Vector2(4, size.y - 2), label, HORIZONTAL_ALIGNMENT_LEFT, size.x - 8, 14, Color(1, 1, 1))


# ---- Speech bubble: a slanted panel with wrapped text ---------------------------------------------------------------------

class Bubble extends Control:
	var label: Label
	var fill := D.CREAM

	func _init(min_size := Vector2(380, 150)) -> void:
		custom_minimum_size = min_size
		mouse_filter = Control.MOUSE_FILTER_IGNORE
		label = Label.new()
		label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		label.add_theme_font_override("font", D.font())
		label.add_theme_font_size_override("font_size", 24)
		label.add_theme_color_override("font_color", D.INK)
		label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		add_child(label)

	func set_text(t: String) -> void:
		label.text = t
		_layout()

	func _layout() -> void:
		var rect := Rect2(Vector2.ZERO, size).grow(-4)
		var pad := rect.size.y * D.SKEW
		label.position = Vector2(rect.position.x + pad + 14.0, rect.position.y + 8.0)
		label.size = Vector2(maxf(rect.size.x - pad * 1.4 - 28.0, 10.0), maxf(rect.size.y - 16.0, 10.0))

	func _notification(what: int) -> void:
		if what == NOTIFICATION_RESIZED:
			_layout()

	func _draw() -> void:
		_layout()
		Art.card(self, Rect2(Vector2(4, 4), size - Vector2(14, 16)), fill, 77)


# ---- Stat bars: how a character plays -------------------------------------------------------------------------------------------

class Bars extends Control:
	var values := {}
	var order: Array = []

	func _init(min_size := Vector2(380, 200)) -> void:
		custom_minimum_size = min_size
		mouse_filter = Control.MOUSE_FILTER_IGNORE

	func set_values(v: Dictionary, keys: Array) -> void:
		values = v
		order = keys
		queue_redraw()

	func _draw() -> void:
		var f := D.font()
		var row := size.y / maxf(order.size(), 1)
		for i in order.size():
			var y := i * row
			draw_string(f, Vector2(4, y + row * 0.68), order[i], HORIZONTAL_ALIGNMENT_LEFT, 150, 22, Color(1, 1, 1, 0.92))
			var track := Rect2(160, y + row * 0.2, size.x - 170, row * 0.55)
			D.draw_slant(self, track, Color(0.16, 0.19, 0.25), D.INK, 2.0, false)
			var amount: float = clampf(float(values.get(order[i], 0.0)), 0.0, 1.0)
			if amount > 0.02:
				var fill := Rect2(track.position, Vector2(track.size.x * amount, track.size.y)).grow(-2)
				D.draw_slant(self, fill, D.GOLD, Color(0, 0, 0, 0), 0.0, false)
