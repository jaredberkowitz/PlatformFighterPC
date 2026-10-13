extends RefCounted
## Body deformation that follows the action (docs/MOTION_POLISH_PLAN.md phase 1), presentation only: the striking limb stretches on the
## strike, a strong hit squashes the victim through its freeze and the launch stretches it along its flight. Everything here is worked
## out from the snapshot (the move's frames, hitlag, velocity), so hitlag freezes it, a replay seek redraws it and it never touches a
## hitbox: the limbs still reach the live hitbox through the arm and leg IK (`FighterView._aim_arm`, `_aim_leg`), which aims the stretched
## limb at it.

## How much longer the striking limb gets on the strike (a share of its length), and how many frames it eases back after the hit.
const STRIKE_STRETCH := 0.22
const STRIKE_RELAX := 6.0
## The victim's squash along the hit for its freeze: per frame of hitlag, and at most.
const IMPACT_PER_HITLAG := 0.012
const IMPACT_MAX := 0.16
## The launch stretch along the flight: from this speed (world units a frame), growing by speed, at most.
const LAUNCH_FROM := 0.3
const LAUNCH_PER_SPEED := 1.0
const LAUNCH_MAX := 0.42


## How stretched (a share of its length) the striking limb is at move frame `f` of a move live from `first` to `last`: it snaps out over
## the frame before the hit, holds through the hit and two frames after (the follow-through), then eases home over `STRIKE_RELAX` frames.
static func strike_stretch(f: float, first: float, last: float) -> float:
	if first <= 0.0 or f < first - 1.0:
		return 0.0
	if f < first:
		return STRIKE_STRETCH * (f - first + 1.0)
	if f <= last + 2.0:
		return STRIKE_STRETCH
	var k := clampf((f - last - 2.0) / STRIKE_RELAX, 0.0, 1.0)
	return STRIKE_STRETCH * (1.0 - k * k * (3.0 - 2.0 * k))


## A bone's scale for a stretch `e` along its length (the bone's y axis), thinner across so the limb keeps its volume.
static func limb_scale(e: float) -> Vector3:
	var across := 1.0 / sqrt(1.0 + e)
	return Vector3(across, 1.0 + e, across)


## The body's deformation now, as a signed amount along a direction: positive stretches along `dir`, negative squashes along it. A fighter
## frozen by a hit that launches it (hitlag, `launch_pending`) is squashed along the hit, more for a stronger hit (across the stage: the
## snapshot does not carry the launch angle before the launch, and the sim has just turned the victim to face whoever hit it); flying fast
## in hitstun it is stretched along its flight. `flight` is its velocity plus its knockback.
static func body_target(state: String, hitlag: int, launch_pending: bool, flight: Vector2) -> Array:
	var speed := flight.length()
	if hitlag > 0 and launch_pending:
		return [-minf(IMPACT_MAX, IMPACT_PER_HITLAG * float(hitlag)), Vector2.RIGHT]
	if state == "Hitstun" and hitlag == 0 and speed > LAUNCH_FROM:
		return [minf(LAUNCH_MAX, (speed - LAUNCH_FROM) * LAUNCH_PER_SPEED), flight / speed]
	return [0.0, flight.normalized() if speed > 0.001 else Vector2.UP]


## A volume-keeping stretch by `k` along `dir` (in the stage's xy plane) about `middle`; negative `k` squashes.
static func along(k: float, dir: Vector2, middle: Vector3) -> Transform3D:
	if absf(k) < 0.001:
		return Transform3D.IDENTITY
	var a := Vector3(dir.x, dir.y, 0.0).normalized()
	var c := Vector3(-a.y, a.x, 0.0)
	var frame := Basis(a, c, Vector3(0, 0, 1))
	var s := 1.0 + k
	var b := frame * Basis.from_scale(Vector3(s, 1.0 / sqrt(s), 1.0 / sqrt(s))) * frame.inverse()
	return Transform3D(b, middle - b * middle)
