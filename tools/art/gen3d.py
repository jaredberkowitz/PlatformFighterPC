#!/usr/bin/env python3
"""Generate a 3D model with an online service (Meshy or Tripo) and download it into art/generated/<name>/.

The first step of the art workflow (docs/ART_WORKFLOW.md): concept art or a text prompt in, a raw .glb out. The raw model is never used
as it is; it goes through art/blender/import_generated.py (scale, budget, flat cel colours) before it reaches the game.

    python tools/art/gen3d.py meshy-image concept/hat_front.png --name straw_hat --polycount 1500
    python tools/art/gen3d.py meshy-text "a round wooden barrel, stylized, chunky" --name barrel --polycount 2000
    python tools/art/gen3d.py tripo-image concept/hat_front.png --name straw_hat
    python tools/art/gen3d.py status meshy-image <task id> --name straw_hat      (pick up a task that is still running)

Every request costs the account credits, so nothing is sent without --yes: without it the request is only printed. The API key comes from
the environment (MESHY_API_KEY or TRIPO_API_KEY), never from the command line or a file in the repository.

Only the Python standard library is used. Meshy follows its documented API (docs.meshy.ai, image-to-3d v1 and text-to-3d v2). Tripo
follows its v3 API (developers.tripo3d.com); its result fields are not fully documented, so the client takes the first .glb link it finds
in the finished task.
"""

import argparse
import base64
import json
import mimetypes
import os
import sys
import time
import urllib.error
import urllib.request
import uuid

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
OUT_ROOT = os.path.join(ROOT, "art", "generated")
MESHY = "https://api.meshy.ai"
TRIPO = "https://openapi.tripo3d.com/v3"
POLL_SECONDS = 10
TIMEOUT_MINUTES = 30


# ---- HTTP -------------------------------------------------------------------------------------------------------------------------------

def _request(method, url, key, body=None, content_type="application/json"):
    data = None
    headers = {"Authorization": "Bearer " + key}
    if body is not None:
        data = body if isinstance(body, bytes) else json.dumps(body).encode()
        headers["Content-Type"] = content_type
    req = urllib.request.Request(url, data=data, method=method, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=120) as r:
            return json.loads(r.read().decode() or "{}")
    except urllib.error.HTTPError as e:
        raise SystemExit("%s %s failed: HTTP %d %s" % (method, url, e.code, e.read().decode(errors="replace")[:500]))


def _download(url, path):
    with urllib.request.urlopen(url, timeout=300) as r, open(path, "wb") as f:
        while True:
            chunk = r.read(1 << 16)
            if not chunk:
                break
            f.write(chunk)


def _key(name):
    key = os.environ.get(name, "")
    if not key:
        raise SystemExit("set %s in the environment first (the key from your account page); it is never stored in the repository" % name)
    return key


def _data_uri(path):
    mime = mimetypes.guess_type(path)[0] or "image/png"
    with open(path, "rb") as f:
        return "data:%s;base64,%s" % (mime, base64.b64encode(f.read()).decode())


def _find_glb(obj):
    """The first link to a .glb anywhere in a JSON value (Tripo's result fields are not documented)."""
    if isinstance(obj, str):
        return obj if obj.startswith("http") and ".glb" in obj.split("?")[0].lower() else None
    if isinstance(obj, dict):
        # Prefer the textured model over the bare mesh when both are there.
        for k in ("pbr_model", "model", "glb"):
            if k in obj:
                found = _find_glb(obj[k])
                if found:
                    return found
        obj = list(obj.values())
    if isinstance(obj, list):
        for v in obj:
            found = _find_glb(v)
            if found:
                return found
    return None


# ---- The services -----------------------------------------------------------------------------------------------------------------------

