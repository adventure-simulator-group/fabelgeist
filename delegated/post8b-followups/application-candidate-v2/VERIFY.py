#!/usr/bin/env python3
"""Verify evidence-only successor images, with optional exact predecessors."""
import argparse
import hashlib
import importlib.util
import json
import os
import stat
import subprocess
import sys
from pathlib import Path

if sys.flags.optimize:
    raise RuntimeError("Evidence assertions require an unoptimized interpreter")
p = Path(__file__).resolve().parent
ap = argparse.ArgumentParser()
ap.add_argument("--publication", type=Path)
ap.add_argument("--predecessor", type=Path)
ap.add_argument("--ancestor", type=Path)
a = ap.parse_args()
assert not a.ancestor or a.predecessor
sha = lambda b: hashlib.sha256(b).hexdigest()
blob = lambda b: hashlib.sha1(b"blob " + str(len(b)).encode() + b"\0" + b).hexdigest()
manifest = json.loads((p / "MANIFEST.json").read_bytes())
assert manifest["schema"] == "726-post8b-candidate-public-v1"
actual = set()
for path in p.rglob("*"):
    assert not path.is_symlink()
    if path.is_file():
        actual.add(path.relative_to(p).as_posix())
    else:
        assert path.is_dir()
assert actual == set(manifest["files"]) | {"MANIFEST.json"}
for name, row in manifest["files"].items():
    raw = (p / name).read_bytes()
    assert (len(raw), sha(raw)) == (row["bytes"], row["sha256"])
pins = json.loads((p / "PINS.json").read_bytes())
assert pins["schema"] == "726-post8b-candidate-pins-v1"
assert pins["parent"] == "8b4b9b9aae3b9d9821418fa3ad5cff4a21e5384b"
assert pins["parent_tree"] == "552bb7853c70fea3c124eb0096d53e3e99b9439a"
assert pins["source_revision"] == "95e63628f9d70b8dd6e6fede64334696c6995009"
assert pins["source_tree"] == "1eec97de340d9b7debbb30ca0c0fb2240a73b93f"
assert sha((p / "TRANSFORM.py").read_bytes()) == "5cece33b4703393559953b1abd35d65baf060f0aa5ed841d9a59b1ed4e9b0f50"
assert sha((p / "PROPOSAL.json").read_bytes()) == "145555a6f5ad092174f4214b04e255660e5903dcb18bbef293baaca8b8500d44"
spec = importlib.util.spec_from_file_location("audit_evidence_transform", p / "TRANSFORM.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
before = {name: (p / "before" / name).read_bytes() for name in module.NAMES}
report = (p / module.REPORT).read_bytes()
observation = (p / module.MAIN).read_bytes()
bindings = json.loads((p / "BINDINGS.json").read_bytes())
assert pins["report_sha256"] == sha(report) and pins["observation_sha256"] == sha(observation)
after = module.transform(before, (p / "PROPOSAL.json").read_bytes(), report, observation, bindings)
assert len(pins["parent_paths"]) == 14 and len({x["path"] for x in pins["parent_paths"]}) == 14
proposal = json.loads((p / "PROPOSAL.json").read_bytes())
packet_paths = [name for item in proposal["assessments"] for name in item["source_packets"]]
assert len(packet_paths) == len(set(packet_paths)) == 10
assert {x["path"] for x in pins["parent_paths"]} == set(module.NAMES) | set(packet_paths)
for name in module.NAMES:
    assert after[name] == (p / "after" / name).read_bytes()
    row = pins["outputs"][name]
    assert (len(after[name]), sha(after[name]), blob(after[name])) == (row["bytes"], row["sha256"], row["git_blob"])
    old = next(x for x in pins["parent_paths"] if x["path"] == name)
    assert (len(before[name]), sha(before[name]), blob(before[name])) == (old["bytes"], old["sha256"], old["git_blob"])
env = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PYTHONNOUSERSITE="1")
env.pop("PYTHONOPTIMIZE", None)
review_root = p.parent / "review-binding"
review_manifest_path = review_root / "MANIFEST.json"
assert review_manifest_path.is_file() and not review_manifest_path.is_symlink()
review_manifest_raw = review_manifest_path.read_bytes()
assert sha(review_manifest_raw) == "08eef34795e57d63f66df3bb711bbe8beb72b2895ec1f9fb93ea6fc3054c427f"
review_manifest = json.loads(review_manifest_raw)
assert review_manifest["schema"] == "726-post8b-review-binding-public-v1"
assert set(review_manifest["files"]) == {"BINDING.json", "README.md", "VERIFY.py"}
assert {x.name for x in review_root.iterdir()} == set(review_manifest["files"]) | {"MANIFEST.json"}
for name, row in review_manifest["files"].items():
    target = review_root / name
    assert target.is_file() and not target.is_symlink()
    raw = target.read_bytes()
    assert (len(raw), sha(raw)) == (row["bytes"], row["sha256"])
