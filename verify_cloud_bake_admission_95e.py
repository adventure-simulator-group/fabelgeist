#!/usr/bin/env python3
"""Verify source provenance and historical scopes, not cloud execution."""
import difflib,gzip,hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent
load=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();contents=lambda p:gzip.decompress((r/p).read_bytes())
def oid(t,b):return hashlib.sha1(t.encode()+b' '+str(len(b)).encode()+b'\0'+b).hexdigest()
def entries(b):
 out={};pos=0
 while pos<len(b):
  end=b.index(b'\0',pos);mode,name=b[pos:end].split(b' ',1);out[name.decode()]=(mode.decode(),b[end+1:end+21].hex());pos=end+21
 assert pos==len(b);return out
advance=load('MAIN-ADVANCE-95E.json');tree={}
for line in (r/advance['trees']['current']['path']).read_text().splitlines():
 head,path=line.split('\t',1);mode,kind,hash_=head.split();tree[path]=hash_
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-CLOUD-BAKE-ADMISSION-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1526-1529.json');w=load('COVERAGE-L01-ROLES.json');proof=load('CLOUD-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes={**shared['tree_objects']}
for f in proof['additional_shared_tree_objects']:nodes.update(load(f)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
assert [x['original_index'] for x in h['historical_source_roles']]==[1526,1527,1528,1529] and [x['lines'] for x in h['historical_source_roles']]==[56,1692,138,13]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1526-1529.json' and x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines']
  c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert c['byte_identical']==(raw==cur)==(i in (1528,1529)) and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']];current[i]=cur
  assert all(1<=lo<=hi<=c['lines'] for lo,hi in a['current_role_ranges'][str(i)])
  if raw!=cur:
   d=c['diff'];diff=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='historical-6533/'+c['path'],tofile='current-95e/'+c['path'])).encode();assert contents(d['preserved_bytes'])==diff and sha(diff)==d['sha256'] and len(diff)==d['bytes']
  p=proof['paths'][c['path']];assert [s['name'] for s in p['chain']]==c['path'].split('/');curtree=proof['root_tree']
  for step in p['chain']:
   info=nodes[curtree];b=contents(info['preserved_bytes']);assert oid('tree',b)==curtree==step['tree'] and sha(b)==info['sha256'] and len(b)==info['bytes']
   assert entries(b)[step['name']]==(step['mode'],step['child']);curtree=step['child']
  assert curtree==p['git_blob']==oid('blob',raw) and p['sha256']==x['sha256']
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
asset=current[1526];cloud=current[1527];noise=current[1528];streams=current[1529];bake=s['crates/adventuresim-tactical-client/src/presentation/clouds/bake.rs'];shader=s['assets/shaders/tactical_clouds.wgsl'];scenery=s['crates/adventuresim-tactical-client/src/art_demo/scenery.rs'];config=s['assets/config/tactical-graphics.yaml'];js=s['crates/strategic-web/static/strategic-renderer.js'];main=s['crates/adventuresim-tactical-client/src/main.rs'];plugin=s['crates/adventuresim-tactical-client/src/presentation/mod.rs'];source=s['crates/adventuresim-tactical-core/src/scene_input.rs'];descriptor=s['crates/adventuresim-tactical-core/src/scene_input/descriptor.rs'];generation=s['crates/adventuresim-tactical-core/src/scene_input/generation.rs'];strategic=s['crates/adventuresim-tactical-client/src/presentation/generation/scene.rs'];benchmark=s['crates/adventuresim-tactical-client/src/tactical_scene_viewer.rs']
assert b'image_from_rgba8(asset.rgba8.to_vec())' in asset and b'pixels.len()' in asset and b'pub(super) fn integrated_pixel' in bake and b'for (slot, layer) in self.key.layers.iter().flatten().enumerate()' in bake and b'let mut optical_depth = 0.0;' in bake and b'let alpha = 1.0 - (-optical_depth).exp();' in bake and b'(alpha * 255.0).round() as u8' in bake
assert b'layers: [None, None, None]' in cloud and b'let baked_texture_a = images.add(cloud_bake_image(&empty_request))' in cloud and b'let baked_texture_b = images.add(cloud_bake_image(&empty_request))' in cloud
assert b'if let Some(prebaked) = prebaked.as_deref()' in cloud and b'#[cfg(not(target_family = "wasm"))]\n        advance_cloud_bake_pipeline(' in cloud and b'#[cfg(target_family = "wasm")]\n        {\n            animation_status.ready = true;' in cloud
assert b'let ray_opacity = baked.r;' in shader and b'if opacity <= 0.002' in shader and b'let wind = cloud_motion.xy;' in shader
assert b'if asset_scene_digest != cloud_asset_digest' in scenery and b'PrebakedCloudEnvironment { rgba8 }' in scenery and b'let cloud_asset_digest = input.digest()?' in scenery
assert b'  clouds:\n    enabled: true' in config and b'fetch("/tactical/assets/config/tactical-graphics.yaml")' in js and b'runtime.wasm_boot(generationGraphicsConfig' in js and b'TacticalPresentationPlugin {' in main and b'setup_tactical_clouds' in plugin
assert b'if active_clouds > 0 && !benchmark_diagnostics.cloud_animation.is_ready()' in benchmark
key=cloud[cloud.index(b'struct CloudBakeKey {'):cloud.index(b'#[derive(Clone, Debug)]\nstruct CloudBakeRequest')];assert b'layers:' in key and b'seed:' in key and b'wind_velocity' not in key
assert b'wind_velocity: completed.request.wind_velocity' in cloud and b'let wind_velocity = cloud_wind_velocity(environment);' in cloud and b'environment.scene_digest.as_bytes()' in noise
assert b'pub weather: WeatherSnapshot' in descriptor and b'let bytes = serde_json::to_vec(self)?' in source and b'weather: self.weather' in source and b'digest: self.digest()?' in generation and b'digest: input.digest()?' in strategic
assert b'profile: f32' in cloud and b'let kind = layer.profile as u32;' in cloud and b'CloudForm::CumulusCongestus => (10.0' in cloud and b'pub(super) const ENVIRONMENT: StreamId' in streams
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
 if x['redacted_local_path_occurrences']:assert 'pinned 059 transport' in x['original_recovery']
