extends SceneTree
## Paints a seamless greyscale value tile from noise, in the same format as tools/art/mm_tile.gd (the stage colour is multiplied by it,
## so a tile serves every hue), for surfaces no Material Maker export covers:
##   bark: tree bark as soft vertical plates parted by wide, gentle crevices and broken into lengths, no thin dark streaks
##         (the owner, 2026-10-15: the old bark's sharp brown streaks read as nothing real and distracted);
##   palm: a palm trunk's horizontal ring scars, each ring a soft lip of light over a band of shade, wobbling a little round the trunk;
##   leaves: overlapping leaf clumps like scales, each lit along its top edge and deeper toward its foot, casting a soft crescent of
##         shade on the clump behind it (the owner, 2026-10-17: the grove's crowns read as uniform green blobs of embossed dimples).
## Run: Godot --headless -s tools/art/noise_tile.gd -- <bark|palm|leaves> <out.png> [seed]

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
			# A few flat values with soft steps between them (the same painted read as mm_tile), then the multiplier range, in sRGB.
			t = lerpf(t, roundf(t * 3.0) / 3.0, 0.4)
			# (Bark spans a deeper range: at the rings' 0.66 its plates all but vanished on the grove's great trunk.)
			var value := lerpf({"bark": 0.42, "leaves": 0.6}.get(kind, 0.66), 1.0, clampf(t, 0.0, 1.0))
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
