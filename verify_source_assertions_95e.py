#!/usr/bin/env python3
"""Verify preserved evidence and source selectors, never execute product tests."""
import gzip
import hashlib
import json
import re
import tarfile
from pathlib import Path

r = Path(__file__).resolve().parent
load = lambda p: json.loads((r / p).read_text())
sha = lambda b: hashlib.sha256(b).hexdigest()
blob = lambda b: hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
base = 'crates/adventuresim-strategic-sim/src/live_core/'
a = load('ASSESSMENT-SOURCE-ASSERTIONS-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1293-1294.json')
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
assert a['new_candidates'] == []
work = load('COVERAGE-L01-ROLES.json')
tree = {}
for line in (r / work['current_tree_inventory']).read_text().splitlines():
    header, path = line.split('\t', 1)
    mode, kind, oid = header.split()
    tree[path] = oid
sources = {}
for item in a['current_sources']:
    raw = gzip.decompress((r / item['preserved_bytes']).read_bytes())
    assert item['revision'] == rev and len(raw) == item['bytes'] and sha(raw) == item['sha256']
    assert blob(raw) == item['git_blob'] == tree[item['path']] and len(raw.splitlines()) == item['lines']
    assert all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    sources[item['path']] = raw
for item in h['historical_source_roles']:
    role = work['roles'][item['original_index']]
    assert role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1293-1294.json'
    assert role['status'] == 'full-historical-body-trace-open'
    assert item['new_personal_read_ranges'] == [[1, item['lines']]]
    with tarfile.open(r / item['evidence_archive'], 'r:gz') as archive:
        raw = archive.extractfile(item['sha256']).read()
    assert sha(raw) == item['sha256'] and len(raw.splitlines()) == item['lines']
    assert raw == sources[item['source_id'].split(':', 1)[1]]
assert [x['original_index'] for x in h['historical_source_roles']] == [1293, 1294]
assert [x['lines'] for x in h['historical_source_roles']] == [812, 1049]

reconstructed = a['reconstructed_live_core_source']
joined = gzip.decompress((r / reconstructed['path']).read_bytes())
assert len(joined) == reconstructed['bytes'] == 744636 and sha(joined) == reconstructed['sha256']
mod = sources[base + 'mod.rs']
concat = mod.split(b'pub(crate) const LIVE_CORE_SOURCE: &str = concat!(', 1)[1].split(b'\n);', 1)[0]
names = [base + n.decode() for n in re.findall(rb'include_str!\("([^"\n]+)"\)', concat)]
assert names == [x['path'] for x in reconstructed['segments']] and len(names) == 26
assert base + 'schema_types.rs' not in names
cursor = 0
for item in reconstructed['segments']:
    assert item['offset'] == cursor
    raw = joined[cursor:cursor + item['bytes']]
    assert len(raw) == item['bytes'] and sha(raw) == item['sha256']
    assert blob(raw) == item['git_blob'] == tree[item['path']]
    cursor += item['bytes']
assert cursor == len(joined)
prefix = joined.split(b'#[cfg(test)]', 1)[0]
assert len(prefix) == reconstructed['production_prefix_bytes'] == 744284
assert sha(prefix) == reconstructed['production_prefix_sha256']
assert joined[len(prefix):].startswith(b'#[cfg(test)]')
assert prefix.count(b'for cycle in 0..config.cycles') == 1
cycle = prefix.split(b'for cycle in 0..config.cycles', 1)[1]
assert len(cycle) == reconstructed['cycle_tail_bytes'] == 29876 and sha(cycle) == reconstructed['cycle_tail_sha256']
def offsets(needle):
    return [m.start() for m in re.finditer(re.escape(needle), cycle)]
