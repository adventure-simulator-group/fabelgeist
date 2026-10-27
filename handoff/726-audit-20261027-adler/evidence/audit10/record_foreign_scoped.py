"""Ignored exact foreign-source provenance bookkeeping, no SDK execution."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

r = Path(__file__).parent
n = int(sys.argv[1])
details = json.loads((r/f'successor-dispositions-{n:03}.json').read_text())
c = json.loads((r/'catalog.json').read_text())['rows']
s = json.loads((r.parent/'audit05/original-scope-072.json').read_text(encoding='utf-8-sig'))['files']
state = json.loads((r/'STATE.json').read_text())
prior = set(state['new_original_indices'])
receipts = []
for k,v in details.items():
    i = int(k)
    assert i not in prior
    row = c[i]
    h = row['expected_sha256']
    if row['origin'].startswith('archive:'):
        ref = 'origin/codex/774-audit-handoff-20261025:handoff/774/adler-original/' + row['origin'].split(':',1)[1]
    else:
        assert row['origin'].startswith('local-native:')
        ref = 'origin/codex/774-audit-handoff-20261025:handoff/774/audit/reconcile-adler-20261023/recovered-sources/objects/' + h
    b = subprocess.check_output(['git','show',ref])
    assert hashlib.sha256(b).hexdigest() == h
    assert len(b.decode().splitlines()) == row['line_count']
    assert all(1 <= a <= z <= row['line_count'] for a,z in v['ranges'])
    (r/'successor-source-objects'/h).write_bytes(b)
    receipts.append(dict(scope_index=i,source_id=row['source_id'],sha256=h,
        immutable_ref=ref,ranges=v['ranges'],scope_kind=s[i]['scope_kind'],
        scope_limits=s[i]['scope_limits'],
        source_basis='Fresh personal exact registered foreign native/macro source reads; immutable Git bytes/hash verified, no other-lens verdict coverage.',
        trait_inventory_and_semantics=v['inventory'],disposition=v['disposition'],
        new_finding=False,checks='not run'))
    prior.add(i)
packet = r/f'successor-{n:03}.json'
assert not packet.exists()
packet.write_text(json.dumps(dict(id=f'ADLER-CONT10-SUCCESSOR-{n:03}',lens='derivability',receipts=receipts,clear=False),indent=2)+'\n')
inv = json.loads((r/'inherited-inventory.json').read_text())['rows']
done = prior | {x['scope_index'] for x in inv if x['status'] != 'PENDING_OWN_LENS10_SOURCE_ANALYSIS'}
state.update(new_original_indices=sorted(prior),original_accounted=len(done),next_original=next((i for i in range(state['original_total']) if i not in done),None),latest_packet=packet.name)
(r/'STATE.json').write_text(json.dumps(state,indent=2)+'\n')
print('Saved foreign roles',len(receipts),'accounted',len(done),'next',state['next_original'])
