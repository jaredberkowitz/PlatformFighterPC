#!/usr/bin/env python3
"""Reads a PF_ANIM_TRACE log (fighter_view.gd `_log_anim`) and reports the shared clips the way the references are measured:

  clip_report.py <log> [--player N] [--seg] [--pops] [--curve bone.axis] [--state NAME] [--height 2.24]

For each stretch of one state and clip: frames, hip height and head height as shares of the body's height, the stance's width along the stage
and in depth, the line of action (hips to head, degrees from upright, positive leaning forward), the knees' bends, the range and period of the
hips' and head's motion, and foot slide (how far a planted foot moves in the world per frame against the ground speed).

--pops lists every frame where a joint jumps further in one frame than the limits below, split into jumps at a hand-off (the frame the state or
clip changes) and inside a clip. Limits in world units a frame: head and hips 0.12, hands 0.3, feet 0.3 on the ground, 0.6 in the air.

Stdlib only."""
import argparse
import math
import re
import sys

BONES = ["hips", "spine", "head", "thigh.L", "thigh.R", "shin.L", "shin.R", "foot.L", "foot.R", "armU.L", "armU.R", "armL.L", "armL.R", "hand.L", "hand.R"]
LIMIT = {"hips": 0.12, "spine": 0.14, "head": 0.16, "hand.L": 0.3, "hand.R": 0.3, "foot.L": 0.3, "foot.R": 0.3, "shin.L": 0.25, "shin.R": 0.25}


def parse(path):
    rows = []
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            if not line.startswith("ATR "):
                continue
            p = line.split()
            r = {"player": int(p[1]), "cls": int(p[2]), "state": p[3], "move": p[4], "sf": int(p[5]), "hitlag": int(p[6]), "facing": int(p[7]),
                 "grounded": int(p[8]), "clip": p[9], "clip_pos": float(p[10])}
            for tok in p[11:]:
                k, _, v = tok.partition(":")
                nums = [float(x) for x in v.split(",")]
                r[k] = nums if len(nums) > 1 else nums[0]
            rows.append(r)
    return rows


def segments(rows):
    segs = []
    cur = []
    for r in rows:
        if cur and (r["state"] != cur[-1]["state"] or r["clip"] != cur[-1]["clip"] or r["sf"] < cur[-1]["sf"]):
            segs.append(cur)
            cur = []
        cur.append(r)
    if cur:
        segs.append(cur)
    return segs


def dist(a, b):
    return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)))


def angle_at(a, b, c):
    """The joint's bend at b, degrees: 0 straight."""
    u = [x - y for x, y in zip(b, a)]
    v = [x - y for x, y in zip(c, b)]
    lu = math.sqrt(sum(x * x for x in u)) or 1e-6
    lv = math.sqrt(sum(x * x for x in v)) or 1e-6
    return math.degrees(math.acos(max(-1.0, min(1.0, sum(x * y for x, y in zip(u, v)) / (lu * lv)))))


