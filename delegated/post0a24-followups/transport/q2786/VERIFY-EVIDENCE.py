#!/usr/bin/env python3
"""Read-only evidence verification, with optional Git root and publication binding."""
import argparse,datetime,gzip,hashlib,io,json,pathlib,subprocess,tarfile
P=pathlib.Path(__file__).resolve().parent
def need(x,m):
 if not x:raise SystemExit('FAIL: '+m)
def sha(b):return hashlib.sha256(b).hexdigest()
def gh(kind,b):return hashlib.sha1((kind+' '+str(len(b))+'\0').encode()+b).hexdigest()
def tm(s):return datetime.datetime.fromisoformat(s.replace('Z','+00:00'))
a=argparse.ArgumentParser();a.add_argument('--repo',help='read-only Git object repository');args=a.parse_args()
o=json.loads((P/'PUBLIC-MANIFEST.json').read_text())
for r in o['files']:
 b=(P/r['path']).read_bytes();need(len(b)==r['bytes'] and sha(b)==r['sha256'],'outer '+r['path'])
files={}
with tarfile.open(fileobj=io.BytesIO(gzip.decompress((P/'q2786-evidence.tar.gz').read_bytes())),mode='r:') as tar:
 for m in tar.getmembers():
  need(m.isfile() and not m.name.startswith('/') and '..' not in pathlib.PurePosixPath(m.name).parts and m.name not in files,'archive safety');files[m.name]=tar.extractfile(m).read()
def read(n):need(n in files,'missing '+n);return files[n]
def j(n):return json.loads(read(n))
def pointer(d,path):
 for s in path.split('/')[1:]:d=d[int(s)] if isinstance(d,list) else d[s.replace('~1','/').replace('~0','~')]
 return d
m=j('PRIVATE-MANIFEST.json');expected={r['path'] for r in m['files']};need(set(files)==expected|{'PRIVATE-MANIFEST.json'},'archive inventory')
for r in m['files']:
 b=read(r['path']);need(len(b)==r['bytes'] and sha(b)==r['sha256'],'member '+r['path'])
g=j('GIT-TREE-PROOFS.json')
for oid,r in g['tree_objects'].items():
 b=read(r['file']);need(len(b)==r['bytes'] and sha(b)==r['sha256'] and gh('tree',b)==oid,'tree '+oid)
def entries(b):
 i=0;out={}
 while i<len(b):
  k=b.find(b'\0',i);need(k>=0 and k+21<=len(b),'tree encoding');head=b[i:k];need(b' ' in head,'tree entry');mode,name=head.split(b' ',1);need(name not in out,'duplicate name');out[name]=(mode.decode(),b[k+1:k+21].hex());i=k+21
 return out
for r in g['proofs']:
 b=read(r['file']);need(sha(b)==r['sha256'] and gh('blob',b)==r['blob_oid'],'blob '+r['file']);cur=r['root_tree'];need([x['component'] for x in r['chain']]==r['path'].split('/'),'path chain')
 for x in r['chain']:
  need(x['tree_oid']==cur,'tree continuity');ent=entries(read(g['tree_objects'][cur]['file'])).get(x['component'].encode());need(ent is not None and int(ent[0],8)==int(x['entry_mode'],8) and ent[1]==x['child_oid'],'tree entry');cur=x['child_oid']
 need(cur==r['blob_oid'],'blob terminus')
