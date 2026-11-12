#!/usr/bin/env python3
"""Verify qualified current-tree evidence reconstructions without product execution."""
import gzip,hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();load=lambda p:json.loads((R/p).read_bytes())
q=load('QUALIFICATION-PUBLIC-EVIDENCE-PRIVACY-56F.json');rows=q['records'];assert len(rows)==9 and len({x['path'] for x in rows})==9 and sum(x['redactions'] for x in rows)==63
for x in rows:
 p=(R/x['path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();b=p.read_bytes();assert (sha(b),len(b))==(x['public_sha256'],x['public_bytes']) and x['public_sha256']!=x['earlier_sha256']
by={x['path']:x for x in rows};base='delegated/privacy-baseline059-proposal/';bs=load(base+'RECONSTRUCTION.json');transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));tp={x['path']:x for x in transport['entries']}
for x in bs['records']:
 r=by[x['public_path']];assert (R/x['public_path']).read_bytes()==(R/base/x['public_path']).read_bytes();o=tp[x['historical_transport_path']];assert (r['earlier_sha256'],r['earlier_bytes'])==(x['original_sha256'],x['original_bytes'])==(o['sha256'],o['bytes']) and r['public_sha256']==x['public_sha256']
prefix='delegated/privacy-historical-objects-proposal/';c=load(prefix+'CROSSWALK.json')
for x in c['objects']:
 o=x['original'];v=x['variant'];r=by[o['publication_path']];assert (r['earlier_sha256'],r['earlier_bytes'])==(o['sha256'],o['bytes']) and (r['public_sha256'],r['public_bytes'])==(v['sha256'],v['bytes']) and (R/o['publication_path']).read_bytes()==(R/prefix/v['path']).read_bytes()
extra=load('recovered/privacy-redaction-evidence/ADDITIONAL-CROSSWALK.json')
assert len(extra['records'])==2
for x in extra['records']:
 r=by[x['publication_path']];assert r['earlier_sha256']==x['original_sha256'] and r['public_sha256']==x['public_sha256']
city=load('delegated/city-streaming-privacy-proposal/crosswalk.json')
for x in city['rows']:
 r=by[x['original_city_publication_path']];b=(R/x['original_city_publication_path']).read_bytes();assert sha(b)==x['cloud_gzip_sha256'] and b==(R/x['cloud_publication_path']).read_bytes() and r['earlier_sha256']==x['original_gzip_sha256']
n=load('NATIVE-SERDE-SAPHYR-PUBLIC-REGISTRY-OBSERVATION.json');archive=(R/n['stored_original_archive']).read_bytes();assert sha(archive)==n['registry_checksum_sha256'] and len(archive)==n['stored_original_bytes']
assert q['repair']=='not started' and q['product_validation']=='not performed' and 'not purged' in q['status']
print('Verified nine distinct public reconstruction hashes, historical identity crosswalks, 63 reported removals and retained public registry checksum. Original/private substitutions and earlier Git history are not verified or purged by this public check.')
