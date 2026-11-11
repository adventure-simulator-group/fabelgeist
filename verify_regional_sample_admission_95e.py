#!/usr/bin/env python3
"""Verify regional sample source evidence and recovered reviewer attribution."""
import gzip
import hashlib
import json
import tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,b):return hashlib.sha1(kind.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
    out,pos={},0
    while pos<len(b):
        end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
    assert pos==len(b);return out
def chain(path,r,body,root_tree,nodes):
    assert [s['name'] for s in r['chain']]==path.split('/')
    current=root_tree
    for s in r['chain']:
        info=nodes[current];data=contents(info['preserved_bytes']);assert oid('tree',data)==current==s['tree'] and sha(data)==info['sha256'] and len(data)==info['bytes']
        assert entries(data)[s['name']]==(s['mode'],s['child']);current=s['child']
    assert current==r['git_blob']==oid('blob',body) and sha(body)==r['sha256'] and len(body)==r['bytes'] and len(body.splitlines())==r['lines']
revision='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-REGIONAL-SAMPLE-ADMISSION-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1657-1661.json');w=load('COVERAGE-L01-ROLES.json');proof=load('REGIONAL-SAMPLE-HISTORICAL-GIT-PROOFS.json')
shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for file in proof['additional_shared_tree_objects']:nodes.update(load(file)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][proof['revision']]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision'] and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (root/advance['trees']['current']['path']).read_text().splitlines():header,path=line.split('\t',1);tree[path]=header.split()[2]
assert a['source_revision']==revision and a['repair']=='not started' and a['validation']=='not performed'
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1657,1662))
lengths={1657:56,1658:190,1659:68,1660:168,1661:91};current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
    for x in h['historical_source_roles']:
        i=x['original_index'];r=w['roles'][i];old=ar.extractfile(x['sha256']).read();c=x['current_comparison'];b=contents(c['preserved_bytes']);ranges=[[1,lengths[i]]]
        assert x['prior_role_status']=='unassessed' and x['new_personal_read_ranges']==ranges and len(old.splitlines())==lengths[i] and sha(old)==r['historical_sha256']
        assert r['status']=='full-historical-body-trace-open' and r['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1657-1661.json'
        assert sha(b)==c['sha256'] and len(b)==c['bytes'] and len(b.splitlines())==lengths[i] and c['current_personal_ranges']==ranges and oid('blob',b)==c['git_blob']==tree[c['path']] and c['byte_identical'] and old==b
        chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);current[i]=b
    for e in a['earlier_historical_source_identities']:
        i=e['original_index'];r=w['roles'][i];body=ar.extractfile(e['sha256']).read();path=e['source_id'].split(':',1)[1]
        assert e['new_personal_historical_full_read'] is False and e['additional_original_frontier_credit']==0 and sha(body)==r['historical_sha256'];chain(path,proof['paths'][path],body,proof['root_tree'],nodes)
    assert [e['original_index'] for e in a['earlier_historical_source_identities']]==[1662,1663,1664,1665,1896,1998,1999,2000,2003,2004,2005,2006]
