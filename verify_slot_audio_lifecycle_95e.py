#!/usr/bin/env python3
"""Verify bounded slot/audio source and historical identities; no product execution."""
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
def inventory(path):
 result={}
 for line in (r/path).read_text().splitlines():
  head,path=line.split('\t',1);mode,kind,hash_=head.split();result[path]=hash_
 return result
rev='95e63628f9d70b8dd6e6fede64334696c6995009';a=load('ASSESSMENT-SLOT-AUDIO-LIFECYCLE-95E.json');h=load('ASSESSMENT-L01-HISTORICAL-1495-1497.json');w=load('COVERAGE-L01-ROLES.json');proof=load('SLOT-AUDIO-HISTORICAL-GIT-PROOFS.json');shared=load(proof['shared_commit_and_tree_objects']);extra=[load(x) for x in proof['additional_shared_tree_objects']];advance=load('MAIN-ADVANCE-95E.json');tree=inventory(advance['trees']['current']['path']);previous=inventory(advance['trees']['previous']['path'])
assert a['source_revision']==rev and a['repair']=='not started' and a['validation']=='not performed'
commit=shared['revisions'][proof['revision']]['commit'];raw=contents(commit['preserved_bytes']);assert oid('commit',raw)==proof['revision'] and raw.splitlines()[0]==b'tree '+proof['root_tree'].encode()
nodes={**shared['tree_objects']}
for p in extra:nodes.update(p['additional_tree_objects'])
nodes.update(proof['additional_tree_objects'])
assert [x['original_index'] for x in h['historical_source_roles']]==[1495,1496,1497] and [x['lines'] for x in h['historical_source_roles']]==[243,609,370]
assert h['historical_source_roles'][0]['prior_role_status']=='bounded-current-trace-only' and h['historical_source_roles'][0]['prior_current_observations']==[{'assessment':'ASSESSMENT-L1-HUD-IDENTITY.json','ranges':[[1,90]]}]
current={};historical={}
with tarfile.open(r/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as ar:
 for x in h['historical_source_roles']:
  i=x['original_index'];role=w['roles'][i];assert role['status']=='full-historical-body-trace-open' and role['historical_assessment']=='ASSESSMENT-L01-HISTORICAL-1495-1497.json' and x['new_personal_read_ranges']==[[1,x['lines']]]
  raw=ar.extractfile(x['sha256']).read();assert sha(raw)==x['sha256']==role['historical_sha256'] and len(raw.splitlines())==x['lines'];historical[i]=raw
  c=x['current_comparison'];cur=contents(c['preserved_bytes']);assert (raw==cur)==c['byte_identical'] and sha(cur)==c['sha256'] and len(cur)==c['bytes'] and len(cur.splitlines())==c['lines'] and oid('blob',cur)==c['git_blob']==tree[c['path']];current[i]=cur
  if raw!=cur:
   d=''.join(difflib.unified_diff(raw.decode().splitlines(keepends=True),cur.decode().splitlines(keepends=True),fromfile=f'historical-{i}/'+c['path'],tofile='95e/'+c['path'])).encode();assert d==(r/c['diff']).read_bytes() and sha(d)==c['diff_sha256']
  p=proof['paths'][c['path']];assert [s['name'] for s in p['chain']]==c['path'].split('/');curtree=proof['root_tree']
  for step in p['chain']:
   info=nodes[curtree];b=contents(info['preserved_bytes']);assert oid('tree',b)==curtree==step['tree'] and sha(b)==info['sha256'] and len(b)==info['bytes']
   assert entries(b)[step['name']]==(step['mode'],step['child']);curtree=step['child']
  assert curtree==p['git_blob']==oid('blob',raw) and p['sha256']==x['sha256']
assert [i for i in current if current[i]!=historical[i]]==[1496,1497]
s={}
for x in a['current_supporting_sources']:
 b=contents(x['preserved_bytes']);assert x['revision']==rev and sha(b)==x['sha256'] and len(b)==x['bytes'] and len(b.splitlines())==x['lines'] and oid('blob',b)==x['git_blob']==tree[x['path']]
 assert all(1<=lo<=hi<=x['lines'] for lo,hi in x['ranges']);s[x['path']]=b
for c in a['prior_source_identity_comparisons']:
 prior=load(c['prior_assessment']);assert c['prior_revision']==prior['current_revision'] and len(c['sources'])==len(prior['current_sources'])
 for x,p in zip(c['sources'],prior['current_sources']):
  b=contents(x['preserved_bytes']);assert (x['path'],x['sha256'],x['git_blob'],x['prior_assessed_ranges'])==(p['path'],p['sha256'],p['git_blob'],p['ranges'])
  assert sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']==tree[x['path']]==previous[x['path']] and x['current_revision']==rev
assert [len(x['sources']) for x in a['prior_source_identity_comparisons']]==[11,15]
config=s['crates/adventuresim-tactical-client/src/audio_config.rs'];sound=s['crates/adventuresim-tactical-client/src/sound_output.rs'];web=s['crates/adventuresim-tactical-client/src/sound_output/web.rs'];silent=s['crates/adventuresim-tactical-client/src/sound_output/silent.rs'];js=s['crates/adventuresim-stdb-module/static/sfx.js'];worklet=s['crates/adventuresim-stdb-module/static/sfx_worklet.js'];rain=s['crates/adventuresim-tactical-client/src/weather_audio.rs'];env=s['crates/adventuresim-tactical-client/src/presentation/environment.rs'];slot=current[1495];move=current[1497];browser=s['crates/adventuresim-tactical-client/src/browser_runtime.rs'];world=s['crates/adventuresim-tactical-client/src/strategic_scene/world.rs'];vista=s['crates/adventuresim-tactical-client/src/presentation/vista/strategic.rs'];scene=s['crates/adventuresim-tactical-client/src/strategic_scene/mod.rs']
assert b'(0..mapping.locations.len()).find_map' in slot and b'% mapping.locations.len()' in slot and b'.filter(|(input, _)| *input == mapping.input)' in slot
assert b'movement.wind.minimum_audible_speed_bps\n            >=' in config and b'value.is_finite() && value > 0.0' in config and b'range[1] >= range[0]' in config
assert b'base_speed * PlaybackSpeed::new(pitch)' in move and b'Self(self.0 * other.0)' in sound and b'js_name = sfxPlay' in web
assert b'this.voiceSpeed[voice] = data[at + 3]' in worklet and b'position += speed;' in worklet and b'if (position >= length)' in worklet and b'const voice = this.freeVoice()' in worklet
assert b'let sound = sounds.get(path);\n      if (sound) return sound;' in js and b'sounds.set(path, sound)' in js and b'if (!audible() || !sound.ready || starts === MAX_STARTS)' in js and b'if (gain > 0 && loop.sound === null) loop.sound = load(path)' in js and b'sounds.delete' not in js and b'sounds.clear' not in js and b'.catch((error) => console.warn' in js
assert b'pub(super) fn play_spatial(_asset_path:' in silent and b'play_spatial(' in move and b'AudioPlayer::new' in historical[1497] and b'AudioPlayer::new' not in move
assert b'.activation_order.push(event.entity)' in env and b'.rev()' in env and b'environments.get(entity)' in env and b'.iter()\n        .next()\n        .map(|environment| environment.weather)' in rain
assert b'let Some(environment) = environments.iter().next() else {\n        return;' in move and b'loopGains[loop.slot] = gain;' in js and b'commands.spawn((environment.clone(), backdrop, ChildOf(root)))' in vista and b'world.get_entity_mut(root)' in scene and b'entity.despawn()' in scene and b'commands.queue(crate::strategic_scene::release_scene)' in browser and b'retained.root = Some(root)' in world
native=load('native-evidence/bevy_ecs-0.19.1/MANIFEST.json');member='bevy_ecs-0.19.1/src/hierarchy.rs';record=next(x for x in native['inspected_members'] if x['path']==member)
with tarfile.open(r/'native-evidence/bevy_ecs-0.19.1'/native['archive'],'r:gz') as ar:b=ar.extractfile(member).read()
assert sha(b)==record['sha256'] and len(b)==record['bytes'] and b'When a parent is despawned, all children' in b and b'#[relationship_target(relationship = ChildOf, linked_spawn)]' in b
manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}
for x in a['recovered_historical_records']:
 original=m[x['original_path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 b=contents(x['preserved_bytes']);assert sha(b)==x['published_sha256'] and len(b)==x['published_bytes'] and (x['redacted_local_path_occurrences'] or (sha(b)==x['sha256'] and oid('blob',b)==x['git_blob']));records[x['original_path']]=b
for batch,indices in [(451,[1496]),(452,[1497])]:
 receipt=json.loads(records[f'segovb/READ-L12-{batch}.json']);check=json.loads(records[f'segovb/VERIFY-L12-{batch}.json']);assert [x['original_index'] for x in receipt['rows']]==indices and receipt['accepted_unique_count_change']==0 and receipt['current_source_reconciled'] is False
 assert check['receipt_sha256']==m[f'segovb/READ-L12-{batch}.json']['sha256'] and check['state_sha256']==m[f'segovb/STATE-L12-{batch}.json']['sha256']
for who,batch in [('adler',212),('adler',256),('adler',217),('adler',241),('segovb',683),('segovb',685)]:
 receipt=json.loads(records[f'{who}/READ-L11-{batch}.json']);check=json.loads(records[f'{who}/VERIFY-L11-{batch}.json']);assert receipt['accepted_unique_count_change']==0 and check['receipt_sha256']==m[f'{who}/READ-L11-{batch}.json']['sha256'] and check['state_sha256']==m[f'{who}/STATE-L11-{batch}.json']['sha256']
 for field in ('proposal','agent_verification'):
  if field in receipt:assert receipt[field] in records and m[receipt[field]]['sha256']==receipt[field+'_sha256']
 for plan in receipt.get('plans',[]):assert plan['path'] in records and m[plan['path']]['sha256']==plan['sha256']
alias=json.loads(records['segovb/ALIAS-L11-P01-004.json']);assert alias['scope_revision']=='e2755a732c84e67296b5e3441a4259a84f461b2f' and alias['accepted_unique_count_change']==0
assert all(json.loads(records[f'segovb/READ-L11-{n}.json'])['source_revision']=='e2755a732c84e67296b5e3441a4259a84f461b2f' for n in (683,685))
q=load('RECONCILIATION-059.json')['entries'];assert all(q[i]['current_reconciliation'].startswith('partial ') for i in (62,96,645,646,650,651,1284,1285,1286,1893));old=load('baseline-059/queue.json')['entries'];assert old[62]['contract_id']=='ADLER-L11-MOVEMENT-AUDIO-PITCH-PRODUCT' and old[96]['contract_id']=='ADLER-L11-RAIN-AUDIO-SCENE-OWNERSHIP' and [old[i]['first_failed_contract_or_unresolved_question']['source_original_index'] for i in (1284,1285,1286)]==[1494,1496,1497]
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(a['conditional_findings'])==5 and all(sum(x['id']==c['id'] and x['record']=='ASSESSMENT-SLOT-AUDIO-LIFECYCLE-95E.json' for x in ledger)==1 for c in a['conditional_findings']) and all(c['accepted_historical_unique_count_change']==0 for c in a['conditional_findings'])
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified three historical slot/audio bodies, current source identities, prior attribution and bounded source connections; no product execution.')
