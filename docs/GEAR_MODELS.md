# Accessories: from code-built to modelled (started 2026-10-18)

The owner: the further the art moves from code-built pieces (primitives assembled in GDScript), the better; they read as basic shapes.
Accessories move to Blender-modelled pieces over time (art/blender/make_gear.py, godot/models/gear/), each part named by what it is
made of (fabric, canvas, leather, metal, glass, straw) so the game colours it for each fighter. The built version stays as the fallback.

| Piece | Now | Needs | Priority |
|---|---|---|---|
| Sailor cap | **modelled** (crown that swells and sags, rolled brim, seam, band) | | done |
| Aviator cap | **modelled** (leather shell with a seam, ear flaps, brow band, brass goggles) | | done |
| Straw hat | **modelled** (waved, rolled brim; pinched crown; band and bow) | | done |
| Plumed helm | ball + cylinder + five spheres for the plume | a shaped helm with a brow ridge and cheek guards; a plume of real feather blades | high (Duelist) |
| Crown | cylinder + five cones | a cast band with set stones and real points | high (Seer) |
| Hood | open shell + cone cowl | already shaped; a folded edge and a draped cowl | medium (Rogue) |
| Swept spikes, topknot | sphere + cones / spheres | sculpted hair masses | medium |
| Beanie | squashed sphere + ball | a knit cuff and a ribbed crown | low |
| Glasses (shades, goggles, specs) | tori and spheres | frames with a real profile | medium |
| Collar, neckerchief, scarf, cape, sash | torus, prism, spring-chain boxes | modelled folds; the spring chains can stay as the rig | medium |
| Shirt collar and placket | prisms and a box | part of the garment mesh (make_rigged_blob.py) | medium |
