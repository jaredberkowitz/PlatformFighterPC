shader_type sky;
// The stages' painted sky (scripts/stage_art.gd `sky`): four bands from the zenith down to the horizon (top, a middle tint, the horizon,
// and the ground below it), a soft glow low over the horizon behind the stage in its own warm colour, and faint painted wisps lit by
// that glow, so a day sky can carry the violets, pinks and peaches of a magic-hour painting rather than one blue fade. Static (no
// TIME), so the sky's lighting is computed once.

uniform vec4 top_color : source_color = vec4(0.32, 0.55, 0.9, 1.0);
uniform vec4 mid_color : source_color = vec4(0.56, 0.72, 0.94, 1.0);
uniform vec4 horizon_color : source_color = vec4(0.8, 0.92, 0.98, 1.0);
uniform vec4 ground_color : source_color = vec4(0.64, 0.74, 0.78, 1.0);
// Where the middle tint is strongest and where the top colour is reached (the sky's elevation, 0 at the horizon, 1 straight up). The match
// camera sees only the lowest few degrees of the sky (about 0.09 up at the top of the screen), so the bands sit low enough for all of
// them to show.
uniform float mid_at = 0.035;
uniform float top_at = 0.13;
uniform vec4 glow_color : source_color = vec4(1.0, 0.8, 0.55, 1.0);
uniform float glow_strength = 0.0;
// The glow's direction (toward it), and how wide it spreads (bigger is wider).
uniform vec3 glow_dir = vec3(-0.45, 0.08, -1.0);
uniform float glow_size = 0.2;
uniform vec4 wisp_color : source_color = vec4(1.0, 0.86, 0.86, 1.0);
uniform float wisp_amount = 0.0;
// The sun's place in the sky (toward it), and how strongly the sky answers it (the judge's round 41, the owner: the skies read as flat
// cards behind the modelled land): `sun_side` warms and lifts the whole low sky on the sun's side and cools it a step on the far side, as
// light scattered forward through the air; `halo` a bright aureole hugging the sun and a wider soft one round it. 0 leaves the sky as it
// was.
uniform vec3 sun_dir = vec3(-0.26, -0.04, -0.96);
uniform float sun_side = 0.0;
uniform float halo = 0.0;

float hash(vec2 p) {
	p = fract(p * vec2(123.34, 456.21));
	p += dot(p, p + 45.32);
	return fract(p.x * p.y);
}

float noise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), f.x), f.y);
}

void sky() {
	vec3 d = normalize(EYEDIR);
	float e = d.y;
	vec3 col;
	if (e < 0.0) {
		col = mix(horizon_color.rgb, ground_color.rgb, smoothstep(0.0, 0.25, -e));
	} else {
		float lo = smoothstep(0.0, mid_at, e);
		float hi = smoothstep(mid_at, top_at, e);
		col = mix(mix(horizon_color.rgb, mid_color.rgb, lo), top_color.rgb, hi);
	}
	// The glow: strongest at the horizon toward `glow_dir`, fading up the sky and round it.
	float toward = max(dot(d, normalize(glow_dir)), 0.0);
	float glow = pow(toward, 1.0 / max(glow_size, 0.01) * 2.0) * (1.0 - smoothstep(0.0, 0.3, abs(e - 0.03)));
	col = mix(col, glow_color.rgb, clamp(glow * glow_strength, 0.0, 1.0));
	// Wisps: long thin streaks of painted cloud low in the sky, brightest near the glow.
	if (wisp_amount > 0.0 && e > 0.0) {
		float az = atan(d.x, -d.z);
		vec2 p = vec2(az * 2.6, e * 22.0);
		float n = noise(p + vec2(noise(p * 0.5) * 2.0, 0.0)) * 0.65 + noise(p * 2.3) * 0.35;
		float band = smoothstep(0.02, 0.07, e) * (1.0 - smoothstep(0.24, 0.36, e));
		float wisp = smoothstep(0.55, 0.78, n) * band * wisp_amount;
		col = mix(col, mix(wisp_color.rgb, glow_color.rgb, clamp(glow * 1.5, 0.0, 0.7)), wisp);
	}
	if (sun_side > 0.0 || halo > 0.0) {
		vec3 sd = normalize(sun_dir);
		vec2 flat_d = normalize(d.xz + vec2(1e-4));
		vec2 flat_s = normalize(sd.xz + vec2(1e-4));
		float facing = dot(flat_d, flat_s);
		// The low sky on the sun's side: warmer and lighter, fading up the sky and round it.
		float side = pow(max(facing, 0.0), 3.0) * (1.0 - smoothstep(0.0, 0.3, max(e, 0.0)));
		col = mix(col, glow_color.rgb, clamp(side * sun_side, 0.0, 0.85));
		// The far side a step cooler and deeper.
		float away = pow(max(-facing, 0.0), 2.0);
		col = mix(col, col * vec3(0.88, 0.9, 1.06), away * sun_side * 0.7);
		// The aureole.
		float mu = max(dot(d, sd), 0.0);
		// (Broad, not a hot core: a bright core round the drawn sun washed its disc into the sky.)
		col += glow_color.rgb * (pow(mu, 60.0) * 0.18 + pow(mu, 12.0) * 0.1) * halo;
	}
	COLOR = col;
}
