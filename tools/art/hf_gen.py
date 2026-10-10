#!/usr/bin/env python3
"""Free model generation through public Hugging Face demos (no account needed; a free account's token gives more GPU time).

  text-image  a picture from a prompt with FLUX.1-schnell (Apache-2.0)          -> art/generated/<name>/concept.png
  image-3d    a textured model from a picture with Microsoft TRELLIS.2 (MIT)     -> art/generated/<name>/raw.glb
  prop        both: a prompt in the prop style, then the model

    python tools/art/hf_gen.py prop "a wooden barrel with iron bands" --name barrel
    python tools/art/hf_gen.py image-3d art/concept/hat_front.png --name hat

The prompt is wrapped in the style sheet's prop wording (docs/ART_WORKFLOW.md) unless --raw. The raw model then goes through
art/blender/import_generated.py like any other. The demos are shared and rate-limited (ZeroGPU): without a token a few models a day;
log in with `hf auth login` (a free Hugging Face account's read token) or set HF_TOKEN for more. Both models' licences allow commercial use; their
outputs are ours.

Only the Python standard library is used: the demos are called through Gradio's HTTP queue protocol (upload, join the queue with a
session, read the event stream), so TRELLIS.2's steps (clean the picture, generate, export) share one session as they do in its page.
"""

import argparse
import json
import mimetypes
import os
import random
import string
import sys
import time
import urllib.error
import urllib.request
import uuid

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
OUT_ROOT = os.path.join(ROOT, "art", "generated")
FLUX = "https://black-forest-labs-flux-1-schnell.hf.space"
TRELLIS = "https://microsoft-trellis-2.hf.space"
SF3D = "https://stabilityai-stable-fast-3d.hf.space"
TRELLIS1 = "https://trellis-community-trellis.hf.space"
PROP_STYLE = ("{}, a single object, chunky stylized cartoon game asset, rounded simple shapes, bold flat colours, soft cel shading, "
              "no text, no shadow, plain white background, straight-on front view at eye level, upright, centered, full object in frame")


def _token():
    """The Hugging Face token: HF_TOKEN, or the one `hf auth login` stored (as the Hugging Face tools look for it). Never printed."""
    token = os.environ.get("HF_TOKEN", "").strip()
    if token.startswith("hf_"):
        return token
    home = os.environ.get("HF_HOME") or os.path.join(os.path.expanduser("~"), ".cache", "huggingface")
    try:
        with open(os.path.join(home, "token")) as f:
            stored = f.read().strip()
        if stored.startswith("hf_"):
            return stored
    except OSError:
        pass
    return ""


def _headers(extra=None):
    h = {"User-Agent": "platform-fighter-art-pipeline"}
    token = _token()
    if token:
        h["Authorization"] = "Bearer " + token
    if extra:
        h.update(extra)
    return h


def _post_json(url, body):
    req = urllib.request.Request(url, data=json.dumps(body).encode(), method="POST", headers=_headers({"Content-Type": "application/json"}))
    try:
        with urllib.request.urlopen(req, timeout=120) as r:
            return json.loads(r.read().decode() or "{}")
    except urllib.error.HTTPError as e:
        raise SystemExit("POST %s failed: HTTP %d %s" % (url, e.code, e.read().decode(errors="replace")[:400]))


def _events(url, timeout_s=900):
    """The server-sent events of a queue or call stream, as (event, data) pairs."""
    req = urllib.request.Request(url, headers=_headers({"Accept": "text/event-stream"}))
    deadline = time.time() + timeout_s
    with urllib.request.urlopen(req, timeout=timeout_s) as r:
        event = "message"
        for raw in r:
            if time.time() > deadline:
                raise SystemExit("timed out waiting for %s" % url)
            line = raw.decode(errors="replace").rstrip("\r\n")
            if line.startswith("event:"):
                event = line[6:].strip()
            elif line.startswith("data:"):
                yield event, line[5:].strip()
                event = "message"


def _download(url, path):
    req = urllib.request.Request(url, headers=_headers())
    with urllib.request.urlopen(req, timeout=300) as r, open(path, "wb") as f:
        f.write(r.read())


def _file_url(root, value):
    """The download link of a Gradio file value (or of the value inside a component update)."""
    if isinstance(value, dict) and isinstance(value.get("value"), (dict, str)):
        value = value["value"]
    if isinstance(value, dict):
        if value.get("url"):
            return value["url"]
        if value.get("path"):
            return "%s/gradio_api/file=%s" % (root, value["path"])
    if isinstance(value, str):
        return value if value.startswith("http") else "%s/gradio_api/file=%s" % (root, value)
    raise SystemExit("no file in the reply: %s" % json.dumps(value)[:300])


