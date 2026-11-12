#!/usr/bin/env python3
"""Evidence-only q2910 integrity, recovery and optional Git binding."""
import argparse,hashlib,io,json,os,pathlib,re,subprocess,tarfile
sha=lambda b:hashlib.sha256(b).hexdigest()
def need(x,m):
 if not x:raise SystemExit('FAIL: '+m)
def obj(b):return json.loads(b)
a=argparse.ArgumentParser();a.add_argument('--key');a.add_argument('--git');a=a.parse_args();here=pathlib.Path(__file__).resolve().parent;m=obj((here/'MANIFEST.json').read_bytes())
for row in m['files']:
 n=row['path'];need('/' not in n and '\\' not in n and n not in ('.','..'),'unsafe name');b=(here/n).read_bytes();need(len(b)==row['bytes'] and sha(b)==row['sha256'],'public '+n)
print('Public integrity:',len(m['files']),'files');need(not a.git or a.key,'--git requires --key')
if not a.key:print('Private originals and Git binding not checked; separate same-machine key required.');raise SystemExit(0)
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
key=pathlib.Path(a.key).read_bytes();need(len(key)==32,'key length');c=(here/'PRIVATE-EVIDENCE.aesgcm').read_bytes();magic=b'726Q2910v1\0';need(c.startswith(magic),'cipher header');nonce=c[len(magic):len(magic)+12];data=AESGCM(key).decrypt(nonce,c[len(magic)+12:],b'726-q2910-lens1-evidence-v1');tar=tarfile.open(fileobj=io.BytesIO(data),mode='r:gz');members=tar.getmembers();need(all(x.isfile() and not x.name.startswith('/') and '..' not in pathlib.PurePosixPath(x.name).parts for x in members),'private paths');need(len({x.name for x in members})==len(members),'duplicate paths');fs={x.name:tar.extractfile(x).read() for x in members};pm=obj(fs['PRIVATE-MANIFEST.json']);idx={x['path']:x for x in pm['files']};need(set(fs)-{'PRIVATE-MANIFEST.json'}==set(idx),'private member set')
for n,r in idx.items():need(len(fs[n])==r['bytes'] and sha(fs[n])==r['sha256'],'private integrity '+n)
p=lambda n:fs['review/'+n];q1=obj(fs['git/original-queue.json']);q2=obj(fs['git/reconstructed-queue.json']);need(q1['entries'][2910]==q2['entries'][2910] and q1['entries'][2910]['contract_id']=='OBLIGATION-ROOT-L3-EXT-0018-LENS-01','q2910 entry')
def spans(raw,wanted):
 s=raw.decode();m=re.search(r'"entries"\s*:\s*\[',s);need(m is not None,'queue locator');pos=m.end();dec=json.JSONDecoder();out={}
 for i in range(max(wanted)+1):
  while s[pos].isspace() or s[pos]==',':pos+=1
  start=pos;_,pos=dec.raw_decode(s,pos)
  if i in wanted:out[i]=s[start:pos].encode()
 return out
wanted={2355,2701,2910,2934};x=spans(fs['git/original-queue.json'],wanted);y=spans(fs['git/reconstructed-queue.json'],{2910});need(x[2910]==y[2910] and x[2910]==p('QUEUE-original.json')==p('QUEUE-reconstructed.json'),'exact q2910 original/reconstruction bytes')
for i in (2355,2701,2934):need(x[i]==p('HIST-Q'+str(i)+'.json'),'historical selection '+str(i))
old=obj(p('HISTORY-1.dat'));adopt=obj(p('HISTORY-3.dat'));need(old['lens']==3 and old['rows'][0]['fresh_personal_ranges']==[[1,110]],'historical read attribution');need(adopt['lens']==3 and adopt['rows'][5]['fresh_personal_ranges']==[] and adopt['rows'][5]['qualified_reuse_ranges']==[[1,110]] and adopt['rows'][5]['source_adoption_pending_for_frozen18_19'] is True,'e275 qualification')
assigned=obj(p('ASSIGNED-IDENTITIES.json'));bodies=[p('ASSIGNED-'+str(i)+'.dat') for i in range(3)];need(bodies[0]==bodies[1]==bodies[2]==p('HIST-ORIGINAL-SOURCE.dat') and sha(bodies[0])=='d39e2fce7f4e99648f2c1b5409b52a5a34a55d97514699a77d803e1e2273ffd1','assigned identities')
for i,body in enumerate(bodies):
 lines=body.decode().splitlines();expected=''.join(f'{j}: {lines[j-1]}\n' for j in range(1,111)).encode();need(p('DISPLAY-ASSIGNED-'+str(i)+'.txt')==expected,'assigned displayed text')
