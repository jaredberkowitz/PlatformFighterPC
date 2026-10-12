# Computer opponents

## The basic CPU (now)

A sparring partner for testing and for playing alone: `sim-core/src/cpu.rs`, `cpu::think(state, content, player, level)`.

* **How to play against it**: in the character select, **C** makes every player but player 1 a computer (its fighter is still
  chosen with the player 2 keys; a slot not yet locked in takes the fighter under its cursor). In training, **F5** switches player 2
  between the computer and a person. `--cpu` on the command line starts with player 2 a computer. `--demo=cpu` plays computer
  against computer with a picture every half second.
* **What it does**: mashes out of grabs; DIs toward the middle of the stage, techs some of the time, and in the last frames of hitstun with
  its attacker close buffers a way out (an air dodge, a jump or an aerial), as a person would; takes a ledge
  option after a moment on the ledge; recovers (air jump, then up special); waits near the edge while its opponent is off the stage
  rather than following it out; shields attacks coming at it up close and grabs out of the shield; otherwise approaches and attacks
  in range (jabs, tilts, grabs, the occasional smash; aerials toward the opponent in the air) and fast falls now and then.
* **Level** (1 to 9, the menus use 5) is how often it decides and how readily it shields, techs and attacks.
* **Deterministic**: it decides from the game state alone. Its "choices" are a hash of the frame (in windows of a few frames) and
  the player, never the game's random number generator, and it keeps no memory. The match feeds its input in as a player's input, so
  replays record and replay it exactly and rollback is unaffected. It is not used in online matches.
* Tests: `cpu::tests` (the same state gives the same input; two CPUs land hits on each other), `godot/tests/cpu_test.gd` (through
  the bridge and the character select).

## A full CPU (later)

For the finished game the plan is "smart" and "fair" opponents with difficulty levels, built on the same deterministic rule (so
they stay replayable): reading the opponent's state and frame data (punishing on shield, whiffs and landings, spacing around the
opponent's reach), edge-guarding and ledge traps, character-specific plans per moveset (the sword's tipper spacing, the claws'
combos), habits that vary per match, and difficulty from reaction time, accuracy and how much of that it uses rather than from
cheating on inputs.
