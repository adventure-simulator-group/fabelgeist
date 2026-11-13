#!/usr/bin/env python3
from pathlib import Path,PurePosixPath
import argparse,hashlib,json
p=Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--capture-script',type=Path);a.add_argument('--installer-template',type=Path);a.add_argument('--scan-runner',type=Path);args=a.parse_args();h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-preinstall-v2-peer-public-manifest-v1' and {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,row in m['files'].items():
 assert len(PurePosixPath(n).parts)==1;f=p/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
b=json.loads((p/'BINDING.json').read_bytes());c=json.loads((p/'STATIC-CHECK.json').read_bytes());assert len(c['checks'])==16 and all(c['checks'].values()) and b['target_scan_pin_blocker'] and not b['final_scan_observed'] and not b['postinstall_placement_observed'];checked=[]
for name,f,key in [('capture',args.capture_script,'capture_script_sha256'),('installer',args.installer_template,'installer_template_sha256'),('runner',args.scan_runner,'scan_runner_sha256')]:
 if f is not None:assert f.is_file() and not f.is_symlink() and h(f.read_bytes())==b[key];checked.append(name)
print(json.dumps({'public_members':len(m['files']),'script_identities_checked':checked,'scan_pin_blocker':True,'qualification':'Public bytes and optional script identities only; target scripts are not executed.'}))
