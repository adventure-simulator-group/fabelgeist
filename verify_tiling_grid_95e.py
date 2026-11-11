#!/usr/bin/env python3
"""Verify source and recovery evidence only; do not run product behavior."""
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';pr='6533e444fa37552927070cc8d8190907fd3b413e';name='ASSESSMENT-TILING-GRID-95E.json';hist='ASSESSMENT-L01-HISTORICAL-2210-2219.json'
a,h,work=load(name),load(hist),load('COVERAGE-L01-ROLES.json');proof=load(a['git_membership_evidence']);shared=load(proof['shared_commit_and_tree_objects']);nodes=dict(shared['tree_objects'])
for p in proof['additional_shared_tree_objects']:nodes.update(load(p)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects']);commit=contents(shared['revisions'][pr]['commit']['preserved_bytes']);assert oid('commit',commit)==proof['revision']==pr and commit.splitlines()[0]==b'tree '+proof['root_tree'].encode()
tree={}
for line in (root/load('MAIN-ADVANCE-95E.json')['trees']['current']['path']).read_text().splitlines():
 header,path=line.split('\t',1);tree[path]=header.split()[2]
registry=json.loads(contents('recovered/original-registry-source-archive/registry.json.gz'));bodies={};roles=h['historical_source_roles'];assert [x['original_index'] for x in roles]==list(range(2210,2220))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
 for item in roles:
  i=item['original_index'];row=work['roles'][i];r=registry['files'][i];n=item['lines'];assert item['prior_role_status']=='unassessed' and row['status']=='full-historical-body-trace-open' and row['historical_assessment']==hist
  assert item['original_locator']==r['path'] and [x['read_ranges'] for x in item['original_supplier_scopes']]==[x['read_ranges'] for x in r.get('owner_contract_proofs',[])] and [x['read_ranges_as_recorded'] for x in item['original_producer_scopes']]==[x['read_ranges_as_recorded'] for x in r.get('producer_reached_contract_observations',[])+r.get('expanded_producer_contract_observations',[])]
  reg=item['local_registration'];raw=(root/reg['path']).read_bytes();assert sha(raw)==reg['sha256'];regrow=next(x for x in json.loads(raw)['roles'] if x['index']==i);assert regrow['sha256']==item['sha256'] and regrow['ranges']==[[1,n]]
  old=archive.extractfile(item['sha256']).read();c=item['current_comparison'];body=contents(c['preserved_bytes']);assert sha(old)==row['historical_sha256'] and old==body and len(old.splitlines())==n and item['new_personal_read_ranges']==[[1,n]] and c['current_personal_ranges']==[[1,n]]
  assert c['byte_identical'] and sha(body)==c['sha256'] and len(body)==c['bytes'] and oid('blob',body)==c['git_blob']==tree[c['path']];chain(c['path'],proof['paths'][c['path']],old,proof['root_tree'],nodes);bodies[i]=body
sources={}
for item in a['current_supporting_sources']:
 body=contents(item['preserved_bytes']);assert item['revision']==rev and sha(body)==item['sha256'] and len(body)==item['bytes'] and len(body.splitlines())==item['lines'] and all(1<=lo<=hi<=item['lines'] for lo,hi in item['ranges']) and oid('blob',body)==item['git_blob']==tree[item['path']];sources[item['path']]=body
grid=bodies[2210];embedded=bodies[2211];lacing=bodies[2212];layout=bodies[2213];tiling=bodies[2214];surface=bodies[2215];template=bodies[2216];skin=sources['crates/fabelgeist-armor/src/skin.rs'];design=sources['crates/fabelgeist-armor/src/design.rs'];construction=sources['crates/fabelgeist-armor/src/construction.rs'];rig=sources['crates/fabelgeist-rig/src/skin.rs']
assert b'pub const MAX_TILES: usize = 20_000' in tiling and b'Layout::new(&surfaces, plate, radius, MAX_TILES)?' in tiling and layout.index(b'layout.lay_grid(grid, surface, plate)?')<layout.index(b'layout.tiles.len() > max_tiles') and b'let mut grid_rows = Vec::with_capacity(rows)' in layout and b'let fits = ((width + plate.gap) / pitch).floor().max(1.0) as usize' in layout and b'width: 0.045' in construction and b'gap: 0.0015' in construction
assert 2000003//93==21505 and 21505>20000
assert b'pub fn regular(rows: u32, columns: u32, cyclic: bool, vertices: Vec<u32>)' in grid and b'i as u32 % columns' in grid and b'pub rows: u32' in grid and b'pub columns: u32' in grid and b'grid.rows * grid.columns' in surface and b'self.vertices[(row * self.columns + column) as usize]' in grid and b'GridSurface::is_spanning(grid)' in tiling and 65536*65536==1<<32
assert b'Self(column as f32)' in grid and b'self.0 > (columns - 1) as f32' in grid and b'let first = self.0.floor() as u32' in grid and b'(first + 1).min(columns - 1)' in grid
# Binary32 spacing immediately below 2^32 is 256; both integers lie above the
# halfway point, so nearest rounding selects 2^32 without executing a cast.
assert (1<<32)-1>(1<<32)-128 and (1<<32)-2>(1<<32)-128
native={x['path']:x for x in a['native_reference_sources']}
for p,x in native.items():assert sha(contents(p))==x['sha256']
ref=contents('native-evidence/rust-nightly-2026-07-13/reference/expressions/operator-expr.html.gz');primitive=contents('native-evidence/rust-nightly-2026-07-13/library/core/src/primitive_docs.rs.gz');assert b'expr.as.numeric.int-as-float' in ref and b'expr.as.numeric.float-as-int' in ref and b'roundTiesToEven' in ref and b'saturate' in ref and b'binary32' in primitive
assert b'pub joint_weights: Vec<[f32; 8]>' in design and b'construction.validate()?' in tiling and b'source.joint_weights[corner]' in embedded and b'weighted_vertex' in embedded and b'if weight <= 0.0' in skin and b'influence.weight / total' in skin and b'pub fn weighted_vertex' in rig and b'pub fn push(&mut self, joints:' in rig
assert b'[s, t] = [s.clamp(0.0, 1.0), t.clamp(0.0, 1.0)]' in surface and b'self.plate.hole_pairs == 0' in construction and b'pairs - 1' in lacing and b'ConstrainedDelaunayTriangulation' in template
transport=json.loads(contents('recovered/original-transport-manifest.json.gz'));lookup={x['path']:x for x in transport['entries']};records={}
for item in a['recovered_historical_records']:
 original=lookup[item['original_path']];raw=contents(item['preserved_bytes']);assert (item['sha256'],item['bytes'],item['git_blob'])==(original['sha256'],original['bytes'],original['git_blob']) and sha(raw)==item['published_sha256'] and len(raw)==item['published_bytes']
 if item['redacted_local_path_occurrences']:assert 'pinned complete059 transport' in item['original_recovery'] and raw.count(b'<LOCAL_PATH_')==item['redacted_local_path_occurrences']
 else:assert sha(raw)==item['sha256'] and oid('blob',raw)==item['git_blob']
 records[item['original_path']]=raw
for batch,indices in [(734,[2210,2211]),(735,[2212,2213,2214,2215]),(736,[2216,2217,2218,2219])]:
 path=f'segovb/READ-L12-{batch}.json';data=json.loads(records[path]);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert data['root_personal_rereads']==[] and data['delegated_attribution_preserved'] and data['accepted_unique_count_change']==0 and check['receipt_sha256']==lookup[path]['sha256'] and data['proposal'] in records
 for i in indices:
  x=next(z for z in data['rows'] if z['original_index']==i);n=roles[i-2210]['lines'];assert x['source_sha256']==roles[i-2210]['sha256'] and 'Subagent /root/lens12_frontier' in x['attribution'] and x['fresh_personal_body_reads']==x['assessed_ranges']==[[1,n]]
 for reg in data['adopted_registration_mapping']:assert reg['path'] in records and reg['sha256']==lookup[reg['path']]['sha256']
for batch,indices in [(350,[2210,2211]),(351,[2212,2213,2214,2215]),(352,[2216,2217,2218]),(353,[2219])]:
 data=json.loads(records[f'adler/READ-L11-{batch}.json']);assert data['accepted_unique_count_change']==0
 for i in indices:
  row=next(x for x in data['sources'] if x['original_index']==i);assert row['fresh_reads']==[[1,roles[i-2210]['lines']]] and row['sha256']==roles[i-2210]['sha256']
adler350=json.loads(records['adler/READ-L11-350.json']);d=next(x['disposition'] for x in adler350['role_dispositions'] if x['source_indices']==[2210]);assert 'columns0 panicsmod' in d and 'first+1overflowhypothetical' in d
adler351=json.loads(records['adler/READ-L11-351.json']);d=next(x['disposition'] for x in adler351['role_dispositions'] if x['source_indices']==[2213]);assert 'checkafterlay_gridcomplete' in d
q=load('RECONCILIATION-059.json')['entries'];old=load('baseline-059/queue.json')['entries'];ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries']
for i in (141,142,1819,1820,1821):assert q[i]['contract_id']==old[i]['contract_id'] and q[i]['current_reconciliation'].startswith('partial ') and name in q[i]['current_reconciliation']
assert q[167]['contract_id']==old[167]['contract_id']
for finding in a['reconnected_historical_findings']:assert old[finding['queue']]['contract_id']==finding['id']
for finding in a['reconnected_historical_findings']+a['new_current_candidates']:
 row=next(x for x in ledger if x['id']==finding['id']);assert row['record']==name and row['repair']=='not started' and row['validation']=='not performed'
entry=next(x for x in ledger if x['id']=='ADLER-L11-TILE-TEMPLATE-HOLE-ADMISSION');assert name in entry['additional_current_records']
assert a['repair']=='not started' and a['validation']=='not performed'
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0 and load(load('BASELINE-95E-20261110.json')['latest_main_recheck'])['observed_revision']==rev
print('Verified historical tiling budget, public grid conditions, source-only arithmetic and skin admission; no product execution.')
