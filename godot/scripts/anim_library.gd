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
	"SpotDodge": "spot_dodge",
	"AirDodge": "air_dodge",
	"Shield": "shield",
	"ShieldDrop": "shield",
	"WallTech": "shield",
	"Hitstun": "hurt",
	"ShieldBreak": "hurt",
	"Grabbed": "hurt",
	"Rebound": "hurt",
	"Knockdown": "knockdown",
	"GetUp": "getup",
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
		# (A stomp in the simulation: the hit is at the foot.)
		"dair": "kick_dair",
		"neutral special": "blaster",
		"side special": "attack_kick",
		"up special": "attack_kick",
		"down special": "attack_kick",
	},
	# Needles: kicks for the kicks (the leg is aimed at the hit), the shared chop for the forward air and lunge for the dash attack's
	# hand strike, hands overhead for the up smash, its down air's landing hit as a landing, the rapid jab's stabs as jabs and its
	# finisher as a heavy strike; Needle Storm aims (wind-up and charge) and flicks (the throw), the line is swung out, Vanish crouches
	# into its explosion, Bouncing Fish is a heel kick and its bounce a jump. (Placeholders until the Needles clips are made.)
	2: {
		"ftilt": "kick_up",
		"utilt": "kick_up",
		"dtilt": "kick_low",
		"nair": "kick_nair",
		"bair": "kick_bair",
		"uair": "kick_uair",
		"dair": "kick_dair",
		"extra move 0": "land",
		"extra move 1": "attack_jab",
		"extra move 2": "attack_smash",
		"fsmash": "attack_kick",
		"usmash": "attack_uair",
		"dsmash": "sword_dsmash",
		"neutral special": "blaster",
		"extra move 3": "blaster",
		"extra move 4": "attack_jab",
		"side special": "attack_swing",
		"up special": "attack_low",
		"down special": "kick_dair",
		"extra move 5": "jump",
	},
	# Embers (the tumbler body): the fireball is thrown with a straight arm, the cape swept across (the rising punch keeps the shared uppercut),
	# the water pump braced with both hands forward (wind-up, charge held, the shots); the down air's landing hit is a landing.
	3: {
		"neutral special": "attack_jab",
		"side special": "attack_swing",
		"down special": "blaster",
		"extra move 0": "land",
		"extra move 1": "blaster",
		"extra move 2": "blaster",
	},
	# Gauntlets (after the reference rusher): the knee for the forward air and jab 3, the axe kick, the stomp, the shoulder charge,
	# the elbow, flip kicks for the up smash, the rapid jab's punches as jabs and its rising backhand finisher as an uppercut.
	4: {
		"jab 3": "gaunt_knee",
		"fair": "gaunt_knee",
		"utilt": "kick_axe",
		"dair": "gaunt_stomp",
		"dash attack": "gaunt_shoulder",
		"fsmash": "gaunt_elbow",
		"usmash": "kick_uair",
		"nair": "kick_nair",
		"bair": "kick_bair",
		"dtilt": "kick_low",
		"extra move 10": "attack_jab",
		"extra move 11": "attack_jab",
		"extra move 12": "attack_uair",
		# The specials: the charged punch keeps the shared heavy strike (its reversed and aerial versions too); the dash's uppercut and
		# the aerial overhand; the leaping grab's catch; the slide kick, its dive-kick and off-edge versions, and their landings.
		"extra move 0": "attack_smash",
		"extra move 1": "attack_smash",
		"extra move 2": "attack_smash",
		"extra move 3": "attack_uair",
		"extra move 4": "attack_smash",
		"extra move 5": "attack_smash",
		"extra move 6": "grab",
		"down special": "kick_low",
		"extra move 7": "kick_dair",
		"extra move 8": "land",
		"extra move 9": "kick_low",
		"extra move 13": "land",
	},
	# Staff (the Seer): the staff spun and swept with the sword's cuts (no blade is drawn), the palm pulse a straight strike, kicks for
	# the back and down airs, the down smash's burst both ways; the rapid jab's flurry as jabs and its burst as a heavy strike.
	5: {
		"ftilt": "sword_ftilt",
		"utilt": "sword_utilt",
		"uair": "sword_utilt",
		"dtilt": "sword_dtilt",
		"dsmash": "sword_dsmash",
		"fair": "attack_jab",
		# (Kicks for the back air and down air, whose hits are close to the body; the staff stays in hand. The neutral air keeps the
		# shared clip: the staff follows its hits round the body.)
		"bair": "kick_bair",
		"dair": "kick_dair",
		"extra move 10": "attack_jab",
		"extra move 11": "attack_jab",
		"extra move 12": "attack_smash",
	},
	# Greatsword (the Champion): the longsword's cuts for its sword moves (the blade follows its hits), front kicks for the first two
	# jabs, a sliding low kick for the down tilt, kicks for the throws (the throw clip, the blade at rest).
	6: {
		"jab": "kick_dash",
		"jab 2": "kick_up",
		"jab 3": "sword_ftilt",
		"ftilt": "sword_ftilt",
		"utilt": "sword_utilt",
		"dtilt": "kick_low",
		"fsmash": "sword_fsmash",
		"usmash": "sword_usmash",
		"dsmash": "sword_dsmash",
		"dair": "sword_dair",
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
const STAND_INS := {"jumpsquat": "crouch", "land": "crouch", "spot_dodge": "roll", "air_dodge": "roll", "getup": "knockdown",
	"tech": "crouch", "run_stop": "skid", "idle_look": "idle", "platform_drop": "fall", "hurt_heavy": "hurt", "hurt_up": "hurt"}

## Clips chosen in fighter_view.gd `_choose_clip` from more than the state (a tech is an intangible landing, a run stop a sliding idle, a
## platform drop the first frames of a fall through a platform, heavy hitstun a fast launch that does not tumble, a fidget a long idle).
const POLISH_CLIPS := ["tech", "run_stop", "idle_look", "platform_drop", "hurt_heavy", "hurt_up"]

## States whose clip is played by progress through the state's own length (the bridge's `fighter_dodge`), so the pose matches the
## simulation's frames whatever a fighter's dodge lengths are.
const BY_PROGRESS := ["Roll", "SpotDodge", "AirDodge", "GetUp"]


## `clip`, or its stand-in on a rig that lacks it.
static func or_stand_in(clip: String, anim: AnimationPlayer) -> String:
	if anim != null and not anim.has_animation(clip):
		return STAND_INS.get(clip, clip)
	return clip


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
	for c in ["idle", "walk", "run", "dash", "skid", "jump", "fall", "air_jump", "ledge_climb", "attack_low"] + POLISH_CLIPS:
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