sources={}
for s in a['current_supporting_sources']:
    b=contents(s['preserved_bytes']);assert s['revision']==revision and sha(b)==s['sha256'] and len(b)==s['bytes'] and len(b.splitlines())==s['lines']
    assert oid('blob',b)==s['git_blob']==tree[s['path']] and all(1<=lo<=hi<=s['lines'] for lo,hi in s['ranges']);sources[s['path']]=b
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in manifest['entries']};records={}
for r in a['recovered_historical_records']:
    original=lookup[r['original_path']];b=contents(r['preserved_bytes']);assert (r['sha256'],r['bytes'],r['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
    assert sha(b)==r['published_sha256'] and len(b)==r['published_bytes'] and (r['redacted_local_path_occurrences'] or (sha(b)==r['sha256'] and oid('blob',b)==r['git_blob']));records[r['original_path']]=b
for n,indices in [(526,[1659,1660,1661]),(669,[1998,1999]),(670,[2000]),(672,[2003,2004,2005,2006])]:
    p=f'segovb/READ-L12-{n}.json';z=json.loads(records[p]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json'])
    assert [r['original_index'] for r in z['rows']]==indices and z['accepted_unique_count_change']==0 and z['current_source_reconciled'] is False
    assert z['plan'] in records and z['plan_sha256']==lookup[z['plan']]['sha256'] and check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup[f'segovb/STATE-L12-{n}.json']['sha256']
    for r in z['rows']:
        assert r['registration'] in records and r['registration_sha256']==lookup[r['registration']]['sha256']
        if r.get('locator_qualification'):assert r['locator_qualification'] in records and r['locator_qualification_sha256']==lookup[r['locator_qualification']]['sha256']
    for r in z.get('qualified_supporting_same_lens_reuse',[]):assert r['registration'] in records and r['registration_sha256']==lookup[r['registration']]['sha256'] and r['receipt'] in records and r['receipt_sha256']==lookup[r['receipt']]['sha256'] and r['fresh_personal_body_reads_this_batch']==[]
assert b'Cellpercent<=100' in records['segovb/READ-L12-672.json'] and 'segovb/QUALIFICATION-L12-ORIGINAL2006-LOCATOR.json' in records
p='adler/READ-L11-239.json';z=json.loads(records[p]);check=json.loads(records['adler/VERIFY-L11-239.json'])
assert [r['original_index'] for r in z['sources']]==list(range(1660,1666)) and z['accepted_unique_count_change']==0 and z['current_reconciled'] is False and z['validated'] is False
assert check['receipt_sha256']==lookup[p]['sha256'] and check['state_sha256']==lookup['adler/STATE-L11-239.json']['sha256']
for r in z['sources']+z['connected_reinspection']:
    if r.get('registration_sha256'):
        reg='adler/'+r['registration'].replace('\\','/').split('/adler/')[-1];assert reg in records and r['registration_sha256']==lookup[reg]['sha256']
for r in z.get('same_lens_context',[]):
    rp='adler/'+r['receipt'].replace('\\','/').split('/adler/')[-1];assert rp in records and r['sha256']==lookup[rp]['sha256']
prev=load('ASSESSMENT-VISTA-ZERO-RING-CITY-GROUND-95E.json');assert [e['original_index'] for e in prev['earlier_historical_source_identities']]==[1657,1658,1659] and all(e['additional_original_frontier_credit']==0 for e in prev['earlier_historical_source_identities'])
pack=sources['crates/adventuresim-terrain/src/lib.rs'];builder=sources['crates/adventuresim-terrain/src/builder.rs'];sample=sources['crates/adventuresim-tactical-server-dispatcher/src/terrain_sampling.rs'];capture=sources['crates/adventuresim-tactical-server-dispatcher/src/regional_terrain.rs'];scene=sources['crates/adventuresim-tactical-server-dispatcher/src/scene_input.rs'];region=sources['crates/adventuresim-tactical-core/src/regional_terrain.rs'];grid=sources['crates/adventuresim-tactical-core/src/scene_input.rs'];basis=sources['crates/adventuresim-world-schema/src/basis_points.rs'];fraction=sources['crates/adventuresim-tactical-client/src/presentation/procedural.rs'];pigment=sources['crates/adventuresim-tactical-client/src/presentation/vista/pigment.rs']
assert b'bytes[2] > 5 || bytes[3] & !0b1111 != 0' in pack and b'canopy_percent: bytes[4]' in pack and b'hex_sha(&decoded) != entry.decoded_sha256' in pack
assert b'if canopy <= 100 { canopy } else { 0 }' in builder and b'u16::from(cell.canopy_percent) * BASIS_POINTS_PER_PERCENT' in sample
assert b'environment: environment_sample(cell)' in capture and b'environment.push(environment_sample(cell))' in scene
assert b'if vertices.len() != REGIONAL_TERRAIN_VERTICES' in region and b'Self::new(wire.request, wire.source, wire.vertices)' in region and b'.any(|value| value > BASIS_POINTS_PER_WHOLE)' in grid
assert b'UnitBasisPoints::saturating(value).as_unit_f32()' in fraction and b'if value > BASIS_POINTS_PER_WHOLE' in basis and b'bps(sample.canopy_bps)' in pigment
assert b'.all(|index| terrain.vertices()[*index].is_some())' in current[1657] and b'vista_sample_color(vertex.environment, weather)' in current[1657]
# Exact symbolic source values, no pack construction or consumer execution.
assert 101*100==10100 and 10100>10000 and min(10100,10000)==10000 and 255*100==25500<65536
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in list(range(1379,1384))+[1668])
f,=a['conditional_findings'];assert f['id']=='Q-L12-3ba77276b61acbc148a8' and load('baseline-059/queue.json')['entries'][1668]['contract_id']==f['id'] and f['accepted_historical_unique_count_change']==0 and f['repair']=='not started' and f['validation']=='not performed'
assert 'environment_sample percent->bps' in load('baseline-059/ledger.json')['segovb_continuation']['lens12']['pending_connected_contracts'][778]
baseline=load('BASELINE-95E-20261110.json');ledger=baseline['newly_assessed_ledger']['entries'];assert sum(r['id']==f['id'] and r['record']=='ASSESSMENT-REGIONAL-SAMPLE-ADMISSION-95E.json' for r in ledger)==1
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
assert load(baseline['latest_main_recheck'])['observed_revision']==revision
print('Verified historical roles, earlier Segovb sample question, and symbolic source admission; no product execution.')
