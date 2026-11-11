#!/usr/bin/env python3
"""Verify source and pinned Serde identities, without running serialization or product code."""
import gzip
import hashlib
import json
import re
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda p: json.loads((root / p).read_text())
contents = lambda p: gzip.decompress((root / p).read_bytes())
sha = lambda b: hashlib.sha256(b).hexdigest()
def oid(kind, data):
    return hashlib.sha1(kind.encode() + b' ' + str(len(data)).encode() + b'\0' + data).hexdigest()
def entries(data):
    result, pos = {}, 0
    while pos < len(data):
        end = data.index(b'\0', pos)
        mode, name = data[pos:end].split(b' ', 1)
        result[name.decode()] = (mode.decode(), data[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(data)
    return result
def chain(path, proof, body, root_tree, nodes):
    assert [part['name'] for part in proof['chain']] == path.split('/')
    current = root_tree
    for part in proof['chain']:
        info = nodes[current]
        data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == part['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[part['name']] == (part['mode'], part['child'])
        current = part['child']
    assert current == proof['git_blob'] == oid('blob', body)
    assert sha(body) == proof['sha256'] and len(body) == proof['bytes']
    assert len(body.splitlines()) == proof['lines']

rev = '95e63628f9d70b8dd6e6fede64334696c6995009'
oldrev = '6533e444fa37552927070cc8d8190907fd3b413e'
name = 'ASSESSMENT-SEMANTIC-NAMES-95E.json'
hist = 'ASSESSMENT-L01-HISTORICAL-1778-1780.json'
a, h, w = load(name), load(hist), load('COVERAGE-L01-ROLES.json')
proof = load('SEMANTIC-NAMES-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects'])
nodes = dict(shared['tree_objects'])
for path in proof['additional_shared_tree_objects']:
    nodes.update(load(path)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
commit = contents(shared['revisions'][oldrev]['commit']['preserved_bytes'])
assert proof['revision'] == oldrev and oid('commit', commit) == oldrev
assert commit.splitlines()[0] == b'tree ' + proof['root_tree'].encode()
advance = load('MAIN-ADVANCE-95E.json')
tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1)
    tree[path] = header.split()[2]
assert a['source_revision'] == rev and a['repair'] == 'not started' and a['validation'] == 'not performed'
lengths = {1778:1132,1779:63,1780:24}
assert [item['original_index'] for item in h['historical_source_roles']] == list(lengths)
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for item in h['historical_source_roles']:
        i = item['original_index']
        n = lengths[i]
        row = w['roles'][i]
        old = archive.extractfile(item['sha256']).read()
        c = item['current_comparison']
        body = contents(c['preserved_bytes'])
        assert item['prior_role_status'] == 'unassessed' and item['new_personal_read_ranges'] == [[1,n]]
        assert len(old.splitlines()) == n and sha(old) == row['historical_sha256']
        assert row['status'] == 'full-historical-body-trace-open' and row['historical_assessment'] == hist
        assert old == body and c['byte_identical'] is True
        assert c['current_personal_ranges'] == ([[1,n]] if i == 1779 else [])
        assert c['current_content_read_via_identical_historical_ranges'] == [[1,n]]
        assert sha(body) == c['sha256'] and len(body) == c['bytes'] and len(body.splitlines()) == n
        assert oid('blob', body) == c['git_blob'] == tree[c['path']]
        chain(c['path'], proof['paths'][c['path']], old, proof['root_tree'], nodes)
        current[i] = body
sources = {}
for item in a['current_supporting_sources']:
    body = contents(item['preserved_bytes'])
    assert item['revision'] == rev and sha(body) == item['sha256'] and len(body) == item['bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['ranges'])
    assert oid('blob', body) == item['git_blob'] == tree[item['path']]
    sources[item['path']] = body
semantic = current[1778].decode()
assert '#[serde(rename_all = "snake_case")]' in semantic
section = semantic[semantic.index('pub enum SemanticPose {'):]
section = section[:section.index('\n}')]
enum = re.findall(r'^\s*([A-Za-z]\w*),\s*$', section, re.M)
section = semantic[semantic.index('pub const ALL:'):]
allv = re.findall(r'Self::(\w+)', section[:section.index('];')])
section = semantic[semantic.index('pub const HUMANOID_REQUIRED:'):]
required = re.findall(r'Self::(\w+)', section[:section.index('];')])
section = semantic[semantic.index('pub fn as_str'):]
section = section[:section.index('\n    }')]
mappings = dict(re.findall(r'^\s*(\w+) => "([^"]+)"', section, re.M))
assert len(enum) == len(allv) == len(mappings) == 41 and enum == allv
assert len(set(allv)) == len(set(mappings.values())) == 41 and set(enum) == set(mappings)
assert len(required) == 30 and len(set(required)) == 30 and set(required) <= set(enum)
assert mappings['AttackSwing'] == 'swing' and mappings['AttackThrust'] == 'thrust'
assert mappings['AttackOffhand'] == 'offhand' and mappings['AttackOffhandPrepared'] == 'offhand_prepared'
assert '.find(|pose| pose.as_str() == value)' in semantic
assert b'pub pose: SemanticPose' in sources['crates/adventuresim-tactical-core/src/animation/evaluation.rs']
catalog = sources['crates/adventuresim-tactical-client/src/animation/catalog.rs']
assert b'poses: BTreeMap<SemanticPose, PoseAnchor>' in catalog and b'#[cfg(test)]' in catalog
assert b'pose.as_str()' in catalog
cache = sources['crates/adventuresim-tactical-client/src/animation/pose_buffer.rs']
assert b'HashMap<SkeletonFamily, Arc<RigDefinition>>' in cache
assert b'HashMap<(SkeletonFamily, AssetId<AnimationClip>), Arc<BakedClip>>' in cache
assert b'pub struct SkeletonFamily(Cow' in current[1780]
assert b'SettleProgress::from_arithmetic((progress.fraction() + elapsed.seconds() / self.0).min(1.0))' in current[1779]

native = {}
for item in a['native_archive_members']:
    raw = (root / item['archive']).read_bytes()
    assert sha(raw) == item['archive_sha256']
    with tarfile.open(root / item['archive'], 'r:gz') as archive:
        body = archive.extractfile(item['member']).read()
    assert sha(body) == item['member_sha256'] and len(body) == item['member_bytes']
    assert len(body.splitlines()) == item['lines'] and all(1 <= lo <= hi <= item['lines'] for lo, hi in item['personal_ranges'])
    native[item['member']] = body
    assert item['archive_sha256'].encode() in sources['Cargo.lock']
case = native['serde_derive-1.0.229/src/internals/case.rs']
attr = native['serde_derive-1.0.229/src/internals/attr.rs']
ast = native['serde_derive-1.0.229/src/internals/ast.rs']
assert b'("snake_case", SnakeCase)' in case and b"snake.push('_');" in case
assert b'snake.push(ch.to_ascii_lowercase());' in case
assert b'rules.serialize.apply_to_variant' in attr and b'.apply_to_variant(&self.name.deserialize.value)' in attr
assert b'variant.attrs.rename_by_rules(attrs.rename_all_rules());' in ast

transport = json.loads(contents('recovered/original-transport-manifest.json.gz'))
lookup = {x['path']: x for x in transport['entries']}
records = {}
for item in a['recovered_historical_records']:
    original = lookup[item['original_path']]
    body = contents(item['preserved_bytes'])
    assert (item['sha256'], item['bytes'], item['git_blob']) == (original['sha256'], original['bytes'], original['git_blob'])
    assert sha(body) == item['published_sha256'] and len(body) == item['published_bytes']
    assert item['redacted_local_path_occurrences'] or (sha(body) == item['sha256'] and oid('blob', body) == item['git_blob'])
    if item['redacted_local_path_occurrences']:
        assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']] = body
for n, indices in [(574,[1778]), (575,[1779,1780])]:
    path = f'segovb/READ-L12-{n}.json'
    receipt = json.loads(records[path])
    check = json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [row['original_index'] for row in receipt['rows']] == indices
    assert receipt['accepted_unique_count_change'] == 0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256'] == lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256'] == lookup[path]['sha256'] and check['state_sha256'] == lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256'] == lookup[row['registration']]['sha256']
    for x in receipt.get('qualified_supporting_same_lens_reuse', []):
        assert x['registration'] in records and x['registration_sha256'] == lookup[x['registration']]['sha256']
        assert x['receipt'] in records and x['receipt_sha256'] == lookup[x['receipt']]['sha256']
        assert x['fresh_personal_body_reads_this_batch'] == []
q = load('RECONCILIATION-059.json')['entries']
for i in (1453,1454,1455,1456):
    assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert 'ASSESSMENT-LANDING-SETTLE-95E.json' in q[1455]['current_reconciliation']
assert not a['conditional_findings']
ledger = load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
assert load('CONTINUATION-PROGRESS-95E.json')['new_current_ledger_candidates'] == len(ledger)
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision'] == rev
print('Verified three historical roles, prior receipts and pinned Serde/name source identities; no product execution.')
