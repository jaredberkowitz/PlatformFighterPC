shader_type spatial;
// An arm or a leg, with its clothing painted on as bands along it instead of separate pieces that could clip: UV.y runs from the shoulder
// or hip (0) to the wrist or ankle (1). `top` covers the start of the limb up to `top_end` (a shirt sleeve, the leg of the shorts),
// `bottom` the end from `bottom_start` (a sock); the rest is `skin`. A thin ink line marks each hem.
render_mode cull_back;

uniform vec4 skin : source_color = vec4(1.0);
uniform vec4 top : source_color = vec4(1.0);
uniform float top_end = -1.0;
uniform vec4 bottom : source_color = vec4(1.0);
uniform float bottom_start = 2.0;
uniform vec4 hem : source_color = vec4(0.1, 0.07, 0.12, 1.0);
uniform float hem_width = 0.025;
instance uniform vec4 flash : source_color = vec4(1.0, 1.0, 1.0, 0.0);
// Which object this is, for the ink lines (shaders/post.gdshader): written as the roughness, which the cel lighting does not use. Each
// fighter has its own (set by fighter_view.gd); the stage and scenery keep the default.
instance uniform float object_id = 0.95;

#include "res://shaders/toon_light.gdshaderinc"
#include "res://shaders/brush.gdshaderinc"
#include "res://shaders/stage_ao.gdshaderinc"

varying vec3 world_pos;

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	float v = UV.y;
	vec3 c = skin.rgb;
	if (v < top_end) {
		c = top.rgb;
	}
	if (v > bottom_start) {
		c = bottom.rgb;
	}
	float line = min(abs(v - top_end), abs(v - bottom_start));
	c = mix(hem.rgb, c, smoothstep(hem_width * 0.5, hem_width, line));
	ALBEDO = c * brush(UV);
	EMISSION = flash.rgb * flash.a;
	ROUGHNESS = object_id;
	AO = pf_stage_ao(world_pos);
	AO_LIGHT_AFFECT = 0.45;
	SPECULAR = 0.0;
}
