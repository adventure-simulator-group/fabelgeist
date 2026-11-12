import argparse,datetime,hashlib,json,pathlib,subprocess
p=pathlib.Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--repo',required=True);repo=a.parse_args().repo;rev='95e63628f9d70b8dd6e6fede64334696c6995009';prefix='crates/adventuresim-character-creator/src/'
spec=[('lib.rs',[[145,235]]),('main.rs',[[200,230],[360,400],[500,535]]),('inventory_acquire.rs',[[100,138]])];items=[]
for i,(file,ranges) in enumerate(spec):
 path=prefix+file; data=subprocess.check_output(['git','-C',repo,'show',rev+':'+path]);entry=subprocess.check_output(['git','-C',repo,'ls-tree',rev,'--',path]).decode().rstrip();lines=len(data.splitlines())
 if any(b>lines for a,b in ranges):raise ValueError('planned window exceeds EOF')
 dest=f'sources/support-{i}.rs'; target=p/dest
 if target.exists():
  if target.read_bytes()!=data:raise ValueError('existing capture differs')
 else:
  with target.open('xb') as f:f.write(data)
 items.append({'revision':rev,'path':path,'git_entry':entry,'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data),'lines':lines,'file':dest,'display_ranges':ranges})
x={'registered_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'reader':'/root/mhr_character','items':items,'qualification':'Bounded producer/admission/caller support only; source bytes retained do not imply full personal reads and q2329 error handling remains separately attributed.'};(p/'SUPPORT-REGISTRATION.json').write_text(json.dumps(x,indent=2)+'\n');print(json.dumps([{'path':x['path'],'lines':x['lines'],'sha256':x['sha256']} for x in items]))
