shader_type spatial;
// The shades' glare (the lens's next pass, over lens.gdshader's tint): two white diagonal streaks across the lens, as the owner's
// reference cast's shades are drawn.
render_mode unshaded, blend_mix, cull_back, shadows_disabled, depth_draw_never;

uniform float glare = 0.85;
// The lens's half size in its own space (the modelled lens has no UVs; its own coordinates place the streaks).
uniform vec2 half_size = vec2(0.225, 0.17);
varying vec2 local;

void vertex() {
	local = VERTEX.xy;
}

void fragment() {
	vec2 p = local / half_size * 0.5;
	// Two parallel streaks running up to the right, a wide one and a thin one.
	float d = p.x * 0.7 + p.y * 0.7;
	float s1 = 1.0 - smoothstep(0.03, 0.05, abs(d + 0.06));
	float s2 = 1.0 - smoothstep(0.012, 0.025, abs(d - 0.07));
	float streak = max(s1, s2) * glare;
	if (streak < 0.01) {
		discard;
	}
	ALBEDO = vec3(1.0);
	ALPHA = streak * 0.9;
}
