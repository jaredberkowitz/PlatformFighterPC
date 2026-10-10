extends Node
## Sound effects. Hits, swings, footsteps and shield hits are recorded sounds (`godot/audio/sfx/`, CC0, from Kenney's Impact Sounds and RPG
## Audio packs, see docs/THIRD_PARTY.md), several takes of each picked at random and layered by the kind of attack and the size of the hit;
## the interface sounds, the KO and the low boom under heavy hits are synthesised at start-up. Everything is cosmetic: sounds are triggered
## from what the simulation reports and never feed back into it. F4 mutes.
##
## `play(name)` plays one effect; `watch(player, before, now)` compares two snapshots of a fighter (the dictionaries `main.gd` builds from the
## simulation) and plays whatever happened between them: a swing, a hit taken, a jump, a landing, a shield, a knock-out.

const RATE := 22050
const NAMES := ["hit_light", "hit_heavy", "whoosh", "jump", "land", "shield", "ko", "blip", "confirm", "go", "clank"]
const VOICES := 18
## The recorded sounds, by group: several takes each.
const FILE_SOUNDS := {
	"punch": ["impactPunch_medium_000", "impactPunch_medium_001", "impactPunch_medium_002", "impactPunch_medium_003", "impactPunch_medium_004"],
	"punch_heavy": ["impactPunch_heavy_000", "impactPunch_heavy_001", "impactPunch_heavy_002", "impactPunch_heavy_003", "impactPunch_heavy_004"],
	"slash": ["knifeSlice", "knifeSlice2"],
	"metal": ["impactMetal_light_000", "impactMetal_light_001", "impactMetal_light_002", "impactMetal_light_003", "impactMetal_light_004"],
	"metal_heavy": ["impactMetal_heavy_000", "impactMetal_heavy_001", "impactMetal_heavy_002"],
	"wood_heavy": ["impactWood_heavy_000", "impactWood_heavy_001", "impactWood_heavy_002"],
	"thud": ["impactSoft_heavy_000", "impactSoft_heavy_001", "impactSoft_heavy_002"],
	"glass": ["impactGlass_light_000", "impactGlass_light_001", "impactGlass_light_002"],
	"shing": ["drawKnife1", "drawKnife2", "drawKnife3"],
	"cloth": ["cloth1", "cloth2", "cloth3", "cloth4"],
	"step": ["footstep_grass_000", "footstep_grass_001", "footstep_grass_002", "footstep_grass_003", "footstep_grass_004",
		"footstep_concrete_000", "footstep_concrete_001", "footstep_concrete_002", "footstep_concrete_003", "footstep_concrete_004"],
}
var groups := {}
var rng := RandomNumberGenerator.new()

var streams := {}
var voices: Array = []
var next_voice := 0
var muted := false


static func _wav(samples: PackedFloat32Array) -> AudioStreamWAV:
	var data := PackedByteArray()
	data.resize(samples.size() * 2)
	for i in samples.size():
		data.encode_s16(i * 2, int(clampf(samples[i], -1.0, 1.0) * 30000.0))
	var w := AudioStreamWAV.new()
	w.format = AudioStreamWAV.FORMAT_16_BITS
	w.mix_rate = RATE
	w.stereo = false
	w.data = data
	return w


## A sound `seconds` long: `f(t, u)` returns one sample, t in seconds and u in 0..1 through the sound.
static func _make(seconds: float, f: Callable) -> AudioStreamWAV:
	var n := int(seconds * RATE)
	var out := PackedFloat32Array()
	out.resize(n)
	for i in n:
		out[i] = f.call(float(i) / RATE, float(i) / maxf(1.0, n - 1.0))
	return _wav(out)


static func _noise(rng: RandomNumberGenerator) -> float:
	return rng.randf_range(-1.0, 1.0)


static func build(name: String) -> AudioStreamWAV:
	var rng := RandomNumberGenerator.new()
	rng.seed = 7
	match name:
		"hit_light":
			return _make(0.16, func(t, u): return ((_noise(rng) * 0.5 + sin(TAU * lerpf(220.0, 90.0, u) * t) * 0.9) * pow(1.0 - u, 2.2)) * 0.9)
		"hit_heavy":
			return _make(0.34, func(t, u): return ((_noise(rng) * 0.6 * pow(1.0 - u, 4.0) + sin(TAU * lerpf(150.0, 45.0, u) * t) * 1.1) * pow(1.0 - u, 1.6)) * 0.95)
		"whoosh":
			# Noise that swells and thins, with a falling tone under it.
			return _make(0.28, func(t, u): return (_noise(rng) * 0.55 + sin(TAU * lerpf(520.0, 180.0, u) * t) * 0.25) * sin(PI * u) * 0.55)
		"jump":
			return _make(0.14, func(t, u): return sin(TAU * lerpf(300.0, 720.0, u) * t) * pow(1.0 - u, 1.5) * 0.5)
		"land":
			return _make(0.1, func(t, u): return (sin(TAU * lerpf(130.0, 60.0, u) * t) * 0.9 + _noise(rng) * 0.2) * pow(1.0 - u, 2.5) * 0.7)
		"shield":
			return _make(0.2, func(t, u): return (sin(TAU * lerpf(260.0, 420.0, u) * t) + sin(TAU * lerpf(390.0, 630.0, u) * t) * 0.5) * sin(PI * u) * 0.28)
		"ko":
			return _make(0.8, func(t, u): return (sin(TAU * lerpf(900.0, 90.0, u) * t) * 0.7 + _noise(rng) * 0.25 * (1.0 - u)) * pow(1.0 - u, 1.2) * 0.8)
		"blip":
			return _make(0.05, func(t, u): return sin(TAU * 760.0 * t) * pow(1.0 - u, 1.5) * 0.35)
		"confirm":
			return _make(0.16, func(t, u): return sin(TAU * (600.0 if u < 0.45 else 900.0) * t) * pow(1.0 - u, 1.3) * 0.38)
		"go":
			return _make(0.4, func(t, u): return (sin(TAU * 440.0 * t) + sin(TAU * 660.0 * t) * 0.7 + sin(TAU * 880.0 * t) * 0.4) * pow(1.0 - u, 1.4) * 0.3)
		"clank":
			return _make(0.2, func(t, u): return (sin(TAU * 1180.0 * t) * 0.5 + sin(TAU * 1770.0 * t) * 0.35 + _noise(rng) * 0.2) * pow(1.0 - u, 3.0) * 0.6)
	return _make(0.01, func(_t, _u): return 0.0)