def _upload(root, path, prefix="/gradio_api"):
    boundary = uuid.uuid4().hex
    with open(path, "rb") as f:
        content = f.read()
    mime = mimetypes.guess_type(path)[0] or "image/png"
    body = (("--%s\r\nContent-Disposition: form-data; name=\"files\"; filename=\"%s\"\r\nContent-Type: %s\r\n\r\n"
             % (boundary, os.path.basename(path), mime)).encode() + content + ("\r\n--%s--\r\n" % boundary).encode())
    req = urllib.request.Request(root + prefix + "/upload", data=body, method="POST",
                                 headers=_headers({"Content-Type": "multipart/form-data; boundary=" + boundary}))
    with urllib.request.urlopen(req, timeout=120) as r:
        paths = json.loads(r.read().decode())
    return {"path": paths[0], "orig_name": os.path.basename(path), "mime_type": mime, "meta": {"_type": "gradio.FileData"}}


def _out(name):
    out = os.path.join(OUT_ROOT, name)
    os.makedirs(out, exist_ok=True)
    return out


# ---- FLUX.1-schnell: a picture from a prompt -------------------------------------------------------------------------------------------

def text_image(prompt, name, seed=None):
    data = [prompt, seed if seed is not None else 0, seed is None, 1024, 1024, 4]
    event_id = _post_json(FLUX + "/gradio_api/call/infer", {"data": data})["event_id"]
    print("picture: queued", flush=True)
    for event, payload in _events("%s/gradio_api/call/infer/%s" % (FLUX, event_id)):
        if event == "complete":
            result = json.loads(payload)
            path = os.path.join(_out(name), "concept.png")
            _download(_file_url(FLUX, result[0]), path)
            print("picture: saved %s" % path)
            return path
        if event == "error":
            raise SystemExit("the picture failed: %s (the demo may be busy or out of free GPU time; try later or set HF_TOKEN)" % payload)
    raise SystemExit("the picture stream ended without a result")


# ---- TRELLIS.2: a model from a picture -------------------------------------------------------------------------------------------------

def _queue(root, session, fn_index, data, label, prefix="/gradio_api"):
    _post_json(root + prefix + "/queue/join", {"data": data, "event_data": None, "fn_index": fn_index, "trigger_id": None,
                                               "session_hash": session})
    for _, payload in _events("%s%s/queue/data?session_hash=%s" % (root, prefix, session)):
        msg = json.loads(payload)
        kind = msg.get("msg")
        if kind == "estimation" and msg.get("rank") is not None:
            print("  %s: place %s in the queue" % (label, msg.get("rank")), flush=True)
        elif kind == "process_starts":
            print("  %s: running" % label, flush=True)
        elif kind == "process_completed":
            if not msg.get("success", False):
                raise SystemExit("%s failed: %s (the demo may be out of free GPU time; try later or set HF_TOKEN)"
                                 % (label, json.dumps(msg.get("output"))[:400]))
            return (msg.get("output") or {}).get("data", [])
        elif kind == "close_stream":
            break
    raise SystemExit("%s: the stream ended without a result" % label)


def _fn_index(config, api_name):
    for i, dep in enumerate(config.get("dependencies", [])):
        if dep.get("api_name") == api_name:
            return dep.get("id", i)
    raise SystemExit("the demo has no step called %s (it may have changed)" % api_name)


