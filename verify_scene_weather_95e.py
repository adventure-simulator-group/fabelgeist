#!/usr/bin/env python3
"""Verify evidence provenance and source provenance and integer bounds, never product behavior."""
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
name='ASSESSMENT-SCENE-WEATHER-ADMISSION-95E.json';hist='ASSESSMENT-L01-HISTORICAL-1914-1919.json';a,h,worklist=load(name),load(hist),load('COVERAGE-L01-ROLES.json')
proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][old]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==old and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
    header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
lengths={1914:1026,1915:438,1916:40,1917:188,1918:48,1919:103};ranges={1914:[[150,174],[252,306],[313,358],[361,391],[427,467]],1915:[[27,85]],1916:[],1917:[],1918:[],1919:[]};current={}
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
scene=current[1914];buildings=current[1915];identity=current[1916];descriptor=current[1919]
assert b'weather.atmosphere.dew_point_deci_c > weather.temperature_deci_c + 5' in scene
assert b'pub fn environment_snapshot(&self, scene_digest: String)' in scene
assert b'let mut ids = std::collections::BTreeSet::new()' in buildings and b'pub struct SceneBuildingId(pub u64)' in identity
assert b'pub weather: WeatherSnapshot' in descriptor
weather=sources['crates/adventuresim-core/src/weather.rs'];assert b'pub temperature_deci_c: i32' in weather and b'pub dew_point_deci_c: i32' in weather
oldweather=a['historical_weather_source']
with tarfile.open(root/oldweather['source_archive'],'r:gz') as archive:before=archive.extractfile(oldweather['sha256']).read()
assert before==weather and oldweather['same_as_current'] and sha(before)==oldweather['sha256'] and len(before)==oldweather['bytes']
assert oldweather['git_blob']==oid('blob',before)==tree['crates/adventuresim-core/src/weather.rs']
check_chain('crates/adventuresim-core/src/weather.rs',proof['paths']['crates/adventuresim-core/src/weather.rs'],before,proof['root_tree'],nodes)
assert b'validate_weather(self.weather)?' in sources['crates/adventuresim-tactical-core/src/scene_input/validation.rs']
assert b'projection.validate_bindings(&placements)?' in sources['crates/adventuresim-tactical-core/src/scene_input/generation/grounding.rs']
assert b'if !members.remove(&placement.id)' in sources['crates/adventuresim-tactical-core/src/city_layout/compiled/grounded/projection.rs']
assert b'if !members.insert(building)' in sources['crates/adventuresim-tactical-core/src/city_layout/compiled/support.rs']
assert b'self.validate_physical_support_members()?' in sources['crates/adventuresim-tactical-core/src/city_layout/compiled/single_support.rs']
assert b'input.environment_snapshot(generated.digest)' in sources['crates/adventuresim-tactical-server/src/main.rs']
assert b'input.environment_snapshot(generated.digest.clone())' in sources['crates/adventuresim-tactical-client/src/strategic_scene/world.rs']
# Integer range reasoning, never execute the Rust validator.
maximum=2**31-1;result=(maximum+5+2**31)%(2**32)-2**31
assert maximum+5>maximum and result==-2147483644 and 0>result
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
    original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(body)==item['published_sha256'] and len(body)==item['published_bytes'] and (item['redacted_local_path_occurrences'] or (sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']))
    if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and b'<LOCAL_PATH_' in body
    records[item['original_path']]=body
for n,indices in [(638,[1914]),(639,[1915]),(640,[1916,1917,1918,1919])]:
    path=f'segovb/READ-L12-{n}.json';z=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [r['original_index'] for r in z['rows']]==indices and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[path]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for row in z['rows']:assert row['registration'] in records and row['registration_sha256']==lookup[row['registration']]['sha256'] and row['fresh_personal_body_reads']==[[1,lengths[row['original_index']]]]
z=json.loads(records['adler/READ-L11-293.json']);check=json.loads(records['adler/VERIFY-L11-293.json']);assert [r['original_index'] for r in z['sources']]==list(range(1917,1924)) and z['current_reconciled'] is False
assert check['receipt_sha256']==lookup['adler/READ-L11-293.json']['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-293.json']['sha256']
for row in z['sources']+z.get('connected_reinspection',[]):
    p='adler/'+row['registration'].replace('\\','/').split('/')[-1];assert p in records and row['registration_sha256']==lookup[p]['sha256']
for row in z.get('same_lens_context',[]):
    p='adler/'+row['receipt'].replace('\\','/').split('/')[-1];assert p in records and row['sha256']==lookup[p]['sha256']
prior=load('ASSESSMENT-GRADE-BOUNDED-95E.json');rec=next(x for x in prior['recovered_historical_records'] if x['original_path']=='adler/READ-L11-292.json');assert sha(contents(rec['preserved_bytes']))==rec['published_sha256'] and rec['sha256']==lookup['adler/READ-L11-292.json']['sha256']
original=load('baseline-059/queue.json')['entries'][1579];assert 'rawi16temperature+5' in original['first_failed_contract_or_unresolved_question']['required']
q=load('RECONCILIATION-059.json')['entries']
for i in (1578,1579,1580,1581,1582):assert q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
f=a['conditional_findings'][0];assert f['id']=='REVIEW-SCENE-WEATHER-TEMPERATURE-OVERFLOW-001' and f['accepted_historical_unique_count_change']==0 and f['repair']=='not started' and f['validation']=='not performed'
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert sum(x['id']==f['id'] and x['record']==name for x in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified six historic roles, corrected original field-width attribution and source-only integer bounds; no product execution.')
