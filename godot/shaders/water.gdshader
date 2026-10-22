shader_type spatial;
// The sea (scripts/stage_art.gd `Waves`): banded, not photographic. Shallow to deep colour in a few cel bands with distance, two slow layers
// of flow streaks drifting across, a gentle swell, small glints that come and go, and a ruffled band of foam wherever something stands
// in the water (read from the depth texture: the distant islands, the lighthouse, the boat). It is drawn after the finishing pass
// (shaders/post.gdshader), so it fades into the stage's haze itself, the same way the scenery does. `cheap` (the Fast tier) keeps only
// the bands.
render_mode unshaded, cull_disabled, depth_draw_always, shadows_disabled, fog_disabled;

uniform sampler2D depth_tex : hint_depth_texture, filter_nearest;
uniform vec4 shallow : source_color = vec4(0.12, 0.74, 0.88, 1.0);
uniform vec4 deep : source_color = vec4(0.04, 0.36, 0.84, 1.0);
uniform vec4 streak : source_color = vec4(0.5, 0.88, 1.0, 1.0);
uniform vec4 foam : source_color = vec4(0.95, 0.99, 1.0, 1.0);
uniform vec4 haze : source_color = vec4(0.22, 0.56, 0.86, 1.0);
uniform float haze_max = 0.15;
uniform float foam_width = 2.0;    // world units of foam out from where something meets the water
uniform bool cheap = false;
// The sun's direction (toward it) and the sky's horizon colour (the judge's round 41: the sea read as a painted floor): a broken path of
// glints on the water under the sun, and the far sea brightening toward the horizon as water does at a low angle, softening the line
// where it meets the sky. `glint_path` 0 turns the path off.
uniform vec3 sun_dir = vec3(-0.42, 0.25, -0.88);
uniform vec4 horizon : source_color = vec4(0.6, 0.86, 1.0, 1.0);
uniform float glint_path = 0.0;
// The water pass (2026-10-22, fidelity review 5's first item: the sea read as one bright cyan sheet with stripes). `look` 0 is the sea
// as it was; 1 clear water: colour by depth from the shore out, a soft caustic net near the stage, fine ripples, no crest rows; 2 clear
// water with the reflections of what stands in and over it (the pier, the boats, the islands, the lighthouse), traced on the screen;
// 3 cut paper: the sea in tiers of card, each a scalloped sheet a step deeper than the one nearer, its cut edge catching the light,
// sliding slowly, with the reflections.
uniform int look = 0;
uniform sampler2D screen_tex : hint_screen_texture, filter_linear_mipmap;
varying vec3 world_pos;

float hash(vec2 p) {
	p = fract(p * vec2(123.34, 456.21));
	p += dot(p, p + 45.32);
	return fract(p.x * p.y);
}

float noise(vec2 p) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	f = f * f * (3.0 - 2.0 * f);
	return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), f.x), f.y);
}

vec2 hash2(vec2 p) {
	return vec2(hash(p), hash(p + vec2(17.3, 5.1)));
}

// A slowly moving cell net: how far a point is from the edge between its two nearest cells (0 on the edge).
float cell_edge(vec2 p, float t) {
	vec2 i = floor(p);
	vec2 f = fract(p);
	float d1 = 8.0;
	float d2 = 8.0;
	for (int y = -1; y <= 1; y++) {
		for (int x = -1; x <= 1; x++) {
			vec2 o = vec2(float(x), float(y));
			vec2 h = hash2(i + o);
			vec2 c = o + 0.5 + 0.38 * sin(t * 0.7 + 6.2831 * h) - f;
			float d = dot(c, c);
			if (d < d1) {
				d2 = d1;
				d1 = d;
			} else if (d < d2) {
				d2 = d;
			}
		}
	}
	return sqrt(d2) - sqrt(d1);
}

