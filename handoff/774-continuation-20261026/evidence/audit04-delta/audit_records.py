"""Ignored stdlib bookkeeping and bounded frozen-source reader; no project checks."""
import json
import sys
from pathlib import Path

root = Path(__file__).parent
sources = json.loads((root / "resolved-source-map.json").read_text())
command = sys.argv[1]
if command == "read":
    low, high = map(int, sys.argv[2:4])
    for source in sources:
        if not low <= source["index"] <= high:
            continue
        lines = Path(source["local_object"]).read_text().splitlines()
        print("MATERIAL", source["index"], source["source_id"])
        for first, last in source["ranges"]:
            for index in range(first - 1, min(last, len(lines))):
                print(f"{index + 1}: {lines[index]}")
elif command == "record":
    number, low, high = map(int, sys.argv[2:5])
    stem = f"{number:03}"
    dispositions = json.loads((root / f"dispositions-{stem}.json").read_text())
    selected = [source for source in sources if low <= source["index"] <= high]
    packet_path = root / f"contract-groups-{stem}.json"
    assert not packet_path.exists()
    prior = [json.loads(line) for line in (root / "coverage.jsonl").read_text().splitlines()]
    prior_indices = {row["index"] for row in prior}
    assert not prior_indices.intersection(range(low, high + 1))
    packet = {
        "id": f"ADLER-CONT04-MATERIAL-{stem}",
        "lens": "04 invariant operations",
        "sources": [dict(source, disposition=dispositions[str(source["index"])],
                         method="FRESH_PERSONAL_EXACT_REGISTERED_RANGE_OPERATION_COMPARISON")
                    for source in selected],
        "new_unique_findings_claimed": 0,
        "execution": "No project execution, source edits or Git changes",
    }
    notes = root / f"notes-{stem}.json"
    if notes.exists():
        packet.update(json.loads(notes.read_text()))
    packet_path.write_text(json.dumps(packet, indent=2) + "\n")
    with (root / "coverage.jsonl").open("a") as stream:
        for source in selected:
            row = {key: source[key] for key in ["index", "source_id", "sha256", "ranges"]}
            row.update(packet=packet_path.name,
                       method="FRESH_PERSONAL_EXACT_REGISTERED_RANGE_OPERATION_COMPARISON",
                       disposition=dispositions[str(source["index"])])
            stream.write(json.dumps(row) + "\n")
    complete = prior_indices.union(range(low, high + 1))
    remaining = [index for index in range(104, 174) if index not in complete]
    progress = {
        "lens": "04",
        "inherited_closure": "SEGOVB-INITIAL04-CONTRACT-678",
        "material_completed": sorted(complete),
        "next_unfinished_material": remaining[0] if remaining else None,
        "initial_delta_complete": not remaining,
        "final_clearance": False,
        "latest_packet": packet_path.name,
    }
    (root / f"progress-{stem}.json").write_text(json.dumps(progress, indent=2) + "\n")
    print("Recorded", len(selected), "entries; completed", len(complete), "of 70; next",
          progress["next_unfinished_material"])
