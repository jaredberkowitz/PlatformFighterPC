// The soft cel material's body, shared by toon.gdshader (one-sided) and toon_cloth.gdshader (two-sided).

global uniform vec4 pf_paint_warm;
// The paper finish on the scenery (scripts/lighting.gd `paper_finish`; the stage refinement's paper pilot): x a fine fibre grain, y a
// slight wobble of colour from sheet to sheet, z a pale cut edge where a surface turns away from the eye. Never on fighters; off is 0.
global uniform vec4 pf_paper_finish;
global uniform vec4 pf_paint_cool;
global uniform float pf_paint_amount;
uniform vec4 albedo : source_color = vec4(1.0);
uniform sampler2D albedo_tex : source_color, filter_linear_mipmap, repeat_enable, hint_default_white;
uniform bool use_tex = false;
uniform vec2 tex_scale = vec2(1.0);
// Above zero: the texture is laid in world space instead of by UV, `world_tile` world units per repeat, on the top for upward faces and
// on the front otherwise (so a stage surface tiles evenly whatever the block's size).
uniform float world_tile = 0.0;
// A garment's print laid three ways by its rest position (x and height in UV, depth in UV2.x, art/blender/make_rigged_blob.py), front,
// side and top, blended by which way the cloth faces at rest: laid front-on only, a print stretched into streaks round the sides (the
// fleck pass, 2026-10-28). For prints that read the same any way round (the fleck); the stripes and flowers keep the front-on map.
uniform bool rest_tri = false;
// The same, laid by the part's own position instead (a rigid part with no rest maps: the hats' cloth, the neckwear; the headwear pass,
// 2026-10-29).
uniform bool part_tri = false;
varying vec3 part_pos;
// World-laid surfaces only: how much their colour drifts lighter and darker in broad, slow patches across the stage (0 = none), so a
// repeating pattern does not read as one tile stamped over and over.
uniform float world_vary = 0.08;
// Painted patches (0 off): large tonal patches with soft but clear edges, a deeper tone and a sunlit one over the base, as a hand-painted
// lawn is laid in (the owner's references and outside brief, 2026-10-17: grass as one flat sheet read cheap). How strong.
uniform float patch_amount = 0.0;
// World-laid tops only: the top deepens and cools from `back_from` (world z) back over `back_span` units by `back_deepen` (0 off), so a
// lawn reads as receding ground rather than one bright sheet (gauntlet round 3).
uniform float back_deepen = 0.0;
uniform float back_from = 1.0;
uniform float back_span = 7.0;
// A play top's soft fall into shade toward its front edge (0 off; the light pass, docs/art/stages/BACKGROUNDS.md): from `lawn_front`
// (world z, the front edge) back over `lawn_span` units, deeper and a little cooler by `lawn_fall`, so the ground has form.
uniform float lawn_fall = 0.0;
uniform float lawn_front = 2.0;
uniform float lawn_span = 4.0;
// World-laid tops only: painted grass-blade clumps (0 off), a few short tapered strokes leaning together in scattered clumps, lit in the
// sunlit patches and deeper in the shaded ones (the owner's material notes, 2026-10-17: grass wants selective highlights and
// directional variation, not one smooth sheet). How strong.
uniform float blades = 0.0;
// How big the blades are drawn (1 on the stage; the far hills take larger, so they still read at their distance).
uniform float blade_scale = 1.0;
// World-laid surfaces only: a solid block's front falls gradually into a cool shade below its top edge (`front_top`, the block's top in
// world units, set per block by stage_art.gd), so the stage reads as one solid mass with a lit rim rather than a flat repeated pattern.
// `front_fall` is how dark it gets at most, reached `front_span` units down.
uniform float front_fall = 0.4;
uniform float front_span = 5.0;
uniform vec3 front_shade : source_color = vec3(0.6, 0.57, 0.72);
// The stage fronts calmed (the gauntlet's round 1: the fronts were the loudest thing in every frame): their colour eased toward grey by
// `pf_front_calm.a` and, the further below the top, toward the stage's haze (`pf_front_calm.rgb`), so the play surface leads and the
// mass under it settles into the air. Set by scripts/lighting.gd for each stage.
global uniform vec4 pf_front_calm;
// A worn path along the play line on a stage top (0 off): where fighters stand and run, a broken band of drier, paler ground (grass worn
// to earth-tinted, boards scuffed lighter), soft-edged and patchy along its length (the gauntlet's round 1: the tops read as flat fills
// without the benchmark's wear). `worn_tint` is what the ground wears toward.
uniform float worn_path = 0.0;
uniform vec3 worn_tint : source_color = vec3(1.45, 1.2, 0.55);
// How much the worn band breaks into patches (1 as drawn; 0 a continuous path, the meadow's cream path after the owner's reference).
uniform float worn_breaks = 1.0;
// The worn band's width (times its drawn width) and how far back its middle sits (world units).
uniform float worn_width = 1.0;
uniform float worn_back = 0.0;
// Fall into the front shade by height (as `front_fall` does on a world-laid front) on any surface: the modelled pieces on a building
// front (scripts/stage_art.gd `_facade_windows`) settle back with the wall they stand on.
uniform bool fall_by_height = false;
// World-laid surfaces only: treat every face as a front (laid in x/y, falling into the front shade), however it tilts (the earth bulges,
// whose lumps tilt past where a face would otherwise count as a top).
uniform bool front_only = false;
// Flat-shaded facets (2026-10-18, art review 9: one shape language, crisp folds as on paper and in a cel-shaded adventure, not smooth clay):
// each triangle takes one light from its own face, whatever the mesh's normals.
uniform bool facets = false;
// World-laid fronts only: broad painted shadow shapes over an earth wall (0 off), large soft-edged cool shapes in `front_shade` and warmer
// lit ones between them, slow in x and leaning as they run down, so a dirt face reads as rounded packed earth under one light rather
// than one flat colour (the owner's material notes, 2026-10-17: dirt wants broad shadow shapes and warm/cool shifts).
uniform float front_shapes = 0.0;
// World-laid fronts only: moss hanging over the top of the wall (0 = none), in `moss_color`, a band a little under a unit deep whose lower
// edge drips in uneven tongues; it sits above the cool fall, so it stays lit like the lawn it grows from.
uniform float moss_amount = 0.0;
uniform vec3 moss_color : source_color = vec3(0.4, 0.66, 0.14);
// World-laid surfaces only: a painted detail tile (godot/art/stage/detail_*.svg, grey around the middle) varies the lightness, and a
// little the hue, at two scales: fine every `detail_scale` world units, broad at 4.7 times that. `detail_amount` 0 (everything by
// default, fighters always) leaves the surface as it was.
uniform sampler2D detail_tex : filter_linear_mipmap, repeat_enable, hint_default_white;
uniform float detail_amount = 0.0;
uniform float detail_scale = 3.0;
// A printed paper surface (scenery only, off by default; tools/art/make_prints.py, fidelity review 3's idea 2): a tiling print, grey
// around the middle, laid in world space on whichever plane the surface faces, every `print_scale` units, darkening and lightening the
// surface's own colour by `print_amount`.
uniform sampler2D print_tex : filter_linear_mipmap_anisotropic, repeat_enable, hint_default_white;
uniform float print_amount = 0.0;
uniform float print_scale = 2.0;
// This surface's own paper finish instead of the stage's (a stage front at the full finish, "C"); x below 0 takes the stage's.
uniform vec4 paper_own = vec4(-1.0);
// Multiply by the mesh's vertex colour (off by default: imported props may carry colours meant for nothing). The crowns' lobed clump balls
// bake each leaf clump's own light into it (scripts/stage_art.gd `_clump_ball`): a lit top and a dark crevice between clumps.
uniform bool vertex_shade = false;
// Baked occlusion (scenery only, off by default, so the fighters never take it; scripts/stage_bake.gd, the grove pilot): a texture baked
// in Blender over the mesh's second UV set, the crevices and contact shade the cel light cannot find, multiplied into AO.
uniform bool use_baked_ao = false;
uniform sampler2D baked_ao : hint_default_white, filter_linear_mipmap, repeat_disable;
uniform float baked_ao_strength = 1.0;
// How far the baked occlusion also dims the direct light (the engine's AO_LIGHT_AFFECT; 0.45 otherwise), and a curve on it (above 1
// deepens the crevices and keeps the open faces clean).
uniform float baked_ao_light = 0.45;
uniform float baked_ao_curve = 1.0;
// Multiply by each MultiMesh instance's custom data (rgb): one material, every instance its own shade (the sea wall's stones).
uniform bool instance_tint = false;
// Painted ground shadows under background props (scripts/stage_art.gd `_ground_shadow`): soft ellipses on the ground in world x/z, each
// (centre x, centre z, half width, half depth), darkening the surface toward a cool shade. 0 of them for everything else.
uniform vec4 ground_shadows[12];
uniform int ground_shadow_count = 0;
varying vec3 vshade;
varying vec3 world_pos;
varying vec3 world_normal;
// A faint warm light from below on fighters (bounce off the ground).
uniform vec4 fighter_bounce : source_color = vec4(1.0, 0.78, 0.55, 1.0);
uniform float fighter_bounce_amount = 0.14;

