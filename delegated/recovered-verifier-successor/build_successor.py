#!/usr/bin/env python3
"""Deterministically derive a separately named evidence-verifier successor.

No source, manifest, or original is modified. --check is read-only; --write
creates a new file and refuses overwrite. This builds only evidence tooling.
"""
import argparse, hashlib, pathlib

SOURCE_SHA = '5fbd014581bbfc412596040ef923f680c72284115d3a5f4f7287f214212cb458'
HERE = pathlib.Path(__file__).resolve().parent

HELPER = '''\n# BEGIN QUALIFIED PUBLIC REDACTIONS (three historical objects only)
# The historic manifest hash and original filename remain historical identity.
# These are public reconstructions: their content hashes are different and the
# original private bytes are not verified by this successor.
_qualified_crosswalk_path = "recovered/privacy-redaction-evidence/CROSSWALK.json"
_qualified_crosswalk_sha = "6f555e7505f101b2b51df1ae793db8cf8a24cba8ad354a5fa39a4f481d731e7f"
_qualified_objects = {
    "recovered/l12-final-registration-evidence/prior-direct-dependencies/0b6bdc7320050b3c799e6cb87f9ecfceb97a8d72d84e04c7aacae8b55b2d1ce0":
        ("0b6bdc7320050b3c799e6cb87f9ecfceb97a8d72d84e04c7aacae8b55b2d1ce0", 109772, "67106f070825d632c93c91a29d495ecb4157d735179ee278f55dc6700e7e1ee9", 109688, 1),
    "recovered/l2-canonical-comparison-evidence/74f8fe4d07b187636dc8568ba5f129f02157d43d67b6d67c618c7259a808be48":
        ("74f8fe4d07b187636dc8568ba5f129f02157d43d67b6d67c618c7259a808be48", 2913027, "25cbe3dcc967db682e2db78a4371d8ed8605976df5f2d3f150db509b2a5b7ea3", 2911040, 28),
    "recovered/prior-current-transition-evidence/7ab4700aa59883fb73e3c7ef55ea2c05ed5e13661ee8a5b82dae1732d76188a5":
        ("7ab4700aa59883fb73e3c7ef55ea2c05ed5e13661ee8a5b82dae1732d76188a5", 120752, "c21dca58dc77ed95ec680a61b91d45f49c1c86be0274ebf48409919f514d8c30", 120669, 1),
}
_qualified_manifests = {
    "recovered/l12-final-registration-evidence/MANIFEST.json": "48d91f4d15f4f4035c94668d707b5956fe65f878664406cfc697621120a537b3",
    "recovered/l2-canonical-comparison-evidence/MANIFEST.json": "fa8daf479053b9185d5efda335d63ab303ea8b2e63649f986c865690d892d4f9",
    "recovered/prior-current-transition-evidence/MANIFEST.json": "cae5df98ad256d51427c669c15cd84128b22d554f7f11116fc1e0b51152de51f",
}
_qualified_crosswalk_bytes = (root / _qualified_crosswalk_path).read_bytes()
if hashlib.sha256(_qualified_crosswalk_bytes).hexdigest() != _qualified_crosswalk_sha:
    raise ValueError("Qualified public redaction crosswalk hash mismatch")
_qualified_crosswalk = json.loads(_qualified_crosswalk_bytes)
if (_qualified_crosswalk.get("schema") != "726-privacy-proposal-crosswalk-v1"
        or _qualified_crosswalk.get("redaction_count") != 30
        or len(_qualified_crosswalk.get("objects", [])) != 3):
    raise ValueError("Qualified public redaction crosswalk scope mismatch")
_qualified_rows = {}
for _row in _qualified_crosswalk["objects"]:
    _original = _row["original"]
    _relative = _original["publication_path"]
    if _relative in _qualified_rows or _relative not in _qualified_objects:
        raise ValueError("Unexpected qualified public redaction identity")
    _orig_sha, _orig_bytes, _public_sha, _public_bytes, _spans = _qualified_objects[_relative]
    _public = _row["variant"]
    if ((_original["sha256"], _original["bytes"], _public["sha256"], _public["bytes"], len(_row["redactions"]))
            != (_orig_sha, _orig_bytes, _public_sha, _public_bytes, _spans)):
        raise ValueError("Qualified public redaction identity mismatch")
    _qualified_rows[_relative] = _row
if set(_qualified_rows) != set(_qualified_objects):
    raise ValueError("Incomplete qualified public redaction crosswalk")
_qualified_seen = set()

def _qualified_historical_manifest(relative):
    if relative not in _qualified_manifests:
        raise ValueError("Unexpected qualified historical manifest")
    data = (root / relative).read_bytes()
    if hashlib.sha256(data).hexdigest() != _qualified_manifests[relative]:
        raise ValueError("Qualified historical manifest hash mismatch")
    return data

def _qualified_public_object(relative, historical_sha, data):
    if relative not in _qualified_objects:
        raise ValueError("Unexpected qualified public object path")
    original_sha, _, public_sha, public_bytes, _ = _qualified_objects[relative]
    if historical_sha != original_sha:
        raise ValueError("Historical manifest and crosswalk identity mismatch")
    if len(data) != public_bytes or hashlib.sha256(data).hexdigest() != public_sha:
        raise ValueError("Qualified reconstructed public bytes mismatch")
    document = json.loads(data)
    for span in _qualified_rows[relative]["redactions"]:
        value = document
        for part in span["json_pointer"].split("/")[1:]:
            part = part.replace("~1", "/").replace("~0", "~")
            value = value[int(part)] if isinstance(value, list) else value[part]
        marker = span["replacement"]
        start, size = span["public_byte_offset"], span["public_token_bytes"]
        if value != marker or data[start:start + size] != json.dumps(marker).encode("utf-8"):
            raise ValueError("Qualified reconstructed public redaction span mismatch")
    _qualified_seen.add(relative)
    return data
# END QUALIFIED PUBLIC REDACTIONS
'''

