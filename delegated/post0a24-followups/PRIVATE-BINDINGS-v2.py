#!/usr/bin/env python3
"""Authenticate exact private evidence and bind it to public transports in memory."""
import argparse,hashlib,io,json,sys,tarfile
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError('Evidence assertions require an unoptimized interpreter')
P=Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest();ap=argparse.ArgumentParser();ap.add_argument('--key',type=Path);a=ap.parse_args();manifest=json.loads((P/'encrypted-originals/MANIFEST.json').read_bytes());rows={x['label']:x for x in manifest['objects']}
needed={'q2708','q2721','q2734','q2747-2773','q2750','q2760','q2786','q2799','q2812','q2884','private-control','transport-prescan','independent-reviews'};assert set(rows)==needed
if not a.key:
 print('Private collection labels are present; NOT REPEATED: keyed authentication, public/private binding and original restoration require the separately held key.')
 raise SystemExit(0)
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
key=a.key.read_bytes();assert len(key)==32;contents={}
for label,row in rows.items():
 blob=(P/'encrypted-originals'/row['ciphertext']).read_bytes();assert (len(blob),H(blob))==(row['ciphertext_bytes'],row['ciphertext_sha256']);plain=AESGCM(key).decrypt(blob[:12],blob[12:],row['associated_data'].encode());assert (len(plain),H(plain))==(row['plaintext_archive_bytes'],row['plaintext_archive_sha256']);members={};dirs=0
 with tarfile.open(fileobj=io.BytesIO(plain),mode='r:gz') as tar:
  for m in tar.getmembers():
   n=PurePosixPath(m.name);assert not n.is_absolute() and n.parts[0]=='evidence' and '..' not in n.parts and (m.isfile() or m.isdir())
   if m.isdir():dirs+=1;continue
   assert m.name not in members;b=tar.extractfile(m).read();members[m.name]=(len(b),H(b),m.mode)
 assert len(members)==row['regular_files'] and dirs==row['directories'];contents[label]=members
transport=json.loads((P/'transport/MANIFEST.json').read_bytes());mapping={'q2708':('q2708','public'),'q2734':('q2734','delivery'),'q2747':('q2747-2773','q2747-delivery'),'q2760':('q2760','public'),'q2786':('q2786','public'),'q2812':('q2812','delivery'),'q2884':('q2884','delivery')}
for r in transport['copied_files']:
 group,tail=r['path'].split('/',1);label,sub=mapping[group];entry=contents[label]['evidence/'+sub+'/'+tail];assert entry==(r['bytes'],r['sha256'],int(r['mode'],8))
for packet in transport['packets']:
 label=packet['label'];collection,sub={'q2721':('q2721','packet'),'q2773':('q2747-2773','q2773-packet'),'q2799':('q2799','packet')}[label]
 expected={m['path'].replace('evidence/','evidence/'+sub+'/',1):(m['bytes'],m['sha256'],int(m['mode'],8)) for m in packet['members']};actual={k:v for k,v in contents[collection].items() if k.startswith('evidence/'+sub+'/')};assert actual==expected
extra=json.loads((P/'transport-extra/MANIFEST.json').read_bytes())
for r in extra['files']:
 assert r['path'].startswith('q2750/');tail=r['path'].split('/',1)[1];assert contents['q2750']['evidence/public/'+tail]==(r['bytes'],r['sha256'],int(r['mode'],8))
anchors=[('q2750','evidence/isolated-replay/MANIFEST.json','18c45d4bb37b6d137b7ecd79f3f255c14d2db73db36ba850e68aacc7cf31f003'),('q2799','evidence/isolated-replay/MANIFEST.json','4171e83490e290d5f953cd011a7f21e49cee8f5ccbe51696b4ed4a71151f972d'),('q2812','evidence/anchors/ANCHOR-MANIFEST.json','e3262991171c1b8279952062621c01b95803d573fbb4a20cd05cb8b2cc794a2d'),('q2884','evidence/selection-private/SELECTION-MANIFEST.json','564683ed26dd840fed18c2ca6ee44d86a5d05aa2ff2136687ca7053feb7156a7'),('independent-reviews','evidence/draft-review/MANIFEST.json','69586e288fc5cb702a4d6a8499cb7eec3c19c1ec5acf3198fc475e0b64c8a0db'),('independent-reviews','evidence/draft-review-dependencies/MANIFEST.json','30cb141a1f774fd2d8f41bd7c39c3c5e3da10f7ba7535e60fd480c8efcb91ab1'),('independent-reviews','evidence/prism-review/MANIFEST.json','e5d234150ac668bc062eed9905bdefc28b17035212faf33754ca50e62da7ff57'),('independent-reviews','evidence/prism-replay-inputs/INVENTORY.json','c6076ea5dd7324b6fdc10edbf63614162cc4e269d9011b52a8e78aafe3382412'),('independent-reviews','evidence/correction-review/MANIFEST.json','5b444c25868ffe7e437ef5d27de13794d9e8510cedbf38670125c21102951fae'),('independent-reviews','evidence/final-attribution-review/MANIFEST.json','513c3db58354d62f0c9c65151a64e2c1e1b87203f40f6d3a8495612d45962c23')]
for label,path,digest in anchors:assert contents[label][path][1]==digest
print('Authenticated all private archives and bound public transports and selected replay anchors in memory; no evidence was restored. Same-machine key custody and product behavior remain separate.')
