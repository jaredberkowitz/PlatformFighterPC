// The cel lighting every character and stage surface shares. Light wraps round the form and steps through the stage's shading ramp
// (`pf_ramp`, a lookup drawn by scripts/lighting.gd from the mood's bands): lit, a midtone, and a painted shadow, the surface's own colour
// turned toward the stage's shadow hue (a cool violet by default, so yellows shade orange, greens teal and blues violet), a little darker
// and richer, while pale colours (white cloth) take a cool tint instead of going grey. Where a cast shadow falls on the shadow side the
// shade deepens (`pf_deep_shade`). A rim in the stage's sky colour lifts the silhouette on the lit side, a small soft highlight gives a
// slight sheen, and fighters get a pale edge light all round (`pf_edge_light`, strongest on dark stages) so they never sink into the
// background. All of it is global (`pf_*`), set for each stage by scripts/lighting.gd.
global uniform vec4 pf_shadow_tint;
global uniform float pf_shadow_hue;
global uniform float pf_shadow_shift;
global uniform float pf_shadow_value;
global uniform vec4 pf_rim_color;
global uniform sampler2D pf_ramp : filter_linear, repeat_disable;
global uniform float pf_deep_shade;
global uniform vec4 pf_edge_light;
uniform float ramp_center : hint_range(0.0, 1.0) = 0.45;
uniform float ramp_softness : hint_range(0.0, 0.5) = 0.16;
// How dark cast shadows (a hat's shadow on the face, a fighter's on the stage) are, 0 to 1: kept light so they read as soft shade.
uniform float shadow_amount : hint_range(0.0, 1.0) = 0.62;
uniform float rim_strength : hint_range(0.0, 2.0) = 0.35;
uniform float rim_width : hint_range(0.0, 1.0) = 0.55;
uniform float sheen : hint_range(0.0, 1.0) = 0.04;
// How much stronger the rim is on this object (fighters set it above 1 so they lift off the background; see `_tag_object`).
instance uniform float rim_boost : instance_index(2) = 1.0;
// 1 on a fighter (set by fighter_view.gd): it gets the edge light, and its shadow side is kept a little lighter so it reads anywhere.
instance uniform float fighter : instance_index(3) = 0.0;
// How much lighter a fighter's shadow side is kept (a share of the way to lit).
const float PF_FIGHTER_LIFT = 0.18;

vec3 pf_rgb2hsv(vec3 c) {
	vec4 k = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
	vec4 p = mix(vec4(c.bg, k.wz), vec4(c.gb, k.xy), step(c.b, c.g));
	vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
	float d = q.x - min(q.w, q.y);
	return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + 1e-6)), d / (q.x + 1e-6), q.x);
}

vec3 pf_hsv2rgb(vec3 c) {
	vec3 p = abs(fract(c.xxx + vec3(1.0, 2.0 / 3.0, 1.0 / 3.0)) * 6.0 - 3.0);
	return c.z * mix(vec3(1.0), clamp(p - 1.0, 0.0, 1.0), c.y);
}

// The colour a surface of colour `a` takes in full shadow.
vec3 pf_painted_shadow(vec3 a) {
	vec3 hsv = pf_rgb2hsv(a);
	float d = pf_shadow_hue - hsv.x;
	d -= round(d);  // the short way round the colour wheel
	vec3 shaded = vec3(fract(hsv.x + clamp(d, -pf_shadow_shift, pf_shadow_shift)), min(1.0, hsv.y * 1.12 + 0.04), hsv.z * pf_shadow_value);
	vec3 rich = pf_hsv2rgb(shaded);
	vec3 pale = a * pf_shadow_tint.rgb;
	return mix(pale, rich, smoothstep(0.1, 0.35, hsv.y));
}

// The stage's ramp at `wrap` (0 facing away from the light, 1 facing it), shifted by this material's `ramp_center` and softened by any
// `ramp_softness` beyond the standard (cloth asks for a softer step).
float pf_band(float wrap) {
	float x = clamp(wrap + (0.45 - ramp_center), 0.0, 1.0);
	float extra = max(ramp_softness - 0.16, 0.0);
	if (extra <= 0.0) {
		return texture(pf_ramp, vec2(x, 0.5)).r;
	}
	return (texture(pf_ramp, vec2(clamp(x - extra, 0.0, 1.0), 0.5)).r + texture(pf_ramp, vec2(x, 0.5)).r
		+ texture(pf_ramp, vec2(clamp(x + extra, 0.0, 1.0), 0.5)).r) / 3.0;
}

void light() {
	float wrap = dot(NORMAL, LIGHT) * 0.5 + 0.5;
	float band = pf_band(wrap);
	float lit = band * mix(1.0, ATTENUATION, shadow_amount);
	// (The diffuse light is multiplied by the albedo afterwards, so the shadow colour is given as a ratio to it.)
	vec3 shade = pf_painted_shadow(ALBEDO) / max(ALBEDO, vec3(0.004));
	// Deep shade: a cast shadow falling on the side already turned from the light.
	float deep = (1.0 - ATTENUATION) * (1.0 - band) * shadow_amount;
	shade = mix(shade, shade * pf_deep_shade, deep);
	shade = mix(shade, vec3(1.0), fighter * PF_FIGHTER_LIFT);
	vec3 tone = mix(shade, vec3(1.0), lit);
	DIFFUSE_LIGHT += tone * LIGHT_COLOR / PI;
	float edge = 1.0 - clamp(dot(NORMAL, VIEW), 0.0, 1.0);
	float rim = smoothstep(1.0 - rim_width, 1.0, edge) * clamp(wrap + 0.2, 0.0, 1.0);
	SPECULAR_LIGHT += pf_rim_color.rgb * rim * rim_strength * rim_boost * LIGHT_COLOR / PI * ALBEDO;
	vec3 half_dir = normalize(LIGHT + VIEW);
	float s = pow(clamp(dot(NORMAL, half_dir), 0.0, 1.0), 24.0);
	SPECULAR_LIGHT += vec3(smoothstep(0.4, 0.6, s) * sheen) * LIGHT_COLOR / PI * ATTENUATION;
	// A fighter's edge light: pale, all round the silhouette whatever the light's direction (half from each of the two lights).
	float around = smoothstep(0.62, 0.95, edge);
	SPECULAR_LIGHT += pf_edge_light.rgb * pf_edge_light.a * around * fighter * 0.5;
}
