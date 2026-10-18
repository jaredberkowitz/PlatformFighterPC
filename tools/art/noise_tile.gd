extends SceneTree
## Paints a seamless greyscale value tile from noise, in the same format as tools/art/mm_tile.gd (the stage colour is multiplied by it,
## so a tile serves every hue), for surfaces no Material Maker export covers:
##   bark: tree bark as soft vertical plates parted by wide, gentle crevices and broken into lengths, no thin dark streaks
##         (the owner, 2026-10-15: the old bark's sharp brown streaks read as nothing real and distracted);
##   palm: a palm trunk's horizontal ring scars, each ring a soft lip of light over a band of shade, wobbling a little round the trunk;
##   leaves: overlapping leaf clumps like scales, each lit along its top edge and deeper toward its foot, casting a soft crescent of
##         shade on the clump behind it (the owner, 2026-10-17: the grove's crowns read as uniform green blobs of embossed dimples).
##   wood: four planks a repeat (as godot/art/stage/wood.svg, so it maps the same), each its own tone, a lit edge along its top and a
##         shaded one along its foot over a dark seam, long painted grain streaks wavering along it and a knot here and there that the
##         grain bends round (the owner's material notes, 2026-10-17: wood wants visible grain and clear light and shadow).
##   paving: pavement slabs in staggered rows, each its own tone, every joint a dark groove with a lit lip on the slab's upper and left
##         edges and shade on its lower and right ones, so the slabs read raised and bevelled (the owner, 2026-10-18: the street read flat);
##   roofing: a flat roof's membrane in long sheets across the tile, each lapped over the next with a lit edge and a shadow under it,
##         soft weathering stains and a fine gravel speckle.
## Run: Godot --headless -s tools/art/noise_tile.gd -- <bark|palm|leaves|wood|paving|roofing> <out.png> [seed]

const SIZE := 512
## Where the bark's crevices cross the tile (a share of its width), uneven so the plates differ in width.
const CREVICES := [0.04, 0.17, 0.26, 0.43, 0.58, 0.71, 0.86]


func _initialize() -> void:
	var a := OS.get_cmdline_user_args()
	if a.size() < 2:
		push_error("usage: <bark|palm> <out.png> [seed]")
		quit(1)
		return
	var kind: String = a[0]
	var seed := int(a[2]) if a.size() > 2 else 7
	var out := Image.create(SIZE, SIZE, false, Image.FORMAT_L8)
	var cells := _noise(FastNoiseLite.TYPE_CELLULAR, seed, 6.0)
	cells.cellular_return_type = FastNoiseLite.RETURN_DISTANCE2_SUB
	var soft := _noise(FastNoiseLite.TYPE_SIMPLEX_SMOOTH, seed + 1, 3.0)
	var fine := _noise(FastNoiseLite.TYPE_SIMPLEX_SMOOTH, seed + 2, 14.0)
	var clumps := _clumps(seed) if kind == "leaves" else []
	for y in SIZE:
		for x in SIZE:
			var u := float(x) / SIZE
			var v := float(y) / SIZE
			var t := 0.0
			match kind:
				"bark":
					# Plates: seven crevices a repeat at uneven spacing, each wandering a little as it runs up the trunk (a broad noise bends it),
					# soft-edged (a groove in shade, not a drawn line); between them each plate gets its own value and a soft wash, and a few
					# short cracks across it break the plates into lengths.
					var wu := u + 0.045 * _tiled(soft, u, v, 1.0, 1.0)
					var d := 1.0
					var plate := 0
					for k in CREVICES.size():
						var c: float = CREVICES[k]
						var dd := absf(fposmod(wu - c + 0.5, 1.0) - 0.5)
						if dd < d:
							d = dd
					for k in CREVICES.size():
						if fposmod(wu - CREVICES[k], 1.0) < fposmod(wu - CREVICES[plate], 1.0):
							plate = k
					var groove := smoothstep(0.006, 0.045, d)
					# (Every plate is cracked across twice a repeat, each at its own height, so the breaks stagger like real bark.)
					var across := absf(fposmod(v * 2.0 + float(plate) * 0.37, 1.0) - 0.5)
					var crack := 1.0 - 0.45 * (1.0 - smoothstep(0.004, 0.025, absf(across - 0.25)))
					var tone := 0.62 + 0.38 * fposmod(float(plate) * 0.618, 1.0)
					t = groove * crack * (0.7 * tone + 0.3 * (0.5 + 0.5 * _tiled(fine, u, v, 1.0, 1.0)))
				"palm":
					# Rings: six a repeat, each lifted to a lit lip and falling into shade under it, the line wobbling round the trunk.
					var wob := 0.03 * _tiled(soft, u, v, 1.0, 1.0)
					var ph := fposmod((v + wob) * 6.0, 1.0)
					var ring := smoothstep(0.0, 0.18, ph) * (1.0 - smoothstep(0.55, 1.0, ph) * 0.55)
					t = 0.7 * ring + 0.3 * (0.5 + 0.5 * _tiled(fine, u, v, 1.0, 1.0))
				"leaves":
					t = _leaves(clumps, u, v) * (0.9 + 0.1 * (0.5 + 0.5 * _tiled(fine, u, v, 1.0, 1.0)))
				"wood":
					t = _wood(soft, fine, u, v)
				"paving":
					t = _paving(soft, fine, u, v)
				"roofing":
					t = _roofing(soft, fine, u, v)
			# A few flat values with soft steps between them (the same painted read as mm_tile), then the multiplier range, in sRGB.
			t = lerpf(t, roundf(t * 3.0) / 3.0, 0.25 if kind == "wood" else 0.4)
			# (Bark spans a deeper range: at the rings' 0.66 its plates all but vanished on the grove's great trunk.)
			var value := lerpf({"bark": 0.42, "leaves": 0.6, "wood": 0.3, "paving": 0.25, "roofing": 0.45}.get(kind, 0.66), 1.0, clampf(t, 0.0, 1.0))
			value = pow(value, 1.0 / 2.2)
			out.set_pixel(x, y, Color(value, value, value))
	out.save_png(a[1])
	print("noise_tile: %s -> %s" % [kind, a[1]])
	quit(0)