// A flash over the whole fighter, per mesh (set by the game with `set_instance_shader_parameter`): white when hit, yellow while charging.
instance uniform vec4 flash : source_color, instance_index(0) = vec4(1.0, 1.0, 1.0, 0.0);
// Which object this is, for the ink lines (shaders/post.gdshader): written as the roughness, which the cel lighting does not use. Each
// fighter has its own (set by fighter_view.gd); the stage and scenery keep the default.
instance uniform float object_id : instance_index(1) = 0.95;
// The top of the block this front belongs to (see `front_fall`); far below anything for everything else, which leaves it untouched.
instance uniform float front_top : instance_index(6) = -100000.0;

#define PF_CLOUDS
#include "res://shaders/toon_light.gdshaderinc"
#include "res://shaders/brush.gdshaderinc"
#include "res://shaders/stage_ao.gdshaderinc"
#include "res://shaders/xray.gdshaderinc"

#ifdef PF_TWO_SIDED
// An ink line along the garment's bottom hem (the shirt's, over the shorts), `hem_ink` wide in rest units above `hem_low`; 0 for none.
uniform float hem_low = -99.0;
// Soft occlusion at a garment's top edge (the neckline under the head; the character judge's round 8), its rest height, -90 or less none.
uniform float neck_shade_z = -99.0;
uniform float hem_ink = 0.0;
uniform vec4 hem_color : source_color = vec4(0.1, 0.07, 0.12, 1.0);
#endif

