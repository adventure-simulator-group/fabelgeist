#!/usr/bin/env python3
"""Read-only, bounded verification of two historical/public registration crosswalks."""
import argparse,gzip,hashlib,json,re,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);ap.add_argument('--private-dir',type=Path);ap.add_argument('--extraction',type=Path);ap.add_argument('--commit',help='optionally check committed copies at this Git commit');args=ap.parse_args()
if args.extraction and not args.private_dir:ap.error('--extraction requires --private-dir')
sha=lambda b:hashlib.sha256(b).hexdigest()
def blob(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
idx=json.loads((HERE/'crosswalk.json').read_bytes()); rows=idx['rows'];assert len(rows)==2 and {x['original_path'] for x in rows}=={'adler/REGISTRATION-ADLER-352.json','adler/REGISTRATION-ADLER-353.json'}
r=args.publication;transport=(r/'recovered/original-transport-manifest.json.gz').read_bytes();assert sha(transport)=='7e45ce9d858f93e0c5e5d50f360f79ed8941e28049f305053f1fd2bdda364834';transport_plain=gzip.decompress(transport);assert sha(transport_plain)=='c3e04095a9e4cb3e2da6ee2a4f561c811ebeca445a50461ddb63b2d426ed6858';transport_doc=json.loads(transport_plain);assert transport_doc['payload_commit']=='3378fe10b90d6ebe9bd90b624f55e4d78a0d1c28' and transport_doc['transport_only'] and transport_doc['source_read_credit']==0;m={x['path']:x for x in transport_doc['entries']}
citybytes=(HERE/'reference/historical-city-assessment.json').read_bytes();cloudbytes=(r/'ASSESSMENT-CLOUD-BAKE-ADMISSION-95E.json').read_bytes();assert sha(citybytes)=='d3d75ce6212526a047b08677495488f6fb8820ecbc9bdbef19f766a73e1a84ae';assert sha(cloudbytes)=='b6fb2bd59ddbea1c369b0836144b48d9ae96a034b432e91915a945236c88ea22';city=json.loads(citybytes);cloud=json.loads(cloudbytes)
candidate_bytes=(HERE/'candidate/ASSESSMENT-CITY-STREAMING-READINESS-95E-CORRECTED.json').read_bytes();assert sha(candidate_bytes)=='7eff3ca86b37c5a8b3738a2d96cf17d1916bce1168b9cf81c1cc553063329b6c';candidate=json.loads(candidate_bytes)
for row in rows:
 original=m[row['original_path']];expected=row['historical_record_tuple'];assert (original['path'],original['sha256'],original['bytes'],original['git_blob'])==(expected['path'],expected['sha256'],expected['bytes'],expected['git_blob'])
 x=city['recovered_historical_records'][row['city_row_index']];y=cloud['recovered_historical_records'][row['cloud_row_index']];z=candidate['recovered_historical_records'][row['city_row_index']]
 for obj in (x,y,z):assert (obj['original_path'],obj['sha256'],obj['bytes'],obj['git_blob'])==(expected['path'],expected['sha256'],expected['bytes'],expected['git_blob'])
 assert x['redacted_local_path_occurrences']==0 and x['published_sha256']==row['original_decoded_sha256'] and x['published_bytes']==row['original_decoded_bytes']
 for obj in (y,z):assert (obj['published_sha256'],obj['published_bytes'],obj['redacted_local_path_occurrences'])==(row['cloud_decoded_sha256'],row['cloud_decoded_bytes'],1)
 stored=(r/row['cloud_publication_path']).read_bytes();staged=(HERE/'candidate'/row['original_city_publication_path']).read_bytes();assert stored==staged==(HERE/(row['original_decoded_sha256']+'.cloud.gz')).read_bytes();assert (sha(stored),len(stored))==(row['cloud_gzip_sha256'],row['cloud_gzip_bytes']);plain=gzip.decompress(stored);assert (sha(plain),len(plain),blob(plain))==(row['cloud_decoded_sha256'],row['cloud_decoded_bytes'],row['cloud_decoded_git_blob']);doc=json.loads(plain);assert doc['provenance']['cache_path']=='<LOCAL_PATH_0>' and doc['provenance']['package']=='bevy_render' and doc['provenance']['version']=='0.19.1' and doc['provenance']['frozen_lock_commit']=='6533e444fa37552927070cc8d8190907fd3b413e';assert doc['provenance']['checksum_file_missing'] and doc['additional_original_frontier_credit']==0 and not doc['other_lens_coverage_transferred']
 source_scopes={'adler/REGISTRATION-ADLER-352.json':('native-support:bevy_render-0.19.1/src/render_asset.rs',[[280,364]],185),'adler/REGISTRATION-ADLER-353.json':('native-support:bevy_render-0.19.1/src/texture/gpu_image.rs',[[26,38]],186)};scope=source_scopes[row['original_path']];assert (doc['source_id'],doc['ranges'],doc['population_index'])==scope and doc['lens']==11 and doc['population']=='later-material' and not doc['new_global_source']
 # Scan this representation for the specific Windows account-path form only.
 assert not re.search(rb'(?i)[a-z]:(?:\\\\){1,2}users(?:\\\\){1,2}',plain)
 if args.private_dir:
  private=(args.private_dir/(row['original_decoded_sha256']+'.original.gz')).read_bytes();assert (sha(private),len(private))==(row['original_gzip_sha256'],row['original_gzip_bytes']);old=gzip.decompress(private);assert (sha(old),len(old),blob(old))==(row['original_decoded_sha256'],row['original_decoded_bytes'],row['original_decoded_git_blob']);od=json.loads(old);oldval=od['provenance']['cache_path'];newval=doc['provenance']['cache_path'];before=json.dumps(oldval,ensure_ascii=True).encode();after=json.dumps(newval,ensure_ascii=True).encode();assert old.count(before)==1 and old.replace(before,after,1)==plain and plain.count(after)==1 and plain.replace(after,before,1)==old;od['provenance']['cache_path']=doc['provenance']['cache_path']=None;assert od==doc
  if args.extraction:assert (args.extraction/row['original_path']).read_bytes()==old
 # Exactly these five row fields may differ, preserving all other assessment data.
 for k in set(x)|set(z):
  if k not in {'published_bytes','published_sha256','redacted_local_path_occurrences','original_recovery','qualification'}:assert x.get(k)==z.get(k)
oldcity=dict(city);newcity=dict(candidate);oldrows=oldcity.pop('recovered_historical_records');newrows=newcity.pop('recovered_historical_records');notice=newcity.pop('publication_reconstruction');assert oldcity==newcity
for i,(x,z) in enumerate(zip(oldrows,newrows)):
 if i not in {row['city_row_index'] for row in rows}:assert x==z
assert len(oldrows)==len(newrows) and notice['original_assessment_sha256']==sha(citybytes)
if args.commit:
 assert re.fullmatch(r'[0-9a-f]{40}',args.commit)
 paths=['ASSESSMENT-CITY-STREAMING-READINESS-95E.json','ASSESSMENT-CLOUD-BAKE-ADMISSION-95E.json','verify_city_streaming_readiness_95e.py','verify_cloud_bake_admission_95e.py','recovered/original-transport-manifest.json.gz','check_published_evidence.py']+[x['original_city_publication_path'] for x in rows]+[x['cloud_publication_path'] for x in rows]
 expected_hash={'ASSESSMENT-CITY-STREAMING-READINESS-95E.json':'d3d75ce6212526a047b08677495488f6fb8820ecbc9bdbef19f766a73e1a84ae','ASSESSMENT-CLOUD-BAKE-ADMISSION-95E.json':'b6fb2bd59ddbea1c369b0836144b48d9ae96a034b432e91915a945236c88ea22','verify_city_streaming_readiness_95e.py':'278e44689fd1310aadc62127958bcc32d555d871055d147d1c5fc0547bf15e9d','verify_cloud_bake_admission_95e.py':'62f4999045d2128bffd470fff0200a49a179bdc63d70f35a91d064ed7d333b26','recovered/original-transport-manifest.json.gz':'7e45ce9d858f93e0c5e5d50f360f79ed8941e28049f305053f1fd2bdda364834','check_published_evidence.py':'6b5f37e644b8c0f57eb18db64b0f60794ad995541a22db3c62fa7bc1216d60eb'}
 expected_hash.update({x['original_city_publication_path']:x['original_gzip_sha256'] for x in rows});expected_hash.update({x['cloud_publication_path']:x['cloud_gzip_sha256'] for x in rows})
 for path in paths:
  b=subprocess.check_output(['git','-C',str(r),'show',args.commit+':'+path]);assert sha(b)==expected_hash[path]
 print('Checked individual historical committed byte identities; this says nothing about overall worktree cleanliness.')
print('Checked public stored/decoded identities, candidate assessment scope, original transport tuples and individual historical attribution; these are crosswalks, not exact original/public byte matches.')
if args.private_dir:print('Checked private original stored/decoded bytes and exact one-string substitution'+(' plus read-only extraction match.' if args.extraction else '.'))
else:print('Private originals not supplied: exact original-to-public substitution and reversal were not checked.')
print('Specific account-path pattern only; no general secret or privacy completeness claim. No product code executed.')
