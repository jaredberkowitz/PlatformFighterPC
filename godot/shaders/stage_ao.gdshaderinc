// The stage's baked soft shading (scripts/stage_light.gd): how much of the sky each point near the stage plane can see, worked out once
// from the blocks and platforms when the stage is built, so the ground under a platform, the foot of a wall and a fighter standing in
// those places are softly shaded. `pf_stage_ao` gives the occlusion at a world position (1 = open sky); it only applies close to the stage
// plane, not to the scenery far behind.
global uniform sampler2D pf_ao_map;
global uniform vec4 pf_ao_rect;
global uniform float pf_ao_strength;

float pf_stage_ao(vec3 p) {
	if (pf_ao_strength <= 0.0 || p.z < -9.0 || p.z > 2.5) {
		return 1.0;
	}
	vec2 uv = (p.xy - pf_ao_rect.xy) / pf_ao_rect.zw;
	if (uv.x < 0.0 || uv.y < 0.0 || uv.x > 1.0 || uv.y > 1.0) {
		return 1.0;
	}
	return mix(1.0, texture(pf_ao_map, vec2(uv.x, 1.0 - uv.y)).r, pf_ao_strength);
}
