#!/usr/bin/env python3
"""Verify the independent, time-bound audit verifier investigation."""
import argparse,gzip,hashlib,json,subprocess
from pathlib import Path,PurePosixPath
root=Path(__file__).resolve().parent;packet=root/'delegated/verifier-evolution-independent';scan=root/'delegated/verifier-evolution-independent-scan';H=lambda b:hashlib.sha256(b).hexdigest()
mb=(packet/'MANIFEST.json').read_bytes();assert H(mb)=='c01b7a445d6cf7791d8b2e420852df728f175f36a94bcaaa9567053e7983ae51';rows=json.loads(mb)['files'];assert len(rows)==3
for e in rows:
 p=PurePosixPath(e['path']);assert not p.is_absolute() and '..' not in p.parts;b=(packet/p).read_bytes();assert (len(b),H(b))==(e['bytes'],e['sha256'])
assert {p.relative_to(packet).as_posix() for p in packet.rglob('*') if p.is_file()}=={'MANIFEST.json'}|{e['path'] for e in rows}
method=json.loads((packet/'METHOD.json').read_bytes());info=method['raw_tree_listing'];stored=(packet/info['gzip_path']).read_bytes();raw=gzip.decompress(stored);assert (len(stored),H(stored),len(raw),H(raw))==(info['gzip_bytes'],info['gzip_sha256'],info['raw_bytes'],info['raw_sha256'])
assert len(raw.split(b'\0'))-1==method['tree_inventory']['tracked_entries']==23030
rb=(scan/'RECEIPT.json').read_bytes();assert H(rb)=='bab206690db5be59ccb069aa7add54190d34c2077a3b95a4db77785ab5580c2b';rec=json.loads(rb);assert rec['packet_manifest_sha256']==H(mb)
for name,digest in rec['files'].items():assert H((scan/name).read_bytes())==digest
digest=json.loads((scan/'DIGEST-SCAN.json').read_bytes());pathscan=json.loads((scan/'PATH-SCAN-RESULT.json').read_bytes());assert digest['forbidden_digest_total']==0 and not digest['other_candidate_counts'] and not digest['exceptions'] and pathscan['stored_sha256']==H(stored) and pathscan['entries']==23030 and not any(pathscan['counts'].values())
p=argparse.ArgumentParser();p.add_argument('--git-repo',type=Path);args=p.parse_args()
if args.git_repo:
 got=subprocess.check_output(['git','-C',str(args.git_repo),'--no-replace-objects','ls-tree','-r','-l','-z',method['publication_commit']]);assert got==raw
 print('Specified local Git repository reproduced the exact time-bound tree listing.')
else:print('No Git repository supplied; exact historical tree enumeration was not repeated here.')
print('Verified bounded investigation, input listing and privacy receipts; no historical replay, full privacy or live-state equivalence is established.')
