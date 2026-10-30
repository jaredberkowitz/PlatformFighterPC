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
// Where the glow is centred (elevation) and how far either side of it it reaches.
uniform float glow_at = 0.03;
uniform float glow_height = 0.3;
// Full strength everywhere below its centre (a glow rising from behind a skyline), not fading on both sides of it.
uniform bool glow_below = false;
uniform vec4 wisp_color : source_color = vec4(1.0, 0.86, 0.86, 1.0);
uniform float wisp_amount = 0.0;
// The sun's place in the sky (toward it), and how strongly the sky answers it (the judge's round 41, the owner: the skies read as flat
// cards behind the modelled land): `sun_side` warms and lifts the whole low sky on the sun's side and cools it a step on the far side, as
// light scattered forward through the air; `halo` a bright aureole hugging the sun and a wider soft one round it. 0 leaves the sky as it
// was.
uniform vec3 sun_dir = vec3(-0.26, -0.04, -0.96);
uniform float sun_side = 0.0;
uniform float halo = 0.0;
// The sky pass (fidelity review 5's second item: the skies read as plain gradients). `look` 0 the sky as it was; 1 horizon light: a
// bright band of haze just over the horizon, a low bank of soft haze along it that hides the hard line, and a broad, soft mottling so
// the sky is never a perfect gradient; 2 adds painted cloud layers: long stratus bands at three heights, broken along their length,
// lit on top and shaded underneath (on a night sky, `night`, lit from below by the city's glow instead); 3 those layers as cut paper:
// crisp scalloped edges, two tones, a lit cut edge and the soft shadow each lays on the sky under it.
uniform int look = 0;
// (How many stratus layers and how thick, look 2 and up: fewer and broader read less as stripes against a deep top, fidelity review
// 14 item 1; StageArt.sky from the palette's `sky_stratus`.)
uniform vec2 stratus = vec2(3.0, 1.0);
uniform vec4 cloud_lit : source_color = vec4(1.0, 0.96, 0.92, 1.0);
uniform vec4 cloud_shade : source_color = vec4(0.7, 0.76, 0.9, 1.0);
uniform vec4 haze_band : source_color = vec4(0.9, 0.94, 0.96, 1.0);
uniform bool night = false;

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
	float glow = pow(toward, 1.0 / max(glow_size, 0.01) * 2.0) * (1.0 - smoothstep(0.0, glow_height, glow_below ? max(e - glow_at, 0.0) : abs(e - glow_at)));
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
	if (look > 0 && e > -0.05) {
		float az = atan(d.x, -d.z);
		// The bright band of haze just over the horizon.
		float hb = exp(-pow((e - 0.01) / 0.022, 2.0));
		col = mix(col, haze_band.rgb, hb * 0.5);
		// A low bank of soft haze along the horizon, its top gently rolling, hiding the hard line.
		float bank_top = 0.012 + 0.012 * noise(vec2(az * 2.2, 1.0)) + 0.006 * noise(vec2(az * 7.0, 4.0));
		float bank = 1.0 - smoothstep(bank_top - 0.014, bank_top + 0.004, e);
		col = mix(col, mix(haze_band.rgb, cloud_shade.rgb, 0.35), bank * 0.55);
		// Broad soft mottling.
		float m = noise(vec2(az * 1.3, e * 9.0)) * 0.6 + noise(vec2(az * 3.1 + 7.0, e * 19.0)) * 0.4;
		col *= mix(0.95, 1.05, m);
		if (look >= 2) {
			// Three layers of stratus: centre height, thickness, how much of the band is cloud.
			for (int k = 0; k < 3; k++) {
				float fk = float(k);
				if (fk >= stratus.x) {
					break;
				}
				// (Low: the cameras see the sky only up to about 5 degrees, 0.087.)
				float c = 0.02 + 0.021 * fk + 0.006 * noise(vec2(az * 0.8 + fk * 5.0, 2.0));
				float th = (0.007 + 0.003 * fk) * stratus.y;
				float n1 = noise(vec2(az * (5.0 + fk * 2.0) + fk * 13.0, 3.0));
				float n2 = noise(vec2(az * (13.0 + fk * 3.0) + fk * 7.0, 5.0));
				float top = c + th * (0.35 + 0.65 * n1);
				float bot = c - th * (0.25 + 0.3 * n2);
				// Broken along its length.
				float run = smoothstep(0.38, 0.55, noise(vec2(az * (2.2 + fk), fk * 11.0)));
				float u = clamp((e - bot) / max(top - bot, 0.001), 0.0, 1.0);
				float lit_k = night ? 1.0 - u : u;
				vec3 cc = mix(cloud_shade.rgb, cloud_lit.rgb, smoothstep(0.15, 0.95, lit_k));
				float fade = 1.0 - fk * 0.18;
				if (look == 3) {
					// Cut paper: a bank of card cloud, its top cut into rounded puffs, its foot flat; a shaded lower body under a lit
					// upper one, a lit cut edge along the puffs, and the soft shadow the card lays on the sky under it.
					float k_az = 16.0 + 5.0 * fk;
					float puff = pow(abs(sin(az * k_az + fk * 2.1)), 0.6) * (0.65 + 0.35 * noise(vec2(az * k_az * 0.5, fk * 9.0)));
					// (Twice the streaks' depth: as thin as them, a bank read as a line with arcs on it.)
					float thb = th * 2.2;
					float base = c - thb * 0.5;
					float crest = c + thb * (0.25 + 0.9 * puff);
					float bank = step(0.42, noise(vec2(az * (1.6 + fk * 0.7), fk * 11.0)));
					float inside = step(base, e) * step(e, crest) * bank;
					float shadow = step(base - 0.004, e) * step(e, base) * bank;
					col = mix(col, col * 0.9, shadow * 0.7 * fade);
					float body_u = (e - base) / max(crest - base, 0.001);
					vec3 two = mix(cloud_shade.rgb, cloud_lit.rgb, night ? step(body_u, 0.4) : step(0.4, body_u));
					float edge = night ? step(e, base + 0.0016) : step(crest - 0.0016, e);
					col = mix(col, mix(two, vec3(1.0), edge * 0.45), inside * 0.95 * fade);
				} else {
					float soft = 0.006 + 0.004 * fk;
					float a = smoothstep(bot - soft, bot + soft, e) * (1.0 - smoothstep(top - soft, top + soft, e)) * run;
					col = mix(col, cc, a * 0.72 * fade);
				}
			}
		}
	}
	COLOR = col;
}
