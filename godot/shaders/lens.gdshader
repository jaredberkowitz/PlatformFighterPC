shader_type spatial;
// Tinted glasses' lenses (scripts/fighter_view.gd, the shades): a light see-through tint and two white diagonal glare streaks across the
// lens, as the owner's reference cast's shades are drawn (the character pass, 2026-10-19: a heavy flat tint turned the eyes behind brown).
render_mode unshaded, cull_back, shadows_disabled, depth_draw_never;

uniform vec4 tint : source_color = vec4(1.0, 0.72, 0.5, 0.32);
uniform float glare = 0.85;
// The lens's half size in its own space (the modelled lens has no UVs; its own coordinates place the streaks).
uniform vec2 half_size = vec2(0.25, 0.18);
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
	ALBEDO = mix(tint.rgb, vec3(1.0), streak);
	ALPHA = max(tint.a, streak * 0.9);
}
