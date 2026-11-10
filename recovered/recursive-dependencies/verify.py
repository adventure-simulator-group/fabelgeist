import hashlib, json
from pathlib import Path

root = Path(__file__).resolve().parent
manifest = json.loads((root / "MANIFEST.json").read_text(encoding="utf-8"))
assert len(manifest) == 4
for item in manifest:
    digest = item["sha256"]
    assert hashlib.sha256((root / "objects" / digest).read_bytes()).hexdigest() == digest
    for ref in item["references"]:
        parent = root.parent / "hash-matched-dependencies" / "objects" / ref["from_dependency"]
        data = parent.read_bytes()
        assert hashlib.sha256(data).hexdigest() == ref["from_dependency"]
        value = json.loads(data)
        def contains(v):
            if isinstance(v, dict):
                return (v.get("path") == ref["cited_path"] and v.get("sha256") == digest) or any(contains(x) for x in v.values())
            if isinstance(v, list):
                return any(contains(x) for x in v)
            return False
        assert contains(value)
print("Verified 4 recursive dependency objects and their parent citations")
