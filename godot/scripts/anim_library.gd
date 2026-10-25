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
	"dash grab": "body_dash_grab",
	"pivot grab": "body_pivot_grab",
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
		# (Its reference's: a held double kick, leaning back.)
		"nair": "kick_double",
		"bair": "kick_bair",
		# (A flip kick overhead, as its reference's.)
		"uair": "kick_uair",
		# (A stomp in the simulation: the hit is at the foot.)
		"dair": "kick_dair",
		"neutral special": "blaster",
		"side special": "attack_kick",
		# (The flash dash started in the air, its own slot since sim 143: the side special's clip.)
		"extra move 3": "attack_kick",
		"up special": "attack_kick",
		"down special": "attack_kick",
	},
	# Needles: kicks for the kicks (the leg is aimed at the hit), the shared chop for the forward air and lunge for the dash attack's
	# hand strike, hands overhead for the up smash, its down air's landing hit as a landing, the rapid jab's stabs as jabs and its
	# finisher as a heavy strike; Needle Volley aims (wind-up and charge) and flicks (the throw), the line is swung out, Flash Step crouches
	# into its explosion, Flip Kick is a heel kick and its bounce a jump. (Placeholders until the Needles clips are made.)
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
		# (Its forward smash is two kicks, the second a side kick with the body leaned right back, and its down smash a low spinning kick,
		# as its reference's: the leg aimed at each hit.)
		"fsmash": "kick_side",
		"usmash": "attack_uair",
		"dsmash": "kick_low",
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
		# After the reference all-rounder: two punches and a kick for the jab, a side kick, a sweep kick, a slide, the headbutt up smash,
		# a breakdance sweep in front and behind, the held flying kick, a drop kick behind and a flip kick overhead (the leg aimed at each).
		"jab 3": "kick_up",
		"ftilt": "kick_side",
		"dtilt": "kick_low",
		"dash attack": "kick_dash",
		"usmash": "body_headbutt",
		"dsmash": "kick_low",
		"nair": "kick_nair",
		"bair": "kick_bair",
		"uair": "kick_uair",
		"neutral special": "attack_jab",
		"side special": "attack_swing",
		"down special": "blaster",
		# (Its down air spins like a top with both arms out, the game turning the body: pass 13.)
		"dair": "spin_arms_air",
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
		# (Its reference's back air is a backhand, the shared back air; its forward tilt a side kick, its down smash a kick in front then
		# behind and its up air a flip kick overhead.)
		"ftilt": "kick_side",
		"dsmash": "kick_low",
		"uair": "kick_uair",
		"dtilt": "kick_low",
		"extra move 10": "attack_jab",
		"extra move 11": "attack_jab",
		"extra move 12": "attack_uair",
		# The specials: the charged punch its own (turned away and wound right back, then a huge lunge: pass 12; its reversed and aerial
		# versions too); the dash's uppercut and
		# the aerial overhand; the leaping grab's catch; the slide kick, its dive-kick and off-edge versions, and their landings.
		"neutral special": "gaunt_punch",
		"extra move 0": "gaunt_punch",
		"extra move 1": "gaunt_punch",
		"extra move 2": "gaunt_punch",
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
		# (A plunge, the blade driven straight down below, as its reference's.)
		"dair": "sword_plunge",
		# The gauge charge: a held, crouched stance with the blade at rest (no hitbox).
		"down special": "crouch",
	},
	# Clutch (the Bounder): kicks for its jabs, dash attack and airs, its tail swipes as low and high kicks, a flip kick overhead for the up
	# smash, the headbutts as heavy strikes. (Placeholders until its own clips; the leg is aimed at each hit.)
	8: {
		"jab": "kick_low",
		"jab 2": "kick_low",
		"ftilt": "kick_low",
		"utilt": "kick_up",
		"dtilt": "kick_low",
		"dash attack": "kick_dash",
		# (Its reference headbutts up and forward for its smashes and its forward air; its tail sweeps low both ways.)
		"usmash": "body_headbutt",
		"fsmash": "body_headbutt_fwd",
		"fair": "body_headbutt_fwd",
		"dsmash": "kick_low",
		"nair": "kick_nair",
		"bair": "kick_bair",
		"uair": "kick_uair",
		"dair": "kick_dair",
		# (The tongue shot out: the head lunged forward, mouth first; the drop: a hop, then seated bottom first, its landing a landing.
		# Pass 13.)
		"neutral special": "body_headbutt_fwd",
		"down special": "body_drop",
		"extra move 4": "body_drop",
		"extra move 5": "land",
	},
	# Buckler (the Wanderer): the longsword's cuts (its blade follows its hits).
	9: {
		# (Its reference's neutral and back airs are kicks; its down and up airs thrust the blade straight down and up.)
		"nair": "kick_nair",
		"bair": "kick_bair",
		"dair": "sword_plunge",
		"uair": "sword_rise",
		"ftilt": "sword_ftilt",
		"utilt": "sword_utilt",
		"dtilt": "sword_dtilt",
		"fsmash": "sword_fsmash",
		"usmash": "sword_usmash",
		"dsmash": "sword_dsmash",
		# (The spin attack: crouched low and wide, the game turning the body round twice with the blade held out; the aerial spin
		# tucked. Pass 14.)
		"up special": "sword_dsmash",
		"extra move 0": "attack_nair",
	},
	# Pockets (the Settler, after the reference villager): the shovel dug in front and behind, the slingshot drawn and loosed forward
	# (aimed like the blaster); its back air keeps the shared turn to face behind.
	10: {
		# The normals: the umbrella brought down in front, the bowling ball heaved out and dropped, the firework held up overhead, the
		# trip with the flowerpot, the shovel dug in front and behind, the slingshot loosed forward; the turnips held up and down.
		"ftilt": "tool_chop",
		"fsmash": "tool_heave",
		"usmash": "tool_raise",
		"dash attack": "tool_trip",
		"dsmash": "sword_dsmash",
		"fair": "blaster",
		# (The neutral air spins with both arms out, the game turning the body: pass 15.)
		"nair": "spin_arms_air",
		"extra move 0": "attack_uair",
		"extra move 1": "attack_uair",
		"extra move 2": "attack_dair",
		"extra move 3": "attack_dair",
		# The specials: the pocket's reach and toss, the rocket pulled out and ridden (bounced, crashed, hopped off), the balloons, the
		# planting, the watering can, the axe, the failed planting's shrug.
		"neutral special": "tool_reach",
		"extra move 9": "tool_toss",
		"side special": "tool_raise",
		"extra move 4": "tool_ride",
		"extra move 5": "tool_ride",
		"extra move 6": "hurt",
		"extra move 7": "jump",
		"up special": "tool_float",
		"extra move 8": "tool_float",
		"down special": "tool_dig",
		"extra move 10": "tool_dig",
		"extra move 11": "tool_pour",
		"extra move 12": "tool_chop",
		"extra move 13": "tool_shrug",
	},
	# Popgun (the Scamp, after its reference's moves; it had no clips of its own: every move was a shared punch). Pass 15: a side kick to
	# end the jab, slaps swung across for the tilts and the forward smash (the forward tilt with both hands), flip kicks overhead for the
	# up smash and up air, the slaps in front then behind low, kicks for the forward and back airs, a spin with the arms out for the
	# neutral air (the game turns the body), both feet stomped down, the popgun drawn and aimed, the peel tossed over the shoulder, the
	# cartwheeling dash attack as a flying kick.
	11: {
		"jab 3": "kick_side",
		"dash attack": "kick_dash",
		"fsmash": "attack_swing",
		"usmash": "kick_uair",
		"dsmash": "sword_dsmash",
		"nair": "spin_arms_air",
		"fair": "kick_nair",
		"bair": "kick_bair",
		"uair": "kick_uair",
		"dair": "gaunt_stomp",
		"neutral special": "blaster",
		"down special": "tool_toss",
	},
	# Nodachi: the longsword's cuts for its sword moves (its long blade follows its hits on the weapon pivot, as the other swords'), the
	# down air a plunge straight down. (Animation overhaul pass 9.)
	13: {
		"ftilt": "sword_ftilt",
		"utilt": "sword_utilt",
		"dtilt": "sword_dtilt",
		"fsmash": "sword_fsmash",
		"usmash": "sword_usmash",
		"dsmash": "sword_dsmash",
		"dair": "sword_plunge",
	},
	# Dagger (the Phantom, after its reference's knife moves; the knife rides the weapon hand, which the game reaches to each hit, and the
	# kicks' leg is aimed): an inward slash, a backhanded stab and an outward slash for the jabs, the double thrust as a lunge, the
	# baseball slide low, the roundhouse dash attack as a flying kick, the forward smash leaned back and cut down into a deep lunge, the
	# spinning uppercut rising onto the feet, the sweep in front then behind, kicks for the forward and up airs, the gun drawn and aimed,
	# the curse bolt flicked from an outstretched hand, the guard's backhand a swing. (Animation overhaul pass 11; were the shared punches.)
	12: {
		"jab": "attack_swing",
		"jab 3": "attack_swing",
		"ftilt": "attack_lunge",
		"dtilt": "kick_low",
		"dash attack": "kick_dash",
		"fsmash": "attack_knife_smash",
		"usmash": "sword_usmash",
		"dsmash": "sword_dsmash",
		"fair": "kick_nair",
		"uair": "kick_uair",
		"neutral special": "blaster",
		"side special": "blaster",
		"extra move 3": "blaster",
		"extra move 4": "blaster",
		"extra move 5": "blaster",
		"extra move 6": "blaster",
		"extra move 7": "blaster",
		"extra move 8": "blaster",
		"extra move 9": "attack_low",
		"extra move 11": "attack_low",
		"extra move 12": "attack_low",
		"extra move 13": "attack_low",
		"extra move 14": "attack_low",
		"extra move 0": "attack_uair",
		"extra move 2": "attack_uair",
		"extra move 1": "throw",
	},
	# Bracers (the Hauler, unarmed and heavy): a hook and an uppercut for the jabs, swings for the swipe and the arms-out spin, the
	# uppercut for the overhead sweep and the headbutt, a low slap, a rolling lunge, heavy two-handed claps for the forward and up
	# smashes, both fists slammed down either side, a heavy overhead hammer for the forward air, a back kick, a stomp. (Placeholders
	# from the shared clips until its own are made; the arms reach the hitboxes either way.)
	# (The hook, the swipe, the overhead sweep, the low slap, the lunge and the forward smash keep the shared clips.)
	7: {
		"jab 2": "attack_uair",
		"usmash": "attack_uair",
		"dsmash": "sword_dsmash",
		# (The spins with both arms out, the game turning the body: pass 13.)
		"nair": "spin_arms_air",
		"up special": "spin_arms",
		"extra move 6": "spin_arms_air",
		# (Both fists raised overhead and hammered down in front: the shared down air's two-handed sweep.)
		"fair": "attack_dair",
		"bair": "kick_bair",
		"uair": "body_headbutt",
		"dair": "gaunt_stomp",
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


## Clips that pose both arms themselves: the tool hand is not reached to a hit (fighter_view.gd).
const OWN_ARMS := ["tool_float", "tool_raise", "tool_shrug", "tool_ride", "tool_reach", "tool_toss", "spin_arms", "spin_arms_air"]


## Clips where the whole body is the weapon (a headbutt): no arm or leg reaches for the hitbox, the pose carries the hit.
static func is_body(clip: String) -> bool:
	return clip.begins_with("body_")


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
	"tech": "crouch", "run_stop": "skid", "idle_look": "idle", "platform_drop": "fall", "hurt_heavy": "hurt", "hurt_up": "hurt",
	"body_dash_grab": "grab", "body_pivot_grab": "grab", "fall_start": "fall", "tumble": "hurt"}

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


