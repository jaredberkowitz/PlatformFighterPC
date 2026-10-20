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
uniform float shade_depth = 1.0;
// How lumpy the puffs are (a share of each puff's radius).
uniform float lump = 0.07;
// The world height of the cloud's flat base (its belly darkens toward it).
uniform float base_y = 0.0;
uniform float belly_height = 2.5;
// How bright the cloud is drawn (the filmic tone curve takes plain white down to a dull grey; the effects' glow starts at 2.3).
uniform float exposure = 1.4;
// How much each puff takes its light from where it sits in the heap (`heap`, set per puff by `_clouds`) rather than from its own
// round surface: the crown puffs catch the sun, the puffs low and on the far side sit in the cloud's own shade, so the heap reads as
// one solid form lit from one side instead of a row of identical lit balls (2026-10-17, the owner's brief: stronger light-facing
// surfaces, a deliberate underside).
uniform float heap_weight = 1.1;
// Where the flat underside's crisp shade edge sits, a share of `belly_height` over the base.
uniform float underside = 0.85;
instance uniform vec3 heap = vec3(0.0, 1.0, 0.0);
// Flat-shaded planes (the folded and faceted clouds, docs/art/stages/FOLDED_STYLE.md): each panel lit as one plane, and the heap bends
// it less, so the folds read.
uniform bool flat_planes = false;

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
	if (flat_planes) {
		vec3 fn = normalize(cross(dFdx(wp), dFdy(wp)));
		n = dot(fn, n) < 0.0 ? -fn : fn;
	}
	vec3 L = normalize(pf_sun_dir);
	// The surface's light: its own normal bent toward where the puff sits in the heap, so one side of the whole cloud is lit.
	vec3 nh = normalize(n + heap * heap_weight * (flat_planes ? 0.25 : 1.0));
	// (Lit from above and the sun's side, a little toward the camera: a cloud seen against the light would otherwise be all shade.)
	float d = dot(nh, normalize(vec3(L.x * 1.4, 1.0, 0.45))) * 0.5 + 0.5;
	// (The shade a clear lavender-blue, deeper than the mood's tint for pale things, so the cloud has distinct tones after the finishing
	// pass quiets the scenery; the lit faces a touch over white and a breath warm, so they catch the bloom and read as sunlit.)
	vec3 shade = mix(colour.rgb, colour.rgb * mix(pf_shadow_tint.rgb, vec3(0.52, 0.6, 1.0), 0.6) * 0.74, shade_depth);
	vec3 soft = mix(shade, colour.rgb, 0.45);
	vec3 lit = colour.rgb * vec3(1.12, 1.1, 1.06);
	// Three flat tones with crisp painted edges (a slow noise wobbles each edge so it is brushed, not ruled).
	float wob = (n3(wp * 0.5 + 3.0) - 0.5) * 0.08;
	vec3 c = mix(shade, soft, smoothstep(0.4, 0.43, d + wob));
	c = mix(c, lit, smoothstep(0.6, 0.63, d + wob));
	// The underside: a flat, deliberate band of shade along the base, its top edge crisp and slightly ragged, deepest at the very bottom.
	float edge_y = base_y + belly_height * underside + (n3(wp * vec3(0.35, 0.0, 0.35)) - 0.5) * belly_height * 0.5;
	float under = 1.0 - smoothstep(edge_y - 0.15, edge_y + 0.15, wp.y);
	float deep = 1.0 - smoothstep(base_y, base_y + belly_height * 0.5, wp.y);
	c = mix(c, shade, under * 0.9);
	c = mix(c, shade * 0.9, deep * 0.5);
	// The silver lining.
	vec3 V = normalize(CAMERA_POSITION_WORLD - wp);
	float rim_edge = 1.0 - clamp(dot(n, V), 0.0, 1.0);
	float behind = clamp(dot(-V, L) * 0.5 + 0.5, 0.0, 1.0);
	float rim = smoothstep(0.6, 0.78, rim_edge) * (0.3 + 0.7 * behind) * (1.0 - under * 0.8) * smoothstep(0.35, 0.6, d);
	c = mix(c, mix(colour.rgb, pf_rim_color.rgb, 0.25) * 1.12, rim);
	// (The exposure lifts the lit faces over the tone curve; the shade gets less of it, so it stays a real tone instead of washing pale.)
	float lum = dot(c, vec3(0.33));
	ALBEDO = c * mix(exposure * 0.85, exposure, smoothstep(0.75, 1.0, lum / max(dot(colour.rgb, vec3(0.33)), 0.01)));
}