review_binding = json.loads((review_root / "BINDING.json").read_bytes())
assert [x["original_manifest_sha256"] for x in review_binding["reviews"]] == module.ASSEMBLY_REVIEW["original_manifest_sha256"]
subprocess.run([sys.executable, "-B", str(p.parent / "review-binding/VERIFY.py")], env=env, check=True)
if a.publication:
    publication = a.publication.resolve()
    expected_package = publication / "delegated/post8b-followups/application-candidate-v2"
    assert not expected_package.is_symlink() and p == expected_package
    for name in module.NAMES:
        target = publication / name
        assert target.is_file() and not target.is_symlink() and target.read_bytes() == after[name]
        row = next(x for x in pins["parent_paths"] if x["path"] == name)
        assert bool(stat.S_IMODE(target.stat().st_mode) & stat.S_IXUSR) == (row["git_mode"] == "100755")
    for name, data in ((module.REPORT, report), (module.MAIN, observation)):
        target = publication / name
        assert target.is_file() and not target.is_symlink() and target.read_bytes() == data
        assert not stat.S_IMODE(target.stat().st_mode) & stat.S_IXUSR
    assert (publication / "delegated/post8b-followups/application-candidate-v2/MANIFEST.json").read_bytes() == (p / "MANIFEST.json").read_bytes()
if a.predecessor:
    predecessor = a.predecessor.resolve()
    def git(*args):
        return subprocess.run(["git", "-c", "gc.auto=0", "-c", "maintenance.auto=false", *args],
                              cwd=predecessor, env=env, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout
    assert git("rev-parse", "HEAD").strip().decode() == pins["parent"]
    assert git("rev-parse", "HEAD^{tree}").strip().decode() == pins["parent_tree"]
    assert not git("status", "--porcelain=v1", "-z")
    for row in pins["parent_paths"]:
        name = row["path"]
        target = predecessor / name
        assert target.is_file() and not target.is_symlink()
        raw = target.read_bytes()
        assert raw == git("show", pins["parent"] + ":" + name)
        assert (len(raw), sha(raw), blob(raw)) == (row["bytes"], row["sha256"], row["git_blob"])
        expected_tree_row = (row["git_mode"] + " blob " + row["git_blob"] + "\t" + name).encode()
        assert git("ls-tree", pins["parent"], "--", name).strip() == expected_tree_row
        assert bool(stat.S_IMODE(target.stat().st_mode) & stat.S_IXUSR) == (row["git_mode"] == "100755")
    if a.ancestor:
        subprocess.run([sys.executable, "-B", str(predecessor / "verify_post8ca_checkpoint.py"),
                        "--predecessor", str(a.ancestor.resolve())], cwd=predecessor, env=env, check=True)
        print("Exact 8b predecessor guard and nested 8ca predecessor replay passed.")
    else:
        print("Exact 8b parent paths matched; old guard and nested 8ca replay were not run without --ancestor.")
else:
    print("Separate exact 8b parent paths and old guard were not checked without --predecessor and --ancestor.")
print("Candidate evidence transform and review-member identity checks passed; connector observation is point-in-time. No product execution or semantic closure follows.")
