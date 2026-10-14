extends RefCounted

const Particles := preload("res://scripts/particles.gd")
const FxMaterial := preload("res://scripts/fx_material.gd")
## Short-lived hit and movement effects (presentation only): a drawn burst or slash where a hit lands, by what landed it, a ring under the
## feet on a jump and a midair jump, and so on. Each effect is a node that animates itself for its short life and then frees itself.
## They animate on twos (30 pictures a second, `STEP`) like hand-drawn effects, while the game runs at 60, and are laid on in brush
## strokes that break up into ragged chunks as they go (`paint` in shaders/fx.gdshaderinc) rather than fading out like vector shapes.

const STEP := 1.0 / 30.0
const INK := Color(0.12, 0.07, 0.12, 1.0)


static func _material(on_top: bool) -> ShaderMaterial:
	return FxMaterial.get_material(on_top, 6 if on_top else 0, null, 1.0, 1.0)



## A ring that expands and thins out as it fades, drawn in its own XY plane (facing the camera unless the node is turned).
class Ring extends MeshInstance3D:
	var age := 0.0
	var life := 0.3
	var from := 0.3
	var to := 1.6
	var thickness := 0.25
	var colour := Color.WHITE
	var inked := true

	func _ready() -> void:
		mesh = ImmediateMesh.new()
		cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF

	func _process(delta: float) -> void:
		age += FxMaterial.delta(delta)
		if age >= life:
			queue_free()
			return
		var t := floorf(age * 30.0) / 30.0 / life
		var eased := 1.0 - pow(1.0 - t, 3.0)
		var r := lerpf(from, to, eased)
		var w := thickness * (1.0 - t)
		var im: ImmediateMesh = mesh
		im.clear_surfaces()
		im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
		var fade := colour.a * (1.0 - t * t)
		var c := Color(colour.r, colour.g, colour.b, fade)
		# Painted as the other effects are: a thin ink line along the outer and inner edges with the colour between (`inked`), side by side
		# so a see-through ring does not show ink through its colour.
		var ink_w := minf(0.04 + w * 0.12, w * 0.3) if inked else 0.0
		var ink := Color(INK, fade)
		var bands := [[r + ink_w, r, ink], [r, maxf(r - w, 0.0), c], [maxf(r - w, 0.0), maxf(r - w - ink_w, 0.0), ink]] if inked else [[r, maxf(r - w, 0.0), c]]
		var n := 40
		for band in bands:
			var ro: float = band[0]
			var ri: float = band[1]
			if ro - ri < 0.001:
				continue
			for k in n:
				var a0 := TAU * k / n
				var a1 := TAU * (k + 1) / n
				var d0 := Vector3(cos(a0), sin(a0), 0)
				var d1 := Vector3(cos(a1), sin(a1), 0)
				for v in [d0 * ro, d1 * ro, d1 * ri, d0 * ro, d1 * ri, d0 * ri]:
					im.surface_set_color(band[2])
					im.surface_add_vertex(v)
		im.surface_end()


## Thin spikes flying out in every direction, shrinking as they go.
class Streaks extends MeshInstance3D:
	var age := 0.0
	var life := 0.22
	var count := 8
	var reach := 1.6
	var colour := Color.WHITE
	var plain := false
	var angles: Array = []

	func _ready() -> void:
		mesh = ImmediateMesh.new()
		cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		for k in count:
			angles.append(TAU * (float(k) + randf_range(-0.3, 0.3)) / count)

	func _process(delta: float) -> void:
		age += FxMaterial.delta(delta)
		if age >= life:
			queue_free()
			return
		var t := floorf(age * 30.0) / 30.0 / life
		var im: ImmediateMesh = mesh
		im.clear_surfaces()
		im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
		var fade := colour.a * (1.0 - t)
		# Painted in layers as the knock-out blast is (an ink sliver, the colour, a thin white core), unless `plain` (a flat white flash).
		var layers := [[1.0, 1.0, Color(colour, fade)]] if plain else [[1.9, 1.05, Color(INK, fade)], [1.0, 1.0, Color(colour, fade)],
			[0.4, 0.86, Color(1.0, 1.0, 0.95, fade)]]
		for layer in layers:
			for a in angles:
				var d := Vector3(cos(a), sin(a), 0)
				var side := Vector3(-d.y, d.x, 0)
				var near := d * reach * (0.25 + 0.75 * t)
				var far := d * reach * (0.6 + 0.6 * t) * float(layer[1])
				var w := (0.05 + 0.03 * reach) * (1.0 - t) * float(layer[0])
				for v in [near - d * w * 0.5 + side * w, far, near - d * w * 0.5 - side * w]:
					im.surface_set_color(layer[2])
					im.surface_add_vertex(v)
		im.surface_end()


