// The stage's baked soft shading (scripts/stage_light.gd): how much of the sky each point near the stage plane can see, worked out once
// from the blocks and platforms when the stage is built, so the ground under a platform, the foot of a wall and a fighter standing in
// those places are softly shaded. `pf_stage_ao` gives the occlusion at a world position (1 = open sky); it only applies close to the stage
// plane, not to the scenery far behind.
global uniform sampler2D pf_ao_map;
global uniform vec4 pf_ao_rect;
global uniform float pf_ao_strength;
// How far the platforms reach front to back (world z, from, to): the map is flat (it knows across and up, not depth), so on the ground
// under a platform its shade ran from the back of the lawn to its front edge, a long hard-sided patch in perspective. Ground (a surface
// facing up, `pf_stage_ao_ground`) takes the shade only under the platforms' depth, fading over a unit outside it.
global uniform vec2 pf_ao_depth;

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

// `pf_stage_ao` for a surface whose world normal is `n`: where it faces up (the ground), only under the platforms' depth.
float pf_stage_ao_ground(vec3 p, vec3 n) {
	float ao = pf_stage_ao(p);
	float outside = max(pf_ao_depth.x - p.z, p.z - pf_ao_depth.y);
	float under = 1.0 - smoothstep(0.0, 1.2, outside);
	return mix(ao, mix(1.0, ao, under), smoothstep(0.5, 0.85, n.y));
}
