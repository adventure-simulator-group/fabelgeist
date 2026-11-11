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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-COLOR-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2232-2237.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2232,2238))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['historical']['sha256']==item['sha256'] and regrow['historical']['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);m=len(body.splitlines());assert sha(old)==row['historical_sha256'] and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,m]] and regrow['current']['sha256']==sha(body) and regrow['current']['ranges']==[[1,m]]
  assert c['byte_identical'] and old==body and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
conversion=bodies[2234];space=bodies[2236];transfer=bodies[2237];yuv=sources['crates/fabelgeist-color/src/yuv.rs'];pixel=sources['crates/fabelgeist-color/src/pixel.rs']
assert b'let num_pixels = width * height' in conversion and b'if src_format == dst_format' in conversion and b'let len = src.len().min(dst.len())' in conversion and b'dst[..len].copy_from_slice(&src[..len])' in conversion and b'Self::Rgba8Unorm' in pixel and b'=> Some(4)' in pixel
assert b'let chroma_len = num_pixels / 4' in conversion and b'let u_plane = &src[y_len..y_len + chroma_len]' in conversion and b'let chroma_w = width.div_ceil(2)' in yuv and b'u_plane[uv_row_offset + uv_col]' in yuv and b'v_plane[uv_row_offset + uv_col]' in yuv
assert b'let uv_len = num_pixels / 2' in conversion and b'let expected_uv = num_pixels / 2' in conversion and b'uv_plane[uv_offset + 1]' in yuv and b'uv_dst[uv_off + 1]' in yuv and b'(width.div_ceil(2), height.div_ceil(2))' in yuv
# Integer extent reasoning only; no buffers, indices or conversion are run.
assert (1*1)//4 < ((1+1)//2)*((1+1)//2) and (2*1)//2 < 2
assert b'let p_inv = invert_3x3(&p).unwrap_or(' in space and b'invert_3x3(&to_xyz).unwrap_or(' in space and b'pub fn new(src_primaries: &ColorPrimaries, dst_primaries: &ColorPrimaries) -> Self' in conversion and b'SingularMatrix' in conversion
# Equal primary columns are linearly dependent in exact arithmetic.
m=(1,1,1,1,1,1,2,2,2);det=m[0]*(m[4]*m[8]-m[5]*m[7])-m[1]*(m[3]*m[8]-m[5]*m[6])+m[2]*(m[3]*m[7]-m[4]*m[6]);assert det==0 and b'Chromaticity::new(0.0001, -0.0770)' in space
assert b'Self::DciP3 => TransferFunction::Dci' in space and b'Self::AdobeRgb => TransferFunction::Gamma22' in space and b'pub const fn to_linear_space(&self)' in space and b'_ => *self' in space and b'Self::Linear => encoded' in transfer
oldproof=load(a['older_l3_git_membership_evidence']);oldrev=oldproof['revision'];assert oldrev=='e2755a732c84e67296b5e3441a4259a84f461b2f';raw=contents(oldproof['commit']['preserved_bytes']);assert oid('commit',raw)==oldrev and sha(raw)==oldproof['commit']['sha256'] and len(raw)==oldproof['commit']['bytes'] and raw.splitlines()[0]==b'tree '+oldproof['root_tree'].encode()
oldbodies={'crates/fabelgeist-color/src/conversion.rs':conversion,'crates/fabelgeist-color/src/yuv.rs':yuv,'crates/fabelgeist-color/src/pixel.rs':pixel}
for path,body in oldbodies.items():chain(path,oldproof['paths'][path],body,oldproof['root_tree'],oldproof['tree_objects'])
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
path='segovb/READ-L12-739.json';data=json.loads(records[path]);check=json.loads(records['segovb/VERIFY-L12-739.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
for i in range(2232,2238):
 x=next(z for z in data['rows'] if z['original_index']==i);n=roles[i-2232]['lines'];expected=[[1,230]] if i==2234 else [[1,229]] if i==2237 else [[1,n]];assert x['source_sha256']==roles[i-2232]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'] and x['fresh_personal_body_reads']==x['assessed_ranges']==expected
for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
tail=data['additional_connected_body_receipts'][0];assert tail['path'] in records and sha(records[tail['path']])==tail['sha256'];tailrow=json.loads(records[tail['path']])['rows'][0];assert tailrow['original_index']==2234 and tailrow['fresh_personal_body_reads']==[[231,432]] and tailrow['combined_L12_coverage']==[[1,432]] and tailrow['excluded_unread_tail']==[[433,472]] and tailrow['registration'] in records
for batch,indices in [(355,[2232,2233,2234]),(356,[2235,2236,2237])]:
 r=json.loads(records[f'adler/READ-L11-{batch}.json']);assert r['accepted_unique_count_change']==0
 for i in indices:
  x=next(z for z in r['sources'] if z['original_index']==i);assert x['fresh_reads']==[[1,roles[i-2232]['lines']]] and x['sha256']==roles[i-2232]['sha256']
for n,i,scope in [(1093,2234,[[1,230]]),(1094,2234,[[1,472]]),(1097,2237,[[1,229]]),(1109,2237,[[1,271]])]:
 r=json.loads(records[f'adler/REGISTRATION-ADLER-{n}.json']);assert r['original_index']==i and r['ranges']==scope and r['prospective']
for file,paths in [('root-l3-extension-20261030/READ-ROOT-L3-CURRENT-005.json',{'crates/fabelgeist-color/src/conversion.rs','crates/fabelgeist-color/src/yuv.rs'}),('root-l3-extension-20261030/READ-ROOT-L3-CURRENT-006.json',{'crates/fabelgeist-color/src/pixel.rs'})]:
 r=json.loads(records[file]);assert r['fresh_personal_lines']==0
 for path in paths:
  x=next(z for z in r['rows'] if z.get('git_path')==path);assert x['revision']==oldrev and x['source_sha256']==sha(oldbodies[path]) and x['fresh_personal_ranges']==[] and x['qualified_reuse_ranges']==x['assessed_ranges'] and x['prior_attribution'].startswith('Root personal independent Lens3') and x['prior_own_lens_receipt']['path'] in records and x['registration_ref']['path'] in records
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (147,148,1824,3145):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
for finding in a['reconnected_historical_findings']:assert old[finding['queue']]['contract_id']==finding['id']
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
correction=load('ASSESSMENT-L01-HISTORICAL-2224-2231.json');assert 'role2231 in full' in correction['observations'][1] and 'partial2234 and2237' in correction['attribution_correction']
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified historical color scopes, e275 qualified reuse, Git proofs and symbolic extent conditions; no product execution.')
