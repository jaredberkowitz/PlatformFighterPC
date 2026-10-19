extends SceneTree
# Greyscale of an image (luma), for the readability test: -- <in> <out>
func _initialize() -> void:
	var a := OS.get_cmdline_user_args()
	var img := Image.load_from_file(a[0])
	img.convert(Image.FORMAT_RGB8)
	for y in img.get_height():
		for x in img.get_width():
			var c := img.get_pixel(x, y)
			var l := c.r * 0.299 + c.g * 0.587 + c.b * 0.114
			img.set_pixel(x, y, Color(l, l, l))
	img.save_png(a[1])
	quit()