REPLACEMENTS = [
('root = Path(__file__).parent\n', 'root = Path(__file__).parent\n' + HELPER),
('prior_transition = json.loads((prior_transition_dir / "MANIFEST.json").read_text(encoding="utf-8"))',
 'prior_transition = json.loads(_qualified_historical_manifest("recovered/prior-current-transition-evidence/MANIFEST.json"))'),
('if hashlib.sha256(prior_data).hexdigest() != prior_sha:\n    raise ValueError("Prior current transition hash mismatch")',
 '_qualified_public_object("recovered/prior-current-transition-evidence/" + prior_sha, prior_sha, prior_data)'),
('comparison = json.loads((comparison_dir / "MANIFEST.json").read_text(encoding="utf-8"))',
 'comparison = json.loads(_qualified_historical_manifest("recovered/l2-canonical-comparison-evidence/MANIFEST.json"))'),
('for item in comparison["files"]:\n    if hashlib.sha256((comparison_dir / item["file"]).read_bytes()).hexdigest() != item["sha256"]:\n        raise ValueError("Canonical comparison evidence hash mismatch")',
 'comparison_master_bytes = None\nfor item in comparison["files"]:\n    if item["file"] == "74f8fe4d07b187636dc8568ba5f129f02157d43d67b6d67c618c7259a808be48":\n        comparison_master_bytes = _qualified_public_object(\n            "recovered/l2-canonical-comparison-evidence/" + item["file"], item["sha256"],\n            (comparison_dir / item["file"]).read_bytes())\n    elif hashlib.sha256((comparison_dir / item["file"]).read_bytes()).hexdigest() != item["sha256"]:\n        raise ValueError("Canonical comparison evidence hash mismatch")\nif (comparison["inherited_master_sha256"] != "74f8fe4d07b187636dc8568ba5f129f02157d43d67b6d67c618c7259a808be48"\n        or comparison_master_bytes is None):\n    raise ValueError("Qualified canonical comparison original identity missing")'),
('master = json.loads((comparison_dir / comparison["inherited_master_sha256"]).read_bytes())',
 'master = json.loads(comparison_master_bytes)'),
('l12_final = json.loads((l12_final_dir / "MANIFEST.json").read_text(encoding="utf-8"))',
 'l12_final = json.loads(_qualified_historical_manifest("recovered/l12-final-registration-evidence/MANIFEST.json"))'),
('    if hashlib.sha256(data).hexdigest() != ref["sha256"]:\n        raise ValueError("Lens 12 prior dependency content hash mismatch")\n    l12_direct_hashes.add(ref["sha256"])',
 '    if ref["sha256"] == "0b6bdc7320050b3c799e6cb87f9ecfceb97a8d72d84e04c7aacae8b55b2d1ce0":\n        _qualified_public_object("recovered/l12-final-registration-evidence/prior-direct-dependencies/" + ref["sha256"], ref["sha256"], data)\n    elif hashlib.sha256(data).hexdigest() != ref["sha256"]:\n        raise ValueError("Lens 12 prior dependency content hash mismatch")\n    l12_direct_hashes.add(ref["sha256"])'),
('print("Verified", count, "evidence files,", dependency_count,',
 'if _qualified_seen != set(_qualified_objects):\n    raise ValueError("Not all three qualified public redactions were checked")\nprint("Qualified redaction: three reconstructed public objects and their original manifest identities checked; original private bytes were not verified.")\n\nprint("Verified", count, "evidence files,", dependency_count,'),
]

def transform(raw):
    if hashlib.sha256(raw).hexdigest() != SOURCE_SHA: raise ValueError('unexpected original verifier identity')
    src = raw.decode('utf-8')
    for old,new in REPLACEMENTS:
        if src.count(old)!=1: raise ValueError('transformation anchor is missing or non-unique')
        src=src.replace(old,new,1)
    return src.encode('utf-8')

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--source',type=pathlib.Path,default=HERE/'ORIGINAL-verify_recovered.py')
    group=ap.add_mutually_exclusive_group();group.add_argument('--write',type=pathlib.Path);group.add_argument('--check',type=pathlib.Path)
    args=ap.parse_args();source=args.source.read_bytes();out=transform(source)
    if args.write:
        with args.write.open('xb') as f:f.write(out)
    elif args.check:
        if args.check.read_bytes()!=out:raise ValueError('successor differs from deterministic transformation')
    print(hashlib.sha256(out).hexdigest())
if __name__=='__main__':main()
