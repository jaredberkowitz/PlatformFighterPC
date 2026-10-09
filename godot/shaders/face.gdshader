shader_type spatial;
// The drawn face: the face artwork (godot/art/faces/*.svg) laid over the skin colour on the shell in front of the head. Where the
// drawing is transparent the shell is exactly the skin, lit the same way, so only the drawing shows.
render_mode cull_back;

uniform vec4 skin : source_color = vec4(1.0);
uniform sampler2D face_tex : source_color, filter_linear_mipmap, repeat_disable, hint_default_transparent;
// The drawings look toward texture right (the fighter's front when it faces screen right); facing left, the drawing is mirrored so the
// fighter still looks the way it faces.
uniform bool mirror = false;

#include "res://shaders/toon_light.gdshaderinc"

void fragment() {
	vec2 uv = mirror ? vec2(1.0 - UV.x, UV.y) : UV;
	vec4 f = texture(face_tex, uv);
	if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
		f = vec4(0.0);
	}
	ALBEDO = mix(skin.rgb, f.rgb, f.a);
	ROUGHNESS = 1.0;
	SPECULAR = 0.0;
}
