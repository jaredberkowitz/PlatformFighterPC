shader_type spatial;
// Character and prop surfaces: a flat colour, optionally multiplied by a tiled texture (cloth prints, straw), lit by the shared soft
// cel lighting.
render_mode cull_back;

uniform vec4 albedo : source_color = vec4(1.0);
uniform sampler2D albedo_tex : source_color, filter_linear_mipmap, repeat_enable, hint_default_white;
uniform bool use_tex = false;
uniform vec2 tex_scale = vec2(1.0);

#include "res://shaders/toon_light.gdshaderinc"

void fragment() {
	vec3 c = albedo.rgb;
	if (use_tex) {
		c *= texture(albedo_tex, UV * tex_scale).rgb;
	}
	ALBEDO = c;
	ROUGHNESS = 1.0;
	SPECULAR = 0.0;
}
