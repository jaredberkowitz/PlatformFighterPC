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
// The chin line: a thin soft ink round the bottom of the ball of the head, where it sits on the body (the owner, 2026-10-14: the head
// melted into the chest like a gourd; the two-tone light draws no line at a soft crease, so it is painted). The head's ball in the rest
// pose (base_rig.json's head_centre and head_radius), and how strong the line is (0 none).
uniform vec3 head_ball = vec3(1.716, 0.003, 0.47);
uniform float chin_line = 1.0;

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
// False when the body wears modelled clothes (the rig's "Shirt" and "Shorts" meshes): then only the socks and shoes are painted here.
uniform bool painted_clothes = true;
// The look's gear (Loadout.GEAR): shorts, and socks and shoes. Without them the legs and feet are the bare body.
uniform bool shorts_on = true;
uniform bool shod = true;
// How far inside a modelled garment's hems the skin stops being drawn (rest units; along the upper arm, a share of it).
const float COVER = 0.03;
const float COVER_ARM = 0.15;

instance uniform vec4 flash : source_color, instance_index(0) = vec4(1.0, 1.0, 1.0, 0.0);
instance uniform float object_id : instance_index(1) = 0.95;

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
	// On the arm: close to its axis (shoulder to wrist, the wrist taken past the elbow), as make_rigged_blob.py's garments decide it. The
	// hips are about as wide as the arms' inner edge, so being far from the middle alone is not enough.
	vec2 sa = p.x < 0.0 ? shoulder_l : shoulder_r;
	vec2 se = p.x < 0.0 ? elbow_l : elbow_r;
	vec2 sw = se + (se - sa) * 1.05;
	float st = clamp(dot(p - sa, sw - sa) / dot(sw - sa, sw - sa), 0.0, 1.0);
	bool arm = abs(p.x) > 0.15 && p.y < neck_z && length(p - (sa + (sw - sa) * st)) < 0.14;
	bool has_shirt = painted_clothes && shirt.a > 0.5;
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
		if (painted_clothes && shorts_on && p.y > shorts_end_z && p.y <= waist_z) {
			c = shorts.rgb;
		}
		if (shod) {
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
			edge = min(edge, abs(p.y - sock_top_z));
			edge = min(edge, abs(p.y - shoe_top_z));
			edge = min(edge, abs(p.y - sole_top_z));
		}
		if (painted_clothes && shorts_on) {
			edge = min(edge, abs(p.y - waist_z));
			edge = min(edge, abs(p.y - shorts_end_z));
		}
		if (has_shirt) {
			edge = min(edge, abs(p.y - neck_z));
		}
	}
	c = mix(hem.rgb, c, smoothstep(hem_width * 0.5, hem_width, edge));
	// Under modelled clothes the skin is not drawn at all (a little inside the hems, so no gap opens at an edge): however the body bends,
	// it cannot poke through the cloth (an upper arm at the shirt's side when the fists come up).
	if (!painted_clothes) {
		bool covered = false;
		if (arm) {
			vec2 a = p.x < 0.0 ? shoulder_l : shoulder_r;
			vec2 b = p.x < 0.0 ? elbow_l : elbow_r;
			covered = shirt.a > 0.5 && along_arm(p, a, b) < sleeve - COVER_ARM;
		} else {
			// (With a shirt over the shorts, one stretch from the shorts' hems to the neck: no band of skin between them to show.)
			// (With no shorts, only the shirt covers: from its hem at the waist.)
			float bottom = shorts_on ? shorts_end_z + COVER : waist_z + COVER;
			float top = shirt.a > 0.5 ? neck_z - COVER * 1.5 : (shorts_on ? waist_z : -1.0);
			covered = p.y > bottom && p.y < top;
		}
		if (covered) {
			discard;
		}
	}
	if (chin_line > 0.0 && !arm) {
		vec3 rest = vec3(p.x, p.y, UV2.x);
		float dist = length(rest - vec3(0.0, head_ball.x, head_ball.y));
		// (On the ball's surface, in a thin ring just above where it meets the shoulders: below about 0.72 of the radius down.)
		float on_ball = 1.0 - smoothstep(0.015, 0.03, abs(dist - head_ball.z));
		float low = (head_ball.x - p.y) / head_ball.z;
		float ring = smoothstep(0.7, 0.74, low) * (1.0 - smoothstep(0.8, 0.84, low));
		c = mix(c, mix(hem.rgb, c, 0.35), on_ball * ring * chin_line);
	}
	ALBEDO = c;
	EMISSION = flash.rgb * flash.a;
	ROUGHNESS = object_id;
	AO = pf_stage_ao(world_pos);
	AO_LIGHT_AFFECT = 0.45;
	SPECULAR = 0.0;
}
