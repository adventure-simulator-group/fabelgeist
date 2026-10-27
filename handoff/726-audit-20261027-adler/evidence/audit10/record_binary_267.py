"""Ignored evidence only: frozen PNG derivability inventory, no project checks."""
import hashlib
import json
from pathlib import Path
import struct
import subprocess

r = Path(__file__).parent
c = json.loads((r / 'catalog.json').read_text())['rows']
s = json.loads((r.parent / 'audit05/original-scope-072.json').read_text(encoding='utf-8-sig'))['files']
data = subprocess.check_output(['git', 'show', c[2415]['origin'][4:]])
old = subprocess.check_output(['git', 'show', 'origin/codex/774-audit-handoff-20261025:handoff/774/adler-original/' + c[2416]['origin'].split(':', 1)[1]])
assert data == old
digest = hashlib.sha256(data).hexdigest()
assert digest == c[2415]['expected_sha256'] == c[2416]['verified_object_sha256']
assert data[:8] == b'\x89PNG\r\n\x1a\n'
width, height, depth, colour = struct.unpack('>IIBB', data[16:26])
chunks = []
pos = 8
while pos < len(data):
    n = int.from_bytes(data[pos:pos+4], 'big')
    chunks.append(data[pos+4:pos+8].decode('ascii'))
    pos += 12 + n
absent = subprocess.run(['git', 'cat-file', '-e', '6533e444fa37552927070cc8d8190907fd3b413e:' + c[2416]['path']], capture_output=True)
assert absent.returncode != 0
(r / 'successor-source-objects' / digest).write_bytes(data)
receipts = []
for i in (2415, 2416):
    receipts.append(dict(scope_index=i, source_id=c[i]['source_id'], sha256=c[i]['expected_sha256'],
                         base_sha256=c[i].get('verified_object_sha256'), bytes=len(data),
                         png_header=dict(width=width, height=height, bit_depth=depth, colour_type=colour),
                         png_chunks=chunks, ranges=[], scope_kind=s[i]['scope_kind'],
                         source_basis='Personal fresh exact bytes/hash, PNG header/chunk envelope inventory; old and new bodies byte-identical. Deleted frozen path absence personally verified.',
                         trait_inventory_and_semantics='PNG raster binary contains no Rust handwritten trait or custom source serialization implementation; encoded image data is not an executable codec implementation.',
                         disposition='No derivability candidate; binary/deleted role accounted only for this lens. No pixel/rendering/generation correctness acceptance.',
                         new_finding=False, checks='not run'))
packet = r / 'successor-267.json'
assert not packet.exists()
packet.write_text(json.dumps(dict(id='ADLER-CONT10-SUCCESSOR-267', lens='derivability', receipts=receipts, clear=False), indent=2) + '\n')
state = json.loads((r / 'STATE.json').read_text())
prior = set(state['new_original_indices'])
assert not prior.intersection((2415, 2416))
prior.update((2415, 2416))
inventory = json.loads((r / 'inherited-inventory.json').read_text())['rows']
completed = prior | {row['scope_index'] for row in inventory if row['status'] != 'PENDING_OWN_LENS10_SOURCE_ANALYSIS'}
state.update(new_original_indices=sorted(prior), original_accounted=len(completed),
             next_original=next(i for i in range(state['original_total']) if i not in completed),
             latest_packet=packet.name)
(r / 'STATE.json').write_text(json.dumps(state, indent=2) + '\n')
print(json.dumps(dict(accounted=len(completed), next_original=state['next_original'], png=[width,height,depth,colour], chunks=chunks)))
