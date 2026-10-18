// Fighters seen through what stands in front of them (the owner, 2026-10-18: a low fighter, recovering under the stage, was hidden behind
// the meadow's hill and other things below the stage). Every scenery surface in front of the fighters' plane (world z 0) that would cover
// a fighter goes see-through over the fighter in a fine screen-door pattern, so the fighter shows and the scenery still reads round it.
// Whether it covers one is exact: the line from the camera through this point is carried on to the fighters' plane and tested against
// each fighter's outline there (an ellipse round its body). Set every frame by scripts/main.gd `_update_xray`: per fighter (x, y) its
// body's middle and (z, w) its half width and half height; w 0 for no fighter.
global uniform vec4 pf_xray_0;
global uniform vec4 pf_xray_1;
global uniform vec4 pf_xray_2;
global uniform vec4 pf_xray_3;

// How much of this point to leave out (0 none, 1 all) for world point `p` seen from `cam`.
float pf_xray_amount(vec3 p, vec3 cam) {
	if (p.z < 0.3 || cam.z <= p.z) {
		return 0.0;
	}
	vec2 q = mix(cam.xy, p.xy, cam.z / (cam.z - p.z));
	float k = 0.0;
	vec4 f[4] = vec4[](pf_xray_0, pf_xray_1, pf_xray_2, pf_xray_3);
	for (int i = 0; i < 4; i++) {
		if (f[i].w <= 0.0) {
			continue;
		}
		// (Nothing below the feet, the ellipse's bottom: the ground in front of a standing fighter is not in front of the fighter.)
		if (q.y < f[i].y - f[i].w) {
			continue;
		}
		float d = length((q - f[i].xy) / f[i].zw);
		k = max(k, 1.0 - smoothstep(0.8, 1.15, d));
	}
	return k;
}

// Leaves the point out where it covers a fighter, as a screen door (`frag` the pixel's FRAGCOORD.xy).
bool pf_xray_cut(vec3 p, vec3 cam, vec2 frag) {
	float k = pf_xray_amount(p, cam);
	if (k <= 0.0) {
		return false;
	}
	// Interleaved gradient noise: an even, fine pattern with no visible grid.
	float n = fract(52.9829189 * fract(dot(frag, vec2(0.06711056, 0.00583715))));
	return n < k * 0.82;
}
