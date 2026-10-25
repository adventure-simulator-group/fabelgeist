"""Count exported metal, clothing and fasteners against the selected body LOD."""
import argparse
import json
from pathlib import Path

from armor_glb import Asset, retains_body_uvs


def count(path):
    asset = Asset(path)
    totals = {"metal": 0, "clothing": 0, "fasteners": 0}
    for mesh in asset.doc["meshes"]:
        for primitive in mesh["primitives"]:
            count = asset.doc["accessors"][primitive["indices"]]["count"]
            if count % 3:
                raise ValueError(f"{path}: incomplete triangle")
            material = asset.doc["materials"][primitive["material"]]
            metallic = material.get("pbrMetallicRoughness", {}).get("metallicFactor", 0)
            if mesh["name"] in {"leather_straps", "buckles"}:
                category = "fasteners"
            elif retains_body_uvs(path) or metallic < .8:
                category = "clothing"
            else:
                category = "metal"
            totals[category] += count // 3
    return totals


def audit(directory, body_path):
    body = Asset(body_path)
    lod = body.doc["extras"]["adventuresim_character"]["lod"]
    if lod not in (4, 5, 6):
        raise ValueError(f"unsupported body LOD {lod}")
    body_triangles = sum(body.doc["accessors"][p["indices"]]["count"] // 3
                         for m in body.doc["meshes"] for p in m["primitives"])
    items = {p.name: count(p) for p in sorted(directory.glob("*.glb"))}
    if not items:
        raise ValueError("no equipment GLBs")
    totals = {key: sum(item[key] for item in items.values())
              for key in ("metal", "clothing", "fasteners")}
    return {"lod": lod, "body_triangles": body_triangles,
            "within_metal_budget": totals["metal"] <= body_triangles,
            "triangles": totals, "items": items}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--body", type=Path, required=True)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    result = audit(args.directory, args.body)
    output = json.dumps(result, indent=2) + "\n"
    if args.report:
        args.report.write_text(output, encoding="utf-8")
    print(json.dumps({key: value for key, value in result.items() if key != "items"}))
    if not result["within_metal_budget"]:
        raise SystemExit("metal armor exceeds the body's triangle count")


if __name__ == "__main__":
    main()
