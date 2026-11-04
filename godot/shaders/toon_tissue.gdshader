shader_type spatial;
// Tissue paper (the style pass's step 6, the grove's mist; docs/art/stages/STYLE_PASS.md): the soft cel material (toon_common.gdshaderinc)
// as a see-through sheet with no thickness, so the sheets and the valley under it show through, more under one sheet than under three; its
// edge torn, wandering in and out, with the paper's fibres reaching past it. Each vertex's alpha says how far in from the sheet's edge it
// lies (StageBake._tissue_mesh); `tissue` sets how.
render_mode cull_back;
#define PF_TISSUE

#include "res://shaders/toon_common.gdshaderinc"
