"""Verify the preserved armory producer evidence without running product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-ARMORY-SAFEGUARD-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 1 and doc['source_revision'] == revision

def tree(path):
    out = {}
    for line in (root / path).read_text().splitlines():
        header, filename = line.split('\t', 1)
        _, kind, oid = header.split()
        assert filename not in out and kind in ('blob', 'commit')
        out[filename] = oid
    return out

current = tree('main-advance-95e/current-tree.tsv')
previous = tree('main-advance-95e/previous-tree.tsv')
sources = {}
for item in doc['current_sources']:
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    oid = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    assert item['revision'] == revision and oid == item['git_blob']
    assert current[item['path']] == previous[item['path']] == oid
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    assert len(raw) == item['bytes'] and len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 4
p = 'crates/adventuresim-character-creator/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)

must(sources[p + 'equipment_catalog.rs'], ['CatalogItem::all(&self.items).filter(|item| is_wearable(item.definition()))',
     'equipment.material.is_some()', '.any(|placement| !placement.surface.is_empty())'])
must(sources['crates/adventuresim-core/src/equipment_assets/selection.rs'],
     ['pub fn retained_placements(self)', 'let definition = Arc::new((*self.definition).clone());',
      '(0..count).map(move |placement| OwnedCatalogPlacement',
      'placement: &equipment.placements[self.placement.0]'])
must(sources[p + 'armory.rs'], ['self.placements[0].item()', '.wearable()',
     '.retained_placements()', '.filter(|placement| !placement.selection().surface.is_empty())',
     'fitted: placements', 'armory.exhibits = exhibits(&catalog);',
     'exhibit\n            .placements\n            .iter()'])
must(sources[p + 'armory_ui.rs'], ['let retained = exhibit.placements[0].clone();'])
historical = json.loads((root / 'ASSESSMENT-L01-HISTORICAL-691-699.json').read_text())
for reference in doc['historical_sources']:
    role = next(x for x in historical['historical_source_roles'] if x['original_index'] == reference['original_role_index'])
    assert role['source_id'] == reference['source_id'] and role['sha256'] == reference['sha256']
    path = role['source_id'].split(':', 1)[1]
    assert role['sha256'] == hashlib.sha256(sources[path].encode()).hexdigest()
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert name in progress['new_nonfinding_assessments']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['dispositions'][0]['validation'] == 'not performed; source-only evidence verification'
print('verified four Git-bound armory sources and historical identities; no armory or product ran')
