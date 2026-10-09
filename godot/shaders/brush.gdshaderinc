// A hand-painted texture for cel surfaces: fine strokes running along the surface (like the grain of a textured brush), wavering with a
// soft noise, with a few lighter flecks. `brush(uv)` returns a multiplier near 1 for the colour.
uniform float brush_strength = 0.09;
uniform float brush_scale = 1.0;

float brush_hash(vec2 p) {
	p = fract(p * vec2(123.34, 456.21));
	p += dot(p, p + 45.32);
	return fract(p.x * p.y);
}

float brush_noise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	return mix(mix(brush_hash(i), brush_hash(i + vec2(1.0, 0.0)), f.x), mix(brush_hash(i + vec2(0.0, 1.0)), brush_hash(i + vec2(1.0, 1.0)), f.x), f.y);
}

float brush(vec2 uv) {
	if (brush_strength <= 0.0 || (uv.x == 0.0 && uv.y == 0.0)) {
		return 1.0;
	}
	vec2 p = uv * brush_scale;
	float warp = brush_noise(p * vec2(4.0, 9.0)) * 2.2 + brush_noise(p * vec2(11.0, 3.0)) * 0.8;
	float lines = sin((p.x * 46.0 + warp) * 6.2831853);
	float stroke = smoothstep(0.55, 1.0, lines) * (0.55 + 0.45 * brush_noise(p * vec2(20.0, 5.0)));
	float fleck = smoothstep(0.82, 1.0, brush_noise(p * vec2(38.0, 26.0)));
	return 1.0 - brush_strength * stroke + brush_strength * 0.6 * fleck;
}
