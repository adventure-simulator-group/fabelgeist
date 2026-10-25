"""Read only source hashes/Git state; never a project validation or audit verdict."""
from pathlib import Path
import json,hashlib,subprocess,datetime,collections
A=Path(__file__).resolve().parent;R=A.parent;P=R.parents[2];m=json.loads((A/'source-manifest.json').read_text());previous=json.loads((A/'source-preservation-observation.json').read_text());mismatches=[];snapshots=[]
def git(root,*args):return subprocess.check_output(['git','-C',str(root),*args],text=True)
for s in m['snapshots']:
 root=Path(s['root']);head=git(root,'rev-parse','HEAD').strip();status=git(root,'status','--porcelain=v1','--untracked-files=all').splitlines();rows=[r for r in m['files'] if r['snapshot']==s['id']]
 for r in rows:
  p=root/r['path'];observed=hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else None
  if observed!=r['sha256']:mismatches.append({'source_id':r['source_id'],'expected':r['sha256'],'observed':observed})
 if head!=s['head']:mismatches.append({'snapshot':s['id'],'head':head,'expected_head':s['head']})
 if status!=s['status_porcelain']:mismatches.append({'snapshot':s['id'],'status':status,'expected_status':s['status_porcelain']})
 snapshots.append({'id':s['id'],'expected_head':s['head'],'observed_head':head,'head_unchanged':head==s['head'],'status_unchanged':status==s['status_porcelain'],'paths_fingerprinted':len(rows),'observed_dirty_paths':len(status)})
# Workspace path is taken from the pinned integration snapshot, not a shell variable.
P=Path(m['snapshots'][0]['root']).parents[3]
# The integration path is P/target/semantic-goal/rig-review-774/worktree.
if P.name!='adventure-simulator':raise SystemExit('Unexpected primary workspace path')
stash=m['original_stash']['commit'];listed=stash in git(P,'stash','list','--format=%H').splitlines();manifests=[]
for r in previous['unrelated_primary_manifest_edits']:
 observed=hashlib.sha256((P/r['path']).read_bytes()).hexdigest();same=observed==r['sha256'];manifests.append({'path':r['path'],'sha256':observed,'unchanged_from_freeze':same})
 if not same:mismatches.append({'primary_manifest':r['path'],'expected':r['sha256'],'observed':observed})
if not listed:mismatches.append({'original_stash_not_listed':stash})
out={'observed_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'kind':'Source/Git preservation metadata only; no project check, lens clearance or acceptance verdict','snapshots':snapshots,'mismatches':mismatches,'original_stash_commit':stash,'original_stash_listed':listed,'unrelated_primary_manifest_edits':manifests,'metadata_correction':previous.get('metadata_correction')};(A/'source-preservation-observation.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'paths_fingerprinted':sum(s['paths_fingerprinted'] for s in snapshots),'mismatches':len(mismatches),'stash_preserved':listed,'primary_manifest_edits_preserved':all(r['unchanged_from_freeze'] for r in manifests),'project_commands_executed':False}))
