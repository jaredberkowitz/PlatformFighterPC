shader_type spatial;
// An effect that sits in the scene (hidden behind whatever is in front of it): a swing's crescent, dust.
render_mode unshaded, cull_disabled, shadows_disabled, fog_disabled, depth_draw_never;

#include "res://shaders/fx.gdshaderinc"
