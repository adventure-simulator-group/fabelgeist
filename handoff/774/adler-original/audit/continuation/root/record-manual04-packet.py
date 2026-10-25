"""Persist authored audit conclusions; no source analysis or project execution."""
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

C = Path(__file__).resolve().parents[1]
A = C / 'root/initial04-invariant-operations'
def read(p): return json.loads(p.read_text(encoding='utf-8'))
def pin(p): return {'record': 'continuation/' + p.relative_to(C).as_posix(), 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()}
payload = read(Path(sys.argv[1]))
number = payload['packet_number']
destination = A / f'contract-groups-{number:03}.json'
assert not destination.exists(), str(destination)
authority_path = A / 'scope-authority-001.json'
authority = read(authority_path)
scope_path = C.parent / authority['scope']['record']
scope = read(scope_path)
byid = {r['source_id']: r for r in scope['files']}
roles = payload['source_roles']
for r in roles:
    scoped = byid[r['source_id']]
    assert r['complete_scoped_comparison'] is True
    r['sha256'] = scoped['sha256']
    path = Path(scoped['path'])
    if not path.is_absolute():
        snap = {'physical-p16': 'p16', 'pr-working': 'pr-working', 'physical-p17': 'p17', 'rig-quality': 'rig-quality'}[r['source_id'].split(':', 1)[0]]
        path = C.parents[1] / '774-snapshots' / snap / r['source_id'].split(':', 1)[1]
    assert hashlib.sha256(path.read_bytes()).hexdigest() == r['sha256']
    actual_eof = len(path.read_text(encoding='utf-8').splitlines())
    assert all(1 <= lo <= hi <= actual_eof for lo, hi in r['actual_compared_ranges'])
    r['actual_EOF'] = actual_eof
    row = next(x for x in authority['rows'] if x['source_id'] == r['source_id'])
    assert row['source_sha256'] == r['sha256']
old_findings = [json.loads(l) for l in (A / 'findings.jsonl').read_text(encoding='utf-8').splitlines()]
assert not set(f['id'] for f in payload.get('findings', [])) & set(f['id'] for f in old_findings)
packet = dict(payload)
packet.update(id=f'ROOT-INITIAL04-CONTRACT-{number:03}', at=datetime.now(timezone.utc).isoformat(), lens='audit-invariant-operations', scope=authority['scope'], mandatory_overlays=authority['mandatory_overlays'], mandatory_qualification170=pin(C / 'root/parallel-scope-qualification-170.json'), repair='frozen_initial_audits_then_handover', validation='not_run', source_edits=0, project_execution=0, other_lens_credit=False)
for finding in packet.get('findings', []): finding['contract_ref'] = packet['id']
destination.write_text(json.dumps(packet, indent=2) + '\n', encoding='utf-8')
with (A / 'coverage.jsonl').open('a', encoding='utf-8') as stream:
    for r in roles:
        stream.write(json.dumps(dict(r, contract_ref=packet['id'], status='exact_current04_operations_compared', repair='frozen', validation='not_run')) + '\n')
with (A / 'findings.jsonl').open('a', encoding='utf-8') as stream:
    for f in packet.get('findings', []): stream.write(json.dumps(f) + '\n')
for r in roles:
    row = next(x for x in authority['rows'] if x['source_id'] == r['source_id'])
    row['status'] = 'exact_scoped_current04_operations_manually_compared'
    row['actual_compared_ranges'] = r['actual_compared_ranges']
    row['evidence'].append(pin(destination))
authority_path.write_text(json.dumps(authority, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'packet': pin(destination), 'completed_sources': sum(r['status'] == 'exact_scoped_current04_operations_manually_compared' for r in authority['rows']), 'findings': len(old_findings) + len(packet.get('findings', []))}))
