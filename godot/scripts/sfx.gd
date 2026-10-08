extends Node
## Sound effects, made from scratch at start-up (no audio files): short synthesised bursts, sweeps and blips. Everything here is original
## and cosmetic: sounds are triggered from what the simulation reports and never feed back into it. F4 mutes.
##
## `play(name)` plays one effect; `watch(player, before, now)` compares two snapshots of a fighter (the dictionaries `main.gd` builds from the
## simulation) and plays whatever happened between them: a swing, a hit taken, a jump, a landing, a shield, a knock-out.

const RATE := 22050
const NAMES := ["hit_light", "hit_heavy", "whoosh", "jump", "land", "shield", "ko", "blip", "confirm", "go", "clank"]
const VOICES := 10

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
	for i in VOICES:
		var p := AudioStreamPlayer.new()
		p.bus = "Master"
		add_child(p)
		voices.append(p)


## Adds the effects to the tree's root once, so every screen can use them (`Sfx.of(self).play("blip")`).
static func of(owner: Node) -> Node:
	var root := owner.get_tree().root
	if not root.has_node("Sfx"):
		var n: Node = load("res://scripts/sfx.gd").new()
		n.name = "Sfx"
		root.add_child(n)
	return root.get_node("Sfx")


func play(name: String, pitch := 1.0, volume_db := 0.0) -> void:
	if muted or not streams.has(name):
		return
	var p: AudioStreamPlayer = voices[next_voice]
	next_voice = (next_voice + 1) % voices.size()
	p.stream = streams[name]
	p.pitch_scale = pitch
	p.volume_db = volume_db
	p.play()


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
	if now.state == "Attack" and before.state != "Attack":
		played.append("whoosh")
	if now.percent > before.percent + 0.01:
		var heavy: bool = now.percent - before.percent >= 11.0 or now.tumble
		played.append("hit_heavy" if heavy else "hit_light")
	if now.state == "JumpSquat" and before.state != "JumpSquat":
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
		play(n, pitch)
	return played
