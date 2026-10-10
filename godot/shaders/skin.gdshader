shader_type spatial;
// The base body's skin with its clothes painted on by region, so they bend with the body and never clip: a shirt (plain or patterned) from
// the waist to the neck with sleeves down the upper arms, shorts from the waist to above the knees, socks up from the ankles, and shoes
// with a pale sole below them, an ink hem at every edge. Each point's place on the body in the rest pose comes from the mesh's texture
// maps (art/blender/make_rigged_blob.py writes them: x and height in UV, depth in UV2; Blender's axes, the glTF export flips the height),
// and the regions' edges from base_rig.json (fighter_view.gd sets them). Lit like every cel surface.
render_mode cull_back;

uniform vec4 skin : source_color = vec4(1.0);
// a = 0: no shirt.
uniform vec4 shirt : source_color = vec4(1.0);
uniform sampler2D shirt_tex : source_color, filter_linear_mipmap, repeat_enable, hint_default_white;
uniform bool use_shirt_tex = false;
uniform vec2 shirt_tex_scale = vec2(1.0);
uniform vec4 shorts : source_color = vec4(0.25, 0.3, 0.6, 1.0);
uniform vec4 sock : source_color = vec4(1.0);
uniform vec4 shoe : source_color = vec4(0.27, 0.2, 0.3, 1.0);
uniform vec4 strap : source_color = vec4(0.4, 0.6, 1.0, 1.0);
uniform vec4 sole : source_color = vec4(1.0);
uniform vec4 hem : source_color = vec4(0.1, 0.07, 0.12, 1.0);
uniform float hem_width = 0.011;

// Where the clothes go, in the rest pose.
uniform float neck_z = 1.44;
uniform float waist_z = 0.89;
uniform float shorts_end_z = 0.62;
uniform float sock_top_z = 0.35;
uniform float shoe_top_z = 0.19;
uniform float sole_top_z = 0.035;
uniform float torso_half_width = 0.29;
uniform vec2 shoulder_l = vec2(-0.41, 1.19);
uniform vec2 elbow_l = vec2(-0.47, 0.97);
uniform vec2 shoulder_r = vec2(0.41, 1.19);
uniform vec2 elbow_r = vec2(0.47, 0.97);
// How far down the upper arm the sleeves reach (0 at the shoulder, 1 at the elbow).
uniform float sleeve = 0.55;

instance uniform vec4 flash : source_color = vec4(1.0, 1.0, 1.0, 0.0);
instance uniform float object_id = 0.95;

#include "res://shaders/toon_light.gdshaderinc"
#include "res://shaders/stage_ao.gdshaderinc"

varying vec3 world_pos;

void vertex() {
	world_pos = (MODEL_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

// How far (0 at the shoulder, 1 at the elbow) a point is down the upper arm.
float along_arm(vec2 p, vec2 a, vec2 b) {
	vec2 ab = b - a;
	return dot(p - a, ab) / dot(ab, ab);
}

void fragment() {
	vec2 p = vec2(UV.x, 1.0 - UV.y);   // rest x and height
	vec3 c = skin.rgb;
	// (The head is never clothed: an arm is below the neck.)
	bool arm = abs(p.x) > torso_half_width && p.y > shorts_end_z && p.y < neck_z;
	bool has_shirt = shirt.a > 0.5;
	vec3 shirt_c = shirt.rgb;
	if (use_shirt_tex) {
		shirt_c *= texture(shirt_tex, vec2(p.x, -p.y) * shirt_tex_scale).rgb;
	}
	// The distance to the nearest hem that applies here (rest units).
	float edge = 1.0;
	if (arm) {
		vec2 a = p.x < 0.0 ? shoulder_l : shoulder_r;
		vec2 b = p.x < 0.0 ? elbow_l : elbow_r;
		float t = along_arm(p, a, b);
		if (has_shirt && t < sleeve) {
			c = shirt_c;
		}
		if (has_shirt) {
			edge = abs(t - sleeve) * length(b - a);
		}
	} else {
		if (has_shirt && p.y > waist_z && p.y < neck_z) {
			c = shirt_c;
		}
		if (p.y > shorts_end_z && p.y <= waist_z) {
			c = shorts.rgb;
		}
		if (p.y <= sock_top_z && p.y > shoe_top_z) {
			c = sock.rgb;
		}
		if (p.y <= shoe_top_z) {
			c = shoe.rgb;
			// A strap over the front of the shoe, in the outfit colour.
			if (p.y > shoe_top_z - 0.075 && p.y < shoe_top_z - 0.03 && UV2.x < -0.06) {
				c = strap.rgb;
			}
		}
		if (p.y <= sole_top_z) {
			c = sole.rgb;
		}
		edge = min(edge, abs(p.y - waist_z));
		edge = min(edge, abs(p.y - shorts_end_z));
		edge = min(edge, abs(p.y - sock_top_z));
		edge = min(edge, abs(p.y - shoe_top_z));
		edge = min(edge, abs(p.y - sole_top_z));
		if (has_shirt) {
			edge = min(edge, abs(p.y - neck_z));
		}
	}
	c = mix(hem.rgb, c, smoothstep(hem_width * 0.5, hem_width, edge));
	ALBEDO = c;
	EMISSION = flash.rgb * flash.a;
	ROUGHNESS = object_id;
	AO = pf_stage_ao(world_pos);
	AO_LIGHT_AFFECT = 0.45;
	SPECULAR = 0.0;
}
