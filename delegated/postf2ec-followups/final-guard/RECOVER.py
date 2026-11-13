#!/usr/bin/env python3
from pathlib import Path,PurePosixPath
import json,subprocess,sys,hashlib,stat,tarfile
p=Path(__file__).resolve().parent;base=p.parent;repo=base.parent/'publication';enc=repo/'delegated/postf2ec-followups/guard-encrypted';restore=p/'restored';keys=list(base.glob('checkpoint-14d-preservation-key-*/KEY.bin'));assert len(keys)==1
r=subprocess.run([sys.executable,'-B',str(enc/'VERIFY.py'),'--key',str(keys[0]),'--restore-root',str(restore)],capture_output=True);(p/'restore.stdout').write_bytes(r.stdout);(p/'restore.stderr').write_bytes(r.stderr);assert r.returncode==0
sources=json.loads((base/'postf2ec-guard-encryption-control-v1/PRIVATE-SOURCE-MAP.json').read_bytes())['sources'];files=dirs=0
for label,s in sources.items():
 src=Path(s);dst=restore/label/'evidence';a={x.relative_to(src).as_posix():x for x in src.rglob('*')};b={x.relative_to(dst).as_posix():x for x in dst.rglob('*')};assert set(a)==set(b)
 for n,x in a.items():
  y=b[n];assert x.is_dir()==y.is_dir() and stat.S_IMODE(x.stat().st_mode)==stat.S_IMODE(y.stat().st_mode)
  if x.is_file():assert x.read_bytes()==y.read_bytes();files+=1
  else:dirs+=1
public=p/'q0-public';public.mkdir(mode=0o700)
with tarfile.open(repo/'delegated/postf2ec-followups/late-transport/q0-behavior-v2.tar.gz','r:gz') as a:
 for x in a:
  q=PurePosixPath(x.name);assert q.parts[0]=='q0-behavior-v2' and '..' not in q.parts and x.isfile();dest=public.joinpath(*q.parts[1:]);dest.parent.mkdir(parents=True,exist_ok=True)
  with dest.open('xb') as f:f.write(a.extractfile(x).read())
  dest.chmod(x.mode)
z=subprocess.run([sys.executable,'-B',str(public/'VERIFY.py'),'--private',str(restore/'q0-original/evidence')],capture_output=True);(p/'q0.stdout').write_bytes(z.stdout);(p/'q0.stderr').write_bytes(z.stderr);assert z.returncode==0
raw=(restore/'final-guard-process/evidence/main-observation/CONNECTOR-RESPONSE.json').read_bytes();main=json.loads((repo/'delegated/postf2ec-followups/final-guard/MAIN.json').read_bytes());assert hashlib.sha256(raw).hexdigest()==main['raw_serialized_response_sha256']
result={'schema':'726-postf2ec-guard-restoration-v1','restore_exit':r.returncode,'collections':len(sources),'matched_file_bytes_and_modes':files,'matched_directory_modes':dirs,'q0_evidence_replay_exit':z.returncode,'main_raw_sha256_matched':True,'qualification':'Fresh authenticated same-machine restoration; normalized uid/gid/mtime and omitted source-root modes. q0 replay uses restored source evidence. Not independent custody, recovery of checkpoint3821, product validation or semantic closure.'}
with (p/'RESULT.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
print(json.dumps({k:result[k] for k in ['restore_exit','collections','matched_file_bytes_and_modes','matched_directory_modes','q0_evidence_replay_exit','main_raw_sha256_matched']}),flush=True)
