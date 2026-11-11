#!/usr/bin/env python3
"""Verify historical sky evidence and symbolic integer-rounding provenance only."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
load = lambda path: json.loads((root / path).read_text())
contents = lambda path: gzip.decompress((root / path).read_bytes())
sha = lambda value: hashlib.sha256(value).hexdigest()

def oid(kind, value):
    return hashlib.sha1(kind.encode() + b' ' + str(len(value)).encode() + b'\0' + value).hexdigest()

def entries(value):
    out, pos = {}, 0
    while pos < len(value):
        end = value.index(b'\0', pos)
        mode, name = value[pos:end].split(b' ', 1)
        out[name.decode()] = (mode.decode(), value[end + 1:end + 21].hex())
        pos = end + 21
    assert pos == len(value)
    return out

revision = '95e63628f9d70b8dd6e6fede64334696c6995009'
a = load('ASSESSMENT-SKY-TIME-STRATEGIC-95E.json')
h = load('ASSESSMENT-L01-HISTORICAL-1623-1625.json')
w = load('COVERAGE-L01-ROLES.json')
proof = load('SKY-TIME-HISTORICAL-GIT-PROOFS.json')
shared = load(proof['shared_commit_and_tree_objects']); nodes = dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:
    nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
advance = load('MAIN-ADVANCE-95E.json'); tree = {}
for line in (root / advance['trees']['current']['path']).read_text().splitlines():
    header, path = line.split('\t', 1); mode, kind, hash_ = header.split(); tree[path] = hash_
assert a['source_revision'] == revision and a['repair'] == 'not started' and a['validation'] == 'not performed'
commit = shared['revisions'][proof['revision']]['commit']; raw = contents(commit['preserved_bytes'])
assert oid('commit', raw) == proof['revision'] and raw.splitlines()[0] == b'tree ' + proof['root_tree'].encode()

def check_chain(path, record, body):
    assert [step['name'] for step in record['chain']] == path.split('/')
    current = proof['root_tree']
    for step in record['chain']:
        info = nodes[current]; data = contents(info['preserved_bytes'])
        assert oid('tree', data) == current == step['tree']
        assert sha(data) == info['sha256'] and len(data) == info['bytes']
        assert entries(data)[step['name']] == (step['mode'], step['child'])
        current = step['child']
    assert current == record['git_blob'] == oid('blob', body)
    assert record['sha256'] == sha(body) and record['bytes'] == len(body)

assert [x['original_index'] for x in h['historical_source_roles']] == [1623,1624,1625]
assert [x['lines'] for x in h['historical_source_roles']] == [32,827,37]
ranges={1623:[[1,32]],1624:[[1,220],[221,440],[441,660],[661,827]],1625:[[1,37]]}
current = {}
with tarfile.open(root / 'recovered/original-registry-source-archive/objects.tar.gz', 'r:gz') as archive:
    for x in h['historical_source_roles']:
        i = x['original_index']; role = w['roles'][i]
        assert role['status'] == 'full-historical-body-trace-open' and role['historical_assessment'] == 'ASSESSMENT-L01-HISTORICAL-1623-1625.json'
        assert x['prior_role_status'] == 'unassessed' and x['new_personal_read_ranges'] == ranges[i]
        original = archive.extractfile(x['sha256']).read()
        assert sha(original) == x['sha256'] == role['historical_sha256'] and len(original.splitlines()) == x['lines']
        c = x['current_comparison']; body = contents(c['preserved_bytes'])
        assert c['source_revision'] == revision and sha(body) == c['sha256'] and len(body) == c['bytes']
        assert len(body.splitlines()) == c['lines'] and oid('blob', body) == c['git_blob'] == tree[c['path']]
        assert c['byte_identical'] is True and original == body and c['current_personal_ranges'] == ranges[i]
        check_chain(c['path'], proof['paths'][c['path']], original); current[i] = body
sources = {}
for x in a['current_supporting_sources']:
    body = contents(x['preserved_bytes'])
    assert x['revision'] == revision and sha(body) == x['sha256'] and len(body) == x['bytes']
    assert len(body.splitlines()) == x['lines'] and oid('blob', body) == x['git_blob'] == tree[x['path']]
    assert all(1 <= lo <= hi <= x['lines'] for lo, hi in x['ranges']); sources[x['path']] = body
asset = a['current_static_asset']; body = contents(asset['preserved_bytes'])
assert asset['revision']==revision and asset['personally_inspected_ranges']==[[1,8]]
assert sha(body)==asset['sha256'] and len(body)==asset['bytes'] and len(body.splitlines())==asset['lines']
assert oid('blob',body)==asset['git_blob']==tree[asset['path']] and body.splitlines()[0]==b'hip,ra_deg,dec_deg,v_mag,b_v'
check_chain(asset['path'], proof['paths'][asset['path']], body)
native = a['native_source_evidence']; assert native['channel']=='nightly-2026-07-13'
assert b'channel = "nightly-2026-07-13"' in sources['rust-toolchain.toml']
expected=['d51cc1160b3f46b63b0189e0bfed536581e30dea52b56b18908ff413b83a758c','2d6bf5d732be1b426294bac2116b96e79db5044a7cbd944f449a71638da4b9c7','41d78dcc60e2f57fd8613ed4d62e6538cf2c6765ad5f57373f92ffc46ca2a026']
docs=[]
for x, hash_ in zip(native['sources'], expected, strict=True):
    b=contents(x['preserved_bytes']); assert sha(b)==x['sha256']==hash_ and len(b)==x['bytes'] and len(b.splitlines())==x['lines']
    assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['personally_inspected_ranges']); docs.append(b)
assert b'Casting from an integer to float will produce the closest possible float' in docs[0] and b'roundTiesToEven' in docs[0]
assert b'binary32' in docs[1] and b'pub const MANTISSA_DIGITS: u32 = 24' in docs[2] and b'pub const RADIX: u32 = 2' in docs[2]
calendar=sources['crates/adventuresim-world-schema/src/calendar.rs']; celestial=sources['crates/adventuresim-core/src/celestial.rs']
assert b'minutes: u64' in calendar and b'self.minutes as f32 / MINUTES_PER_DAY as f32' in calendar
assert b'(self.minutes % MINUTES_PER_DAY) as u16' in calendar and b'WORLD_START_YEAR: CalendarYear = CalendarYear { year: 1544 }' in calendar
assert b'absolute_minute.minute_of_day()' in celestial and b'((local_minutes / 60.0 - 12.0) * 15.0)' in celestial
assert b'absolute_minute.days_since_epoch_f32()' in current[1623] and b'.rem_euclid(core::f32::consts::TAU)' in current[1623]
assert b'equatorial_to_world(' in current[1624] and b'material.equatorial_to_world = celestial.equatorial_to_world' in current[1624]
assert b'absolute_minute: self.absolute_minute' in sources['crates/adventuresim-tactical-core/src/scene_input.rs']
assert b'absolute_minute,' in sources['crates/adventuresim-tactical-server-dispatcher/src/scene_input.rs']
assert b'input.digest()?' in current[1625] and b'scene.digest == digest' in sources['crates/adventuresim-tactical-client/src/presentation/generation/installation.rs']
# Integer-only proof of the two representable binary32 neighbours and midpoint.
n=2**24
assert (n, n+1, n+2)==(16777216,16777217,16777218)
assert n//2 % 2 == 0 and (n+2)//2 % 2 == 1
assert n%1440==1216 and (n+1)%1440==1217 and 1544+n//(365*1440)==1575

manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
    original=lookup[x['original_path']]
    assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    body=contents(x['preserved_bytes']);assert sha(body)==x['published_sha256'] and len(body)==x['published_bytes']
    assert x['redacted_local_path_occurrences'] or (sha(body)==x['sha256'] and oid('blob',body)==x['git_blob'])
    records[x['original_path']]=body
for n,indices in [(512,[1623,1624]),(513,[1625])]:
    path=f'segovb/READ-L12-{n}.json';receipt=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [x['original_index'] for x in receipt['rows']]==indices
    assert receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
    assert receipt['plan'] in records and receipt['plan_sha256']==lookup[receipt['plan']]['sha256']
    assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in receipt['rows']:
        assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
path='adler/READ-L11-232.json';receipt=json.loads(records[path]);check=json.loads(records['adler/VERIFY-L11-232.json'])
assert [x['original_index'] for x in receipt['sources']]==[1623,1624,1625]
assert receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False
assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-232.json']['sha256']
for row in receipt['sources']+receipt['connected_reinspection']:
    if row.get('registration_sha256'):
        reg='adler/'+row['registration'].replace('\\','/').split('/adler/')[-1]
        assert reg in records and row['registration_sha256']==lookup[reg]['sha256']
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1346,1347))
baseline=load('BASELINE-95E-20261110.json');ledger=baseline['newly_assessed_ledger']['entries']
assert [x['id'] for x in a['conditional_findings']]==['REVIEW-STAR-SIDEREAL-TIME-PRECISION-001']
for item in a['conditional_findings']:
    assert item['accepted_historical_unique_count_change']==0 and item['repair']=='not started' and item['validation']=='not performed'
    assert sum(x['id']==item['id'] and x['record']=='ASSESSMENT-SKY-TIME-STRATEGIC-95E.json' for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json')
assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(baseline['latest_main_recheck'])['observed_revision']==revision
print('Verified three historical bodies, pinned-channel source and symbolic binary32 midpoint; no product execution.')
