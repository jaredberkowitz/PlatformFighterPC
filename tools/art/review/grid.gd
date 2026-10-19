extends SceneTree
# 2-column grid: -- <out> <scale> <in...>
func _initialize() -> void:
	var a := OS.get_cmdline_user_args()
	var k := float(a[1])
	var imgs := []
	for i in range(2, a.size()):
		var img := Image.load_from_file(a[i])
		img.convert(Image.FORMAT_RGB8)
		img.resize(int(img.get_width() * k), int(img.get_height() * k), Image.INTERPOLATE_BILINEAR)
		imgs.append(img)
	var w: int = imgs[0].get_width()
	var h: int = imgs[0].get_height()
	var rows := (imgs.size() + 1) / 2
	var out := Image.create(w * 2 + 6, rows * (h + 6), false, Image.FORMAT_RGB8)
	for i in imgs.size():
		out.blit_rect(imgs[i], Rect2i(0, 0, w, h), Vector2i((i % 2) * (w + 6), (i / 2) * (h + 6)))
	out.save_png(a[0])
	quit()
