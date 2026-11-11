#!/usr/bin/env python3
"""Verify source provenance and static contracts; no product execution."""
import difflib,gzip,hashlib,json,tarfile,tomllib
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
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-GRAPHICS-CONFIG-ADMISSION-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1530-1534.json');w=load('COVERAGE-L01-ROLES.json');proof=load('GRAPHICS-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);nodes={**shared['tree_objects']}
for f in proof['additional_shared_tree_objects']:nodes.update(load(f)['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
assert [x['original_index'] for x in h['historical_source_roles']]==[1530,1531,1532,1533,1534] and [x['lines'] for x in h['historical_source_roles']]==[414,61,577,127,89]
current={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1530-1534.json' and x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines']
  c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert c['byte_identical']==(raw==cur)==(i!=1530) and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']];current[i]=cur
  assert all(1<=lo<=hi<=c['lines'] for lo,hi in a['current_role_ranges'][str(i)])
  if raw!=cur:
   d=c['diff'];diff=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile='historical-6533/'+c['path'],tofile='current-95e/'+c['path'])).encode();assert contents(d['preserved_bytes'])==diff and sha(diff)==d['sha256'] and len(diff)==d['bytes']
   assert b'pub casts_shadows: GrassShadowTiers' in raw and b'pub casts_shadows: GrassShadowTiers' not in cur
  p=proof['paths'][c['path']];assert [s['name'] for s in p['chain']]==c['path'].split('/');curtree=proof['root_tree']
  for step in p['chain']:
   info=nodes[curtree];b=contents(info['preserved_bytes']);assert oid('tree',b)==curtree==step['tree'] and sha(b)==info['sha256'] and len(b)==info['bytes']
   assert entries(b)[step['name']]==(step['mode'],step['child']);curtree=step['child']
  assert curtree==p['git_blob']==oid('blob',raw) and p['sha256']==x['sha256']
assert h['historical_source_roles'][2]['prior_role_status']=='bounded-current-trace-only' and h['historical_source_roles'][2]['prior_current_observations']
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
cfg=current[1530];doors=current[1531];environment=current[1532];furniture=current[1533];generation=current[1534];grass=s['crates/adventuresim-tactical-client/src/presentation/ground_scatter/instanced_grass.rs'];mesh=s['crates/adventuresim-tactical-client/src/presentation/ground_scatter/grass.rs'];diag=s['crates/adventuresim-tactical-client/src/presentation/ground_scatter/instanced_grass/diagnostics.rs'];material=s['crates/adventuresim-tactical-client/src/presentation/ground_scatter/instanced_grass/material.rs'];shader=s['assets/shaders/tactical_grass_instanced.wgsl'];player=s['crates/adventuresim-tactical-client/src/player.rs'];city=s['crates/adventuresim-tactical-client/src/presentation/generation/city.rs'];prep=s['crates/adventuresim-tactical-client/src/presentation/preparation.rs']
assert b'serde_saphyr::from_str(text)' in cfg and b'config.validate()?' in cfg and b'root_occlusion <= 0.0 || self.grass.lighting.root_occlusion > 1.0' in cfg and b'interaction.maximum_push < interaction.minimum_push' in cfg and b'value.is_finite() && value > 0.0' in cfg
assert b'native_tufts_per_cell_side == 0 || tier.native_blades_per_tuft_side == 0' in cfg and b'} as i32;' in grass and b'for tuft_z in 0..side' in grass and b'0..grid_side * grid_side' in mesh
assert b'grass.lighting.root_occlusion' in diag and b'params: material.params' in material and b'mix(grass.params.x, 1.0' in shader and b'GrassInteractor,' in player
assert b'interactors.iter().next()' in grass and b'for (_, material) in materials.iter_mut()' in grass and b'.clamp(interaction.minimum_push, interaction.maximum_push)' in grass
assert b'GraphicsConfiguration { message }' in city and b'GraphicsConfiguration { message: String }' in prep and b'retain(|entity| environments.contains(*entity))' in environment and b'CityGate::owns_opening' in doors and b'Transform::IDENTITY' in furniture and b'Self::RegionalCity { .. } => ticket.require_owner(PresentationOwner::RegionalMap)' in generation
lock=tomllib.loads(s['Cargo.lock'].decode());native={}
for folder in ('serde-saphyr-1.2.0','serde_core-1.0.229'):
 m=load('native-evidence/'+folder+'/MANIFEST.json');archive=r/'native-evidence'/folder/m['archive'];raw=archive.read_bytes();pkg=next(x for x in lock['package'] if x['name']==m['package'] and x['version']==m['version']);assert sha(raw)==m['sha256']==pkg['checksum'] and len(raw)==m['bytes']
 with tarfile.open(archive,'r:gz') as t:
  for path,ranges in a['native_source_evidence']['selected_ranges'].items():
   if not path.startswith(folder+'/'):continue
   b=t.extractfile(path).read();member=next(x for x in m['inspected_members'] if x['path']==path);assert sha(b)==member['sha256'] and len(b)==member['bytes'] and all(1<=lo<=hi<=len(b.splitlines()) for lo,hi in ranges);native[path]=b
