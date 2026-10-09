shader_type spatial;
// The ink outline drawn round every character and prop (the toon material's next pass): the back faces pushed out along their normals by
// an amount that grows with distance from the camera, so the line stays about the same thickness on screen whether the camera is close in
// or pulled far back (between `min_width` and `max_width` world units).
render_mode unshaded, cull_front, shadows_disabled;

uniform vec4 color : source_color = vec4(0.1, 0.07, 0.12, 1.0);
uniform float width = 0.0032;
uniform float min_width = 0.03;
uniform float max_width = 0.16;

void vertex() {
	vec4 view = MODELVIEW_MATRIX * vec4(VERTEX, 1.0);
	vec3 n = normalize((MODELVIEW_MATRIX * vec4(NORMAL, 0.0)).xyz);
	view.xyz += n * clamp(-view.z * width, min_width, max_width);
	POSITION = PROJECTION_MATRIX * view;
}

void fragment() {
	ALBEDO = color.rgb;
}
