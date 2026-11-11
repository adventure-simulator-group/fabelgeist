#!/usr/bin/env python3
"""Verify historical source claims and current authority evidence; no product runs."""
import gzip, hashlib, json, tarfile
from pathlib import Path
root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(kind, body):
    return hashlib.sha1(kind.encode() + b' ' + str(len(body)).encode() + b'\0' + body).hexdigest()
def entries(body):
    result, pos = {}, 0
    while pos < len(body):
        end = body.index(b'\0', pos)
        mode, name = body[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), body[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(body)
    return result
def verify_chain(path, proof, body, start, nodes):
    parts = path.split('/')
    assert [x['name'] for x in proof['chain']] == parts[:len(proof['chain'])]
    current = start
    for step in proof['chain']:
        info = nodes[current]
        raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current == step['tree']
        assert sha(raw) == info['sha256'] and len(raw) == info['bytes']
        assert entries(raw)[step['name']] == (step['mode'], step['child'])
        current = step['child']
    if body is None:
        assert proof['present'] is False and current == proof['missing_at_tree']
        info = nodes[current]; raw = contents(info['preserved_bytes'])
        assert oid('tree', raw) == current and sha(raw) == info['sha256'] and len(raw) == info['bytes']
        rest = parts[len(proof['chain']):]
        assert rest == proof['unresolved_suffix'] and rest[0] == proof['missing_component']
        assert rest[0] not in entries(raw)
    else:
        assert proof.get('present', True) and len(proof['chain']) == len(parts)
        assert current == proof['git_blob'] == oid('blob', body)
        assert sha(body) == proof['sha256'] and len(body) == proof['bytes'] and len(body.splitlines()) == proof['lines']
rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
pr = '6533e444fa37552927070cc8d8190907fd3b413e'
claimed = 'f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
matched = '039329518e264f19206cf5b0b98d177433f40c17'
name = 'ASSESSMENT-COMBAT-AUTHORITY-PROVENANCE-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-2027-2029.json'
a, h, work = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load(a['git_membership_evidence'])
shared = load(proof['shared_commit_and_tree_objects']); nodes = dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:
    nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][pr]['commit']['preserved_bytes'])