def meshy_body(args, kind):
    if kind == "meshy-image":
        body = {"image_url": _data_uri(args.source), "should_texture": not args.no_texture, "target_formats": ["glb"],
                "should_remesh": True, "topology": "triangle", "target_polycount": args.polycount, "origin_at": "bottom"}
    else:
        body = {"mode": "preview", "prompt": args.source, "should_remesh": True, "topology": "triangle",
                "target_polycount": args.polycount}
    if args.pose:
        body["pose_mode"] = args.pose
    return body


def meshy_paths(kind):
    return "/openapi/v1/image-to-3d" if kind == "meshy-image" else "/openapi/v2/text-to-3d"


def meshy_wait(key, path, task_id):
    deadline = time.time() + TIMEOUT_MINUTES * 60
    while True:
        t = _request("GET", MESHY + path + "/" + task_id, key)
        status = t.get("status", "")
        print("  %s %s%%" % (status, t.get("progress", 0)), flush=True)
        if status == "SUCCEEDED":
            return t
        if status in ("FAILED", "CANCELED"):
            raise SystemExit("task %s: %s" % (status, (t.get("task_error") or {}).get("message", "")))
        if time.time() > deadline:
            raise SystemExit("still running after %d minutes; pick it up later with: status %s" % (TIMEOUT_MINUTES, task_id))
        time.sleep(POLL_SECONDS)


def run_meshy(args):
    kind = args.command
    body = meshy_body(args, kind)
    path = meshy_paths(kind)
    if not args.yes:
        shown = dict(body)
        if "image_url" in shown:
            shown["image_url"] = "<%s as a data URI>" % args.source
        print("would POST %s%s\n%s\n(add --yes to send it; this spends credits)" % (MESHY, path, json.dumps(shown, indent=2)))
        return
    key = _key("MESHY_API_KEY")
    task_id = _request("POST", MESHY + path, key, body)["result"]
    print("task", task_id)
    task = meshy_wait(key, path, task_id)
    if kind == "meshy-text" and args.refine:
        # The preview is an untextured shape; the refine paints it.
        refine = {"mode": "refine", "preview_task_id": task_id}
        if args.texture_prompt:
            refine["texture_prompt"] = args.texture_prompt
        task_id = _request("POST", MESHY + path, key, refine)["result"]
        print("refine task", task_id)
        task = meshy_wait(key, path, task_id)
    save(args.name, task, (task.get("model_urls") or {}).get("glb"), task.get("consumed_credits"))


def run_tripo(args):
    kind = args.command
    if kind == "tripo-image":
        body = {"input": "<file token of %s>" % args.source}
        endpoint = "/generation/image-to-model"
    else:
        body = {"prompt": args.source}
        endpoint = "/generation/text-to-model"
    for opt in args.opt or []:
        k, _, v = opt.partition("=")
        body[k] = json.loads(v) if v[:1] in "[{0123456789-tf" and v not in ("", "-") else v
    if not args.yes:
        print("would POST %s%s\n%s\n(add --yes to send it; this spends credits)" % (TRIPO, endpoint, json.dumps(body, indent=2)))
        return
    key = _key("TRIPO_API_KEY")
    if kind == "tripo-image":
        body["input"] = tripo_upload(key, args.source)
    created = _request("POST", TRIPO + endpoint, key, body)
    data = created.get("data", created)
    task_id = data.get("task_id") or data.get("id")
    if not task_id:
        raise SystemExit("no task id in the reply: %s" % json.dumps(created)[:500])
    print("task", task_id)
    task = tripo_wait(key, task_id)
    save(args.name, task, _find_glb(task), (task.get("data") or task).get("credits_consumed"))


def tripo_upload(key, path):
    boundary = uuid.uuid4().hex
    with open(path, "rb") as f:
        content = f.read()
    mime = mimetypes.guess_type(path)[0] or "image/png"
    body = (("--%s\r\nContent-Disposition: form-data; name=\"file\"; filename=\"%s\"\r\nContent-Type: %s\r\n\r\n"
             % (boundary, os.path.basename(path), mime)).encode() + content + ("\r\n--%s--\r\n" % boundary).encode())
    reply = _request("POST", TRIPO + "/files", key, body, "multipart/form-data; boundary=" + boundary)
    token = (reply.get("data") or {}).get("file_token")
    if not token:
        raise SystemExit("upload gave no file_token: %s" % json.dumps(reply)[:500])
    return token


