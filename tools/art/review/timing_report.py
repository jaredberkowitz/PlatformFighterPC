"""Timing report for attacks, from a PF_MOTION_TRACE log (animation round 3).

Run a capture with the trace on, for example:
    PF_MOTION_TRACE=1 Godot --path godot --fixed-fps 60 res://main.tscn -- --demo=gallery --gallery-class=0 --gallery-step=40 --noui
then:
    python tools/art/review/timing_report.py <log> <class> [--frames=<json>] [--curves]

For each move it prints, from the move's first active frame S and last active frame L (from --frames, a JSON object
{"move": [S, L], ...}; the reference site's numbers, which ours match 1:1):
- FT: frames of fast motion after the last active frame (the swing carrying on) before it stops (speed under 30% of the peak);
- hold: frames from that stop until the pose has moved a quarter of the way back home (the strike or follow-through held);
- rec: frames from there until it is within 15% of home (the recovery);
- settle: the frames left;
- creep and rec speed: the mean speed through the hold and the recovery, as a share of the strike's peak speed;
- w50: the frame by which the wind-up has gone half its way; lurch: the speed from the frame before the move to its first frame, as a
  share of the peak (the reference's first gif frame is about the idle, so its frame 1 to 2 is compared).
The same measures are taken from the reference's hitbox gifs in the browser (silhouette change between frames for speed and against
the last frame for the pose's distance from home), so the two compare.
"""
import json
import math
import sys


def parse(path, cls):
    moves = {}
    order = []
    last_key = None
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.startswith("TRACE "):
            continue
        parts = line.split()
        if int(parts[1]) != cls or parts[2] != "Attack":
            last_key = None
            continue
        move, sf = parts[3], int(parts[4])
        pts = {}
        for tok in parts[6:]:
            name, xy = tok.split(":")
            x, y = xy.split(",")
            pts[name] = (float(x), float(y))
        if move not in moves:
            if last_key is not None and last_key != move:
                pass
            moves[move] = {}
            order.append(move)
        if sf not in moves[move]:
            moves[move][sf] = pts
        last_key = move
    return moves, order


def curves(frames):
    keys = sorted(frames)
    lurch = None
    if keys[0] == 0:
        keys = keys[1:]  # (frame 0 is the frame before the move: the reference's gif starts on frame 1)
        a, b = frames[0], frames[keys[0]]
        lurch = sum(math.dist(a[n], b[n]) for n in a if n in b) / len(a)
    names = [n for n in frames[keys[0]] if all(n in frames[k] for k in keys)]
    end = frames[keys[-1]]
    first = frames[keys[0]]
    D, V, D1 = [], [], []
    prev = None
    for k in keys:
        p = frames[k]
        D.append(sum(math.dist(p[n], end[n]) for n in names) / len(names))
        D1.append(sum(math.dist(p[n], first[n]) for n in names) / len(names))
        V.append(0.0 if prev is None else sum(math.dist(p[n], prev[n]) for n in names) / len(names))
        prev = p
    if lurch is not None:
        V[0] = lurch
    return keys, D, V, D1


def measure(D, V, S, L):
    """D, V indexed from frame 1 (index 0). Returns the timing measures."""
    T = len(D)
    vpk = max(V[1: min(L, T)]) or 1e-6
    stop = T
    for t in range(L, T):  # frame t+1
        if V[t] < 0.3 * vpk:
            stop = t + 1
            break
    ft = stop - L - 1
    d0 = D[stop - 1] if stop <= T else 0.0
    hold_end = T
    for t in range(stop, T):
        if D[t] < 0.75 * d0:
            hold_end = t + 1
            break
    rec_end = T
    for t in range(hold_end, T):
        if D[t] < 0.15 * d0:
            rec_end = t + 1
            break
    def mean(a):
        return sum(a) / len(a) if a else 0.0
    creep = mean(V[stop: hold_end - 1]) / vpk
    rspd = mean(V[hold_end - 1: rec_end - 1]) / vpk
    return dict(T=T, FT=ft, hold=hold_end - stop, rec=rec_end - hold_end, settle=T - rec_end, creep=round(creep, 2),
                rec_speed=round(rspd, 2), stop=stop, hold_end=hold_end, rec_end=rec_end)


def windup(D1, V, S):
    """The wind-up: w50, the frame by which the body has gone half its furthest distance from its first frame before the first active
    frame; first, the speed of the first frame of motion as a share of the peak speed up to the strike (a lurch out of the idle shows as
    a high first)."""
    mx = max(D1[: S - 1]) if S > 1 else 0.0
    w50 = next((t + 1 for t in range(S - 1) if D1[t] >= 0.5 * mx), 0)
    vpk = max(V[1: S + 1]) or 1e-6
    return w50, round(V[0] / vpk, 2)


def main():
    path, cls = sys.argv[1], int(sys.argv[2])
    fr = {}
    show = "--curves" in sys.argv
    for a in sys.argv[3:]:
        if a.startswith("--frames="):
            fr = json.load(open(a[9:]))
    moves, order = parse(path, cls)
    print("%-10s %4s %4s %4s %5s %5s %6s %6s %6s %4s %6s" % ("move", "S-L", "T", "FT", "hold", "rec", "settle", "creep", "recspd", "w50",
                                                         "lurch"))
    for m in order:
        keys, D, V, D1 = curves(moves[m])
        S, L = fr.get(m, [0, 0])
        if not S:
            continue
        r = measure(D, V, S, L)
        w50, first = windup(D1, V, S)
        print("%-10s %2d-%-2d %3d %4d %5d %5d %6d %6.2f %6.2f %4d %6.2f" % (m, S, L, r["T"], r["FT"], r["hold"], r["rec"], r["settle"],
                                                                           r["creep"], r["rec_speed"], w50, first))
        if show:
            dp = max(D[S - 1:]) or 1
            vp = max(V[:L]) or 1
            print("   D " + " ".join("%.2f" % (d / dp) for d in D))
            print("   V " + " ".join("%.2f" % (v / vp) for v in V))


if __name__ == "__main__":
    main()
