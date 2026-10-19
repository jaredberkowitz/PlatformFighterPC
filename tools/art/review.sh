#!/bin/bash
# One art review round (docs/ART_WORKFLOW.md): captures every stage at the match camera into docs/gauntlet/round<N>/, with a contact
# sheet, a greyscale of each stage (the readability check) and a before/after against the round before it.
#
#   tools/art/review.sh <round> [stages...]          e.g.  tools/art/review.sh 40      (all five)   tools/art/review.sh 40 0 4
#   PROPS=pine,bush tools/art/review.sh 40 4         line up authored props on the stage under its own light (stage_view.gd --props)
#
# Stages: 0 meadow, 1 canyon, 2 beach, 3 city, 4 grove.
set -e
cd "$(dirname "$0")/../.."
G=tools/godot/Godot_v4.7.1-stable_win64_console.exe
H=tools/art/review
N=$1; shift
STAGES=${*:-0 1 2 3 4}
OUT=docs/gauntlet/round$N
PREV=docs/gauntlet/round$((N - 1))
mkdir -p "$OUT"
EXTRA=""
[ -n "$PROPS" ] && EXTRA="--props=$PROPS"
for s in $STAGES; do
	tmp=$(mktemp -d)
	timeout 150 $G --path godot res://main.tscn -- --demo=stage --stage=$s --noui --shots="$tmp" $EXTRA 2>&1 \
		| grep -i -E "SCRIPT ERROR|Parse Error|shader error" | head -5 || true
	mv "$tmp/stage_a_stage.png" "$OUT/stage$s.png"
	rm -rf "$tmp"
	$G --headless -s $H/grey.gd -- "$OUT/stage$s.png" "$OUT/grey$s.png" > /dev/null 2>&1
	if [ -f "$PREV/stage$s.png" ]; then
		$G --headless -s $H/stack.gd -- "$OUT/before_after_$s.png" 0.5 "$PREV/stage$s.png" "$OUT/stage$s.png" > /dev/null 2>&1
	fi
	echo "stage $s -> $OUT/stage$s.png"
done
sheet=()
for s in 0 4 2 1 3; do [ -f "$OUT/stage$s.png" ] && sheet+=("$OUT/stage$s.png"); done
$G --headless -s $H/grid.gd -- "$OUT/contact_sheet.png" 0.5 "${sheet[@]}" > /dev/null 2>&1
echo "contact sheet -> $OUT/contact_sheet.png"
