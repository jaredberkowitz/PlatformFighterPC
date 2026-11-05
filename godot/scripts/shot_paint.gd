extends RefCounted
## Energy shots painted as the hit effects are (presentation only): fire, water, light, curses, the Exile's flares and beam, the
## Claws' blaster. Each is drawn into its projectile slot's painted-shape mesh (main.gd `proj_waves`, the Champion's wave's) in layers
## outside in (a dark rim, the colour, a paler band, a white-hot core), flickering on twos like the swing arcs and the waves, and
## pointed the way it flies. Until the props pass of 2026-11-04 every one of them was the one generic purple bolt.

## The painted shots by the move that fires them: Vector2i(class, move id) -> kind.
const PAINTED := {
	Vector2i(1, 13): "laser",  # the Claws' blaster
	Vector2i(3, 13): "fireball",  # Embers' fireball (bounces)
	Vector2i(3, 30): "water",  # Embers' water pump shot (extra move 2)
	Vector2i(10, 40): "water",  # the Settler's watering can (extra move 11)
	Vector2i(5, 13): "light",  # the Seer's quick light shots
	Vector2i(5, 14): "flame_burst",  # the Seer's flame burst, where it is set off
	Vector2i(8, 33): "star",  # the Bounder's landing stars (extra move 5)
	Vector2i(12, 14): "curse",  # the Phantom's curse bolt
	Vector2i(12, 36): "curse",  # and its partner's faster one (extra move 8)
	Vector2i(13, 14): "beam",  # the Exile's thin dark beam (the shadow orbs)
	Vector2i(13, 16): "barrier",  # the Exile's honeycomb barrier
	Vector2i(13, 31): "orb",  # the Exile's flare, bigger flare and giant flare (extra moves 3, 4 and 5)
	Vector2i(13, 32): "orb",
	Vector2i(13, 33): "orb",
	Vector2i(13, 35): "dark_burst",  # the Exile's ring burst (extra move 7)
}

## The Seer's flame burst grows to this radius (its last hit's) over this many frames.
const FLAME_BURST_MAX := 1.9
const FLAME_BURST_GROW := 24.0
## The flares' sizes by move: [orb radius, burst radius] (their burst hits reach 1.6, 2.5 and 3.8).
const ORB_SIZE := {31: [0.34, 1.6], 32: [0.42, 2.5], 33: [0.5, 3.8]}

## Layer colours, outside in.
const FIRE := [Color(0.34, 0.06, 0.04), Color(0.96, 0.3, 0.06), Color(1.0, 0.6, 0.1), Color(1.0, 0.88, 0.32), Color(1.0, 0.98, 0.86)]
const WATER := [Color(0.1, 0.2, 0.46), Color(0.26, 0.56, 0.96), Color(0.62, 0.86, 1.0), Color(1.0, 1.0, 1.0)]
const LIGHT := [Color(0.5, 0.34, 0.06), Color(1.0, 0.8, 0.26), Color(1.0, 0.95, 0.7), Color(1.0, 1.0, 0.96)]
const CURSE := [Color(0.07, 0.02, 0.1), Color(0.36, 0.08, 0.5), Color(0.78, 0.24, 0.78), Color(0.98, 0.84, 0.98)]
const DARK := [Color(0.92, 0.36, 0.86), Color(0.1, 0.03, 0.14), Color(0.26, 0.06, 0.32), Color(0.95, 0.7, 0.95)]
const LASER := [Color(0.2, 0.05, 0.3), Color(0.74, 0.26, 0.96), Color(0.96, 0.76, 1.0), Color(1.0, 1.0, 1.0)]
const STAR := [Color(0.5, 0.34, 0.04), Color(1.0, 0.84, 0.2), Color(1.0, 0.97, 0.72)]
const HEX := [Color(0.42, 0.3, 0.62, 0.95), Color(0.78, 0.7, 1.0, 0.32), Color(0.95, 0.9, 1.0, 0.9)]


static func kind_of(from: Array) -> String:
	if from.size() < 2:
		return ""
	return String(PAINTED.get(Vector2i(int(from[0]), int(from[1])), ""))


