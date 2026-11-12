#!/usr/bin/env python3
"""Verify retained source and review attribution; no product execution."""
import difflib,gzip,hashlib,json,tarfile
from pathlib import Path
# Qualified successor, generated from the historical verifier. Only the
# named assessment and its two gzip paths may resolve through the overlay.
import argparse
ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,default=Path(__file__).resolve().parent);ap.add_argument('--overlay',type=Path);args=ap.parse_args()
r=args.publication
_overlay_paths={'ASSESSMENT-CITY-STREAMING-READINESS-95E-CORRECTED.json','recovered/city-streaming-059/66f8a7c9ae457dfa40e9e89a0c8de7fce95b97d222de2e211fc48b6b16f53f63.gz','recovered/city-streaming-059/537934fe1102efa623885d1bb2db72868f72f6c3193c8fdc20f28f2981cac49a.gz'}
def resolved(p):
 p=str(p)
 return (args.overlay/p if args.overlay is not None and p in _overlay_paths else r/p)
load=lambda p:json.loads(resolved(p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();contents=lambda p:gzip.decompress(resolved(p).read_bytes())
assert sha(resolved('ASSESSMENT-CITY-STREAMING-READINESS-95E-CORRECTED.json').read_bytes())=='7eff3ca86b37c5a8b3738a2d96cf17d1916bce1168b9cf81c1cc553063329b6c'

def oid(t,b):return hashlib.sha1(t.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};pos=0
 while pos<len(b):
  end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
 assert pos==len(b);return out
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (r/advance['trees']['current']['path']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,hash_=head.split();tree[path]=hash_
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-CITY-STREAMING-READINESS-95E-CORRECTED.json');h=load('ASSESSMENT-L01-HISTORICAL-1521-1525.json');w=load('COVERAGE-L01-ROLES.json');proof=load('CITY-STREAMING-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes={**shared['tree_objects']}
for f in proof['additional_shared_tree_objects']:nodes.update(load(f)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
assert [x['original_index'] for x in h['historical_source_roles']]==list(range(1521,1526)) and [x['lines'] for x in h['historical_source_roles']]==[568,54,155,272,148]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1521-1525.json' and x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines']
  c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert c['byte_identical']==(raw==cur) and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']];current[i]=cur
  if i==1521:
   assert raw!=cur and raw.split(b'#[cfg(test)]')[0]==cur.split(b'#[cfg(test)]')[0]
   d=c['diff'];diff=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='historical-6533/'+c['path'],tofile='current-95e/'+c['path'])).encode();assert contents(d['preserved_bytes'])==diff and sha(diff)==d['sha256'] and len(diff)==d['bytes']
  else:assert raw==cur
  p=proof['paths'][c['path']];assert [s['name'] for s in p['chain']]==c['path'].split('/');curtree=proof['root_tree']
  for step in p['chain']:
   info=nodes[curtree];b=contents(info['preserved_bytes']);assert oid('tree',b)==curtree==step['tree'] and sha(b)==info['sha256'] and len(b)==info['bytes']
   assert entries(b)[step['name']]==(step['mode'],step['child']);curtree=step['child']
  assert curtree==p['git_blob']==oid('blob',raw) and p['sha256']==x['sha256']
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
prepared=current[1522];signs=current[1523];stream=current[1524];closure=current[1525]
art=s['crates/adventuresim-tactical-client/src/art_demo.rs'];status=s['crates/adventuresim-tactical-client/src/strategic_scene/status.rs'];cached=s['crates/adventuresim-tactical-client/src/strategic_scene/cached_views.rs'];visibility=s['crates/adventuresim-tactical-client/src/strategic_scene/visibility.rs'];js=s['crates/strategic-web/static/strategic-scene.js'];installation=s['crates/adventuresim-tactical-client/src/presentation/generation/installation.rs'];windows=s['crates/adventuresim-tactical-client/src/presentation/windows.rs'];dimensions=s['crates/adventuresim-building-generator/src/spatial_geometry/dimensions.rs']
assert b'interior: Some(' in prepared and prepared.index(b'let origin =')<prepared.index(b'lod0: batches(') and b'sign.mount = mount;' in signs and b'LETTERING_LOAD_DISTANCE_METRES: f32 = 48.0' in signs and b'LETTERING_RELEASE_DISTANCE_METRES: f32 = 60.0' in signs and b'camera.is_active' in signs
assert b'let placement = self.placements.pop_front()' in stream and b'Err(error) => {' in stream and b'self.report_failure(error)' in stream and b'self.total - self.placements.len()' in stream and b'self.placements.is_empty()' in stream and b'failure: Option<String>' in stream and b'Ok(false) => self.placements.push_back(placement)' in stream
assert b'.position(|facade| facade.program == *program)' in installation and b'product: ProductKind::Facade' in installation and b'Ok(products.facades.swap_remove(index))' in installation
assert b'if !pending.finished()' in art and art.index(b'if !pending.finished()')<art.index(b'else if let Some(message) = pending.failure()') and b'DemoStatus::Unavailable' in art
installed=status[status.index(b'let installed ='):status.index(b'installed_ready.0 = installed;')];complete=status[status.index(b'let complete ='):status.index(b'assets_ready.0 = complete;')]
assert b'city.finished()' in installed and b'city.failure()' not in installed and b'city.failure()' not in complete and b'city_gpu_ready(PresentationOwner::Scene)' in complete and b'city.failure().map(str::to_owned)' in status
assert b'let may_prepare = installed.0 && !scene_changed;' in cached and b'if camera.is_active && ready.0' in cached and b'frame.remaining = frame.remaining.saturating_sub(1);' in cached and b'*opened |= installed.0' in visibility
assert js.index(b'if (state.error)')<js.index(b'const ready =')
assert b'compile_window_leaf(size.cuboid(), kind, ClosureState::Operable)?' in closure and b'MissingPrimaryMaterial { kind: window.leaf }' in windows and b'Self::from_metres(Vec3::deserialize(d)?)' in dimensions and b'finite(value, role)?' in dimensions and b'if positive && component == 0.0' in dimensions
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
# The historical transport identity is an original-to-public crosswalk, not
# a claim that the publicly redacted JSON equals the original JSON.
_expected_redactions=[{'original_path': 'adler/REGISTRATION-ADLER-352.json', 'original_city_publication_path': 'recovered/city-streaming-059/66f8a7c9ae457dfa40e9e89a0c8de7fce95b97d222de2e211fc48b6b16f53f63.gz', 'original_decoded_sha256': '66f8a7c9ae457dfa40e9e89a0c8de7fce95b97d222de2e211fc48b6b16f53f63', 'original_decoded_bytes': 1822, 'original_decoded_git_blob': '4443fc6c4d0af983dfb8fb7016580eaa203e8811', 'cloud_publication_path': 'recovered/cloud-059/66f8a7c9ae457dfa40e9e89a0c8de7fce95b97d222de2e211fc48b6b16f53f63.gz', 'cloud_gzip_sha256': '82b9471f873416d514f7c9b5aed52f1afe5901a9a53ab2f0425c5f0b7afae112', 'cloud_gzip_bytes': 891, 'cloud_decoded_sha256': 'db3da4b2f0b6dc3d16aa448c5089b9b60ae62fd8400cb1eb72b956b6951c89c8', 'cloud_decoded_bytes': 1721}, {'original_path': 'adler/REGISTRATION-ADLER-353.json', 'original_city_publication_path': 'recovered/city-streaming-059/537934fe1102efa623885d1bb2db72868f72f6c3193c8fdc20f28f2981cac49a.gz', 'original_decoded_sha256': '537934fe1102efa623885d1bb2db72868f72f6c3193c8fdc20f28f2981cac49a', 'original_decoded_bytes': 1831, 'original_decoded_git_blob': '7222745855fae84e10b3d3964d9c743aadbaeacd', 'cloud_publication_path': 'recovered/cloud-059/537934fe1102efa623885d1bb2db72868f72f6c3193c8fdc20f28f2981cac49a.gz', 'cloud_gzip_sha256': 'd4dfb06c8be60de5f776b06afa637a29aa4ff94a8ba441353aad96de38478b5c', 'cloud_gzip_bytes': 899, 'cloud_decoded_sha256': '96f7f274a6b9d50a196a528eb1be9cad5d5d3ddfdc2aa870a898ddeb4401b415', 'cloud_decoded_bytes': 1724}]
for expected in _expected_redactions:
 matches=[x for x in a['recovered_historical_records'] if x['original_path']==expected['original_path']]
 assert len(matches)==1
 x=matches[0];original=m[x['original_path']]
 assert (x['sha256'],x['bytes'],x['git_blob'])==(expected['original_decoded_sha256'],expected['original_decoded_bytes'],expected['original_decoded_git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 assert x['preserved_bytes']==expected['original_city_publication_path'] and x['redacted_local_path_occurrences']==1
 assert (x['published_sha256'],x['published_bytes'])==(expected['cloud_decoded_sha256'],expected['cloud_decoded_bytes'])
 public=resolved(x['preserved_bytes']).read_bytes();cloud=(r/expected['cloud_publication_path']).read_bytes()
 assert public==cloud and (sha(public),len(public))==(expected['cloud_gzip_sha256'],expected['cloud_gzip_bytes'])
 plain=gzip.decompress(public);assert (sha(plain),len(plain))==(expected['cloud_decoded_sha256'],expected['cloud_decoded_bytes'])
 assert json.loads(plain)['provenance']['cache_path']=='<LOCAL_PATH_0>'

for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for batch,indices in [(423,[1449]),(464,[1521]),(465,[1522,1523,1524]),(466,[1525,1526])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
reg=json.loads(records['segovb/REGISTRATION-SEGOVB-L12-429.json']);assert reg['actual_line_count']==54 and reg['ranges']==[[1,54]] and '\\' in reg['recorded_registry_path'] and 'inherited1..50' in reg['purpose']
for batch in (215,216):
 receipt=json.loads(records[f'adler/READ-L11-{batch}.json']);check=json.loads(records[f'adler/VERIFY-L11-{batch}.json']);assert receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False
 assert check['receipt_sha256']==m[f'adler/READ-L11-{batch}.json']['sha256'] and check['state_sha256']==m[f'adler/STATE-L11-{batch}.json']['sha256']
assert [x['original_index'] for x in json.loads(records['adler/READ-L11-216.json'])['sources']]==[1524,1525,1526,1527,1528,1529]
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1298,1299,1300));old=load('baseline-059/queue.json')['entries'];assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in (1298,1299,1300)]==[1521,1524,1526]
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(a['conditional_findings'])==2 and all(sum(x['id']==c['id'] and x['record']=='ASSESSMENT-CITY-STREAMING-READINESS-95E.json' for x in ledger)==1 and c['accepted_historical_unique_count_change']==0 for c in a['conditional_findings'])
assert a['conditional_findings'][0]['id'] in load('baseline-059/ledger.json')['discovery_findings_and_dependencies'] and a['conditional_findings'][1]['id'].startswith('REVIEW-')
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified qualified city source/evidence assertions and original-to-redacted 059 crosswalks; no product execution.')
