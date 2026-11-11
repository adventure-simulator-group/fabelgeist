"""Verify rights validity source and historical attribution without product code."""
import gzip
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent
revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
name = 'ASSESSMENT-L03-RIGHTS-VALIDITY-95E.json'
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
assert len(sources) == 2
rights = sources['crates/adventuresim-core/src/rights.rs']
validity = sources['crates/adventuresim-core/src/rights/validity.rs']
must(validity, ['pub struct RightsValidity {', 'pub valid_from_minute: StrategicMinute,',
                'pub valid_through_minute: Option<StrategicMinute>,',
                'valid_through_minute.is_some_and(|through| through < valid_from_minute)',
                'minute >= self.valid_from_minute', 'through| minute <= through'])
constructor = rights.split('> PermissionGrant<S, R, O, J, G, C>', 1)[1].split('    pub const fn id(&self)', 1)[0]
must(constructor, ['pub fn try_new(', 'validity: RightsValidity,', 'validity,',
                   'RightsIdentityError::InconsistentLifecycle', 'RightsIdentityError::ReceiptProvenanceMismatch'])
assert 'validity.contains(' not in constructor and 'validity.valid_' not in constructor
assessment = rights.split('pub fn assess_permission_grant<', 1)[1].split('pub struct OwnershipEvidence', 1)[0]
must(assessment, ['RightsGrantState::Consumed(receipt)', 'GrantAssessment::IdempotentReplay(receipt.clone())',
                  'RightsGrantState::Active => {', 'current_minute < grant.validity.valid_from_minute',
                  'current_minute > through', 'GrantUnavailable::NotYetValid', 'GrantUnavailable::Expired'])
assert assessment.index('IdempotentReplay') < assessment.index('RightsGrantState::Active =>')
# For any minute m, either m < start, or m >= start > end; there is no valid active instant.
start, end = 100, 90
assert end < start and all((m < start) or (m > end) for m in (0, 90, 99, 100, 101, 2**64 - 1))
search = doc['search_evidence']
raw = (root / search['path']).read_bytes()
assert len(raw) == search['bytes'] and hashlib.sha256(raw).hexdigest() == search['sha256']
for line in raw.decode().splitlines():
    assert line.startswith(revision + ':crates/adventuresim-core/src/rights')
    if any(term in line for term in ('RightsValidity::try_new', 'PermissionGrant::try_new', 'assess_permission_grant(')):
        path, line_number, _ = line[len(revision)+1:].split(':', 2)
        assert path == 'crates/adventuresim-core/src/rights.rs' and int(line_number) > 798
transport = json.loads(gzip.decompress((root / 'recovered/original-transport-manifest.json.gz').read_bytes()))
support = {}
for item in doc['recovered_historical_receipts'] + doc['recovered_historical_support']:
    raw = preserved(item)
    entry = next(x for x in transport['entries'] if x['path'] == item['original_path'])
    assert entry['sha256'] == item['sha256'] and entry['bytes'] == len(raw) and entry['git_blob'] == item['git_blob']
    support[item['original_path']] = (item, json.loads(raw))
assert len(support) == 7
old = json.loads(preserved(doc['recovered_historical_receipts'][0]))
assert old['lens'] == 13 and old['accepted_unique_count_change'] == 0 and old['current_source_reconciled'] is False
assert any('Publicfields permitinvalidinvertedwindows' in x for x in old['evidence']['numeric_role_inventory'])
for ref in (old['delegated_receipt'], old['delegated_verification'], old['agent_validation_manifest'],
            old['evidence']['registration'], *old['evidence']['partial_checkpoints']):
    prefix = 'target/726-audit-adler-20261027/'
    assert ref['path'].startswith(prefix)
    assert support[ref['path'][len(prefix):]][0]['sha256'] == ref['sha256']
ledger = json.loads((root / 'baseline-059/ledger.json').read_text())
prior = ledger['segovb_continuation']['lens13']['discovery_receipts'][195]
assert prior['path'] == doc['recovered_historical_receipts'][0]['original_path']
assert prior['sha256'] == doc['recovered_historical_receipts'][0]['sha256']
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
print('verified rights source, bounded search, active-window reasoning and 7 historical records; no product ran')
