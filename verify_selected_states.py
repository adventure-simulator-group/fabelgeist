"""Verify historical selected-state evidence; no product validation."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
comparison = json.loads((root / 'SELECTED-STATE-COMPARISON.json').read_text())
selector_bytes = (root / 'baseline-059/selected.json').read_bytes()
if hashlib.sha256(selector_bytes).hexdigest() != '5796e1d5a28a33c537a006cce8dd429ebf957450c6ace458a566c137f1cf4a82':
    raise ValueError('checkpoint selector mismatch')
selector = json.loads(selector_bytes)
snapshot_bytes = (root / comparison['snapshot_440_file']).read_bytes()
if hashlib.sha256(snapshot_bytes).hexdigest() != comparison['snapshot_440_sha256']:
    raise ValueError('snapshot mismatch')
snapshot = json.loads(snapshot_bytes)
old = {item['lens']: item['selected_state'] for item in selector['lenses']}
new = {item['lens']: item['selected_state'] for item in snapshot['lenses']}
if set(old) != set(range(1, 14)) or set(new) != set(old):
    raise ValueError('lens inventory mismatch')
if len(comparison['rows']) != 13 or {row['lens'] for row in comparison['rows']} != set(old):
    raise ValueError('comparison inventory mismatch')
for row in comparison['rows']:
    lens = row['lens']
    if row['checkpoint_059'] != old[lens] or row['snapshot_440'] != new[lens]:
        raise ValueError('selector mismatch')
    data = (root / row['checkpoint_059_file']).read_bytes()
    if len(data) != old[lens]['bytes'] or hashlib.sha256(data).hexdigest() != old[lens]['sha256']:
        raise ValueError('state mismatch')
    if row['snapshot_440_same_as_059'] != (old[lens]['sha256'] == new[lens]['sha256']):
        raise ValueError('comparison mismatch')
print('verified 13 historical states and 13 selector comparisons')
