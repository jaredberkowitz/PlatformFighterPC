// The face, put together from parts (godot/art/face_parts, drawn by tools/art/make_face_parts.py): two eyes, two brows, two cheeks, a
// nose and a mouth, each placed, sized, turned and coloured by the fighter's look (scripts/face_look.gd sets these). Used by the face
// on the head (face.gdshader) and by the creator's face pictures (face_ui.gdshader), so both always agree.
//
// Coordinates are the face square's: 0..1 across and down, the middle of the face at 0.5. Eyes, brows and cheeks come in pairs: the
// drawing is the right-hand part and the left one is its mirror image (inner ends toward the nose). Each part's drawing is a 256 square
// shown `size` across.

uniform sampler2D eye_fill_tex : filter_linear_mipmap, repeat_disable, hint_default_transparent;
uniform sampler2D eye_line_tex : filter_linear_mipmap, repeat_disable, hint_default_transparent;
// Shut or spiral eyes (an expression's), in place of the open eye.
uniform sampler2D eye_extra_tex : filter_linear_mipmap, repeat_disable, hint_default_transparent;
uniform sampler2D brow_tex : filter_linear_mipmap, repeat_disable, hint_default_transparent;
uniform sampler2D nose_tex : filter_linear_mipmap, repeat_disable, hint_default_transparent;
uniform sampler2D cheek_tex : filter_linear_mipmap, repeat_disable, hint_default_transparent;
uniform sampler2D mouth_tex : source_color, filter_linear_mipmap, repeat_disable, hint_default_transparent;

// Eyes: distance from the middle, height, size, turn (radians; raises the outer corners), and a squash (across, up and down).
uniform vec4 eye_place = vec4(0.1875, 0.418, 0.38, 0.0);
uniform vec2 eye_squash = vec2(1.0);
// 0 open (a pupil and lids drawn inside the white), 1 shut (only `eye_extra_tex`, in ink), 2 a spiral in the white (dazed).
uniform int eye_mode = 0;
uniform float pupil_size = 0.257;
// Where the pupil sits in the eye (the eyes look toward the fighter's front).
uniform vec2 pupil_shift = vec2(0.051, 0.01);
// The lids: how far the top one is down (0 open .. 1 shut), its slope (lowers the inner end), how far the lower one is up.
uniform vec3 lids = vec3(0.0);
// Extra upper lid on one side (positive the right, negative the left): an uneven face.
uniform float lid_lean = 0.0;
// A faint hand-drawn wobble on every part's lines (in the face square's units).
uniform float line_wobble = 0.0024;
uniform vec4 eye_color : source_color = vec4(0.176, 0.102, 0.235, 1.0);
uniform vec4 lid_color : source_color = vec4(1.0, 0.85, 0.2, 1.0);
uniform vec4 brow_place = vec4(0.17, 0.258, 0.36, 0.0);
uniform vec2 brow_squash = vec2(1.0);
uniform vec4 brow_color : source_color = vec4(0.165, 0.102, 0.2, 1.0);
// The mouth: across, height, size, turn.
uniform vec4 mouth_place = vec4(0.5, 0.582, 0.22, 0.0);
uniform vec2 mouth_squash = vec2(1.0);
uniform vec3 nose_place = vec3(0.5, 0.5, 0.14);
uniform vec4 nose_color : source_color = vec4(0.165, 0.102, 0.2, 0.8);
// Cheeks: distance from the middle, height, size.
uniform vec3 cheek_place = vec3(0.309, 0.566, 0.2);
uniform vec4 cheek_color : source_color = vec4(0.96, 0.576, 0.576, 0.85);
uniform vec4 ink_color : source_color = vec4(0.165, 0.102, 0.2, 1.0);
uniform vec4 cream_color : source_color = vec4(1.0, 0.957, 0.863, 1.0);

// Where `uv` falls in a part's drawing (0..1 inside it) for a part at `centre`. `side` is 1 for the right-hand part (or a single one)
// and -1 for the left, which is mirrored and turned the other way.
vec2 face_part_uv(vec2 uv, vec2 centre, float size, vec2 squash, float turn, float side) {
	vec2 p = uv - centre;
	float r = -turn * side;
	float c = cos(r);
	float s = sin(r);
	p = vec2(c * p.x - s * p.y, s * p.x + c * p.y);
	p /= size * squash;
	p.x *= side;
	// (A hand-drawn wobble, the character judge's round 12: crisp vector lines read as an emoji set.)
	p += vec2(sin(p.y * 23.0 + side * 1.7), sin(p.x * 19.0 + 0.6)) * line_wobble / max(size, 0.05);
	return p + 0.5;
}

float face_mask(sampler2D t, vec2 p) {
	if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0) {
		return 0.0;
	}
	return texture(t, p).a;
}

// Coverage of a shape from its signed distance (negative inside), smoothed over about a pixel.
float face_edge(float d) {
	return clamp(0.5 - d / max(fwidth(d), 1e-5), 0.0, 1.0);
}

