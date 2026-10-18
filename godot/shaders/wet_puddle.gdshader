shader_type spatial;
// A shallow puddle on the city's wet roof (scripts/stage_art.gd `_roof_puddles`), painted rather than reflected (the camera looks along the
// roof, so a real mirror would show only what stands behind it, and the scenery's roughness carries its object id): a dark glossy patch
// with a soft irregular edge, sheened with the horizon's pink, strongest along its far side, and the skyline's lit windows smeared down it in long broken
// streaks of warm, blue and rose light that ripple slowly. Laid flat on the roof; UV.y runs from its far side (0) to its near side (1).
render_mode unshaded, blend_mix, depth_draw_never, cull_disabled, shadows_disabled;

uniform vec4 tint : source_color = vec4(0.14, 0.11, 0.24, 1.0);
uniform vec4 sky : source_color = vec4(0.72, 0.3, 0.56, 1.0);
uniform float seed = 0.0;
uniform float strength = 0.55;
uniform float columns = 18.0;

float hash(float x) {
	return fract(sin(x * 127.1 + seed * 31.7) * 43758.5453);
}

void fragment() {
	vec2 c = UV * 2.0 - 1.0;
	float ang = atan(c.y, c.x);
	float r = 0.8 + 0.1 * sin(ang * 3.0 + seed * 5.0) + 0.06 * sin(ang * 5.0 + seed * 2.3);
	float mask = 1.0 - smoothstep(r - 0.14, r, length(c));
	float far = 1.0 - UV.y;
	// (A sheen of the sky over all of it, strongest along the far side: seen along the roof, wet ground shines; a dark patch read as a hole.)
	vec3 col = mix(tint.rgb, sky.rgb * 1.2, 0.6 + 0.4 * smoothstep(0.2, 1.0, far));
	// The windows' reflections: some columns carry a lit streak, broken by ripples, brightest toward the far side.
	float id = floor(UV.x * columns);
	float h = hash(id);
	float within = fract(UV.x * columns);
	float streak = step(0.6, h) * (1.0 - smoothstep(0.12, 0.4, abs(within - 0.5)));
	float rip = smoothstep(-0.1, 0.6, 0.5 + 0.5 * sin(UV.y * 11.0 + h * 20.0 + TIME * 0.7));
	float pick = hash(id + 7.0);
	vec3 light = pick < 0.65 ? vec3(1.0, 0.66, 0.3) : (pick < 0.88 ? vec3(0.45, 0.7, 1.0) : vec3(1.0, 0.42, 0.76));
	col += light * streak * rip * mix(0.25, 0.95, far);
	ALBEDO = col;
	ALPHA = mask * strength;
}
