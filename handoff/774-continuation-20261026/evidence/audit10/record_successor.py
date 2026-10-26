"""Ignored append-only lens10 receipts and moving pointer; no project checks."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).parent
number = int(sys.argv[1])
dispositions = json.loads((root / f'successor-dispositions-{number:03}.json').read_text())
catalog = json.loads((root / 'catalog.json').read_text())['rows']
state = json.loads((root / 'STATE.json').read_text())
prior = set(state['new_original_indices'])
receipts = []
(root / 'successor-source-objects').mkdir(exist_ok=True)
for text_index, values in dispositions.items():
    index = int(text_index)
    assert index not in prior
    row = catalog[index]
    deleted = row.get('source_state') == 'deleted'
    if deleted:
        archive_path = row['origin'].split(':', 1)[1]
        source_ref = 'origin/codex/774-audit-handoff-20261025:handoff/774/adler-original/' + archive_path
        data = subprocess.check_output(['git', 'show', source_ref])
        expected = row['verified_object_sha256']
    else:
        assert row['origin'].startswith('git:')
        revision, path = row['origin'][4:].split(':', 1)
        data = subprocess.check_output(['git', 'show', revision + ':' + path])
        expected = row['expected_sha256']
    assert hashlib.sha256(data).hexdigest() == expected
    assert len(data.decode().splitlines()) == row['line_count']
    (root / 'successor-source-objects' / expected).write_bytes(data)
    receipts.append(dict(scope_index=index, source_id=row['source_id'],
                         sha256=row['expected_sha256'], ranges=[[1, row['line_count']]],
                         source_basis='Fresh recovery_826 personal complete exact frozen-object read; independent own10 comparison.',
                         trait_inventory_and_semantics=values['inventory'],
                         disposition=values['disposition'], new_finding=False, checks='not run'))
    if deleted:
        receipts[-1].update(sha256=None, base_sha256=expected,
                            ranges=[], base_ranges=[[1, row['line_count']]],
                            source_basis='Fresh personal exact recovered-base read and frozen6533 current-path absence; no current-body coverage inferred.',
                            qualification='Deleted original role accounted by absence plus actual relocated owner1612; not restored or double-counted as live source.')
    prior.add(index)
packet_path = root / f'successor-{number:03}.json'
assert not packet_path.exists()
packet_path.write_text(json.dumps(dict(id=f'ADLER-CONT10-SUCCESSOR-{number:03}',
                                     lens='derivability', receipts=receipts, clear=False), indent=2) + '\n')
inventory = json.loads((root / 'inherited-inventory.json').read_text())['rows']
inherited_done = {r['scope_index'] for r in inventory if r['status'] != 'PENDING_OWN_LENS10_SOURCE_ANALYSIS'}
completed = inherited_done | prior
state['owner'] = 'recovery_826 (root delegated after saved successor009)'
state['new_original_indices'] = sorted(prior)
state['original_accounted'] = len(completed)
state['next_original'] = next((i for i in range(state['original_total']) if i not in completed), None)
state['latest_packet'] = packet_path.name
(root / 'STATE.json').write_text(json.dumps(state, indent=2) + '\n')
print('Saved', len(receipts), 'own10 roles;', state['original_accounted'], '/', state['original_total'], 'next', state['next_original'])
