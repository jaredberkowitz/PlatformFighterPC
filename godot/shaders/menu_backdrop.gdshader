shader_type canvas_item;
// The menus' ground (ui/ui_kit.gd `Backdrop`). Daylight since the owner's pick of 2026-10-19 (docs/UI_DESIGN_SYSTEM.md): a cool white
// falling to a light steel, a soft light from the upper left where the logo sits, broad faint stripes at the UI's own slant drifting
// slowly across in a pale blue ink (`mark`), and a halftone of dots fading in from the lower corners in the same ink. The colour lives in
// the cards; the ground stays quiet. `tint` leans the whole toward a screen's own colour; `vignette` shades the edges a little.

uniform vec4 top : source_color = vec4(0.957, 0.969, 0.988, 1.0);
uniform vec4 bottom : source_color = vec4(0.788, 0.827, 0.91, 1.0);
uniform vec4 light : source_color = vec4(1.0, 1.0, 1.0, 1.0);
uniform vec4 mark : source_color = vec4(0.235, 0.353, 0.612, 0.06);
uniform vec4 tint : source_color = vec4(0.0, 0.0, 0.0, 0.0);
uniform float vignette = 0.1;
uniform float skew = 0.22;
uniform float aspect = 1.7778;

void fragment() {
	vec2 uv = UV;
	vec3 c = mix(top.rgb, bottom.rgb, smoothstep(0.0, 1.0, uv.y));
	// The light: a broad soft pool from the upper left.
	vec2 p = (uv - vec2(0.18, 0.12)) * vec2(aspect, 1.0);
	c = mix(c, light.rgb, 0.35 * exp(-dot(p, p) * 1.6));
	// Stripes at the UI's slant, wide and faint, drifting.
	float s = (uv.x * aspect + uv.y * skew) * 3.2 - TIME * 0.025;
	float band = smoothstep(0.42, 0.5, abs(fract(s) - 0.5));
	c = mix(c, mark.rgb, band * mark.a);
	// A halftone of dots in the lower corners.
	vec2 g = FRAGCOORD.xy / 14.0;
	vec2 cell = fract(g) - 0.5;
	float corner = max(smoothstep(0.55, 1.0, uv.y) * smoothstep(0.45, 1.0, uv.x), smoothstep(0.6, 1.0, uv.y) * smoothstep(0.5, 0.0, uv.x));
	float dot_r = 0.42 * corner;
	c = mix(c, mark.rgb, (1.0 - smoothstep(dot_r - 0.06, dot_r, length(cell))) * step(0.01, corner) * mark.a * 1.6);
	c = mix(c, tint.rgb, tint.a);
	// A soft vignette toward the steel.
	vec2 v = uv - 0.5;
	c = mix(c, bottom.rgb * 0.82, vignette * smoothstep(0.35, 0.9, length(v * vec2(1.1, 1.0))));
	COLOR = vec4(c, 1.0);
}
