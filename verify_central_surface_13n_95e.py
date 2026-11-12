#!/usr/bin/env python3
"""Verify bounded Surface evidence, not source semantics or product behavior."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
stage = root / "delegated/central-13n-surface"
sha = lambda value: hashlib.sha256(value).hexdigest()
oid = lambda kind, value: hashlib.sha1(kind + b" " + str(len(value)).encode() + b"\0" + value).hexdigest()
reg_bytes = (stage / "REGISTRATION.json").read_bytes()
display = (stage / "DISPLAY.txt").read_bytes()
registration = json.loads(reg_bytes)
receipt = json.loads((stage / "READ-RECEIPT.json").read_bytes())
assert registration["reader"] == receipt["reader"] == "/root"
assert receipt["registration_sha256"] == sha(reg_bytes) and receipt["display_sha256"] == sha(display)
assert len(registration["entries"]) == len(receipt["rows"]) == 4
revision = "95e63628f9d70b8dd6e6fede64334696c6995009"

def tree_entry(data, target):
    cursor = 0
    found = []
    while cursor < len(data):
        end = data.index(b"\0", cursor)
        mode, name = data[cursor:end].split(b" ", 1)
        raw = data[end + 1:end + 21]
        assert len(raw) == 20
        if name.decode() == target:
            found.append((mode.decode().lstrip("0"), raw.hex()))
        cursor = end + 21
    assert len(found) == 1
    return found[0]

expected = []
with tarfile.open(root / "delegated/followup-42/archives/hair-surface-grid.tar.gz", "r:gz") as packet:
    proofs = json.load(packet.extractfile("bundle/git-proofs.json"))
    commit = gzip.decompress(packet.extractfile(f"proof-metadata/git-objects/{revision}.commit.gz").read())
    assert oid(b"commit", commit) == revision
    root_tree = commit.splitlines()[0].removeprefix(b"tree ").decode()
    for entry, row in zip(registration["entries"], receipt["rows"], strict=True):
        assert entry["revision"] == row["revision"] == revision
        assert (entry["tag"], entry["git_path"], entry["registered_ranges"]) == (
            row["tag"], row["git_path"], row["displayed_ranges"]
        )
        source = (stage / entry["capture"]).read_bytes()
        assert (len(source), sha(source), oid(b"blob", source)) == (
            entry["bytes"], entry["sha256"], entry["git_blob"]
        )
        matches = [p for p in proofs["proofs"] if p["revision"] == revision
                   and p["path"] == entry["git_path"] and p["blob"] == entry["git_blob"]]
        assert len(matches) == 1
        proof = matches[0]
        assert packet.extractfile("bundle/" + proof["local"]).read() == source
        current = root_tree
        for part, hop in zip(entry["git_path"].split("/"), proof["chain"], strict=True):
            assert (hop["tree"], hop["name"]) == (current, part)
            meta = proofs["objects"][current]
            data = gzip.decompress(packet.extractfile("bundle/" + meta["file"]).read())
            assert (len(data), sha(data), oid(b"tree", data)) == (meta["bytes"], meta["sha256"], current)
            mode, current = tree_entry(data, part)
            assert (mode, current) == (hop["mode"].lstrip("0"), hop["object"])
        assert current == entry["git_blob"]
        lines = source.decode().splitlines()
        assert len(lines) == entry["eof"]
        read = set()
        for first, last in entry["registered_ranges"]:
            expected.append(f"--- {entry['tag']} {first}-{last} ---\n")
            expected.extend(f"{n:4d} {lines[n-1]}\n" for n in range(first, last + 1))
            read.update(range(first, last + 1))
        unread = {n for first, last in row["unread_complement"] for n in range(first, last + 1)}
        assert unread == set(range(1, len(lines) + 1)) - read
assert "".join(expected).encode() == display
assessment = (root / "ASSESSMENT-CENTRAL-SURFACE-13N-95E.json").read_bytes()
assert assessment == (stage / "ASSESSMENT-DRAFT.json").read_bytes()
assert sha(assessment) == "b3523418aa7a6edd2ab706a95a63bf02535f13bfded6615a0a6115dce0a35eeb"
ledger = json.loads((root / "BASELINE-95E-20261110.json").read_bytes())["newly_assessed_ledger"]["entries"]
found = [e for e in ledger if e["id"] == "REVIEW-HAIR-SURFACE-ZERO-AREA-SHARE-95E"]
assert len(found) == 1 and found[0]["record"] == "ASSESSMENT-CENTRAL-SURFACE-13N-95E.json"
assert found[0]["repair"] == "not started" and found[0]["validation"] == "not performed"
q = json.loads((root / "RECONCILIATION-059.json").read_bytes())["entries"][619]
assert q["contract_id"] == "Q-L11-dedc11f7b4dab040148a" and q["current_reconciliation"].startswith("partial:")
progress = json.loads((root / "CONTINUATION-PROGRESS-95E.json").read_bytes())
assert progress["new_current_ledger_candidates"] == len(ledger)
assert progress["lenses_complete"] == progress["queue_records_semantically_reconciled"] == 0
print("Verified four commit-anchored source chains, registered display, receipt, conditional ledger link and partial q619.")
print("Reconstructed text does not prove a personal display, source semantics or product behavior.")
