// The painted ridge's shape, shared by the card (shaders/painted_ridge.gdshader) and the shadow it casts on the layer behind
// (shaders/painted_ridge_shadow.gdshader), so the two always match.
// 0 rolling hills, 1 peaked mountains, 2 flat-topped mesas, 3 a bank of cloud.
uniform int shape = 0;
uniform float seed = 0.0;
// Bumps across the card, the ridge's height range (0..1 of the card), and how soft its edge is.
uniform float frequency = 4.0;
uniform float base = 0.45;
uniform float amplitude = 0.35;

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

// The ridge's broad shape only (its first two octaves), for the light on its flanks: one lit side and one shaded side per big peak.
float ridge_broad(float x) {
	float h = 0.0;
	float amp = 0.55;
	float freq = 1.0;
	for (int o = 0; o < 2; o++) {
		float n = noise1(x * freq + seed * 17.0 + float(o) * 31.0);
		if (shape == 1) {
			n = 1.0 - abs(n * 2.0 - 1.0);
			n = n * n;
		}
		h += n * amp;
		amp *= shape == 1 ? 0.5 : 0.42;
		freq *= 2.1;
	}
	return h;
}
