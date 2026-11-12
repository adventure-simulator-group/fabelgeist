#!/usr/bin/env python3
"""Check early successor identities and their deterministic source diffs."""
import difflib, hashlib, json
from pathlib import Path
root = Path(__file__).resolve().parent
stage = root / 'delegated/qualified-059-early-links'
data = json.loads((stage / 'INVENTORY.json').read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
assert data['schema'] == '726-059-early-link-qualified-successors-v1'
assert data['qualified_direct_link_count'] == 22
assert {r['original'] for r in data['rows']} == {f'verify_delegated_links_{i}.py' for i in range(11, 16)}
for row in data['rows']:
    old = (root / row['original']).read_bytes()
    new = (stage / row['successor']).read_bytes()
    diff = (stage / row['diff']).read_bytes()
    assert sha(old) == row['original_sha256']
    assert sha(new) == row['successor_sha256']
    assert sha(diff) == row['diff_sha256']
    computed = ''.join(difflib.unified_diff(old.decode().splitlines(True), new.decode().splitlines(True), fromfile=row['original'], tofile=Path(row['successor']).name)).encode()
    assert computed == diff
print('Verified five retained historical scripts, separately named successor identities and source diffs; 22 links require the qualified public crosswalk.')
