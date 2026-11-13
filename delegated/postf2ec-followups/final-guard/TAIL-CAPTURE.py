#!/usr/bin/env python3
from pathlib import Path,PurePosixPath
import subprocess,json,hashlib,os
p=Path(__file__).resolve().parent;base=p.parent;repo=base.parent/'publication';tree=p/'tree';tree.mkdir(mode=0o700);git=lambda *a:subprocess.check_output(['git',*a],cwd=repo);h=lambda b:hashlib.sha256(b).hexdigest()
old=json.loads((base/'postf2ec-final-staged-capture-v2/CAPTURE.json').read_bytes());late=json.loads((base/'postf2ec-late-capture-v1/CAPTURE.json').read_bytes());oldrows={x['path']:x for x in old['rows']+late['new_rows']};assert len(oldrows)==265
check=subprocess.run(['git','diff','--cached','--check'],cwd=repo,capture_output=True);(p/'strict.stdout').write_bytes(check.stdout);(p/'strict.stderr').write_bytes(check.stderr);assert check.returncode==0 and not git('diff','--name-only','-z') and not git('ls-files','--others','--exclude-standard','-z')
paths=[x.decode() for x in git('diff','--cached','--name-only','-z').split(b'\0') if x];assert set(oldrows)<=set(paths);rows=[]
for name in paths:
 q=PurePosixPath(name);assert not q.is_absolute() and '..' not in q.parts;src=repo.joinpath(*q.parts);assert src.is_file() and not src.is_symlink();s=git('ls-files','--stage','--',name).decode().strip().split('\t',1);mode,oid,num=s[0].split();assert s[1]==name and num=='0' and mode in ('100644','100755');b=git('cat-file','blob',oid);row={'path':name,'bytes':len(b),'sha256':h(b),'git_blob':oid,'mode':mode};assert b==src.read_bytes() and hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==oid and bool(src.stat().st_mode&0o111)==(mode=='100755')
 if name in oldrows:assert row==oldrows[name];continue
 dst=tree.joinpath(*q.parts);dst.parent.mkdir(parents=True,exist_ok=True)
 with dst.open('xb') as f:f.write(b)
 dst.chmod(0o755 if mode=='100755' else 0o644);rows.append(row)
r={'schema':'726-postf2ec-tail-staged-capture-v1','prior_capture_sha256':h((base/'postf2ec-late-capture-v1/CAPTURE.json').read_bytes()),'unchanged_prior_paths':len(oldrows),'new_rows':rows,'strict_diff_check_exit':check.returncode,'qualification':'Separate additive guard snapshot after 265 earlier captured paths; any later receipt files remain outside this snapshot. Evidence-only; no product validation.'}
with (p/'CAPTURE.json').open('x') as f:json.dump(r,f,indent=2);f.write('\n')
print(json.dumps({'prior_paths':len(oldrows),'new_paths':len(rows),'capture_sha256':h((p/'CAPTURE.json').read_bytes())}))
