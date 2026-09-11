"""Check material atlases and exact preservation of indexed source attributes.

python scripts/check_armor_uvs.py ORIGINAL_DIRECTORY UNWRAPPED_DIRECTORY
"""
import argparse
import json
from pathlib import Path

import numpy as np
from armor_glb import Asset, retains_body_uvs


def overlap_pairs(triangles):
    """Strict separating-axis test; shared chart edges are allowed."""
    triangles = np.asarray(triangles, dtype=np.float64)
    lower, upper = triangles.min(axis=1), triangles.max(axis=1)
    bins, candidates = {}, set()
    for index, (a, b) in enumerate(zip(lower, upper)):
        for x in range(int(a[0] * 64), int(b[0] * 64) + 1):
            for y in range(int(a[1] * 64), int(b[1] * 64) + 1):
                for other in bins.setdefault((x, y), []):
                    candidates.add((other, index))
                bins[x, y].append(index)
    result = []
    pairs = np.array(list(candidates), dtype=int).reshape(-1, 2)
    for start in range(0, len(pairs), 8192):
        batch = pairs[start:start + 8192]
        a, b = batch.T
        keep = np.all(np.minimum(upper[a], upper[b]) - np.maximum(lower[a], lower[b]) > 1e-9, axis=1)
        batch = batch[keep]
        if not len(batch):
            continue
        ta, tb = triangles[batch[:, 0]], triangles[batch[:, 1]]
        edges = np.concatenate((np.roll(ta, 1, axis=1) - ta,
                                np.roll(tb, 1, axis=1) - tb), axis=1)
        axes = edges[:, :, ::-1] * [1, -1]
        axes /= np.maximum(np.linalg.norm(axes, axis=2)[:, :, None], 1e-30)
        ap, bp = np.einsum("bvi,bai->bva", ta, axes), np.einsum("bvi,bai->bva", tb, axes)
        inside = np.all(np.minimum(ap.max(axis=1), bp.max(axis=1)) -
                        np.maximum(ap.min(axis=1), bp.min(axis=1)) > 1e-9, axis=1)
        result.extend(map(tuple, batch[inside].tolist()))
    return result


def overlaps(triangles):
    return len(overlap_pairs(triangles))


def audit(original, current):
    source, asset = Asset(original), Asset(current)
    assert source.doc.get("materials") == asset.doc.get("materials"), "materials changed"
    assert len(source.doc.get("skins", [])) == len(asset.doc.get("skins", []))
    for key in ("nodes", "skins", "scenes", "animations", "extras"):
        if key == "skins":
            for a, b in zip(source.doc.get(key, []), asset.doc.get(key, [])):
                assert {k: v for k, v in a.items() if k != "inverseBindMatrices"} == {
                    k: v for k, v in b.items() if k != "inverseBindMatrices"}, "skin changed"
                np.testing.assert_array_equal(source.array(a["inverseBindMatrices"]),
                                              asset.array(b["inverseBindMatrices"]))
        elif key != "animations":
            assert source.doc.get(key) == asset.doc.get(key), f"changed {key}"
    assert len(source.doc["meshes"]) == len(asset.doc["meshes"])
    result = []
    for old_mesh, mesh in zip(source.doc["meshes"], asset.doc["meshes"]):
        for key in ("name", "weights"):
            assert old_mesh.get(key) == mesh.get(key), f"changed mesh {key}"
        assert old_mesh.get("extras", {}).get("targetNames") == mesh.get("extras", {}).get("targetNames")
        assert len(old_mesh["primitives"]) == len(mesh["primitives"])
        for old, new in zip(old_mesh["primitives"], mesh["primitives"]):
            material = source.doc["materials"][old["material"]]
            retained = retains_body_uvs(original) or "baseColorTexture" in material.get("pbrMetallicRoughness", {})
            a = source.array(old["indices"]).flatten()
            b = asset.array(new["indices"]).flatten()
            assert len(a) == len(b), "triangle count changed"
            assert len(old.get("targets", [])) == len(new.get("targets", []))
            for before, after in zip([old["attributes"], *old.get("targets", [])],
                                     [new["attributes"], *new.get("targets", [])]):
                for name, index in before.items():
                    if name == "TANGENT":
                        continue  # Tangent frame now belongs to material UV0.
                    output_name = "TEXCOORD_1" if name == "TEXCOORD_0" and not retained else name
                    np.testing.assert_array_equal(source.array(index)[a],
                                                  asset.array(after[output_name])[b], err_msg=name)
            if retained:
                continue
            assert new.get("extras", {}).get("adventuresim_material_uv", {}).get("channel") == 0, "missing material atlas"
            assert {"TEXCOORD_0", "TANGENT"} <= new["attributes"].keys(), "missing material attributes"
            if "TEXCOORD_0" in old["attributes"]:
                assert mesh["extras"]["adventuresim_anatomical_uv"]["channel"] == 1
            uv = asset.array(new["attributes"]["TEXCOORD_0"])
            assert np.isfinite(uv).all() and uv.min() >= 0 and uv.max() <= 1
            triangles = uv[b.reshape(-1, 3)]
            e, f = triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0]
            area = np.abs(e[:, 0] * f[:, 1] - e[:, 1] * f[:, 0]) / 2
            assert area.min() > 1e-14, "zero-area material UV"
            collisions = overlaps(triangles)
            assert collisions == 0, f"{mesh['name']}: {collisions} overlapping UV triangles"
            tangent = asset.array(new["attributes"]["TANGENT"])
            normal = asset.array(new["attributes"]["NORMAL"])
            assert np.isfinite(tangent).all()
            assert np.isin(tangent[:, 3], [-1, 1]).all(), "invalid handedness"
            assert np.max(np.abs(np.linalg.norm(tangent[:, :3], axis=1) - 1)) < 1e-4
            assert np.max(np.abs((normal * tangent[:, :3]).sum(axis=1))) < 1e-4
            result.append({"mesh": mesh["name"], "triangles": len(triangles),
                           "uv_area": float(area.sum()), "overlaps": collisions})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("original", type=Path)
    parser.add_argument("current", type=Path)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    result = {}
    for path in sorted(args.current.glob("*.glb")):
        result[path.name] = audit(args.original / path.name, path)
        print(path.name, result[path.name], flush=True)
    assert result, "no equipment assets"
    if args.report:
        args.report.write_text(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