## What landed a hit, which picks its look (and sound): a blade (the sword: a cool blue-white burst), a blow (punches and kicks: a warm
## burst), fire, or claws (a lilac burst). A weapon's cut or rake is shown by the swing's own arc (`ARCS` in fighter_view.gd), not here.
enum Kind { BLADE, BLOW, FIRE, CLAW }

## Layer colours of a burst by kind, outside in (under an ink outline): the outer flare, the body, the core.
## (As the knock-out blast: the flare and body stay below the effects' glow threshold (luma 0.72, shaders/fx.gdshaderinc), so they keep
## their colour and only the white core blooms. Brighter bodies bloomed to white and the burst lost its ink and colour.)
const BURST_COLOURS := {
	Kind.BLADE: [Color(0.22, 0.46, 0.95), Color(0.45, 0.72, 1.0), Color(1.0, 1.0, 1.0)],
	Kind.CLAW: [Color(0.52, 0.22, 0.95), Color(0.78, 0.5, 1.0), Color(1.0, 0.96, 1.0)],
	Kind.BLOW: [Color(0.96, 0.4, 0.1), Color(1.0, 0.66, 0.16), Color(1.0, 1.0, 0.92)],
	Kind.FIRE: [Color(0.88, 0.12, 0.06), Color(1.0, 0.5, 0.08), Color(1.0, 0.9, 0.4)],
}


## A drawn hit burst, the reference game's hit flash in our ink style: a spiky star in layers (an ink outline, the outer flare, the body and
## a white core) that pops out in a frame with a little overshoot, holds, then breaks up, the spikes pulling in and the outer layers going
## first. It is stretched a little along `dir` (where the hit sends the fighter).
class Pow extends MeshInstance3D:
	var age := 0.0
	var life := 0.2
	var size := 1.0
	var dir := Vector2.RIGHT
	var colours: Array = []
	var spikes: Array = []     # [angle, length 0..1]

	func _ready() -> void:
		mesh = ImmediateMesh.new()
		cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		extra_cull_margin = 64.0
		var n := 11
		var turn := randf() * TAU
		for k in n:
			var a := turn + TAU * (float(k) + randf_range(-0.22, 0.22)) / n
			spikes.append([a, randf_range(0.62, 1.0) if k % 2 == 0 else randf_range(0.45, 0.75)])

	func _process(delta: float) -> void:
		age += FxMaterial.delta(delta)
		if age >= life:
			queue_free()
			return
		var t := floorf(age * 30.0) / 30.0 / life
		var pop := lerpf(0.55, 1.18, t / 0.15) if t < 0.15 else (lerpf(1.18, 1.0, (t - 0.15) / 0.15) if t < 0.3 else 1.0)
		var breakup := smoothstep(0.5, 1.0, t)
		var im: ImmediateMesh = mesh
		im.clear_surfaces()
		# Ink, flare, body, core: each layer smaller, the outer ones leaving first as it breaks up.
		var layers := [[1.0, INK, 0.0], [0.9, colours[0], 0.15], [0.66, colours[1], 0.3], [0.38, colours[2], 0.45]]
		var shown := []
		for layer in layers:
			var c: Color = layer[1]
			c.a *= 1.0 - clampf(breakup * 1.6 - float(layer[2]), 0.0, 1.0)
			if c.a > 0.01:
				shown.append([layer[0], c])
		# (Once every layer has gone there is nothing to draw: an empty surface is an error.)
		if shown.is_empty():
			return
		im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
		for layer in shown:
			_star(im, size * float(layer[0]) * pop * (1.0 - 0.35 * breakup), layer[1], 1.0 - 0.55 * breakup)
		im.surface_end()

	## One layer: a fan of spikes round the middle, `reach` scaling how far the spikes go beyond the round inner part.
	func _star(im: ImmediateMesh, r: float, c: Color, reach: float) -> void:
		var inner := r * 0.36
		var along := Vector3(dir.x, dir.y, 0).normalized()
		var across := Vector3(-along.y, along.x, 0)
		var n := spikes.size()
		for i in n:
			var a0: float = spikes[i][0]
			var a1: float = spikes[(i + 1) % n][0]
			if a1 < a0:
				a1 += TAU
			var mid := (a0 + a1) * 0.5
			var tip := inner + (r - inner) * float(spikes[i][1]) * reach
			var p_tip := _at(a0, tip, along, across)
			var p_mid := _at(mid, inner, along, across)
			var p_before := _at(a0 - (a1 - a0) * 0.5, inner, along, across)
			for v in [Vector3.ZERO, p_before, p_tip, Vector3.ZERO, p_tip, p_mid]:
				im.surface_set_color(c)
				im.surface_add_vertex(v)

	func _at(angle: float, radius: float, along: Vector3, across: Vector3) -> Vector3:
		# Stretched 1.3 times along the hit and pinched across it.
		return along * cos(angle) * radius * 1.3 + across * sin(angle) * radius * 0.85


