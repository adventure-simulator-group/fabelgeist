#!/usr/bin/env python3
from pathlib import Path,PurePosixPath
import argparse,hashlib,json
p=Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--runner',type=Path);a.add_argument('--capture',type=Path);a.add_argument('--installer',type=Path);args=a.parse_args();h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-preinstall-v3-peer-public-manifest-v1' and {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,row in m['files'].items():
 assert len(PurePosixPath(n).parts)==1;f=p/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
b=json.loads((p/'BINDING.json').read_bytes());c=json.loads((p/'STATIC-CHECK.json').read_bytes());assert len(c['checks'])==14 and all(c['checks'].values()) and b['scan_pin_blocker_corrected_statically'] and not b['final_scan_observed'] and not b['postinstall_placement_observed'];checked=[]
for name,f,key in [('runner',args.runner,'runner_sha256'),('capture',args.capture,'capture_sha256'),('installer',args.installer,'installer_sha256')]:
 if f is not None:assert f.is_file() and not f.is_symlink() and h(f.read_bytes())==b[key];checked.append(name)
print(json.dumps({'public_members':len(m['files']),'script_identities_checked':checked,'qualification':'Public bytes and optional script identities only; no reviewed script is executed.'}))
