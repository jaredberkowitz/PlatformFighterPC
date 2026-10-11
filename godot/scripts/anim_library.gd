extends RefCounted
## The fighters' animation library: which clip every state and every move plays. Presentation only; the simulation never reads it.
##
## One rig and one set of clips serve every fighter (godot/models/base_rig.glb, authored by art/blender/make_rigged_blob.py, copied once
## per fighter by fighter_view.gd). A moveset (the fighter's class, Roster.CLASS_NAMES) does not bring its own animations: its moves map
## to the shared attack archetypes below, and a class overrides only the moves whose pose or path genuinely differs (a sword's rising cut,
## a claws fighter's kicks). Locomotion, jumps, landings, defence and hit reactions never change with the moveset.
##
## Attack clips share one timeline (wind-up by frame 21, strike at 33, follow-through to 45, settled by 60) and are played by progress
## through the move's own frames (fighter_view.gd `_attack_clip`), so a clip shared by two moves lands on each move's own active frames:
## the simulation's frame data stays the only clock. Docs: docs/ANIMATION.md.

## Shared clips by simulation state. States not listed here are chosen with more context in fighter_view.gd `_choose_clip` (walking and
## running speed, ledge options, the air, attacks).
const STATE_CLIPS := {
	"Crouch": "crouch",
	"JumpSquat": "jumpsquat",
	"Landing": "land",
	"WaveLand": "land",
	"Roll": "roll",
	"SpotDodge": "roll",
	"AirDodge": "roll",
	"Shield": "shield",
	"ShieldDrop": "shield",
	"WallTech": "shield",
	"Hitstun": "hurt",
	"ShieldBreak": "hurt",
	"Grabbed": "hurt",
	"Rebound": "hurt",
	"Knockdown": "knockdown",
	"GetUp": "knockdown",
	"LedgeHang": "ledge",
	"Grabbing": "grab",
	"Idle": "idle",
	"Respawn": "idle",
	"ShieldRelease": "idle",
}

## The shared attack archetypes, by move name (sim-core `MoveId::name`). Every moveset uses these unless its class overrides the move.
const SHARED_MOVES := {
	# A quick straight strike.
	"jab": "attack_jab",
	"jab 2": "attack_jab",
	"jab 3": "attack_jab",
	# A strike swung across the body.
	"ftilt": "attack_swing",
	# Upward strikes.
	"utilt": "attack_uair",
	"up special": "attack_uair",
	# Low strikes.
	"dtilt": "attack_low",
	"down special": "attack_low",
	# Lunges.
	"dash attack": "attack_lunge",
	"side special": "attack_lunge",
	# Heavy committed strikes.
	"fsmash": "attack_smash",
	"usmash": "attack_smash",
	"dsmash": "attack_smash",
	"neutral special": "attack_smash",
	# Aerials.
	"nair": "attack_nair",
	"fair": "attack_fair",
	"bair": "attack_bair",
	"uair": "attack_uair",
	"dair": "attack_dair",
	# Grabs and throws.
	"grab": "grab",
	"dash grab": "grab",
	"pivot grab": "grab",
	"pummel": "grab",
	"forward throw": "throw",
	"back throw": "throw",
	"up throw": "throw",
	"down throw": "throw",
	# Get-up attack, and the follow-up slots only scripts reach (a combo's later hits, a counter's answer).
	"get-up attack": "attack_swing",
}

## Per-class overrides: only the moves whose movement differs from the shared archetype.
const CLASS_MOVES := {
	# Longsword: the blade's own cuts.
	0: {
		"ftilt": "sword_ftilt",
		"utilt": "sword_utilt",
		"dtilt": "sword_dtilt",
		"fsmash": "sword_fsmash",
		"usmash": "sword_usmash",
		"dsmash": "sword_dsmash",
		"dair": "sword_dair",
	},
	# Claws: kicks and the blaster.
	1: {
		"ftilt": "attack_claws",
		"utilt": "kick_up",
		"dtilt": "kick_low",
		"dash attack": "kick_dash",
		"usmash": "kick_uair",
		"dsmash": "sword_dsmash",
		"nair": "kick_nair",
		"bair": "kick_bair",
		"neutral special": "blaster",
		"side special": "attack_kick",
		"up special": "attack_kick",
		"down special": "attack_kick",
	},
}

## The clip every move falls back to when nothing maps it (with a warning, once per move).
const FALLBACK := "attack_swing"
## Slots only a script reaches: they take the fallback quietly.
const EXTRA_PREFIX := "extra move"

## Clips whose kicking leg the game aims at the move's hitbox (fighter_view.gd `_aim_leg`). (The claws fighter's flame specials play the
## generic kick but rush the whole body, so their leg is not aimed.)
static func is_kick(clip: String) -> bool:
	return clip.begins_with("kick_")


static var _warned := {}


## The clip for move `move` of a fighter of class `cls`, checked against the clips `anim` has (when given): a class override, the shared
## archetype, or the fallback.
static func attack_clip(cls: int, move: String, anim: AnimationPlayer = null) -> String:
	var clip: String = CLASS_MOVES.get(cls, {}).get(move, SHARED_MOVES.get(move, ""))
	if clip == "":
		if not move.begins_with(EXTRA_PREFIX):
			_warn("move '%s' (class %d) has no clip in AnimLibrary; it plays %s" % [move, cls, FALLBACK])
		clip = FALLBACK
	if anim != null and not anim.has_animation(clip):
		_warn("clip '%s' for move '%s' is not in the rig; it plays %s" % [clip, move, FALLBACK])
		clip = FALLBACK
	return clip


## Clips added to the library later, and what a rig built before them plays instead (the older blob rigs have none of them).
const STAND_INS := {"jumpsquat": "crouch", "land": "crouch"}


## The shared clip for a simulation state, or "" for a state chosen with more context; with `anim`, a clip that rig lacks becomes its
## stand-in.
static func state_clip(state: String, anim: AnimationPlayer = null) -> String:
	var clip: String = STATE_CLIPS.get(state, "")
	if clip != "" and anim != null and not anim.has_animation(clip):
		clip = STAND_INS.get(clip, clip)
	return clip


## Everything wrong with the library against a rig's clips: every mapped clip must exist. Empty when all is well (tests check it).
static func problems(anim: AnimationPlayer) -> Array[String]:
	var out: Array[String] = []
	var named := {}
	for c in STATE_CLIPS.values():
		named[c] = "state"
	for c in ["idle", "walk", "run", "dash", "skid", "jump", "fall", "air_jump", "ledge_climb", "attack_low"]:
		named[c] = "state"
	for c in SHARED_MOVES.values():
		named[c] = "shared move"
	for cls in CLASS_MOVES:
		for c in CLASS_MOVES[cls].values():
			named[c] = "class %d move" % cls
	named[FALLBACK] = "fallback"
	for c in named:
		if not anim.has_animation(c):
			out.append("%s clip '%s' is missing from the rig" % [named[c], c])
	return out


static func _warn(text: String) -> void:
	if _warned.has(text):
		return
	_warned[text] = true
	push_warning(text)
