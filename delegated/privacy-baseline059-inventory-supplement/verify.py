#!/usr/bin/env python3
"""Read-only verification of the additive 059 reference inventory."""
from pathlib import Path
import argparse,gzip,hashlib,io,json,tarfile
P=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path);args=ap.parse_args()
m=json.loads((P/'MANIFEST.json').read_bytes());assert m['schema']=='726-baseline059-inventory-supplement-manifest-v1';actual={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file()}-{'MANIFEST.json'};assert actual==set(m['files'])
for n,x in m['files'].items():
 b=(P/n).read_bytes();assert sha(b)==x['sha256'] and len(b)==x['bytes'],n
report=json.loads((P/'REPORT.json').read_bytes());plan=json.loads((P/'INTEGRATION-PLAN.json').read_bytes());tail=json.loads((P/'TAILCHECK-INVENTORY.json').read_bytes());non=json.loads((P/'NONDELEGATED-TAILCHECK.json').read_bytes());old=json.loads(gzip.decompress((P/'REFERENCE-SHALLOW-INVENTORY.json.gz').read_bytes()));new=json.loads((P/'NEW-DEEP-MATCHES.json').read_bytes());rec=json.loads((P/'REFERENCE-RECONSTRUCTION.json').read_bytes());pointer=json.loads((P/'REFERENCE-AFFECTED-POINTERS.json').read_bytes());links=json.loads((P/'REFERENCE-HASH-LINK-INVENTORY.json').read_bytes())
assert len(plan['fifteen_affected_checks'])==15 and [x['checkpoint'] for x in plan['fifteen_affected_checks']]==list(range(15,30))
assert plan['seven_semantic_pointer_references']==pointer['records'] and len(pointer['records'])==7
assert plan['existing_manifest_bindings']['explicit_checkpoint20_29_direct_link_records']==links['direct_link_manifest_records'] and len(links['direct_link_manifest_records'])==20
assert plan['existing_manifest_bindings']['checkpoint16_19_archive_hash_link_records']==links['archived_hash_link_records_checkpoint16_19'] and len(links['archived_hash_link_records_checkpoint16_19'])==35
for a in rec['records']:
 t=next(x for x in plan['historical_transport_proof']['records'] if x['path']==a['historical_transport_path'])
 assert (a['original_sha256'],a['original_bytes'],a['original_git_blob'],a['public_sha256'],a['public_bytes'],a['public_git_blob'])==(t['sha256'],t['bytes'],t['git_blob'],t['public_sha256'],t['public_bytes'],t['public_git_blob'])
assert tail['archive_count_enumerated']==len(tail['archive_records'])==354 and tail['outer_regular_member_count']==34990 and len(tail['matching_representations'])==1739 and not tail['errors']
assert len(tail['bounded_unscanned'])==1 and tail['bounded_unscanned'][0]['parser_offset']==69120 and tail['bounded_unscanned'][0]['member_chain']=='prior-web-map-route-packet.tar.gz'
assert non['file_count_enumerated']==len(non['file_records'])==15091 and len(non['matching_representations'])==35 and not non['bounded_unscanned'] and not non['errors']
assert {x['path']:x['sha256'] for x in old['archive_records']}=={x['path']:x['sha256'] for x in tail['archive_records']}
key=lambda x:(x['archive'],x['member_chain'],x['representation'],x['sha256']);olds={key(x) for x in old['matching_representations']};extras=[x for x in tail['matching_representations'] if key(x) not in olds]
assert extras==new['records'] and len(extras)==68
if args.publication:
 r=args.publication
 for x in plan['fifteen_affected_checks']:
  b=(r/x['script']).read_bytes();assert sha(b)==x['script_sha256'] and len(b)==x['script_bytes']
 p=plan['historical_transport_proof'];b=(r/p['path']).read_bytes();plain=gzip.decompress(b);assert sha(b)==p['stored_sha256'] and sha(plain)==p['uncompressed_sha256']
 items={x['path']:x for x in json.loads(plain)['entries']}
 for x in p['records']:
  y=items[x['path']];assert (x['sha256'],x['bytes'],x['git_blob'])==(y['sha256'],y['bytes'],y['git_blob'])
 c=report['cp7_exception'];index=(r/c['index_path']).read_bytes();assert sha(index)==c['index_sha256'];successor=r/c['successor_path'];assert sha(successor.read_bytes())==c['successor_sha256']
 with tarfile.open(successor,'r:gz') as t:assert len(t.getmembers())==c['successor_entries']==186
 with tarfile.open(r/'delegated/strategic-map-evidence.tar.gz','r:gz') as t:b=t.extractfile('prior-web-map-route-packet.tar.gz').read()
 assert sha(b)==c['damaged_public_sha256'];plain=gzip.decompress(b)
 with tarfile.open(fileobj=io.BytesIO(plain),mode='r:') as t:entries=list(t);offset=t.offset
 assert len(entries)==c['damaged_entries_visible']==5 and offset==c['bad_header_parser_offset']==69120
 try:tarfile.TarInfo.frombuf(plain[offset:offset+512],'utf-8','surrogateescape')
 except tarfile.InvalidHeaderError:pass
 else:raise AssertionError('Expected damaged checkpoint 7 next-header checksum')
 print('Checked recorded 15 script hashes, original transport tuples and the two distinct checkpoint 7 archive representations, read-only.')
else:print('Publication tree not supplied: external script, transport and checkpoint 7 identities were not rechecked in this invocation.')
print('Verified frozen supplement, exact 15 checks and seven pointer records, deep scan comparison and documented archive exception. No old verifier or product code was run.')
