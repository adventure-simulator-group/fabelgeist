"""Verify selected threat-identity source evidence without product execution."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L11-THREAT-ID-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 11 and doc['source_revision'] == revision

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
assert len(sources) == 6
bestiary = sources['crates/adventuresim-core/src/bestiary.rs']
must(bestiary, ['pub fn try_new(value: &str) -> Result<Self, UnknownThreatId>',
                'value.len() > 63', 'byte.is_ascii_lowercase()',
                'impl FromStr for ThreatId', '.monster(value)',
                'pub fn profile(id: ThreatId) -> ThreatProfile',
                '.expect("threat ID was validated against startup catalog")',
                'pub struct ReportDescription(ThreatId);', 'pub struct EvidenceKind(ThreatId);',
                'ThreatId::try_new(value).map(Self)',
                '.expect("validated threat identity")'])
assert bestiary.count('ThreatId::try_new(value).map(Self)') >= 2
sample = 'not_an_authored_threat'
assert 1 <= len(sample.encode()) <= 63 and all(c.isascii() and (c.islower() or c.isdigit() or c in '_-.:') for c in sample)
inv = json.loads((root / doc['catalog_inventory']).read_text())
assert inv['revision'] == revision and inv['monster_count'] == 25 and sample not in inv['monster_ids']
assert inv['sample_id'] == 'small_upright_figures' and inv['sample_is_description'] and not inv['sample_is_monster']
fear = sources['crates/adventuresim-stdb-module/src/condition/fear.rs']
must(fear, ['enemy_type.parse::<adventuresim_core::bestiary::ThreatId>()', 'id.profile()'])
projection = sources['crates/adventuresim-stdb-module/src/investigation/projections.rs']
must(projection, ['.parse::<adventuresim_core::bestiary::ThreatId>()', 'monster_kind: threat.profile().display_name.into()'])
web = sources['crates/strategic-web/src/templates/settlement/travel.rs']
must(web, ['encounter.archetype.parse::<ThreatId>().ok()', 'id.display_name('])
search = doc['search_evidence']; raw = (root / search['path']).read_bytes()
assert len(raw) == search['bytes'] and hashlib.sha256(raw).hexdigest() == search['sha256']
assert all(line.startswith(revision + ':') for line in raw.decode().splitlines())
assert 'condition/fear.rs:4:' in raw.decode()
prior = json.loads((root / 'ASSESSMENT-L1-THREAT-IDENTITY.json').read_text())
assert prior['contract_id'] == 'NEW-THREAT-ID-ADMISSION-001'
assert prior['current_revision'] == 'c100638b720a7281a5fc96f6a3c5c23d5dda85da'
oldbase = json.loads((root / 'BASELINE-20261110.json').read_text())
assert oldbase['repair_ledger']['entries'][26]['id'] == prior['contract_id']
queue = json.loads((root / 'baseline-059/queue.json').read_text())['entries']
assert queue[975]['contract_id'] == 'Q-L12-d3c2d0c28c4cc6667828'
roles = json.loads((root / 'COVERAGE-L01-ROLES.json').read_text())['roles']
for ref in doc['historical_source_roles']:
    role = roles[ref['original_index']]
    assert role['source_id'] == ref['source_id'] and role['historical_sha256'] == ref['historical_sha256']
    assert role['historical_assessment'] == ref['historical_assessment']
baseline = json.loads((root / 'BASELINE-95E-20261110.json').read_text())
progress = json.loads((root / 'CONTINUATION-PROGRESS-95E.json').read_text())
assert len(doc['candidates']) == 1 and doc['candidates'][0]['id'] == prior['contract_id']
assert sum(x['id'] == prior['contract_id'] and x['record'] == name
           for x in baseline['newly_assessed_ledger']['entries']) == 1
assert progress['new_current_ledger_candidates'] == len(baseline['newly_assessed_ledger']['entries'])
assert name in progress['post_transition_bounded_granularity_traces']
recon = json.loads((root / 'RECONCILIATION-059.json').read_text())['entries'][975]
assert recon['contract_id'] == queue[975]['contract_id'] and name in recon['current_reconciliation']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified selected threat identity sources and predecessor linkage; no product ran')
