// Scenery on twos (the style pass, docs/art/stages/STYLE_PASS.md; StageArt.scene_steps): the scenery's own clock. With pictures a second
// set (`pf_scene_steps`), the time of the scenery's last picture (`pf_scene_t`, taken on the same frames as the scenery that moves in
// scripts), so the water, the leaves and the clouds' shadows hold still between pictures as a stop-motion set does; otherwise TIME. The
// fighters and the effects never read it.
#ifndef PF_SCENE_TIME
#define PF_SCENE_TIME

global uniform float pf_scene_steps;
global uniform float pf_scene_t;

float pf_scene_time(float t) {
	return pf_scene_steps > 0.0 ? pf_scene_t : t;
}

#define SCENE_TIME pf_scene_time(TIME)

#endif
