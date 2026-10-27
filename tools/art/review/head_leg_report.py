"""Reads a PF_HEAD_TRACE log (fighter_view.gd `_log_head_legs`) and reports the head's and the legs' motion: turn speeds and snaps,
where the face limiter engages, roll wobble, knee directions, shin twist, feet and leg crossing, by state and move.

    python tools/art/review/head_leg_report.py <log> [--player=N] [--by=state|move] [--frames=STATE[:MOVE]]

Angles are degrees; yaws are measured from the camera (0) toward the facing (+90) for the head, chest and hips, and from forward (0)
toward the camera (+90) for knees and toes. A frame-to-frame change is only counted when the sim moved on (not in a hit's freeze).
"""
import sys
from collections import defaultdict

SNAP = 9.0          # a head turn faster than this per frame reads as a snap at 60 frames a second
JERK = 7.0          # a change in turn speed bigger than this per frame reads as a jolt
KNEE_CAM = 45.0     # a knee pointing more than this toward or away from the camera reads as turned in or out
POP = 25.0          # a knee bend changing more than this in one frame reads as a pop


def parse(path, player=None):
    rows = []
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.startswith("HL "):
            continue
        p = line.split()
        if len(p) > 8 and p[8].startswith("yaw:"):
            p.insert(4, "-")   # (an empty move name, older logs)
        try:
            r = dict(player=int(p[1]), cls=int(p[2]), state=p[3], move=p[4], sf=int(p[5]), hitlag=int(p[6]), facing=int(p[7]),
                     ground=int(p[8]))
            for tok in p[9:]:
                k, v = tok.split(":", 1)
                r[k] = [float(x) for x in v.split(",")]
        except (ValueError, IndexError):
            continue
        if player is not None and r["player"] != player:
            continue
        rows.append(r)
    return rows


def leg(r, side):
    v = r[side]
    return dict(flex=v[0], kyaw=v[1], kup=v[2], twist=v[3], off=v[4], toe=v[5], toep=v[6], roll=v[7], ax=v[8], ay=v[9], az=v[10],
                kz=v[11])


def key(r, by):
    return r["state"] if by == "state" or r["move"] in ("", "_") or r["state"] != "Attack" else r["state"] + ":" + r["move"]


