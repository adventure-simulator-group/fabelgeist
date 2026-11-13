#!/usr/bin/env python3
from pathlib import Path
import subprocess,sys,json,hashlib
p=Path(__file__).resolve().parent;base=p.parent;repo=base.parent/'publication';keys=list(base.glob('checkpoint-14d-preservation-key-*/KEY.bin'));assert len(keys)==1 and len(keys[0].read_bytes())==32
r=subprocess.run([sys.executable,'-B',str(p/'ENCRYPT.py'),str(keys[0])],cwd=repo,capture_output=True);(p/'stdout').write_bytes(r.stdout);(p/'stderr').write_bytes(r.stderr)
with (p/'RESULT.json').open('x') as f:json.dump({'exit':r.returncode,'stdout_sha256':hashlib.sha256(r.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(r.stderr).hexdigest(),'qualification':'In-memory archive comparisons and authenticated roundtrips only; separately held primary key and historical anchors remain required. No independent custody or product validation.'},f,indent=2);f.write('\n')
print(json.dumps({'exit':r.returncode}),flush=True);assert r.returncode==0
