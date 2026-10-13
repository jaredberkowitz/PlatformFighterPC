shader_type spatial;
// Leaf-cluster cards for tree crowns and tufts (scripts/stage_art.gd `_leaf_cards`): a hand-drawn clump of leaves (godot/art/stage/
// leaf_cluster.svg or grass_tuft.svg, greys tinted by each card's colour) cut out with alpha to coverage, lit by the shared cel light.
// Each card's normals bend away from the middle of the mass it belongs to (INSTANCE_CUSTOM.xyz: that middle, in the card's own space), so
// a crown shades as one rounded volume rather than as flat cards; light from behind shines through a little (`translucency`); and the
// cards flutter in the wind (`pf_wind`, a global, 0 still), each on its own phase (INSTANCE_CUSTOM.w). Scenery for the ink pass.
render_mode cull_disabled, alpha_to_coverage;

global uniform float pf_wind;
uniform sampler2D leaf_tex : source_color, filter_linear_mipmap, repeat_disable;
uniform vec4 albedo : source_color = vec4(1.0);
uniform float flutter = 0.06;     // how far a card's top sways (its own units)
uniform float translucency = 0.35;
// A tuft stands on its bottom edge and bends from there; a crown card bends from its middle.
uniform bool rooted = false;

#define PF_TRANSLUCENT
#include "res://shaders/toon_light.gdshaderinc"

void vertex() {
	vec3 centre = INSTANCE_CUSTOM.xyz;
	NORMAL = normalize(VERTEX - centre);
	float phase = INSTANCE_CUSTOM.w * 6.2832;
	float reach = rooted ? (VERTEX.y + 0.5) : 0.6 + 0.4 * VERTEX.y;
	float sway = sin(TIME * 1.6 + phase) * 0.7 + sin(TIME * 2.9 + phase * 1.7) * 0.3;
	VERTEX.x += sway * flutter * pf_wind * reach;
	VERTEX.z += cos(TIME * 1.3 + phase) * flutter * 0.4 * pf_wind * reach;
}

void fragment() {
	vec4 t = texture(leaf_tex, UV);
	ALBEDO = albedo.rgb * COLOR.rgb * t.rgb;
	// A tuft is dark down among its roots and bright at its tips (UV.y is 1 at its foot), deeper and a touch cooler low down, so a lawn
	// of them reads as thick grass with depth rather than a pale fringe.
	if (rooted) {
		float low = smoothstep(0.25, 1.0, UV.y);
		ALBEDO *= mix(vec3(1.08, 1.06, 0.98), vec3(0.46, 0.56, 0.6), low);
	}
	ALPHA = t.a;
	ALPHA_SCISSOR_THRESHOLD = 0.5;
	ALPHA_ANTIALIASING_EDGE = 0.25;
	ROUGHNESS = 0.95;
	SPECULAR = 0.0;
}
