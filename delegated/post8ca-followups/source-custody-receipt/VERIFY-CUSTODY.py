#!/usr/bin/env python3
"""Read-only verification of receipt and optionally its ciphertext group."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json,stat,sys
if sys.flags.optimize:raise RuntimeError('unoptimized')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--ciphertext-dir',type=Path);a=ap.parse_args();H=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());expected={r['path']:r for r in m['files']};assert {x.name for x in p.iterdir()}==set(expected)|{'MANIFEST.json'}
for name,row in expected.items():
 q=PurePosixPath(name);assert len(q.parts)==1;f=p/name;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),H(b),oct(stat.S_IMODE(f.stat().st_mode)))==(row['bytes'],row['sha256'],row['mode'])
r=json.loads((p/'RESTORATION.json').read_bytes());assert r['restore_exit']==0 and r['matched_file_bytes_and_modes'] and r['matched_directory_modes'] and r['matched_root_modes'] and r['source_root_count']==4 and r['prior_unique_nonce_count']==646 and r['new_unique_nonce_count']==4 and r['all_nonces_distinct'];assert all(x['exit']==0 and x['stderr_bytes']==0 for x in r['selected_replay_checks'])
s=json.loads((p/'SCAN.json').read_bytes());assert s['positive_controls_pass'] and not any(s['pattern_counts'].values()) and s['private_source_literal_hits']==s['individual_path_digest_hits']==0
checked=0
if a.ciphertext_dir:
 root=a.ciphertext_dir;raw=(root/'MANIFEST.json').read_bytes();assert H(raw)==r['ciphertext_manifest_sha256'];x=json.loads(raw);assert len(x['objects'])==4
 for row in x['objects']:
  name=PurePosixPath(row['ciphertext']);assert len(name.parts)==1;q=root/name;b=q.read_bytes();assert not q.is_symlink() and (len(b),H(b))==(row['ciphertext_bytes'],row['ciphertext_sha256']);checked+=1
print(json.dumps({'receipt_files':len(expected),'ciphertext_objects_checked':checked,'restoration_comparison':'recorded passing','qualification':'Authenticated restore requires the separately held key; receipt verification does not independently replay product behavior or establish custody independence.'}))
