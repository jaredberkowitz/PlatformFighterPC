shader_type spatial;
// The shield bubble, painted in cel steps like the rest of the game: a faint fill in the tint with soft brush streaks swirling over it, a
// rim that steps up crisply in two bands toward the edge (a lighter inner band, then a bright outer one with an ink-dark line at the very
// edge), a sharp white highlight crescent on the upper left where the key light catches it, and a thin band of reflected light along the
// lower right. It pulses a little. `tint` follows the shield's health (blue when full, red when nearly broken).
render_mode unshaded, blend_mix, depth_draw_never, cull_back, shadows_disabled;

uniform vec4 tint : source_color = vec4(0.4, 0.7, 1.0, 1.0);
uniform float strength = 1.0;

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
	// The bubble's own screen-facing direction: x right, y up (view space), for the highlight and the bounce light.
	// (From the world-space normal: the camera looks down the stage's depth, so world x and y are the screen's.)
	vec2 n = (INV_VIEW_MATRIX * vec4(NORMAL, 0.0)).xy;
	// Brush streaks swirling round the fill.
	float ang = atan(n.y, n.x);
	float streak = sh_noise(vec2(ang * 3.0 + TIME * 0.6, edge * 4.0));
	// Rim bands, stepped.
	float band1 = smoothstep(0.36, 0.4, edge);
	float band2 = smoothstep(0.62, 0.66, edge);
	float ink = smoothstep(0.86, 0.9, edge);
	// The highlight crescent (upper left) and the bounce (lower right), crisp.
	float hl = smoothstep(0.5, 0.54, dot(n, normalize(vec2(-0.6, 0.75)))) * smoothstep(0.18, 0.22, edge) * (1.0 - smoothstep(0.5, 0.54, edge));
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
