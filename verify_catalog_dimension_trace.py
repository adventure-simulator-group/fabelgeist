"""Verify catalogue dimension source evidence and arithmetic, without executing the product."""
import gzip
import hashlib
import json
import struct
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L03-CATALOG-DIMENSIONS-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 3 and doc['source_revision'] == revision

def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256'] and len(raw) == item['bytes']
    assert hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == item['git_blob']
    return raw

def must(body, markers):
    for marker in markers:
        if marker not in body:
            raise ValueError('missing source marker: ' + marker)

tree = {}
for line in (root / 'main-advance-95e/current-tree.tsv').read_text().splitlines():
    header, path = line.split('\t', 1)
    _, kind, oid = header.split()
    assert kind in ('blob', 'commit') and path not in tree
    tree[path] = oid
sources = {}
for item in doc['current_sources']:
    raw = preserved(item)
    assert item['revision'] == revision and item['git_blob'] == tree[item['path']]
    assert len(raw.splitlines()) == item['lines']
    assert all(1 <= a <= b <= item['lines'] for a, b in item['ranges'])
    assert item['path'] not in sources
    sources[item['path']] = raw.decode()
assert len(sources) == 12
core = 'crates/adventuresim-core/'
validation = sources[core + 'src/item_catalog_validation.rs']
physical = validation.split('match physical.get("dimensions_m")', 1)[1].split('let grip_to_tip', 1)[0]
must(physical, ['values.len() == 3', '.as_f64()', 'value.is_finite() && value > 0.0'])
assert 'positive_f32_in' not in physical
must(validation, ['serde_json::from_value::<crate::item_catalog_schema::ItemCatalogDocument>',
                  'document.clone()', 'if diagnostics.is_empty()'])
must(sources[core + 'src/item_catalog_validation_numeric.rs'],
     ['pub(super) fn positive_f32_in(', 'let stored = value as f32;',
      '!stored.is_finite() || stored <= 0.0'])
must(sources[core + 'src/item_catalog_schema.rs'],
     ['pub struct EquipmentPhysical {', 'pub dimensions_m: [f32; 3]'])
must(sources[core + 'build.rs'], ['validate_documents_with_sources(&documents, &source_files, &sources)',
                                 'let json = serde_json::to_string(&documents).unwrap()'])
must(sources[core + 'src/item_catalog.rs'], ['serde_json::from_str(ITEM_CATALOG_JSON)',
                                            'equipment.physical.dimensions_m[1] = physical.length_m'])
must(sources['crates/adventuresim-tactical-core/src/inventory.rs'],
     ['impl TacticalEquipmentPhysical {', 'self.dimensions_m.cmpgt(Vec3::ZERO).all()'])
projection = sources['crates/adventuresim-tactical-server/src/player_projection.rs']
must(projection, ['physical.dimensions_m[1] = instance.total_length_m;',
                  'dimensions_m: Vec3::from_array(physical.dimensions_m)'])
equipment = sources['crates/adventuresim-tactical-server/src/equipment.rs']
must(equipment, ['fn drop_hand(', 'physical.copied().filter(|physical| physical.is_valid())',
                 'fn pickup(', '.filter(|physical| physical.is_valid())'])
must(sources['crates/adventuresim-weapon-model/Cargo.toml'], ['features = ["float_roundtrip"]'])

for native in doc['native_sources']:
    manifest_path = root / native['package_manifest']
    manifest = json.loads(manifest_path.read_text())
    archive = manifest_path.parent / manifest['archive']
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == manifest['sha256']
    with tarfile.open(archive) as ar:
        raw = ar.extractfile(native['archive_member']).read()
    assert hashlib.sha256(raw).hexdigest() == native['sha256'] and len(raw) == native['bytes']
    assert all(1 <= a <= b <= len(raw.splitlines()) for a, b in native['ranges'])
    body = raw.decode()
    if native['archive_member'].endswith('/value/de.rs'):
        must(body, ['#[cfg(not(feature = "arbitrary_precision"))]',
                    'Value::Number(n) => n.deserialize_any(visitor)', 'deserialize_number!(deserialize_f32)'])
    elif native['archive_member'].endswith('/number.rs'):
        must(body, ['N::Float(f) => visitor.visit_f64(f)',
                    '#[cfg(not(feature = "arbitrary_precision"))]'])
    elif native['archive_member'].endswith('/de/impls.rs'):
        must(body, ['num_as_copysign_self!(f64:visit_f64)', 'Ok(v as Self::Value)',
                    'Ok((v as Self::Value).copysign(sign))'])
    else:
        must(body, ['#[cfg(feature = "float_roundtrip")]',
                    'lexical::parse_concise_float::<f32>(significand, exponent) as f64'])

# Evidence-only arithmetic and read-only inspection of the pinned authored blob.
assert 1e-100 > 0.0 and struct.unpack('!f', struct.pack('!f', 1e-100))[0] == 0.0
assert sorted(p for p in tree if p.startswith('content/items/') and p.endswith('.yaml')) == ['content/items/catalog.yaml']
catalog = json.loads(sources['content/items/catalog.yaml'])
components = [v for item in catalog['items'] if item.get('equipment')
              for v in item['equipment']['physical']['dimensions_m']]
observation = doc['checked_in_source_observation']
assert len(components) == observation['dimension_components'] == 270
assert min(components) == observation['minimum_m'] == 0.025
knife = next(item for item in catalog['items'] if item['id'] == observation['example_item'])
assert knife['equipment']['physical']['dimensions_m'] == observation['authored_dimensions_m']
assert {'left_hand', 'right_hand'} <= {p['id'] for p in knife['equipment']['placements']}

transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
item = doc['recovered_historical_receipts'][0]
raw = preserved(item)
entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
old = json.loads(raw)
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][281]
assert prior['receipt'] == item['original_path'] and prior['receipt_sha256'] == item['sha256']
assert prior['contract']['proposed_id'] == 'PL13-175-CATALOGUE-NUMERIC-RECEIVER-CLOSURE-PENDING'
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
ref = doc['historical_source_roles'][0]
role = roles[ref['original_index']]
assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
assert ref['current_source_sha256'] != ref['historical_sha256']
assert ref['current_source_sha256'] == next(x['sha256'] for x in doc['current_sources']
                                             if x['path'].endswith('/item_catalog_validation.rs'))
assert role['historical_assessment'] == ref['historical_assessment']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified 12 Git-bound sources, pinned decoder sources, authored values and prior receipt; no product ran')
