#!/usr/bin/env python3
"""Verify historical provenance, source evidence and exact rational bounds only."""
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
name='ASSESSMENT-SHORT-ROUTE-HEIGHTFIELD-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1895-1897.json'
a,h,worklist=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1895:245,1896:112,1897:553};current={};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths)
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in h['historical_source_roles']:
        i=item['original_index'];n=lengths[i];row=worklist['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
        assert item['prior_role_status']=='unassessed' and item['prior_current_observations']==row.get('current_observations',[])
        assert item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256']
        assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
        assert before==body and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
        assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']]
        check_chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
    body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines']
    assert all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
route=current[1895];region=current[1896];scene=current[1897]
assert b'if !(2..=MAX_MAP_ROUTE_POINTS).contains(&points.len())' in route
assert b'pub const REGIONAL_TERRAIN_SIDE: usize = 65' in region and b'vertices: Vec<Option<RegionalTerrainVertex>>' in region
assert b'.flat_map(|x|' in scene and b'(0..self.grid_depth()).map(move |z| self.heightmap[x + z * self.grid_width()])' in scene
assert b'.chunks_exact(self.grid_width())' in scene and b'collider_heightfield_matches_the_rendered_surface_without_transposing_axes' in scene
terrain=sources['crates/adventuresim-terrain/src/lib.rs']
for snippet in (b'pub const MAX_ROUTE_NODES: usize = 750_000',b'let mut step_lat = 30.0 / 111_320.0',b'let padding = (span * factor).max(0.01)',b'let start_node = snap(&grid, coordinate(start), 8)',b'let goal_node = snap(&grid, coordinate(goal), 8)',b'if current.index == goal_i',b'let mut nodes = vec![goal]',b'for pair in nodes.windows(2)',b'if points.len() <= 2',b'if points.len() <= cap'):
    assert snippet in terrain,snippet
travel=sources['crates/strategic-web/src/routes/travel.rs']
assert b'destination.distance_m = plan.distance_m' in travel and b'destination.journey_minutes = plan.minutes' in travel and b'destination.terrain_route = Some(plan)' in travel
assert b'.plans\n            .get(&key)' in travel
producer=sources['crates/strategic-web/src/strategic_map/producer.rs']
assert b'Some(link) if link.marker.origin != origin' in producer and b'Some(route) => MapRoute::new(' in producer
markup=sources['crates/strategic-web/src/strategic_map/mod.rs'];assert b'strategic_map_bundle_unavailable()' in markup and b'Map unavailable' in markup
capture=sources['crates/adventuresim-tactical-server-dispatcher/src/regional_terrain.rs'];assert b'for row in 0..REGIONAL_TERRAIN_SIDE' in capture and b'for column in 0..REGIONAL_TERRAIN_SIDE' in capture and b'vertices.push(vertex)' in capture
surface=sources['crates/adventuresim-tactical-client/src/presentation/regional_map/geographic_surface.rs'];assert b'if let [Some(first), Some(second), Some(third)] = indices.map(vertex)' in surface
# Exact rational calculation of the hypothetical first window; no native float or product operation is run.
step=Fraction(30,111320);first=Fraction(1,100)/step;second=(Fraction(1,100)+Fraction(1,1000000))/step
assert first==Fraction(2783,75) and second==Fraction(27832783,750000)
assert 37<first<Fraction(75,2) and 37<second<Fraction(75,2)
ceil=lambda x:(x.numerator+x.denominator-1)//x.denominator
width=ceil(Fraction(20001,1000000)/step)+1;height=ceil(Fraction(2,100)/step)+1
assert (width,height,width*height)==(76,76,5776)
# Compare archived source bytes, not a collider, and inspect the rectangular storage correspondence.
native=a['native_evidence'];parrymanifest=load(native['parry_manifest']);avianmanifest=load(native['avian_manifest'])
with tarfile.open(root/native['avian_archive'],'r:gz') as archive:
    member=next(x for x in avianmanifest['inspected_members'] if x['path']=='avian3d-0.7.0/src/collision/collider/parry/mod.rs');avian=archive.extractfile(member['path']).read();assert sha(avian)==member['sha256'] and len(avian)==member['bytes']
    assert b'let row_count = heights.len()' in avian and b'let column_count = heights[0].len()' in avian and b'Array2::new(row_count, column_count, data)' in avian
with tarfile.open(root/native['parry_archive'],'r:gz') as archive:
    data={}
    for path,ranges in native['parry_ranges'].items():
        item=next(x for x in parrymanifest['inspected_members'] if x['path']==path);body=archive.extractfile(path).read();assert sha(body)==item['sha256'] and len(body)==item['bytes'] and all(r in parrymanifest['personally_inspected_ranges'][path] for r in ranges);data[path]=body
    hf=data['parry3d-0.27.0/src/shape/heightfield3.rs'];arr=data['parry3d-0.27.0/src/utils/array2.rs']
    assert b'grid coordinates along the `z` axis' in hf and b'grid coordinates along the `x` axis' in hf
    assert b'let z0 = -0.5 + cell_height * (i as Real)' in hf and b'let x0 = -0.5 + cell_width * (j as Real)' in hf
    assert b'i + j * self.nrows' in arr and b'Some(Triangle::new(p00, p10, p01))' in hf
# No product objects built: this is an index identity for a 3-by-2 source matrix.
source=[0,1,4,10,13,20];flat=[source[x+z*3] for x in range(3) for z in range(2)]
assert flat==[0,10,1,13,4,20] and all(flat[z+x*2]==source[x+z*3] for x in range(3) for z in range(2))
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,i in [(627,1895),(628,1896),(629,1897)]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==[i] and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    row=z['rows'][0];assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256'] and row['fresh_personal_body_reads']==[[1,lengths[i]]] and row['same_lens_reuse']==[]
reg=json.loads(records['segovb/REGISTRATION-SEGOVB-L12-803.json']);qual=json.loads(records[reg['locator_qualification']]);assert reg['locator_qualification_sha256']==lookup[reg['locator_qualification']]['sha256']
assert qual['body_coverage']==qual['new_population_credit']==0 and qual['historical_locator_proof_body_not_read'] is True and qual['original_index']==1896
assert qual['exact_source_sha256']==worklist['roles'][1896]['historical_sha256']
path='adler/READ-L11-289.json';adler=json.loads(records[path]);check=json.loads(records['adler/VERIFY-L11-289.json'])
assert [r['original_index'] for r in adler['sources']]==list(range(1895,1901)) and adler['current_reconciled'] is False
assert check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-289.json']['sha256']
for row in adler['sources']+adler.get('connected_reinspection',[]):
    p='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert p in records and row['registration_sha256']==lookup[p]['sha256']
for row in adler.get('same_lens_context',[]):
    p='adler/'+row['receipt'].replace('\\','/').split('/')[-1];assert p in records and row['sha256']==lookup[p]['sha256']
# The exact older producer packets were independently recovered in prior checkpoints.
prior=load('ASSESSMENT-CONDITION-REGIONAL-95E.json');rootitem=prior['recovered_older_774_records'][0];rootbody=contents(rootitem['preserved_bytes']);assert sha(rootbody)==rootitem['sha256']
rootrecord=next(json.loads(x) for x in rootbody.splitlines() if json.loads(x).get('id')=='ROOT-PPS-TERRAIN-DISPATCH-WEB-037')
assert rootrecord['source_proofs'][6]==qual['raw_producer_observations'][0]['source_context']
inventory=load('ASSESSMENT-INVENTORY-CACHE-IDENTITY-95E.json');olditems=inventory['recovered_older_774_records'];nativeitem=next(x for x in olditems if x['path_in_774'].endswith('/native/traces.jsonl'));nativebody=contents(nativeitem['preserved_bytes'])
nativerecord=next(json.loads(x) for x in nativebody.splitlines() if json.loads(x).get('id')=='NATIVE-PS-ROOT-TERRAIN-SOURCE-PACKET-015');assert nativerecord['related_current_reads'][6]==qual['raw_producer_observations'][1]['source_context']
oldloc=next(x for x in olditems if x['path_in_774'].endswith('source-scope-reconciliation-044.json'));assert oldloc['package_original_sha256']!=oldloc['package_exported_sha256']
loc=json.loads(contents(oldloc['preserved_bytes']))['new_variant_identity_and_range_map'][109];assert loc['source_id']==worklist['roles'][1896]['source_id'] and loc['recorded_sha256']==worklist['roles'][1896]['historical_sha256']
q=load('RECONCILIATION-059.json')['entries']
for i in (1565,1566,1567,1568):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
f=a['conditional_findings'][0];assert f['id']=='REVIEW-SAME-SNAPPED-NODE-ZERO-ROUTE-001' and f['accepted_historical_unique_count_change']==0 and f['repair']=='not started' and f['validation']=='not performed'
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']==f['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified three historical roles, earlier geographic producer attribution and source-only route/heightfield evidence; no product execution.')
