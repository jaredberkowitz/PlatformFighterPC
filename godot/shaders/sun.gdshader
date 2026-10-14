shader_type spatial;
// A low sun (scripts/stage_art.gd `_painted_sun`), drawn on one card facing the camera: a hot core bright enough to bloom, its disc
// deepening to orange at the rim, a soft corona fading far out into the sky, and faint rays turning slowly round it, so it reads as a
// light in the sky rather than a flat painted disc. Its radius is `disc` of the card's half-width.
render_mode unshaded, blend_add, depth_draw_never, cull_disabled, shadows_disabled, fog_disabled;

uniform vec4 core : source_color = vec4(1.0, 0.97, 0.86, 1.0);
uniform vec4 body : source_color = vec4(1.0, 0.74, 0.36, 1.0);
uniform vec4 rim : source_color = vec4(1.0, 0.46, 0.22, 1.0);
uniform vec4 corona : source_color = vec4(1.0, 0.66, 0.4, 1.0);
uniform float disc = 0.2;
// How bright the disc is drawn: its hot core reaches the bloom, its gold body and orange rim keep their colour under the tone curve.
uniform float brightness = 1.7;
uniform float corona_strength = 0.8;
uniform float rays = 0.3;

float hash(float n) {
	return fract(sin(n) * 43758.5453);
}

float noise1(float x) {
	float i = floor(x);
	float f = fract(x);
	f = f * f * (3.0 - 2.0 * f);
	return mix(hash(i), hash(i + 1.0), f);
}

void fragment() {
	vec2 p = UV * 2.0 - 1.0;
	float r = length(p) / disc;
	float a = atan(p.y, p.x);
	// The disc: hot in the middle, deepening toward the rim, with a soft anti-aliased edge.
	float inside = 1.0 - smoothstep(0.96, 1.0, r);
	vec3 d = mix(core.rgb, body.rgb, smoothstep(0.0, 0.7, r));
	d = mix(d, rim.rgb, smoothstep(0.75, 1.0, r));
	vec3 c = d * inside * brightness * mix(1.5, 1.0, smoothstep(0.0, 0.45, r));
	// The corona: a broad falloff beyond the rim, and a tighter bright ring hugging it.
	float out_r = max(r - 1.0, 0.0);
	float glow = exp(-out_r * 0.9) * 0.6 + exp(-out_r * 4.0) * 0.8;
	glow *= 1.0 - smoothstep(0.7, 1.0, length(p));
	// Rays: soft spokes of uneven length, turning slowly.
	float spokes = noise1(a * 7.0 + TIME * 0.05) * 0.6 + noise1(a * 17.0 - TIME * 0.08 + 3.0) * 0.4;
	float ray = smoothstep(0.45, 0.85, spokes) * exp(-out_r * 0.7) * (1.0 - smoothstep(0.6, 1.0, length(p)));
	c += corona.rgb * (glow * corona_strength + ray * rays) * (1.0 - inside);
	ALBEDO = c;
}
