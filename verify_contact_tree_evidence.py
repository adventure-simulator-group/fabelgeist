"""Verify source-evidence identity and record links without running the product."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
contact = json.loads((root / 'ASSESSMENT-L1-CONTACT-GEOMETRY.json').read_text())
tree_doc = json.loads((root / 'ASSESSMENT-L1-TREE-RESIDENCY.json').read_text())

def tree_index(path):
    result = {}
    for line in (root / path).read_text().splitlines():
        head, name = line.split('\t', 1)
        _, _, blob = head.split()
        if name in result:
            raise ValueError('duplicate tree path')
        result[name] = blob
    return result

trees = {contact['current_revision']: tree_index('CURRENT-TREE.tsv'),
         tree_doc['reported_selected_revision']: tree_index('lock-evidence/selected-tree.tsv')}
sources = {}
for doc in (contact, tree_doc):
    for key in ('current_sources', 'reported_selected_sources'):
        for item in doc.get(key, []):
            path = (root / item['preserved_bytes']).resolve()
            if not path.is_relative_to(root):
                raise ValueError('source escapes checkpoint')
            raw = gzip.decompress(path.read_bytes())
            blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
            if (hashlib.sha256(raw).hexdigest() != item['sha256'] or
                    len(raw) != item['bytes'] or len(raw.splitlines()) != item['lines'] or
                    blob != item['git_blob'] or blob != trees[item['revision']].get(item['path'])):
                raise ValueError('source identity mismatch: ' + item['path'])
            if any(not 1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges']):
                raise ValueError('invalid source range')
            sources[(item['revision'], item['path'])] = raw.decode()

current = contact['current_revision']
geometry = sources[(current, 'crates/adventuresim-core/src/combat/contact_geometry.rs')]
server = sources[(current, 'crates/adventuresim-tactical-server/src/combat/melee.rs')]
round_code = sources[(current, 'crates/adventuresim-core/src/autoresolve/melee_round/mod.rs')]
tree = sources[(current, 'crates/adventuresim-tactical-client/src/presentation/obstacles/tree/presentation.rs')]
view = sources[(current, 'crates/adventuresim-tactical-client/src/tactical_scene_viewer/view_specs.rs')]
lod = sources[(current, 'crates/adventuresim-tactical-client/src/presentation/obstacles/tree/lod.rs')]
checks = [(geometry, ('pub struct MeleeContactAtTimeFacts', 'pub effective_reach_metres: f32',
                      '.any(|value| !value.is_finite() || value < 0.0)',
                      'facts.effective_reach_metres <= 0.0')),
          (server, ('resolve_melee_contact_at_time(MeleeContactAtTimeFacts',)),
          (round_code, ('resolve_melee_contact_at_time(MeleeContactAtTimeFacts',)),
          (tree, ('active_mask: u8::MAX', 'const EVERY_TREE_TIER: u8 = 0b11_1111;',
                  '(1 << (lod + 1)) | u8::from(lod < 4)',
                  'selected_leaf != Some(TreeLeafRepresentation::AlphaCard)',
                  'selected_leaf != Some(TreeLeafRepresentation::TexturedMesh)')),
          (view, ('pub render_lod_override: Option<u8>',
                  'pub const fn render_lod(mut self, value: u8)')),
          (lod, ('pub(crate) struct TreeLodRenderOverride', 'pub(crate) lod: Option<u8>'))]
for body, needles in checks:
    if any(needle not in body for needle in needles):
        raise ValueError('expected trace source missing')

baseline = json.loads((root / 'BASELINE-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS.json').read_text())
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for item in contact['historical_source_roles']:
    row = roles[item['original_index']]
    if (row['source_id'] != item['source_id'] or row['historical_sha256'] != item['sha256'] or
            row.get('historical_assessment') != 'ASSESSMENT-L1-CONTACT-GEOMETRY.json' or
            row['status'] != 'full-historical-body-trace-open'):
        raise ValueError('historical role linkage mismatch')
if [x['original_index'] for x in contact['historical_source_roles']] != [81,82,83,84,86,87,88]:
    raise ValueError('unexpected historical roles')
for ident in contact['connected_ledger_entries']:
    row = next(x for x in baseline['repair_ledger']['entries'] if x['id'] == ident)
    if 'ASSESSMENT-L1-CONTACT-GEOMETRY.json' not in row['additional_assessments']:
        raise ValueError('contact ledger linkage mismatch')
if sum(x['id'] == tree_doc['contract_id'] and x['record'] == 'ASSESSMENT-L1-TREE-RESIDENCY.json'
       for x in baseline['repair_ledger']['entries']) != 1:
    raise ValueError('tree ledger linkage mismatch')
if len(baseline['repair_ledger']['entries']) != progress['partial_connected_assessments']:
    raise ValueError('ledger count mismatch')
for path in ('crates/adventuresim-tactical-client/src/presentation/obstacles/tree/presentation.rs',
             'crates/adventuresim-tactical-client/src/tactical_scene_viewer/view_specs.rs'):
    row = next(x for x in baseline['source_delta'] if x['path'] == path)
    if not row['lens_status'][0].startswith('partial'):
        raise ValueError('tree delta disposition mismatch')
print('verified contact and tree source identities and links; product behavior remains unverified')
