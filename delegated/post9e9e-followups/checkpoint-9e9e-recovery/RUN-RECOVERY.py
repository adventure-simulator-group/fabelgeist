from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys
scratch=Path(sys.argv[1]).resolve();expected='9e9e237c0c3c28278342e831da9f243fc2229e58';parent='b147e0caaf982e0adc765f188353152db994b184';pre=Path('../private-originals/checkpoint-postb147-final-evidence-p7pg3pab/evidence.log').resolve()
env=os.environ.copy()
for n in ['PYTHONOPTIMIZE','PYTHONPATH','PYTHONHOME','PYTHONSTARTUP','PYTHONINSPECT','PYTHONUSERBASE']:env.pop(n,None)
tmp=scratch/'tmp';tmp.mkdir();env.update({'PYTHONNOUSERSITE':'1','PYTHONDONTWRITEBYTECODE':'1','GIT_NO_REPLACE_OBJECTS':'1','GIT_NO_LAZY_FETCH':'1','TMPDIR':str(tmp)})
def git(*args):return subprocess.check_output(['git',*args],text=True,env=env).strip()
def state():
 return {'head':git('rev-parse','HEAD'),'clean':not git('status','--porcelain'),'symbolic_ref_exit':subprocess.run(['git','symbolic-ref','-q','HEAD'],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,env=env).returncode}
assert state()=={'head':parent,'clean':True,'symbolic_ref_exit':1};fetch=git('rev-parse','FETCH_HEAD');assert fetch==expected
with (scratch/'CHECKOUT.log').open('xb') as f:r=subprocess.run(['git','-c','core.hooksPath=/dev/null','-c','gc.auto=0','-c','maintenance.auto=false','checkout','--detach',expected],stdout=f,stderr=subprocess.STDOUT,env=env)
assert r.returncode==0;checked=state();assert checked=={'head':expected,'clean':True,'symbolic_ref_exit':1}
with (scratch/'CHECKED-OUT.json').open('x') as f:json.dump(checked,f,indent=2);f.write('\n')
a=subprocess.run([sys.executable,'-B','-c','assert False'],env=env,capture_output=True);b=subprocess.run([sys.executable,'-B','-c','import sys; print(sys.flags.optimize)'],env=env,capture_output=True,text=True);assert a.returncode!=0 and b.returncode==0 and b.stdout.strip()=='0' and b'AssertionError' in a.stderr
preflight={'assertions_enabled':True,'optimization_level':0}
with (scratch/'ASSERT-PREFLIGHT.log').open('x') as f:json.dump(preflight,f,sort_keys=True);f.write('\n')
started=datetime.datetime.now(datetime.timezone.utc).isoformat();print('Clean detached recovery checkout; evidence-only driver started.',flush=True)
with (scratch/'evidence.log').open('xb') as f:r=subprocess.run([sys.executable,'-B','-u','check_published_evidence_privacy.py'],stdout=f,stderr=subprocess.STDOUT,env=env)
after=state()
with (scratch/'AFTER.json').open('x') as f:json.dump(after,f,indent=2);f.write('\n')
log=(scratch/'evidence.log').read_bytes();old=pre.read_bytes();record={'checkpoint':expected,'previous_checkpoint':parent,'fetch_head_asserted_before_checkout':fetch,'recovery_driver_exit':r.returncode,'started_utc':started,'finished_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'clean_before_and_after':after['clean'],'symbolic_ref_exit_after':after['symbolic_ref_exit'],'post_replay_head':after['head'],'post_replay_clean':after['clean'],'prepublication_and_recovery_transcripts_byte_identical':log==old,'transcript_bytes':len(log),'transcript_lines':len(log.splitlines()),'transcript_sha256':hashlib.sha256(log).hexdigest(),'assertion_preflight':preflight,'scratch_residue':[x.name for x in tmp.iterdir()],'qualification':'Both checkouts and retained evidence are on the same machine. Transcript does not embed checkout identity; HEAD, cleanliness and detached state were separately observed. Direct child assertion preflight does not prove every nested child flag. This does not prove independent custody, product behavior, audit completion, complete privacy or missing checkpoint3821 records.'}
with (scratch/'RECOVERY-RECORD.json').open('x') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps(record),flush=True)
assert r.returncode==0 and after=={'head':expected,'clean':True,'symbolic_ref_exit':1} and log==old
