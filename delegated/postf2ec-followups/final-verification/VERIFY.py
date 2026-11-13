#!/usr/bin/env python3
"""Verify the exact staged snapshot used for evidence-only checks."""
import argparse,hashlib,json,sys
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-postf2ec-final-verification-v1' and {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,want in m['files'].items():assert h((p/n).read_bytes())==want
capture=json.loads((p/'CAPTURE-173.json').read_bytes());scan=json.loads((p/'SCAN-173.json').read_bytes());replay=json.loads((p/'REPLAY-173.json').read_bytes());assert len(capture['rows'])==173 and capture['strict_diff_check_exit']==0 and scan['scanned_paths']==173 and scan['forbidden_digest_total']==0 and not scan['other_candidate_counts'] and scan['exceptions']==scan['tar_pattern_issues']==0
assert replay['evidence_driver_exit']==0 and replay['capture_sha256']==h((p/'CAPTURE-173.json').read_bytes()) and replay['direct_child_assertion_preflight']=={'assertions_enabled':True,'optimization_level':0}
a=argparse.ArgumentParser();a.add_argument('--publication',type=Path,required=True);root=a.parse_args().publication.resolve()
for row in capture['rows']:
 name=PurePosixPath(row['path']);assert not name.is_absolute() and '..' not in name.parts;f=root.joinpath(*name.parts);assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(root);b=f.read_bytes();assert (len(b),h(b),hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest())==(row['bytes'],row['sha256'],row['git_blob']);assert bool(f.stat().st_mode&0o111)==(row['mode']=='100755')
print('Verified 173 captured byte/mode/Git-blob identities, the evidence replay snapshot and bounded scans; later additive files, restricted originals and semantic closure remain separate.')
