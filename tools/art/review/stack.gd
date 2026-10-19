extends SceneTree
# Stacks PNGs vertically: -- <out> <scale> <in...>
func _initialize() -> void:
	var a := OS.get_cmdline_user_args()
	var k := float(a[1])
	var imgs := []
	var w := 0
	var h := 0
	for i in range(2, a.size()):
		var img := Image.load_from_file(a[i])
		img.convert(Image.FORMAT_RGB8)
		img.resize(int(img.get_width() * k), int(img.get_height() * k), Image.INTERPOLATE_BILINEAR)
		imgs.append(img)
		w = max(w, img.get_width())
		h += img.get_height() + 6
	var out := Image.create(w, h, false, Image.FORMAT_RGB8)
	var y := 0
	for img in imgs:
		out.blit_rect(img, Rect2i(Vector2i.ZERO, img.get_size()), Vector2i(0, y))
		y += img.get_height() + 6
	out.save_png(a[0])
	quit()