def tripo_wait(key, task_id):
    deadline = time.time() + TIMEOUT_MINUTES * 60
    while True:
        t = _request("GET", TRIPO + "/tasks/" + task_id, key)
        data = t.get("data", t)
        status = str(data.get("status", "")).lower()
        print("  %s %s%%" % (status, data.get("progress", "")), flush=True)
        if status in ("success", "succeeded", "completed", "done"):
            return t
        if status in ("failed", "cancelled", "canceled", "banned", "expired", "error"):
            raise SystemExit("task %s: %s" % (status, json.dumps(data)[:500]))
        if time.time() > deadline:
            raise SystemExit("still running after %d minutes; pick it up later with: status tripo %s" % (TIMEOUT_MINUTES, task_id))
        time.sleep(POLL_SECONDS)


def save(name, task, glb_url, credits):
    out = os.path.join(OUT_ROOT, name)
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "task.json"), "w") as f:
        json.dump(task, f, indent=2)
    if not glb_url:
        raise SystemExit("the task finished but has no .glb link; see %s" % os.path.join(out, "task.json"))
    path = os.path.join(out, "raw.glb")
    _download(glb_url, path)
    print("saved %s (%d KB)%s" % (path, os.path.getsize(path) // 1024, "" if credits is None else ", %s credits" % credits))
    print("next: blender --background --python art/blender/import_generated.py -- --in %s --name %s" % (path, name))


def run_status(args):
    """Picks up a task that was still running when the client gave up waiting."""
    if args.service.startswith("meshy"):
        key = _key("MESHY_API_KEY")
        path = meshy_paths(args.service if args.service != "meshy" else "meshy-image")
        task = meshy_wait(key, path, args.task)
        save(args.name, task, (task.get("model_urls") or {}).get("glb"), task.get("consumed_credits"))
    else:
        key = _key("TRIPO_API_KEY")
        task = tripo_wait(key, args.task)
        save(args.name, task, _find_glb(task), (task.get("data") or task).get("credits_consumed"))


def main(argv):
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = p.add_subparsers(dest="command", required=True)
    for cmd in ("meshy-image", "meshy-text", "tripo-image", "tripo-text"):
        s = sub.add_parser(cmd)
        s.add_argument("source", help="an image file (front view of the concept art) or a text prompt")
        s.add_argument("--name", required=True, help="the asset's name: art/generated/<name>/")
        s.add_argument("--yes", action="store_true", help="really send it (spends credits)")
        if cmd.startswith("meshy"):
            s.add_argument("--polycount", type=int, default=6000, help="triangles the service remeshes to (see the budgets)")
            s.add_argument("--pose", choices=["a-pose", "t-pose"], help="for a character: ask for an A or T pose")
            s.add_argument("--no-texture", action="store_true", help="shape only (the colours are painted in Blender)")
        if cmd == "meshy-text":
            s.add_argument("--refine", action="store_true", help="also paint the preview (a second task, more credits)")
            s.add_argument("--texture-prompt", default="", help="what the refine should paint")
        if cmd.startswith("tripo"):
            s.add_argument("--opt", action="append", help="an extra request field, key=value (sent as JSON when it parses)")
    s = sub.add_parser("status")
    s.add_argument("service", choices=["meshy-image", "meshy-text", "tripo"])
    s.add_argument("task")
    s.add_argument("--name", required=True)
    args = p.parse_args(argv)
    if args.command == "status":
        run_status(args)
    elif args.command.startswith("meshy"):
        run_meshy(args)
    else:
        run_tripo(args)


if __name__ == "__main__":
    main(sys.argv[1:])