assert offsets(b'recover_or_evacuate_off_settlement') == reconstructed['cycle_recovery_offsets'] == [2245, 17696]
assert offsets(b'QuestDecision') == reconstructed['cycle_quest_decision_offsets'] == [11438, 11498]
recovery_test = sources[base + 'tests/recovery_and_cases.rs']
assert b'assert!(cycle.find("recover_or_evacuate_off_settlement") < cycle.find("QuestDecision"));' in recovery_test
assert b'cycle.find("recover_or_evacuate_off_settlement").unwrap()' in recovery_test
option = gzip.decompress((r / a['option_ordering']['source']).read_bytes())
assert sha(option) == '64206926907928f03c3e7a18fe44022207366e6ecffe5e5d2dd02e5171f2f888'
assert b'(None, Some(_)) => Some(cmp::Ordering::Less)' in option
assert b'(None, None) => Some(cmp::Ordering::Equal)' in option
failure_test = sources[base + 'tests/failure_security.rs']
assert b'fn failure_artifact_version_nine_has_an_exact_full_wire_golden' in failure_test
assert b'let artifact = CoreLoopFailureArtifact {' in failure_test
assert b'include_str!("../../../fixtures/core-loop-failure-v9.json")' in failure_test
assert b'.split("fn advance_generated_case_inner")' in failure_test
assert b'tail.split("pub(super) fn turn_in_ready_direct_contract")' in failure_test
entry = next(x for x in reconstructed['segments'] if x['path'] == base + 'generated_cases.rs')
generated = joined[entry['offset']:entry['offset'] + entry['bytes']]
start, end = b'fn advance_generated_case_inner', b'pub(super) fn turn_in_ready_direct_contract'
assert generated.count(start) == generated.count(end) == 1 and generated.index(start) < generated.index(end)
assert generated[:generated.index(end)].endswith(b'\n    }\n\n    ')

manifest = json.loads(gzip.decompress((r / 'recovered/original-transport-manifest.json.gz').read_bytes()))
lookup = {x['path']: x for x in manifest['entries']}
records = {}
for item in a['recovered_historical_records']:
    orig = lookup[item['original_path']]
    assert item['sha256'] == orig['sha256'] and item['bytes'] == orig['bytes'] and item['git_blob'] == orig['git_blob']
    raw = gzip.decompress((r / item['preserved_bytes']).read_bytes())
    assert len(raw) == item['published_bytes'] and sha(raw) == item['published_sha256']
    assert item['redacted_local_path_occurrences'] or (sha(raw) == item['sha256'] and blob(raw) == item['git_blob'])
    records[item['original_path']] = json.loads(raw)
receipt = records['segovb/READ-L12-345.json']
assert receipt['lens'] == 12 and receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
assert receipt['rows'][0]['original_index'] == 1293 and receipt['rows'][0]['fresh_personal_body_reads'] == [[1,812]]
assert records['segovb/VERIFY-L12-345.json']['receipt_sha256'] == lookup['segovb/READ-L12-345.json']['sha256']
earlier = load('ASSESSMENT-EXPEDITION-DIALOGUE-RULES-95E.json')
old = next(x for x in earlier['recovered_historical_records'] if x['original_path'] == 'segovb/READ-L12-346.json')
raw = gzip.decompress((r / old['preserved_bytes']).read_bytes())
assert sha(raw) == old['published_sha256'] and old['sha256'] == lookup[old['original_path']]['sha256']
old_receipt = json.loads(raw)
assert old_receipt['rows'][0]['original_index'] == 1294 and old_receipt['rows'][0]['fresh_personal_body_reads'] == [[1,1049]]
assert old_receipt['accepted_unique_count_change'] == 0 and old_receipt['current_source_reconciled'] is False
assert any('LocalOptionfind' in s and 'unwrap341' in s for s in old_receipt['rows'][0]['function_contract']['string_inventory'])
q = load('RECONCILIATION-059.json')['entries']
assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (253, 1055, 1078, 1084, 1085))
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert sum(x['id'] == 'SEGOVB-L12-SURRENDER-SOURCE-TEST-END-MARKER' for x in ledger) == 1
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
print('Verified two historical source reads, earlier attribution and source-concat selectors; no product execution.')
