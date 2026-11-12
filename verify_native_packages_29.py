#!/usr/bin/env python3
"""Verify locked archive checksums and selected members without native execution."""
import hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads((R/p).read_text());cp=load('DELEGATED-EVIDENCE-CHECKPOINT-29.json');bs={b['bundle_id']:b for b in cp['bundles']};work=load('COVERAGE-L01-ROLES.json')['roles']
anchor=bs['wgpu-native-0-27-30']
with tarfile.open(R/anchor['archive']) as t:
 proof=json.load(t.extractfile('git-proofs.json'));ps=proof.get('proofs',proof.get('paths',[]));locks={}
 for label,rev,path in [('old','6533e444fa37552927070cc8d8190907fd3b413e','locks/6533e44/Cargo.lock'),('current','95e63628f9d70b8dd6e6fede64334696c6995009','locks/95e6362/Cargo.lock')]:
  raw=t.extractfile(path).read();blob=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest();assert any(x['revision']==rev and x['path']=='Cargo.lock' and x.get('blob',x.get('git_blob'))==blob and x['sha256']==sha(raw) for x in ps);locks[label]={(x['name'],x['version']):x.get('checksum') for x in tomllib.loads(raw.decode())['package']}
native_names=['wgpu-native-0-27-30','native-anyhow-png-1-2-17','bevy-observer-3-8','bevy-ecs-observer-9-14','direction-parry-15-16','native-serde-saphyr-18-19','stdb-native-20-24','native-tiny-skia-25-26'];archives=0;matches={};reconstructed=[]
for name in native_names:
 b=bs[name];native_roles=[i for i in b['scope_original_roles'] if i<31];found=set()
 with tarfile.open(R/b['archive']) as t:
  for m in b['members']:
   if not m['path'].endswith('.crate'):continue
   raw=t.extractfile(m['path']).read();base=m['path'].rsplit('/',1)[-1][:-6];package,version=base.rsplit('-',1);assert all(lock.get((package,version))==m['original_sha256'] for lock in locks.values())
   if m['changed']:
    assert 'Reconstructed public copy' in m['qualification'];reconstructed.append((name,m['path']))
   else:assert sha(raw)==m['original_sha256']
   with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as crate:
    digest={sha(crate.extractfile(x).read()) for x in crate.getmembers() if x.isfile()}
   found.update(i for i in native_roles if work[i]['historical_sha256'] in digest);archives+=1
 assert found==set(native_roles),(name,sorted(set(native_roles)-found));matches.update({i:name for i in found})
assert set(matches)==set(range(31)) and archives==20
assert reconstructed==[('bevy-observer-3-8','packages/bevy_asset-0.19.1.crate'),('native-serde-saphyr-18-19','native/serde-saphyr-1.2.0.crate')]
print('Verified 20 archived package occurrences against old/current root-lock checksums and all 31 selected source members; two public archives are reconstructed with originals retained locally. No feature resolution or native execution.')
