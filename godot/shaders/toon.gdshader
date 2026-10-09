shader_type spatial;
// Character and prop surfaces: a flat colour, optionally multiplied by a tiled texture (cloth prints, straw), lit by the shared soft
// cel lighting.
render_mode cull_back;

uniform vec4 albedo : source_color = vec4(1.0);
uniform sampler2D albedo_tex : source_color, filter_linear_mipmap, repeat_enable, hint_default_white;
uniform bool use_tex = false;
uniform vec2 tex_scale = vec2(1.0);
// Above zero: the texture is laid in world space instead of by UV, `world_tile` world units per repeat, on the top for upward faces and
// on the front otherwise (so a stage surface tiles evenly whatever the block's size).
uniform float world_tile = 0.0;
varying vec3 world_pos;
varying vec3 world_normal;

// A flash over the whole fighter, per mesh (set by the game with `set_instance_shader_parameter`): white when hit, yellow while charging.
instance uniform vec4 flash : source_color = vec4(1.0, 1.0, 1.0, 0.0);

#include "res://shaders/toon_light.gdshaderinc"
#include "res://shaders/brush.gdshaderinc"

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	vec3 c = albedo.rgb;
	if (use_tex) {
		vec2 uv = UV * tex_scale;
		if (world_tile > 0.0) {
			uv = (abs(world_normal.y) > 0.5 ? world_pos.xz : vec2(world_pos.x, -world_pos.y)) / world_tile;
		}
		c *= texture(albedo_tex, uv).rgb;
	}
	if (world_tile <= 0.0) {
		c *= brush(UV);
	}
	ALBEDO = c;
	EMISSION = flash.rgb * flash.a;
	ROUGHNESS = 1.0;
	SPECULAR = 0.0;
}
