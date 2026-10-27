import hashlib
import json
import pathlib
import subprocess
import sys

p = pathlib.Path(__file__).parent
number = int(sys.argv[1])
raw = (p.parent / 'audit05/raw-overlay-registration-000.jsonl').read_bytes()
assert hashlib.sha256(raw).hexdigest() == 'c26e2def8f8520c870f359ef66919ce730fcddb65beb051365fce42371a7bc68'
registry = [json.loads(x) for x in raw.splitlines()]
catalog = json.loads((p / 'catalog.json').read_text())['rows']
dispositions = json.loads((p / f'overlay-dispositions-{number:03}.json').read_text())
out = p / f'overlay-successor-{number:03}.json'
assert not out.exists()
receipts = []
for key, disposition in dispositions.items():
    index = int(key)
    registration = registry[index]
    candidates = [r for r in catalog if r['source_id'] == registration['source_id'] and r['expected_sha256'] == registration['sha256']]
    assert len(candidates) == 1, (index, candidates)
    origin = candidates[0]['origin']
    if origin.startswith('git:'):
        git_source = origin[4:]
    elif origin.startswith('local-native:'):
        git_source = 'origin/codex/774-audit-handoff-20261025:handoff/774/audit/reconcile-adler-20261023/recovered-sources/objects/' + registration['sha256']
    elif origin.startswith('archive:'):
        git_source = 'origin/codex/774-audit-handoff-20261025:handoff/774/adler-original/' + origin[8:]
    else:
        raise AssertionError((index, origin))
    blob = subprocess.check_output(['git', 'show', git_source])
    assert hashlib.sha256(blob).hexdigest() == registration['sha256']
    objects = p / 'overlay-source-objects'
    objects.mkdir(exist_ok=True)
    (objects / registration['sha256']).write_bytes(blob)
    receipts.append({'overlay_index': index, 'registration': registration, 'source_origin': origin, 'exact_git_source': git_source,
                     'source_sha256': registration['sha256'], 'read_ranges': registration['material_ranges'],
                     'evidence_basis': 'Fresh personal exact registered body comparison by own10; original bodies may support but do not automatically discharge overlay scope.',
                     **disposition})
payload = {'id': f'OWN10-OVERLAY-SUCCESSOR-{number:03}', 'lens': 'derivability',
           'registration_authority': '../audit05/raw-overlay-registration-000.jsonl',
           'registration_sha256': hashlib.sha256(raw).hexdigest(), 'receipts': receipts,
           'checks': 'not run', 'source_edits': False, 'clear': False}
out.write_text(json.dumps(payload, indent=2) + '\n')
state = json.loads((p / 'STATE.json').read_text())
done = set(state.get('completed_overlay_indices', []))
assert not done.intersection(int(k) for k in dispositions)
done.update(int(k) for k in dispositions)
state.update(completed_overlay_indices=sorted(done), overlays_accounted=len(done),
             next_overlay=next((i for i in range(len(registry)) if i not in done), None),
             latest_overlay_packet=out.name)
(p / 'STATE.json').write_text(json.dumps(state, indent=2) + '\n')
print(f'Saved {len(dispositions)} own10 overlay roles; {len(done)}/246 next {state["next_overlay"]}')
