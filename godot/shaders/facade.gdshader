shader_type spatial;
// The city backdrop's buildings (scripts/stage_art.gd `_skyline_layer`): flat-shaded like the rest of the far scenery, with their windows
// painted on rather than built of boxes. Each face carries its own coordinates: UV is the place on the face in world units (across from
// its left edge, up from the street), UV2 the face's width and the building's height, and COLOR.r a seed for the building, COLOR.g what
// the face is (0 a wall with windows, 1 a roof or a rooftop box, plain) and COLOR.b a lit sign. Lit windows are drawn brighter than white
// (`lit_energy`), so they glow softly; the far layers get less.
//
// No two buildings share a pattern: each one's seed picks its bay and pane size, how many of its windows are lit and how bright, and about
// one in five has long office strips instead of separate windows. A few windows switch on or off now and then. `detail` 0 (the far layer)
// leaves out frames, sills and floor bands, which would only be noise at that distance.
render_mode unshaded, cull_back, shadows_disabled;

uniform vec4 base_shade : source_color = vec4(0.16, 0.14, 0.3, 1.0);
uniform vec4 roof_shade : source_color = vec4(0.22, 0.19, 0.36, 1.0);
uniform float lit_energy = 3.5;        // how many times white a lit window is (above the glow threshold, 2.3, to bloom)
uniform float lit_share = 0.42;        // the share of windows lit, on average
uniform vec2 cell = vec2(2.0, 2.6);    // a window's bay, on average (world units)
uniform vec2 pane = vec2(0.72, 1.0);   // the window itself, on average
uniform float first_row = 1.0;         // street to the first bay
uniform float parapet = 0.55;          // the lighter cap along the top
uniform float detail = 1.0;            // frames, sills and floor bands (0 for the far layer)
uniform float sign_energy = 3.0;       // a sign's neon, times white

float hash(vec2 p) {
	p = fract(p * vec2(123.34, 456.21));
	p += dot(p, p + 45.32);
	return fract(p.x * p.y);
}

// A lit sign: a neon border round a dark panel with a few bars of lettering, in one of three colours; now and then it stutters off.
vec3 neon_sign(vec2 at, vec2 size, float seed) {
	float pick = hash(vec2(seed, 9.1));
	vec3 tube = pick < 0.4 ? vec3(1.0, 0.3, 0.7) : (pick < 0.75 ? vec3(0.3, 0.9, 1.0) : vec3(1.0, 0.6, 0.2));
	vec2 e = min(at, size - at);
	float border = 1.0 - step(0.26, min(e.x, e.y));
	// Lettering: a row of short bars across the middle, their lengths from the seed.
	float letters = 0.0;
	float n = max(2.0, floor(size.x / 0.9));
	float k = floor(at.x / (size.x / n));
	float in_k = fract(at.x / (size.x / n));
	float tall = mix(0.4, 0.7, hash(vec2(seed, k)));
	letters = step(0.15, in_k) * step(in_k, 0.85) * step(abs(at.y - size.y * 0.5), size.y * tall * 0.5) * step(0.45, e.x) * step(0.4, e.y);
	float stutter = step(0.985, fract(sin(floor(TIME * 7.0) + seed * 13.0) * 4375.5)) * step(0.5, hash(vec2(seed, 2.0)));
	// The panel glows faintly with its own colour, so the sign reads as one lit shape from far away.
	vec3 panel = mix(vec3(0.08, 0.06, 0.14), tube * 0.6, 0.35);
	vec3 col = mix(panel, tube * sign_energy, max(border, letters * 0.85));
	return mix(col, panel, stutter);
}