assert oid('commit', commit) == proof['revision'] == pr
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
tree = {}
for line in (root / load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {2027: 674, 2028: 62, 2029: 54}
assert [x['original_index'] for x in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']; n = lengths[i]; row = work['roles'][i]
        before = archive.extractfile(item['sha256']).read(); c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] in ('unassessed', 'bounded-current-trace-only')
        assert item['prior_current_observations'] == row.get('current_observations', [])
        assert item['new_personal_read_ranges'] == [[1, n]] and len(before.splitlines()) == n
        assert sha(before) == row['historical_sha256'] and before == body
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert c['byte_identical'] and c['current_personal_ranges'] == []
        assert c['current_content_read_via_identical_historical_ranges'] == [[1, n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes']
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        verify_chain(c['path'], proof['paths'][c['path']], before, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
assert all((root / p).is_file() for p in a['prior_current_evidence'])
def walk(x):
    if isinstance(x, dict):
        yield x
        for value in x.values(): yield from walk(value)
    elif isinstance(x, list):
        for value in x: yield from walk(value)
for item in h['historical_source_roles']:
    for prior in item['preexisting_current_support_not_indexed_in_prior_role_row']:
        record = load(prior['assessment'])
        assert any(x.get('path') == item['current_comparison']['path'] and x.get('revision') == rev and x.get('ranges') == prior['ranges'] for x in walk(record)), (item['original_index'], prior)
authority = current[2027]; clock = current[2028]; ranged_authority = current[2029]
ranged = sources['crates/adventuresim-tactical-server/src/combat/ranged.rs']
rules = sources['crates/AGENTS.md']; core_defense = sources['crates/adventuresim-core/src/combat/defense.rs']
assert b'Never use `unwrap` or `expect` outside tests' in rules
assert b'windup: Option<ObservedMeleeWindup>' in authority and b'active: Option<ActiveMeleeAttack>' in authority
assert b'self.windup = Some(ObservedMeleeWindup' in authority and b'self.active = Some(ActiveMeleeAttack' in authority
assert b'.expect("an observed windup has active attack metadata")' in authority
assert b'self.active = None;' in authority and b'self.windup = None;' in authority
assert b'active.started_at <= incoming_started_at' in authority
assert b'(delay.as_secs_f32() <= window).then(||' in authority and b'delay.as_secs_f32() / window' in authority
assert b'target_position: facts.target_position.expect(' in authority and b'target_yaw: facts.target_yaw.expect(' in authority
assert b'ValidatedRangedImpact::Miss' in authority and b'value.is_finite().then_some(Self(value))' in authority
assert b'let target_character = event.target.and_then(|target| q_character.get(target).ok());' in ranged
for needle in (b'separation: target_character.map(', b'target_in_aim_cone: target_character.map(', b'target_position: target_character.map(', b'target_yaw: target_character.map('): assert needle in ranged
assert b'saturating_sub' in clock and b'saturating_add' in clock
assert b'own_contact_after_incoming_seconds.seconds() <= 0.0' in core_defense
assert b'CombatDuration' in ranged_authority
# Original record bytes and attribution are checked against the immutable transport.
transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}; records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]; body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery']
        assert body.count(b'<LOCAL_PATH_') == item['redacted_local_path_occurrences']
    else:
        assert sha(body) == item['sha256'] and oid('blob', body) == item['git_blob']
    records[item['original_path']] = body
for n, delegated in ((5, 7), (7, 9)):
    path = f'segovb/READ-L13-PARALLEL-{n:03d}.json'; data = json.loads(records[path])
    verify = json.loads(records[f'segovb/VERIFY-L13-PARALLEL-{n:03d}.json'])
    state = f'segovb/STATE-L13-PARALLEL-{n:03d}.json'
    assert verify['receipt_sha256'] == lookup[path]['sha256'] and verify['state_sha256'] == lookup[state]['sha256']
    assert 'root adoption is zero personal rereads' in data['attribution']
    dp = f'parallel-l13-20261029/READ-PL13-{delegated:03d}.json'
    assert data['delegated_receipt'] == dp and data['delegated_receipt_sha256'] == lookup[dp]['sha256']
    assert data['delegated_verification_sha256'] == lookup[data['delegated_verification']]['sha256']
    assert json.loads(records[dp]) == data['evidence']
    reg = data['evidence']['registration']; rp = reg['path'].split('726-audit-adler-20261027/', 1)[-1]
    assert rp in records and lookup[rp]['sha256'] == reg['sha256']
    assert data['accepted_unique_count_change'] == 0 and data['current_source_reconciled'] is False
same = json.loads(records['segovb/READ-L13-PARALLEL-005.json'])['evidence']['same_lens_reuse382']
assert same['ranges'] == [[1, 70]] and same['fresh_personal_reads_this_batch'] == []
assert same['prior_ordered_credit'] == 0 and same['proposed_original_credit_once_now'] == 1
assert json.loads(records['parallel-l13-20261029/VERIFY-PL13-006.json'])['original382_partial_only'] == [1, 70]
seg = json.loads(records['segovb/READ-L12-680.json'])
assert [x['original_index'] for x in seg['rows']] == [2027, 2028, 2029]
assert all(x['fresh_personal_body_reads'] == [[1, lengths[x['original_index']]]] for x in seg['rows'])
for n, indices in ((314, [2026, 2027, 2028]), (315, [2029, 2030, 2031, 2032, 2033, 2034])):
    data = json.loads(records[f'adler/READ-L11-{n}.json'])
    assert [x['original_index'] for x in data['sources']] == indices and data['current_reconciled'] is False
# Independently bind all 15 historical recorded identities to both Git trees.
c = load(a['historical_revision_comparison']); assert c['recorded_revision_claim'] == claimed
assert c['older_revision_with_identical_paths_and_bytes'] == matched
for gitrev in (matched, claimed):
    v = c['revisions'][gitrev]; info = v['commit']; raw = contents(info['preserved_bytes'])
    assert oid('commit', raw) == gitrev and sha(raw) == info['sha256'] and len(raw) == info['bytes']
    assert raw.splitlines()[0] == b'tree ' + v['root_tree'].encode()
rows = c['comparisons']; assert [x['original_index'] for x in rows] == [381, 382, 383, 384] + list(range(389, 400))
assert [x['original_index'] for x in rows if not x['claimed_committed_body_matches_recorded']] == [381, 382, 389, 390, 393, 395, 398, 399]
assert [x['original_index'] for x in rows if not x['claimed_committed_body_present']] == [395]
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for row in rows:
        p = row['source_id'].split(':', 1)[1]; before = archive.extractfile(row['recorded_sha256']).read()
        assert sha(before) == row['recorded_sha256'] and len(before.splitlines()) == row['recorded_lines']
        assert row['revision_label_as_recorded'] == 'physical-p16 ' + claimed + ' exact'
        original = json.loads(records[row['original_receipt']]); original_row = next(x for x in original['rows'] if x['original'] == row['original_index'])
        assert original_row['source_id'] == row['source_id'] and original_row['sha256'] == row['recorded_sha256']
        assert original_row['source_revision'] == row['revision_label_as_recorded']
        old = c['revisions'][matched]; new = c['revisions'][claimed]
        verify_chain(p, old['paths'][p], before, old['root_tree'], c['tree_objects'])
        path = new['paths'][p]; body = contents(path['preserved_bytes']) if path['present'] else None
        verify_chain(p, path, body, new['root_tree'], c['tree_objects'])
        assert row['older_committed_body_matches_recorded'] and row['claimed_committed_body_present'] == (body is not None)
        assert row['claimed_committed_body_matches_recorded'] == (body == before if body is not None else False)
        assert row['claimed_committed_sha256'] == (sha(body) if body is not None else None)
        assert row['claimed_committed_lines'] == (len(body.splitlines()) if body is not None else None)
for i in (381, 382, 2689): assert a['historical_revision_comparison'] in work['roles'][i]['revision_comparison_qualification']
for f in ('ASSESSMENT-L01-HISTORICAL-381-384.json', 'ASSESSMENT-L01-RANGED-SOURCE-ASSERTION-95E.json'):
    assert a['historical_revision_comparison'] in load(f)['subsequent_revision_provenance_qualification']
q = load('RECONCILIATION-059.json')['entries']; baseline_q = load('baseline-059/queue.json')['entries']
for i in (1703, 1707, 1708, 1709, 1710):
    assert q[i]['contract_id'] == baseline_q[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert a['conditional_findings'] == ['PL13-ACTIVE-WINDUP-EXPECT', 'PL13-RANGED-GEOMETRY-BUNDLE']
for ident in a['conditional_findings']:
    row = next(x for x in ledger if x['id'] == ident)
    assert row['record'] == name and row['repair'] == 'not started' and row['validation'] == 'not performed'
p = load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates'] == len(ledger) and p['lenses_complete'] == 0 and p['queue_records_semantically_reconciled'] == 0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified 15 original revision claims, both Git trees, original attribution and three current authority source roles; no product execution.')
