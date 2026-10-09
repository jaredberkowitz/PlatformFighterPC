shader_type spatial;
// The shield bubble: nearly clear in the middle and bright toward its rim (a fresnel edge), with a soft glow that pulses a little, so it
// reads as a bubble round the fighter rather than a flat disc. `tint` follows the shield's health (blue when full, red when nearly broken).
render_mode unshaded, blend_mix, depth_draw_never, cull_back, shadows_disabled;

uniform vec4 tint : source_color = vec4(0.4, 0.7, 1.0, 1.0);
uniform float strength = 1.0;

void fragment() {
	float facing = clamp(dot(NORMAL, VIEW), 0.0, 1.0);
	float rim = pow(1.0 - facing, 2.2);
	float pulse = 0.92 + 0.08 * sin(TIME * 6.0);
	ALBEDO = mix(tint.rgb, vec3(1.0), rim * 0.55);
	ALPHA = clamp((0.16 + rim * 0.75) * pulse * strength, 0.0, 1.0);
}
