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
// How far round the head (radians) the drawing slides toward the camera (set each frame by fighter_view.gd from the head's turn), so on
// a head turned three-quarters the far eye stays on the visible face instead of squashing into the head's edge (the face pass,
// 2026-10-19). The shell's texture map is a straight-on projection (make_rigged_blob.py: u is x over 1.75 head radii), so the slide is
// done on the ball: u back to an angle round the head at its height, turned, and projected again.
uniform float face_slide = 0.0;

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
	vec2 uv = UV;
	if (abs(face_slide) > 0.0001) {
		vec2 n = (uv - 0.5) * 1.75;
		float ring = sqrt(max(0.0001, 1.0 - n.y * n.y));
		float phi = asin(clamp(n.x / ring, -1.0, 1.0)) - face_slide;
		uv.x = 0.5 + ring * sin(phi) / 1.75;
	}
	uv = mirror ? vec2(1.0 - uv.x, uv.y) : uv;
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
