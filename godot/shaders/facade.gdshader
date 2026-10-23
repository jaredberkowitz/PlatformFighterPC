shader_type spatial;
// The city backdrop's buildings (scripts/stage_art.gd `_skyline_layer`): flat-shaded like the rest of the far scenery, with their windows
// painted on rather than built of boxes. Each face carries its own coordinates: UV is the place on the face in world units (across from
// its left edge, up from the street), UV2 the face's width and the building's height, and COLOR.r a seed for the building, COLOR.g what
// the face is (0 a wall with windows, 1 a roof or a rooftop box, plain) and COLOR.b a lit sign. Lit windows are drawn brighter than white
// (`lit_energy`), so they glow softly; the far layers get less.
//
// No two buildings share a pattern: each one's seed picks its bay and pane size, its hue, how many of its windows are lit (in clusters)
// and how bright, and about one in eight has office strips (broken by piers and service floors) instead of separate windows. A few
// windows switch on or off now and then. `detail` 0 (the far layer)
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
uniform float moon_edge = 1.0;         // how strong the moonlit edge up each front's left corner is

// The quiet behind the fight (fidelity review 5's fourth item: the towers right behind the fighters were the busiest background in the
// game): within `calm_half` of the middle, fewer windows lit and the facade pulled toward its own wall colour, by `calm` (0 off),
// fading out over the outer two-fifths of the band.
uniform float calm = 0.0;
uniform float calm_half = 26.0;

// Haze between the rows (fidelity review 6, the city's depth): the foot of the layer, from `street_y` up `foot_height`, fades into the
// street's lit haze, `foot_color`, by `foot_haze` (0 off).
uniform float foot_haze = 0.0;
// The city's glow catching the towers (fidelity review 7, shaped light): a warm edge up each front's corner away from the moon, strongest
// low where the street's light reaches, and along the parapet's lip; `glow_edge` how much (0 off), in `glow_edge_color`.
uniform float glow_edge = 0.0;
uniform vec4 glow_edge_color : source_color = vec4(1.0, 0.62, 0.5, 1.0);
uniform float foot_height = 10.0;
uniform vec4 foot_color : source_color = vec4(0.6, 0.4, 0.6, 1.0);
uniform float street_y = 0.0;

varying vec3 wn;
varying float wx;
varying float wy;

