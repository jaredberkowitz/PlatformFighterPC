shader_type canvas_item;
// The menus' backdrop (ui/ui_kit.gd `Backdrop`, the owner 2026-10-18: the live meadow behind the menus read cheap and childish; a
// stylised ground as the genre's leaders use): a deep ink-to-indigo gradient, a soft light from the upper left where the logo sits,
// broad faint stripes at the UI's own slant drifting slowly across, and a halftone of dots fading in from the lower corners. `tint`
// leans the whole toward a screen's own colour.

uniform vec4 top : source_color = vec4(0.17, 0.2, 0.42, 1.0);
uniform vec4 bottom : source_color = vec4(0.06, 0.07, 0.16, 1.0);
uniform vec4 light : source_color = vec4(0.45, 0.62, 1.0, 1.0);
uniform vec4 tint : source_color = vec4(0.0, 0.0, 0.0, 0.0);
uniform float skew = 0.22;
uniform float aspect = 1.7778;

void fragment() {
	vec2 uv = UV;
	vec3 c = mix(top.rgb, bottom.rgb, smoothstep(0.0, 1.0, uv.y));
	// The light: a broad soft pool from the upper left.
	vec2 p = (uv - vec2(0.18, 0.12)) * vec2(aspect, 1.0);
	c += light.rgb * 0.22 * exp(-dot(p, p) * 1.6);
	// Stripes at the UI's slant, wide and faint, drifting.
	float s = (uv.x * aspect + uv.y * skew) * 3.2 - TIME * 0.025;
	float band = smoothstep(0.42, 0.5, abs(fract(s) - 0.5));
	c = mix(c, c * 1.12 + 0.012, band * 0.5);
	// A halftone of dots in the lower corners.
	vec2 g = FRAGCOORD.xy / 14.0;
	vec2 cell = fract(g) - 0.5;
	float corner = max(smoothstep(0.55, 1.0, uv.y) * smoothstep(0.45, 1.0, uv.x), smoothstep(0.6, 1.0, uv.y) * smoothstep(0.5, 0.0, uv.x));
	float dot_r = 0.42 * corner;
	c = mix(c, c * 1.25 + 0.02, (1.0 - smoothstep(dot_r - 0.06, dot_r, length(cell))) * step(0.01, corner) * 0.6);
	c = mix(c, tint.rgb, tint.a);
	// A soft vignette.
	vec2 v = uv - 0.5;
	c *= 1.0 - 0.35 * smoothstep(0.35, 0.9, length(v * vec2(1.1, 1.0)));
	COLOR = vec4(c, 1.0);
}
