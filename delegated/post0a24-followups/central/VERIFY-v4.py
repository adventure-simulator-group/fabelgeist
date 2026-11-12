#!/usr/bin/env python3
"""Verify bounded central audit evidence, not product behavior or semantic truth."""
import argparse,hashlib,io,json,sys,tarfile
from pathlib import Path,PurePosixPath
if sys.flags.optimize: raise RuntimeError('Evidence assertions require an unoptimized interpreter')
P=Path(__file__).resolve().parent
ap=argparse.ArgumentParser();ap.add_argument('--reports-root',type=Path,required=True);args=ap.parse_args()
H=lambda b:hashlib.sha256(b).hexdigest()
manifest=json.loads((P/'MANIFEST.json').read_bytes()); data=(P/manifest['archive']).read_bytes()
assert manifest['schema']=='726-central-evidence-15bc-v1' and manifest['parent_checkpoint']=='0a2466a7485388575ef272e0a8355b9714204a37'
assert len(data)==manifest['archive_bytes'] and H(data)==manifest['archive_sha256']
files={}
with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz') as tar:
 for member in tar.getmembers():
  n=PurePosixPath(member.name);assert member.isfile() and not n.is_absolute() and n.parts[0]=='evidence' and '..' not in n.parts and member.name not in files
  b=tar.extractfile(member).read();files[member.name]=(b,member.mode)
assert len(files)==len(manifest['members'])
for row in manifest['members']:
 b,mode=files[row['path']];assert (len(b),H(b),oct(mode))==(row['bytes'],row['sha256'],row['mode'])
def get(part,group,name): return files['evidence/'+part+'/'+group+'/'+name][0]
def obj(part,group,name):return json.loads(get(part,group,name))
for part in ('15b','15c'):
 rr=obj(part,'reports','READ-RECEIPT.json'); reports=rr.get('files',rr.get('reports'))
 for row in reports:
  b=get(part,'reports',row['file']);assert (len(b),H(b))==(row['bytes'],row['sha256']) and row['display'] in ('complete and untruncated','complete, untruncated')
 if 'registration_sha256' in rr:assert H(get(part,'reports','REGISTRATION.json'))==rr['registration_sha256']
 plan=obj(part,'source','REGISTRATION.json');receipt=obj(part,'source','READ-RECEIPT.json');assert H(get(part,'source','REGISTRATION.json'))==receipt['registered_plan_sha256']
 for row in plan['rows']:
  b=get(part,'source',row['captured_as']);assert (len(b),H(b))==(row['bytes'],row['sha256'])
  assert hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==row['blob']
  assert len(b.decode().splitlines())==row['source_lines']
 for view in receipt['displays']:
  b=get(part,'source',view['file']);assert (len(b),H(b))==(view['bytes'],view['sha256']) and view['display']=='complete and untruncated'
  group=view['file'][len('DISPLAY-'):-len('.txt')];output=''
  for row in plan['rows']:
   if row['group']!=group:continue
   lines=get(part,'source',row['captured_as']).decode().splitlines();output+='\n### '+row['path']+'\n'
   for a,z in row['display_ranges']:
    output+=''.join('%5d %s\n'%(n,lines[n-1]) for n in range(a,z+1))
  assert output.encode()==b
 if part=='15b':
  reg=obj(part,'source','REGISTRATION-SUPPORT.json');r=obj(part,'source','READ-SUPPORT.json');b=get(part,'source','SOURCE-SUPPORT.dat');v=get(part,'source','DISPLAY-SUPPORT.txt')
  assert H(get(part,'source','REGISTRATION-SUPPORT.json'))==r['registration_sha256'] and (len(b),H(b))==(reg['bytes'],reg['sha256']) and H(v)==r['display_sha256'] and len(v)==r['display_bytes']
  assert hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==reg['blob'];lines=b.decode().splitlines();a,z=reg['range'];assert ''.join('%5d %s\n'%(n,lines[n-1]) for n in range(a,z+1)).encode()==v
 base=get(part,'ledger','BASELINE.json');pl=obj(part,'ledger','PROJECTION-PLAN.json');lr=obj(part,'ledger','READ-RECEIPT.json');assert H(base)==pl['baseline_sha256']=='2dc3ba57b36d4792ea4990b53beb266b1d2d513064e8df149afdeb4d991ab593' and H(get(part,'ledger','PROJECTION-PLAN.json'))==lr['plan_sha256'];entries=json.loads(base)['newly_assessed_ledger']['entries']
 for ix,ids in enumerate(pl['selection']):
  name='PROJECTION-%02d.json'%ix;v=get(part,'ledger',name);check=next(t for t in lr['displays'] if t['file']==name);assert (len(v),H(v))==(check['bytes'],check['sha256']);assert json.loads(v)==[{'index':j,'entry':entries[j]} for j in ids]
 draft=get(part,'drafts','ASSESSMENT-CENTRAL-FOLLOWUPS-'+part.upper()+'-95E-v2.json');final=json.loads((args.reports_root/('ASSESSMENT-CENTRAL-FOLLOWUPS-'+part.upper()+'-95E.json')).read_bytes());assert H(draft)==final['source_draft_sha256'];d=json.loads(draft)
 correction=json.loads((P.parent/'REVIEW-CORRECTIONS.json').read_bytes());change=correction['reports'][part];report_hash=H((args.reports_root/('ASSESSMENT-CENTRAL-FOLLOWUPS-'+part.upper()+'-95E.json')).read_bytes());assert H(draft)==change['source_draft_sha256']
 if part=='15c':
  spot=json.loads((P.parent/'SPOTCHECK-CORRECTION.json').read_bytes());assert spot['previous_sha256']==change['corrected_sha256'] and report_hash==spot['corrected_sha256']
 else:assert report_hash==change['corrected_sha256']
 for patch in change['draft_patches']:
  tokens=patch['pointer'].strip('/').split('/');parent=d
  for token in tokens[:-1]:parent=parent[int(token)] if isinstance(parent,list) else parent[token]
  key=tokens[-1];assert (key in parent)==patch['old_present']
  if patch['old_present']:assert parent[key]==patch['old']
  parent[key]=patch['new']
 if part=='15c':
  for patch in spot['draft_patches']:
   tokens=patch['pointer'].strip('/').split('/');parent=d
   for token in tokens[:-1]:parent=parent[int(token)] if isinstance(parent,list) else parent[token]
   key=tokens[-1];assert (key in parent)==patch['old_present']
   if patch['old_present']:assert parent[key]==patch['old']
   parent[key]=patch['new']
 for k in ('assessments','ledger_relationships','source_revision','parent_published_checkpoint','recovery_qualification','repair','product_validation'):assert final[k]==d[k]
 assert final['repair']=='not started' and final['product_validation']=='not performed'
print('Verified exact central evidence archive, registered source/display/ledger identities and corrected report links; historical attribution, native behavior, semantic truth and complete privacy remain separate.')
