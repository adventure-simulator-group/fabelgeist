"""Audit actual exported armor GLBs, including morph correspondence and blends.

Requires NumPy. Usage: python scripts/check_parametric_armor_assets.py DIRECTORY
Use --allow-partial for a deliberately filtered pilot export.
"""
import argparse
import json
import struct
from collections import Counter
from pathlib import Path

import numpy as np

EXPECTED_TARGETS = [f"mhr_identity_{i:02}" for i in range(45)] + [
    "mhr_skeletal_spine_short", "mhr_skeletal_spine_long"]
MINIMUM_TRIANGLE_AREA_M2 = 1e-12


class Glb:
    def __init__(self, path):
        data = path.read_bytes()
        magic, version, length = struct.unpack_from("<III", data)
        assert (magic, version, length) == (0x46546C67, 2, len(data)), path
        size, kind = struct.unpack_from("<II", data, 12)
        assert kind == 0x4E4F534A
        self.doc = json.loads(data[20:20 + size])
        binary_size, kind = struct.unpack_from("<II", data, 20 + size)
        assert kind == 0x004E4942
        self.binary = data[28 + size:28 + size + binary_size]

    def array(self, index):
        accessor = self.doc["accessors"][index]
        view = self.doc["bufferViews"][accessor["bufferView"]]
        dtype = {5121: "u1", 5123: "<u2", 5125: "<u4", 5126: "<f4"}[accessor["componentType"]]
        width = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}[accessor["type"]]
        offset = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
        stride = view.get("byteStride", np.dtype(dtype).itemsize * width)
        result = np.ndarray((accessor["count"], width), dtype=dtype,
                            buffer=self.binary, offset=offset, strides=(stride, np.dtype(dtype).itemsize))
        assert np.isfinite(result).all(), "nonfinite exported accessor"
        return result.astype(np.float64 if dtype.endswith("f4") else np.int64)


def area(positions, faces):
    a, b, c = (positions[faces[:, i]] for i in range(3))
    return np.linalg.norm(np.cross(b - a, c - a), axis=1).min() * 0.5


def audit(path):
    glb = Glb(path)
    results = []
    if path.stem == "close_helmet--worn":
        audit_close_helmet_assembly(glb)
    for mesh in glb.doc["meshes"]:
        assert mesh["extras"]["targetNames"] == EXPECTED_TARGETS, "morph names/order"
        assert all(weight == 0 for weight in mesh["weights"]), "nonzero exported baseline"
        for primitive in mesh["primitives"]:
            attributes = primitive["attributes"]
            positions = glb.array(attributes["POSITION"])
            faces = glb.array(primitive["indices"]).reshape(-1, 3)
            assert faces.min() >= 0 and faces.max() < len(positions), "invalid indices"
            # Hard shading/UV seams may duplicate coincident vertices. Audit the
            # physical surface after welding to one micrometre, and below verify
            # those seams remain coincident at each exported morph endpoint.
            _, representatives, welded = np.unique(np.round(positions, 6), axis=0,
                                                   return_index=True, return_inverse=True)
            physical_faces = welded[faces]
            edges = Counter((int(a), int(b)) for face in physical_faces for a, b in zip(face, np.roll(face, -1)))
            assert all(count == 1 and edges[(b, a)] == 1 for (a, b), count in edges.items()), "unclosed or inconsistently wound physical edges"
            assert glb.array(attributes["NORMAL"]).shape == positions.shape
            weights = glb.array(attributes["WEIGHTS_0"]).sum(axis=1) + glb.array(attributes["WEIGHTS_1"]).sum(axis=1)
            assert np.max(np.abs(weights - 1)) < 1e-4, "unnormalized skin weights"
            joint_count = len(glb.doc["skins"][0]["joints"])
            assert max(glb.array(attributes[name]).max() for name in ("JOINTS_0", "JOINTS_1")) < joint_count
            targets = [glb.array(target["POSITION"]) for target in primitive["targets"]]
            assert len(targets) == 47 and all(target.shape == positions.shape for target in targets)
            for delta in targets:
                assert np.max(np.abs(delta - delta[representatives][welded])) < 2e-6, "morph tears a welded seam"
            for target in primitive["targets"]:
                assert glb.array(target["NORMAL"]).shape == positions.shape
            samples = [("zero", positions)] + [(name, positions + delta) for name, delta in zip(EXPECTED_TARGETS, targets)]
            # Representative bounded cosmetic extremes and a mixed sign blend.
            identity = np.stack(targets[:45])
            for name, weights in [("all_positive_035", np.full(45, .35)),
                                  ("all_negative_035", np.full(45, -.35)),
                                  ("alternating_035", np.array([.35 if i % 2 else -.35 for i in range(45)]))]:
                samples.append((name, positions + np.einsum("i,ijk->jk", weights, identity)))
            areas = {name: float(area(sample, faces)) for name, sample in samples}
            assert min(areas.values()) > MINIMUM_TRIANGLE_AREA_M2, (path, "degenerate morph triangle", areas)
            results.append({"vertices": len(positions), "triangles": len(faces), "minimum_areas_m2": areas})
    return results


