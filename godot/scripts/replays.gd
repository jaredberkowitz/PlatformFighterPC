extends RefCounted
## Replay files on disk: every finished match is saved to `user://replays/` (the newest 50 are kept), and the Replays screen lists
## and plays them. The bytes themselves are made and read by the Rust bridge (`SimRunner.replay_bytes`, `replay_peek`, `replay_load`).

const DIR := "user://replays"
const KEEP := 50


static func save(bytes: PackedByteArray) -> String:
	if bytes.is_empty():
		return ""
	DirAccess.make_dir_recursive_absolute(DIR)
	var stamp := Time.get_datetime_string_from_system().replace("T", "_").replace(":", "-")
	var path := "%s/%s.pfr" % [DIR, stamp]
	var n := 2
	while FileAccess.file_exists(path):
		path = "%s/%s_%d.pfr" % [DIR, stamp, n]
		n += 1
	var f := FileAccess.open(path, FileAccess.WRITE)
	if f == null:
		return ""
	f.store_buffer(bytes)
	f.close()
	prune()
	return path


## The replay files, newest first (by name: they start with the date and time).
static func files() -> Array:
	var out: Array = []
	var dir := DirAccess.open(DIR)
	if dir == null:
		return out
	for f in dir.get_files():
		if f.ends_with(".pfr"):
			out.append(DIR + "/" + f)
	out.sort()
	out.reverse()
	return out


static func prune() -> void:
	var all := files()
	for i in range(KEEP, all.size()):
		DirAccess.remove_absolute(all[i])


static func read(path: String) -> PackedByteArray:
	return FileAccess.get_file_as_bytes(path)


static func delete(path: String) -> void:
	DirAccess.remove_absolute(path)
