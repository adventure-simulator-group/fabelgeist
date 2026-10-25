"""Export actual installed GLB identity blends for body-visible static review.

Usage: python scripts/export_armor_morph_review.py ASSET_DIRECTORY OUTPUT_DIRECTORY
Skeletal deformation and animation must be checked in the gameplay renderer.
"""
import argparse
import json
from pathlib import Path

import numpy as np

from check_parametric_armor_assets import EXPECTED_TARGETS, Glb


def geometry(path, weights):
    glb = Glb(path)
    positions, normals, indices, components = [], [], [], []
    for mesh_index, mesh in enumerate(glb.doc["meshes"]):
        node = next(node for node in glb.doc["nodes"] if node.get("mesh") == mesh_index)
        vertex_start, index_start = len(positions), len(indices)
        assert mesh["extras"]["targetNames"] == EXPECTED_TARGETS
        for primitive in mesh["primitives"]:
            fitted = glb.array(primitive["attributes"]["POSITION"]).copy()
            shading = glb.array(primitive["attributes"]["NORMAL"]).copy()
            for weight, target in zip(weights, primitive["targets"][:45]):
                fitted += weight * glb.array(target["POSITION"])
                shading += weight * glb.array(target["NORMAL"])
            offset = len(positions)
            indices.extend((glb.array(primitive["indices"]).reshape(-1) + offset).tolist())
            positions.extend(fitted.tolist())
            shading /= np.linalg.norm(shading, axis=1)[:, None]
            normals.extend(shading.tolist())
        components.append({"role": mesh["name"],
                           "vertices": {"start": vertex_start, "end": len(positions)},
                           "indices": {"start": index_start, "end": len(indices)},
                           "hinge": node.get("extras", {}).get("adventuresim_hinge")})
    return {"positions": positions, "normals": normals, "indices": indices, "components": components}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("assets", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[1]
    body_path = repository / "assets/animations/biped/unarmed/base.glb"
    manifest = json.loads((args.assets / "manifest.json").read_text())
    blends = {
        "neutral": np.zeros(45),
        "positive": np.full(45, .35),
        "negative": np.full(45, -.35),
        "mixed": np.array([.35 if i % 2 else -.35 for i in range(45)]),
    }
    for name, weights in blends.items():
        output = args.output / name
        output.mkdir(parents=True, exist_ok=True)
        body = geometry(body_path, weights)
        body["faces"] = np.array(body.pop("indices")).reshape(-1, 3).tolist()
        body["recipe"] = {"identity": weights.tolist(), "skeletal": "reference bind pose"}
        body["source"] = str(body_path)
        (output / "body.json").write_text(json.dumps(body))
        for row in manifest["assets"]:
            if "armor_generator_version" not in row:
                continue
            mesh = geometry(args.assets / row["file"], weights)
            mesh.update(id=row["item_id"], placement=row["placement_id"],
                        design={"source_asset": row["file"], "design_hash": row["armor_design_hash"]},
                        generator_version=row["armor_generator_version"])
            (output / f"{Path(row['file']).stem}.json").write_text(json.dumps(mesh))
        (output / "provenance.json").write_text(json.dumps({
            "stage": "actual GLB morph interpolation, before skeletal deformation",
            "asset_manifest": str((args.assets / "manifest.json").resolve()),
            "identity_weights": weights.tolist(),
            "limitations": "Static identity blends only; skeletal proportions and animation require runtime capture.",
        }, indent=2))


if __name__ == "__main__":
    main()