## A hit landing at `at` (where the hit met the fighter): the drawn effect for what landed it (`kind`, see `Kind`), stretched along `dir`
## (where the hit sends the fighter), sparks, and for a strong hit a crisp shockwave ring. Bigger and longer for a stronger hit (`strength`
## 0..1); `colour` is the hitter's (the sparks).
static func hit(parent: Node, at: Vector3, dir: Vector2, color: Color, strength: float, kind: int = Kind.BLOW) -> void:
	var s := clampf(strength, 0.0, 1.0)
	var pow_fx := Pow.new()
	pow_fx.material_override = _material(true)
	pow_fx.colours = BURST_COLOURS.get(kind, BURST_COLOURS[Kind.BLOW])
	pow_fx.dir = dir if dir != Vector2.ZERO else Vector2.RIGHT
	# Small and quick for a light hit, big for a strong one (so the big ones stay special and a jab never hides the fighter); even the
	# strongest stops short of covering the fighter it hit.
	# (A light and a medium hit start bigger than before, so a mid-strength hit reads at play distance; the strongest stays the same size.)
	pow_fx.size = (0.5 + 0.6 * s) * (0.8 if kind == Kind.BLADE or kind == Kind.CLAW else 1.0)
	pow_fx.life = 0.17 + 0.1 * s
	pow_fx.position = at + Vector3(0, 0, 0.05)
	parent.add_child(pow_fx)
	if s > 0.5:
		var ring := Ring.new()
		ring.material_override = _material(true)
		ring.colour = Color(1, 1, 0.95, 0.9)
		ring.from = 0.4
		ring.to = 1.1 + 1.0 * s
		ring.thickness = 0.07
		ring.life = 0.2
		ring.position = at
		parent.add_child(ring)
	# A strong hit bursts: speed lines flung out round it and a ring of smoke balls.
	if s > 0.45:
		var lines := Streaks.new()
		lines.material_override = _material(true)
		lines.colour = Color((pow_fx.colours[0] as Color), 0.95)
		lines.count = 10
		lines.reach = 1.6 + 1.6 * s
		lines.life = 0.2
		lines.position = at
		parent.add_child(lines)
		Particles.impact_smoke(parent, at, s)
	if kind == Kind.FIRE:
		Particles.embers(parent, at, s)
	Particles.sparks(parent, at, color, s * 0.7, dir)


