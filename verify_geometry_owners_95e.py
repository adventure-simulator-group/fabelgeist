#!/usr/bin/env python3
"""Verify publication evidence only; do not execute product behavior."""
import gzip,hashlib,json,tarfile
from fractions import Fraction
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-GEOMETRY-OWNERS-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2267-2272.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2267,2273))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']==('bounded-current-trace-only' if i in (2270,2272) else 'unassessed') and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical']==(old==body)==True and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
assert [len(bodies[i].splitlines()) for i in range(2267,2273)]==[21,126,95,40,16,104]
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
cuboid=bodies[2268];error=bodies[2269];length=bodies[2270];measurement=bodies[2272];cloth=sources['crates/fabelgeist-cloth/src/garment/mod.rs'];triangulate=sources['crates/fabelgeist-cloth/src/triangulate/mod.rs'];selection=sources['crates/adventuresim-character-creator/src/garment/selection.rs'];drape=sources['crates/adventuresim-character-creator/src/garment/drape.rs'];fit=sources['crates/fabelgeist-garment-fit/src/lib.rs'];lock=sources['Cargo.lock'];adapter=sources['crates/adventuresim-building-generator/src/spatial_geometry/error.rs']
assert b'pub struct PositiveLength(f32)' in length and b'if !value.is_finite()' in length and b'if value == 0.0' in length and b'pub struct SignedLength(f32)' in measurement and b'pub struct Area(f32)' in measurement and b'Self::from_metres(self.0 * factor)' in measurement
assert b'pub struct CuboidDimensions([SignedLength; 3])' in cuboid and b'component.metres() < 0.0' in cuboid and b'Self::from_metres(axes).map_err(de::Error::custom)' in cuboid and b'd.deserialize_tuple_struct("Vec3", 3, AxesVisitor)' in cuboid
assert b'if target_edge <= 0.0 || target_edge.is_nan()' in cloth and b'outline.len() < 3 || target_edge <= 0.0 || target_edge.is_nan()' in triangulate and b'settings.resolution_cm * CM_TO_M' in fit and b'!self.resolution_cm.is_finite()' in selection and b'RESOLUTION_CM: RangeInclusive<f32> = 2.5..=6.0' in selection and b'input.selection.validate()?' in drape
assert b'let axis = CoordinateAxis::X;' in length and b'axis: CoordinateAxis::X' in measurement and b'{role:?} has a nonfinite {axis:?} component' in error and b'Positive scalar measurements are independent of coordinate frame' in length
assert b'impl serde::Serialize for DirectionError' in error and b'serializer.serialize_str(match self {' in error and b'Self::NaN => "nan"' in error and b'InvalidDirectionError::NaN => DirectionError::NaN' in adapter
native={}
for item in a['native_reference_sources']:
 m=load(item['manifest']);archive=(root/item['archive']).read_bytes();assert sha(archive)==item['archive_sha256']==m['sha256'] and m['sha256'].encode() in lock
 with tarfile.open(root/item['archive'],'r:gz') as tar:body=tar.extractfile(item['member']).read()
 assert sha(body)==item['member_sha256'] and len(body)==item['member_bytes'] and all(1<=lo<=hi<=len(body.splitlines()) for lo,hi in item['personally_inspected_ranges']);native[item['member']]=body
case=native['serde_derive-1.0.229/src/internals/case.rs'];derive=native['serde_derive-1.0.229/src/ser.rs'];jsonser=native['serde_json-1.0.151/src/ser.rs'];jsonde=native['serde_json-1.0.151/src/de.rs'];post=native['postcard-1.1.3/src/ser/serializer.rs']
assert b'LowerCase => variant.to_ascii_lowercase()' in case and b'if i > 0 && ch.is_uppercase()' in case and b'_serde::Serializer::serialize_unit_variant(' in derive
assert b'self.serialize_str(variant)' in jsonser and b'fn end_seq(&mut self)' in jsonde and b'ErrorCode::TrailingCharacters' in jsonde and b'fn deserialize_tuple_struct<V>' in jsonde and b'self.deserialize_seq(visitor)' in jsonde
assert b'self.try_push_varint_usize(v.len())' in post and b'.try_extend(v.as_bytes())' in post and b'self.try_push_varint_u32(variant_index)' in post
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
assert len(records)==71
adler=json.loads(records['adler/READ-L11-361.json']);assert adler['accepted_unique_count_change']==0
for i,n in ((2267,21),(2268,126),(2269,95),(2270,40),(2271,16),(2272,104)):
 x=next(r for r in adler['sources'] if r['original_index']==i);assert x['fresh_reads']==x['scope']==[[1,n]] and x['sha256']==roles[i-2267]['sha256']
for batch,indices in ((746,(2267,2268)),(747,(2269,2270,2271,2272))):
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256']
 for i in indices:
  x=next(r for r in data['rows'] if r['original_index']==i);n=len(bodies[i].splitlines());assert x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,n]] and x['source_sha256']==roles[i-2267]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution']
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for i in (2270,2272):
 r=roles[i-2267];assert r['prior_role_status']=='bounded-current-trace-only' and r['prior_current_observations']
 for obs in r['prior_current_observations']:assert load(obs['assessment'])['current_revision']=='c100638b720a7281a5fc96f6a3c5c23d5dda85da'
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (2435,3445):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified geometry owners, predecessor scope and native codec source; no product execution.')
