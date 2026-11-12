#!/usr/bin/env python3
"""Create a city-assessment and separately named source-only verifier candidate in a new directory."""
from pathlib import Path
import argparse,gzip,hashlib,json
ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
p=args.publication;o=args.output
if o.exists():raise SystemExit('refusing to overwrite an existing candidate directory')
o.mkdir(parents=True)
sha=lambda b:hashlib.sha256(b).hexdigest()
source=p/'ASSESSMENT-CITY-STREAMING-READINESS-95E.json'; raw=source.read_bytes();assert sha(raw)=='d3d75ce6212526a047b08677495488f6fb8820ecbc9bdbef19f766a73e1a84ae'
a=json.loads(raw); cross=json.loads((Path(__file__).parent/'crosswalk.json').read_bytes());rows=cross['rows']; changed=[]
for row in rows:
 x=a['recovered_historical_records'][row['city_row_index']];assert x['original_path']==row['original_path'] and x['sha256']==row['original_decoded_sha256'] and x['redacted_local_path_occurrences']==0
 public=(p/row['cloud_publication_path']).read_bytes();assert sha(public)==row['cloud_gzip_sha256'];assert sha(gzip.decompress(public))==row['cloud_decoded_sha256']
 candidate=o/row['original_city_publication_path'];candidate.parent.mkdir(parents=True,exist_ok=True);candidate.write_bytes(public)
 x['published_bytes']=row['cloud_decoded_bytes'];x['published_sha256']=row['cloud_decoded_sha256'];x['redacted_local_path_occurrences']=1
 x['original_recovery']='Exact original from pinned 059 transport at original path, separately retained privately and verified by SHA-256 and Git blob; this public gzip is path-redacted.'
 x['qualification']+=' Public reconstruction changes only provenance/cache_path; original reviewer, revision, ranges and read credit are unchanged.'
 changed.append(row['city_row_index'])
a['publication_reconstruction']={'kind':'qualified public reconstruction; original assessment SHA-256 bound separately','original_assessment_sha256':sha(raw),'affected_recovered_historical_record_indices':changed,'scope':'One cache path string substituted in each of two archived historical registrations. No source read, audit credit, repair or product validation is implied.','crosswalk':'CITY-STREAMING-REDACTION-CROSSWALK.json'}
assessment_name='ASSESSMENT-CITY-STREAMING-READINESS-95E-CORRECTED.json';assessment=(json.dumps(a,indent=2,ensure_ascii=False)+'\n').encode();(o/assessment_name).write_bytes(assessment)
script=(p/'verify_city_streaming_readiness_95e.py').read_text();assert sha(script.encode())=='278e44689fd1310aadc62127958bcc32d555d871055d147d1c5fc0547bf15e9d'
old="r=Path(__file__).resolve().parent\nload=lambda p:json.loads((r/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();contents=lambda p:gzip.decompress((r/p).read_bytes())"
new="""# Qualified successor, generated from the historical verifier. Only the
# named assessment and its two gzip paths may resolve through the overlay.
import argparse
ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,default=Path(__file__).resolve().parent);ap.add_argument('--overlay',type=Path);args=ap.parse_args()
r=args.publication
_overlay_paths={%r,%r,%r}
def resolved(p):
 p=str(p)
 return (args.overlay/p if args.overlay is not None and p in _overlay_paths else r/p)
load=lambda p:json.loads(resolved(p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();contents=lambda p:gzip.decompress(resolved(p).read_bytes())
assert sha(resolved(%r).read_bytes())==%r
"""%(assessment_name,rows[0]['original_city_publication_path'],rows[1]['original_city_publication_path'],assessment_name,sha(assessment))
assert script.count(old)==1;script=script.replace(old,new)
old_name="a=load('ASSESSMENT-CITY-STREAMING-READINESS-95E.json')";new_name="a=load('"+assessment_name+"')";assert script.count(old_name)==1;script=script.replace(old_name,new_name)
anchor="manifest=json.loads(contents('recovered/original-transport-manifest.json.gz'));m={x['path']:x for x in manifest['entries']};records={}"
assert script.count(anchor)==1
expected=[{k:row[k] for k in ('original_path','original_city_publication_path','original_decoded_sha256','original_decoded_bytes','original_decoded_git_blob','cloud_publication_path','cloud_gzip_sha256','cloud_gzip_bytes','cloud_decoded_sha256','cloud_decoded_bytes')} for row in rows]
extra="""\n# The historical transport identity is an original-to-public crosswalk, not
# a claim that the publicly redacted JSON equals the original JSON.
_expected_redactions=%r
for expected in _expected_redactions:
 matches=[x for x in a['recovered_historical_records'] if x['original_path']==expected['original_path']]
 assert len(matches)==1
 x=matches[0];original=m[x['original_path']]
 assert (x['sha256'],x['bytes'],x['git_blob'])==(expected['original_decoded_sha256'],expected['original_decoded_bytes'],expected['original_decoded_git_blob'])==(original['sha256'],original['bytes'],original['git_blob'])
 assert x['preserved_bytes']==expected['original_city_publication_path'] and x['redacted_local_path_occurrences']==1
 assert (x['published_sha256'],x['published_bytes'])==(expected['cloud_decoded_sha256'],expected['cloud_decoded_bytes'])
 public=resolved(x['preserved_bytes']).read_bytes();cloud=(r/expected['cloud_publication_path']).read_bytes()
 assert public==cloud and (sha(public),len(public))==(expected['cloud_gzip_sha256'],expected['cloud_gzip_bytes'])
 plain=gzip.decompress(public);assert (sha(plain),len(plain))==(expected['cloud_decoded_sha256'],expected['cloud_decoded_bytes'])
 assert json.loads(plain)['provenance']['cache_path']=='<LOCAL_PATH_0>'
"""%expected
script=script.replace(anchor,anchor+extra)
script=script.replace("print('Verified five historical bodies, source diff, original attribution and conditional streaming/readiness connections; no product execution.')","print('Verified qualified city source/evidence assertions and original-to-redacted 059 crosswalks; no product execution.')")
(o/'verify_city_streaming_readiness_redacted_95e.py').write_text(script)
(o/'CITY-STREAMING-REDACTION-CROSSWALK.json').write_bytes((Path(__file__).parent/'crosswalk.json').read_bytes())
print('built',assessment_name,sha(assessment),'and separately named qualified successor',sha(script.encode()))