def angdiff(a, b):
    d = (a - b + 180.0) % 360.0 - 180.0
    return d


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    opts = dict(a[2:].split("=", 1) for a in sys.argv[1:] if a.startswith("--") and "=" in a)
    rows = parse(args[0], int(opts["player"]) if "player" in opts else None)
    by = opts.get("by", "move")
    if "frames" in opts:
        want = opts["frames"].split(":")
        for r in rows:
            if r["state"] == want[0] and (len(want) == 1 or r["move"] == want[1]):
                n, f = leg(r, "N"), leg(r, "F")
                print("t%4d p%d %s %s %3d hl%-2d f%+d g%d yaw %6.1f head %6.1f %6.1f %6.1f chest %6.1f rel %6.1f %6.1f fix %5.1f %5.1f | N fl %4.0f ky %4.0f tw %4.0f off %3.0f toe %4.0f roll %4.0f z %.2f | F fl %4.0f ky %4.0f tw %4.0f off %3.0f toe %4.0f roll %4.0f z %.2f" % (
                    int(r.get("t", [0])[0]), r["player"], r["state"], r["move"], r["sf"], r["hitlag"], r["facing"], r["ground"], r["yaw"][0], *r["head"], r["chest"][0],
                    *r["rel"], *r["fix"], n["flex"], n["kyaw"], n["twist"], n["off"], n["toe"], n["roll"], n["az"], f["flex"], f["kyaw"],
                    f["twist"], f["off"], f["toe"], f["roll"], f["az"]))
        return
    groups = defaultdict(lambda: defaultdict(list))
    spikes = []
    prev = {}
    for r in rows:
        g = groups[key(r, by)]
        g["n"].append(1)
        for i, nm in enumerate(("hyaw", "hpitch", "hroll")):
            g[nm].append(r["head"][i])
        g["rel"].append(r["rel"][0])
        g["fixT"].append(r["fix"][0])
        g["fixL"].append(r["fix"][1])
        pv = prev.get(r["player"])
        moved = pv is not None and r["hitlag"] == 0 and (r["sf"] != pv["sf"] or r["state"] != pv["state"])
        if moved:
            dy = angdiff(r["head"][0], pv["head"][0]) if r["facing"] == pv["facing"] else 0.0
            dp = r["head"][1] - pv["head"][1]
            dr = angdiff(r["head"][2], pv["head"][2])
            # (Yaw and roll mean nothing for a head looking nearly straight up or down.)
            if max(abs(r["head"][1]), abs(pv["head"][1])) > 60.0:
                dy = dr = 0.0
            g["dyaw"].append(abs(dy))
            g["dpitch"].append(abs(dp))
            g["droll"].append(abs(dr))
            if "dy" in pv and r["facing"] == pv["facing"]:
                g["jerk"].append(abs(dy - pv["dy"]))
            r["dy"] = dy
            if "spikes" in opts and max(abs(dy), abs(dp), abs(dr)) > float(opts["spikes"]):
                spikes.append((max(abs(dy), abs(dp), abs(dr)), r, pv, dy, dp, dr))
            # The limiter switching on or off between frames.
            if (pv["fix"][0] > 0.5) != (r["fix"][0] > 0.5) or (pv["fix"][1] > 0.5) != (r["fix"][1] > 0.5):
                g["switch"].append(abs(dy) + abs(dp))
            for s in ("N", "F"):
                a, b = leg(r, s), leg(pv, s)
                g["pop" + s].append(abs(a["flex"] - b["flex"]))
        for s in ("N", "F"):
            L = leg(r, s)
            if L["flex"] > 25.0 and abs(L["kup"]) < 50.0:
                # (Against the hips' own forward: a knee turned in or out of the pelvis, not a body turned to the camera.)
                g["kyaw" + s].append(angdiff(90.0 - L["kyaw"], r["hips"][0]))
            g["twist" + s].append(abs(L["twist"]))
            if L["flex"] > 25.0:
                g["off" + s].append(L["off"])
            g["flex" + s].append(L["flex"])
            if r["ground"] and L["ay"] < 0.25 and abs(L["toep"]) < 50.0:
                g["toe" + s].append(L["toe"])
                g["sroll" + s].append(L["roll"])
        nn, ff = leg(r, "N"), leg(r, "F")
        g["depth"].append(nn["az"] - ff["az"])
        g["kdepth"].append(nn["kz"] - ff["kz"])
        if r["ground"]:
            g["widthx"].append(nn["ax"] - ff["ax"])
            g["widthz"].append(nn["az"] - ff["az"])
        prev[r["player"]] = r

    if "spikes" in opts:
        for m, r, pv, dy, dp, dr in sorted(spikes, key=lambda x: -x[0])[:int(opts.get("top", "40"))]:
            print("t%4d p%d %-10s %-14s sf%3d f%+d  dyaw %6.1f dpitch %6.1f droll %6.1f | head %6.1f %6.1f %6.1f <- %6.1f %6.1f %6.1f | fix %5.1f,%5.1f <- %5.1f,%5.1f rel %6.1f,%6.1f <- %6.1f,%6.1f" % (
                int(r.get("t", [0])[0]), r["player"], r["state"], r["move"], r["sf"], r["facing"], dy, dp, dr, *r["head"], *pv["head"], *r["fix"],
                *pv["fix"], *r["rel"], *pv["rel"]))
        return

    def st(v, f=max):
        return f(v) if v else 0.0

    def mean(v):
        return sum(v) / len(v) if v else 0.0

    print("%-26s %5s | %-17s %-15s %-9s %-11s %-6s %-6s | %-23s %-11s %-9s %-11s %-11s %-11s" % (
        "group", "n", "head yaw mn/mx", "|d| yaw p r", "snaps", "jerk>7", "fixon", "sw", "knee yaw N/F (in/out%)", "twist N/F", "off N/F",
        "toe N/F", "roll N/F", "farfront%"))
    for k in sorted(groups):
        g = groups[k]
        n = len(g["n"])
        snaps = sum(1 for d in g["dyaw"] if d > SNAP) + sum(1 for d in g["dpitch"] if d > SNAP) + sum(1 for d in g["droll"] if d > SNAP)
        jerks = sum(1 for d in g["jerk"] if d > JERK)
        fixon = sum(1 for v in g["fixT"] if v > 0.5) + sum(1 for v in g["fixL"] if v > 0.5)

        def kpct(s):
            v = g["kyaw" + s]
            if not v:
                return "  -  "
            cam = sum(1 for a in v if abs(a) > KNEE_CAM and abs(a) < 180 - KNEE_CAM)
            back = sum(1 for a in v if abs(a) >= 180 - KNEE_CAM)
            return "%4.0f(%2d/%2d)" % (mean(v), 100 * cam // len(v), 100 * back // len(v))

        farfront = sum(1 for d in g["depth"] if d < -0.02)
        print("%-26s %5d | %6.1f/%6.1f/%5.1f %4.1f %4.1f %4.1f %4d     %4d       %4d   %3d  | %s %s %4.0f %4.0f %4.0f %4.0f %4.0f/%4.0f %4.0f/%4.0f %3d%%" % (
            k[:26], n, st(g["hyaw"], min), st(g["hyaw"]), st(g["hroll"], lambda v: max(abs(x) for x in v)), st(g["dyaw"]), st(g["dpitch"]),
            st(g["droll"]), snaps, jerks, fixon, len(g["switch"]), kpct("N"), kpct("F"), st(g["twistN"]), st(g["twistF"]), st(g["offN"]),
            st(g["offF"]), st(g["toeN"], min), st(g["toeN"]), st(g["srollN"], min), st(g["srollN"]), 100 * farfront // max(1, n)))


if __name__ == "__main__":
    main()