def period(v, lo=8):
    n = len(v)
    if n < lo * 3:
        return 0
    m = sum(v) / n
    d = [x - m for x in v]
    e = sum(x * x for x in d)
    if e < 1e-9:
        return 0
    ac = [sum(d[i] * d[i + lag] for i in range(n - lag)) / e for lag in range(n // 2)]
    for lag in range(lo, len(ac) - 1):
        if ac[lag] > ac[lag - 1] and ac[lag] >= ac[lag + 1] and ac[lag] > 0.15:
            return lag
    return 0


def seg_line(seg, H):
    n = len(seg)
    hips = [r["hips"] for r in seg]
    head = [r["head"] for r in seg]
    fl = [r["foot.L"] for r in seg]
    fr = [r["foot.R"] for r in seg]
    mean = lambda v: sum(v) / len(v)
    width_x = mean([abs(a[0] - b[0]) for a, b in zip(fl, fr)])
    width_z = mean([abs(a[2] - b[2]) for a, b in zip(fl, fr)])
    # The line of action on screen, from the hips to the head.
    loa = mean([math.degrees(math.atan2(h[0] - p[0], h[1] - p[1])) for h, p in zip(head, hips)])
    knee_l = mean([angle_at(r["thigh.L"], r["shin.L"], r["foot.L"]) for r in seg])
    knee_r = mean([angle_at(r["thigh.R"], r["shin.R"], r["foot.R"]) for r in seg])
    hip_y = [p[1] for p in hips]
    hip_x = [p[0] for p in hips]
    head_y = [p[1] for p in head]
    head_x = [p[0] for p in head]
    # Foot slide: a foot within 0.2 of the lowest it reaches this stretch counts as planted; its world motion per frame while so.
    slide = []
    for key in ("foot.L", "foot.R"):
        low = min(r[key][1] for r in seg)
        for a, b in zip(seg, seg[1:]):
            if a[key][1] < low + 0.04 and b[key][1] < low + 0.04 and a["grounded"] and b["grounded"]:
                wa = a["world"][0] + a[key][0] * a["facing"]
                wb = b["world"][0] + b[key][0] * b["facing"]
                slide.append(abs(wb - wa))
    speed = mean([abs(r["vel"][0]) for r in seg])
    slide_s = f" slide {mean(slide):.3f}/f ({mean(slide) / speed:.2f} of speed {speed:.3f})" if slide and speed > 0.01 else (f" slide {mean(slide):.3f}/f" if slide else "")
    elbow_l = mean([angle_at(r["armU.L"], r["armL.L"], r["hand.L"]) for r in seg])
    elbow_r = mean([angle_at(r["armU.R"], r["armL.R"], r["hand.R"]) for r in seg])
    near, far = ("L", "R")
    hand_n = (mean([r["hand." + near][0] for r in seg]) / H, mean([r["hand." + near][1] for r in seg]) / H)
    hand_f = (mean([r["hand." + far][0] for r in seg]) / H, mean([r["hand." + far][1] for r in seg]) / H)
    return (f"f{seg[0]['t']:.0f}+{n:<3d} {seg[0]['state']:<11s} {seg[0]['clip']:<14s} hips {mean(hip_y) / H:.3f}H ({(max(hip_y) - min(hip_y)) / H * 100:.1f}%y {(max(hip_x) - min(hip_x)) / H * 100:.1f}%x p{period(hip_y)})"
            f" head {mean(head_y) / H:.3f}H ({(max(head_y) - min(head_y)) / H * 100:.1f}%y {(max(head_x) - min(head_x)) / H * 100:.1f}%x)"
            f" feet x{width_x / H:.2f}H z{width_z / H:.2f}H loa {loa:+.0f} knees {knee_l:.0f}/{knee_r:.0f} elbows {elbow_l:.0f}/{elbow_r:.0f}"
            f" hands L{hand_n[0]:+.2f},{hand_n[1]:.2f} R{hand_f[0]:+.2f},{hand_f[1]:.2f}{slide_s}")


def pops(rows, out=sys.stdout):
    hand = {}
    inner = {}
    lines = []
    for a, b in zip(rows, rows[1:]):
        if b["sf"] < a["sf"] and b["state"] == a["state"] and b["clip"] == a["clip"]:
            handoff = True
        else:
            handoff = a["state"] != b["state"] or a["clip"] != b["clip"]
        if b.get("t", 0) - a.get("t", 0) != 1:
            continue
        for bone, lim in LIMIT.items():
            d = dist(a[bone], b[bone])
            lim2 = lim * (2.0 if bone.startswith("foot") and not (a["grounded"] and b["grounded"]) else 1.0)
            if d > lim2:
                key = f"{a['state']}/{a['clip']}>{b['state']}/{b['clip']}" if handoff else f"{b['state']}/{b['clip']}"
                bucket = hand if handoff else inner
                bucket.setdefault(key, []).append((bone, round(d, 2), b.get("t")))
    for title, bucket in (("HAND-OFF POPS", hand), ("INSIDE A CLIP", inner)):
        print(f"{title}: {sum(len(v) for v in bucket.values())}", file=out)
        for key, v in sorted(bucket.items(), key=lambda kv: -len(kv[1])):
            worst = max(v, key=lambda x: x[1])
            print(f"  {key}: {len(v)} jumps, worst {worst[0]} {worst[1]} at t{worst[2]}", file=out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("log")
    ap.add_argument("--player", type=int, default=0)
    ap.add_argument("--seg", action="store_true")
    ap.add_argument("--pops", action="store_true")
    ap.add_argument("--curve", action="append", default=[])
    ap.add_argument("--state", default="")
    ap.add_argument("--height", type=float, default=2.24)
    args = ap.parse_args()
    rows = [r for r in parse(args.log) if r["player"] == args.player]
    if args.state:
        rows_s = [r for r in rows if r["state"] == args.state]
    else:
        rows_s = rows
    if args.seg or not (args.pops or args.curve):
        for seg in segments(rows_s):
            print(seg_line(seg, args.height))
    if args.pops:
        pops(rows)
    for c in args.curve:
        bone, axis = c.split(".") if c.count(".") == 1 else (c.rsplit(".", 1))
        i = "xyz".index(axis)
        print(c, " ".join(f"{r[bone][i]:.3f}" for r in rows_s))


if __name__ == "__main__":
    main()