## The directional impact of a hit: a sharp white spike with a coloured edge thrust along `dir` (where the hit sends the fighter) and a
## shorter one behind, snapping out and shrinking; longer for a harder hit.
class Spike extends MeshInstance3D:
	var age := 0.0
	var life := 0.14
	var length := 2.4
	var width := 0.5
	var dir := Vector2(1, 0)
	var core := Color.WHITE
	var edge := Color.WHITE

	func _ready() -> void:
		mesh = ImmediateMesh.new()
		cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF

	func _process(delta: float) -> void:
		age += FxMaterial.delta(delta)
		if age >= life:
			queue_free()
			return
		var t := floorf(age * 30.0) / 30.0 / life
		var grow := 1.0 - pow(1.0 - minf(1.0, t * 2.5), 2.0)
		var shrink := 1.0 - t
		var d := Vector3(dir.x, dir.y, 0).normalized()
		var side := Vector3(-d.y, d.x, 0)
		var im: ImmediateMesh = mesh
		im.clear_surfaces()
		im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
		for spec in [[1.08, INK, 1.25], [1.0, edge, 1.0], [1.0, core, 0.45], [-0.5, INK, 1.0], [-0.45, edge, 0.8], [-0.45, core, 0.35]]:
			var reach: float = length * float(spec[0]) * grow
			var w: float = width * float(spec[2]) * shrink
			var c: Color = spec[1]
			c.a *= shrink
			for v in [side * w, d * reach, -side * w]:
				im.surface_set_color(c)
				im.surface_add_vertex(v)
		im.surface_end()


static func impact(parent: Node, at: Vector3, dir: Vector2, color: Color, strength: float) -> void:
	var s := clampf(strength, 0.0, 1.0)
	var spike := Spike.new()
	spike.material_override = _material(true)
	spike.dir = dir
	# From a short dart on a medium hit to a long spear on a strong one.
	spike.length = 0.9 + 3.3 * s
	spike.width = 0.24 + 0.46 * s
	spike.life = 0.1 + 0.08 * s
	spike.edge = Color(color.r, color.g, color.b, 0.95)
	spike.core = Color(1, 1, 0.95, 1)
	spike.position = at + Vector3(0, 0, 0.05)
	parent.add_child(spike)


## The line a strong launch flies along, as DI left it: a long, thin blue streak from the fighter, there for a moment as the launch
## begins (the reference game's DI indicator).
static func launch_line(parent: Node, at: Vector3, dir: Vector2) -> void:
	var spike := Spike.new()
	spike.material_override = _material(true)
	spike.dir = dir
	spike.length = 4.2
	spike.width = 0.16
	spike.life = 0.26
	spike.edge = Color(0.35, 0.65, 1.0, 0.9)
	spike.core = Color(0.85, 0.95, 1.0, 1.0)
	spike.position = at
	parent.add_child(spike)


