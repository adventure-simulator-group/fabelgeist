#!/usr/bin/env python3
"""Verify evidence provenance and source-only integer bounds, never product behavior."""
import gzip,hashlib,json,tarfile
from fractions import Fraction
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,body):return hashlib.sha1(kind.encode()+b' '+str(len(body)).encode()+b'\0'+body).hexdigest()
def entries(body):
    result,pos={},0
    while pos<len(body):
        end=body.index(b'\0',pos);mode,name=body[pos:end].split(b' ',1);result[name.decode()]=(mode.decode(),body[end+1:end+21].hex());pos=end+21
    assert pos==len(body);return result
def check_chain(path,proof,body,start,nodes):
    assert [x['name'] for x in proof['chain']]==path.split('/')
    current=start
    for part in proof['chain']:
        info=nodes[current];raw=contents(info['preserved_bytes']);assert oid('tree',raw)==current==part['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes']
        assert entries(raw)[part['name']]==(part['mode'],part['child']);current=part['child']
    assert current==proof['git_blob']==oid('blob',body) and sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e'
name='ASSESSMENT-URBAN-BUCKET-CAPACITY-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1902-1905.json';a,h,worklist=load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1902:28,1903:378,1904:181,1905:170};ranges={1902:[],1903:[[1,300]],1904:[[1,181]],1905:[[1,170]]};current={}
assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i=item['original_index'];n=lengths[i];row=worklist['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
        assert item['prior_role_status']==('bounded-current-trace-only' if i==1904 else 'unassessed') and item['prior_current_observations']==row.get('current_observations',[])
        assert item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256']
        assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
        assert before==body and c['byte_identical'] and c['current_personal_ranges']==ranges[i] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
        assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']]
        check_chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
    body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines']
    assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
urban=current[1903];support=current[1904];queries=current[1905]
assert b'for x in minimum.x..=maximum.x' in urban and b'for y in minimum.y..=maximum.y' in urban
assert b'buckets.entry((x, y)).or_default().push(index)' in urban
assert b'pub fn new(' in urban and b'street_buckets: SpatialBuckets::new(streets.iter().map(street_bounds))' in urban
assert b'CityStreetPatch::Market { corners_metres, .. } => corners_bounds(corners_metres)' in urban
assert b'#[serde(skip)]' in urban and b'.get_or_init(|| UrbanGroundLookup::new(&self.streets, &self.yards, &self.buildings))' in urban
city=sources['crates/adventuresim-tactical-core/src/city_layout.rs'];surface=sources['crates/adventuresim-tactical-core/src/city_layout/surfaces.rs']
assert b'SPATIAL_BUCKET_METRES: f32 = 32.0' in city and b'MAX_CITY_STREET_PATCHES: usize = 12_000' in surface
assert b'Self::Market { corners_metres, .. } => corners_metres.into_iter().all(Vec2::is_finite)' in surface
assert b'urban::validate(self)?' in sources['crates/adventuresim-tactical-core/src/scene_input/validation.rs']
assert b'input.streets.len() > MAX_CITY_STREET_PATCHES' in sources['crates/adventuresim-tactical-core/src/scene_input/urban.rs']
assert b'street.is_valid()' in sources['crates/adventuresim-tactical-core/src/scene_input/urban.rs']
assert b'ground.urban = crate::scene::UrbanGroundSurfaces::new(streets, yards, buildings)?' in sources['crates/adventuresim-tactical-core/src/scene_ground.rs']
assert b'pub use ground::*' in sources['crates/adventuresim-tactical-core/src/scene.rs']
assert b'UrbanGroundLookup' in sources['crates/adventuresim-tactical-core/src/lib.rs']
assert b'pub(super) enum TerrainGeometry' in support and b'Owned(BoundedSettlementTerrain)' in support
assert b'.flat_map(|f| f.cut_faces().iter())' in support and b'.collect::<Result<Vec<_>, _>>()' in support
assert b'.ok_or(TerrainSupportError::MissingSupport { point })' in queries
# Source-derived integer count only; no product lookup or floating operation is executed.
lo,hi=-1_000_000//32,1_000_000//32
assert (lo,hi)==(-31250,31250) and (hi-lo+1)**2==3_906_375_001
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(632,[1903]),(633,[1904,1905,1906])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:
        assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256']
        if row['original_index'] in lengths:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
    for row in z.get('qualified_supporting_same_lens_reuse',[]):
        assert row['receipt'] in records and row['registration'] in records and row['receipt_sha256']==lookup[row['receipt']]['sha256'] and row['registration_sha256']==lookup[row['registration']]['sha256']
assert 'finite/boundedpatchproducer/current beforeworkcap' in json.loads(records['segovb/READ-L12-632.json'])['rows'][0]['function_contract']
q=load('RECONCILIATION-059.json')['entries']
for i in (1569,1570,1571,1572):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
f=a['conditional_findings'][0];assert f['id']=='REVIEW-URBAN-BUCKET-SPAN-CAPACITY-001' and f['accepted_historical_unique_count_change']==0 and f['repair']=='not started' and f['validation']=='not performed'
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']==f['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified four historical roles, prior reviewer qualifications and source-only urban bucket arithmetic; no product execution.')
