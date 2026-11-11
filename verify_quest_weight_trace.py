"""Verify quest mass source and prior evidence without executing product code."""
import gzip
import hashlib
import json
import struct
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L03-QUEST-WEIGHT-NARROWING-95E.json'
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
assert len(sources) == 11
core = 'crates/adventuresim-core/'
validation = sources[core + 'src/quest_catalog_validation.rs']
must(validation, ['let weight = combat', '.and_then(Value::as_f64)',
                  'if !weight.is_finite() || weight <= 0.0 || weight > f64::from(f32::MAX)'])
quest = sources[core + 'src/quest_catalog.rs']
must(quest, ['pub struct MonsterCombat {', 'pub weight_kg: f32,',
             'fn validate_monster(monster: &Monster)', 'monster.combat.weight_kg <= 0.0',
             'serde_json::from_value(value).expect("strict validated catalog schema")',
             'Catalog::compile(documents).expect("validated quest catalog references")',
             'static CATALOG: OnceLock<Catalog>'])
build = sources[core + 'build.rs']
must(build, ['item_catalog_validation::parse_document(&text)',
             'quest_catalog_validation::validate_documents(&documents, &source_files)',
             'let json = serde_json::to_string(&documents).unwrap()'])
must(sources[core + 'src/item_catalog_validation.rs'],
     ['pub fn parse_document(text: &str)', 'serde_json::from_str::<StrictValue>(text)',
      'fn visit_f64<E: de::Error>(self, value: f64)', 'serde_json::Number::from_f64(value)'])
must(sources[core + 'src/quest_generation/solver.rs'],
     ['fn description_candidates(cause: CanonicalCause)', 'crate::quest_catalog::catalog()'])
must(sources['crates/adventuresim-stdb-module/src/local_problem.rs'],
     ['fn incident_circumstance_label(', 'adventuresim_core::quest_catalog::catalog()'])
must(sources['Cargo.toml'], ['serde_json = "1"'])
must(sources[core + 'Cargo.toml'], ['serde_json.workspace = true'])
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
        must(body, ['N::Float(f) => visitor.visit_f64(f)', '#[cfg(not(feature = "arbitrary_precision"))]'])
    elif native['archive_member'].endswith('/de/impls.rs'):
        must(body, ['num_as_copysign_self!(f64:visit_f64)', 'Ok(v as Self::Value)',
                    'Ok((v as Self::Value).copysign(sign))'])
    else:
        must(body, ['#[cfg(feature = "float_roundtrip")]',
                    'lexical::parse_concise_float::<f32>(significand, exponent) as f64'])
assert 1e-100 > 0.0 and struct.unpack('!f', struct.pack('!f', 1e-100))[0] == 0.0
paths = sorted(p for p in tree if p.startswith('content/quests/') and p.endswith('.yaml'))
assert paths == ['content/quests/bestiary.yaml', 'content/quests/generation.yaml', 'content/quests/investigation.yaml']
monsters = [monster for path in paths for monster in json.loads(sources[path]).get('monsters', [])]
assert len(monsters) == 25
weights = [(monster['id'], monster['combat']['weight_kg']) for monster in monsters]
assert min(weights, key=lambda value: value[1]) == ('skeleton', 18.0)
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
all_records = doc['recovered_historical_receipts'] + doc['recovered_historical_support']
support = {}
for item in all_records:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 5
old = json.loads(preserved(doc['recovered_historical_receipts'][0]))
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
for ref in (old['delegated_receipt'], old['delegated_verification'],
            old['agent_validation_manifest'], old['evidence']['registration']):
    prefix = 'target/726-audit-adler-20261027/'
    assert ref['path'].startswith(prefix)
    assert support[ref['path'][len(prefix):]][0]['sha256'] == ref['sha256']
assert any('Subnormalunderflow' in value for value in old['evidence']['numeric_role_inventory'])
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][291]
assert prior['receipt'] == doc['recovered_historical_receipts'][0]['original_path']
assert prior['receipt_sha256'] == doc['recovered_historical_receipts'][0]['sha256']
assert prior['contract']['proposed_id'] == 'PL13-186-QUEST-SAMPLING-AND-NATIVE-PENDING'
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
base = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in base['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(base['newly_assessed_ledger']['entries'])
assert name in progress['post_transition_bounded_admission_traces']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified 11 Git-bound sources, native decoder evidence, authored weights and 5 historical records; no product ran')
