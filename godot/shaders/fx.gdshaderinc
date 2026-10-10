// The body shared by the effect shaders (fx.gdshader, fx_top.gdshader): unshaded vertex colours, optionally a soft round texture, with
// their near-white cores pushed above the glow threshold (`pf_fx_glow`, set by scripts/lighting.gd), so effects bloom and the fighters
// and the stage do not.
uniform sampler2D tex : source_color, filter_linear, hint_default_white;
uniform bool use_tex = false;
// Extra brightness for this material's cores on top of the shared glow (0 for an effect that should never bloom).
uniform float glow_scale = 1.0;
global uniform float pf_fx_glow;

void fragment() {
	vec4 c = COLOR;
	if (use_tex) {
		c *= texture(tex, UV);
	}
	float luma = dot(c.rgb, vec3(0.3, 0.59, 0.11));
	float boost = 1.0 + (pf_fx_glow - 1.0) * glow_scale * smoothstep(0.72, 0.97, luma);
	ALBEDO = c.rgb * boost;
	ALPHA = c.a;
}
