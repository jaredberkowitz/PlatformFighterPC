shader_type spatial;
// The city backdrop's buildings (scripts/stage_art.gd `_skyline_layer`): flat-shaded like the rest of the far scenery, with their windows
// painted on rather than built of boxes. Each face carries its own coordinates: UV is the place on the face in world units (across from
// its left edge, up from the street), UV2 the face's width and the building's height, and COLOR.r a seed for the building and COLOR.g what
// the face is (0 a wall with windows, 1 a roof or a rooftop box, plain). Lit windows are drawn brighter than white (`lit_energy`), so they
// glow softly; the far layers get less.
render_mode unshaded, cull_back, shadows_disabled;

uniform vec4 base_shade : source_color = vec4(0.16, 0.14, 0.3, 1.0);
uniform vec4 roof_shade : source_color = vec4(0.22, 0.19, 0.36, 1.0);
uniform float lit_energy = 3.5;        // how many times white a lit window is (above the glow threshold, 2.3, to bloom)
uniform float lit_share = 0.42;        // the share of windows lit
uniform vec2 cell = vec2(2.0, 2.6);    // one window's bay (world units)
uniform vec2 pane = vec2(0.72, 1.0);   // the window itself
uniform float first_row = 1.0;         // street to the first bay
uniform float parapet = 0.55;          // the lighter cap along the top

float hash(vec2 p) {
	p = fract(p * vec2(123.34, 456.21));
	p += dot(p, p + 45.32);
	return fract(p.x * p.y);
}

void fragment() {
	// (Rounded to a whole number: an interpolated colour wobbles in its last bits, which the hash would turn into stripes.)
	float seed = floor(COLOR.r * 255.0 + 0.5);
	bool plain = COLOR.g > 0.5;
	vec2 at = UV;
	float width = max(UV2.x, 0.001);
	float height = max(UV2.y, 0.001);
	vec3 base = plain ? roof_shade.rgb : base_shade.rgb;
	// Darker toward the street, a little lighter toward the top.
	vec3 col = base * mix(0.72, 1.06, smoothstep(0.0, 16.0, at.y));
	if (!plain) {
		// The parapet cap.
		float cap = step(height - parapet, at.y);
		col = mix(col, base * 1.28, cap);
		// Bays fitted to the face so its windows sit evenly; rows from the street up to below the parapet.
		float cols = max(1.0, floor(width / cell.x));
		float bay_w = width / cols;
		vec2 bay = vec2(floor(at.x / bay_w), floor((at.y - first_row) / cell.y));
		vec2 in_bay = vec2(fract(at.x / bay_w) * bay_w, at.y - first_row - bay.y * cell.y);
		float rows = floor((height - parapet - first_row) / cell.y);
		bool inside = at.y > first_row && bay.y < rows && bay.y >= 0.0;
		// A faint floor band every fourth floor.
		float band = inside && mod(bay.y, 4.0) == 3.0 && in_bay.y > cell.y - 0.18 ? 1.0 : 0.0;
		col = mix(col, base * 1.18, band);
		if (inside) {
			vec2 c0 = vec2((bay_w - pane.x) * 0.5, (cell.y - pane.y) * 0.45);
			vec2 c1 = c0 + pane;
			float in_x = step(c0.x, in_bay.x) * step(in_bay.x, c1.x);
			float in_y = step(c0.y, in_bay.y) * step(in_bay.y, c1.y);
			// The frame round the pane (a touch lighter than the wall) and the sill under it.
			float frame_x = step(c0.x - 0.08, in_bay.x) * step(in_bay.x, c1.x + 0.08);
			float frame_y = step(c0.y - 0.08, in_bay.y) * step(in_bay.y, c1.y + 0.06);
			col = mix(col, base * 1.22, frame_x * frame_y);
			float sill = step(c0.x - 0.12, in_bay.x) * step(in_bay.x, c1.x + 0.12) * step(c0.y - 0.16, in_bay.y) * step(in_bay.y, c0.y - 0.06);
			col = mix(col, base * 1.45, sill);
			float h = hash(bay + vec2(seed * 0.37, seed * 0.13));
			float lit = step(1.0 - lit_share, h);
			// Most lit windows warm, a few pale blue (a screen), a rare magenta; each a little brighter or dimmer.
			float pick = hash(bay * 1.7 + seed * 0.31);
			// (Saturated, so the hue survives being pushed past white: a pale yellow at four times white tone-maps to plain white.)
			vec3 warm = vec3(1.0, 0.66, 0.26);
			vec3 glow = pick < 0.78 ? warm : (pick < 0.94 ? vec3(0.42, 0.68, 1.0) : vec3(1.0, 0.36, 0.78));
			float vary = 0.78 + 0.44 * hash(bay * 3.1 + seed * 0.07);
			// A lit pane is brightest in its upper part (the room's light), dark ones keep a faint cool reflection.
			float room = mix(0.8, 1.0, smoothstep(0.0, pane.y, in_bay.y - c0.y));
			vec3 dark = mix(base * 0.62, vec3(0.32, 0.36, 0.6), 0.18);
			vec3 pane_col = lit > 0.5 ? glow * lit_energy * vary * room : dark;
			col = mix(col, pane_col, in_x * in_y);
		}
	}
	ALBEDO = col;
	// Scenery's object id (post.gdshader reads it from the roughness).
	ROUGHNESS = 0.95;
}
