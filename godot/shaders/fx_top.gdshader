shader_type spatial;
// An effect drawn over everything (a hit's burst, sparks, a punch's smear).
render_mode unshaded, cull_disabled, shadows_disabled, fog_disabled, depth_draw_never, depth_test_disabled;

#include "res://shaders/fx.gdshaderinc"
