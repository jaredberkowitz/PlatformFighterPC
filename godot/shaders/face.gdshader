shader_type spatial;
// The drawn face on the shell in front of the head: the face's parts (face_common.gdshaderinc) over the skin colour. Away from the
// parts the shell is exactly the skin, lit the same way, so only the face shows.
render_mode cull_back;

uniform vec4 skin : source_color = vec4(1.0);
// The face looks toward its right (the fighter's front when it faces screen right); facing left, the drawing is mirrored so the
// fighter still looks the way it faces.
uniform bool mirror = false;
// How much larger than the shell's square the drawing is shown (about its middle): the concept art's eyes are big for the head.
uniform float face_zoom = 1.15;

// A flash over the whole fighter, per mesh (set by the game with `set_instance_shader_parameter`): white when hit, yellow while charging.
instance uniform vec4 flash : source_color, instance_index(0) = vec4(1.0, 1.0, 1.0, 0.0);
// Which object this is, for the ink lines (shaders/post.gdshader): written as the roughness, which the cel lighting does not use. Each
// fighter has its own (set by fighter_view.gd); the stage and scenery keep the default.
instance uniform float object_id : instance_index(1) = 0.95;

#include "res://shaders/toon_light.gdshaderinc"
#include "res://shaders/face_common.gdshaderinc"
#include "res://shaders/stage_ao.gdshaderinc"

varying vec3 world_pos;

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec2 uv = mirror ? vec2(1.0 - UV.x, UV.y) : UV;
	uv = (uv - 0.5) / face_zoom + 0.5;
	vec3 col = face_draw(uv, skin.rgb);
	// Where nothing is drawn the shell shows the head's own skin instead (it is discarded): the shell sits a hair off the generated
	// head, which is not quite symmetric, so its bare skin shaded a little differently and its rim showed as a seam down the face on
	// one side (a fighter facing left). Only the drawn parts are the shell's.
	if (dot(abs(col - skin.rgb), vec3(1.0)) < 0.004) {
		discard;
	}
	ALBEDO = col;
	EMISSION = flash.rgb * flash.a;
	ROUGHNESS = object_id;
	AO = pf_stage_ao(world_pos);
	AO_LIGHT_AFFECT = 0.45;
	SPECULAR = 0.0;
}
