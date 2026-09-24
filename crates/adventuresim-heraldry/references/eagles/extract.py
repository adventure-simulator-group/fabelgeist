"""Extract the credited eagle charges; preserve source contours and transforms.

Run from any directory. Hashes pin the reviewed originals. Only the shield is
removed and tongue paths named. The double's mirrored wing is expanded so
SVG readers cannot discard its tongue IDs. No new geometry is authored.
"""

import hashlib
from copy import deepcopy
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parent
SVG = "http://www.w3.org/2000/svg"
ET.register_namespace("", SVG)
ET.register_namespace("xlink", "http://www.w3.org/1999/xlink")

SOURCES = {
    "single": (
        "c277aa3b748291a6680889e4f7a20ff1f183ea28ebce346e858ccbdd5964c2b2",
        "Arms_of_the_King_of_the_Romans_(c.1433-1486).svg",
    ),
    "double": (
        "644e05e78ed0f2af886ea6c6254b53ce64822f894fa02e30c3874ed887402ef0",
        "Arms_of_the_Holy_Roman_Emperor_(c.1433-c.1450).svg",
    ),
}

for name, (digest, title) in SOURCES.items():
    data = (ROOT / f"{name}.original.svg").read_bytes()
    assert hashlib.sha256(data).hexdigest() == digest, "Unreviewed source"
    root = ET.fromstring(data)
    if name == "single":
        shield = root[0]
        assert shield.tag == f"{{{SVG}}}path"
        assert shield.get("fill") == "#ffd833"
        root.remove(shield)
        # The optimized single source has no head-path IDs. These are the
        # reviewed tongue's base, highlight and outline, in source coordinates.
        starts = ("m263.87 79.26", "m263.48 77.455", "m263.55 78.917")
        tongues = [
            p for p in root.iter(f"{{{SVG}}}path")
            if p.get("d", "").startswith(starts)
        ]
    else:
        shield = next(p for p in root.iter() if p.get("id") == "escutcheon")
        parent = next(p for p in root.iter() if shield in list(p))
        parent.remove(shield)
        tongues = [p for p in root.iter() if p.get("id", "").startswith("path4162-")]
    assert len(tongues) == 3
    for i, path in enumerate(tongues):
        path.set("id", f"tongue-{i}")
    if name == "double":
        # usvg strips duplicated IDs when resolving <use>. Expand this one
        # instance with unique IDs so each head retains its tongue semantics.
        prototype = next(p for p in root.iter() if p.get("id") == "eagle_dexter-4")
        instance = next(p for p in root.iter() if p.get("id") == "eagle_sinister-3")
        clone = deepcopy(prototype)
        ids = {
            p.get("id"): (
                "tongue-mirrored-" + p.get("id").removeprefix("tongue-")
                if p.get("id", "").startswith("tongue-")
                else "mirrored-" + p.get("id")
            )
            for p in clone.iter() if p.get("id")
        }
        href = "{http://www.w3.org/1999/xlink}href"
        for path in clone.iter():
            if path.get("id") in ids:
                path.set("id", ids[path.get("id")])
            target = path.get(href, "").removeprefix("#")
            if target in ids:
                path.set(href, "#" + ids[target])
        group = ET.Element(f"{{{SVG}}}g", {"transform": instance.get("transform")})
        group.append(clone)
        parent = next(p for p in root.iter() if instance in list(p))
        parent.insert(list(parent).index(instance), group)
        parent.remove(instance)
    ET.SubElement(root, f"{{{SVG}}}metadata").text = (
        f"Adapted from {title} by Tom Lemmens (Tom-L) and Heralder, "
        f"https://commons.wikimedia.org/wiki/File:{title} . "
        "CC BY-SA 3.0, https://creativecommons.org/licenses/by-sa/3.0/ . "
        "Fabelgeist contributors removed the shield, identified tongue paths "
        "and expanded the double eagle's mirrored wing with unique IDs. "
        "The double eagle retains the modern source's halos. "
        "Historical provenance and modification details: ../EAGLES.md."
    )
    ET.ElementTree(root).write(ROOT / f"{name}.svg", encoding="utf-8", xml_declaration=True)
