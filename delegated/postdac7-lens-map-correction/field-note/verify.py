#!/usr/bin/env python3
import argparse,hashlib,json,pathlib,sys
p=pathlib.Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--q2968',type=pathlib.Path);ap.add_argument('--q3016',type=pathlib.Path);ap.add_argument('--map-correction',type=pathlib.Path);a=ap.parse_args()
def sha(b):return hashlib.sha256(b).hexdigest()
def need(x,s):
 if not x:raise ValueError(s)
try:
 m=json.loads((p/'MANIFEST.json').read_text());wanted={x['path']:x for x in m['files']};found={str(x.relative_to(p)) for x in p.rglob('*') if x.is_file() and x!=p/'MANIFEST.json'};need(found==set(wanted) and not any(x.is_symlink() for x in p.rglob('*')),'file set')
 for n,v in wanted.items():
  b=(p/n).read_bytes();need(sha(b)==v['sha256'] and len(b)==v['bytes'],'file identity')
 d=json.loads((p/'FIELD-NOTE.json').read_text())
 for n,root in [('q2968',a.q2968),('q3016',a.q3016)]:
  if root:
   for file,key in [('MANIFEST.json','manifest_sha256'),('REPORT.md','report_sha256'),('READ-RECEIPT.json','receipt_sha256'),('FOCUSED-SKILL.md','focused_skill_sha256')]:need(sha((root/file).read_bytes())==d['packets'][n][key],'packet binding '+n)
 if a.map_correction:need(sha((a.map_correction/'MANIFEST.json').read_bytes())==d['prior_mapping_correction_manifest_sha256'],'prior map binding')
 print('Field-level correction verified; external bindings '+('complete.' if a.q2968 and a.q3016 and a.map_correction else 'partial or omitted.'))
except Exception as e:print('FAILED:',e,file=sys.stderr);sys.exit(1)
