#!/usr/bin/env python3
"""Read-only checks of this bounded evidence review, not product validation."""
import hashlib,json,pathlib,datetime
P=pathlib.Path('<PRIVATE_PATH_0011>')
H=lambda b: hashlib.sha256(b).hexdigest()
def read(p): return (P/p).read_bytes()
def js(p): return json.loads(read(p))
def check_manifest(root, m):
    assert len({x['path'] for x in m['files']})==len(m['files'])
    for x in m['files']:
        p=root/x['path']; assert not p.is_symlink() and p.is_file()
        b=p.read_bytes(); assert len(b)==x['bytes'] and H(b)==x['sha256'],x['path']
    actual={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file() and p.relative_to(root).as_posix()!='MANIFEST.json'}
    assert actual=={x['path'] for x in m['files']}, (actual-{x['path'] for x in m['files']})
m=js('MANIFEST.json'); check_manifest(P,m)
assert H(read('initial/MANIFEST.json'))=='b3a09e212700d250c45fd17cc8f779c078e453b0fe07201eab6dd02b59e094ec'
check_manifest(P/'initial',js('initial/MANIFEST.json'))
expected={'initial/INITIAL-REVIEW.json':'e3136dc3ff3c8ab89c87909f47b36b241769e9f4718ab0c626e8fdbc2b2bb576','initial/coordinator/ASSESSMENT.json':'3b03e72fb781089e7974fe20c2e96ce66ceaa505963f281af61f6d3056f02106','coordinator/FINAL-ASSESSMENT.json':'ac429433dd7c22ab6d0db209a410bd867c48e87a76516c4734841471fe9ad68f','coordinator/EXT-REGISTRATION.json':'7958fb0ef04d9ac510c23021d865624b24478ae4878c43556f2d9d5690db6284','coordinator/EXT-READ-RECEIPT.json':'b1eee0036e30018336024e83617c9d599f4b35d622a4e6c898afe9c02227439b','coordinator/EXT-DISPLAY.txt':'87dcbe985eb8e22ff6489b6dbb42a41424e069a967d73394a809ebfe8fa5d17f','FINAL-REVIEW.json':'6ad953cb1f1bef162a16a0f9605a827072e8595c14843b2976b2a8c42d5bea16'}
for k,v in expected.items(): assert H(read(k))==v,k
old=js('initial/coordinator/ASSESSMENT.json'); final=js('coordinator/FINAL-ASSESSMENT.json')
assert {k for k in set(old)|set(final) if old.get(k)!=final.get(k)}=={'central_display_evidence','observations','review_qualification','source_references'}
r=js('coordinator/EXT-REGISTRATION.json'); receipt=js('coordinator/EXT-READ-RECEIPT.json')
assert receipt['registration_sha256']==expected['coordinator/EXT-REGISTRATION.json'] and receipt['display_sha256']==expected['coordinator/EXT-DISPLAY.txt']
assert datetime.datetime.fromisoformat(r['registered_utc'])<datetime.datetime.fromisoformat(receipt['recorded_utc'])
transcript=[]
for e in r['entries']:
    raw=read('initial/source/'+e['tag']+'.rs'); assert H(raw)==e['sha256'] and len(raw)==e['bytes']
    lines=raw.decode().splitlines(); assert len(lines)==e['eof']
    rr=next(x for x in receipt['rows'] if x['tag']==e['tag']); assert rr['displayed_ranges']==e['registered_ranges']
    for a,b in e['registered_ranges']:
        transcript.append(f"--- {e['tag']} {a}-{b} ---")
        transcript.extend(f'{i:4d} {lines[i-1]}' for i in range(a,b+1))
assert ('\n'.join(transcript)+'\n').encode()==read('coordinator/EXT-DISPLAY.txt')
entries=js('initial/coordinator/REGISTRATION.json')['entries']+r['entries']
for ref in final['source_references']:
    covered=set()
    for e in entries:
        if e['git_path']==ref['path']:
            for a,b in e['registered_ranges']: covered.update(range(a,b+1))
    for a,b in ref['lines']: assert set(range(a,b+1))<=covered,(ref['path'],a,b)
assert [x['index'] for x in js('initial/SELECTED-059-QUEUE.json')['rows']]==[1843,1844,1845]
assert 'positive' in json.dumps(final['observations']).lower()
print('PASS: final and embedded initial manifest identities, 7 bound hashes, registered 35-line extension, all final source-range coverage, selected 059 records; evidence only')
