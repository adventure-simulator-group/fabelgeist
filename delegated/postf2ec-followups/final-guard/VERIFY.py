#!/usr/bin/env python3
"""Verify the captured public evidence snapshot and prepublication observation."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);root=ap.parse_args().publication.resolve();h=lambda b:hashlib.sha256(b).hexdigest();c=json.loads((p/'CAPTURE.json').read_bytes());s=json.loads((p/'SCAN.json').read_bytes());m=json.loads((p/'MAIN.json').read_bytes());assert c['schema']=='726-postf2ec-final-staged-capture-v1' and c['parent_checkpoint']=='f2ec8cbad7becbcc9a8e9da22ba512f52975768f' and c['source_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and c['strict_diff_check_exit']==0 and c['unstaged_and_untracked_nonignored_files']==0
assert s['scanned_paths']==len(c['rows']) and s['capture_sha256']==h((p/'CAPTURE.json').read_bytes()) and s['forbidden_digest_total']==s['exceptions']==s['tar_pattern_issues']==0 and not s['other_candidate_counts'] and all(x['exit']==0 for x in s['checks'])
assert m['source_commit']=='95e63628f9d70b8dd6e6fede64334696c6995009' and m['source_tree']=='1eec97de340d9b7debbb30ca0c0fb2240a73b93f' and len(m['raw_serialized_response_sha256'])==64 and m['before_utc']<=m['after_utc'] and m['method']=='GitHub connector fetch branches/main API'
seen=set()
for row in c['rows']:
 n=PurePosixPath(row['path']);assert not n.is_absolute() and '..' not in n.parts and str(n) not in seen;seen.add(str(n));f=root.joinpath(*n.parts);assert f.is_file() and not f.is_symlink() and f.resolve().is_relative_to(root);b=f.read_bytes();assert (len(b),h(b),hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest())==(row['bytes'],row['sha256'],row['git_blob']) and bool(f.stat().st_mode&0o111)==(row['mode']=='100755')
print(f'Verified {len(seen)} captured public paths, bounded scan receipt and bracketed main observation metadata. The later guard files and opaque ciphertext are separately qualified.')
