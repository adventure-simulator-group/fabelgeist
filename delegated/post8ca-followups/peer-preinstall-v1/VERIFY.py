#!/usr/bin/env python3
"""Verify public review bytes and optionally supplied reviewed-script bytes."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json
p=Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--capture-script',type=Path);a.add_argument('--installer-template',type=Path);args=a.parse_args();h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-preinstall-peer-public-manifest-v1' and {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,row in m['files'].items():
 assert len(PurePosixPath(n).parts)==1;f=p/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
b=json.loads((p/'BINDING.json').read_bytes());c=json.loads((p/'STATIC-CHECK-v2.json').read_bytes());s=json.loads((p/'SCANNER-CHECK.json').read_bytes());assert len(c['checks'])==28 and all(c['checks'].values()) and len(s['checks'])==5 and all(s['checks'].values()) and not b['final_scan_observed'] and not b['postinstall_placement_observed'];checked=[]
for name,f,key in [('capture',args.capture_script,'capture_script_sha256'),('installer',args.installer_template,'installer_template_sha256')]:
 if f is not None:assert f.is_file() and not f.is_symlink() and h(f.read_bytes())==b[key];checked.append(name)
print(json.dumps({'public_members':len(m['files']),'reviewed_script_identities_checked':checked,'qualification':'Public review bytes and optional script identities only; static checks, scan and installation are not repeated.'}))
