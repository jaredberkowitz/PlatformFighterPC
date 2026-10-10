shader_type spatial;
// The match's illustrated finish, drawn over the 3D scene on a quad in front of the camera: ink lines at the real edges of things (where
// one object meets another, read from the object id each surface writes as its roughness, and where the depth jumps, as an arm passes in
// front of the body), creases on the stage and scenery only (so no stray lines inside a fighter), thinner and fainter with distance; a
// warm paper tone, a fine grain like printed paper, and a soft vignette. Transparent things (effects, the shield, damage numbers) are
// drawn after it, so they stay clean.
render_mode unshaded, depth_test_disabled, depth_draw_never, cull_disabled, shadows_disabled, fog_disabled;

uniform sampler2D screen_tex : hint_screen_texture, filter_linear_mipmap;
uniform sampler2D depth_tex : hint_depth_texture, filter_nearest;
uniform sampler2D normal_tex : hint_normal_roughness_texture, filter_nearest;

uniform vec4 ink : source_color = vec4(0.14, 0.08, 0.12, 1.0);
uniform float line_width = 1.25;          // pixels at 1080 lines tall
uniform float depth_edge = 0.07;          // relative jump in distance that draws a line
uniform float normal_edge = 0.6;         // 1 - cosine between neighbouring surface directions that draws a line
uniform float line_strength = 0.9;
uniform float line_fade_start = 60.0;     // world units from the camera
uniform float line_fade_end = 180.0;
uniform float grain = 0.045;
uniform float paper = 0.03;
uniform float vignette = 0.32;
uniform vec4 tint : source_color = vec4(1.0, 0.95, 0.86, 1.0);
uniform float tint_amount = 0.1;
// The stage's colour grade (set with its mood, scripts/lighting.gd): its shadows and its lights tinted toward two colours.
uniform vec4 grade_shadows : source_color = vec4(1.0);
uniform vec4 grade_lights : source_color = vec4(1.0);
uniform float grade_amount = 0.0;
// A finishing hit: for a moment the picture goes to two tones, the fighters pale against a dark frame (the hit's effects, drawn after
// this, stay in colour on top).
uniform float impact = 0.0;
uniform vec4 impact_dark : source_color = vec4(0.1, 0.05, 0.13, 1.0);
uniform vec4 impact_light : source_color = vec4(1.0, 0.97, 0.9, 1.0);

void vertex() {
	POSITION = vec4(VERTEX.xy, 1.0, 1.0);
}

float view_depth(vec2 uv, mat4 inv_proj) {
	float d = texture(depth_tex, uv).r;
	vec4 v = inv_proj * vec4(uv * 2.0 - 1.0, d, 1.0);
	return -v.z / v.w;
}

// The object id a surface wrote (see `object_id` in toon.gdshader): the roughness channel holds it as written (checked by drawing it).
float object_at(vec2 uv) {
	return texture(normal_tex, uv).w;
}

float hash(vec2 p) {
	p = fract(p * vec2(123.34, 456.21));
	p += dot(p, p + 45.32);
	return fract(p.x * p.y);
}

float value_noise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	float a = hash(i);
	float b = hash(i + vec2(1.0, 0.0));
	float c = hash(i + vec2(0.0, 1.0));
	float d = hash(i + vec2(1.0, 1.0));
	return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

void fragment() {
	vec2 uv = SCREEN_UV;
	vec3 col = texture(screen_tex, uv).rgb;
	float d0 = view_depth(uv, INV_PROJECTION_MATRIX);
	// Lines are thinner further away (full width up close, about half on the far scenery).
	float width = line_width * clamp(28.0 / max(d0, 0.001), 0.5, 1.15);
	vec2 px = width * (VIEWPORT_SIZE.y / 1080.0) / VIEWPORT_SIZE;

	// Ink: compare this pixel with its four neighbours.
	vec3 n0 = texture(normal_tex, uv).xyz * 2.0 - 1.0;
	float id0 = object_at(uv);
	float edge = 0.0;
	vec2 offs[4] = vec2[](vec2(px.x, 0.0), vec2(-px.x, 0.0), vec2(0.0, px.y), vec2(0.0, -px.y));
	for (int k = 0; k < 4; k++) {
		vec2 q = uv + offs[k];
		float d = view_depth(q, INV_PROJECTION_MATRIX);
		// Only the nearer side draws the line, so it sits on the edge of the object in front.
		float jump = (d - d0) / max(d0, 0.001);
		edge = max(edge, smoothstep(depth_edge, depth_edge * 2.0, jump));
		float id = object_at(q);
		if (abs(id - id0) > 0.03 && d > d0) {
			edge = max(edge, 1.0);
		}
		// Creases only where both sides are stage or scenery (ids from 0.8 up).
		if (d0 < 400.0 && id0 > 0.8 && id > 0.8) {
			vec3 n = texture(normal_tex, q).xyz * 2.0 - 1.0;
			float crease = 1.0 - dot(normalize(n0), normalize(n));
			edge = max(edge, smoothstep(normal_edge, normal_edge * 1.5, crease) * 0.55);
		}
	}
	edge *= 1.0 - smoothstep(line_fade_start, line_fade_end, d0);
	col = mix(col, ink.rgb, edge * line_strength);

	// A warm paper tone, soft blotches like paper fibre, and a fine print grain.
	col = mix(col, col * tint.rgb, tint_amount);
	vec2 fc = FRAGCOORD.xy / (VIEWPORT_SIZE.y / 1080.0);
	col *= 1.0 + (value_noise(fc / 140.0) - 0.5) * paper;
	col *= 1.0 + (hash(floor(fc)) - 0.5) * grain;

	// Vignette.
	vec2 c = uv - 0.5;
	col *= 1.0 - vignette * smoothstep(0.35, 0.85, length(c * vec2(1.25, 1.0)));

	// The stage's grade.
	float luma = dot(col, vec3(0.3, 0.59, 0.11));
	col = mix(col, col * mix(grade_shadows.rgb, grade_lights.rgb, smoothstep(0.1, 0.85, luma)), grade_amount);

	// The finishing hit's two tones: fighters (ids below 0.6) pale, everything else dark, the ink lines kept.
	if (impact > 0.0) {
		float fighter = id0 < 0.6 && d0 < 400.0 ? 1.0 : 0.0;
		vec3 two = mix(impact_dark.rgb, impact_light.rgb, fighter);
		two = mix(two, impact_dark.rgb * 0.6, edge * fighter);
		col = mix(col, two, impact);
	}

	ALBEDO = col;
}