vec3 face_eye(vec3 col, vec2 uv, float side) {
	vec2 t = face_part_uv(uv, vec2(0.5 + side * eye_place.x, eye_place.y), eye_place.z, eye_squash, eye_place.w, side);
	if (eye_mode == 1) {
		return mix(col, ink_color.rgb, face_mask(eye_extra_tex, t));
	}
	float white = face_mask(eye_fill_tex, t);
	vec2 m = t - 0.5;
	vec3 e = cream_color.rgb;
	if (eye_mode == 0) {
		// (Flat, after the reference cast, 2026-10-18: a plain pale white, a solid dark pupil and one small catchlight; the domed white, the
		// glowing iris and two catchlights read as glossy doll's eyes, not the reference's deadpan drawn eyes.)
		vec2 pc = vec2(pupil_shift.x * side, pupil_shift.y);
		vec2 q = m - pc;
		vec3 iris = mix(eye_color.rgb, ink_color.rgb, 0.6);
		e = mix(e, iris, face_edge(length(q) - pupil_size));
		// (No catchlight, the character pass, 2026-10-19: the reference cast's pupils are plain dark dots; a shine read as glossy doll's
		// eyes.)
		// Lids in the fighter's colour (lit from above: lighter toward their top), each with an ink edge and a soft shadow on the eye
		// below it.
		float lid_x = lids.x + max(lid_lean * side, 0.0);
		if (lid_x > 0.001) {
			float d = m.y - (-0.36 + 0.72 * lid_x + lids.y * -m.x);
			e *= mix(1.0, 0.7, smoothstep(0.14, 0.0, -d) * step(-d, 0.0) + 0.0);
			vec3 lid = lid_color.rgb * mix(1.12, 0.92, smoothstep(-0.36, 0.2, m.y));
			e = mix(e, lid, face_edge(d));
			// (Bolder, the character pass, 2026-10-19: the reference's lids are confident ink strokes.)
			e = mix(e, ink_color.rgb, face_edge(abs(d) - 0.036));
		}
		if (lids.z > 0.001) {
			float d = (0.36 - 0.72 * lids.z) - m.y;
			e = mix(e, lid_color.rgb * 0.95, face_edge(d));
			e = mix(e, ink_color.rgb, face_edge(abs(d) - 0.022));
		}
	}
	col = mix(col, e, white);
	if (eye_mode == 2) {
		col = mix(col, ink_color.rgb, face_mask(eye_extra_tex, t));
	}
	// (Only the heavy upper lid line, 2026-10-18: the reference's eyes have no ring round them, just a thick dark line along the top
	// that thins out toward the corners. Shut eyes keep their own drawing.)
	float upper = smoothstep(0.06, -0.12, m.y);
	return mix(col, ink_color.rgb, face_mask(eye_line_tex, t) * upper);
}

// The whole face over `base` (the skin).
vec3 face_draw(vec2 uv, vec3 base) {
	vec3 col = base;
	for (int i = 0; i < 2; i++) {
		float side = i == 0 ? -1.0 : 1.0;
		vec2 t = face_part_uv(uv, vec2(0.5 + side * cheek_place.x, cheek_place.y), cheek_place.z, vec2(1.0), 0.0, side);
		// (Soft toward its edge, as a flush of colour is.)
		float soft = smoothstep(0.5, 0.18, length((t - 0.5) * vec2(1.0, 1.5)));
		col = mix(col, cheek_color.rgb, cheek_color.a * face_mask(cheek_tex, t) * mix(0.55, 1.0, soft));
	}
	vec2 nt = face_part_uv(uv, nose_place.xy, nose_place.z, vec2(1.0), 0.0, 1.0);
	col = mix(col, nose_color.rgb, nose_color.a * face_mask(nose_tex, nt));
	vec2 mt = face_part_uv(uv, mouth_place.xy, mouth_place.z, mouth_squash, mouth_place.w, 1.0);
	if (mt.x >= 0.0 && mt.x <= 1.0 && mt.y >= 0.0 && mt.y <= 1.0) {
		vec4 mc = texture(mouth_tex, mt);
		// An open mouth is hollow: its inside (anything lighter than the ink line) is in shadow toward the top, under the upper lip.
		float inside = smoothstep(0.12, 0.25, dot(mc.rgb, vec3(0.3, 0.5, 0.2)));
		mc.rgb *= mix(1.0, mix(0.62, 1.06, smoothstep(0.36, 0.66, mt.y)), inside);
		col = mix(col, mc.rgb, mc.a);
	}
	for (int i = 0; i < 2; i++) {
		float side = i == 0 ? -1.0 : 1.0;
		vec2 t = face_part_uv(uv, vec2(0.5 + side * brow_place.x, brow_place.y), brow_place.z, brow_squash, brow_place.w, side);
		col = mix(col, brow_color.rgb, face_mask(brow_tex, t));
	}
	col = face_eye(col, uv, -1.0);
	col = face_eye(col, uv, 1.0);
	return col;
}
