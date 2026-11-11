#!/usr/bin/env python3
"""Verify terrain source and provenance evidence without executing product code."""
import gzip,hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
load=lambda p:json.loads((root/p).read_text())
contents=lambda p:gzip.decompress((root/p).read_bytes())
sha=lambda b:hashlib.sha256(b).hexdigest()
def oid(kind,body):return hashlib.sha1(kind.encode()+b' '+str(len(body)).encode()+b'\0'+body).hexdigest()
def entries(body):
 d={};i=0
 while i<len(body):
  end=body.index(b'\0',i);mode,n=body[i:end].split(b' ',1);d[n.decode()]=(mode.decode(),body[end+1:end+21].hex());i=end+21
 assert i==len(body);return d
def chain(path,proof,body,start,nodes):
 assert [x['name'] for x in proof['chain']]==path.split('/');current=start
 for step in proof['chain']:
  info=nodes[current];raw=contents(info['preserved_bytes']);assert oid('tree',raw)==current==step['tree'] and sha(raw)==info['sha256'] and len(raw)==info['bytes'];assert entries(raw)[step['name']]==(step['mode'],step['child']);current=step['child']
 assert current==proof['git_blob']==oid('blob',body) and sha(body)==proof['sha256'] and len(body)==proof['bytes'] and len(body.splitlines())==proof['lines']
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-TERRAIN-ADMISSION-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2079-2083.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
lengths={2079:646,2080:13,2081:28,2082:2059,2083:344};assert [x['original_index'] for x in h['historical_source_roles']]==list(lengths);current={}
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in h['historical_source_roles']:
  i=item['original_index'];n=lengths[i];row=work['roles'][i];before=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes'])
  assert item['prior_role_status']=='unassessed' and item['new_personal_read_ranges']==[[1,n]] and len(before.splitlines())==n and sha(before)==row['historical_sha256'] and before==body
  assert row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist and c['byte_identical'] and c['current_personal_ranges']==[] and c['current_content_read_via_identical_historical_ranges']==[[1,n]]
  assert sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],before,proof['root_tree'],nodes);current[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
assert all((root/p).is_file() for p in a['prior_current_evidence']) and a['repair']=='not started' and a['validation']=='not performed'
builder=current[2079];features=current[2080];digest=current[2081];terrain=current[2082];road=current[2083];producer=sources['crates/adventuresim-world-import/src/bin/build-strategic-map/terrain_features.rs'];cli=sources['crates/adventuresim-world-import/src/bin/build-strategic-map.rs'];schema=sources['crates/adventuresim-world-schema/src/source_package.rs']
assert b'pub water: Vec<Vec<Vec<[f64; 2]>>>' in features and b'pub wetlands: Vec<Vec<Vec<[f64; 2]>>>' in features and b'features.cultivated = cultivated.polygons' in producer
assert b'for polygon in polygons {' in builder and b'mask[y as usize * width as usize + x] ^= 1' in builder and (0^1^1)==0
assert b'entry.width > CHUNK_SIDE' in terrain and b'local_y as usize * entry.width as usize + local_x as usize' in terrain and 0*1+1==1
assert b'SourcePackageDigest::from_hex(value).is_ok()' in digest and b'pub content_sha256: String' in terrain and b'source: SourcePackageDigest' in road and b'pub struct SourcePackageDigest(String)' in schema
assert b'Validation(String)' in terrain and b'map_err(|error| crate::Error::Validation(error.to_string()))' in builder
assert b'let manifest_length = fs::metadata(manifest_path)?.len()' in terrain and b'let json = fs::read(manifest_path)?' in terrain and b'file.take(limit + 1).read_to_end(&mut bytes)?' in road
assert b'#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]' in terrain and b'pub struct TerrainWeights {' in terrain and b'debug_assert!(self.is_normalized())' in terrain and sum([0]*5)!=1000
assert b'west.floor() as i16' in builder and b'format!("S{:02}", -south)' in builder and -(-(2**15))>2**15-1 and b'const BOUNDS: [f64; 4] = PLAYABLE_BOUNDS' in cli
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];body=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(body)==item['published_sha256'] and len(body)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and body.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(body)==item['sha256'] and oid('blob',body)==item['git_blob']
 records[item['original_path']]=body
for n,indices in ((699,[2077,2078,2079]),(700,[2080,2081,2082]),(701,[2083,2084])):
 path=f'segovb/READ-L12-{n}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{n}.json']);assert [x['original_index'] for x in data['rows']]==indices and data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] is True and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for row in data['rows']:
  i=row['original_index'];expected=lengths.get(i,{2077:75,2078:294,2084:111}.get(i));assert row['fresh_personal_body_reads']==[[1,expected]]
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for n,indices in ((325,[2079,2080,2081,2082]),(326,[2083])):
 data=json.loads(records[f'adler/READ-L11-{n}.json'])
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,lengths[i]]] and row['sha256']==sha(current[i])
assert [x['original_index'] for x in json.loads(records['adler/READ-L11-326.json'])['sources'] if 2084<=x['original_index']<=2089]==list(range(2084,2090))
q=load('RECONCILIATION-059.json')['entries'];base=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for finding in a['reconnected_historical_findings']:
 i=finding['queue'];assert q[i]['contract_id']==base[i]['contract_id']==finding['id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation'];row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
for fid in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==fid);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert next(x for x in ledger if x['id']=='REVIEW-STARTUP-INPUT-UNBOUNDED-READ-001')['terrain_package_followup']==name
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified five historical roles, retained reviewer attribution, six original findings and symbolic source bounds; no product execution.')
