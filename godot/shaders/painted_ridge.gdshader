shader_type spatial;
// A painted backdrop layer drawn by the shader: a ridge (rolling hills, peaked mountains, flat-topped mesas or a bank of cloud) cut out
// against the sky on a flat card, filled with a soft gradient from its foot up to its ridge, its top edge caught by the light, and a light
// brush grain running along it, so it reads like a painted flat in a stage set (a cut-out). Unshaded, and out of the engine's depth fog (which would
// wash it out entirely this far back): it takes its own measured haze toward the horizon instead. Set up by scripts/stage_art.gd (`ridge`).
render_mode unshaded, cull_disabled, shadows_disabled, fog_disabled;

uniform vec4 top_colour : source_color = vec4(0.6, 0.7, 0.85, 1.0);
uniform vec4 foot_colour : source_color = vec4(0.5, 0.6, 0.78, 1.0);
uniform vec4 edge_colour : source_color = vec4(0.85, 0.9, 1.0, 1.0);
uniform float edge_amount = 0.5;
// 0 rolling hills, 1 peaked mountains, 2 flat-topped mesas, 3 a bank of cloud.
uniform int shape = 0;
uniform float seed = 0.0;
// Bumps across the card, the ridge's height range (0..1 of the card), and how soft its edge is.
uniform float frequency = 4.0;
uniform float base = 0.45;
uniform float amplitude = 0.35;
uniform float softness = 0.006;
uniform float brush = 0.06;
uniform float opacity = 1.0;
uniform vec4 haze_colour : source_color = vec4(0.8, 0.9, 1.0, 1.0);
uniform float haze = 0.3;

float hash(float n) {
	return fract(sin(n) * 43758.5453);
}

float noise1(float x) {
	float i = floor(x);
	float f = fract(x);
	f = f * f * (3.0 - 2.0 * f);
	return mix(hash(i), hash(i + 1.0), f);
}

float noise2(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	float a = hash(i.x + i.y * 57.0);
	float b = hash(i.x + 1.0 + i.y * 57.0);
	float c = hash(i.x + (i.y + 1.0) * 57.0);
	float d = hash(i.x + 1.0 + (i.y + 1.0) * 57.0);
	return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// The ridge's height (0..1) at x.
float ridge(float x) {
	float h = 0.0;
	float amp = 0.55;
	float freq = 1.0;
	for (int o = 0; o < 4; o++) {
		float n = noise1(x * freq + seed * 17.0 + float(o) * 31.0);
		if (shape == 1) {
			n = 1.0 - abs(n * 2.0 - 1.0);
			n = n * n;
		}
		h += n * amp;
		amp *= shape == 1 ? 0.5 : 0.42;
		freq *= 2.1;
	}
	if (shape == 2) {
		// Mesas: the tops pressed flat, the sides steep.
		h = smoothstep(0.38, 0.52, h) * 0.8 + h * 0.2;
	}
	if (shape == 3) {
		// Cloud: round puffs.
		h = 0.5 + 0.5 * sin(x * 3.1 + seed) * 0.3 + h * 0.6;
	}
	return h;
}

void fragment() {
	float x = UV.x * frequency;
	float y = 1.0 - UV.y;
	float h = base + amplitude * ridge(x);
	float cut = 1.0 - smoothstep(h - softness, h + softness, y);
	float k = clamp(y / max(h, 0.001), 0.0, 1.0);
	vec3 c = mix(foot_colour.rgb, top_colour.rgb, k);
	// The light catches the top of the ridge.
	c = mix(c, edge_colour.rgb, smoothstep(h - 0.045, h - 0.008, y) * edge_amount);
	// Brush grain along the ridge.
	float grain = noise2(vec2(x * 6.0 + noise2(vec2(x * 1.5, y * 3.0)) * 2.0, y * 55.0));
	c *= 1.0 + (grain - 0.5) * brush;
	// Thicker haze toward the foot, as the air near the ground is.
	c = mix(c, haze_colour.rgb, haze * (1.0 - 0.5 * k));
	ALBEDO = c;
	// Cut out rather than blended, so the card is drawn with the solid scenery and takes the finishing pass (ink, grade) like it.
	ALPHA = cut * opacity;
	ALPHA_SCISSOR_THRESHOLD = 0.5;
}