cap=obj(p('SUPPORT-CAPTURE.json'));srows={r['label']:r for r in cap['rows']}
for r in cap['rows']:need(sha(p('SUPPORT-'+r['label']+'.dat'))==r['sha256'],'support source '+r['label'])
groups=[('DISPLAY-SUPPORT-A.txt',['lib','runtime']),('DISPLAY-SUPPORT-B.txt',['setup','effects','terrain','presentation']),('DISPLAY-SUPPORT-C.txt',['bake','catalogue','wire']),('DISPLAY-SUPPORT-D.txt',['texture-cargo','tactical-cargo'])]
for name,labels in groups:
 out=[]
 for lab in labels:
  r=srows[lab];lines=p('SUPPORT-'+lab+'.dat').decode().splitlines();out.append('\n### '+lab+' '+r['path']+'\n')
  for lo,hi in r['applied_ranges']:out.extend(f'{i}: {lines[i-1]}' for i in range(lo,hi+1))
 need(p(name)==('\n'.join(out)+'\n').encode(),'support display '+name)
lines=p('SUPPORT-blood.dat').decode().splitlines()
for name,lo,hi in [('DISPLAY-BLOOD-A.txt',1,160),('DISPLAY-BLOOD-B.txt',200,315)]:need(p(name)==''.join(f'{i}: {lines[i-1]}\n' for i in range(lo,hi+1)).encode(),'blood display')
led=obj(fs['git/dac7d-baseline.json']);selected=obj(p('LEDGER-SELECTED.json'))
for row in selected:need(led['newly_assessed_ledger']['entries'][row['index']]==row['record'],'ledger row '+str(row['index']))
h1=obj(fs['git/original-ledger.json']);h2=obj(fs['git/reconstructed-ledger.json']);k='ADLER-L11-BLOOD-SURFACE-DISTANCE-FRAME';need(h1['discovery_findings_and_dependencies'][k]==h2['discovery_findings_and_dependencies'][k],'historical blood record equality')
archive=tarfile.open(fileobj=io.BytesIO(fs['git/q2734-transport.tar.gz']),mode='r:gz');need(archive.extractfile('evidence/ASSESSMENT.md').read()==p('Q2734-ASSESSMENT.md'),'published separate delegate assessment')
print('Authenticated queue selections, source identities/displays, historical attribution, ledger selections and separate delegate assessment: verified')
if a.git:
 env=dict(os.environ,GIT_NO_REPLACE_OBJECTS='1',GIT_NO_LAZY_FETCH='1',GIT_OPTIONAL_LOCKS='0')
 def git(rev,path):return subprocess.check_output(['git','-C',a.git,'show',rev+':'+path],env=env,stderr=subprocess.DEVNULL)
 for n,r in idx.items():
  if r.get('git_revision'):need(git(r['git_revision'],r['git_path'])==fs[n],'Git binding '+n)
 for i,r in enumerate(assigned['rows']):need(git(r['revision'],assigned['source'])==bodies[i],'assigned Git '+str(i))
 for r in cap['rows']:need(git(cap['revision'],r['path'])==p('SUPPORT-'+r['label']+'.dat'),'support Git '+r['label'])
 need(git(cap['revision'],'crates/adventuresim-tactical-client/src/combat_effects/blood_decals.rs')==p('SUPPORT-blood.dat'),'blood Git');need(git(cap['revision'],'Cargo.toml')==p('SUPPORT-workspace-cargo.dat'),'workspace Git');print('External immutable Git content bindings: verified')
else:print('External immutable Git content bindings not checked (supply --git).')
print('No product execution, feature resolution, implementation, product validation, global alias or checkpoint3821 claim.')
