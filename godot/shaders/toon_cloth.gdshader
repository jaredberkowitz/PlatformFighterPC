shader_type spatial;
// Modelled clothes: the soft cel material (toon_common.gdshaderinc) drawn on both sides, so the cloth needs no thickness and its inside
// shows (a shade darker) through a hem or a sleeve rather than a hole.
render_mode cull_disabled;
#define PF_TWO_SIDED

#include "res://shaders/toon_common.gdshaderinc"
