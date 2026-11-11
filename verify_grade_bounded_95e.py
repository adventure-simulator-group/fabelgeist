#!/usr/bin/env python3
"""Verify evidence provenance and exact rational bounds, never product behavior."""
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
name='ASSESSMENT-GRADE-BOUNDED-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1898-1901.json';a,h,worklist=load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1898:98,1899:296,1900:109,1901:182};ranges={1898:[[1,98]],1899:[],1900:[],1901:[]};current={}
assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i=item['original_index'];n=lengths[i];row=worklist['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
        assert item['prior_role_status']=='unassessed' and item['prior_current_observations']==row.get('current_observations',[])
        assert item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256']
        assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
        assert before==body and c['byte_identical'] and c['current_personal_ranges']==ranges[i] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
        assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']]
        check_chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
    body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines']
    assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
admission=current[1898];grade=current[1899];ground=current[1901]
assert b'if !scale.is_finite()' in admission and b'heights.iter().any' not in admission
assert b'self.heightmap.iter().any(|h| !h.is_finite())' in admission
assert b'pub const PLAYABLE: Self = Self(0.65)' in grade
assert b'let roundoff = magnitude * f64::from(f32::EPSILON) * HEIGHTMAP_ROUNDOFF_ULPS' in grade
assert b'if initial <= bound + roundoff' in grade and b'let target = (bound - roundoff).max(0.0)' in grade
assert b'heights.copy_from_slice(&replacement)' in grade
assert b'#[derive(Component, Serialize, Deserialize, Debug, Reflect, Clone, PartialEq)]' in ground
assert b'pub fn uniform_for_terrain' in ground and b'pub fn from_samples(' in ground
scene=sources['crates/adventuresim-tactical-core/src/scene_input.rs']
assert b'(-12_000.0..=12_000.0).contains(height)' in scene and b'(0.25..=2_000.0).contains(&grid.spacing_metres)' in scene
assert b'if source.spacing_metres <= TARGET_SPACING_METRES' in scene and b'TacticalSurface::Road | TacticalSurface::Water' in scene
assert b'crate::scene::grade::constrain(' in scene
validation=sources['crates/adventuresim-tactical-core/src/scene_input/validation.rs'];assert b'validate_grid(&self.playable, MAX_PLAYABLE_SIDE, SampleGridKind::Playable)?' in validation
geo=sources['crates/adventuresim-tactical-core/src/scene_input/generation/geographic.rs'];assert geo.index(b'self.validate_source()?')<geo.index(b'prepare_terrain(self, width, depth, spacing')
assert b'refine_authoritative_terrain(' in geo
assert b'terrain.constrain_max_grade(crate::scene::grade::TerrainGradeLimit::PLAYABLE)?' in sources['crates/adventuresim-tactical-core/src/scene_input/refinement.rs']
assert b'if ground.urban.overlaps_disk(position, envelope_radius)' in sources['crates/adventuresim-tactical-client/src/presentation/ground_scatter/loose_stone/spawn.rs']
# Rational model only. The 0.65 binary32 value and dyadic sample magnitudes are exact.
a0=Fraction(8192);delta=Fraction(85,512);spacing=Fraction(1,4);limit=Fraction(5452595,8388608);eps=Fraction(1,2**23)
assert (delta*1024).denominator==1 and -12000<=a0<a0+delta<=12000
assert delta/spacing==Fraction(85,128)>limit
bound=spacing*limit;allowance=(a0+delta)*eps*4
assert delta<=bound+allowance and delta>bound+spacing*eps*4
assert Fraction(419509,1073741824)==bound+allowance-delta
assert Fraction(8)<Fraction(65,100)+(Fraction(100000008)*eps*4)
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(630,[1898,1899]),(631,[1900,1901,1902])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:
        assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256'] and row['same_lens_reuse']==[]
        if row['original_index'] in lengths:assert row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
for n,indices in [(290,[1901,1902,1903,1904,1905]),(292,[1914,1915,1916])]:
    path=f'adler/READ-L11-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'adler/VERIFY-L11-{n}.json']);assert [r['original_index'] for r in z['sources']]==indices and z['current_reconciled'] is False
    assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'adler/STATE-L11-{n}.json']['sha256']
    for row in z['sources']+z.get('connected_reinspection',[]):
        p='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert p in records and row['registration_sha256']==lookup[p]['sha256']
    for row in z.get('same_lens_context',[]):
        p='adler/'+row['receipt'].replace('\\','/').split('/')[-1];assert p in records and row['sha256']==lookup[p]['sha256']
for assessment,path in [('ASSESSMENT-SHORT-ROUTE-HEIGHTFIELD-95E.json','adler/READ-L11-289.json'),('ASSESSMENT-GROUND-RASTER-ADMISSION-95E.json','adler/READ-L11-299.json')]:
    prior=load(assessment);record=next(x for x in prior['recovered_historical_records'] if x['original_path']==path);assert sha(contents(record['preserved_bytes']))==record['published_sha256'] and record['sha256']==lookup[path]['sha256']
original=load('baseline-059/ledger.json')['discovery_findings_and_dependencies']['ADLER-L11-TERRAIN-GRADE-OFFSET-TOLERANCE']
assert 'no newlyassertedboundedheightcounterexample' in original['fresh_frozen_source_qualifications'][1]['qualification']
assert 'excludes the recorded1e8 counterexample' in original['classification']
q=load('RECONCILIATION-059.json')['entries']
for i in (109,110,1568,1569,1570,1571):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
f=a['conditional_findings'][0];assert f['id']=='ADLER-L11-TERRAIN-GRADE-OFFSET-TOLERANCE' and f['accepted_historical_unique_count_change']==0 and f['repair']=='not started' and f['validation']=='not performed'
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']==f['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified four historical roles, original and later grade qualifications, and source-only bounded arithmetic; no product execution.')
