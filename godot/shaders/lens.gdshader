shader_type spatial;
// Tinted glasses' lenses (scripts/fighter_view.gd, the shades): coloured glass, multiplying what is behind by its tint, so the eyes read
// through with their dark pupils as the reference's shades do; the white glare streaks are drawn over it by lens_glare.gdshader.
// (Laid over the face with a see-through tint until the glasses pass, 2026-10-28: in linear light 42% orange over the navy pupil lifted it
// to a mid orange, and pupil and white ran together.)
render_mode unshaded, blend_mul, cull_back, shadows_disabled, depth_draw_never;

// (A saturated orange, the reference's signature lenses, the owner, 2026-10-19; paler and yellower since the glasses pass: multiplied, the
// orange turned blue skin behind the lens a dirty grey.)
uniform vec4 tint : source_color = vec4(1.0, 0.86, 0.6, 1.0);

void fragment() {
	ALBEDO = tint.rgb;
}
