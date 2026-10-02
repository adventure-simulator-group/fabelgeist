"""Validate finished normal/AO channels and preservation of shape and rig data."""
import argparse
import json
from pathlib import Path

import numpy as np
from PIL import Image
from armor_glb import Asset, retains_body_uvs
from armor_bake_math import raster_triangles, dilate


def audit(source, current):
    before, after = Asset(source), Asset(current)
    result = []
    for key in ("nodes", "scenes", "extras"):
        assert before.doc.get(key) == after.doc.get(key), f"changed {key}"
    assert len(before.doc["meshes"]) == len(after.doc["meshes"])
    for a, b in zip(before.doc["meshes"], after.doc["meshes"]):
        assert a.get("extras") == b.get("extras") and a.get("weights") == b.get("weights")
        for old, new in zip(a["primitives"], b["primitives"]):
            old_corners = before.array(old["indices"]).flatten()
            new_corners = after.array(new["indices"]).flatten()
            assert len(old_corners) == len(new_corners), "triangle count changed"
            for name, index in old["attributes"].items():
                if name not in {"NORMAL", "TANGENT"}:
                    np.testing.assert_array_equal(before.array(index)[old_corners],
                                                  after.array(new["attributes"][name])[new_corners], err_msg=name)
            assert len(old.get("targets", [])) == len(new.get("targets", []))
            for a_target, b_target in zip(old.get("targets", []), new.get("targets", [])):
                np.testing.assert_array_equal(before.array(a_target["POSITION"])[old_corners],
                                              after.array(b_target["POSITION"])[new_corners])
            if retains_body_uvs(current):
                continue
            normal = after.array(new["attributes"]["NORMAL"])
            tangent = after.array(new["attributes"]["TANGENT"])
            assert np.isfinite(normal).all() and np.isfinite(tangent).all()
            assert np.max(np.abs(np.linalg.norm(normal, axis=1) - 1)) < 1e-4
            assert np.max(np.abs((normal * tangent[:, :3]).sum(axis=1))) < 1e-4
            for target in new.get("targets", []):
                endpoint = normal + after.array(target["NORMAL"])
                assert np.max(np.abs(np.linalg.norm(endpoint, axis=1) - 1)) < 1e-4, "invalid morph shading normal"
            old_material = before.doc["materials"][old["material"]]
            material = after.doc["materials"][new["material"]]
            assert old_material.get("pbrMetallicRoughness") == material.get("pbrMetallicRoughness"), "unlit color changed"
            assert "adventuresim_surface_bake" in new.get("extras", {}), "missing bake"
            files = []
            for channel in ("normalTexture", "occlusionTexture"):
                info = material[channel]
                assert info["texCoord"] == 0, "wrong material UV channel"
                texture = after.doc["textures"][info["index"]]
                path = current.parent / after.doc["images"][texture["source"]]["uri"]
                files.append(path)
                image = np.asarray(Image.open(path).convert("RGB"))
                assert image.max() > 0, "empty baked map"
                if channel == "occlusionTexture":
                    covered = np.zeros(image.shape[:2], dtype=bool)
                    uv = after.array(new["attributes"]["TEXCOORD_0"])
                    faces = after.array(new["indices"]).reshape(-1, 3)
                    for _, y, x, _ in raster_triangles(uv, faces, image.shape[0]):
                        covered[y, x] = True
                    dilate(np.zeros_like(image), covered)
                    assert np.all(image[~covered] == 255), "unused AO atlas space causes dark mip bleeding"
                    assert np.all(image[:, :, 0] == image[:, :, 1]) and np.all(image[:, :, 1] == image[:, :, 2]), "AO must be grayscale"
                else:
                    normal = image.astype(float) / 127.5 - 1
                    error = np.abs(np.linalg.norm(normal, axis=2) - 1)
                    assert np.quantile(error, .999) < .03, "invalid tangent normals"
            assert files[0] != files[1], "normal and AO channels share an image"
            result.append({"mesh": b["name"], "normal": files[0].name, "ao": files[1].name})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("finished", type=Path)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    results = {}
    for path in sorted(args.finished.glob("*.glb")):
        results[path.name] = audit(args.source / path.name, path)
        print(path.name, "PASS", flush=True)
    assert results, "no equipment assets"
    if args.report:
        args.report.write_text(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
