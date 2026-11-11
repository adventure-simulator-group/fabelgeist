"""Verify selected serialization-derive evidence without product execution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L10-REPORT-SERIALIZE-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 10 and doc['source_revision'] == revision

def preserved(item):
    path = (root / item['preserved_bytes']).resolve()
    assert path.is_relative_to(root) and item['content_encoding'] == 'gzip'
    raw = gzip.decompress(path.read_bytes())
    assert len(raw) == item['bytes'] and hashlib.sha256(raw).hexdigest() == item['sha256']
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
assert len(sources) == 13
bestiary = sources['crates/adventuresim-core/src/bestiary.rs']
must(bestiary, ['pub struct ReportDescription(ThreatId);',
                'ThreatId::try_new(value).map(Self)',
                'impl Serialize for ReportDescription',
                'serializer.serialize_str(self.as_str())',
                'impl Serialize for ThreatId',
                'impl FromStr for ThreatId',
                '.monster(value)',
                'Self::try_new(&String::deserialize(deserializer)?)',
                'pub const SmallUprightFigures: Self = Self(ThreatId::from_static("small_upright_figures"));'])
root_cargo = sources['Cargo.toml']
must(root_cargo, ['serde = { version = "1", default-features = false, features = [', '  "derive",'])
must(sources['crates/adventuresim-core/Cargo.toml'], ['serde.workspace = true'])
lock = sources['Cargo.lock']
must(lock, ['name = "serde_derive"\nversion = "1.0.229"',
            'checksum = "e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348"'])
macro = doc['native_macro_source']; archive_path = root / macro['archive']; raw = archive_path.read_bytes()
assert len(raw) == macro['archive_bytes'] and hashlib.sha256(raw).hexdigest() == macro['archive_sha256']
with tarfile.open(archive_path, 'r:gz') as package:
    member = package.extractfile(macro['member']).read()
    check = package.extractfile(macro['eligibility_source']['member']).read()
assert len(member) == macro['member_bytes'] and hashlib.sha256(member).hexdigest() == macro['member_sha256']
assert len(check) == macro['eligibility_source']['bytes'] and hashlib.sha256(check).hexdigest() == macro['eligibility_source']['sha256']
must(member.decode(), ['if cont.attrs.transparent()', 'serialize_transparent(cont, params)',
                       '_serde::Serialize::serialize', '#path(&#self_var.#member, __serializer)'])
must(check.decode(), ['fn check_transparent(', 'Data::Struct(_, fields) => fields',
                      'Derive::Serialize => !field.attrs.skip_serializing()'])
must(sources['crates/adventuresim-core/build.rs'], ['let content = root.join("content/quests")',
                                                    'fs::read_dir(&content)', 'files.sort()'])
quest = sources['crates/adventuresim-core/src/quest_catalog.rs']
must(quest, ['for monster in &document.monsters', 'pub fn monster(&self, id: &str)', 'self.monsters.get(id)'])
catalog = doc['catalog_inventory']; raw = (root / catalog['path']).read_bytes()
assert len(raw) == catalog['bytes'] and hashlib.sha256(raw).hexdigest() == catalog['sha256']
inv = json.loads(raw)
paths = sorted(path for path in tree if path.startswith('content/quests/'))
assert paths == inv['files'] and all(path.endswith('.yaml') for path in paths)
docs = [json.loads(sources[path]) for path in paths]
monsters = [m['id'] for x in docs for m in x.get('monsters', [])]
descriptions = [d['id'] for x in docs for d in x.get('descriptions', [])]
assert monsters == inv['monster_ids'] and len(monsters) == inv['monster_count']
assert len(descriptions) == inv['description_count'] and inv['sample_id'] in descriptions and inv['sample_id'] not in monsters
selection = sources['crates/adventuresim-core/src/equipment_assets/selection.rs']
must(selection, ['std::ptr::eq(self.definition, other.definition)',
                 'std::ptr::eq(self.placement, other.placement)', 'std::ptr::hash(self.definition, state)'])
must(sources['crates/adventuresim-core/src/survival.rs'], ['Self::Field(FieldShelter::default())'])
must(sources['crates/adventuresim-core/src/settlement_property.rs'], ['Self(HousingTier::ALL.to_vec())'])
must(sources['crates/adventuresim-core/src/strategic_schedule.rs'], ['dodge: 1.0,', 'balance: 1.0,', 'will: 1.0,'])
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
prior = json.loads((root / 'ASSESSMENT-L1-THREAT-IDENTITY.json').read_text())
assert prior['current_revision'] == 'c100638b720a7281a5fc96f6a3c5c23d5dda85da'
assert any('ReportDescription and EvidenceKind' in line for line in prior['trace'])
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1
assert sum(x['id'] == doc['candidates'][0]['id'] and x['record'] == name
           for x in baseline['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(baseline['newly_assessed_ledger']['entries'])
assert name in progress['post_transition_bounded_derivability_traces']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified selected derive source and static catalog inventory; no product ran')