def image_3d(image, name, resolution="1024", tris=200000, texture=1024):
    req = urllib.request.Request(TRELLIS + "/config", headers=_headers())
    with urllib.request.urlopen(req, timeout=60) as r:
        config = json.loads(r.read().decode())
    session = "".join(random.choice(string.ascii_lowercase + string.digits) for _ in range(11))
    _queue(TRELLIS, session, _fn_index(config, "start_session"), [], "session")
    picture = _upload(TRELLIS, image)
    cleaned = _queue(TRELLIS, session, _fn_index(config, "preprocess_image"), [picture], "clean the picture")[0]
    # The defaults of the demo's page for everything but the resolution.
    params = [cleaned, 0, resolution, 7.5, 0.7, 12, 5.0, 7.5, 0.5, 12, 3.0, 1.0, 0.0, 12, 3.0]
    _queue(TRELLIS, session, _fn_index(config, "image_to_3d"), params, "generate")
    # (Its first input is the session's stored result: sent as nothing, the server fills it in.)
    exported = _queue(TRELLIS, session, _fn_index(config, "extract_glb"), [None, tris, texture], "export")
    path = os.path.join(_out(name), "raw.glb")
    _download(_file_url(TRELLIS, exported[-1]), path)
    with open(os.path.join(_out(name), "source.json"), "w") as f:
        json.dump({"model": "microsoft/TRELLIS.2 (MIT)", "picture": os.path.relpath(image, ROOT).replace("\\", "/"),
                   "resolution": resolution}, f, indent=2)
    print("model: saved %s (%d KB)" % (path, os.path.getsize(path) // 1024))
    print("next: blender --background --python art/blender/import_generated.py -- --in %s --name %s --kind prop" % (path, name))
    return path


# ---- TRELLIS (the first version): a textured model from a picture, on a smaller GPU -----------------------------------------------------

def image_3d_trellis1(image, name, texture=1024):
    """Microsoft TRELLIS, the first version (MIT): textured models nearly as good as TRELLIS.2's for simple props, on a smaller GPU, so
    it fits the free GPU time far more often. One step generates and exports."""
    req = urllib.request.Request(TRELLIS1 + "/config", headers=_headers())
    with urllib.request.urlopen(req, timeout=60) as r:
        config = json.loads(r.read().decode())
    session = "".join(random.choice(string.ascii_lowercase + string.digits) for _ in range(11))
    _queue(TRELLIS1, session, _fn_index(config, "start_session"), [], "session")
    picture = _upload(TRELLIS1, image)
    cleaned = _queue(TRELLIS1, session, _fn_index(config, "preprocess_image"), [picture], "clean the picture")[0]
    # Picture, no extra views, the page's own state, seed, then its default sampling settings, simplify 0.95, the texture size.
    made = _queue(TRELLIS1, session, _fn_index(config, "generate_and_extract_glb"),
                  [cleaned, [], None, 0, 7.5, 12, 3.0, 12, "stochastic", 0.95, texture], "generate")
    path = os.path.join(_out(name), "raw.glb")
    _download(_file_url(TRELLIS1, made[-1] if made[-1] else made[-2]), path)
    with open(os.path.join(_out(name), "source.json"), "w") as f:
        json.dump({"model": "microsoft/TRELLIS (MIT)", "picture": os.path.relpath(image, ROOT).replace("\\", "/")}, f, indent=2)
    print("model: saved %s (%d KB)" % (path, os.path.getsize(path) // 1024))
    return path


# ---- Stable Fast 3D: a model from a picture in seconds ----------------------------------------------------------------------------------

def image_3d_sf3d(image, name, texture=1024):
    """Stable Fast 3D (Stability AI Community License: free under $1M a year in revenue, commercial use needs a free registration; the
    outputs are ours). It runs in seconds, so it costs little of the free GPU time. Its page works in two clicks that keep state in the
    session: the background is taken off the picture, then the model is made from it."""
    prefix = ""   # (an older Gradio: no /gradio_api prefix)
    req = urllib.request.Request(SF3D + "/config", headers=_headers())
    with urllib.request.urlopen(req, timeout=60) as r:
        config = json.loads(r.read().decode())
    session = "".join(random.choice(string.ascii_lowercase + string.digits) for _ in range(11))
    picture = _upload(SF3D, image, prefix)
    ratio = 0.85
    state = _queue(SF3D, session, _fn_index(config, "requires_bg_remove"), [picture, ratio], "check the picture", prefix)
    button = state[0].get("value") if isinstance(state[0], dict) else state[0]
    if button == "Remove Background":
        _queue(SF3D, session, _fn_index(config, "run_button"), ["Remove Background", picture, None, ratio, "None", -1, texture],
               "remove the background", prefix)
    made = _queue(SF3D, session, _fn_index(config, "run_button"), ["Run", picture, None, ratio, "None", -1, texture], "generate", prefix)
    path = os.path.join(_out(name), "raw.glb")
    _download(_file_url(SF3D, made[4]), path)
    with open(os.path.join(_out(name), "source.json"), "w") as f:
        json.dump({"model": "stabilityai/stable-fast-3d (Stability AI Community License)",
                   "picture": os.path.relpath(image, ROOT).replace("\\", "/")}, f, indent=2)
    print("model: saved %s (%d KB)" % (path, os.path.getsize(path) // 1024))
    return path


def main(argv):
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = p.add_subparsers(dest="command", required=True)
    s = sub.add_parser("text-image")
    s.add_argument("prompt")
    s.add_argument("--name", required=True)
    s.add_argument("--raw", action="store_true", help="send the prompt as it is, without the prop style wording")
    s.add_argument("--seed", type=int)
    s = sub.add_parser("image-3d")
    s.add_argument("image")
    s.add_argument("--name", required=True)
    s.add_argument("--resolution", default="1024", choices=["512", "1024", "1536"])
    s.add_argument("--model", default="trellis", choices=["trellis", "trellis2", "sf3d"],
                   help="trellis (MIT, good, fits the free GPU time), trellis2 (MIT, best, needs about twice a free day's GPU time) or "
                        "sf3d (Stability community licence, seconds, weaker)")
    s = sub.add_parser("prop")
    s.add_argument("prompt")
    s.add_argument("--name", required=True)
    s.add_argument("--raw", action="store_true")
    s.add_argument("--seed", type=int)
    s.add_argument("--resolution", default="1024", choices=["512", "1024", "1536"])
    s.add_argument("--model", default="trellis", choices=["trellis", "trellis2", "sf3d"])
    args = p.parse_args(argv)
    if args.command in ("text-image", "prop"):
        prompt = args.prompt if args.raw else PROP_STYLE.format(args.prompt)
        picture = text_image(prompt, args.name, args.seed)
        if args.command == "prop":
            make_model(args.model, picture, args.name, args.resolution)
    else:
        make_model(args.model, os.path.abspath(args.image), args.name, args.resolution)


def make_model(model, picture, name, resolution):
    if model == "sf3d":
        return image_3d_sf3d(picture, name)
    if model == "trellis2":
        return image_3d(picture, name, resolution)
    return image_3d_trellis1(picture, name)


if __name__ == "__main__":
    main(sys.argv[1:])
