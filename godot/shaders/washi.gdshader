shader_type spatial;
// Washi (the style pass's step 6, the grove's craft; docs/art/stages/STYLE_PASS.md): a paper lantern's body lit from inside, in place of
// a flat glow. Unshaded: brightest where the paper faces the eye and the flame shows through it, deeper and warmer toward its edges, where
// the light crosses more paper; the paper's fibres and its thicker clouds darker against the light (art/stage/png/washi.png,
// tools/art/make_washi.py). StageArt.washi_lantern.
render_mode unshaded, cull_disabled;

uniform vec4 glow : source_color = vec4(1.0, 0.74, 0.44, 1.0);
uniform sampler2D washi_tex : filter_linear_mipmap, repeat_enable, hint_default_white;
// [core, rim, fibres, repeats a unit]: how far the middle turns toward a white-hot cream; how much deeper and warmer the edges go; how much
// the fibres and clouds take off against the light; the paper's scale.
uniform vec4 washi = vec4(0.55, 0.5, 0.35, 1.2);
// The glow times each vertex's colour (the mood light's lanterns carry their shading, darker toward the caps, in it).
uniform bool vertex_tone = false;
varying vec3 obj;

void vertex() {
	obj = VERTEX;
}

void fragment() {
	vec3 c = glow.rgb * (vertex_tone ? COLOR.rgb : vec3(1.0));
	// (Either way round: both sides drawn, the near side's normal may face in.)
	float face = abs(dot(NORMAL, VIEW));
	vec3 hot = mix(c, vec3(1.0, 0.96, 0.86), washi.x);
	vec3 deep = c * mix(vec3(1.0), vec3(0.86, 0.56, 0.36), washi.y);
	vec3 col = mix(deep, hot, smoothstep(0.2, 0.95, face));
	vec3 w = texture(washi_tex, obj.xy * washi.w).rgb;
	float thick = w.r * 0.75 + w.b * 0.35 + (w.g - 0.5) * 0.6;
	col *= 1.0 - washi.z * clamp(thick, 0.0, 1.0);
	ALBEDO = col;
}
