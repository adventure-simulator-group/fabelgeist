"""Verify the pinned sampler, source chain and historical records without product code."""
import gzip
import hashlib
import json
import tarfile
import tomllib
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L04-FOLLOWUP-WEIGHT-TOTAL-95E.json'
doc = json.loads((root / name).read_text())
assert doc['lens'] == 4 and doc['source_revision'] == revision

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
assert len(sources) == 15
core = 'crates/adventuresim-core/'
module = 'crates/adventuresim-stdb-module/src/'
validation = sources[core + 'src/quest_catalog_validation.rs']
must(validation, ['"plausibility",\n                                0,\n                                u32::MAX.into()',
                  '"curation", 0, u32::MAX.into()',
                  '(p == 0 || c == 0) != zero_reason.is_some()',
                  'evidence relations cannot require bridges'])
model = sources[core + 'src/quest_generation/model.rs']
must(model, ['pub fn combined(self) -> u64 {', 'u64::from(self.plausibility) * u64::from(self.curation)'])
projection = sources[core + 'src/quest_generation/projection.rs']
must(projection, ['fn evidence_candidates(cause: CanonicalCause, site: SiteKind)',
                  '.relation("evidence.baseline")',
                  'matches!(site, SiteKind::Roadside | SiteKind::Riverside)',
                  '"evidence.skeleton"', '"evidence.incidental_loss"',
                  '"evidence.fabricated_claim"', '"evidence.trackable_ground"',
                  'pub fn select_follow_up_evidence(', 'if indices.is_empty() { return None; }',
                  '.weighted_index(&weights).expect("bounded catalog weights have positive non-overflowing total")'])
sampling = sources[core + 'src/quest_generation/sampling.rs']
must(sampling, ['.weighted_index(&weights)', '.map_err(GenerationError::Sampling)?',
                '.weighted_order(&weights)'])
assembly = sources[core + 'src/quest_generation/assembly.rs']
must(assembly, ['kind: if family == TemplateFamily::RecurringDepredation {',
                'SiteKind::Roadside', 'let (evidence_kind, evidence_bridge) = choose('])
local = sources[module + 'local_problem.rs']
must(local, ['let sites = incident_draws::sites(&validated.manifest);',
             'let site = sites[draws.index(incident_draws::SITE, sites.len())];',
             'quest_generation::select_follow_up_evidence(',
             'validated.manifest.cause,\n                site.kind,'])
must(sources[module + 'local_problem/incident_draws.rs'],
     ['pub(super) fn sites(case: &GeneratedCase)', 'let mut sites = case.sites.iter().collect::<Vec<_>>()',
      'sites.sort_by(|a, b| a.id.0.cmp(&b.id.0));'])
stream = sources['crates/fabelgeist-determinism/src/stream.rs']
must(stream, ['WeightedIndex::<u64>::new(weights)', 'Error::Overflow => SamplingError::WeightOverflow'])
must(sources['crates/fabelgeist-determinism/Cargo.toml'],
     ['rand = { version = "=0.9.5", default-features = false, features = ["alloc"] }'])
must(sources[core + 'build.rs'], ['quest_catalog_validation::validate_documents(&documents, &source_files)',
                                 'let json = serde_json::to_string(&documents).unwrap()'])
manifest_path = root / doc['native_manifest']
manifest = json.loads(manifest_path.read_text())
archive = manifest_path.parent / manifest['archive']
assert len(archive.read_bytes()) == manifest['bytes']
assert hashlib.sha256(archive.read_bytes()).hexdigest() == manifest['sha256']
lock = tomllib.loads(sources['Cargo.lock'])
locked = next(p for p in lock['package'] if p['name'] == 'rand' and p['version'] == '0.9.5')
assert locked['checksum'] == manifest['sha256']
with tarfile.open(archive) as ar:
    bodies = {}
    for entry in manifest['inspected_members']:
        raw = ar.extractfile(entry['path']).read()
        assert len(raw) == entry['bytes'] and hashlib.sha256(raw).hexdigest() == entry['sha256']
        bodies[entry['path']] = raw.decode()
must(bodies['rand-0.9.5/src/distr/weighted/weighted_index.rs'],
     ['total_weight.checked_add_assign(w.borrow())', 'return Err(Error::Overflow);'])
must(bodies['rand-0.9.5/src/distr/weighted/mod.rs'],
     ['match self.checked_add(*v)', 'impl_weight_int!(u8, u16, u32, u64, u128, usize)'])
max_u32 = 2**32 - 1
product = max_u32 * max_u32
assert product == 18446744065119617025 <= 2**64 - 1
assert 2 * product == 36893488130239234050 > 2**64 - 1
paths = sorted(p for p in tree if p.startswith('content/quests/') and p.endswith('.yaml'))
assert paths == ['content/quests/bestiary.yaml', 'content/quests/generation.yaml', 'content/quests/investigation.yaml']
documents = [json.loads(sources[path]) for path in paths]
relations = {relation['id']: relation for document in documents for relation in document.get('relations', [])}
names = ['evidence.baseline', 'evidence.skeleton', 'evidence.incidental_loss',
         'evidence.fabricated_claim', 'evidence.trackable_ground']
base = relations[names[0]]['candidates']
assert len(base) == 7 and {'footprints', 'cloth_scrap'} <= {candidate['id'] for candidate in base}
assert [c['id'] for c in relations['evidence.trackable_ground']['candidates']] == ['footprints']
assert [c['id'] for c in relations['evidence.skeleton']['candidates']] == ['bone_dust']
bound = 0
for candidate in base:
    options = [candidate] + [other for rel in names[1:] for other in relations[rel]['candidates']
                             if other['id'] == candidate['id']]
    bound += max(other['plausibility'] * other['curation'] for other in options)
assert bound == 26950 < 2**64 - 1
assert any(c['id'] == 'skeleton' and c['plausibility'] > 0 and c['curation'] > 0
           for c in relations['cause.recurring_depredation']['candidates'])
assert any(c['id'] == 'crypt' and c['plausibility'] > 0 and c['curation'] > 0
           for c in relations['site.skeleton']['candidates'])
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 9
for item in doc['recovered_historical_receipts']:
    old = json.loads(preserved(item))
    assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
    for ref in (old['delegated_receipt'], old['delegated_verification'],
                old['agent_validation_manifest'], old['evidence']['registration']):
        prefix = 'target/726-audit-adler-20261027/'
        assert ref['path'].startswith(prefix)
        assert support[ref['path'][len(prefix):]][0]['sha256'] == ref['sha256']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
for index, item in zip([294, 295], doc['recovered_historical_receipts']):
    prior = ledger['segovb_continuation']['lens13']['delegated_connected_contracts_pending_alias'][index]
    assert prior['receipt'] == item['original_path'] and prior['receipt_sha256'] == item['sha256']
    assert prior['contract']['proposed_id'] == 'PL13-189-FOLLOWUP-WEIGHT-TOTAL-NATIVE-PENDING'
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
assert name in progress['post_transition_bounded_operations_traces']
assert progress['queue_records_semantically_reconciled'] == 0
assert doc['repair'] == 'not started' and doc['validation'] == 'not performed'
print('verified 15 Git-bound sources, pinned Rand, authored upper bound and 9 historical records; no product ran')