## Draws shot `kind` into `node` (an ImmediateMesh in the effects material) at `at`, flying `vel` (`dir` +1 right); `age` frames since
## it appeared, `bursting` while its burst runs (`move` picks the flare's size). Redrawn on twos only.
static func draw(node: MeshInstance3D, kind: String, at: Vector3, vel: Vector2, dir: float, age: int, bursting: bool, move: int) -> void:
	node.position = at
	node.scale = Vector3.ONE
	node.rotation = Vector3.ZERO
	var step := int(Time.get_ticks_msec() / 33)
	var key := "%s|%d|%s|%d" % [kind, step, bursting, age / 2]
	if node.get_meta("paint", "") == key:
		return
	node.set_meta("paint", key)
	# (main.gd's waves share the node and redraw only on their own step: mark it as no longer theirs.)
	node.set_meta("kind", "paint")
	var im: ImmediateMesh = node.mesh
	im.clear_surfaces()
	im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
	var ang := vel.angle() if vel.length() > 0.001 else (0.0 if dir >= 0.0 else PI)
	match kind:
		"fireball":
			_flame_ball(im, ang, 0.28, FIRE, step, 2.4)
		"curse":
			if bursting:
				_flames_up(im, 0.9, 1.2, CURSE, step, age)
			else:
				_flame_ball(im, ang, 0.27, CURSE, step, 2.6)
		"water":
			_water(im, ang, 0.3, step)
		"light":
			_light(im, ang, step)
		"flame_burst":
			# As big as its hit, which grows from 0.6 round to 1.9 over its life (the content's phases).
			var r := lerpf(0.6, FLAME_BURST_MAX, clampf(float(age) / FLAME_BURST_GROW, 0.0, 1.0))
			_flames_up(im, r * 1.6, r * 1.5, FIRE, step, age)
		"star":
			_star(im, float(step) * 0.5 * dir, 0.3)
		"laser":
			_streak(im, ang, 1.3, 0.085, LASER, step)
		"beam":
			_streak(im, ang, 1.7, 0.1, CURSE, step)
		"barrier":
			_barrier(im, dir, step)
		"orb":
			var size: Array = ORB_SIZE.get(move, [0.4, 2.0])
			if bursting:
				_ring_burst(im, float(size[1]), DARK, step, age)
			else:
				# A dark fireball: a magenta rim round black-violet fire, its tongues streaming back (the flare is a fireball).
				_flame_ball(im, ang, float(size[0]), [DARK[0], DARK[1], DARK[2], DARK[0]], step, 1.3)
		"dark_burst":
			_ring_burst(im, 2.2, DARK, step, age)
	im.surface_end()


static func _rot(p: Vector2, a: float) -> Vector3:
	var c := cos(a)
	var s := sin(a)
	return Vector3(p.x * c - p.y * s, p.x * s + p.y * c, 0.0)


## A filled shape round `centre` from its outline points (star-shaped about the centre), in one colour.
static func _fan(im: ImmediateMesh, centre: Vector3, pts: Array, colour: Color) -> void:
	for j in pts.size():
		var p0: Vector3 = pts[j]
		var p1: Vector3 = pts[(j + 1) % pts.size()]
		for v in [centre, p0, p1]:
			im.surface_set_color(colour)
			im.surface_add_vertex(v)


## A fireball (or a curse bolt) flying along `ang`: a round head with three pointed flame tongues streaming back, the longest
## `tail` head radii long, each layer smaller and set forward (the hot core leads), the tongues licking on twos.
static func _flame_ball(im: ImmediateMesh, ang: float, r: float, colours: Array, step: int, tail: float) -> void:
	var n := 36
	for li in colours.size():
		var k := 1.0 - 0.19 * float(li)
		var pts := []
		for j in n:
			var th := TAU * float(j) / n
			# Behind (th near PI) the outline draws out into three sharp tongues (peaked, not rounded), each its own length,
			# swapping on twos; in front it stays round.
			var back := clampf(-cos(th) * 1.25, 0.0, 1.0)
			var lick := 0.0
			for t in 3:
				var centre := PI + (float(t) - 1.0) * 0.62
				var near := maxf(0.0, 1.0 - absf(wrapf(th - centre, -PI, PI)) / 0.36)
				var length := 0.7 + 0.3 * sin(float(step) * 1.7 + float(t) * 2.4 + float(li))
				lick = maxf(lick, pow(near, 1.6) * length * (1.0 if t == 1 else 0.75))
			var rr := r * k * (1.0 + 0.12 * back + tail * back * lick)
			var shift := r * 0.2 * float(li)
			pts.append(_rot(Vector2(cos(th) * rr + shift, sin(th) * rr * 0.94), ang))
		_fan(im, _rot(Vector2(r * 0.2 * float(li), 0.0), ang), pts, colours[li])


