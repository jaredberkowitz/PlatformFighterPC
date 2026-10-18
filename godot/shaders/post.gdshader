shader_type spatial;
// The match's illustrated finish, drawn over the 3D scene on a quad in front of the camera: ink lines at the real edges of things (where
// one object meets another, read from the object id each surface writes as its roughness, and where the depth jumps, as an arm passes in
// front of the body), creases on the stage and scenery only (so no stray lines inside a fighter), thinner and fainter with distance; a
// warm paper tone, a fine grain like printed paper, and a soft vignette. Transparent things (effects, the shield, damage numbers) are
// drawn after it, so they stay clean.
render_mode unshaded, depth_test_disabled, depth_draw_never, cull_disabled, shadows_disabled, fog_disabled;

uniform sampler2D screen_tex : hint_screen_texture, filter_linear_mipmap;
uniform sampler2D depth_tex : hint_depth_texture, filter_nearest;
uniform sampler2D normal_tex : hint_normal_roughness_texture, filter_nearest;

uniform vec4 ink : source_color = vec4(0.14, 0.08, 0.12, 1.0);
uniform float line_width = 1.25;          // pixels at 1080 lines tall
uniform float depth_edge = 0.07;          // relative jump in distance that draws a line
uniform float normal_edge = 0.6;         // 1 - cosine between neighbouring surface directions that draws a line
uniform float line_strength = 0.9;
uniform float line_fade_start = 60.0;     // world units from the camera
uniform float line_fade_end = 180.0;
// How strong the screen-space ink is on the stage and scenery (the fighters get it all): about half, so the world reads as painted, its
// forms told by light and colour, and only the fighters are inked in full (one even black stroke round everything is the look of vector
// art, not of a cel-shaded world).
uniform float scenery_lines = 0.4;
uniform float inner_lines = 0.3;
// The paper look under trial (scripts/lighting.gd `paper`): where a line would be drawn on the scenery, its nearer edge is lit instead, a
// cut card's edge catching the light (0 off).
uniform float paper_edge = 0.0;
// The inverted-hull outlines' object id (shaders/outline.gdshader): where a silhouette has one, it is the line, and this pass adds none
// beside it (a second line just inside doubled it into a thick, wavering stroke); where it has none (blades), this pass draws the line.
const float HULL_ID = 0.7;          // how strong a line inside one fighter is (an arm in front of the body), against its silhouette
uniform float grain = 0.045;
uniform float paper = 0.03;
uniform float vignette = 0.16;          // kept light: dark corners make a fighter near the edge hard to read
uniform vec4 tint : source_color = vec4(1.0, 0.95, 0.86, 1.0);
uniform float tint_amount = 0.04;
// The stage's colour grade (set with its mood, scripts/lighting.gd): its shadows and its lights tinted toward two colours.
uniform vec4 grade_shadows : source_color = vec4(1.0);
uniform vec4 grade_lights : source_color = vec4(1.0);
uniform float grade_amount = 0.0;
// The pastel finish (the game's look: soft, airy colours rather than saturated ones): colours eased toward grey by `pastel_desat`, and
// the darks lifted toward the stage's haze colour by `pastel_lift`, before the ink lines, so the lines stay dark and crisp. Fighters
// get half of it, so they stay the most vivid thing on screen.
uniform float pastel_desat = 0.1;
uniform float pastel_lift = 0.13;
uniform vec4 pastel_haze : source_color = vec4(0.88, 0.9, 1.0, 1.0);
// Atmospheric perspective: scenery behind the stage fades toward the haze colour (lighter, less saturated, lower in contrast) the further
// back it is, from `haze_start` to `haze_end` world units behind the stage plane, at most `haze_max`. `stage_dist` is the camera's
// distance to the stage plane (set every frame by main.gd). The sky is left alone.
uniform float stage_dist = 30.0;
uniform float haze_start = 14.0;  // (4 until art review 6 round 23: the Wind Waker HD reference keeps near scenery vivid and hazes only the distance)
uniform float haze_end = 110.0;
uniform float haze_max = 0.36;
// A gentle S-curve on the values (0 none): darker darks and brighter lights for more depth in the picture. Only the 0..1 range is curved
// (lit windows above white keep their brightness); fighters take half.
uniform float s_curve = 0.0;
// The value hierarchy: the stage and scenery a step quieter than the fighters (less saturation, values drawn toward their middle), and
// the fighters a step crisper (more saturation, a little more contrast), so the fighters are the most vivid, most contrasty thing on
// screen on every stage, as in the genre's leaders. 0 is off.
uniform float stage_quiet = 0.0;
// Far scenery goes a little soft in play (a depth of field done here rather than by the camera, so the effects, name tags and halos drawn
// after this pass stay sharp): `far_blur` pixels (at 1080 lines) at most, from `far_blur_start` world units behind the stage plane over
// `far_blur_ramp`. Only pixels that are far themselves are gathered, so nothing near bleeds into the blur. 0 turns it off (Fast, Balanced).
uniform float far_blur = 0.0;
uniform float far_blur_start = 14.0;
uniform float far_blur_ramp = 30.0;
// A finishing hit: for a moment the picture goes to two tones, the fighters pale against a dark frame (the hit's effects, drawn after
// this, stay in colour on top).
// The soft glow of a bright day (the high-definition cel look): everything brighter than `bloom_threshold` (the sky, clouds, sunlit
// walls, a white hat in the sun) bleeds a wide, soft light over its neighbours, screened on at `bloom` (0 off). Gathered from the screen's
// small mip levels, so it costs a dozen lookups. The fighters take `bloom_fighter` of it, so their own colours stay crisp; the effects
// bloom on their own, after this pass.
uniform float bloom = 0.0;
uniform float bloom_threshold = 0.62;
// (0.5 until art review 5: screened on, the glow pushed a pure body colour toward white, so a yellow fighter on the bright beach read as
// one flat glowing shape with soft edges, its shading gone.)
uniform float bloom_fighter = 0.15;
uniform float impact = 0.0;
uniform vec4 impact_dark : source_color = vec4(0.1, 0.05, 0.13, 1.0);
uniform vec4 impact_light : source_color = vec4(1.0, 0.97, 0.9, 1.0);

