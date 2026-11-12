#!/usr/bin/env python3
import argparse,hashlib,json,pathlib,sys
p=pathlib.Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--q2968',type=pathlib.Path);ap.add_argument('--q3016',type=pathlib.Path);ap.add_argument('--baseline',type=pathlib.Path);a=ap.parse_args()
def sha(b):return hashlib.sha256(b).hexdigest()
def need(x,s):
 if not x:raise ValueError(s)
try:
 m=json.loads((p/'MANIFEST.json').read_text());expected={x['path']:x for x in m['files']};found={str(x.relative_to(p)) for x in p.rglob('*') if x.is_file() and x!=p/'MANIFEST.json'};need(found==set(expected) and not any(x.is_symlink() for x in p.rglob('*')),'file set')
 for n,x in expected.items():
  b=(p/n).read_bytes();need(len(b)==x['bytes'] and sha(b)==x['sha256'],'file identity')
 o=json.loads((p/'CORRECTION.json').read_text());need(o['lens_names'][0]=='producer propagation' and o['lens_names'][5]=='type ownership','map')
 for n,src in [('q2968',a.q2968),('q3016',a.q3016)]:
  if src:need(sha((src/'MANIFEST.json').read_bytes())==o['packet_identities'][n]['manifest_sha256'] and sha((src/'REPORT.md').read_bytes())==o['packet_identities'][n]['report_sha256'],'prior binding '+n)
 if a.baseline:
  b=a.baseline.read_bytes();need(sha(b)==o['observed_working_baseline_sha256'] and json.loads(b)['lens_names']==o['lens_names'],'baseline binding')
 print('Correction evidence verified; external bindings '+('complete.' if a.q2968 and a.q3016 and a.baseline else 'partial or omitted.'))
except Exception as e:print('FAILED:',e,file=sys.stderr);sys.exit(1)