for batch,indices in [(466,[1525,1526]),(467,[1527]),(468,[1528,1529])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
qualification=json.loads(records['segovb/QUALIFICATION-L12-ORIGINAL1526-LOCATOR.json']);assert qualification['actual_EOF']==56 and qualification['inherited_requested_ranges']==[[1,56]] and qualification['new_population_credit']==0 and '\\' in qualification['recorded_path']
for suffix,fresh,tail in [('001',[[1,600]],[[601,1692]]),('002',[[601,1200]],[[1201,1692]]),('003-EOF',[[1201,1692]],[])]:
 x=json.loads(records[f'segovb/PARTIAL-L12-467-{suffix}.json']);assert x['fresh_personal_body_reads']==fresh and x['unread_tail']==tail and x['actual_EOF']==1692
support=json.loads(records['segovb/SUPPORT-READ-L12-466-001.json']);assert support['qualified_supporting_same_lens_reuse'][0]['original_index']==1449 and support['qualified_supporting_same_lens_reuse'][0]['fresh_personal_body_reads_this_batch']==[]
for batch in (216,217,218):
 receipt=json.loads(records[f'adler/READ-L11-{batch}.json']);check=json.loads(records[f'adler/VERIFY-L11-{batch}.json']);assert receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False
 assert check['receipt_sha256']==m[f'adler/READ-L11-{batch}.json']['sha256'] and check['state_sha256']==m[f'adler/STATE-L11-{batch}.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1300,1301,1302));old=load('baseline-059/queue.json')['entries'];assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in (1300,1301,1302)]==[1526,1527,1528]
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];prior=load('baseline-059/ledger.json')['discovery_findings_and_dependencies'];assert len(a['conditional_findings'])==3 and all(c['id'] in prior and sum(x['id']==c['id'] and x['record']=='ASSESSMENT-CLOUD-BAKE-ADMISSION-95E.json' for x in ledger)==1 and c['accepted_historical_unique_count_change']==0 for c in a['conditional_findings'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified four historical cloud bodies, source diffs, earlier partial/locator attribution and bounded current connections; no product execution.')
