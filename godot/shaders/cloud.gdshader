shader_type spatial;
// The sky's cumulus (scripts/stage_art.gd `_clouds`), lit by hand from the stage's sun (`pf_sun_dir`) the way a cel-shaded adventure
// paints them: three flat tones (sunlit white, a soft mid and a cool shade in the mood's shadow tint), a belly that darkens toward the
// cloud's flat base, a bright silver lining round the edge (strongest when the sun is behind it), and lumpy edges from a slow noise, so
// no two puffs are the same smooth ball. Unshaded: the engine's light on something this far and this pale only greyed it.
render_mode unshaded, cull_back, shadows_disabled, fog_disabled;

global uniform vec3 pf_sun_dir;
global uniform vec4 pf_shadow_tint;
global uniform vec4 pf_rim_color;
uniform vec4 colour : source_color = vec4(1.0);
// How far the cloud's colour goes into its shade (1 the mood's full shadow tint).
uniform float shade_depth = 0.75;
// How lumpy the puffs are (a share of each puff's radius).
uniform float lump = 0.07;
// The world height of the cloud's flat base (its belly darkens toward it).
uniform float base_y = 0.0;
uniform float belly_height = 2.5;
// How bright the cloud is drawn (the filmic tone curve takes plain white down to a dull grey; the effects' glow starts at 2.3).
uniform float exposure = 1.55;

varying vec3 wn;
varying vec3 wp;

float h3(vec3 p) {
	return fract(sin(dot(p, vec3(127.1, 311.7, 74.7))) * 43758.5453);
}

float n3(vec3 p) {
	vec3 i = floor(p);
	vec3 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	return mix(mix(mix(h3(i), h3(i + vec3(1, 0, 0)), f.x), mix(h3(i + vec3(0, 1, 0)), h3(i + vec3(1, 1, 0)), f.x), f.y),
		mix(mix(h3(i + vec3(0, 0, 1)), h3(i + vec3(1, 0, 1)), f.x), mix(h3(i + vec3(0, 1, 1)), h3(i + vec3(1, 1, 1)), f.x), f.y), f.z);
}

void vertex() {
	vec3 w = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	float n = n3(w * 0.35) * 0.6 + n3(w * 0.9 + 7.0) * 0.4;
	VERTEX += NORMAL * (n - 0.5) * lump * 2.0;
	wp = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	wn = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	vec3 n = normalize(wn);
	vec3 L = normalize(pf_sun_dir);
	// (Lit mostly from above, leaning to the sun's side and a little toward the camera: a cloud seen against the light would otherwise be
	// all shade, and painted cumulus are mostly white with their shade underneath and on the far side.)
	float d = dot(n, normalize(vec3(L.x, 1.3, 0.5))) * 0.5 + 0.5;
	// (The shade a clear lavender-blue, deeper than the mood's tint for pale things, so the cloud has two distinct tones after the finishing
	// pass quiets the scenery; the lit tops a touch over white, so they catch the bloom.)
	vec3 shade = mix(colour.rgb, colour.rgb * mix(pf_shadow_tint.rgb, vec3(0.56, 0.64, 1.0), 0.45) * 0.9, shade_depth);
	vec3 soft = mix(shade, colour.rgb, 0.55);
	vec3 c = mix(shade, soft, smoothstep(0.3, 0.36, d));
	c = mix(c, colour.rgb * 1.1, smoothstep(0.5, 0.56, d));
	// The belly: deeper shade toward the flat base, whatever the light.
	float low = 1.0 - smoothstep(base_y, base_y + belly_height, wp.y);
	c = mix(c, shade * 0.94, low * 0.55);
	// The silver lining.
	vec3 V = normalize(CAMERA_POSITION_WORLD - wp);
	float edge = 1.0 - clamp(dot(n, V), 0.0, 1.0);
	float behind = clamp(dot(-V, L) * 0.5 + 0.5, 0.0, 1.0);
	float rim = smoothstep(0.55, 0.75, edge) * (0.3 + 0.7 * behind) * (1.0 - low * 0.7);
	c = mix(c, mix(colour.rgb, pf_rim_color.rgb, 0.25) * 1.08, rim);
	ALBEDO = c * exposure;
}