## Each unarmed family's own stance and run by class (the base body's rig only; the others play the shared clips): the brawlers (Claws,
## Gauntlets, Bracers) hunch over bent knees with both fists up (their own idle, the shared run), the armed classes keep the sword stance
## and the shared run, the quick fighters (Needles, Dagger) crouch low (the Nodachi, a sword class since pass 9, keeps the sword stance)
## and sprint long, the heavy (Clutch) stands wide and lumbers, the caster (Embers) holds a palm out and runs neat, the tool users
## (Pockets, Popgun) stand relaxed and jog loose. The pose review, 2026-10-17.
const CLASS_FAMILY := {1: "brawler", 4: "brawler", 7: "brawler", 2: "quick", 12: "quick", 8: "heavy", 3: "caster", 10: "tool",
	11: "tool"}
## The shared clips a family has its own take on (`<clip>_<family>` in the rig).
const FAMILY_CLIPS := ["idle", "run"]
## Families that keep a shared clip of FAMILY_CLIPS rather than having their own (the brawlers run the shared run).
const FAMILY_SHARES := {"brawler": ["run"]}


## Class `cls`'s own take on the shared clip `clip` (its family's, `<clip>_<family>`) when it has one and the rig has it, else `clip`.
static func family_clip(cls: int, clip: String, anim: AnimationPlayer = null) -> String:
	if not FAMILY_CLIPS.has(clip) or not CLASS_FAMILY.has(cls):
		return clip
	if (FAMILY_SHARES.get(CLASS_FAMILY[cls], []) as Array).has(clip):
		return clip
	var own := "%s_%s" % [clip, CLASS_FAMILY[cls]]
	if anim != null and not anim.has_animation(own):
		return clip
	return own


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