## Flames rising from a base `w` wide, `h` tall, in layers; they flare up and die down over the shot's life.
static func _flames_up(im: ImmediateMesh, w: float, h: float, colours: Array, step: int, age: int) -> void:
	var tongues := 5
	for li in colours.size():
		var k := 1.0 - 0.18 * float(li)
		for t in tongues:
			var x := (float(t) / (tongues - 1) - 0.5) * w * 0.9
			var tall := h * k * (0.65 + 0.35 * sin(float(t) * 2.1 + float(step) * 0.9)) * (1.0 - 0.3 * absf(x) / maxf(w, 0.01))
			var half := w / tongues * 0.9 * k
			var sway := 0.12 * h * sin(float(step) * 1.1 + float(t) * 1.7)
			var pts := []
			var m := 10
			for j in m + 1:
				var u := float(j) / m
				# A teardrop: widest a quarter of the way up, to a point at the top, swaying.
				var wide := half * sin(PI * pow(u, 0.6)) * (1.0 - u * 0.35)
				pts.append(Vector3(x + wide + sway * u * u, -h * 0.35 + tall * u, 0.0))
			for j in range(m, -1, -1):
				var u := float(j) / m
				var wide := half * sin(PI * pow(u, 0.6)) * (1.0 - u * 0.35)
				pts.append(Vector3(x - wide + sway * u * u, -h * 0.35 + tall * u, 0.0))
			_fan(im, Vector3(x + sway * 0.1, -h * 0.35 + tall * 0.25, 0.0), pts, colours[li])


## A water blob flying along `ang`, with two droplets trailing and a white glint.
static func _water(im: ImmediateMesh, ang: float, r: float, step: int) -> void:
	var blobs := [[Vector2.ZERO, r], [Vector2(-r * 1.5, r * 0.25), r * 0.45], [Vector2(-r * 2.3, -r * 0.2), r * 0.3]]
	for li in 3:
		var k := 1.0 - 0.22 * float(li)
		for b in blobs:
			var c: Vector2 = b[0] + Vector2(r * 0.08 * li, r * 0.1 * li)
			var rr: float = float(b[1]) * k
			var pts := []
			for j in 20:
				var th := TAU * float(j) / 20.0
				var wob := 1.0 + 0.06 * sin(th * 3.0 + float(step) * 1.4)
				pts.append(_rot(c + Vector2(cos(th), sin(th)) * rr * wob, ang))
			_fan(im, _rot(c, ang), pts, WATER[li])
	# The glint: a small white oval up and forward.
	var g := []
	for j in 12:
		var th := TAU * float(j) / 12.0
		g.append(_rot(Vector2(r * 0.25 + cos(th) * r * 0.22, r * 0.38 + sin(th) * r * 0.12), ang))
	_fan(im, _rot(Vector2(r * 0.25, r * 0.38), ang), g, WATER[3])


## A quick shot of light flying along `ang`: a long pointed diamond with a four-pointed glint at its head.
static func _light(im: ImmediateMesh, ang: float, step: int) -> void:
	var length := 0.95
	for li in LIGHT.size():
		var k: float = [1.0, 0.86, 0.5, 0.26][li]
		var half_w := 0.17 * k * (1.0 + 0.1 * sin(float(step) * 2.0))
		var pts := [_rot(Vector2(length * 0.45, 0.0), ang), _rot(Vector2(length * 0.1, half_w), ang),
			_rot(Vector2(-length * 0.55 * k, 0.0), ang), _rot(Vector2(length * 0.1, -half_w), ang)]
		_fan(im, _rot(Vector2(length * 0.1, 0.0), ang), pts, LIGHT[li])
	var glint := []
	for j in 8:
		var th := TAU * float(j) / 8.0 + float(step) * 0.3
		var rr := 0.24 if j % 2 == 0 else 0.06
		glint.append(_rot(Vector2(length * 0.4, 0.0) + Vector2(cos(th), sin(th)) * rr, ang))
	_fan(im, _rot(Vector2(length * 0.4, 0.0), ang), glint, LIGHT[2])


