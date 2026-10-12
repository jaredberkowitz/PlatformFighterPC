shader_type spatial;
// The sea (scripts/stage_art.gd `Waves`): banded, not photographic. Shallow to deep colour in a few cel bands with distance, two slow layers
// of flow streaks drifting across, a gentle swell, small glints that come and go, and a ruffled band of foam wherever something stands
// in the water (read from the depth texture: the distant islands, the lighthouse, the boat). It is drawn after the finishing pass
// (shaders/post.gdshader), so it fades into the stage's haze itself, the same way the scenery does. `cheap` (the Fast tier) keeps only
// the bands.
render_mode unshaded, cull_disabled, depth_draw_always, shadows_disabled, fog_disabled;

uniform sampler2D depth_tex : hint_depth_texture, filter_nearest;
uniform vec4 shallow : source_color = vec4(0.36, 0.78, 0.9, 1.0);
uniform vec4 deep : source_color = vec4(0.16, 0.5, 0.8, 1.0);
uniform vec4 streak : source_color = vec4(0.62, 0.88, 0.98, 1.0);
uniform vec4 foam : source_color = vec4(0.95, 0.99, 1.0, 1.0);
uniform vec4 haze : source_color = vec4(0.84, 0.94, 1.0, 1.0);
uniform float haze_max = 0.3;
uniform float foam_width = 1.4;    // world units of foam out from where something meets the water
uniform bool cheap = false;
varying vec3 world_pos;

float hash(vec2 p) {
	p = fract(p * vec2(123.34, 456.21));
	p += dot(p, p + 45.32);
	return fract(p.x * p.y);
}

float noise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), f.x), f.y);
}

void vertex() {
	vec3 w = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	if (!cheap) {
		VERTEX.y += sin(w.x * 0.07 + TIME * 0.6) * 0.18 + sin(w.z * 0.11 + w.x * 0.02 + TIME * 0.8) * 0.12;
	}
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	// How far back (behind the stage plane at z = 0) this water is: bands of colour, lighter and paler toward the horizon.
	float back = max(-world_pos.z, 0.0);
	float band = floor(smoothstep(0.0, 180.0, back) * 4.0) / 3.0;
	vec3 col = mix(deep.rgb, shallow.rgb, 0.35 + band * 0.45);
	if (!cheap) {
		// Two layers of long streaks drifting at different speeds, stretched along the shore.
		vec2 p = world_pos.xz;
		float s1 = noise(p * vec2(0.06, 0.45) + vec2(TIME * 0.05, 0.0));
		float s2 = noise(p * vec2(0.1, 0.8) + vec2(-TIME * 0.08, TIME * 0.02) + 17.0);
		float streaks = smoothstep(0.76, 0.81, s1) * 0.7 + smoothstep(0.8, 0.85, s2) * 0.5;
		col = mix(col, streak.rgb, streaks * (1.0 - band * 0.6));
		// Glints: a few cells light up and fade, more of them nearer.
		vec2 cell = floor(p * vec2(0.9, 2.2));
		float g = hash(cell);
		float twinkle = smoothstep(0.75, 1.0, sin(TIME * 2.3 + g * 40.0));
		col = mix(col, vec3(1.0), step(0.965, g) * twinkle * (1.0 - band) * 0.85);
		// Foam where something stands in the water: the scene behind is only a little below the surface there.
		float d = texture(depth_tex, SCREEN_UV).r;
		vec4 v = INV_PROJECTION_MATRIX * vec4(SCREEN_UV * 2.0 - 1.0, d, 1.0);
		float scene = -v.z / v.w;
		float gap = scene - (-VERTEX.z);
		float ruffle = noise(p * 1.3 + vec2(TIME * 0.3, 0.0)) * 0.6;
		float shore = 1.0 - smoothstep(0.0, foam_width, gap + ruffle);
		col = mix(col, foam.rgb, step(0.45, shore) * 0.9);
	}
	// The stage's distance haze (the scenery gets it from the finishing pass; the water is drawn after it).
	col = mix(col, mix(haze.rgb, vec3(1.0), 0.3), smoothstep(4.0, 110.0, back) * haze_max);
	ALBEDO = col;
	ALPHA = 1.0;
}
