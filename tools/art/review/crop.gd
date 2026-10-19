extends SceneTree
# Crops and scales a region of a PNG: -- <in> <out> <x> <y> <w> <h> <scale>
func _initialize() -> void:
	var a := OS.get_cmdline_user_args()
	var img := Image.load_from_file(a[0])
	img.convert(Image.FORMAT_RGB8)
	var r := img.get_region(Rect2i(int(a[2]), int(a[3]), int(a[4]), int(a[5])))
	var k := float(a[6])
	r.resize(int(r.get_width() * k), int(r.get_height() * k), Image.INTERPOLATE_BILINEAR)
	r.save_png(a[1])
	quit()