func _noise(type: int, seed: int, frequency: float) -> FastNoiseLite:
	var n := FastNoiseLite.new()
	n.noise_type = type
	n.seed = seed
	n.frequency = frequency
	return n


## Noise `n` at (u, v) in a tile repeating every `ru` across and `rv` up, made seamless by sampling it on a torus in 4D (two circles).
func _tiled(n: FastNoiseLite, u: float, v: float, ru: float, rv: float) -> float:
	var au := TAU * u / ru
	var av := TAU * v / rv
	var k := 1.0 / TAU
	# (FastNoiseLite has no 4D: a 3D slice along the torus, blended with its mirror, is seamless across both edges.)
	var p := Vector3(cos(au) * ru * k, sin(au) * ru * k, (cos(av) + sin(av)) * rv * k)
	var q := Vector3(cos(au) * ru * k + 7.3, (cos(av) - sin(av)) * rv * k, sin(au) * ru * k - 3.1)
	return lerpf(n.get_noise_3dv(p), n.get_noise_3dv(q), 0.5)


## The leaf clumps of a tile: rows of round clumps, each row offset by half a clump and every centre and size jittered, [centre, radius].
func _clumps(seed: int) -> Array:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed
	var out := []
	var rows := 4
	var cols := 3
	for r in rows:
		for c in cols:
			var cx := (float(c) + 0.5 * float(r % 2) + rng.randf_range(-0.25, 0.25)) / cols
			var cy := (float(r) + rng.randf_range(-0.2, 0.2)) / rows
			out.append([Vector2(cx, cy), rng.randf_range(0.17, 0.27), rng.randf() * TAU])
	return out


## A leaf tile's value at (u, v): the clump in front (the lowest centre covering the point) lit toward its top and deeper toward its
## foot, a lit lip along its upper edge, and a crescent of shade where a clump in front of it overlaps just below.
func _leaves(clumps: Array, u: float, v: float) -> float:
	var p := Vector2(u, v)
	var best_y := -INF
	var best := []
	var shadow := 0.0
	for c in clumps:
		for ox in [-1.0, 0.0, 1.0]:
			for oy in [-1.0, 0.0, 1.0]:
				var ctr: Vector2 = c[0] + Vector2(ox, oy)
				var d := p.distance_to(ctr) / (float(c[1]) * _lobe(p - ctr, c[2]))
				if d < 1.0 and ctr.y > best_y:
					best_y = ctr.y
					best = [ctr, float(c[1]), d]
	if best.is_empty():
		return 0.5
	var ctr: Vector2 = best[0]
	var r: float = best[1]
	var dy := (v - ctr.y) / r
	var t := 0.74 - 0.2 * dy
	# The lit lip along the top edge.
	if best[2] > 0.86 and dy < -0.3:
		t = 0.95
	# Shade cast by a clump in front (lower), just below this point.
	for c in clumps:
		for ox in [-1.0, 0.0, 1.0]:
			for oy in [-1.0, 0.0, 1.0]:
				var c2: Vector2 = c[0] + Vector2(ox, oy)
				if c2.y > ctr.y + 0.001:
					var d2 := p.distance_to(c2 + Vector2(0, -0.02)) / (float(c[1]) * _lobe(p - c2, c[2]))
					if d2 < 1.18:
						shadow = maxf(shadow, 1.0 - smoothstep(1.0, 1.18, d2))
	return lerpf(t, 0.1, shadow * 0.8)


## A clump's edge as a few soft lobes (a share of its radius by angle), so the clumps are leafy, not drawn with a compass.
func _lobe(off: Vector2, phase: float) -> float:
	var a := atan2(off.y, off.x)
	return 1.0 + 0.07 * sin(a * 5.0 + phase) + 0.04 * sin(a * 9.0 + phase * 2.0)