## A five-pointed star `r` round, turned `spin`, in layers.
static func _star(im: ImmediateMesh, spin: float, r: float) -> void:
	for li in STAR.size():
		var k := 1.0 - 0.25 * float(li)
		var pts := []
		for j in 10:
			var th := spin + TAU * float(j) / 10.0 + PI * 0.5
			var rr := r * k * (1.0 if j % 2 == 0 else 0.45)
			pts.append(Vector3(cos(th) * rr, sin(th) * rr, 0.0))
		_fan(im, Vector3.ZERO, pts, STAR[li])


## A long thin bolt along `ang` (`length` long, `half_h` thick): a laser or a dark beam, pointed at both ends, flickering.
static func _streak(im: ImmediateMesh, ang: float, length: float, half_h: float, colours: Array, step: int) -> void:
	for li in colours.size():
		var k := 1.0 - 0.22 * float(li)
		var n := 10
		var top := []
		var bottom := []
		for j in n + 1:
			var u := float(j) / n
			var thick := half_h * k * pow(sin(PI * u), 0.5) * (1.0 + 0.12 * sin(float(step) * 2.3 + u * 11.0))
			var x := (u - 0.5) * length * (0.8 + 0.2 * k)
			top.append(_rot(Vector2(x, thick), ang))
			bottom.append(_rot(Vector2(x, -thick), ang))
		var pts := top.duplicate()
		bottom.reverse()
		pts.append_array(bottom)
		_fan(im, Vector3.ZERO, pts, colours[li])


## A burst of dark fire `radius` round: a ring that swells out from the middle and thins as it goes, magenta at its edge.
static func _ring_burst(im: ImmediateMesh, radius: float, colours: Array, step: int, age: int) -> void:
	var t := clampf(float(age) / 14.0, 0.0, 1.0)
	var outer := radius * (0.45 + 0.55 * (1.0 - pow(1.0 - t, 2.0)))
	var width := radius * 0.42 * (1.0 - 0.6 * t)
	var n := 40
	for band in [[outer, outer - width, colours[0]], [outer - width * 0.12, outer - width * 0.88, colours[1]],
			[outer - width * 0.3, outer - width * 0.7, colours[2]]]:
		var ro: float = band[0]
		var ri: float = maxf(float(band[1]), 0.0)
		for j in n:
			var a0 := TAU * float(j) / n
			var a1 := TAU * float(j + 1) / n
			var wob0 := 1.0 + 0.05 * sin(a0 * 6.0 + float(step) * 1.5)
			var wob1 := 1.0 + 0.05 * sin(a1 * 6.0 + float(step) * 1.5)
			var d0 := Vector3(cos(a0), sin(a0), 0.0)
			var d1 := Vector3(cos(a1), sin(a1), 0.0)
			for v in [d0 * ro * wob0, d1 * ro * wob1, d1 * ri * wob1, d0 * ro * wob0, d1 * ri * wob1, d0 * ri * wob0]:
				im.surface_set_color(band[2])
				im.surface_add_vertex(v)


## The Exile's honeycomb barrier in front of it: a stand of hexagonal cells, see-through lilac with pale rims, shimmering.
static func _barrier(im: ImmediateMesh, dir: float, step: int) -> void:
	var r := 0.3
	var cols := 3
	var rows := 5
	for c in cols:
		for row in rows:
			# (Centred on the shot: its catching hit is a capsule round this point.)
			var x := dir * (float(c) - float(cols - 1) * 0.5) * r * 1.5
			var y := (float(row) - (rows - 1) * 0.5) * r * 1.73 + (r * 0.866 if c % 2 == 1 else 0.0)
			var shimmer := 0.85 + 0.15 * sin(float(step) * 0.9 + float(c + row) * 1.3)
			var centre := Vector3(x, y, 0.0)
			for li in 2:
				var rr := r * (0.98 if li == 0 else 0.8)
				var pts := []
				for j in 6:
					var th := TAU * float(j) / 6.0
					pts.append(centre + Vector3(cos(th), sin(th), 0.0) * rr)
				var col: Color = HEX[li]
				_fan(im, centre, pts, Color(col.r, col.g, col.b, col.a * shimmer))