void vertex() {
	part_pos = VERTEX;
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
	vshade = vertex_shade ? COLOR.rgb : vec3(1.0);
	if (instance_tint) {
		vshade *= INSTANCE_CUSTOM.rgb;
	}
}

// A stage front's colour calmed (`pf_front_calm`), `depth` units below its top.
vec3 pf_calm_front(vec3 col, float depth) {
	if (pf_front_calm.a <= 0.0 || depth < 0.0 || depth > 1000.0) {
		return col;
	}
	// (Round 2's judge: tinting toward the haze washed the canyon's warm strata to mauve and the grove's roots to lavender. Now a light
	// easing of saturation and contrast, and a darkening with depth that keeps each material's own hue, a touch cooler.)
	float y = dot(col, vec3(0.3, 0.59, 0.11));
	col = mix(col, vec3(y), 0.15 * pf_front_calm.a);
	col = mix(col, vec3(y * 0.6 + 0.16), 0.18 * pf_front_calm.a);
	float down = smoothstep(0.8, 9.0, depth) * pf_front_calm.a;
	return col * mix(vec3(1.0), vec3(0.62, 0.64, 0.74), down);
}


void fragment() {
	// (For light(), which reads only what the fragment hands it.)
	pf_world_z = world_pos.z;
	// (Scenery in front of a fighter goes see-through over it, shaders/xray.gdshaderinc.)
	if (fighter < 0.5 && pf_xray_cut(world_pos, CAMERA_POSITION_WORLD, FRAGCOORD.xy)) {
		discard;
	}
	vec3 c = albedo.rgb * vshade;
	// (How lit the grass blades are, -1 until the painted patches say; see `blades`.)
	float blade_light = -1.0;
	if (lawn_fall > 0.0 && world_normal.y > 0.5) {
		float edge = smoothstep(lawn_front - lawn_span, lawn_front, world_pos.z);
		c *= mix(vec3(1.0), vec3(0.62, 0.68, 0.84), edge * edge * lawn_fall);
	}
	if (use_tex) {
		vec2 uv = UV * tex_scale;
		if (world_tile > 0.0) {
			uv = (abs(world_normal.y) > 0.5 && !front_only ? world_pos.xz : vec2(world_pos.x, -world_pos.y)) / world_tile;
		}
		if (rest_tri || part_tri) {
			vec3 rest = (part_tri ? part_pos : vec3(UV.x, UV.y, UV2.x)) * tex_scale.x;
			// (The rest-pose facing from the rest position's slope across the pixel: the posed normal turns with every bone.)
			vec3 w = pow(abs(normalize(cross(dFdx(rest), dFdy(rest)))), vec3(4.0));
			w /= w.x + w.y + w.z + 1e-5;
			c *= texture(albedo_tex, rest.xy).rgb * w.z + texture(albedo_tex, rest.zy).rgb * w.x + texture(albedo_tex, rest.xz).rgb * w.y;
		} else {
			c *= texture(albedo_tex, uv).rgb;
		}
		if (world_tile > 0.0) {
			vec2 p = abs(world_normal.y) > 0.5 && !front_only ? world_pos.xz : world_pos.xy;
			float drift = sin(p.x * 0.37 + sin(p.y * 0.53) * 1.7) * sin(p.y * 0.29 + p.x * 0.11 + 1.3);
			c *= 1.0 + world_vary * drift;
			if (patch_amount > 0.0) {
				float big = sin(p.x * 0.21 + sin(p.y * 0.41) * 2.2) + sin(p.y * 0.33 - p.x * 0.09 + 0.7) * 0.8;
				float deep = smoothstep(0.55, 0.75, big);
				float sunlit = smoothstep(0.55, 0.75, -big - 0.2);
				c *= 1.0 - patch_amount * 0.55 * deep;
				c = mix(c, c * vec3(1.08, 1.1, 0.92) + 0.03, patch_amount * sunlit);
				blade_light = clamp(sunlit + 0.5 - deep, 0.0, 1.0);
			}
			if (back_deepen > 0.0 && world_normal.y > 0.5) {
				float back = smoothstep(back_from, back_from - back_span, world_pos.z);
				// (Strong in linear light: 0.84 showed as barely a tenth darker.)
				c = mix(c, c * vec3(0.5, 0.6, 0.85), back * back_deepen);
			}
			// And a slight drift in hue between warmer and cooler patches, slower still, so neighbouring areas differ a little.
			float hue_drift = sin(p.x * 0.17 + 2.1 + sin(p.y * 0.23) * 1.3) * sin(p.y * 0.13 + p.x * 0.07);
			c *= vec3(1.0) + vec3(0.035, 0.0, -0.035) * hue_drift * (world_vary / 0.08);
			if (detail_amount > 0.0) {
				float fine = texture(detail_tex, p / detail_scale).r - 0.5;
				float broad = texture(detail_tex, p / (detail_scale * 4.7) + vec2(0.37, 0.61)).r - 0.5;
				c *= 1.0 + detail_amount * (fine * 0.7 + broad * 1.3);
				c *= vec3(1.0) + vec3(0.04, 0.0, -0.04) * broad * detail_amount * 2.0;
			}
			if (abs(world_normal.y) < 0.5 || front_only) {
				float below = smoothstep(0.6, front_span, front_top - world_pos.y);
				if (front_shapes > 0.0) {
					vec2 q = world_pos.xy;
					// (Long shapes running down the face, a little wavering, as weathering falls down a bank: free blobs read as stains.)
					float sh = sin(q.x * 0.55 + sin(q.y * 0.35 + q.x * 0.2) * 0.9) * 0.6 + sin(q.x * 0.21 + 1.9 + q.y * 0.05) * 0.55
						+ sin(q.x * 1.3 + q.y * 0.15 + 0.6) * 0.2;
					// (Kept clear of the lit rim at the top, where the wall turns to the light.)
					float reach = smoothstep(0.3, 1.4, front_top - q.y);
					float cool = smoothstep(0.38, 0.46, sh) * reach;
					float warm = smoothstep(0.38, 0.46, -sh - 0.1);
					c = mix(c, c * vec3(0.34, 0.36, 0.62), front_shapes * cool);
					c = mix(c, c * vec3(1.3, 1.16, 0.82), front_shapes * warm);
				}
				c *= mix(vec3(1.0), front_shade, front_fall * below);
				c = pf_calm_front(c, front_top - world_pos.y);
				if (moss_amount > 0.0) {
					float x = world_pos.x;
					float drip = 1.15 + 0.45 * sin(x * 1.1 + sin(x * 0.4) * 2.0) * sin(x * 0.37 + 1.1) + 0.55 * pow(max(0.0, sin(x * 2.3 + 0.7)), 3.0);
					float depth = front_top - world_pos.y;
					// A clumpy edge (small bumps on the big tongues), the moss mottled lighter and darker in soft patches, lit at its top and
					// shaded toward its hanging edge, a thin darker rim where it curls under, and a soft shadow it casts on the stone below.
					float bumps = 0.07 * sin(x * 9.0 + 1.3) + 0.05 * sin(x * 15.0 + 0.4);
					float edge = drip + bumps;
					float moss = (1.0 - smoothstep(edge - 0.035, edge + 0.035, depth)) * step(-0.5, depth);
					float mottle = sin(x * 4.3 + world_pos.y * 3.1) * sin(x * 2.1 - world_pos.y * 5.3 + 0.8);
					vec3 mc = moss_color * (1.08 - 0.3 * smoothstep(0.0, edge, depth)) * (1.0 + 0.08 * mottle);
					mc = mix(mc, moss_color * 0.55, smoothstep(edge - 0.16, edge - 0.02, depth));
					float cast = (1.0 - smoothstep(edge, edge + 0.3, depth)) * (1.0 - moss);
					c *= 1.0 - 0.25 * cast * moss_amount;
					c = mix(c, mc, moss * moss_amount);
				}
			}
		}
	}
	if (world_tile <= 0.0) {
		c *= brush(UV);
	}
	if (fall_by_height) {
		c *= mix(vec3(1.0), front_shade, front_fall * smoothstep(0.6, front_span, front_top - world_pos.y));
		c = pf_calm_front(c, front_top - world_pos.y);
	}
	if (worn_path > 0.0 && world_normal.y > 0.6) {
		float x = world_pos.x;
		// (Its middle wanders a little either side of the play line, its width breathes, and it breaks into patches.)
		float mid = -0.4 + 0.35 * sin(x * 0.23 + 1.1) + 0.2 * sin(x * 0.61);
		mid -= worn_back;
		float half_w = (0.9 + 0.35 * sin(x * 0.41 + 2.0)) * worn_width;
		float band = 1.0 - smoothstep(half_w * 0.5, half_w, abs(world_pos.z - mid));
		float breaks = smoothstep(-0.35, 0.25, sin(x * 0.53 + sin(x * 0.17) * 2.0) + 0.5 * sin(x * 1.7 + world_pos.z * 2.3));
		float w = band * mix(1.0, breaks, worn_breaks) * worn_path;
		c = mix(c, c * worn_tint, w);
		blade_light = blade_light < 0.0 ? blade_light : blade_light * (1.0 - 0.6 * w);
	}
	if (ground_shadow_count > 0) {
		float gs = 0.0;
		for (int i = 0; i < 12; i++) {
			if (i >= ground_shadow_count) {
				break;
			}
			vec4 e = ground_shadows[i];
			float r = length((world_pos.xz - e.xy) / e.zw);
			gs = max(gs, 1.0 - smoothstep(0.25, 1.0, r));
		}
		// (Strong: the cel light gives back only part of a change in surface colour.)
		c = mix(c, c * vec3(0.26, 0.32, 0.56), gs);
	}
	// Painted grass blades (`blades`): on what faces up they are laid across the ground and point back (up the screen, the top being seen at
	// a low angle); on a hillside facing the camera they stand up its face. Cells across the surface, about two in five holding a clump of
	// three tapered blades leaning together; lit where the patches are sunlit (or where the land turns up to the sky), deeper elsewhere.
	if (blades > 0.0 && world_normal.y > -0.2) {
		bool on_top = world_normal.y > 0.75;
		vec2 q = (on_top ? world_pos.xz : vec2(world_pos.x, -world_pos.y)) / blade_scale;
		vec2 cs = on_top ? vec2(0.42, 0.9) : vec2(0.42, 0.5);
		float blen = on_top ? 0.75 : 0.42;
		float hit = 0.0;
		float tint = 0.5;
		for (int ox = -1; ox <= 1; ox++) {
			for (int oy = 0; oy <= 1; oy++) {
				vec2 cell = floor(q / cs) + vec2(float(ox), float(oy));
				float h1 = fract(sin(dot(cell, vec2(127.1, 311.7))) * 43758.5453);
				float h2 = fract(sin(dot(cell, vec2(269.5, 183.3))) * 43758.5453);
				if (h1 < 0.4) {
					vec2 root = (cell + vec2(h2, fract(h1 * 7.13))) * cs;
					float lean = (h2 - 0.5) * 0.5;
					for (int b = 0; b < 3; b++) {
						float fb = float(b) - 1.0;
						float len = blen * (1.0 - 0.25 * abs(fb)) * (0.8 + 0.4 * h2);
						vec2 d = q - root - vec2(fb * 0.07, 0.0);
						float up = -d.y / len;
						if (up > 0.0 && up < 1.0) {
							float w = 0.045 * (1.0 - up);
							float hb = 1.0 - smoothstep(w * 0.6, w, abs(d.x - (lean + fb * 0.12) * up * len));
							if (hb > hit) {
								hit = hb;
								tint = h2;
							}
						}
					}
				}
			}
		}
		float lit_side = blade_light >= 0.0 ? blade_light : clamp(smoothstep(0.35, 0.8, world_normal.y) + (tint - 0.5) * 0.7, 0.0, 1.0);
		vec3 blade_c = mix(c * vec3(0.62, 0.72, 0.8), c * vec3(1.25, 1.2, 0.8) + 0.04, lit_side);
		c = mix(c, blade_c, hit * blades);
	}
	// The painted gradient (the stage's profile, scripts/lighting.gd): what faces up leans warm, what faces down leans cool. Fighters take
	// a third of it, so their own colours stay true.
	float up = smoothstep(0.2, 0.8, world_normal.y * 0.5 + 0.5);
	vec3 paint = mix(pf_paint_cool.rgb, pf_paint_warm.rgb, up);
	c *= mix(vec3(1.0), paint, pf_paint_amount * mix(1.0, 0.33, fighter));
#ifdef PF_TWO_SIDED
	if (hem_ink > 0.0 && FRONT_FACING) {
		float above = (1.0 - UV.y) - hem_low;
		c = mix(hem_color.rgb, c, smoothstep(hem_ink * 0.6, hem_ink, above));
	}
	// The inside of a garment (seen through a hem): lit from its own side, a shade darker.
	if (!FRONT_FACING) {
		NORMAL = -NORMAL;
		c *= 0.8;
	}
#endif
	// (On what faces up: on a wall the drifting field showed as long slanted stripes. Not on fighters: a passing shade over one read as
	// it dimming, and every fighter keeps the same value on any spot of the stage.)
	// (Only near the stage, the gauntlet's round 15: on the far ranges the dapples read as low-resolution mottling, not light.)
	pf_sun = mix(1.0, pf_cloud_sun(world_pos, TIME), smoothstep(0.1, 0.6, world_normal.y) * (1.0 - fighter)
		* (1.0 - smoothstep(25.0, 55.0, -world_pos.z)));
	if (pf_cloud_tone.a > 0.0) {
		// (On the colour, shading ambient and sun alike: see `pf_cloud_tone`. Clouds only, not the platforms' cast shadow below.)
		c *= mix(vec3(1.0), pf_cloud_tone.rgb, clamp(1.0 - pf_sun, 0.0, 1.0) * pf_cloud_tone.a);
	}
	// (And the platforms' cast shadow on the ground under them, `pf_stage_sun`; not on fighters either.)
	pf_sun *= mix(1.0, pf_stage_sun(world_pos, world_normal), 1.0 - fighter);
	if (fighter < 0.5 && print_amount > 0.0) {
		// (On what faces up, stretched five times along the depth: the match camera sees a top almost edge on, squeezed about that much,
		// and laid evenly every mark blurred into the grey of the texture's smallest copies.)
		vec2 pp = abs(world_normal.y) > 0.5 ? vec2(world_pos.x, world_pos.z * 0.2) : (abs(world_normal.x) > 0.6 ? vec2(world_pos.z, -world_pos.y) : vec2(world_pos.x, -world_pos.y));
		c *= 1.0 + print_amount * (texture(print_tex, pp / print_scale).r - 0.5) * 2.0;
	}
	vec4 paper_fin = paper_own.x >= 0.0 ? paper_own : pf_paper_finish;
	if (fighter < 0.5 && paper_fin.x + paper_fin.y + paper_fin.z > 0.0) {
		// Fibre: fine grain stretched along the sheet, on whichever plane the surface lies.
		vec2 fp = abs(world_normal.y) > 0.5 ? world_pos.xz : (abs(world_normal.x) > 0.6 ? world_pos.zy : world_pos.xy);
		float fib = brush_noise(fp * vec2(7.0, 26.0)) * 0.6 + brush_noise(fp * vec2(19.0, 57.0)) * 0.4;
		c *= 1.0 + paper_fin.x * 0.22 * (fib - 0.5);
		// Sheet to sheet: each piece a touch lighter or darker and warmer or cooler, broad across the merged meshes.
		float sheet = fract(sin(dot(NODE_POSITION_WORLD, vec3(12.9898, 78.233, 37.719))) * 43758.5453);
		float broad = brush_noise(world_pos.xy * 0.11 + world_pos.zz * 0.07);
		float w = (sheet - 0.5) * 0.6 + (broad - 0.5);
		c *= 1.0 + paper_fin.y * 0.14 * w;
		c *= mix(vec3(1.0), vec3(1.02, 1.0, 0.97), paper_fin.y * clamp(w + 0.5, 0.0, 1.0));
		// The cut edge: where the surface turns away from the eye, a pale card edge.
		float rim = pow(1.0 - clamp(dot(NORMAL, VIEW), 0.0, 1.0), 5.0);
		c = mix(c, c * 0.4 + vec3(0.56, 0.54, 0.5), rim * paper_fin.z * 0.55);
	}
	// (Not on flat tops, whose own normal is already the face's: rebuilt from the screen derivatives it came out turned over on them, so
	// the background's grass, sand and terrace tops were lit as undersides, in their shade colour, and took no cast shadow; the graphics
	// review 5, 2026-10-22.)
	if (facets && world_normal.y < 0.97) {
		NORMAL = normalize(cross(dFdx(VERTEX), dFdy(VERTEX)));
	}
	ALBEDO = c;
	EMISSION = flash.rgb * flash.a + c * fighter_bounce.rgb * fighter_bounce_amount * fighter * clamp(-world_normal.y, 0.0, 1.0);
	ROUGHNESS = object_id;
	AO = pf_stage_ao_ground(world_pos, world_normal) * pf_fighter_sky(NORMAL, VIEW_MATRIX, fighter);
	if (use_baked_ao) {
		AO *= mix(1.0, pow(texture(baked_ao, UV2).r, baked_ao_curve), baked_ao_strength);
		AO_LIGHT_AFFECT = baked_ao_light;
	}
	AO_LIGHT_AFFECT = 0.45;
	SPECULAR = 0.0;
}
