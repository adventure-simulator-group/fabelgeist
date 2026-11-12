"""Run current evidence checks with explicitly qualified privacy successors.

The earlier driver and its original-byte verifiers remain unchanged as
historical evidence. This driver uses separately preserved successor scripts
for the redacted current tree; a public crosswalk is not an original-byte match.
No product build, test, shader, macro, fixture or runtime probe is invoked.
"""
import json,subprocess,sys
from pathlib import Path
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
optional={'verify_local_checkpoint_059.py','verify_recovered_public_redacted_plus2.py'}
remaining=sorted(p.name for p in root.glob('verify_*.py') if p.name not in first and p.name not in optional)
native=sorted(p.relative_to(root).as_posix() for p in (root/'native-evidence').glob('*/verify.py'))
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
 subprocess.run([sys.executable,str(script),*args],cwd=root,check=True)
print('Published evidence checks passed for the qualified public reconstructions; retained private originals and previous Git history are separate. No product build, test or probe ran.')