rust=load('native-evidence/rust-nightly-2026-07-13/MANIFEST.json');p='library/core/src/num/f32.rs';b=(r/'native-evidence/rust-nightly-2026-07-13'/p).read_bytes();entry=next(x for x in rust['files'] if x['path']==p);assert sha(b)==entry['sha256'] and len(b)==entry['bytes'] and b'channel = "nightly-2026-07-13"' in s['rust-toolchain.toml'];assert b'pub const fn clamp(mut self, min: f32, max: f32)' in b and b'min <= max' in b and b'Panics if `min > max`, `min` is NaN, or `max` is NaN.' in b
api=native['serde-saphyr-1.2.0/src/de/api.rs'];options=native['serde-saphyr-1.2.0/src/de/options.rs'];parser=native['serde-saphyr-1.2.0/src/parse_scalars.rs'];de=native['serde-saphyr-1.2.0/src/de/deserializer.rs'];visitor=native['serde_core-1.0.229/src/de/impls.rs']
assert b'from_str_with_options(input, Options::default())' in api and b'angle_conversions: false' in options and b'".nan" | "+.nan" | "-.nan" => Ok(T::nan())' in parser and b'".inf" | "+.inf" => Ok(T::infinity())' in parser and b'visitor.visit_f32(v)' in de and b'num_self!(f32:visit_f32)' in visitor
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for batch,indices in [(469,[1530,1531]),(470,[1532,1533]),(471,[1534,1535,1536])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
for batch in (217,218,219):
 receipt=json.loads(records[f'adler/READ-L11-{batch}.json']);check=json.loads(records[f'adler/VERIFY-L11-{batch}.json']);assert receipt['accepted_unique_count_change']==0 and receipt['current_reconciled'] is False and receipt['validated'] is False
 assert check['receipt_sha256']==m[f'adler/READ-L11-{batch}.json']['sha256'] and check['state_sha256']==m[f'adler/STATE-L11-{batch}.json']['sha256']
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (1303,1304,1305));old=load('baseline-059/queue.json')['entries'];assert [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in (1303,1304,1305)]==[1530,1532,1536]
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];prior=load('baseline-059/ledger.json')['discovery_findings_and_dependencies'];assert len(a['conditional_findings'])==3 and all(c['id'] in prior and sum(x['id']==c['id'] and x['record']=='ASSESSMENT-GRAPHICS-CONFIG-ADMISSION-95E.json' for x in ledger)==1 and c['accepted_historical_unique_count_change']==0 for c in a['conditional_findings'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified five historical graphics bodies, parser/Rust sources, earlier attribution and bounded admission connections; no product execution.')
