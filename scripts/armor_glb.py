"""Lossless attribute editing for generated, single-buffer equipment GLBs.

Do not round-trip a skinned asset through Blender's glTF exporter: its importer
can reinterpret the rig and morphs. Only geometry accessors are rewritten here.
"""
import copy
import json
import struct
from pathlib import Path

import numpy as np

DTYPES = {5120: "i1", 5121: "u1", 5122: "<i2", 5123: "<u2",
          5125: "<u4", 5126: "<f4"}
WIDTHS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}


def retains_body_uvs(path):
    """Body-derived padding and mail use the anatomical material layout."""
    item = Path(path).stem.split("--")[0]
    return item.startswith("mail_") or item in {"arming_doublet", "padded_chausses"}


class Asset:
    def __init__(self, path):
        data = Path(path).read_bytes()
        magic, version, size = struct.unpack_from("<4sII", data)
        if (magic, version, size) != (b"glTF", 2, len(data)):
            raise ValueError("expected a complete GLB 2.0")
        length, kind = struct.unpack_from("<II", data, 12)
        if kind != 0x4E4F534A:
            raise ValueError("missing JSON chunk")
        self.doc = json.loads(data[20:20 + length])
        length2, kind = struct.unpack_from("<II", data, 20 + length)
        if kind != 0x004E4942 or len(self.doc["buffers"]) != 1:
            raise ValueError("expected one embedded binary buffer")
        self.binary = bytearray(data[28 + length:28 + length + length2])

    def array(self, index):
        accessor = self.doc["accessors"][index]
        if "sparse" in accessor:
            raise ValueError("sparse accessors are not generated equipment")
        view = self.doc["bufferViews"][accessor["bufferView"]]
        dtype, width = np.dtype(DTYPES[accessor["componentType"]]), WIDTHS[accessor["type"]]
        offset = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
        return np.ndarray((accessor["count"], width), dtype=dtype,
                          buffer=self.binary, offset=offset,
                          strides=(view.get("byteStride", dtype.itemsize * width),
                                   dtype.itemsize)).copy()

    def append(self, values, template=None, component=5126, kind=None):
        values = np.asarray(values)
        if values.ndim == 1:
            values = values[:, None]
        accessor = copy.deepcopy(template) if template else {
            "componentType": component,
            "type": kind or {1: "SCALAR", 2: "VEC2", 3: "VEC3", 4: "VEC4"}[values.shape[1]],
        }
        accessor.pop("byteOffset", None)
        accessor["count"] = len(values)
        for key, operation in [("min", np.min), ("max", np.max)]:
            if key in accessor:
                accessor[key] = operation(values, axis=0).tolist()
        self.binary.extend(b"\0" * (-len(self.binary) % 4))
        payload = values.astype(DTYPES[accessor["componentType"]]).tobytes()
        accessor["bufferView"] = len(self.doc["bufferViews"])
        self.doc["bufferViews"].append({"buffer": 0, "byteOffset": len(self.binary),
                                        "byteLength": len(payload)})
        self.binary.extend(payload)
        self.doc["accessors"].append(accessor)
        return len(self.doc["accessors"]) - 1

    def remap(self, primitive, sources, indices):
        """Copy every source attribute and morph exactly to seam-split vertices."""
        for attributes in [primitive["attributes"], *primitive.get("targets", [])]:
            for name, index in list(attributes.items()):
                attributes[name] = self.append(self.array(index)[sources],
                                                self.doc["accessors"][index])
        primitive["indices"] = self.append(indices, component=5125)

    def compact(self):
        references = []
        for mesh in self.doc.get("meshes", []):
            for primitive in mesh["primitives"]:
                references.append((primitive, "indices"))
                for attributes in [primitive["attributes"], *primitive.get("targets", [])]:
                    references.extend((attributes, key) for key in attributes)
        references.extend((skin, "inverseBindMatrices") for skin in self.doc.get("skins", [])
                          if "inverseBindMatrices" in skin)
        for animation in self.doc.get("animations", []):
            for sampler in animation["samplers"]:
                references.extend((sampler, key) for key in ("input", "output"))
        used = sorted({owner[key] for owner, key in references})
        mapping = {old: new for new, old in enumerate(used)}
        for owner, key in references:
            owner[key] = mapping[owner[key]]
        self.doc["accessors"] = [self.doc["accessors"][i] for i in used]
        owners = self.doc["accessors"] + [image for image in self.doc.get("images", [])
                                         if "bufferView" in image]
        used_views = sorted({owner["bufferView"] for owner in owners})
        views, binary, mapping = [], bytearray(), {}
        for old in used_views:
            view = copy.deepcopy(self.doc["bufferViews"][old])
            start = view.get("byteOffset", 0)
            binary.extend(b"\0" * (-len(binary) % 4))
            view["byteOffset"] = len(binary)
            binary.extend(self.binary[start:start + view["byteLength"]])
            mapping[old] = len(views)
            views.append(view)
        for owner in owners:
            owner["bufferView"] = mapping[owner["bufferView"]]
        self.binary, self.doc["bufferViews"] = binary, views

    def write(self, path):
        self.compact()
        self.doc["buffers"][0]["byteLength"] = len(self.binary)
        document = json.dumps(self.doc, separators=(",", ":")).encode()
        document += b" " * (-len(document) % 4)
        binary = bytes(self.binary) + b"\0" * (-len(self.binary) % 4)
        data = struct.pack("<4sII", b"glTF", 2, 28 + len(document) + len(binary))
        data += struct.pack("<II", len(document), 0x4E4F534A) + document
        data += struct.pack("<II", len(binary), 0x004E4942) + binary
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_suffix(".glb.tmp")
        temporary.write_bytes(data)
        temporary.replace(path)