func _ready() -> void:
	for n in NAMES:
		streams[n] = build(n)
	for g in FILE_SOUNDS:
		var takes: Array = []
		for f in FILE_SOUNDS[g]:
			var s: AudioStream = load("res://audio/sfx/%s.ogg" % f)
			if s != null:
				takes.append(s)
		groups[g] = takes
	for i in VOICES:
		var p := AudioStreamPlayer.new()
		p.bus = "Master"
		add_child(p)
		voices.append(p)


## Adds the effects to the tree's root once, so every screen can use them (`Sfx.of(self).play("blip")`).
static var _instance: Node


static func of(owner: Node) -> Node:
	if is_instance_valid(_instance):
		return _instance
	var n: Node = load("res://scripts/sfx.gd").new()
	n.name = "Sfx"
	_instance = n
	# Deferred: the tree may be busy setting up its children when a screen asks.
	owner.get_tree().root.add_child.call_deferred(n)
	return n


func play(name: String, pitch := 1.0, volume_db := 0.0) -> void:
	if muted or not streams.has(name):
		return
	var p: AudioStreamPlayer = voices[next_voice]
	next_voice = (next_voice + 1) % voices.size()
	p.stream = streams[name]
	p.pitch_scale = pitch
	p.volume_db = volume_db
	p.play()


## Plays one take, picked at random, of a recorded group (pitch varied a little so repeats don't sound identical).
func play_group(group: String, pitch := 1.0, volume_db := 0.0) -> void:
	if muted or not groups.has(group) or groups[group].is_empty():
		return
	var takes: Array = groups[group]
	var p: AudioStreamPlayer = voices[next_voice]
	next_voice = (next_voice + 1) % voices.size()
	p.stream = takes[rng.randi() % takes.size()]
	p.pitch_scale = pitch * rng.randf_range(0.94, 1.06)
	p.volume_db = volume_db
	p.play()


## A hit landing, layered by what landed it (`kind`: 0 a blade, 1 a blow, 2 the maul) and how hard (`strength`, 0..1): a blade slices
## and rings, a blow smacks, the maul thumps; a heavy hit adds a deep thud and a low boom. Returns the groups played (for tests).
func hit(kind: int, strength: float) -> Array:
	var heavy := strength > 0.5
	var played: Array = []
	match kind:
		0:
			played.append(["slash", 1.0, 0.0])
			played.append(["metal_heavy" if heavy else "metal", 1.0, -8.0])
		2:
			played.append(["wood_heavy", 0.9, 0.0])
			played.append(["punch_heavy", 0.8, -4.0])
		_:
			played.append(["punch_heavy" if heavy else "punch", 1.0, 0.0])
	if heavy:
		played.append(["thud", 0.9, -3.0])
	for g in played:
		play_group(g[0], g[1], g[2])
	if heavy:
		play("hit_heavy", 1.0, -6.0)
	return played.map(func(g): return g[0])


func toggle_mute() -> bool:
	muted = not muted
	if muted:
		for p in voices:
			p.stop()
	return muted


## What happened to one fighter between two snapshots, as sounds. Returns the names played (for tests).
func watch(player: int, before: Dictionary, now: Dictionary) -> Array:
	var played: Array = []
	if before.is_empty() or now.is_empty():
		return played
	var pitch := 1.0 + 0.04 * player
	# The swing is heard as the move swings: a few frames before its first hit (or as it starts, if its timing is not known). A blade rings
	# as it is drawn through the air.
	if now.state == "Attack":
		var timing = now.get("move_timing", PackedInt32Array())
		var at: int = int(now.get("state_frame", 0))
		var was: int = int(before.get("state_frame", -1)) if before.state == "Attack" else -1
		# A new move straight out of another starts its count again.
		if at < was:
			was = -1
		var swing: bool = before.state != "Attack"
		if timing is PackedInt32Array and timing.size() >= 2 and timing[1] > 0:
			var cue := maxi(0, timing[1] - 4)
			swing = was < cue and at >= cue
		if swing:
			played.append("shing" if int(now.get("class", 1)) == 0 else "whoosh")
	if (now.state == "JumpSquat" and before.state != "JumpSquat") or (now.state == "LedgeJump" and before.state != "LedgeJump"):
		played.append("jump")
	elif now.state == "Airborne" and before.state == "Airborne" and int(now.jumps) < int(before.jumps):
		played.append("jump")
	if now.platform >= 0 and before.platform < 0 and now.state != "Hitstun":
		played.append("land")
	if (now.state == "Shield" and before.state != "Shield"):
		played.append("shield")
	if now.stocks < before.stocks:
		played.append("ko")
	for n in played:
		match n:
			"shing":
				play_group("shing", pitch * 1.1, -4.0)
				play("whoosh", pitch, -6.0)
			"jump":
				play_group("cloth", pitch, -2.0)
				play("jump", pitch, -10.0)
			"land":
				play_group("step", pitch * 0.9, 0.0)
			_:
				play(n, pitch)
	return played
