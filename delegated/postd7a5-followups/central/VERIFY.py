#!/usr/bin/env python3
"""Verify source-capture transports and registered display identities."""
import hashlib
import io
import json
import sys
import tarfile
from pathlib import Path, PurePosixPath

if sys.flags.optimize:
    raise RuntimeError("Evidence verification requires assertions")
p = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
m = json.loads((p / "MANIFEST.json").read_bytes())
assert m["schema"] == "726-postd7a5-central-source-transport-v1"
raw = (p / m["archive"]).read_bytes()
assert (len(raw), sha(raw)) == (m["archive_bytes"], m["archive_sha256"])
expected = {x["path"]: x for x in m["members"]}
assert len(expected) == len(m["members"])
data = {}
with tarfile.open(fileobj=io.BytesIO(raw), mode="r:gz") as archive:
    for member in archive:
        name = PurePosixPath(member.name)
        assert not name.is_absolute() and ".." not in name.parts
        assert member.isfile() and member.name in expected and member.name not in data
        body = archive.extractfile(member).read()
        row = expected[member.name]
        assert (len(body), sha(body), oct(member.mode)) == (row["bytes"], row["sha256"], row["mode"])
        data[member.name] = body
assert set(data) == set(expected)

owner = json.loads(data["owner/CAPTURE.json"])
registration = json.loads(data["owner/REGISTRATION.json"])
receipt = json.loads(data["owner/READ-RECEIPT.json"])
assert owner["revision"] == registration["revision"] == receipt["revision"]
assert len(owner["rows"]) == len(registration["planned"]) == 13
owner_views = 0
owner_lines = 0
for row, registered in zip(owner["rows"], registration["planned"]):
    assert (row["label"], row["path"]) == (registered["label"], registered["path"])
    source = data["owner/" + row["label"] + ".source"]
    lines = source.decode().splitlines(keepends=True)
    assert (len(source), len(lines), sha(source)) == (row["source_bytes"], row["line_count"], row["source_sha256"])
    assert [(x["start"], x["end"]) for x in row["displays"]] == [tuple(x) for x in registered["ranges"]]
    for view in row["displays"]:
        body = data["owner/" + view["file"]]
        expected_view = "".join(f"{i}: {lines[i-1]}" for i in range(view["start"], view["end"] + 1)).encode()
        assert body == expected_view and (len(body), sha(body)) == (view["bytes"], view["sha256"])
        owner_views += 1
        owner_lines += view["end"] - view["start"] + 1
assert (owner_views, owner_lines) == (receipt["displayed_ranges"], receipt["displayed_lines"]) == (23, 757)

other_views = 0
for label in ["language-bestiary", "enums", "helmet"]:
    reg = data[label + "/REGISTRATION.json"]
    cap = json.loads(data[label + "/CAPTURE.json"])
    read = json.loads(data[label + "/READ-RECEIPT.json"])
    assert sha(reg) == cap["registration_sha256"] == read["registration_sha256"]
    assert sha(data[label + "/CAPTURE.json"]) == read["capture_sha256"]
    sources = {r["path"]: r for r in cap["sources"]}
    assert cap["views"] == read["displayed_views"]
    for row in sources.values():
        source = data[label + "/" + row["captured_file"]]
        assert sha(source) == row["sha256"] and len(source.decode().splitlines()) == row["EOF"]
    for view in cap["views"]:
        source = data[label + "/" + sources[view["path"]]["captured_file"]].decode().splitlines()
        lo, hi = view["range"]
        expected_view = "".join(f"{i:5d}  {source[i-1]}\n" for i in range(lo, hi + 1)).encode()
        body = data[label + "/" + view["view"]]
        assert body == expected_view and sha(body) == view["view_sha256"]
        other_views += 1
    assert read["distinct_displayed_lines"] == sum(v["range"][1] - v["range"][0] + 1 for v in cap["views"])
assert other_views == 34
failure = json.loads(data["helmet-failed-capture/CAPTURE-FAILURE.json"])
assert "335-344" in failure["failure"] and "no personal read credit" in failure["effects"].lower()
draft = json.loads(data["drafts/CONSOLIDATED-DRAFT.json"])
assert draft["parent_published_checkpoint"] == "d7a5e77ebdbd44a05cde30881493d6fc22ef025d"
assert draft["counts"]["new_partial_queue_assessments"] == 6
assert draft["counts"]["context_only_observations"] == 2
for item in draft["draft_provenance"]:
    assert sha(data["drafts/" + item["selected_draft"]]) == item["sha256"]
print(f"Verified {len(data)} central evidence members, {owner_views + other_views} registered views "
      "and draft identities; pinned Git and historical queue proofs remain separately held.")
