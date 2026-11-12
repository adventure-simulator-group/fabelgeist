#!/usr/bin/env python3
"""Verify bounded collider evidence, not product execution or source semantics."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
stage = root / "delegated/central-13m-collider"
sha256 = lambda value: hashlib.sha256(value).hexdigest()
gitid = lambda kind, value: hashlib.sha1(kind + b" " + str(len(value)).encode() + b"\0" + value).hexdigest()
registration_bytes = (stage / "REGISTRATION.json").read_bytes()
display_bytes = (stage / "DISPLAY.txt").read_bytes()
registration = json.loads(registration_bytes)
receipt = json.loads((stage / "READ-RECEIPT.json").read_bytes())
assert receipt["reader"] == registration["reader"] == "/root"
assert receipt["registration_sha256"] == sha256(registration_bytes)
assert receipt["display_sha256"] == sha256(display_bytes)
rev = "95e63628f9d70b8dd6e6fede64334696c6995009"
assert len(registration["entries"]) == len(receipt["rows"]) == 4

# The older checkpoint19 archive supplies a raw commit anchor. No author
# metadata is displayed; the commit-to-tree relationship is checked in memory.
with tarfile.open(root / "delegated/hair-collider-evidence.tar.gz", "r:gz") as old:
    commit = gzip.decompress(old.extractfile(f"git-objects/{rev}.commit.gz").read())
assert gitid(b"commit", commit) == rev
root_tree = commit.splitlines()[0].removeprefix(b"tree ").decode()

def find_tree_entry(data, target):
    cursor = 0
    found = []
    while cursor < len(data):
        end = data.index(b"\0", cursor)
        mode, name = data[cursor:end].split(b" ", 1)
        oid = data[end + 1:end + 21].hex()
        assert len(data[end + 1:end + 21]) == 20
        if name.decode() == target:
            found.append((mode.decode(), oid))
        cursor = end + 21
    assert len(found) == 1
    return found[0]

expected_display = []
with tarfile.open(root / "delegated/followup-42/archives/hair-collider.tar.gz", "r:gz") as packet:
    proofs = json.load(packet.extractfile("evidence/TREE-PROOFS.json"))
    assert proofs["roots"][rev] == root_tree
    for entry, row in zip(registration["entries"], receipt["rows"], strict=True):
        assert entry["revision"] == row["revision"] == rev
        assert (entry["tag"], entry["git_path"], entry["registered_ranges"]) == (
            row["tag"], row["git_path"], row["displayed_ranges"]
        )
        source = (stage / entry["capture"]).read_bytes()
        assert sha256(source) == entry["sha256"] and len(source) == entry["bytes"]
        assert gitid(b"blob", source) == entry["git_blob"]
        matches = [p for p in proofs["proofs"] if p["revision"] == rev
                   and p["path"] == entry["git_path"] and p["git_blob"] == entry["git_blob"]]
        assert matches
        proof = matches[0]
        assert packet.extractfile("evidence/" + proof["archive"]).read() == source
        current_tree = root_tree
        parts = entry["git_path"].split("/")
        assert len(parts) == len(proof["chain"])
        for i, (part, hop) in enumerate(zip(parts, proof["chain"], strict=True)):
            assert hop["tree"] == current_tree and hop["name"] == part
            info = proofs["objects"][current_tree]
            tree = gzip.decompress(packet.extractfile("evidence/" + info["archive"]).read())
            assert sha256(tree) == info["sha256"] and gitid(b"tree", tree) == current_tree
            mode, oid = find_tree_entry(tree, part)
            assert mode == hop["mode"].lstrip("0") and oid == hop["object"]
            if i + 1 == len(parts):
                assert oid == entry["git_blob"] and hop["type"] == "blob"
            else:
                assert hop["type"] == "tree"
                current_tree = oid
        lines = source.decode().splitlines()
        assert len(lines) == entry["eof"]
        read = set()
        for start, end in entry["registered_ranges"]:
            assert 1 <= start <= end <= len(lines)
            expected_display.append(f"--- {entry['tag']} {start}-{end} ---\n")
            expected_display.extend(f"{n:4d} {lines[n-1]}\n" for n in range(start, end + 1))
            read.update(range(start, end + 1))
        unread = {n for start, end in row["unread_complement"] for n in range(start, end + 1)}
        assert unread == set(range(1, len(lines) + 1)) - read
assert "".join(expected_display).encode() == display_bytes
extension_bytes = (stage / "EXT-REGISTRATION.json").read_bytes()
extension = json.loads(extension_bytes)
extension_display = (stage / "EXT-DISPLAY.txt").read_bytes()
extension_receipt = json.loads((stage / "EXT-READ-RECEIPT.json").read_bytes())
assert extension_receipt["registration_sha256"] == sha256(extension_bytes)
assert extension_receipt["display_sha256"] == sha256(extension_display)
assert extension_receipt["reader"] == extension["reader"] == "/root"
assert [e["tag"] for e in extension["entries"]] == ["groom", "vec3", "collider"]
extension_lines = []
for entry, row in zip(extension["entries"], extension_receipt["rows"], strict=True):
    assert entry["revision"] == row["revision"] == rev
    assert (entry["tag"], entry["git_path"], entry["registered_ranges"]) == (
        row["tag"], row["git_path"], row["displayed_ranges"]
    )
    source = (stage / entry["capture"]).read_bytes()
    assert sha256(source) == entry["sha256"] and len(source) == entry["bytes"]
    assert gitid(b"blob", source) == entry["git_blob"]
    if entry["tag"] != "vec3":
        earlier = next(e for e in registration["entries"] if e["tag"] == entry["tag"])
        assert (entry["git_path"], entry["git_blob"], entry["sha256"]) == (
            earlier["git_path"], earlier["git_blob"], earlier["sha256"]
        )
    else:
        proof = json.loads((stage / "EXT-TREE-PROOF.json").read_bytes())
        assert proof["root_tree"] == root_tree and proof["revision"] == rev
        assert (proof["path"], proof["git_blob"]) == (entry["git_path"], entry["git_blob"])
        current_tree = root_tree
        for part, hop in zip(entry["git_path"].split("/"), proof["chain"], strict=True):
            assert hop["tree"] == current_tree and hop["name"] == part
            tree = gzip.decompress((stage / "extension-trees" / f"{current_tree}.tree.gz").read_bytes())
            assert gitid(b"tree", tree) == current_tree
            mode, oid = find_tree_entry(tree, part)
            assert (mode, oid) == (hop["mode"], hop["object"])
            current_tree = oid
        assert current_tree == entry["git_blob"]
    lines = source.decode().splitlines()
    assert len(lines) == entry["eof"]
    read = set()
    for start, end in entry["registered_ranges"]:
        extension_lines.append(f"--- {entry['tag']} {start}-{end} ---\n")
        extension_lines.extend(f"{n:4d} {lines[n-1]}\n" for n in range(start, end + 1))
        read.update(range(start, end + 1))
    unread = {n for start, end in row["unread_complement"] for n in range(start, end + 1)}
    assert unread == set(range(1, len(lines) + 1)) - read
assert "".join(extension_lines).encode() == extension_display
assessment = (root / "ASSESSMENT-CENTRAL-COLLIDER-13M-95E.json").read_bytes()
assert sha256((stage / "ASSESSMENT-DRAFT.json").read_bytes()) == "3b03e72fb781089e7974fe20c2e96ce66ceaa505963f281af61f6d3056f02106"
assert sha256(assessment) == "ac429433dd7c22ab6d0db209a410bd867c48e87a76516c4734841471fe9ad68f"
base = json.loads((root / "BASELINE-95E-20261110.json").read_bytes())
ledger = base["newly_assessed_ledger"]["entries"]
for candidate in ("local-candidate-mesh-reach-no-hit-semantics", "local-candidate-keep-clear-between-samples"):
    rows = [e for e in ledger if e["id"] == candidate]
    assert len(rows) == 1 and rows[0]["record"] == "ASSESSMENT-CENTRAL-COLLIDER-13M-95E.json"
    assert rows[0]["repair"] == "not started" and rows[0]["validation"] == "not performed"
queue = json.loads((root / "RECONCILIATION-059.json").read_bytes())["entries"]
assert queue[1843]["contract_id"] == "Q-L12-0f9de10a33ac66397bd3"
assert queue[1843]["current_reconciliation"].startswith("partial:")
progress = json.loads((root / "CONTINUATION-PROGRESS-95E.json").read_bytes())
assert progress["new_current_ledger_candidates"] == len(ledger)
assert progress["lenses_complete"] == progress["queue_records_semantically_reconciled"] == 0
print("Verified five commit-anchored source chains, both registered displays and receipts, two conditional ledger links and partial q1843.")
print("Transcript reconstruction is not proof of personal display; no product behavior or independent defect count is established.")
