shader_type canvas_item;
// The face in the menus (the creator's option pictures): the same face as on the head (face_common.gdshaderinc), on a round patch of
// skin. `view` picks what shows: its centre in the face square and how much it is magnified (1 shows the whole square).

#include "res://shaders/face_common.gdshaderinc"

uniform vec4 skin : source_color = vec4(1.0, 0.85, 0.2, 1.0);
uniform vec3 view = vec3(0.5, 0.45, 1.0);
// Draw the skin as a round patch (a head); off, only the parts show, on nothing.
uniform bool head = true;

void fragment() {
	vec2 uv = (UV - 0.5) / view.z + view.xy;
	vec3 c = face_draw(uv, skin.rgb);
	float a = 1.0;
	if (head) {
		float d = length(UV - 0.5) - 0.48;
		a = clamp(0.5 - d / max(fwidth(d), 1e-5), 0.0, 1.0);
	} else {
		a = clamp(length(c - skin.rgb) * 8.0, 0.0, 1.0);
	}
	COLOR = vec4(c, a);
}
