#!/usr/bin/env python3
"""Verify additive mapping correction; original and restricted checks optional."""
from pathlib import Path
import argparse,hashlib,json,os,subprocess,sys
r=Path(__file__).resolve().parent
if sys.flags.optimize:raise SystemExit('disable Python optimization')
a=argparse.ArgumentParser();a.add_argument('--publication',type=Path);a.add_argument('--restricted',type=Path);a=a.parse_args()
def h(b):return hashlib.sha256(b).hexdigest()
def need(v,m):
 if not v:raise SystemExit('FAIL: '+m)
m=json.loads((r/'MANIFEST.json').read_bytes());rows=m['files'];names={x['path'] for x in rows};need(len(names)==len(rows) and {p.name for p in r.iterdir()}==names|{'MANIFEST.json'},'packet file set')
for x in rows:
 f=r/x['path'];need(f.is_file() and not f.is_symlink(),'unsafe packet file');b=f.read_bytes();need(len(b)==x['bytes'] and h(b)==x['sha256'],'packet file')
c=json.loads((r/'CORRECTION.json').read_bytes());need(c['mapping_authority']['lens_1']=='producer propagation' and c['mapping_authority']['lens_6']=='type ownership' and [x['queue_index'] for x in c['packets']]==[2754,2910] and all(x['historical_lens']==1 and x['observed_perspective']=='type ownership' and x['historical_entry']['body_or_population_credit']==0 for x in c['packets']),'correction semantics')
print('Public correction packet and preserved historical labels verified.')
if a.publication:
 q1,q2=c['packets'];need(h((a.publication/q1['public_archive']).read_bytes())==q1['archive_sha256'],'q2754 archive');need(h((a.publication/q2['public_report']).read_bytes())==q2['report_sha256'],'q2910 report');need(h((a.publication/'delegated/postdac7-followups/q2910-packet/MANIFEST.json').read_bytes())==q2['public_manifest_sha256'],'q2910 manifest');print('Original public archive and q2910 report bound.')
else:print('Original public reports not checked; --publication is required.')
if a.restricted:
 f=a.restricted/'VERIFY-EVIDENCE-v2.py';need(h(f.read_bytes())=='ca632b271f434a4ffdb27c102bf98d00de4c2237105c165cf7db3dd770ca6356','restricted verifier identity');env={k:v for k,v in os.environ.items() if not k.startswith('PYTHON') and not k.startswith('GIT_')};env.update(PYTHONDONTWRITEBYTECODE='1',PYTHONNOUSERSITE='1');p=subprocess.run([sys.executable,'-B',str(f)],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE);need(p.returncode==0 and not p.stderr,'restricted evidence proof');print('Restricted historical and source-attribution proof verified.')
else:print('Restricted historical and source-attribution proof not checked; --restricted is required.')
print('No product execution, Lens1/Lens6 closure, count, alias or checkpoint3821 conclusion.')
