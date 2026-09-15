"""Package an existing museum comparison assembly as one static display GLB.

Run with Blender: --background --python scripts/export_art_demo_armor.py --
--renderer PATH/render_museum_armor.py --preset PATH/render-input.json
--output PATH/suit.glb --lod 4. Uses the museum renderer's import and pose rules.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--renderer", type=Path, required=True)
    parser.add_argument("--preset", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--lod", type=int, choices=[4, 5, 6], required=True)
    args = parser.parse_args(sys.argv[sys.argv.index("--") + 1:])
    spec = importlib.util.spec_from_file_location("museum_renderer", args.renderer)
    renderer = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = renderer
    spec.loader.exec_module(renderer)
    import bpy
    from armor_budget import count
    from armor_glb import Asset

    preset = renderer.Preset.load(args.preset.resolve(), None)
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    objects = []
    sources = []
    totals = {"metal": 0, "clothing": 0, "fasteners": 0}
    body_triangles = None
    for item in preset.items:
        if item.name == "body":
            if item.path.suffix == ".json":
                body_triangles = len(json.loads(item.path.read_text())["faces"])
            else:
                body = Asset(item.path)
                body_triangles = sum(body.doc["accessors"][p["indices"]]["count"] // 3
                                     for m in body.doc["meshes"] for p in m["primitives"])
        else:
            if item.path.suffix != ".glb":
                raise ValueError("display armor must use finished native LOD GLBs")
            equipment = Asset(item.path)
            if equipment.doc["extras"]["adventuresim_character"]["lod"] != args.lod:
                raise ValueError(f"{item.name}: equipment LOD differs from the assembly")
            counts = count(item.path)
            for category, triangles in counts.items():
                totals[category] += triangles
        imported = (renderer.import_glb(item.path) if item.path.suffix == ".glb"
                    else renderer.import_review(item.path, preset))
        for obj in imported:
            if obj.type != "MESH":
                continue
            role = obj.get("museum_component", obj.data.name.rsplit(".", 1)[0]
                           if obj.data.name[-4:-3] == "." else obj.data.name)
            if item.material:
                renderer.override_material(obj, item.material)
            if role in item.components:
                renderer.override_material(obj, item.components[role])
            obj.name = f"{item.name}/{role}"
        objects.extend(imported)
        sources.append({"name": item.name,
                        "source": item.path.relative_to(args.renderer.resolve().parent.parent).as_posix(),
                        "sha256": hashlib.sha256(item.path.read_bytes()).hexdigest()})
    renderer.apply_pose(objects, preset.pose)
    if body_triangles is None or totals["metal"] > body_triangles:
        raise ValueError(f"metal armor exceeds body budget: {totals}, body={body_triangles}")
    bpy.context.view_layer.update()
    # Bake the saved display pose; discard runtime morphs, skins and hidden rigs.
    # This keeps existing geometry and materials while avoiding duplicate skeletons.
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objects:
        if obj.type == "MESH":
            obj.select_set(True)
            bpy.context.view_layer.objects.active = obj
    bpy.ops.object.convert(target="MESH")
    # The source includes 8K garment atlases. Bound their browser residency while
    # preserving the authored maps and all geometry; metal maps already fit.
    texture_limit = 2048
    for image in bpy.data.images:
        width, height = image.size
        if max(width, height) > texture_limit:
            scale = texture_limit / max(width, height)
            image.scale(round(width * scale), round(height * scale))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(args.output.resolve()), export_format="GLB",
                              use_selection=True, export_animations=False,
                              export_skins=False, export_morph=False, export_yup=True)
    args.output.with_suffix(".sources.json").write_text(json.dumps({
        "source_branch": "codex/museum-armor-sets",
        "preset": args.preset.resolve().relative_to(args.renderer.resolve().parent.parent).as_posix(),
        "pose": preset.pose, "items": sources, "maximum_texture_dimension": texture_limit,
        "lod": args.lod, "triangles": {"body": body_triangles, **totals},
        "scope": "Native LOD museum recipes with dense-source normal bakes; saved display pose."
    }, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
