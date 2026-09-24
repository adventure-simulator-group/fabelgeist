"""Repack static art-demo armor textures and write a reproducible inventory.

Occlusion-only maps use one channel and need less resolution than silhouette and
finish-defining maps. KTX2 is deliberately not emitted: Bevy 0.19's glTF loader
does not enable KHR_texture_basisu in this project's WebGPU build.
"""
import argparse
from collections import Counter
from io import BytesIO
import hashlib
import json
from pathlib import Path
import struct

from PIL import Image


def read_glb(path):
    payload = Path(path).read_bytes()
    magic, version, length = struct.unpack_from("<4sII", payload)
    if (magic, version, length) != (b"glTF", 2, len(payload)):
        raise ValueError("expected a complete GLB 2.0")
    json_length, json_kind = struct.unpack_from("<II", payload, 12)
    document = json.loads(payload[20:20 + json_length])
    binary_offset = 20 + json_length
    binary_length, binary_kind = struct.unpack_from("<II", payload, binary_offset)
    if json_kind != 0x4E4F534A or binary_kind != 0x004E4942:
        raise ValueError("expected JSON and BIN chunks")
    return document, payload[binary_offset + 8:binary_offset + 8 + binary_length]


def write_glb(path, document, binary):
    document["buffers"][0]["byteLength"] = len(binary)
    encoded = json.dumps(document, separators=(",", ":")).encode()
    encoded += b" " * (-len(encoded) % 4)
    binary += b"\0" * (-len(binary) % 4)
    payload = struct.pack("<4sII", b"glTF", 2, 28 + len(encoded) + len(binary))
    payload += struct.pack("<II", len(encoded), 0x4E4F534A) + encoded
    payload += struct.pack("<II", len(binary), 0x004E4942) + binary
    Path(path).write_bytes(payload)


def image_roles(document):
    roles = [set() for _ in document.get("images", [])]
    for material in document.get("materials", []):
        pbr = material.get("pbrMetallicRoughness", {})
        fields = (("base_color", pbr.get("baseColorTexture")),
                  ("metallic_roughness", pbr.get("metallicRoughnessTexture")),
                  ("normal", material.get("normalTexture")),
                  ("occlusion", material.get("occlusionTexture")),
                  ("emissive", material.get("emissiveTexture")))
        for role, reference in fields:
            if reference is not None:
                texture = document["textures"][reference["index"]]
                roles[texture["source"]].add(role)
    return roles


def encode_image(payload, roles):
    image = Image.open(BytesIO(payload))
    source_size = image.size
    # AO is broad self-shadowing rather than finish-defining surface detail.
    limit = 256 if roles == {"occlusion"} else 512
    if max(image.size) > limit:
        scale = limit / max(image.size)
        image = image.resize((round(image.width * scale), round(image.height * scale)),
                             Image.Resampling.LANCZOS)
    if roles == {"occlusion"} and image.mode != "L":
        image = image.getchannel("R")
    output = BytesIO()
    image.save(output, format="PNG", compress_level=9, optimize=False)
    return output.getvalue(), source_size, image.size, len(image.getbands())


def optimize(source, output, report_path):
    document, binary = read_glb(source)
    roles = image_roles(document)
    geometry_views = {accessor["bufferView"] for accessor in document.get("accessors", [])
                      if "bufferView" in accessor}
    geometry_bytes = sum(document["bufferViews"][index]["byteLength"]
                         for index in geometry_views)
    records, packed, unique = [], bytearray(), {}
    old_to_new_view = {}
    image_views = {image["bufferView"] for image in document.get("images", [])}
    retained_views = [index for index in range(len(document["bufferViews"]))
                      if index not in image_views]
    views = []
    for index in retained_views:
        view = dict(document["bufferViews"][index])
        start = view.get("byteOffset", 0)
        packed.extend(b"\0" * (-len(packed) % 4))
        view["byteOffset"] = len(packed)
        packed.extend(binary[start:start + view["byteLength"]])
        old_to_new_view[index] = len(views)
        views.append(view)
    for accessor in document.get("accessors", []):
        if "bufferView" in accessor:
            accessor["bufferView"] = old_to_new_view[accessor["bufferView"]]
    for index, descriptor in enumerate(document.get("images", [])):
        view = document["bufferViews"][descriptor["bufferView"]]
        start = view.get("byteOffset", 0)
        original = binary[start:start + view["byteLength"]]
        encoded, source_size, final_size, channels = encode_image(original, roles[index])
        digest = hashlib.sha256(encoded).hexdigest()
        duplicate_of = unique.get(digest)
        if duplicate_of is None:
            packed.extend(b"\0" * (-len(packed) % 4))
            replacement = {"buffer": 0, "byteOffset": len(packed),
                           "byteLength": len(encoded)}
            descriptor["bufferView"] = len(views)
            views.append(replacement)
            packed.extend(encoded)
            unique[digest] = descriptor["bufferView"]
        else:
            descriptor["bufferView"] = duplicate_of
        descriptor["mimeType"] = "image/png"
        records.append({"index": index, "name": descriptor.get("name", ""),
                        "roles": sorted(roles[index]),
                        "source_dimensions": list(source_size),
                        "dimensions": list(final_size), "channels": channels,
                        "format": "image/png", "source_bytes": len(original),
                        "bytes": len(encoded), "sha256": digest,
                        "duplicate_of": duplicate_of})
    document["bufferViews"] = views
    write_glb(output, document, bytes(packed))
    gpu_before = sum(r["source_dimensions"][0] * r["source_dimensions"][1] * 4
                     for r in records) * 4 // 3
    gpu_after = sum(r["dimensions"][0] * r["dimensions"][1] * r["channels"]
                    for r in records if r["duplicate_of"] is None) * 4 // 3
    report = {
        "tool": "scripts/optimize_art_demo_armor.py", "pillow": Image.__version__,
        "policy": {"occlusion_only": "256px, R8 PNG", "all_other": "512px PNG",
                   "ktx2": "not used; KHR_texture_basisu unavailable in Bevy 0.19 glTF"},
        "geometry_bytes": geometry_bytes, "source_glb_bytes": Path(source).stat().st_size,
        "glb_bytes": Path(output).stat().st_size,
        "source_image_bytes": sum(r["source_bytes"] for r in records),
        "image_bytes": sum(r["bytes"] for r in records if r["duplicate_of"] is None),
        "source_images": len(records), "unique_images": len(unique),
        "duplicate_images": len(records) - len(unique),
        "gpu_bytes_estimate_before": gpu_before, "gpu_bytes_estimate_after": gpu_after,
        "dimensions": dict(Counter(f"{r['dimensions'][0]}x{r['dimensions'][1]}x{r['channels']}"
                                   for r in records)), "images": records,
        "output_sha256": hashlib.sha256(Path(output).read_bytes()).hexdigest(),
    }
    Path(report_path).write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    optimize(args.source, args.output, args.report)
    sources = args.output.with_suffix(".sources.json")
    if sources.exists():
        metadata = json.loads(sources.read_text(encoding="utf-8"))
        metadata["output_sha256"] = hashlib.sha256(args.output.read_bytes()).hexdigest()
        sources.write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