void vertex() {
	POSITION = vec4(VERTEX.xy, 1.0, 1.0);
}

float view_depth(vec2 uv, mat4 inv_proj) {
	float d = texture(depth_tex, uv).r;
	vec4 v = inv_proj * vec4(uv * 2.0 - 1.0, d, 1.0);
	return -v.z / v.w;
}

// The object id a surface wrote (see `object_id` in toon.gdshader): the roughness channel holds it as written (checked by drawing it).
float object_at(vec2 uv) {
	return texture(normal_tex, uv).w;
}

float hash(vec2 p) {
	p = fract(p * vec2(123.34, 456.21));
	p += dot(p, p + 45.32);
	return fract(p.x * p.y);
}

float value_noise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	float a = hash(i);
	float b = hash(i + vec2(1.0, 0.0));
	float c = hash(i + vec2(0.0, 1.0));
	float d = hash(i + vec2(1.0, 1.0));
	return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

void fragment() {
	vec2 uv = SCREEN_UV;
	vec3 col = texture(screen_tex, uv).rgb;
	float d0 = view_depth(uv, INV_PROJECTION_MATRIX);
	float blur_k = 0.0;
	if (far_blur > 0.0) {
		float k = smoothstep(far_blur_start, far_blur_start + far_blur_ramp, d0 - stage_dist);
		blur_k = k;
		if (k > 0.0) {
			vec2 r = k * far_blur * (VIEWPORT_SIZE.y / 1080.0) / VIEWPORT_SIZE;
			vec3 sum = col;
			float n = 1.0;
			// Ten taps on a spiral (the golden angle), out to radius r.
			for (int t = 0; t < 10; t++) {
				float f = (float(t) + 0.5) / 10.0;
				float a = float(t) * 2.39996;
				vec2 q = uv + vec2(cos(a), sin(a)) * r * sqrt(f);
				if (view_depth(q, INV_PROJECTION_MATRIX) - stage_dist > far_blur_start) {
					sum += texture(screen_tex, q).rgb;
					n += 1.0;
				}
			}
			col = sum / n;
		}
	}
	// Lines are thinner further away (full width up close, about half on the far scenery).
	float width = line_width * clamp(28.0 / max(d0, 0.001), 0.5, 1.15);
	vec2 px = width * (VIEWPORT_SIZE.y / 1080.0) / VIEWPORT_SIZE;

	// Ink: compare this pixel with its four neighbours.
	vec3 n0 = texture(normal_tex, uv).xyz * 2.0 - 1.0;
	float id0 = object_at(uv);
	bool on_hull = abs(id0 - HULL_ID) < 0.03;
	float edge = 0.0;
	vec2 offs[4] = vec2[](vec2(px.x, 0.0), vec2(-px.x, 0.0), vec2(0.0, px.y), vec2(0.0, -px.y));
	for (int k = 0; k < 4; k++) {
		vec2 q = uv + offs[k];
		float d = view_depth(q, INV_PROJECTION_MATRIX);
		// Only the nearer side draws the line, so it sits on the edge of the object in front.
		float jump = (d - d0) / max(d0, 0.001);
		float id = object_at(q);
		if (on_hull || abs(id - HULL_ID) < 0.03) {
			continue;
		}
		// Silhouettes are the strongest lines; a fold inside one fighter is lighter, so forms read without a tangle of black.
		float inner = abs(id - id0) <= 0.03 && id0 < 0.6 ? inner_lines : 1.0;
		edge = max(edge, smoothstep(depth_edge, depth_edge * 2.0, jump) * inner);
		if (abs(id - id0) > 0.03 && d > d0) {
			edge = max(edge, 1.0);
		}
		// Creases only where both sides are stage or scenery (ids from 0.8 up).
		if (d0 < 400.0 && id0 > 0.8 && id > 0.8) {
			vec3 n = texture(normal_tex, q).xyz * 2.0 - 1.0;
			float crease = 1.0 - dot(normalize(n0), normalize(n));
			edge = max(edge, smoothstep(normal_edge, normal_edge * 1.5, crease) * 0.55);
		}
	}
	edge *= 1.0 - smoothstep(line_fade_start, line_fade_end, d0);
	// (Lines on blurred far scenery soften with it.)
	edge *= 1.0 - 0.6 * blur_k;
	float card_edge = id0 > 0.6 ? edge * paper_edge : 0.0;
	edge *= id0 > 0.6 ? scenery_lines : 1.0;
	// Distance haze on the scenery behind the stage (before the pastel finish and the ink, so far lines fade with it).
	if (d0 < 400.0) {
		float far_k = smoothstep(haze_start, haze_end, d0 - stage_dist) * haze_max;
		// (Toward the stage's own haze colour, not toward white: coloured air, as in a painted landscape, rather than a milky veil.)
		col = mix(col, pastel_haze.rgb, far_k);
		edge *= 1.0 - far_k;
		card_edge *= 1.0 - far_k;
	}
	// The value hierarchy (see `stage_quiet`).
	if (stage_quiet > 0.0 && d0 < 400.0) {
		float yq = dot(col, vec3(0.3, 0.59, 0.11));
		if (id0 < 0.6) {
			// (A small lift only since the muted trial, 2026-10-18: a strong one made the fighters the most saturated, striking thing on screen.)
			col = mix(vec3(yq), col, 1.0 + 0.1 * stage_quiet);
			col = (col - 0.5) * (1.0 + 0.04 * stage_quiet) + 0.5;
		} else {
			col = mix(col, vec3(yq), 0.45 * stage_quiet);
			col = mix(col, vec3(mix(yq, 0.58, 0.5)), 0.1 * stage_quiet);
		}
	}
	// The pastel finish (fighters, ids below 0.6, get half).
	float soften = id0 < 0.6 && d0 < 400.0 ? 0.5 : 1.0;
	float y0 = dot(col, vec3(0.3, 0.59, 0.11));
	col = mix(col, vec3(y0), pastel_desat * soften);
	// Darks are lifted toward the haze, but not the near-black ink of the outlines, which stays ink.
	float not_ink = smoothstep(0.05, 0.16, y0);
	col = mix(col, pastel_haze.rgb, pastel_lift * soften * not_ink * (1.0 - smoothstep(0.0, 0.85, y0)));
	vec3 c01 = clamp(col, 0.0, 1.0);
	col += (c01 * c01 * (3.0 - 2.0 * c01) - c01) * s_curve * soften;
	col = mix(col, ink.rgb, edge * line_strength);
	col = mix(col, col * 1.12 + vec3(0.07, 0.07, 0.06), clamp(card_edge, 0.0, 1.0));

	// A warm paper tone, soft blotches like paper fibre, and a fine print grain.
	col = mix(col, col * tint.rgb, tint_amount);
	vec2 fc = FRAGCOORD.xy / (VIEWPORT_SIZE.y / 1080.0);
	col *= 1.0 + (value_noise(fc / 140.0) - 0.5) * paper;
	col *= 1.0 + (hash(floor(fc)) - 0.5) * grain;

	// Vignette.
	vec2 c = uv - 0.5;
	col *= 1.0 - vignette * smoothstep(0.35, 0.85, length(c * vec2(1.25, 1.0)));

	// The stage's grade: its colour accent on the scenery; fighters take a third of it, so their own colours stay true on every stage.
	float luma = dot(col, vec3(0.3, 0.59, 0.11));
	float grade_k = id0 < 0.6 && d0 < 400.0 ? 0.33 : 1.0;
	col = mix(col, col * mix(grade_shadows.rgb, grade_lights.rgb, smoothstep(0.1, 0.85, luma)), grade_amount * grade_k);

	if (bloom > 0.0) {
		vec2 t4 = 1.5 / (VIEWPORT_SIZE / 16.0);
		vec3 g = textureLod(screen_tex, uv, 4.0).rgb * 0.2;
		g += (textureLod(screen_tex, uv + vec2(t4.x, t4.y), 4.0).rgb + textureLod(screen_tex, uv + vec2(-t4.x, t4.y), 4.0).rgb
			+ textureLod(screen_tex, uv + vec2(t4.x, -t4.y), 4.0).rgb + textureLod(screen_tex, uv + vec2(-t4.x, -t4.y), 4.0).rgb) * 0.1;
		vec2 t6 = 1.2 / (VIEWPORT_SIZE / 64.0);
		g += (textureLod(screen_tex, uv + vec2(t6.x, 0.0), 6.0).rgb + textureLod(screen_tex, uv - vec2(t6.x, 0.0), 6.0).rgb
			+ textureLod(screen_tex, uv + vec2(0.0, t6.y), 6.0).rgb + textureLod(screen_tex, uv - vec2(0.0, t6.y), 6.0).rgb) * 0.1;
		float gy = dot(g, vec3(0.3, 0.59, 0.11));
		// A soft knee, so the glow eases in rather than switching on at the threshold.
		float over = smoothstep(bloom_threshold, 1.0, gy) * min(gy, 1.6);
		vec3 glow = g / max(gy, 0.001) * over;
		float bk = bloom * (id0 < 0.6 && d0 < 400.0 ? bloom_fighter : 1.0);
		col = col + clamp(glow, 0.0, 1.0) * bk * (1.0 - clamp(col, 0.0, 1.0));
	}

	// The finishing hit's two tones: fighters (ids below 0.6) pale, everything else dark, the ink lines kept.
	if (impact > 0.0) {
		float fighter = id0 < 0.6 && d0 < 400.0 ? 1.0 : 0.0;
		vec3 two = mix(impact_dark.rgb, impact_light.rgb, fighter);
		two = mix(two, impact_dark.rgb * 0.6, edge * fighter);
		col = mix(col, two, impact);
	}

	ALBEDO = col;
}
