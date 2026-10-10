shader_type spatial;
// The drawn face: the face artwork (godot/art/faces/*.svg) laid over the skin colour on the shell in front of the head. Where the
// drawing is transparent the shell is exactly the skin, lit the same way, so only the drawing shows.
render_mode cull_back;

uniform vec4 skin : source_color = vec4(1.0);
uniform sampler2D face_tex : source_color, filter_linear_mipmap, repeat_disable, hint_default_transparent;
// The drawings look toward texture right (the fighter's front when it faces screen right); facing left, the drawing is mirrored so the
// fighter still looks the way it faces.
uniform bool mirror = false;

// A flash over the whole fighter, per mesh (set by the game with `set_instance_shader_parameter`): white when hit, yellow while charging.
instance uniform vec4 flash : source_color = vec4(1.0, 1.0, 1.0, 0.0);
// Which object this is, for the ink lines (shaders/post.gdshader): written as the roughness, which the cel lighting does not use. Each
// fighter has its own (set by fighter_view.gd); the stage and scenery keep the default.
instance uniform float object_id = 0.95;

#include "res://shaders/toon_light.gdshaderinc"
#include "res://shaders/stage_ao.gdshaderinc"

varying vec3 world_pos;

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec2 uv = mirror ? vec2(1.0 - UV.x, UV.y) : UV;
	vec4 f = texture(face_tex, uv);
	if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
		f = vec4(0.0);
	}
	ALBEDO = mix(skin.rgb, f.rgb, f.a);
	EMISSION = flash.rgb * flash.a;
	ROUGHNESS = object_id;
	AO = pf_stage_ao(world_pos);
	AO_LIGHT_AFFECT = 0.45;
	SPECULAR = 0.0;
}
