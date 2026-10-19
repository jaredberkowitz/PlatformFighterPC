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

void vertex() {
	vec3 w = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	if (!cheap) {
		VERTEX.y += sin(w.x * 0.07 + TIME * 0.6) * 0.18 + sin(w.z * 0.11 + w.x * 0.02 + TIME * 0.8) * 0.12;
	}
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
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
	// The stage's distance haze (the scenery gets it from the finishing pass; the water is drawn after it).
	col = mix(col, mix(haze.rgb, vec3(1.0), 0.3), smoothstep(4.0, 110.0, back) * haze_max);
	ALBEDO = col;
	ALPHA = 1.0;
}
