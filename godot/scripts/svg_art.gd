extends RefCounted
## Turns the vector artwork in `godot/art/` (drawn faces, cloth prints) into textures. The SVG files are the asset: edit them in any
## vector editor. They are kept as files rather than imported (their `.import` says "keep") so their colours can be swapped when they
## load: a placeholder colour in the drawing (for example #ff00ff for the shirt colour) becomes the colour the fighter wears.
## Textures are made once per file, colours and size, then shared.

static var _cache := {}


## The texture for the SVG at `path` with each placeholder colour in `recolor` (a "#rrggbb" string, lower case) replaced, drawn
## `scale` times its own size. Null if the file is missing or cannot be drawn.
static func texture(path: String, recolor := {}, scale := 1.0) -> Texture2D:
	var key := "%s|%s|%s" % [path, recolor, scale]
	if _cache.has(key):
		return _cache[key]
	_cache[key] = null
	var text := FileAccess.get_file_as_string(path)
	if text == "":
		push_warning("Missing artwork: %s" % path)
		return null
	for from in recolor:
		text = text.replace(from, "#" + (recolor[from] as Color).to_html(false))
	var image := Image.new()
	if image.load_svg_from_string(text, scale) != OK:
		push_warning("Could not draw %s" % path)
		return null
	image.generate_mipmaps()
	var tex := ImageTexture.create_from_image(image)
	_cache[key] = tex
	return tex


static func release() -> void:
	_cache.clear()