## A wood tile's value at (u, v): see the `wood` kind.
func _wood(soft: FastNoiseLite, fine: FastNoiseLite, u: float, v: float) -> float:
	var plank := int(floor(v * 4.0))
	var pv := fposmod(v * 4.0, 1.0)
	var tone := 0.72 + 0.2 * fposmod(float(plank) * 0.618 + 0.3, 1.0)
	# A knot on two of the four planks; the grain bends round it.
	var bend := 0.0
	var knot := 0.0
	if plank == 1 or plank == 3:
		var kc := Vector2(0.3 if plank == 1 else 0.72, 0.5)
		var dk := Vector2((fposmod(u - kc.x + 0.5, 1.0) - 0.5) * 4.0, pv - kc.y)
		var r := dk.length()
		bend = 0.12 * exp(-r * r * 18.0) * signf(dk.y)
		knot = 1.0 - smoothstep(0.06, 0.1, Vector2(dk.x * 0.8, dk.y * 1.4).length())
		knot = maxf(knot, 0.5 * (1.0 - smoothstep(0.0, 0.03, absf(Vector2(dk.x * 0.8, dk.y * 1.4).length() - 0.14))))
	# Grain: lines along the plank, wavering slowly, a few darker; and long painted streaks of lighter and darker.
	var gy := (pv + bend + 0.05 * _tiled(soft, u, float(plank) * 0.25, 1.0, 1.0)) * 9.0 + float(plank) * 1.7
	var line := 1.0 - smoothstep(0.015, 0.06, absf(fposmod(gy + 0.5, 1.0) - 0.5))
	var streak := 0.5 + 0.5 * _tiled(fine, u * 0.35, pv * 3.0 + float(plank), 1.0, 1.0)
	var t := tone * (1.0 - 0.4 * line) * (0.62 + 0.46 * streak)
	t *= 1.0 - 0.45 * knot
	# The edges: a lit lip along the plank's top, shade toward its foot, a dark seam under it.
	t = lerpf(t, 1.0, 0.6 * (1.0 - smoothstep(0.0, 0.08, pv)))
	t *= 1.0 - 0.25 * smoothstep(0.78, 0.94, pv)
	t *= 1.0 - 0.75 * smoothstep(0.94, 0.965, pv)
	return t


## A paving tile's value at (u, v): see the `paving` kind. Three rows of slabs, each row offset, slabs of uneven length.
func _paving(soft: FastNoiseLite, fine: FastNoiseLite, u: float, v: float) -> float:
	var rows := 3.0
	var row := floorf(v * rows)
	var rv := fposmod(v * rows, 1.0)
	var cuts: Array = [[0.0, 0.38, 0.7], [0.18, 0.55, 0.82], [0.1, 0.45, 0.76]][int(row) % 3]
	var su := 0.0
	var sw := 1.0
	var slab := 0
	for c in 3:
		var a: float = cuts[c]
		var b: float = cuts[(c + 1) % 3] + (1.0 if c == 2 else 0.0)
		var uu := u if u >= a else u + 1.0
		if uu >= a and uu < b:
			su = (uu - a) / (b - a)
			sw = (b - a) * 3.0
			slab = c
	var tone := 0.72 + 0.2 * fposmod(row * 0.37 + float(slab) * 0.618, 1.0)
	var t := tone * (0.92 + 0.12 * (0.5 + 0.5 * _tiled(fine, u, v, 1.0, 1.0))) * (0.94 + 0.08 * _tiled(soft, u, v, 1.0, 1.0))
	# Bevels: lit along the slab's top and left, shaded along its bottom and right, a dark groove at the joint.
	var ex := 0.035 / sw
	var ey := 0.06
	if rv < ey or su < ex:
		t = lerpf(t, 1.0, 0.55)
	if rv > 1.0 - ey * 1.3 or su > 1.0 - ex * 1.3:
		t *= 0.68
	if rv < 0.02 or rv > 0.98 or su < 0.008 or su > 0.992:
		t *= 0.35
	return t


## A roofing tile's value at (u, v): see the `roofing` kind. Four sheets down the tile.
func _roofing(soft: FastNoiseLite, fine: FastNoiseLite, u: float, v: float) -> float:
	var sheets := 4.0
	var sv := fposmod(v * sheets, 1.0)
	var sheet := floorf(v * sheets)
	var tone := 0.8 + 0.12 * fposmod(sheet * 0.618, 1.0)
	var stain := 0.5 + 0.5 * _tiled(soft, u, v, 1.0, 1.0)
	var t := tone * (0.84 + 0.2 * stain)
	# Gravel speckle.
	var g := _tiled(fine, u * 3.0, v * 3.0, 3.0, 3.0)
	t *= 0.96 + 0.06 * signf(g) * smoothstep(0.3, 0.5, absf(g))
	# The lap: a lit edge where this sheet overlaps the next, a shadow just past it.
	if sv < 0.05:
		t = lerpf(t, 1.0, 0.5)
	elif sv < 0.11:
		t *= 0.72
	return t
