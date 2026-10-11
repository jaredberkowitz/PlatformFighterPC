shader_type spatial;
// The ink outline drawn round every character and prop (the toon material's next pass): the back faces pushed out along their normals by
// an amount that grows with distance from the camera, so the line stays about the same thickness on screen whether the camera is close in
// or pulled far back (between `min_width` and `max_width` world units). The faces are pushed out only across the screen (not toward the
// camera) and nudged a little back, so the line rims the silhouette without cutting across the model where one part is in front of
// another (an arm across the body).
render_mode unshaded, cull_front, shadows_disabled;

uniform vec4 color : source_color = vec4(0.07, 0.05, 0.09, 1.0);
uniform float width = 0.0022;
uniform float min_width = 0.02;
uniform float max_width = 0.11;
uniform float push_back = 0.6;   // how far back the line sits, as a share of its width

void vertex() {
	vec4 view = MODELVIEW_MATRIX * vec4(VERTEX, 1.0);
	vec3 n = (MODELVIEW_MATRIX * vec4(NORMAL, 0.0)).xyz;
	vec2 across = n.xy / max(length(n.xy), 0.0001);
	float w = clamp(-view.z * width, min_width, max_width);
	view.xy += across * w;
	view.z -= w * push_back;
	POSITION = PROJECTION_MATRIX * view;
}

void fragment() {
	ALBEDO = color.rgb;
}
