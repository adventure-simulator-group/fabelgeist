#!/usr/bin/env python3
"""Read-only file, mode and hash check of this evidence packet."""
import hashlib,json,pathlib,stat,sys
if sys.flags.optimize:raise SystemExit('unoptimized interpreter required')
r=pathlib.Path(__file__).resolve().parent
name='PRIVATE-MANIFEST.json' if (r/'PRIVATE-MANIFEST.json').is_file() else 'MANIFEST.json'
m=json.loads((r/name).read_bytes());assert oct(stat.S_IMODE(r.stat().st_mode))==m['root_mode']
files={};dirs={}
for p in sorted(r.rglob('*')):
 assert not p.is_symlink(); rel=p.relative_to(r).as_posix();mode=oct(stat.S_IMODE(p.stat().st_mode))
 if p.is_dir():dirs[rel]=mode
 elif p.is_file() and rel!=name:
  b=p.read_bytes();files[rel]={'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'mode':mode}
 else:assert p.is_file()
assert dirs=={x['path']:x['mode'] for x in m['directories']}
assert files=={x['path']:{'bytes':x['bytes'],'sha256':x['sha256'],'mode':x['mode']} for x in m['files']}
print('Evidence packet bytes and modes match; semantic, privacy and external provenance claims remain separately qualified.')
