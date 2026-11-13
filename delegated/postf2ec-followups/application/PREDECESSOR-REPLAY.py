#!/usr/bin/env python3
"""Replay predecessor audit verifiers against their archived after-images."""
import os,shutil,subprocess,sys,tarfile,tempfile
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('Assertions required')
assert len(sys.argv)==3 and sys.argv[1] in ('d7','ede')
kind=sys.argv[1];root=Path(sys.argv[2]).resolve();app=root/'delegated/postf2ec-followups/application'
subprocess.run([sys.executable,'-B',str(app/'VERIFY.py'),'--published',str(root)],check=True)
shadow=Path(tempfile.mkdtemp(prefix='audit-f2ec-predecessor-evidence-'))
with tarfile.open(app/'APPLICATION-EVIDENCE.tar.gz','r:gz') as t:
 for n in ['BASELINE-95E-20261110.json','CONTINUATION-PROGRESS-95E.json','RECONCILIATION-059.json','CONTINUATION.md']:
  (shadow/n).write_bytes(t.extractfile('before/'+n).read())
for n in ['ASSESSMENT-CENTRAL-FOLLOWUPS-17-95E.json','MAIN-RECHECK-20261113-0033.json','ASSESSMENT-CENTRAL-FOLLOWUPS-18-95E.json','MAIN-RECHECK-20261113-0140.json']:
 shutil.copy2(root/n,shadow/n)
name='verify_postd7a5_application.py' if kind=='d7' else 'verify_postede12_application.py'
shutil.copy2(app/('PREDECESSOR-D7.py' if kind=='d7' else 'PREDECESSOR-EDE.py'),shadow/name)
(shadow/'delegated').symlink_to(root/'delegated',target_is_directory=True)
env=dict(os.environ);env['PYTHONDONTWRITEBYTECODE']='1';env['PYTHONNOUSERSITE']='1';env.pop('PYTHONOPTIMIZE',None)
subprocess.run([sys.executable,'-B',str(shadow/name)],check=True,env=env)
print('Replayed predecessor',kind,'against the exact archived f2ec after-image; the current successor is checked separately.')
