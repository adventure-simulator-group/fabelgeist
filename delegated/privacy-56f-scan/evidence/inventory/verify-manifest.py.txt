import hashlib,json,pathlib
p=pathlib.Path(__file__).parent;m=json.loads((p/'BUNDLE-MANIFEST.json').read_bytes());files={f.relative_to(p).as_posix() for f in p.rglob('*') if f.is_file() and f.relative_to(p).as_posix()!='BUNDLE-MANIFEST.json'};assert len(m['files'])==m['file_count'] and files=={x['path'] for x in m['files']}
for x in m['files']:
 b=(p/x['path']).read_bytes();assert (len(b),hashlib.sha256(b).hexdigest())==(x['bytes'],x['sha256'])
print('PASS: every packet file, including any nested manifests; evidence integrity only')
