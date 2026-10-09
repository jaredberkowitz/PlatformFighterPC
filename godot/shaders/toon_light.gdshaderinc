// The soft cel lighting every character surface shares: light wraps round the form and steps softly from lit to a cool shadow tint
// (rather than to black), a warm rim lifts the silhouette on the lit side, and a small soft highlight gives a slight sheen.
uniform vec4 shadow_tint : source_color = vec4(0.58, 0.52, 0.74, 1.0);
uniform float ramp_center : hint_range(0.0, 1.0) = 0.42;
uniform float ramp_softness : hint_range(0.0, 0.5) = 0.16;
uniform vec4 rim_color : source_color = vec4(1.0, 0.93, 0.82, 1.0);
uniform float rim_strength : hint_range(0.0, 2.0) = 0.35;
uniform float rim_width : hint_range(0.0, 1.0) = 0.55;
uniform float sheen : hint_range(0.0, 1.0) = 0.04;

void light() {
	float wrap = dot(NORMAL, LIGHT) * 0.5 + 0.5;
	float lit = smoothstep(ramp_center - ramp_softness, ramp_center + ramp_softness, wrap) * ATTENUATION;
	vec3 tone = mix(shadow_tint.rgb, vec3(1.0), lit);
	DIFFUSE_LIGHT += tone * LIGHT_COLOR / PI;
	float edge = 1.0 - clamp(dot(NORMAL, VIEW), 0.0, 1.0);
	float rim = smoothstep(1.0 - rim_width, 1.0, edge) * clamp(wrap + 0.2, 0.0, 1.0);
	SPECULAR_LIGHT += rim_color.rgb * rim * rim_strength * LIGHT_COLOR / PI * ALBEDO;
	vec3 half_dir = normalize(LIGHT + VIEW);
	float s = pow(clamp(dot(NORMAL, half_dir), 0.0, 1.0), 24.0);
	SPECULAR_LIGHT += vec3(smoothstep(0.4, 0.6, s) * sheen) * LIGHT_COLOR / PI * ATTENUATION;
}