void vertex() {
	vec3 w = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	if (!cheap) {
		VERTEX.y += sin(w.x * 0.07 + TIME * 0.6) * 0.18 + sin(w.z * 0.11 + w.x * 0.02 + TIME * 0.8) * 0.12;
	}
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

vec3 linear_depth_pos(vec2 uv, mat4 inv_proj) {
	float d = texture(depth_tex, uv).r;
	vec4 v = inv_proj * vec4(uv * 2.0 - 1.0, d, 1.0);
	return v.xyz / v.w;
}

void fragment() {
	if (look > 0) {
		float back = max(-world_pos.z, 0.0);
		float far_k = smoothstep(0.0, 170.0, back);
		vec2 q = world_pos.xz;
		// Colour by depth: a clear turquoise over the sand near the stage, a deeper teal-blue through the middle, paler toward the
		// horizon where it takes the sky.
		// (Written as display colours and taken to linear light, as the uniforms are: written straight, they read milky.)
		vec3 near_c = pow(vec3(0.16, 0.7, 0.74), vec3(2.2));
		vec3 mid_c = pow(vec3(0.06, 0.46, 0.66), vec3(2.2));
		vec3 far_c = mix(pow(vec3(0.16, 0.46, 0.7), vec3(2.2)), horizon.rgb, 0.25);
		vec3 col = mix(near_c, mid_c, smoothstep(4.0, 45.0, back));
		col = mix(col, far_c, smoothstep(45.0, 170.0, back));
		// Broad soft patches, deeper and lighter, drifting: the water's large forms.
		float sw = noise(q * vec2(0.02, 0.06) + vec2(TIME * 0.025, 0.0)) * 0.6 + noise(q * vec2(0.05, 0.14) + vec2(-TIME * 0.04, 2.0)) * 0.4;
		col *= mix(0.9, 1.08, sw);
		if (look == 3) {
			// Paper waves: the sea as overlapping sheets of card, each a tier a step further out, its far edge cut into rounded wave
			// caps. Each sheet is shaded across (lighter toward its cut edge, as a card curling toward the light), its edge is a lit lip,
			// and it lays a soft shadow on the sheet behind it. Big near, wider and calmer toward the horizon, sliding slowly.
			// (The owner on the first paper sea: it read as lines. That one lit the sheet behind each edge, not the sheet's own lip, and
			// its tiers were one tone.)
			float bs = -world_pos.z;
			float width = mix(14.0, 36.0, smoothstep(0.0, 150.0, max(bs, 0.0)));
			float tier = floor(bs / width);
			float b = bs;
			for (int it = 0; it < 2; it++) {
				float ph = tier * 1.7 + TIME * 0.12 * (mod(tier, 2.0) * 2.0 - 1.0);
				float cap = pow(abs(sin(world_pos.x * 0.12 + ph)), 0.55);
				b = bs + width * 0.34 * (1.0 - cap);
				tier = floor(b / width);
			}
			float u = fract(b / width);
			float calm = 1.0 - smoothstep(70.0, 190.0, bs);
			// Each sheet its own tone, and shaded across toward its edge.
			col *= 1.0 - 0.08 * mod(tier, 2.0) * calm;
			col *= mix(1.0, mix(0.93, 1.07, smoothstep(0.15, 1.0, u)), calm);
			// The nearer sheet's shadow on this one, just past its edge (u from 0), soft.
			float shadow = (1.0 - smoothstep(0.0, 0.16, u));
			col = mix(col, col * vec3(0.7, 0.8, 0.9), shadow * 0.55 * calm);
			// This sheet's own cut edge, lit (u toward 1).
			float lip = smoothstep(0.925, 0.965, u);
			col = mix(col, mix(col, pow(vec3(0.92, 1.0, 0.98), vec3(2.2)), 0.6), lip * calm);
		} else {
			// A soft caustic net near the stage, the light through the ripples (the reference's turquoise lattice), and fine ripples
			// further out: no crest rows.
			float near_k = 1.0 - smoothstep(8.0, 70.0, back);
			// (The cells' edges wander, warped by a slow noise, and are thin: straight, they read as cracked tiles.)
			vec2 warp = vec2(noise(q * 0.21 + TIME * 0.05), noise(q * 0.21 + 7.0 - TIME * 0.04)) * 1.6;
			float e1 = cell_edge(q * vec2(0.3, 0.3) + warp + vec2(TIME * 0.06, 0.0), TIME);
			float e2 = cell_edge(q * vec2(0.62, 0.62) + warp * 1.7 + vec2(-TIME * 0.05, 3.0), TIME * 1.3);
			float net = (1.0 - smoothstep(0.0, 0.035, e1)) * 0.6 + (1.0 - smoothstep(0.0, 0.03, e2)) * 0.35;
			col = mix(col, mix(col, pow(vec3(0.7, 0.98, 0.95), vec3(2.2)), 0.6), net * near_k * 0.5);
			float rip = noise(q * vec2(0.5, 1.6) + vec2(TIME * 0.3, 0.0));
			col *= 1.0 - 0.06 * smoothstep(0.55, 0.8, rip) * (1.0 - far_k);
			col = mix(col, mix(col, vec3(1.0), 0.3), smoothstep(0.8, 0.95, rip) * 0.35 * (1.0 - far_k) * (1.0 - near_k * 0.6));
		}
		// (Not on the Fast tier: the trace is the look's one real cost.)
		if (look >= 2 && !cheap) {
			// Reflections: march the reflected view ray across the screen until it passes behind something drawn, and take that
			// colour, darkened and tinted by the water; rippled a little; fading with distance and where the ray finds nothing.
			vec3 vdir = normalize(world_pos - CAMERA_POSITION_WORLD);
			float wob = (noise(q * vec2(0.6, 2.4) + vec2(TIME * 0.5, 0.0)) - 0.5) * 0.05;
			vec3 rdir = normalize(reflect(vdir, normalize(vec3(wob, 1.0, wob * 0.5))));
			vec3 hit_col = vec3(0.0);
			float hit = 0.0;
			float t = 0.6;
			for (int k = 0; k < 24; k++) {
				vec3 pw = world_pos + rdir * t;
				vec4 pv = VIEW_MATRIX * vec4(pw, 1.0);
				vec4 pc = PROJECTION_MATRIX * pv;
				vec2 uv = pc.xy / pc.w * 0.5 + 0.5;
				if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
					break;
				}
				vec3 sp = linear_depth_pos(uv, INV_PROJECTION_MATRIX);
				float behind = -pv.z - (-sp.z);
				if (behind > 0.05 && behind < 2.0 + t * 0.25) {
					hit_col = textureLod(screen_tex, uv, 0.0).rgb;
					hit = 1.0;
					break;
				}
				t *= 1.22;
			}
			// (Darker than what they show, as water's reflections are; that also turns the pier's deck top, which the screen holds where
			// the real reflection would show its underside, into a shade.)
			float rk = hit * (1.0 - smoothstep(30.0, 150.0, back)) * 0.55;
			col = mix(col, hit_col * vec3(0.42, 0.58, 0.64) + col * 0.22, rk);
		}
		// Foam and the wet band where things stand in the water, as before.
		float dd = texture(depth_tex, SCREEN_UV).r;
		vec4 vv = INV_PROJECTION_MATRIX * vec4(SCREEN_UV * 2.0 - 1.0, dd, 1.0);
		float gap = (-vv.z / vv.w) - (-VERTEX.z);
		float ruffle = noise(q * 1.3 + vec2(TIME * 0.3, 0.0)) * 0.6;
		float shore = 1.0 - smoothstep(0.0, foam_width, gap + ruffle);
		col = mix(col, col * vec3(0.7, 0.8, 0.88), smoothstep(0.75, 0.95, shore) * 0.5);
		float ring1 = smoothstep(0.35, 0.5, shore) * (1.0 - smoothstep(0.55, 0.72, shore));
		col = mix(col, foam.rgb, ring1 * 0.5);
		if (glint_path > 0.0) {
			vec3 to_p = normalize(world_pos - CAMERA_POSITION_WORLD);
			vec2 sd = normalize(sun_dir.xz);
			float across = abs(to_p.x * sd.y - to_p.z * sd.x);
			float path = (1.0 - smoothstep(0.02, 0.12 + 0.1 * far_k, across)) * smoothstep(20.0, 90.0, back);
			vec2 gp = q * vec2(0.5, 1.6) + vec2(TIME * 0.2, TIME * 0.05);
			float sparkle = smoothstep(0.62, 0.8, noise(gp) * 0.6 + noise(gp * 2.7 + 5.0) * 0.4);
			col = mix(col, vec3(1.0, 0.98, 0.9), path * sparkle * glint_path * 0.8);
		}
		col = mix(col, horizon.rgb, smoothstep(110.0, 220.0, back) * 0.4);
		col = mix(col, mix(haze.rgb, vec3(1.0), 0.3), smoothstep(4.0, 110.0, back) * haze_max);
		ALBEDO = col;
		ALPHA = 1.0;
	} else {
	// How far back (behind the stage plane at z = 0) this water is: bands of colour, lighter and paler toward the horizon.
	float back = max(-world_pos.z, 0.0);
	// (One smooth gradient, deepening toward the horizon, 2026-10-19: four hard steps read as bright bands across the sea, and a far
	// sea paling toward the horizon ran into the sky.)
	float band = smoothstep(0.0, 180.0, back);
	vec3 col = mix(deep.rgb, shallow.rgb, 0.72 - band * 0.5);
	// The shallows near the stage a lighter turquoise (the water over sand), in one soft step.
	col = mix(col, mix(shallow.rgb, vec3(0.75, 1.0, 0.95), 0.35), (1.0 - smoothstep(6.0, 22.0, back)) * 0.55);
	if (!cheap) {
		// Broad swaths under everything (art review 5: the sea read as one flat cyan with a pattern on it): long, slow patches a little
		// deeper and a little lighter, stretched along the shore and drifting one way, so the water has large forms before any detail.
		vec2 q = world_pos.xz;
		float swath = noise(q * vec2(0.018, 0.07) + vec2(TIME * 0.03, 0.0)) * 0.65 + noise(q * vec2(0.045, 0.16) + vec2(TIME * 0.05, 3.0)) * 0.35;
		float swath_k = (1.0 - band * 0.7);
		col = mix(col, deep.rgb * 0.92, smoothstep(0.45, 0.75, swath) * 0.5 * swath_k);
		col = mix(col, mix(shallow.rgb, vec3(1.0), 0.18), (1.0 - smoothstep(0.22, 0.42, swath)) * 0.32 * swath_k);
		// The swell as painted form (art review 7: the sea read as flat bands): long crests running along the shore, each lit on its face
		// toward the light and falling into a deeper trough behind, wavering and breaking up along their length; strongest near the stage,
		// gone toward the horizon where they would only shimmer.
		float swell_k = 1.0 - smoothstep(10.0, 110.0, back);
		float ph = world_pos.z * 0.62 + noise(q * vec2(0.035, 0.05)) * 5.0 + sin(world_pos.x * 0.06 + 1.3) * 1.2 + TIME * 0.7;
		float slope = cos(ph);
		float broken = smoothstep(0.3, 0.6, noise(q * vec2(0.09, 0.25) + vec2(TIME * 0.04, 0.0)));
		col = mix(col, mix(col, shallow.rgb * 1.15 + 0.04, 0.6), smoothstep(0.6, 0.78, slope) * swell_k * broken * 0.75);
		col = mix(col, deep.rgb * 0.8, smoothstep(0.55, 0.85, -slope) * swell_k * 0.35);
		// Two layers of long streaks drifting at different speeds, stretched along the shore (crisp painted strokes, not soft smudges).
		vec2 p = world_pos.xz;
		float s1 = noise(p * vec2(0.06, 0.45) + vec2(TIME * 0.05, 0.0));
		float s2 = noise(p * vec2(0.1, 0.8) + vec2(-TIME * 0.08, TIME * 0.02) + 17.0);
		// (Soft painted highlights rather than crisp strokes, the gauntlet's round 7 after the owner's harbour reference.)
		float streaks = smoothstep(0.76, 0.86, s1) * 0.32 + smoothstep(0.82, 0.92, s2) * 0.2;
		col = mix(col, streak.rgb, streaks * (1.0 - band * 0.6));
		// Wave crests, painted: rows across the sea (closer together further off), each a wavy line broken into dashes that drift
		// slowly, a pale crest with a darker trough just behind it, fading with distance.
		float rowz = -p.y / mix(4.6, 8.0, smoothstep(0.0, 120.0, back));
		float row = floor(rowz);
		float wob = 0.18 * sin(p.x * 0.11 + row * 1.7 + TIME * 0.35) + 0.08 * sin(p.x * 0.37 + row * 3.1);
		float fz = fract(rowz + wob);
		// (Fewer and of varied length: a slower noise along each row, cut higher; each crest thicker in its middle than at its ends.)
		float along = noise(vec2(p.x * mix(0.05, 0.11, hash(vec2(row, 1.3))) + TIME * 0.04 * (mod(row, 2.0) * 2.0 - 1.0), row * 3.7));
		float dash = smoothstep(0.6, 0.66, along);
		float thick = mix(0.035, 0.085, smoothstep(0.6, 0.85, along));
		float crest = (1.0 - smoothstep(thick * 0.5, thick + 0.09, abs(fz - 0.5))) * dash;
		float trough = (1.0 - smoothstep(0.06, 0.11, abs(fz - 0.6))) * dash * (1.0 - crest);
		float near_k = 1.0 - smoothstep(30.0, 150.0, back);
		// The lit face of each swell, a broad soft band on the far side of its crest, so the rows read as waves rolling in, not lines.
		float face = smoothstep(0.32, 0.46, fz) * (1.0 - smoothstep(0.5, 0.52, fz)) * smoothstep(0.35, 0.6, along);
		col = mix(col, mix(col, vec3(1.0), 0.28), face * 0.55 * near_k);
		col = mix(col, col * vec3(0.78, 0.84, 0.92), trough * 0.6 * near_k);
		col = mix(col, mix(streak.rgb, vec3(1.0), 0.3), crest * 0.4 * near_k);
		// Lighter, clearer turquoise near the stage (the reference's shallows round the pier).
		col = mix(col, mix(shallow.rgb, vec3(0.6, 0.95, 0.95), 0.35), (1.0 - smoothstep(2.0, 22.0, back)) * 0.35);
		// Glints: a few cells light up and fade, more of them nearer; each a small soft four-point sparkle in its cell (they were the
		// whole cell, hard little rectangles).
		vec2 cs = p * vec2(0.9, 2.2);
		vec2 cell = floor(cs);
		float g = hash(cell);
		float twinkle = smoothstep(0.75, 1.0, sin(TIME * 2.3 + g * 40.0));
		vec2 in_cell = (fract(cs) - 0.5) * vec2(1.0, 2.4);
		float star = max(1.0 - smoothstep(0.0, 0.28, abs(in_cell.x) + abs(in_cell.y) * 3.0), 1.0 - smoothstep(0.0, 0.28, abs(in_cell.y) + abs(in_cell.x) * 3.0));
		col = mix(col, vec3(1.0), step(0.965, g) * twinkle * star * (1.0 - band) * 0.9);
		// Foam where something stands in the water: the scene behind is only a little below the surface there.
		float d = texture(depth_tex, SCREEN_UV).r;
		vec4 v = INV_PROJECTION_MATRIX * vec4(SCREEN_UV * 2.0 - 1.0, d, 1.0);
		float scene = -v.z / v.w;
		float gap = scene - (-VERTEX.z);
		float ruffle = noise(p * 1.3 + vec2(TIME * 0.3, 0.0)) * 0.6;
		float shore = 1.0 - smoothstep(0.0, foam_width, gap + ruffle);
		// A solid band of foam right at the waterline, and a fainter broken ring further out where the wash spreads.
		// (A soft ripple ring rather than a solid band of white, round 7: a darker wet band right at the waterline, a pale soft ring out
		// from it, and a fainter one further out, as the reference's piles stand in the water.)
		col = mix(col, col * vec3(0.7, 0.8, 0.88), smoothstep(0.75, 0.95, shore) * 0.6);
		float ring1 = smoothstep(0.35, 0.5, shore) * (1.0 - smoothstep(0.55, 0.72, shore));
		col = mix(col, foam.rgb, ring1 * 0.55);
		float ring2 = smoothstep(0.12, 0.2, shore) * (1.0 - smoothstep(0.24, 0.32, shore)) * smoothstep(0.35, 0.6, noise(p * 2.1 - vec2(TIME * 0.2, 0.0)));
		col = mix(col, foam.rgb, ring2 * 0.3);
	}
	// The far sea brightening toward the horizon, and the line softened into the sky's own colour.
	col = mix(col, horizon.rgb, smoothstep(80.0, 200.0, back) * 0.45);
	if (!cheap && glint_path > 0.0) {
		// The sun's path: a column of broken sparkle along the line from the camera toward the sun, wider and denser further off.
		vec3 to_p = normalize(world_pos - CAMERA_POSITION_WORLD);
		vec2 sd = normalize(sun_dir.xz);
		float across = abs(to_p.x * sd.y - to_p.z * sd.x);
		float path = (1.0 - smoothstep(0.02, 0.12 + 0.1 * band, across)) * smoothstep(20.0, 90.0, back);
		vec2 gp = world_pos.xz * vec2(0.5, 1.6) + vec2(TIME * 0.2, TIME * 0.05);
		float sparkle = smoothstep(0.62, 0.8, noise(gp) * 0.6 + noise(gp * 2.7 + 5.0) * 0.4);
		col = mix(col, vec3(1.0, 0.98, 0.9), path * sparkle * glint_path);
		col = mix(col, mix(col, vec3(1.0), 0.25), path * 0.3 * glint_path);
	}
	// The stage's distance haze (the scenery gets it from the finishing pass; the water is drawn after it).
	col = mix(col, mix(haze.rgb, vec3(1.0), 0.3), smoothstep(4.0, 110.0, back) * haze_max);
	ALBEDO = col;
	ALPHA = 1.0;
	}
}
