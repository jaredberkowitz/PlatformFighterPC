"""Builds the blob fighter's body parts and exports them as one glTF file (godot/models/blob_parts.glb).

Run:  blender --background --python art/blender/make_blob_parts.py

Every part is modelled at the size and position-of-origin the game uses (the part's centre is its origin), so the game drops each
mesh where its old sphere was and only chooses the colour. Everything here is original geometry in the spirit of docs/ART_DIRECTION.md:
round, near-featureless bodies, chunky shoes, mitten hands. The game paints the flat colours; the meshes carry only shape.
"""

import math
import os

import bmesh
import bpy

OUT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "godot", "models", "blob_parts.glb"))


def clear() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()


def sphere(name: str, radius: float, scale=(1.0, 1.0, 1.0), segments=36, rings=24):
    """A smooth UV sphere with its own mesh data; returns the object and a bmesh to deform (call finish() after)."""
    mesh = bpy.data.meshes.new(name)
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=segments, v_segments=rings, radius=radius)
    for v in bm.verts:
        v.co.x *= scale[0]
        v.co.y *= scale[1]
        v.co.z *= scale[2]
    return obj, bm


def finish(obj, bm) -> None:
    bm.normal_update()
    bm.to_mesh(obj.data)
    bm.free()
    for poly in obj.data.polygons:
        poly.use_smooth = True


def make_body():
    # Blender is Z-up; the glTF exporter converts to Y-up. Work in Blender axes: up = +Z, forward (the face side) = -Y.
    obj, bm = sphere("Body", 0.62, (1.0, 0.9, 0.95))
    for v in bm.verts:
        t = -v.co.z / 0.62  # +1 at the bottom, -1 at the top
        v.co.x *= 1.0 + 0.10 * t  # a pear: wider below, narrower above
        v.co.y *= 1.0 + 0.10 * t
        if v.co.y < 0:
            v.co.y *= 1.0 + 0.05 * (1.0 - abs(t))  # a soft belly
    finish(obj, bm)
    return obj


def make_head():
    obj, bm = sphere("Head", 0.8, (1.0, 0.97, 0.94))
    for v in bm.verts:
        t = -v.co.z / 0.8
        if t > 0:  # a slightly fuller jaw
            v.co.x *= 1.0 + 0.04 * t
            v.co.y *= 1.0 + 0.04 * t
    finish(obj, bm)
    return obj


def make_foot(side: int):
    name = "FootL" if side < 0 else "FootR"
    obj, bm = sphere(name, 0.24, (1.0, 1.35, 0.8), segments=28, rings=18)
    for v in bm.verts:
        if v.co.z < -0.1:  # a flat sole
            v.co.z = -0.1 + (v.co.z + 0.1) * 0.15
        if v.co.y < 0:  # the toe cap is rounder and a little taller
            v.co.z *= 1.0 + 0.18 * min(1.0, -v.co.y / 0.3)
    finish(obj, bm)
    return obj


def make_hand(side: int):
    """A mitten: a round palm with a small thumb on the inner side."""
    name = "HandL" if side < 0 else "HandR"
    obj, bm = sphere(name, 0.22, (1.0, 1.0, 1.0), segments=28, rings=18)
    thumb = bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=10, radius=0.09)
    inner = -side  # the thumb points toward the body
    for v in thumb["verts"]:
        v.co.x = v.co.x * 0.9 + inner * 0.17
        v.co.y = v.co.y - 0.04
        v.co.z = v.co.z * 1.3 + 0.07
    finish(obj, bm)
    return obj


def main() -> None:
    clear()
    parts = [make_body(), make_head(), make_foot(-1), make_foot(1), make_hand(-1), make_hand(1)]
    bpy.ops.object.select_all(action="DESELECT")
    for p in parts:
        p.select_set(True)
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    bpy.ops.export_scene.gltf(
        filepath=OUT,
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_materials="NONE",
        export_yup=True,
    )
    print("wrote", OUT, "with", [p.name for p in parts])


main()
