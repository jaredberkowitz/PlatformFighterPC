extends Node
## Background music, composed in code at start-up (no audio files): a looping bass line, arpeggio and drums over a four-chord progression.
## Original and cosmetic, like the sound effects. Tracks are rendered on a worker thread the first time they are asked for, so a screen never
## waits for them. F4 (the effects mute) silences it too, and `volume_db` is the one volume setting.
##
## `Music.of(node).play("menu")` starts the menu theme (nothing happens if it is already playing); `play("battle")` the match theme.

const RATE := 16000
const NAMES := ["menu", "battle"]
const BASE_VOLUME_DB := -17.0

## bpm; root note (MIDI); the four chords as [root offset in semitones, third, fifth]; which drums play.
const TRACKS := {
	"menu": {
		"bpm": 96, "root": 48, "chords": [[0, 4, 7], [5, 9, 12], [7, 11, 14], [0, 4, 7]],
		"kick": [0, 8], "snare": [], "hat": [4, 12], "bass_every": 4, "arp_step": 2,
	},
	"battle": {
		"bpm": 146, "root": 45, "chords": [[0, 3, 7], [-4, 0, 3], [3, 7, 10], [-2, 2, 5]],
		"kick": [0, 4, 8, 12], "snare": [4, 12], "hat": [2, 6, 10, 14], "bass_every": 2, "arp_step": 1,
	},
}

var tracks := {}
var wanted := ""
var current := ""
var muted := false
var volume_db := BASE_VOLUME_DB
var player: AudioStreamPlayer
var threads := {}
var force_in_headless := false


static func _hz(midi: float) -> float:
	return 440.0 * pow(2.0, (midi - 69.0) / 12.0)


## Adds a note to `buf`: a sine with a few harmonics and an exponential decay, starting at sample `at`.
static func _note(buf: PackedFloat32Array, at: int, midi: float, seconds: float, gain: float, bright: float) -> void:
	var n := mini(int(seconds * RATE), buf.size() - at)
	var f := _hz(midi)
	for i in n:
		var t := float(i) / RATE
		var env := exp(-t * (3.0 / maxf(seconds, 0.05)))
		var attack := minf(1.0, t * 400.0)
		var s := sin(TAU * f * t) + bright * 0.5 * sin(TAU * f * 2.0 * t) + bright * 0.25 * sin(TAU * f * 3.0 * t)
		buf[at + i] += s * env * attack * gain


static func _kick(buf: PackedFloat32Array, at: int, gain: float) -> void:
	var n := mini(int(0.16 * RATE), buf.size() - at)
	var phase := 0.0
	for i in n:
		var t := float(i) / RATE
		phase += TAU * lerpf(130.0, 42.0, minf(1.0, t / 0.1)) / RATE
		buf[at + i] += sin(phase) * exp(-t * 22.0) * gain


static func _noise_hit(buf: PackedFloat32Array, at: int, seconds: float, gain: float, rng: RandomNumberGenerator) -> void:
	var n := mini(int(seconds * RATE), buf.size() - at)
	var last := 0.0
	for i in n:
		var t := float(i) / RATE
		var raw := rng.randf_range(-1.0, 1.0)
		last = raw - last * 0.6   # a little high-pass colour
		buf[at + i] += last * exp(-t * (5.0 / seconds)) * gain


## One loop of a track, as a stream that repeats seamlessly (every note is cut off at the loop end).
static func render(name: String) -> AudioStreamWAV:
	var spec: Dictionary = TRACKS[name]
	var rng := RandomNumberGenerator.new()
	rng.seed = 11
	var step := 60.0 / float(spec.bpm) / 4.0 # one sixteenth, in seconds
	var step_samples := int(step * RATE)
	var chords: Array = spec.chords
	var steps := 16 * chords.size()
	var buf := PackedFloat32Array()
	buf.resize(steps * step_samples)
	var root: int = spec.root
	for s in steps:
		var bar := s / 16
		var k := s % 16
		var chord: Array = chords[bar]
		var at := s * step_samples
		if k % int(spec.bass_every) == 0:
			_note(buf, at, root + chord[0] - 12, step * spec.bass_every * 0.95, 0.34, 0.9)
		if k % int(spec.arp_step) == 0:
			var idx := (k / int(spec.arp_step)) % 4
			var tone: int = chord[idx % 3] + (12 if idx == 3 else 0)
			_note(buf, at, root + 24 + tone, step * 4.0, 0.12, 0.7)
		if spec.kick.has(k):
			_kick(buf, at, 0.55)
		if spec.snare.has(k):
			_noise_hit(buf, at, 0.14, 0.2, rng)
		if spec.hat.has(k):
			_noise_hit(buf, at, 0.04, 0.07, rng)
	# A soft pad: the chord held across its bar.
	for bar in chords.size():
		var chord: Array = chords[bar]
		for tone in chord:
			_note(buf, bar * 16 * step_samples, root + 12 + tone, step * 16.0, 0.05, 0.2)
	var peak := 0.0001
	for v in buf:
		peak = maxf(peak, absf(v))
	var data := PackedByteArray()
	data.resize(buf.size() * 2)
	for i in buf.size():
		data.encode_s16(i * 2, int(clampf(buf[i] / peak * 0.85, -1.0, 1.0) * 30000.0))
	var w := AudioStreamWAV.new()
	w.format = AudioStreamWAV.FORMAT_16_BITS
	w.mix_rate = RATE
	w.stereo = false
	w.data = data
	w.loop_mode = AudioStreamWAV.LOOP_FORWARD
	w.loop_begin = 0
	w.loop_end = buf.size()
	return w


func _ready() -> void:
	player = AudioStreamPlayer.new()
	player.bus = "Master"
	player.volume_db = volume_db
	add_child(player)


static var _instance: Node


## The music node, added to the tree's root once so it keeps playing across screens.
static func of(owner: Node) -> Node:
	if is_instance_valid(_instance):
		return _instance
	var n: Node = load("res://scripts/music.gd").new()
	n.name = "Music"
	_instance = n
	owner.get_tree().root.add_child.call_deferred(n)
	return n


## Starts a track (and stops the one before it). Rendering happens on a thread the first time; the track starts when it is ready.
func play(name: String) -> void:
	if not TRACKS.has(name) or wanted == name:
		return
	if DisplayServer.get_name() == "headless" and not force_in_headless:
		return # nothing to hear (and tests should not wait for a render)
	wanted = name
	if tracks.has(name):
		_start(name)
	elif not threads.has(name):
		var t := Thread.new()
		threads[name] = t
		t.start(func(): _finished.call_deferred(name, render(name)))


func _finished(name: String, stream: AudioStreamWAV) -> void:
	tracks[name] = stream
	if threads.has(name):
		threads[name].wait_to_finish()
		threads.erase(name)
	if wanted == name:
		_start(name)


func _start(name: String) -> void:
	if player == null:
		return
	current = name
	player.stream = tracks[name]
	player.volume_db = volume_db
	if not muted:
		player.play()


func stop() -> void:
	wanted = ""
	current = ""
	if player != null:
		player.stop()


func set_muted(on: bool) -> void:
	muted = on
	if player == null:
		return
	if muted:
		player.stop()
	elif current != "":
		player.play()


func _exit_tree() -> void:
	for n in threads:
		threads[n].wait_to_finish()
	threads.clear()
