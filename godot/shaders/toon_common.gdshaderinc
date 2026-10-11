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
varying vec3 world_pos;
varying vec3 world_normal;

// A flash over the whole fighter, per mesh (set by the game with `set_instance_shader_parameter`): white when hit, yellow while charging.
instance uniform vec4 flash : source_color = vec4(1.0, 1.0, 1.0, 0.0);
// Which object this is, for the ink lines (shaders/post.gdshader): written as the roughness, which the cel lighting does not use. Each
// fighter has its own (set by fighter_view.gd); the stage and scenery keep the default.
instance uniform float object_id = 0.95;

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
	ALBEDO = c;
	EMISSION = flash.rgb * flash.a;
	ROUGHNESS = object_id;
	AO = pf_stage_ao(world_pos);
	AO_LIGHT_AFFECT = 0.45;
	SPECULAR = 0.0;
}
