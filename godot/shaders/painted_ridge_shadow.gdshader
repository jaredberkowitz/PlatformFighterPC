shader_type spatial;
// The shadow a painted ridge card casts on the layer behind it (scripts/stage_art.gd `ridge`): the card's own silhouette, shifted up and
// away from the light, as a soft-edged translucent shade, so the backdrop reads as cut cards standing apart in a paper stage set (art
// review 9). Drawn on a card just behind the ridge it belongs to.
render_mode unshaded, cull_disabled, shadows_disabled, fog_disabled, depth_draw_never, blend_mix;

#include "res://shaders/painted_ridge.gdshaderinc"

uniform vec4 shadow_colour : source_color = vec4(0.1, 0.12, 0.3, 1.0);
uniform float strength = 0.3;
// The shift (a share of the card's width and height) and how soft the shadow's edge is (a share of its height).
uniform vec2 offset = vec2(0.01, 0.04);
uniform float softness = 0.025;
// Only on a card behind, not on the open sky (a shadow there read as smoke): beyond `reach` world units from the camera there is nothing.
uniform sampler2D depth_tex : hint_depth_texture, filter_nearest;
uniform float reach = 600.0;

void fragment() {
	float x = (UV.x - offset.x) * frequency;
	float y = 1.0 - UV.y - offset.y;
	float h = base + amplitude * ridge(x);
	float d = texture(depth_tex, SCREEN_UV).r;
	vec4 view = INV_PROJECTION_MATRIX * vec4(SCREEN_UV * 2.0 - 1.0, d, 1.0);
	float behind = step(-view.z / view.w, reach);
	ALBEDO = shadow_colour.rgb;
	ALPHA = (1.0 - smoothstep(h - softness, h + softness, y)) * strength * behind;
}