void vertex() {
	wn = (MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz;
	vec3 wp = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	wx = wp.x;
	wy = wp.y;
}

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
	// A real word in neon tubes (the owner, round 41): SHOP or CAFE by the sign's seed, four letters, each built from segments like a
	// seven-segment display. Bits: 1 top, 2 middle, 4 bottom, 8 left upper, 16 left lower, 32 right upper, 64 right lower.
	float n = 4.0;
	// (Across the sign's inner width, inside its border: laid across the whole sign, the last letter's right-hand strokes fell in the
	// border's clip and a narrow sign read SHOF, the gauntlet's round 29.)
	float lx = (at.x - 0.5) / max(size.x - 1.0, 0.1);
	int k = int(clamp(floor(lx * n), 0.0, 3.0));
	float in_k = fract(clamp(lx, 0.0, 0.9999) * n);
	// (Four words, not two, the judge's round 39: two SHOP signs in one frame read as a duplicated asset. OPEN and SALE join them, every
	// letter a capital: a lower-case d in DELI read as unfinished, round 40.)
	int word = int(clamp(floor(hash(vec2(seed, 5.0)) * 4.0), 0.0, 3.0));
	int glyphs[16] = int[](79, 122, 125, 59, 29, 123, 27, 31, 125, 59, 31, 121, 79, 123, 28, 31);
	int mask = glyphs[word * 4 + k];
	float v = (at.y - size.y * 0.5) / (size.y * 0.62 * 0.5) * 0.5 + 0.5;
	float inside = step(0.14, in_k) * step(in_k, 0.86) * step(0.0, v) * step(v, 1.0) * step(0.0, lx) * step(lx, 1.0) * step(0.4, e.y);
	float u = (in_k - 0.14) / 0.72;
	float t = 0.2;
	float strokes = 0.0;
	strokes += float((mask >> 0) & 1) * step(1.0 - t, v);
	strokes += float((mask >> 1) & 1) * step(abs(v - 0.5), t * 0.5);
	strokes += float((mask >> 2) & 1) * step(v, t);
	strokes += float((mask >> 3) & 1) * step(u, t) * step(0.5, v);
	strokes += float((mask >> 4) & 1) * step(u, t) * step(v, 0.5);
	strokes += float((mask >> 5) & 1) * step(1.0 - t, u) * step(0.5, v);
	strokes += float((mask >> 6) & 1) * step(1.0 - t, u) * step(v, 0.5);
	letters = inside * min(strokes, 1.0);
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
	// A glass tower (blue 0.3): a cool curtain wall in bands of glass, lighter toward its top where it takes the sky (the owner's city
	// plan, 2026-10-19: a skyline of one kind of slab read as a pattern).
	bool glass = COLOR.b > 0.2 && COLOR.b < 0.5;
	vec2 at = UV;
	float width = max(UV2.x, 0.001);
	float height = max(UV2.y, 0.001);
	if (COLOR.b > 0.5) {
		ALBEDO = neon_sign(at, vec2(width, height), seed);
		ROUGHNESS = 0.95;
	} else {
		vec3 base = plain ? roof_shade.rgb : base_shade.rgb;
		if (glass && !plain) {
			base = base * vec3(0.78, 0.98, 1.3) + vec3(0.0, 0.02, 0.05);
			base *= mix(0.9, 1.25, smoothstep(0.0, height, at.y));
		}
		// Each building's own shade, a touch lighter or darker than its layer's, and its own muted hue: plum, indigo, mauve or blue-grey
		// (so neighbouring buildings read as different buildings, not one wall).
		base *= mix(0.88, 1.12, hash(vec2(seed, 6.6)));
		float hue_pick = hash(vec2(seed, 8.8));
		float grey = dot(base, vec3(0.3, 0.59, 0.11));
		if (hue_pick < 0.3) {
			base *= vec3(1.08, 0.93, 0.98);
		} else if (hue_pick < 0.55) {
			base *= vec3(0.9, 0.95, 1.08);
		} else if (hue_pick < 0.78) {
			base *= vec3(1.14, 1.0, 1.06);
		} else {
			base = mix(base, vec3(grey), 0.4) * vec3(0.96, 1.02, 1.1);
		}
		// Darker toward the street, a little lighter toward the top.
		vec3 col = base * mix(0.72, 1.06, smoothstep(0.0, 16.0, at.y));
		// The side walls a step darker than the fronts (the moon is up on the left, so the right sides darkest): the buildings read as
		// blocks with depth, not flat cut-outs (the owner, 2026-10-17: basic shapes, no real detail).
		float side = abs(wn.x) > 0.5 ? (wn.x > 0.0 ? 0.7 : 0.84) : 1.0;
		col *= side;
		// The moonlit edge: a thin bright strip up the corner of each front facing the moon, cool, so each tower's outline reads as a lit
		// block against the one behind it (art review 9: the skyline towers were flat boxes). Not on the side walls or the roofs.
		if (!plain && abs(wn.z) > 0.5) {
			float edge = 1.0 - smoothstep(0.12, 0.3, at.x);
			col = mix(col, base * vec3(1.55, 1.6, 1.95) + vec3(0.02, 0.03, 0.07), edge * moon_edge);
		}
		// Each building's wall (`material`): brick courses, horizontal stone bands, or plain render with pilasters between the bays.
		float material = hash(vec2(seed, 11.1));
		if (!plain && detail > 0.0) {
			if (material < 0.4) {
				float course = floor(at.y / 0.3);
				float bx = fract((at.x + mod(course, 2.0) * 0.3) / 0.6);
				float mortar = max(step(fract(at.y / 0.3), 0.12), step(bx, 0.06));
				col *= 1.0 - 0.07 * mortar - 0.04 * hash(vec2(floor((at.x + mod(course, 2.0) * 0.3) / 0.6), course) + seed);
			} else if (material < 0.7) {
				float groove = step(fract(at.y / 0.9), 0.08);
				col *= 1.0 - 0.1 * groove * (at.y < 4.2 ? 1.6 : 1.0);
			}
		}
		if (!plain) {
			// This building's pattern.
			vec2 bay_size = cell * vec2(mix(0.8, 1.3, hash(vec2(seed, 1.1))), mix(0.88, 1.16, hash(vec2(seed, 2.2))));
			bool strips = glass || hash(vec2(seed, 3.3)) < 0.12;
			// A strip building's runs are broken by a pier every few bays and a plain service floor every sixth, so its face is not one
			// stack of identical bands.
			float pier_every = floor(mix(3.0, 5.99, hash(vec2(seed, 9.9))));
			float share = clamp(lit_share * mix(0.4, 1.55, hash(vec2(seed, 5.5))), 0.06, 0.85);
			float ck = calm * (1.0 - smoothstep(calm_half * 0.6, calm_half, abs(wx)));
			share *= 1.0 - 0.75 * ck;
			// The parapet cap, and the cornice's shadow on the wall just under the roofline.
			float cap = step(height - parapet, at.y);
			col = mix(col, base * side * 1.28, cap);
			float eave = step(height - parapet - 0.5, at.y) * (1.0 - cap);
			col *= 1.0 - 0.22 * eave * smoothstep(height - parapet - 0.5, height - parapet, at.y);
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
			// Pilasters: a lighter strip up between the bays, with a shaded edge, on the rendered buildings.
			if (material >= 0.7 && detail > 0.0 && at.y < height - parapet) {
				float px = fract(at.x / bay_w) * bay_w;
				float pil = 1.0 - step(0.16, min(px, bay_w - px));
				float pil_edge = step(0.16, px) * (1.0 - step(0.24, px));
				col = mix(col, base * side * 1.2, pil);
				col *= 1.0 - 0.12 * pil_edge;
			}
			// A faint floor band every fourth floor.
			float band = inside && mod(bay.y, 4.0) == 3.0 && in_bay.y > bay_size.y - 0.18 ? detail : 0.0;
			col = mix(col, base * 1.18, band);
			if (inside) {
				vec2 c0 = vec2((bay_w - pane_size.x) * 0.5, (bay_size.y - pane_size.y) * 0.45);
				vec2 c1 = c0 + pane_size;
				float in_x = step(c0.x, in_bay.x) * step(in_bay.x, c1.x);
				float in_y = step(c0.y, in_bay.y) * step(in_bay.y, c1.y);
				if (strips && (mod(bay.x, pier_every) == pier_every - 1.0 || mod(bay.y, 6.0) == 5.0)) {
					in_x = 0.0;
				}
				// Arched windows on some brick buildings: the pane's top rounded off.
				bool arched = material < 0.4 && hash(vec2(seed, 12.2)) < 0.5 && !strips;
				if (arched) {
					float r = pane_size.x * 0.5;
					vec2 q = in_bay - vec2((c0.x + c1.x) * 0.5, c1.y - r);
					if (q.y > 0.0) {
						in_y *= step(length(q), r);
					}
				}
				if (detail > 0.0 && !strips) {
					// The frame round the pane (a touch lighter than the wall), the sill under it standing out, and a lintel over it.
					float frame_x = step(c0.x - 0.08, in_bay.x) * step(in_bay.x, c1.x + 0.08);
					float frame_y = step(c0.y - 0.08, in_bay.y) * step(in_bay.y, c1.y + 0.06);
					col = mix(col, base * side * 1.22, frame_x * frame_y * detail);
					float sill = step(c0.x - 0.12, in_bay.x) * step(in_bay.x, c1.x + 0.12) * step(c0.y - 0.16, in_bay.y) * step(in_bay.y, c0.y - 0.06);
					col = mix(col, base * side * 1.45, sill * detail);
					// (The sill's shadow on the wall under it.)
					float under = step(c0.x - 0.12, in_bay.x) * step(in_bay.x, c1.x + 0.12) * step(c0.y - 0.26, in_bay.y) * step(in_bay.y, c0.y - 0.16);
					col *= 1.0 - 0.18 * under * detail;
					if (!arched) {
						float lintel = step(c0.x - 0.1, in_bay.x) * step(in_bay.x, c1.x + 0.1) * step(c1.y + 0.06, in_bay.y) * step(in_bay.y, c1.y + 0.18);
						col = mix(col, base * side * (material < 0.4 ? 1.35 : 1.15), lintel * detail);
					}
				}
				// A strip's lit stretch is a run of a few bays (an office floor), not single windows.
				vec2 key = strips ? vec2(floor(bay.x / pier_every), bay.y) : bay;
				float h = hash(key + vec2(seed * 0.37, seed * 0.13));
				// Lit windows come in clusters (a busy floor, a dark wing) rather than evenly salted over the face: each block of a few
				// bays and two floors has its own occupancy.
				vec2 group = floor(bay / vec2(floor(mix(2.0, 4.99, hash(vec2(seed, 10.1)))), 2.0));
				float occupancy = mix(0.15, 1.75, hash(group + vec2(seed * 0.71, seed * 0.29)));
				float lit = step(1.0 - clamp(share * occupancy, 0.0, 0.95), h);
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
				vec2 pu = (in_bay - c0) / max(pane_size, vec2(0.001));
				if (detail > 0.0 && !strips) {
					if (lit > 0.5) {
						// Inside a lit room: curtains drawn to the sides, or blinds half down, or nothing.
						float inner = hash(key * 4.7 + seed * 0.11);
						if (inner < 0.35) {
							float curtain = 1.0 - step(0.2, min(pu.x, 1.0 - pu.x));
							pane_col *= mix(1.0, 0.55, curtain);
						} else if (inner < 0.6) {
							float blind = step(0.55, pu.y) * step(0.5, fract(pu.y * 9.0));
							pane_col *= 1.0 - 0.35 * blind;
						}
					} else {
						// A dark pane catches a faint diagonal glint of the sky.
						float glint = step(abs(pu.x + pu.y * 0.6 - 0.7), 0.08);
						pane_col = mix(pane_col, pane_col * 1.5 + vec3(0.03, 0.03, 0.06), glint * 0.6);
					}
					// Mullions: a cross of frame through the pane (two panes over two), in the frame's colour.
					float mull = step(abs(pu.x - 0.5), 0.045) + step(abs(pu.y - 0.58), 0.035);
					pane_col = mix(pane_col, base * side * 1.1, clamp(mull, 0.0, 1.0));
				}
				col = mix(col, pane_col, in_x * in_y);
			}
			col = mix(col, base * side, ck * 0.4);
		}
		if (glow_edge > 0.0 && !plain && abs(wn.z) > 0.5) {
			// (Wide enough to read at play distance: a strip a third of a unit across was a pixel or two.)
			float corner = 1.0 - smoothstep(0.35, 1.0, width - at.x);
			float low = mix(1.0, 0.35, smoothstep(0.0, height, at.y));
			float lip = step(height - parapet - 0.35, at.y) * (1.0 - step(height - parapet + 0.15, at.y));
			col = mix(col, glow_edge_color.rgb * 1.15, clamp((corner * low + lip * 0.7) * glow_edge, 0.0, 0.85));
		}
		ALBEDO = col;
		// Scenery's object id (post.gdshader reads it from the roughness).
		ROUGHNESS = 0.95;
	}
	if (foot_haze > 0.0) {
		float f = 1.0 - smoothstep(0.0, foot_height, wy - street_y);
		ALBEDO = mix(ALBEDO, foot_color.rgb, foot_haze * f);
	}
}
