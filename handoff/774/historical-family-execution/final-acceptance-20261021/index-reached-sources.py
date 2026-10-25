"""Index source proofs already recorded in this wave; no project execution."""
from pathlib import Path
import json
import hashlib
import datetime

A = Path(__file__).resolve().parent
D = A / 'wave-01/audit-producer-propagation'
manifest = json.loads((A / 'source-manifest.json').read_text())
snapshots = {x['id']: x for x in manifest['snapshots']}
frozen = {x['source_id'] for x in manifest['files']}
records = {}
rejected = []

def consider(node, evidence, shard):
    if not isinstance(node, dict):
        return
    path = node.get('path', node.get('file'))
    sha = node.get('observed_sha256', node.get('sha256'))
    if not isinstance(path, str) or not isinstance(sha, str) or len(sha) != 64:
        return
    if not path.startswith(('crates/', 'assets/', 'content/', 'scripts/', '.github/')):
        return
    raw_snapshot = node.get('snapshot', node.get('source_id', '').split(':')[0])
    snapshot = next((key for key, value in snapshots.items()
                     if raw_snapshot.startswith(key) or raw_snapshot == value['head']), 'pr-working')
    # Preserved increments do not bring their unchanged stale parent source
    # into integration scope. Non-delta receiver records use current PR bytes.
    if snapshot != 'pr-working' and f'{snapshot}:{path}' not in frozen:
        snapshot = 'pr-working'
    file = Path(snapshots[snapshot]['root']) / path
    if not file.is_file():
        return
    actual = hashlib.sha256(file.read_bytes()).hexdigest()
    if actual != sha:
        rejected.append({'evidence': evidence, 'path': path, 'snapshot': snapshot, 'reason': 'record hash does not match selected frozen source; not admitted to derived index'})
        return
    sid = f'{snapshot}:{path}'
    row = records.setdefault(sid, {'source_id': sid, 'snapshot': snapshot, 'path': path, 'sha256': actual, 'bytes': file.stat().st_size, 'already_in_frozen_variants': sid in frozen, 'evidence': []})
    proof = {'shard': shard, 'record': evidence, 'ranges': node.get('read_ranges', node.get('ranges')), 'source_read_status': node.get('read_status'), 'trace_refs': node.get('trace_refs', node.get('trace_ids', []))}
    if proof not in row['evidence']:
        row['evidence'].append(proof)

def walk(node, evidence, shard):
    if isinstance(node, dict):
        consider(node, evidence, shard)
        for value in node.values():
            walk(value, evidence, shard)
    elif isinstance(node, list):
        for value in node:
            walk(value, evidence, shard)

for shard in ['root', 'camera', 'rig', 'native']:
    for file in sorted((D / shard).glob('*.json*')):
        # Only current-wave proof inventories are indexed. This is not an
        # acceptance judgement, and old verdict prose never supplies coverage.
        evidence = str(file.relative_to(A))
        if file.suffix == '.jsonl':
            for line in file.read_text().splitlines():
                if line.strip():
                    walk(json.loads(line), evidence, shard)
        else:
            walk(json.loads(file.read_text()), evidence, shard)

output = {
    'generated_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'authority': 'Derived index of actual current-wave source proof records. Original source-manifest.json remains immutable. Audit status lives only in audit-ledger.json; exact repair ownership remains in claim files.',
    'limits': 'Hash matching admits source scope only. Neither recorded read ranges, historical verdicts nor this index establish lens discovery or clearance. Partial unchanged-module reads close only the actual traced paths; subsequent lenses must inventory every materially affected path independently.',
    'frozen_manifest_sha256': hashlib.sha256((A / 'source-manifest.json').read_bytes()).hexdigest(),
    'rows': sorted(records.values(), key=lambda x: x['source_id']),
    'unmatched_record_observations': rejected,
}
(A / 'reached-source-index.json').write_text(json.dumps(output, indent=2) + '\n')
print(json.dumps({'indexed_current_source_paths': len(records), 'additional_reached_variants': sum(not x['already_in_frozen_variants'] for x in records.values()), 'unmatched_observations': len(rejected), 'source_or_project_commands_executed': False}))
