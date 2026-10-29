shader_type spatial;
// The shield bubble. `tint` follows the shield's health (blue when full, red when nearly broken). It pulses a little.
// - `paper` (the default, as the effects' paper look, scripts/fx_material.gd): a medallion of tinted tissue paper folded into six facets,
//   each a step lighter or darker as it faces the key light (upper left), with a faint fibre, a pale cut edge round the rim lit on the
//   upper left and dimmed on the lower right, and a thin ink line at the very edge. No glassy highlight: the art audit of 2026-10-29
//   found the old bubble the one glassy, realistic piece in a paper game.
// - otherwise the older painted look: a faint fill with soft brush streaks swirling over it, a rim that steps up in two bands toward the
//   edge with an ink-dark line, a white highlight crescent on the upper left and a band of reflected light along the lower right.
render_mode unshaded, blend_mix, depth_draw_never, cull_back, shadows_disabled;

uniform vec4 tint : source_color = vec4(0.4, 0.7, 1.0, 1.0);
uniform float strength = 1.0;
uniform bool paper = true;
// The pale of a cut paper edge (as fx.gdshaderinc).
const vec3 PAPER_EDGE = vec3(1.0, 0.97, 0.9);

float sh_hash(vec2 p) {
	return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float sh_noise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	return mix(mix(sh_hash(i), sh_hash(i + vec2(1.0, 0.0)), f.x), mix(sh_hash(i + vec2(0.0, 1.0)), sh_hash(i + vec2(1.0, 1.0)), f.x), f.y);
}

void fragment() {
	float facing = clamp(dot(NORMAL, VIEW), 0.0, 1.0);
	float edge = 1.0 - facing;
	float pulse = 0.94 + 0.06 * sin(TIME * 6.0);
	// The bubble's own screen-facing direction: x right, y up (view space), for the light and the folds.
	// (From the world-space normal: the camera looks down the stage's depth, so world x and y are the screen's.)
	vec2 n = (INV_VIEW_MATRIX * vec4(NORMAL, 0.0)).xy;
	vec2 key = normalize(vec2(-0.6, 0.75));
	float ang = atan(n.y, n.x);
	if (paper) {
		// Six folded facets: each takes the light of its middle direction, stepped, so the creases read as sharp lines.
		float sector = floor((ang + PI) / (TAU / 6.0));
		float mid = (sector + 0.5) * (TAU / 6.0) - PI;
		float lit = dot(vec2(cos(mid), sin(mid)), key);
		vec3 col = tint.rgb * (0.88 + 0.26 * lit);
		// The crease itself: a fine darker line where two facets meet, fading out toward the middle of the disc.
		float to_crease = abs(fract((ang + PI) / (TAU / 6.0)) - 0.5) * 2.0;
		float crease = smoothstep(0.92, 0.98, to_crease) * smoothstep(0.08, 0.3, edge);
		col = mix(col, tint.rgb * 0.6, crease * 0.7);
		// Tissue fibre.
		col *= 1.0 + (sh_noise(n * 18.0) - 0.5) * 0.09;
		float a = 0.36 + 0.06 * lit;
		// The cut edge: a pale band round the rim, lit on the upper left, dimmer on the lower right, crisp on both sides.
		float rim_lit = 0.5 + 0.5 * dot(normalize(n + vec2(1e-4)), key);
		float cut = step(0.7, edge) * (1.0 - step(0.88, edge));
		col = mix(col, mix(tint.rgb * 0.8, PAPER_EDGE, 0.35 + 0.55 * rim_lit), cut);
		a = mix(a, 0.8, cut);
		// The ink line at the very edge.
		float ink = step(0.88, edge);
		col = mix(col, tint.rgb * 0.35, ink);
		a = mix(a, 0.9, ink);
		ALBEDO = col;
		ALPHA = clamp(a * pulse * strength, 0.0, 1.0);
	} else {
	// Brush streaks swirling round the fill.
	float streak = sh_noise(vec2(ang * 3.0 + TIME * 0.6, edge * 4.0));
	// Rim bands, stepped.
	float band1 = smoothstep(0.36, 0.4, edge);
	float band2 = smoothstep(0.62, 0.66, edge);
	float ink = smoothstep(0.86, 0.9, edge);
	// The highlight crescent (upper left) and the bounce (lower right), crisp.
	float hl = smoothstep(0.5, 0.54, dot(n, key)) * smoothstep(0.18, 0.22, edge) * (1.0 - smoothstep(0.5, 0.54, edge));
	float bounce = smoothstep(0.7, 0.74, dot(n, normalize(vec2(0.7, -0.6)))) * band1 * (1.0 - band2);
	vec3 col = tint.rgb * (0.85 + 0.15 * streak);
	col = mix(col, mix(tint.rgb, vec3(1.0), 0.12), band1);
	col = mix(col, mix(tint.rgb, vec3(1.0), 0.45), band2);
	col = mix(col, tint.rgb * 0.45, ink);
	col = mix(col, vec3(1.0), max(hl, bounce * 0.6));
	float a = 0.22 + 0.08 * streak;
	a = mix(a, 0.48, band1);
	a = mix(a, 0.72, band2);
	a = mix(a, 0.85, ink);
	a = max(a, hl * 0.95);
	a = max(a, bounce * 0.55);
	ALBEDO = col;
	ALPHA = clamp(a * pulse * strength, 0.0, 1.0);
	}
}