void fragment() {
	// (Rounded to a whole number: an interpolated colour wobbles in its last bits, which the hash would turn into stripes.)
	float seed = floor(COLOR.r * 255.0 + 0.5);
	bool plain = COLOR.g > 0.5;
	vec2 at = UV;
	float width = max(UV2.x, 0.001);
	float height = max(UV2.y, 0.001);
	if (COLOR.b > 0.5) {
		ALBEDO = neon_sign(at, vec2(width, height), seed);
		ROUGHNESS = 0.95;
	} else {
		vec3 base = plain ? roof_shade.rgb : base_shade.rgb;
		// Each building's own shade, a touch lighter or darker than its layer's.
		base *= mix(0.88, 1.12, hash(vec2(seed, 6.6)));
		// Darker toward the street, a little lighter toward the top.
		vec3 col = base * mix(0.72, 1.06, smoothstep(0.0, 16.0, at.y));
		if (!plain) {
			// This building's pattern.
			vec2 bay_size = cell * vec2(mix(0.8, 1.3, hash(vec2(seed, 1.1))), mix(0.88, 1.16, hash(vec2(seed, 2.2))));
			bool strips = hash(vec2(seed, 3.3)) < 0.2;
			float share = clamp(lit_share * mix(0.4, 1.55, hash(vec2(seed, 5.5))), 0.06, 0.85);
			// The parapet cap.
			float cap = step(height - parapet, at.y);
			col = mix(col, base * 1.28, cap);
			// Bays fitted to the face so its windows sit evenly; rows from the street up to below the parapet.
			float cols = max(1.0, floor(width / bay_size.x));
			float bay_w = width / cols;
			vec2 pane_size = pane * vec2(mix(0.75, 1.35, hash(vec2(seed, 4.4))), mix(0.8, 1.2, hash(vec2(seed, 7.7))));
			if (strips) {
				// A row of wide panes with thin mullions between them, a little shorter than a window.
				pane_size = vec2(bay_w - 0.22, pane.y * 0.62);
			}
			pane_size = min(pane_size, vec2(bay_w - 0.06, bay_size.y - 0.4));
			vec2 bay = vec2(floor(at.x / bay_w), floor((at.y - first_row) / bay_size.y));
			vec2 in_bay = vec2(fract(at.x / bay_w) * bay_w, at.y - first_row - bay.y * bay_size.y);
			float rows = floor((height - parapet - first_row) / bay_size.y);
			bool inside = at.y > first_row && bay.y < rows && bay.y >= 0.0;
			// A faint floor band every fourth floor.
			float band = inside && mod(bay.y, 4.0) == 3.0 && in_bay.y > bay_size.y - 0.18 ? detail : 0.0;
			col = mix(col, base * 1.18, band);
			if (inside) {
				vec2 c0 = vec2((bay_w - pane_size.x) * 0.5, (bay_size.y - pane_size.y) * 0.45);
				vec2 c1 = c0 + pane_size;
				float in_x = step(c0.x, in_bay.x) * step(in_bay.x, c1.x);
				float in_y = step(c0.y, in_bay.y) * step(in_bay.y, c1.y);
				if (detail > 0.0 && !strips) {
					// The frame round the pane (a touch lighter than the wall) and the sill under it.
					float frame_x = step(c0.x - 0.08, in_bay.x) * step(in_bay.x, c1.x + 0.08);
					float frame_y = step(c0.y - 0.08, in_bay.y) * step(in_bay.y, c1.y + 0.06);
					col = mix(col, base * 1.22, frame_x * frame_y * detail);
					float sill = step(c0.x - 0.12, in_bay.x) * step(in_bay.x, c1.x + 0.12) * step(c0.y - 0.16, in_bay.y) * step(in_bay.y, c0.y - 0.06);
					col = mix(col, base * 1.45, sill * detail);
				}
				// A strip's lit stretch is a run of a few bays (an office floor), not single windows.
				vec2 key = strips ? vec2(floor(bay.x / 2.0), bay.y) : bay;
				float h = hash(key + vec2(seed * 0.37, seed * 0.13));
				float lit = step(1.0 - share, h);
				// Now and then a window switches (one in forty, each on its own slow clock).
				float flicker_pick = hash(key * 2.3 + seed * 0.19);
				if (flicker_pick < 0.025) {
					lit = step(0.5, fract(TIME * 0.05 + flicker_pick * 37.0));
				}
				// Most lit windows warm, a few pale blue (a screen), a rare magenta; each a little brighter or dimmer.
				float pick = hash(key * 1.7 + seed * 0.31);
				// (Saturated, so the hue survives being pushed past white: a pale yellow at four times white tone-maps to plain white.)
				vec3 warm = vec3(1.0, 0.66, 0.26);
				vec3 glow = pick < 0.78 ? warm : (pick < 0.94 ? vec3(0.42, 0.68, 1.0) : vec3(1.0, 0.36, 0.78));
				float vary = 0.78 + 0.44 * hash(key * 3.1 + seed * 0.07);
				// A lit pane is brightest in its upper part (the room's light), dark ones keep a faint cool reflection.
				float room = mix(0.8, 1.0, smoothstep(0.0, pane_size.y, in_bay.y - c0.y));
				vec3 dark = mix(base * 0.62, vec3(0.32, 0.36, 0.6), 0.18);
				// (A strip's long runs of light would outshine everything at full strength: they glow at half.)
				vec3 pane_col = lit > 0.5 ? glow * lit_energy * vary * room * (strips ? 0.5 : 1.0) : dark;
				col = mix(col, pane_col, in_x * in_y);
			}
		}
		ALBEDO = col;
		// Scenery's object id (post.gdshader reads it from the roughness).
		ROUGHNESS = 0.95;
	}
}
