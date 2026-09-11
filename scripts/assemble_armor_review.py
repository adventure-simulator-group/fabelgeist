"""Combine actual generated parts into body-visible harness review candidates.

python scripts/assemble_armor_review.py CANDIDATE_DIRECTORY
No geometry is refitted, hidden, or removed for these assembled views.
"""
import json
import shutil
import sys
from pathlib import Path

source = Path(sys.argv[1]).resolve()
output = source / "assemblies"
output.mkdir(exist_ok=True)
shutil.copyfile(source / "body.json", output / "body.json")


def paired(*names):
    return [f"{name}--{side}" for name in names for side in ("left", "right")]


assemblies = {
    "plate_harness": ["morion--worn", "gorget--worn", "cuirass--worn", "fauld--worn"]
    + paired("spaulder", "rerebrace", "couter", "vambrace", "mitten_gauntlet", "cuisse", "poleyn", "greave", "sabaton"),
    "mail_harness": ["mail_coif--worn", "mail_shirt--worn", "mail_skirt--worn"]
    + paired("mail_sleeve", "mail_chausses", "leather_boot"),
    "padded_harness": ["arming_cap--worn", "arming_doublet--worn"]
    + paired("padded_chausses", "leather_boot"),
    "underlayers": ["arming_doublet--worn", "mail_voiders--worn", "mail_brayette--worn"]
    + paired("padded_chausses", "mail_knee_voider"),
}
assemblies["plate_underlayers"] = assemblies["plate_harness"] + assemblies["underlayers"]
assemblies["underlayers"] = assemblies["underlayers"] + ["mail_standard--worn"]
for name, parts in assemblies.items():
    positions, normals, indices, provenance = [], [], [], []
    for part in parts:
        path = source / f"{part}.json"
        mesh = json.loads(path.read_text())
        indices.extend(index + len(positions) for index in mesh["indices"])
        positions.extend(mesh["positions"])
        normals.extend(mesh["normals"])
        provenance.append({"file": str(path), "design": mesh["design"]})
    (output / f"{name}--worn.json").write_text(json.dumps({
        "id": name, "placement": "worn", "positions": positions, "normals": normals, "indices": indices,
        "parts": provenance, "stage": "unmodified ordinary generator outputs assembled for review"
    }))
