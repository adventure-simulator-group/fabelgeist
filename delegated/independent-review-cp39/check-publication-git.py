#!/usr/bin/env python3
"""Read only: compare the review inputs to already committed publication objects."""
import argparse,gzip,hashlib,json,pathlib,subprocess
ap=argparse.ArgumentParser();ap.add_argument('--public',type=pathlib.Path,required=True);ap.add_argument('--publication',type=pathlib.Path,required=True);a=ap.parse_args()
commit='56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e';f82='f82adcabd9d89c4fa2dd7f99253a45297a0681ee'
def git(*v):return subprocess.run(['git','--no-optional-locks','-C',str(a.publication),'-c','core.fsmonitor=false','--no-pager',*v],check=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE).stdout
H=lambda b:hashlib.sha256(b).hexdigest()
raw=git('cat-file','commit',commit);indexed=gzip.decompress((a.public/'bundles/2623-2624/git-objects'/f'{commit}.commit.gz').read_bytes());assert raw==indexed and hashlib.sha1(b'commit '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==commit
base=git('show',f'{commit}:BASELINE-95E-20261110.json');captured=(a.public/'bundles/2623-2624/history/published/BASELINE-95E-20261110.json').read_bytes();assert base==captured and H(base)=='28138b4548101716246fee073984868472996f0e350d3f3526d237d61c6a67f9'
rel=f'recovered/physical-p16-revision-comparison/git-objects/{f82}.commit.gz';original=git('show',f'{commit}:{rel}');stage=(a.public/'bundles/2623-2624/git-objects'/f'{f82}.commit.gz').read_bytes();assert original==stage and H(stage)=='a8523ddb9764b93a37c52c0a0b188a43cfbfe55c6d7c54257af75f8ae3ea9927'
print(json.dumps({'commit_oid':commit,'commit_raw_sha256':H(raw),'committed_baseline_sha256':H(base),'committed_baseline_git_blob':git('rev-parse',f'{commit}:BASELINE-95E-20261110.json').decode().strip(),'f82_object_stored_sha256':H(stage),'f82_object_git_blob':git('rev-parse',f'{commit}:{rel}').decode().strip(),'f82_committed_path':rel,'qualification':'Read-only comparisons to retained local Git objects. This does not establish live-remote availability, working-tree cleanliness or product behavior.'},indent=2))
