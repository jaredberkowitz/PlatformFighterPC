shader_type spatial;
// The glass of the city building's modelled windows (scripts/stage_art.gd `_facade_windows`, one MultiMesh of unit quads). Per window
// (INSTANCE_CUSTOM): rgb the curtain colour, a whether the room is lit. A lit room is warm and brightest toward the top, its curtains
// drawn to the sides in soft folds; a dark pane is a deep blue falling darker toward its foot with a faint sheen of the sky. Both sit
// under the frame's inner shadow along the top and left (the light is up and to the left), so the glass reads as set back in the wall.
render_mode unshaded, cull_back, shadows_disabled;

varying vec4 info;

void vertex() {
	info = INSTANCE_CUSTOM;
}

void fragment() {
	vec2 uv = vec2(UV.x, UV.y);
	vec3 c;
	if (info.a > 0.5) {
		c = mix(vec3(1.0, 0.7, 0.36), vec3(0.78, 0.38, 0.2), uv.y);
		// Curtains: a band either side, folded (a soft ripple of light and shade), hanging a little wider at the foot.
		float reach = mix(0.2, 0.27, uv.y);
		float side = min(uv.x, 1.0 - uv.x);
		if (side < reach) {
			float fold = 0.82 + 0.18 * sin(side / reach * 9.0);
			c = info.rgb * 0.85 * fold * mix(1.0, 0.72, uv.y);
		}
	} else {
		c = mix(vec3(0.13, 0.14, 0.27), vec3(0.05, 0.05, 0.12), uv.y);
		float sheen = smoothstep(0.07, 0.0, abs(uv.x * 0.9 + uv.y * 0.55 - 0.55));
		c += vec3(0.07, 0.08, 0.14) * sheen;
	}
	float top = 1.0 - smoothstep(0.0, 0.14, uv.y);
	float left = 1.0 - smoothstep(0.0, 0.09, uv.x);
	c *= 1.0 - 0.45 * max(top, left * 0.7);
	ALBEDO = c;
	ROUGHNESS = 0.95;
}
