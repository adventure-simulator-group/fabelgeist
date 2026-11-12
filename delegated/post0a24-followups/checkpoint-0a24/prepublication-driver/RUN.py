from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys,tempfile
base=Path('../private-originals'); capture=base/'checkpoint-post31e1-staged-copy-sn90er6_/CAPTURE.json'; saved=json.loads(capture.read_bytes());
assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()=='31e130bfa8f8914f759c7d5cddc3db92962704c7'
changed=[x.decode() for x in subprocess.check_output(['git','diff','--cached','--name-only','--diff-filter=ACMRT','-z']).split(b'\0') if x];assert set(changed)=={x['path'] for x in saved['rows']}
idx={}
for row in subprocess.check_output(['git','ls-files','--stage','-z']).split(b'\0'):
 if row:
  meta,name=row.split(b'\t',1);mode,oid,stage=meta.decode().split();idx[name.decode()]=(mode,oid,stage)
for row in saved['rows']:
 p=Path(row['path']);assert idx[row['path']]==(row['mode'],row['git_blob'],'0') and p.is_file() and not p.is_symlink();assert hashlib.sha256(p.read_bytes()).hexdigest()==row['sha256'];assert bool(p.stat().st_mode&0o111)==(row['mode']=='100755')
scratch=Path(tempfile.mkdtemp(prefix='checkpoint-post31e1-final-evidence-',dir=base)).resolve();tmp=scratch/'tmp';tmp.mkdir();env=os.environ.copy()
for n in ['PYTHONOPTIMIZE','PYTHONPATH','PYTHONHOME','PYTHONSTARTUP','PYTHONINSPECT','PYTHONUSERBASE']:env.pop(n,None)
env.update({'PYTHONNOUSERSITE':'1','PYTHONDONTWRITEBYTECODE':'1','GIT_NO_REPLACE_OBJECTS':'1','GIT_NO_LAZY_FETCH':'1','TMPDIR':str(tmp)})
a=subprocess.run([sys.executable,'-B','-c','assert False'],env=env,capture_output=True);b=subprocess.run([sys.executable,'-B','-c','import sys; print(sys.flags.optimize)'],env=env,capture_output=True,text=True);assert a.returncode!=0 and b.returncode==0 and b.stdout.strip()=='0' and b'AssertionError' in a.stderr
pre={'assertions_enabled':True,'optimization_level':0}
with (scratch/'ASSERT-PREFLIGHT.log').open('x') as f:json.dump(pre,f,sort_keys=True);f.write('\n')
started=datetime.datetime.now(datetime.timezone.utc).isoformat()
with (scratch/'STARTED.json').open('x') as f:json.dump({'started_utc':started,'captured_staged_aggregate':saved['aggregate']},f,indent=2);f.write('\n')
print('Frozen evidence checkout; verification directory:',scratch,flush=True)
with (scratch/'evidence.log').open('xb') as f:r=subprocess.run([sys.executable,'-B','-u','check_published_evidence_privacy.py'],stdout=f,stderr=subprocess.STDOUT,env=env)
raw=(scratch/'evidence.log').read_bytes();result={'exit':r.returncode,'started_utc':started,'finished_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'transcript_bytes':len(raw),'transcript_lines':len(raw.splitlines()),'transcript_sha256':hashlib.sha256(raw).hexdigest(),'assertion_preflight':pre,'qualification':'Evidence-only driver in frozen staged checkout with selected environment normalization. Direct child assertion preflight does not establish all nested child flags, dynamic inputs, independent custody, complete privacy or product behavior. No product builds, tests or probes.'}
with (scratch/'RESULT.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps(result),flush=True)
raise SystemExit(r.returncode)
