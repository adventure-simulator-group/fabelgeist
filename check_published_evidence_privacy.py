"""Run current evidence checks with explicitly qualified privacy successors.

The earlier driver and its original-byte verifiers remain unchanged as
historical evidence. This driver uses separately preserved successor scripts
for the redacted current tree; a public crosswalk is not an original-byte match.
No product build, test, shader, macro, fixture or runtime probe is invoked.
"""
import json,os,subprocess,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
env=os.environ.copy();env.pop('PYTHONOPTIMIZE',None);env['PYTHONDONTWRITEBYTECODE']='1';env['PYTHONNOUSERSITE']='1'
root=Path(__file__).resolve().parent
stage=root/'delegated/qualified-059-successors'
plan=json.loads((stage/'DRIVER-INTEGRATION.json').read_text())
replacements={x['original']:stage/x['successor'] for x in plan['exact_replacements']}
assert len(replacements)==15 and set(replacements)=={'verify_reconciliation_proposal_links_15.py'}|{f'verify_delegated_links_{i}.py' for i in range(16,30)}
early=root/'delegated/qualified-059-early-links'
early_plan=json.loads((early/'INVENTORY.json').read_text())
assert {x['original'] for x in early_plan['rows']}=={f'verify_delegated_links_{i}.py' for i in range(11,16)}
replacements.update({x['original']:early/x['successor'] for x in early_plan['rows']})
assert len(replacements)==20
city='verify_city_streaming_readiness_95e.py'
first=['verify_recovered.py','verify_selected_states.py','verify_lock_inventory.py','verify_delta_sources.py','verify_role_worklist.py']
optional={'verify_local_checkpoint_059.py','verify_recovered_public_redacted_plus2.py','verify_post8ca_checkpoint.py'}
remaining=sorted(p.name for p in root.glob('verify_*.py') if p.name not in first and p.name not in optional)
successor=root/'verify_post8b_checkpoint.py'
assert successor.is_file() and not successor.is_symlink() and remaining.count(successor.name)==1
native=sorted(p.relative_to(root).as_posix() for p in (root/'native-evidence').glob('*/verify.py'))
print('CHECK verify_post8ca_checkpoint.py -> skipped at successor root; use verify_post8b_checkpoint.py with separate --predecessor and --ancestor for the exact older guards',flush=True)
for name in first+remaining+native:
 args=[];script=root/name
 if name=='verify_recovered.py':
  script=root/'verify_recovered_public_redacted_plus2.py'
  print('CHECK',name,'-> qualified five-object public successor; original-byte verifier retained separately',flush=True)
 elif name in replacements:
  script=replacements[name];args=['--publication',str(root)]
  print('CHECK',name,'-> qualified 059 original-to-public identity successor; not a direct byte match',flush=True)
 elif name==city:
  script=root/'delegated/city-streaming-privacy-proposal/candidate/verify_city_streaming_readiness_redacted_95e.py';args=['--publication',str(root)]
  print('CHECK',name,'-> qualified city registration reconstruction successor',flush=True)
 else:print('CHECK',name,flush=True)
 subprocess.run([sys.executable,'-B',str(script),*args],cwd=root,env=env,check=True)
print('Published evidence checks passed for the qualified public reconstructions; retained private originals and previous Git history are separate. No product build, test or probe ran.')
