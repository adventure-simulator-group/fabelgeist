"""Verify preserved clothing topology evidence without invoking the product."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L01-CLOTHING-REFIT-95E.json'
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
assert len(sources) == 8
p = 'crates/adventuresim-character-creator/src/'
def must(source, markers):
    for marker in markers:
        if marker not in source:
            raise ValueError('missing source marker: ' + marker)

must(sources[p + 'lib.rs'], ['pub mod clothing;'])
clothing = sources[p + 'clothing.rs']
must(clothing, ['pub struct ClothingShell {', 'pub positions: Vec<[f32; 3]>',
     'pub faces: Vec<[u32; 3]>', 'if faces\n        .iter()\n        .flatten()',
     'return Err(ClothingError::MissingBodyVertex);',
     'let mut neighbors = vec![Vec::new(); positions.len()];',
     'neighbors[a as usize].push(b as usize);', 'neighbors[b as usize].push(a as usize);'])
fit = sources[p + 'clothing/fitting.rs']
must(fit, ['pub fn refit(', 'positions.len() != self.positions.len()',
     'normals.len() != positions.len()', 'let (positions, normals) = fitted_surface(',
     'let faces = validated_placeholder_faces(', 'let relaxed = relax_concavities(positions, normals, faces);'])
assert fit.index('let (positions, normals) = fitted_surface(') < fit.index('let faces = validated_placeholder_faces(')
must(sources[p + 'clothing/validation.rs'], ['*vertex as usize >= positions.len()',
     'ClothingError::OutOfRangeFace'])
must(sources[p + 'clothing/error.rs'], ['OutOfRangeFace {', 'face: ClothingFaceContext'])
must(sources[p + 'outfit.rs'], ['generate_clothing_shells('])
must(sources[p + 'character_export.rs'], ['let clothed = outfit::clothing(model, &loadout, &generated)?;',
     'let clothing_morphs = morphs.clothing(&clothed.shells)?;'])
must(sources[p + 'character_morphs.rs'], ['.refit(&sample.positions, &sample.normals)'])
face = [0, 1, 3]
assert len([0, 0, 0]) == 3 and face[2] >= 3
receipts = {}
for item in doc['recovered_historical_receipts']:
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert hashlib.sha256(raw).hexdigest() == item['sha256']
    receipts[item['original_path']] = json.loads(raw)
assert len(receipts) == 3
assert receipts['segovb/VERIFY-L11-540.json']['receipt_sha256'] == '07999ee136d7c583e46b42246c533778bb1e8d9efe6572923bd0191e3c19edbd'
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
queue = json.loads((root / 'baseline-059/queue.json').read_text())
assert queue['entries'][573]['contract_id'] == doc['historical_records'][0]['id']
old = ledger['segovb_continuation']['current_connected_dispositions'][40]
assert old['disposition']['id'] == doc['historical_records'][1]['id']
contract = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][155]
assert contract['contract']['contract'] == doc['historical_records'][2]['id']
assert not old['latest_current_target_reconciled'] and old['disposition']['accepted_unique_count_change'] == 0
work = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())
for reference in doc['historical_source_roles']:
    role = work['roles'][reference['original_index']]
    assert role['source_id'] == reference['source_id'] and role['historical_sha256'] == reference['historical_sha256']
    assert reference['byte_identical'] == (reference['historical_sha256'] == reference['current_sha256'])
recon = json.loads((root / 'RECONCILIATION-059.json').read_text())
match = [x for x in recon['entries'] if x['source_pointer'] == '/entries/573']
assert len(match) == 1 and match[0]['contract_id'] == queue['entries'][573]['contract_id']
assert match[0]['current_reconciliation'].startswith('partial current 95e') and name in match[0]['current_reconciliation']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
candidate = doc['candidates'][0]
entries = base['newly_assessed_ledger']['entries']
assert sum(x['id'] == candidate['id'] and x['record'] == name for x in entries) == 1
assert progress['new_current_ledger_candidates'] == len(entries) and progress['queue_records_semantically_reconciled'] == 0
assert candidate['discovery'].startswith('partial') and candidate['repair'] == 'not started' and candidate['validation'] == 'not performed'
print('verified eight current sources and three recovered receipts; no refit or product ran')
