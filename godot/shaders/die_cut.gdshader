shader_type canvas_item;
// A fighter cut out of paper (fighter select's face-off, the owner's pick of 2026-11-03): around the picture's shape, a rim of cream
// card `width` pixels wide (the die-cut edge), and under it a hard paper shadow offset down and right. Put on a SubViewportContainer
// whose viewport has a transparent background; leave a margin of at least width + the shadow's offset round the subject.

uniform float width = 5.0;
uniform vec4 edge : source_color = vec4(1.0, 0.98, 0.94, 1.0);
uniform vec2 shadow_offset = vec2(9.0, 12.0);
uniform vec4 shadow : source_color = vec4(0.09, 0.106, 0.208, 0.22);

float dilated(sampler2D tex, vec2 uv, vec2 px, float r) {
	float m = texture(tex, uv).a;
	for (int i = 0; i < 16; i++) {
		float a = float(i) * 0.3926991;
		m = max(m, texture(tex, uv + vec2(cos(a), sin(a)) * r * px).a);
		m = max(m, texture(tex, uv + vec2(cos(a), sin(a)) * r * 0.55 * px).a);
	}
	return m;
}

void fragment() {
	vec2 px = TEXTURE_PIXEL_SIZE;
	vec4 c = texture(TEXTURE, UV);
	float rim = dilated(TEXTURE, UV, px, width);
	float drop = dilated(TEXTURE, UV - shadow_offset * px, px, width);
	vec4 out_col = vec4(shadow.rgb, drop * shadow.a);
	out_col = mix(out_col, vec4(edge.rgb, 1.0), rim * edge.a);
	out_col = mix(out_col, vec4(c.rgb, 1.0), c.a);
	out_col.a = max(max(drop * shadow.a, rim * edge.a), c.a);
	COLOR = out_col;
}
