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
// The light on the ridge's slopes (0 none): each peak lit on the side toward the stage's sun (`pf_sun_dir`) and turned into a cool shade
// on the other, the way a painted range has a lit flank and a shadow flank, fading toward the hazy foot. `aspect` is the card's height
// over its width, so the slopes are measured as they look.
global uniform vec3 pf_sun_dir;
global uniform vec4 pf_shadow_tint;
uniform float flank = 0.0;
uniform float aspect = 0.1;
// Paper layers (0 off): the body cut into this many bands up from the foot, each a flat step lighter toward the peak with a lit edge on
// top, following the ridge's own shape, as layered paper mountains are (the owner's style reference). `flank_from` is where the flank
// shade starts (a share of the height; 0.2 left a pale lower half that read as a flat cloud band across the range).
uniform int layers = 0;
uniform float flank_from = 0.2;
// Ridges and gullies (0 off): the finer crags' slopes carried down the face as narrow lit and shaded facets, each running from a crag on
// the ridge line down and outward and fading as it goes, so a range reads as folded rock under one light, not a flat band (art review 7).
uniform float gullies = 0.0;

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

void fragment() {
	float x = UV.x * frequency;
	float y = 1.0 - UV.y;
	float h = base + amplitude * ridge(x);
	float cut = 1.0 - smoothstep(h - softness, h + softness, y);
	float k = clamp(y / max(h, 0.001), 0.0, 1.0);
	float kk = k;
	float layer_edge = 0.0;
	if (layers > 0) {
		float f = k * float(layers);
		kk = floor(f) / float(max(layers - 1, 1));
		// A thin lit lip at the top of each band.
		layer_edge = smoothstep(0.86, 0.97, fract(f)) * (1.0 - step(float(layers) - 0.5, f));
	}
	vec3 c = mix(foot_colour.rgb, top_colour.rgb, clamp(kk, 0.0, 1.0));
	c = mix(c, c * 1.18 + 0.04, layer_edge * 0.8);
	if (flank > 0.0) {
		// (The slope of the broad shape only: the fine crags flipped it column by column into stripes.)
		float e = 0.03;
		float slope = (ridge_broad(x + e) - ridge_broad(x - e)) / (2.0 * e) * amplitude * frequency * aspect;
		vec2 nrm = normalize(vec2(-slope, 1.0));
		vec2 sun = normalize(vec2(pf_sun_dir.x, max(pf_sun_dir.y, 0.15) * 0.6));
		float lit = smoothstep(-0.03, 0.03, dot(nrm, sun) - dot(vec2(0.0, 1.0), sun));
		vec3 shade = c * mix(vec3(1.0), pf_shadow_tint.rgb, 0.85) * 0.78;
		// (Down from the ridge line only so far: a flank's shade belongs to the peak, not a column to the foot.)
		float reach = flank * smoothstep(flank_from, flank_from + 0.5, y / max(h, 0.001));
		c = mix(c, mix(shade, c * 1.08, lit), reach);
	}
	if (gullies > 0.0) {
		// Each point looks up the slope to the ridge line, a little sideways the deeper it is (the spurs fan out from the crags), and
		// takes that point's fine slope: facing the sun lit, facing away shaded, in crisp painted steps.
		float depth_k = 1.0 - clamp(y / max(h, 0.001), 0.0, 1.0);
		float xs = x + (noise1(x * 3.0 + seed) - 0.5) * depth_k * 0.35;
		float e2 = 0.03;
		float fine = (ridge(xs + e2) - ridge(xs - e2)) - (ridge_broad(xs + e2) - ridge_broad(xs - e2));
		float side = sign(pf_sun_dir.x + 0.0001);
		float face = fine * side / e2 * amplitude * aspect;
		float fade = smoothstep(0.0, 0.12, 1.0 - depth_k) * (1.0 - smoothstep(0.55, 0.95, depth_k));
		vec3 shade2 = c * mix(vec3(1.0), pf_shadow_tint.rgb, 0.9) * 0.68;
		c = mix(c, shade2, smoothstep(0.08, 0.16, -face) * fade * gullies);
		c = mix(c, c * 1.15 + 0.04, smoothstep(0.08, 0.16, face) * fade * gullies * 0.8);
	}
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