## A knock-out's blast: a plume of painted smoke in the player's colour exploding from where the fighter left the stage back across the
## screen, drawn in its own XY plane along its +Y (turn the node to aim it). A row of overlapping round puffs, widest at the blast and
## tapering away, each in layers (an ink silhouette round the whole plume, the colour, a lighter top lit from above and a white-hot core
## near the blast), redrawn on twos so the puffs swell, roll and drift outward. It bursts open in a few frames, holds, then the puffs
## thin and break up (the painted material's ragged breakup).
class Beam extends MeshInstance3D:
	var age := 0.0
	var life := 0.9
	var length := 26.0
	var width := 2.6
	var colour := Color.WHITE
	var _step := -1
	var _seed := randf() * 100.0

	func _ready() -> void:
		mesh = ImmediateMesh.new()
		cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
		extra_cull_margin = 64.0

	func _process(delta: float) -> void:
		age += FxMaterial.delta(delta)
		if age >= life:
			queue_free()
			return
		var step := int(age * 30.0)
		if step == _step:
			return
		_step = step
		var t := float(step) / 30.0 / life
		var open := 1.0 - pow(1.0 - minf(1.0, t / 0.12), 3.0)
		var fade := 1.0 - smoothstep(0.4, 1.0, t)
		var im: ImmediateMesh = mesh
		im.clear_surfaces()
		im.surface_begin(Mesh.PRIMITIVE_TRIANGLES)
		# (The body in a deeper, purer version of the colour, so the white core's glow does not wash it out.)
		var body := Color.from_hsv(colour.h, minf(1.0, colour.s * 1.35), colour.v * 0.92)
		var light := body.lerp(Color.WHITE, 0.22)
		var n := 28
		var puffs: Array = []
		for k in n:
			var u := float(k) / float(n - 1)
			var wob := sin(_seed + k * 2.3)
			var r := width * lerpf(1.0, 0.3, u) * (0.8 + 0.3 * absf(sin(_seed * 0.7 + k * 1.7))) * open * (1.0 + 0.35 * t)
			var y := u * length * open + t * 2.0 * (1.0 - u)
			puffs.append([Vector3(wob * width * 0.22, y, 0), r * lerpf(1.0, 0.6, smoothstep(0.5, 1.0, t)), u])
		# Layers back to front: the ink round everything, the colour, the lit tops (shifted up), the hot core (near the blast only).
		var layers := [[1.0, Vector3.ZERO, Color(INK, fade), 1.0], [0.88, Vector3.ZERO, Color(body, fade), 1.0],
			[0.6, Vector3(-0.1, 0.32, 0), Color(light, fade), 1.0], [0.3, Vector3(0.0, 0.05, 0), Color(1, 1, 0.95, fade), 0.3]]
		for layer in layers:
			for p in puffs:
				if float(p[2]) > float(layer[3]):
					continue
				var r: float = float(p[1]) * float(layer[0])
				_disc(im, (p[0] as Vector3) + (layer[1] as Vector3) * float(p[1]), r, layer[2])
		im.surface_end()

	func _disc(im: ImmediateMesh, c: Vector3, r: float, col: Color) -> void:
		var seg := 18
		for k in seg:
			var a0 := TAU * k / seg
			var a1 := TAU * (k + 1) / seg
			for v in [c, c + Vector3(cos(a0), sin(a0), 0) * r, c + Vector3(cos(a1), sin(a1), 0) * r]:
				im.surface_set_color(col)
				im.surface_add_vertex(v)


## A hit on a shield: a bright ring on the bubble and a few sparks.
static func shield_hit(parent: Node, at: Vector3, strength: float) -> void:
	var ring := Ring.new()
	ring.material_override = _material(true)
	ring.colour = Color(0.75, 0.92, 1.0, 0.9)
	ring.from = 0.6
	ring.to = 1.5 + 0.6 * strength
	ring.thickness = 0.22
	ring.life = 0.2
	ring.position = at
	parent.add_child(ring)
	Particles.sparks(parent, at, Color(0.6, 0.85, 1.0), strength * 0.5)


## The star that flashes when a fast fall starts.
static func sparkle(parent: Node, at: Vector3) -> void:
	var streaks := Streaks.new()
	streaks.material_override = _material(true)
	streaks.colour = Color(0.8, 0.9, 1.0, 1.0)
	streaks.count = 4
	streaks.reach = 0.7
	streaks.life = 0.16
	streaks.position = at
	parent.add_child(streaks)
	var ring := Ring.new()
	ring.material_override = _material(true)
	ring.colour = Color(0.85, 0.75, 1.0, 0.9)
	ring.from = 0.05
	ring.to = 0.45
	ring.thickness = 0.12
	ring.life = 0.18
	ring.position = at
	parent.add_child(ring)


## A jump: a soft ring spreading over the ground at the feet; `air` makes it the midair jump's ring, a little bigger and brighter.
static func jump_ring(parent: Node, feet: Vector3, air: bool) -> void:
	var ring := Ring.new()
	ring.material_override = _material(false)
	ring.colour = Color(1, 1, 1, 0.85 if air else 0.6)
	ring.from = 0.3
	ring.to = 1.3 if air else 1.1
	ring.thickness = 0.16
	ring.life = 0.3
	ring.position = feet + Vector3(0, 0.05, 0)
	# Laid flat (the ring is drawn in its own XY plane).
	ring.rotation_degrees = Vector3(-90, 0, 0)
	parent.add_child(ring)
