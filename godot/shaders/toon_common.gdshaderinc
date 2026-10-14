// The soft cel material's body, shared by toon.gdshader (one-sided) and toon_cloth.gdshader (two-sided).

global uniform vec4 pf_paint_warm;
global uniform vec4 pf_paint_cool;
global uniform float pf_paint_amount;
uniform vec4 albedo : source_color = vec4(1.0);
uniform sampler2D albedo_tex : source_color, filter_linear_mipmap, repeat_enable, hint_default_white;
uniform bool use_tex = false;
uniform vec2 tex_scale = vec2(1.0);
// Above zero: the texture is laid in world space instead of by UV, `world_tile` world units per repeat, on the top for upward faces and
// on the front otherwise (so a stage surface tiles evenly whatever the block's size).
uniform float world_tile = 0.0;
// World-laid surfaces only: how much their colour drifts lighter and darker in broad, slow patches across the stage (0 = none), so a
// repeating pattern does not read as one tile stamped over and over.
uniform float world_vary = 0.08;
// World-laid surfaces only: a solid block's front falls gradually into a cool shade below its top edge (`front_top`, the block's top in
// world units, set per block by stage_art.gd), so the stage reads as one solid mass with a lit rim rather than a flat repeated pattern.
// `front_fall` is how dark it gets at most, reached `front_span` units down.
uniform float front_fall = 0.4;
uniform float front_span = 5.0;
uniform vec3 front_shade : source_color = vec3(0.6, 0.57, 0.72);
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
varying vec3 world_pos;
varying vec3 world_normal;

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

#ifdef PF_TWO_SIDED
// An ink line along the garment's bottom hem (the shirt's, over the shorts), `hem_ink` wide in rest units above `hem_low`; 0 for none.
uniform float hem_low = -99.0;
uniform float hem_ink = 0.0;
uniform vec4 hem_color : source_color = vec4(0.1, 0.07, 0.12, 1.0);
#endif

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
	world_normal = normalize((MODEL_MATRIX * vec4(NORMAL, 0.0)).xyz);
}

void fragment() {
	vec3 c = albedo.rgb;
	if (use_tex) {
		vec2 uv = UV * tex_scale;
		if (world_tile > 0.0) {
			uv = (abs(world_normal.y) > 0.5 ? world_pos.xz : vec2(world_pos.x, -world_pos.y)) / world_tile;
		}
		c *= texture(albedo_tex, uv).rgb;
		if (world_tile > 0.0) {
			vec2 p = abs(world_normal.y) > 0.5 ? world_pos.xz : world_pos.xy;
			float drift = sin(p.x * 0.37 + sin(p.y * 0.53) * 1.7) * sin(p.y * 0.29 + p.x * 0.11 + 1.3);
			c *= 1.0 + world_vary * drift;
			// And a slight drift in hue between warmer and cooler patches, slower still, so neighbouring areas differ a little.
			float hue_drift = sin(p.x * 0.17 + 2.1 + sin(p.y * 0.23) * 1.3) * sin(p.y * 0.13 + p.x * 0.07);
			c *= vec3(1.0) + vec3(0.035, 0.0, -0.035) * hue_drift * (world_vary / 0.08);
			if (detail_amount > 0.0) {
				float fine = texture(detail_tex, p / detail_scale).r - 0.5;
				float broad = texture(detail_tex, p / (detail_scale * 4.7) + vec2(0.37, 0.61)).r - 0.5;
				c *= 1.0 + detail_amount * (fine * 0.7 + broad * 1.3);
				c *= vec3(1.0) + vec3(0.04, 0.0, -0.04) * broad * detail_amount * 2.0;
			}
			if (abs(world_normal.y) < 0.5) {
				float below = smoothstep(0.6, front_span, front_top - world_pos.y);
				c *= mix(vec3(1.0), front_shade, front_fall * below);
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
	pf_sun = mix(1.0, pf_cloud_sun(world_pos, TIME), smoothstep(0.1, 0.6, world_normal.y) * (1.0 - fighter));
	ALBEDO = c;
	EMISSION = flash.rgb * flash.a;
	ROUGHNESS = object_id;
	AO = pf_stage_ao_ground(world_pos, world_normal);
	AO_LIGHT_AFFECT = 0.45;
	SPECULAR = 0.0;
}
