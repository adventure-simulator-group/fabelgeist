#!/usr/bin/env python3
"""Verify the captured audit files and additive source-only evidence."""
import argparse,hashlib,json,stat,sys
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,want in m['files'].items():assert h((p/n).read_bytes())==want
old=json.loads((p/'CAPTURE-134.json').read_bytes());new=json.loads((p/'CAPTURE-164.json').read_bytes());a={x['path']:x for x in old['rows']};b={x['path']:x for x in new['rows']};assert len(a)==134 and len(b)==164 and set(a)<=set(b) and all(a[k]==b[k] for k in a) and old['strict_diff_check_exit']==new['strict_diff_check_exit']==0
scan=json.loads((p/'DIGEST-SCAN-164.json').read_bytes());assert scan['stats']['top_files']==164 and scan['forbidden_digest_total']==0 and not scan['other_candidate_counts'] and not scan['exceptions'];assert 'issues 0' in (p/'TAR-PATTERN-SCAN-164.txt').read_text()
arg=argparse.ArgumentParser();arg.add_argument('--publication',type=Path,required=True);root=arg.parse_args().publication.resolve()
for name,row in b.items():
 path=PurePosixPath(name);assert not path.is_absolute() and '..' not in path.parts;f=root.joinpath(*path.parts);assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(root);body=f.read_bytes();assert len(body)==row['bytes'] and h(body)==row['sha256'];assert bool(f.stat().st_mode&0o111)==(row['mode']=='100755');assert hashlib.sha1(b'blob '+str(len(body)).encode()+b'\0'+body).hexdigest()==row['git_blob']
r=json.loads((root/'delegated/postede12-followups/REPLAY-134.json').read_bytes());assert r['stage_capture_sha256']==h((p/'CAPTURE-134.json').read_bytes()) and r['evidence_driver_exit']==0 and r['stage_capture_count']==134
print('Verified 164 captured byte/mode/Git-blob identities, the unchanged 134-file replay snapshot and bounded 164-file scans; later proof files and restricted originals remain separately qualified.')
