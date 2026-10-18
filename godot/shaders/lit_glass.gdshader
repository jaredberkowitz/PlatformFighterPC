shader_type spatial;
// A lit window seen from the street (scripts/stage_art.gd `_glass`, on a quad): the room's light brightest toward the top of the pane and
// falling off toward its foot and corners, the glass set back in its frame under an inner shadow along the top and left (the light
// comes from the upper left), and a faint diagonal sheen across it, so a shop window reads as glass with a room behind it rather than a
// flat fill (the owner, 2026-10-18: the windows and doors read flat). Unshaded: it gives its own light.
render_mode unshaded, cull_back, shadows_disabled;

uniform vec4 colour : source_color = vec4(0.9, 0.76, 0.5, 1.0);
// How deep the frame's inner shadow is, and how far it reaches into the pane (a share of the pane's height and width).
uniform float recess = 0.5;
uniform vec2 recess_reach = vec2(0.12, 0.07);

void fragment() {
	vec2 uv = UV;
	// The room's light: brightest up the middle, down toward the floor and the sides.
	float room = mix(1.12, 0.78, uv.y) * mix(0.86, 1.0, 1.0 - pow(abs(uv.x - 0.5) * 2.0, 2.0));
	vec3 c = colour.rgb * room;
	// The frame's inner shadow along the top and left.
	float top = 1.0 - smoothstep(0.0, recess_reach.x, uv.y);
	float left = 1.0 - smoothstep(0.0, recess_reach.y, uv.x);
	c *= 1.0 - recess * max(top, left * 0.8);
	// A faint sheen on the glass.
	float sheen = smoothstep(0.06, 0.0, abs(uv.x * 0.8 + uv.y * 0.5 - 0.62));
	c += vec3(0.1) * sheen;
	ALBEDO = c;
	// Scenery's object id (post.gdshader reads it from the roughness).
	ROUGHNESS = 0.95;
}
