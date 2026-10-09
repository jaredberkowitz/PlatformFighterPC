shader_type spatial;
// The pale halo round a fighter (as in an illustration where the characters glow against the scene): a cream hull outside the ink line,
// pushed back along the view ray so it is hidden wherever the fighter's own body is behind it. Only the outer silhouette gets it, never
// a hand in front of the chest. The width stays about the same on screen at any zoom.
render_mode unshaded, cull_front, shadows_disabled, fog_disabled;

uniform vec4 color : source_color = vec4(1.0, 0.97, 0.86, 1.0);
uniform float width = 0.0062;
uniform float min_width = 0.06;
uniform float max_width = 0.3;
uniform float push = 1.6;

void vertex() {
	vec4 view = MODELVIEW_MATRIX * vec4(VERTEX, 1.0);
	vec3 n = normalize((MODELVIEW_MATRIX * vec4(NORMAL, 0.0)).xyz);
	view.xyz += n * clamp(-view.z * width, min_width, max_width);
	// Further away along the ray from the camera, so it lands in the same place on screen but behind the body.
	float dist = length(view.xyz);
	view.xyz *= (dist + push) / max(dist, 0.001);
	POSITION = PROJECTION_MATRIX * view;
}

void fragment() {
	ALBEDO = color.rgb;
}