receipts=[n for n in files if n.startswith('VIEW-') and n.endswith('.receipt.json')];need(len(receipts)==19,'display receipt count')
for n in receipts:
 r=j(n);plan=j(r['registration']);matches=[x for x in plan['items'] if x['file']==r['source'] and r['range'] in x['display_ranges']];need(len(matches)==1,'registered '+n);src=read(r['source']);need(sha(src)==matches[0]['sha256'],'source '+n);lo,hi=r['range'];lines=src.decode().splitlines();need(1<=lo<=hi<=len(lines),'bounds');b=''.join(f'{i:5}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode();need(read(r['view'])==b and sha(b)==r['sha256'] and not r['truncated'] and tm(plan['registered_at_utc'])<=tm(r['displayed_at_utc']),'display '+n)
for rn in ['HISTORY-VIEW-RECEIPT.json','HISTORY-EXTRA-VIEW-RECEIPT.json']:
 r=j(rn);plan=j(r['registration'])
 for x in r['items']:
  need(sha(read(x['file']))==x['sha256'] and j(x['file'])==pointer(j(x['source']),x['pointer']) and not x['truncated'] and tm(plan['registered_at_utc'])<=tm(x['displayed_at_utc']),'history view')
for rn in ['RELATED-DISPLAY-RECEIPT.json','EOF-DISPLAY-RECEIPT.json']:
 r=j(rn);plan=j(r['registration'])
 for x in r['items']:need(sha(read(x['file']))==x['sha256'] and not x['truncated'] and tm(plan['registered_at_utc'])<=tm(x['displayed_at_utc']),'record receipt')
need(j('EOF-ROW.json')==j('EOF-0.json')['files'][1622],'original registry selected row');need(j('EOF-OLD-REG-VIEW.json')==j('EOF-1.json'),'superseded registration');reg=j('HISTORY-EXTRA-0.json')['inherited_scope_extent_qualification'];need(sha(read('EOF-0.json'))==reg['registry_sha256'] and sha(read('EOF-1.json'))==reg['superseded_registration_sha256'],'original registry/superseded hashes');need(reg['actual_EOF']==231 and reg['excluded_nonexistent_ranges']==[[232,240]],'EOF qualification')
q=j('QUEUE-RAW.json');need(q['contract_id']=='OBLIGATION-ROOT-L3-EXT-0007-LENS-10','q2786 id');need(sha(read('HISTORY-0.json'))==q['dependencies'][0]['sha256'],'original registration');need(sha(read('HISTORY-1.json'))==q['exact_source_identities_and_assessed_ranges'][0]['discovering_lens_receipt']['sha256'],'Lens3 receipt');need(sha(read('HISTORY-2.json'))==q['exact_current_revision_revisit_requirements'][0]['current_registration']['sha256'],'e275 registration');need(j('QUEUE-2704.json')['contract_id']=='REVIEW-L3-REGIONAL-SURFACE-ADMISSION','q2704');need(j('QUEUE-2358.json')['contract_id']=='L2-EXT-REGIONAL-SURFACE-OWNERS','q2358')
for k,v in j('QUEUE-2358-VIEW.json').items():need(j('QUEUE-2358.json')[k]==v,'q2358 projection')
need(j('PRIOR-Q2358-OBS.json')==j('PRIOR-Q2358-REPORT.json')['observations'][1],'prior observation');prior=j('PRIOR-Q2358-PRIVATE-MANIFEST.json')['files']
for old,new in [('REPORT.json','PRIOR-Q2358-REPORT.json'),('REPORT.md','PRIOR-Q2358-REPORT.md'),('LEDGER-ROW-266.json','LEDGER-266.json')]:need(sha(read(new))==prior[old]['sha256'],'prior manifest '+old)
assessment=j('SURFACE-ASSESSMENT.json');view=j('SURFACE-ASSESSMENT-VIEW.json');need(all(assessment[k]==v for k,v in view.items()),'assessment projection')
limits=j('READ-LIMITS.json');assigned=[x for x in limits['sources'] if x['role']=='assigned Lens10'];need(len(assigned)==3 and all(x['display_ranges']==[[1,80],[81,160],[161,231]] and x['unread_ranges']==[] for x in assigned),'assigned full range')
bound=False
if args.repo:
 def git(*v):return subprocess.check_output(['git','-C',args.repo,*v])
 for r in g['proofs']:need(git('rev-parse',r['revision']+'^{tree}').decode().strip()==r['root_tree'],'commit root')
 old=git('show','56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e:baseline-059/queue.json');pub=git('show','e709f11f0aa6bf736bd20819a628e4790cd03999:baseline-059/queue.json');c=j('QUEUE-COMPARE.json');need(sha(old)==c['original_sha256'] and sha(pub)==c['public_sha256'],'queue whole identity');x=json.loads(old)['entries'];y=json.loads(pub)['entries']
 for i,name in [(2358,'QUEUE-2358.json'),(2704,'QUEUE-2704.json'),(2786,'QUEUE-RAW.json')]:need(x[i]==y[i]==j(name),'queue selected object')
 need(git('show','31e130bfa8f8914f759c7d5cddc3db92962704c7:ASSESSMENT-REGIONAL-MAP-SURFACE-95E.json')==read('SURFACE-ASSESSMENT.json'),'assessment commit');bound=True
print(json.dumps({'result':'PASS','archive_members':len(files),'source_display_receipts':len(receipts),'git_path_proofs':len(g['proofs']),'raw_tree_objects':len(g['tree_objects']),'git_commit_and_queue_binding':bound,'qualification':'Evidence integrity only; no product behavior, unregistered historical reads, cross-lens credit, 3821 promotion or queue/lens closure inferred. Without --repo, Git commit root and publication binding remain unverified.'},indent=2))
