// The body shared by the effect shaders (fx.gdshader, fx_top.gdshader): unshaded vertex colours, optionally a soft round texture, with
// their near-white cores pushed above the glow threshold (`pf_fx_glow`, set by scripts/lighting.gd), so effects bloom and the fighters
// and the stage do not.
//
// Two painted touches, each off unless the material asks (scripts/fx_material.gd):
// - `paint`: the effect is laid on in brush strokes rather than as flat vector fills: its colour varies a little in streaks running out
//   from the effect's middle, and as it fades (its alpha falls) it does not go see-through but breaks up into ragged chunks, the way a
//   hand-drawn burst is erased a few frames at a time; a thin darker edge runs round each chunk, like the pooled edge of a wash.
// - `puff`: each quad is a cel-shaded ball of smoke (as the painted smoke of a hand-painted sea adventure): a lumpy round cloud in the
//   vertex colour, lit from the upper left, a cool shade crescent on the lower right and a soft darker rim, eaten away by the same ragged
//   breakup as it fades.
// - `paper` (with either): the effects in the stages' cut-paper language (docs/art/stages/PAINTOVERS.md): flat fills with a faint paper
//   fibre instead of brush streaks, and as it fades it tears into crisp pieces with a pale torn edge (the paper's fibres) rather than a
//   darker wash; a puff is a flat two-tone paper cloud with a pale cut rim.
uniform sampler2D tex : source_color, filter_linear, hint_default_white;
uniform bool use_tex = false;
// Extra brightness for this material's cores on top of the shared glow (0 for an effect that should never bloom).
uniform float glow_scale = 1.0;
uniform float paint = 0.0;
uniform bool puff = false;
uniform bool paper = false;
// The pale of a torn or cut paper edge.
const vec3 PAPER_EDGE = vec3(1.0, 0.97, 0.9);
// How see-through the whole effect is, on top of the breakup (1 opaque). Painted effects use their alpha for the ragged breakup, so a
// translucent trail asks for it here instead.
uniform float opacity = 1.0;
// The puff's shade (multiplied into its colour): a cool lilac-blue, as painted shadow on white smoke.
uniform vec3 puff_shade = vec3(0.84, 0.86, 0.97);
global uniform float pf_fx_glow;

varying vec3 v_pos;
varying vec2 v_seed;

float fx_hash(vec2 p) {
	return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float fx_noise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	return mix(mix(fx_hash(i), fx_hash(i + vec2(1.0, 0.0)), f.x), mix(fx_hash(i + vec2(0.0, 1.0)), fx_hash(i + vec2(1.0, 1.0)), f.x), f.y);
}

float fx_fbm(vec2 p) {
	return fx_noise(p) * 0.62 + fx_noise(p * 2.3 + 17.0) * 0.38;
}

void vertex() {
	// Local position (so the strokes and the breakup stay on the effect as it moves), shifted by where the effect is so no two match.
	v_pos = VERTEX + MODEL_MATRIX[3].xyz * 1.37;
	v_seed = UV2;
}

void fragment() {
	vec4 c = COLOR;
	float edge_dark = 0.0;
	if (puff) {
		// A lumpy ball: the rim wobbles round the circle (its lumps differ ball to ball by the seed in UV2).
		vec2 p = UV * 2.0 - 1.0;
		float r = length(p);
		float a = atan(p.y, p.x);
		float s = v_seed.x * 6.283;
		float rim = 0.84 + 0.07 * sin(a * 5.0 + s) + 0.05 * sin(a * 8.0 - s * 1.7);
		float inside = 1.0 - smoothstep(rim - 0.05, rim, r);
		// Lit from the upper left (UV's y runs down): the shade is what falls outside a disc shifted up and left.
		float lit_r = length(p - vec2(-0.2, -0.26));
		float shade = smoothstep(0.8, 0.86, lit_r + 0.06 * fx_noise(p * 3.0 + s));
		vec3 col = mix(c.rgb, c.rgb * puff_shade, shade);
		if (paper) {
			// A cut paper cloud: the shade a flat second sheet, the rim a pale cut edge, crisp.
			inside = 1.0 - step(rim, r);
			col = mix(c.rgb, c.rgb * puff_shade, step(0.5, shade));
			col = mix(col, PAPER_EDGE, step(rim - 0.07, r) * 0.85);
		} else {
			col *= 1.0 - 0.1 * smoothstep(rim - 0.12, rim - 0.04, r);
		}
		// The ragged breakup as it fades: noise over the ball, cut at the colour's alpha.
		float n = fx_fbm(p * 2.2 + v_seed * 13.0);
		float keep = smoothstep(n * 0.9 + 0.06, n * 0.9 + 0.1, c.a);
		if (paper) {
			keep = step(n * 0.9 + 0.06, c.a);
			col = mix(col, PAPER_EDGE, (1.0 - step(n * 0.9 + 0.12, c.a)) * keep * (1.0 - step(0.995, c.a)));
		}
		c = vec4(col, inside * keep);
	} else {
	if (use_tex) {
		c *= texture(tex, UV);
	}
	if (paper && paint > 0.0) {
		// Cut paper: a flat fill with a faint fibre, torn crisply as it fades, the torn edge pale.
		vec2 q = use_tex ? (UV * 2.0 - 1.0) * 1.6 + v_seed * 9.0 : v_pos.xy;
		c.rgb *= 1.0 + (fx_noise(q * 14.0) - 0.5) * 0.05;
		float n = fx_fbm(q * 1.8 + 3.1);
		float cut = n * 0.92 + 0.04;
		float keep = step(cut, c.a);
		// (Only once it has begun to go: at full strength the noise's high spots reached past the edge band and paled whole patches.)
		float torn = (1.0 - step(cut + 0.07, c.a)) * keep * (1.0 - step(0.995, c.a));
		c.rgb = mix(c.rgb, PAPER_EDGE, torn * 0.9);
		c.a = keep;
	} else if (paint > 0.0) {
		// Strokes running out from the middle: noise over (angle, distance), so the variation streaks radially.
		vec2 q = use_tex ? (UV * 2.0 - 1.0) * 1.6 + v_seed * 9.0 : v_pos.xy;
		float ang = atan(q.y, q.x);
		float streak = fx_noise(vec2(ang * 5.0, length(q) * 1.2) + v_seed * 7.0);
		c.rgb *= 1.0 + paint * (streak - 0.5) * 0.28;
		// The breakup: coverage (alpha) against chunky noise, with a darker line along each cut.
		float n = fx_fbm(q * 2.6 + 3.1);
		float cut = n * 0.92 + 0.04;
		float keep = smoothstep(cut, cut + 0.03, c.a);
		edge_dark = paint * (1.0 - smoothstep(cut + 0.03, cut + 0.12, c.a)) * keep;
		c.a = mix(c.a, keep, paint);
		c.rgb *= 1.0 - 0.3 * edge_dark;
	}
	}
	float luma = dot(c.rgb, vec3(0.3, 0.59, 0.11));
	float boost = 1.0 + (pf_fx_glow - 1.0) * glow_scale * smoothstep(0.72, 0.97, luma);
	ALBEDO = c.rgb * boost;
	ALPHA = c.a * opacity;
}
