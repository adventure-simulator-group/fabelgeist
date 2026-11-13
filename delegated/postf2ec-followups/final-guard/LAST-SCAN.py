#!/usr/bin/env python3
from pathlib import Path
import subprocess,sys,json,hashlib,re
p=Path(__file__).resolve().parent;base=p.parent;target=base/'postf2ec-last-capture-v1/tree';corpus=base/'next-15a-digest-corpus-358yvabw/corpus';pattern=base/'checkpoint-after13v-prepared-success-paAYncnb/tar-aware-pattern-scanner.py';rows=[]
for name,cmd in [('private-digests',[sys.executable,'-B',str(corpus/'scan-private-digests.py'),str(target),'--private-capture',str(corpus)]),('tar-patterns',[sys.executable,'-B',str(pattern),str(target)])]:
 z=subprocess.run(cmd,capture_output=True);(p/(name+'.out')).write_bytes(z.stdout);(p/(name+'.err')).write_bytes(z.stderr);rows.append({'name':name,'exit':z.returncode,'stdout_sha256':hashlib.sha256(z.stdout).hexdigest(),'stderr_bytes':len(z.stderr)})
d=json.loads((p/'private-digests.out').read_bytes());text=(p/'tar-patterns.out').read_text();m=re.search(r'issues\s+(\d+)',text)
r={'schema':'726-postf2ec-last-staged-scan-v1','checks':rows,'scanned_paths':d['stats']['top_files'],'forbidden_digest_total':d['forbidden_digest_total'],'other_candidate_counts':d['other_candidate_counts'],'exceptions':len(d['exceptions']),'tar_pattern_issues':int(m.group(1)) if m else None,'qualification':'Bounded scans over the separate 12-path final receipt snapshot; its own receipt files remain outside this scan. Ciphertext is opaque; unknown private values, encodings and unscanned Git history remain outside scope.'}
r['capture_sha256']=hashlib.sha256((base/'postf2ec-last-capture-v1/CAPTURE.json').read_bytes()).hexdigest()
with (p/'RESULT.json').open('x') as f:json.dump(r,f,indent=2);f.write('\n')
print(json.dumps({k:r[k] for k in ['scanned_paths','forbidden_digest_total','other_candidate_counts','exceptions','tar_pattern_issues']}),flush=True)
raise SystemExit(any(x['exit'] for x in rows) or r['forbidden_digest_total']!=0 or bool(r['other_candidate_counts']) or r['exceptions']!=0 or r['tar_pattern_issues']!=0)
