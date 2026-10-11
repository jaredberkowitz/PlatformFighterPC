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
// The outline hierarchy: fighters get the full line (their meshes set this to 1, see `_tag_object` in fighter_view.gd); everything
// else keeps this default, a thinner line in a softer ink, so the fighters are the clearest things on screen.
instance uniform float outline_weight : instance_index(4) = 0.6;
uniform vec4 scenery_color : source_color = vec4(0.2, 0.15, 0.24, 1.0);
// The base body's skin under modelled clothes: its vertex colour's green (under the shirt) and blue (under the shorts) say how much line
// it gets there, so the body's outline does not poke through the cloth. Off for everything else.
uniform bool hide_under_shirt = false;
uniform bool hide_under_shorts = false;
// A fighter's line takes a little of its own colour (`rgb`, by `a`), so its line art reads as its own rather than a generic black.
instance uniform vec4 outline_tint : instance_index(5) = vec4(0.0);

void vertex() {
	vec4 view = MODELVIEW_MATRIX * vec4(VERTEX, 1.0);
	vec3 n = (MODELVIEW_MATRIX * vec4(NORMAL, 0.0)).xyz;
	vec2 across = n.xy / max(length(n.xy), 0.0001);
	// A mesh can leave the line out in places through its vertex colour's red (the shirt, round the armpits); without colours it is white.
	float cover = (hide_under_shirt ? COLOR.g : 1.0) * (hide_under_shorts ? COLOR.b : 1.0);
	float w = clamp(-view.z * width, min_width, max_width) * outline_weight * COLOR.r * cover;
	view.xy += across * w;
	view.z -= w * push_back;
	POSITION = PROJECTION_MATRIX * view;
}

void fragment() {
	ALBEDO = mix(scenery_color.rgb, color.rgb, smoothstep(0.6, 1.0, outline_weight));
	ALBEDO = mix(ALBEDO, outline_tint.rgb, outline_tint.a);
}