def audit_close_helmet_assembly(glb):
    expected = ["skull", "bevor", "visor"]
    assert [mesh["name"] for mesh in glb.doc["meshes"]] == expected, "helmet component meshes"
    nodes = [node for node in glb.doc["nodes"] if "mesh" in node]
    assert [node["name"] for node in nodes] == expected, "helmet component nodes"
    for index, node in enumerate(nodes):
        assert node["mesh"] == index and node["skin"] == 0, "component mesh/skin binding"
        assert not any(key in node for key in ("matrix", "translation", "rotation", "scale")), "reference-body component transform"
        hinge = node.get("extras", {}).get("adventuresim_hinge")
        if node["name"] == "skull":
            assert hinge is None, "fixed skull"
        else:
            assert hinge["space"] == "reference_body"
            assert np.isfinite(hinge["origin"]).all() and len(hinge["origin"]) == 3
            assert abs(np.linalg.norm(hinge["axis"]) - 1) < 1e-6, "unit hinge axis"
        mesh = glb.doc["meshes"][index]
        assert len(mesh["primitives"]) == 1, "independent component primitive"
        attributes = mesh["primitives"][0]["attributes"]
        joints = np.concatenate([glb.array(attributes[key]) for key in ("JOINTS_0", "JOINTS_1")], axis=1)
        weights = np.concatenate([glb.array(attributes[key]) for key in ("WEIGHTS_0", "WEIGHTS_1")], axis=1)
        skin = glb.doc["skins"][node["skin"]]
        head = next(i for i, joint in enumerate(skin["joints"]) if glb.doc["nodes"][joint]["name"] == "c_head")
        assert np.all(joints[weights > 0] == head), "rigid helmet must follow head, not jaw"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--allow-partial", action="store_true")
    args = parser.parse_args()
    manifest = json.loads((args.directory / "manifest.json").read_text())
    armor = [row for row in manifest["assets"] if "armor_generator_version" in row]
    if not args.allow_partial:
        catalog = json.loads((Path(__file__).resolve().parents[1] / "content/items/catalog.yaml").read_text())
        expected = {item["id"] for item in catalog["items"] if item["kind"] == "armor"}
        assert expected <= {row["item_id"] for row in armor}, "catalog armor missing from parametric manifest"
    report = {"stage": "actual exported GLBs", "triangle_area_threshold_m2": MINIMUM_TRIANGLE_AREA_M2,
              "limitations": "Fixed topology, finite attributes and sampled morph triangle areas; does not prove all-body clearance or animation fit.", "assets": []}
    for row in armor:
        assert isinstance(row["armor_generator_version"], int) and row["armor_generator_version"] > 0
        assert len(row["armor_design_hash"]) == 64 and row["morph_targets"] == 47
        result = audit(args.directory / row["file"])
        report["assets"].append({"file": row["file"], "primitives": result})
        print(f"PASS {row['file']}", flush=True)
    (args.directory / "parametric-audit.json").write_text(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
