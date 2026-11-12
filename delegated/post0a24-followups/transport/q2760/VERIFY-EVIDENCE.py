#!/usr/bin/env python3
"""Read-only source/evidence integrity verification. Never invokes product code."""
import argparse,datetime,gzip,hashlib,io,json,pathlib,subprocess,tarfile
P=pathlib.Path(__file__).resolve().parent

def need(x,msg):
 if not x:raise SystemExit('FAIL: '+msg)
def sha(b):return hashlib.sha256(b).hexdigest()
def git_hash(kind,b):return hashlib.sha1((kind+' '+str(len(b))+'\0').encode()+b).hexdigest()
def time(s):return datetime.datetime.fromisoformat(s.replace('Z','+00:00'))
a=argparse.ArgumentParser();a.add_argument('--repo',help='Git repository exposing pinned source and 059 publication objects');args=a.parse_args()
outer=json.loads((P/'PUBLIC-MANIFEST.json').read_text())
for x in outer['files']:
 b=(P/x['path']).read_bytes();need(len(b)==x['bytes'] and sha(b)==x['sha256'],'outer identity '+x['path'])
files={}
with tarfile.open(fileobj=io.BytesIO(gzip.decompress((P/'q2760-evidence.tar.gz').read_bytes())),mode='r:') as tar:
 for m in tar.getmembers():
  need(m.isfile() and not m.name.startswith('/') and '..' not in pathlib.PurePosixPath(m.name).parts and m.name not in files,'member safety');files[m.name]=tar.extractfile(m).read()
def read(n):
 need(n in files,'member missing '+n);return files[n]
def j(n):return json.loads(read(n))
def ptr(d,p):
 for s in p.split('/')[1:]:d=d[int(s)] if isinstance(d,list) else d[s.replace('~1','/').replace('~0','~')]
 return d
m=j('PRIVATE-MANIFEST.json');expected={x['path'] for x in m['files']};need(set(files)==expected|{'PRIVATE-MANIFEST.json'},'member set')
for x in m['files']:
 b=read(x['path']);need(len(b)==x['bytes'] and sha(b)==x['sha256'],'member '+x['path'])
g=j('GIT-TREE-PROOFS.json')
for oid,row in g['tree_objects'].items():
 b=read(row['file']);need(len(b)==row['bytes'] and sha(b)==row['sha256'] and git_hash('tree',b)==oid,'tree '+oid)
def entries(b):
 i=0;out={}
 while i<len(b):
  z=b.find(b'\0',i);need(z>=0 and z+21<=len(b),'tree encoding');head=b[i:z];need(b' ' in head,'tree entry');mode,name=head.split(b' ',1);need(name not in out,'duplicate name');out[name]=(mode.decode(),b[z+1:z+21].hex());i=z+21
 return out
for x in g['proofs']:
 b=read(x['file']);need(sha(b)==x['sha256'] and git_hash('blob',b)==x['blob_oid'],'blob '+x['file']);cur=x['root_tree'];need([s['component'] for s in x['chain']]==x['path'].split('/'),'path chain')
 for s in x['chain']:
  need(s['tree_oid']==cur,'tree continuity');entry=entries(read(g['tree_objects'][cur]['file'])).get(s['component'].encode());need(entry is not None and int(entry[0],8)==int(s['entry_mode'],8) and entry[1]==s['child_oid'],'tree entry');cur=s['child_oid']
 need(cur==x['blob_oid'],'blob terminus')
dp=j('DISPLAY-PLAN.json');main=j('DISPLAY-RECEIPTS.json');need(main['registration']=='DISPLAY-PLAN.json' and len(main['items'])==7,'assigned/history receipt count')
for x in main['items']:
 b=read(x['file']);need(sha(b)==x['sha256'] and not x['truncated'] and time(dp['registered_at_utc'])<=time(x['displayed_at_utc']),'assigned/history display receipt')
 if 'range' in x:
  matches=[v for v in dp['source_windows'] if v['file']==x['source'] and v['range']==x['range']];need(len(matches)==1,'assigned registration');source=read(x['source']);need(sha(source)==matches[0]['sha256'],'assigned source');lo,hi=x['range'];lines=source.decode().splitlines();view=''.join(f'{i:5}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode();need(view==b,'assigned exact display')
 else:need(j(x['file'])==ptr(j(x['source']),x['pointer']),'history projected view')
rs=[n for n in files if n.startswith('VIEW-') and n.endswith('.receipt.json')];need(len(rs)==32,'support receipt count')
for n in rs:
 x=j(n);plan=j(x['registration']);matched=[v for v in plan['items'] if v['file']==x['source'] and x['range'] in v['display_ranges']];need(len(matched)==1,'support registration');src=read(x['source']);need(sha(src)==matched[0]['sha256'],'support source');lo,hi=x['range'];lines=src.decode().splitlines();view=''.join(f'{i:5}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode();need(read(x['view'])==view and sha(view)==x['sha256'] and not x['truncated'] and time(plan['registered_at_utc'])<=time(x['displayed_at_utc']),'support display '+n)
for x in j('HISTORY-EXTRA-VIEW-RECEIPT.json')['items']:
 need(sha(read(x['file']))==x['sha256'] and j(x['file'])==ptr(j(x['source']),x['pointer']) and not x['truncated'],'historical projection')
for x in j('CAPTURE.json')['history']:
 need(sha(read(x['file']))==x['sha256'],'historical capture')
for x in j('HISTORY-EXTRA-CAPTURE.json')['items']:
 need(sha(read(x['file']))==x['sha256'],'historical extra capture')
q=j('QUEUE-RAW.json');need(q['contract_id']=='OBLIGATION-ROOT-L3-EXT-0005-LENS-08','q2760 id');need(sha(read('HISTORY-0.json'))==q['dependencies'][0]['sha256'],'original registration identity');need(sha(read('HISTORY-1.json'))==q['exact_source_identities_and_assessed_ranges'][0]['discovering_lens_receipt']['sha256'],'old receipt identity');need(sha(read('HISTORY-2.json'))==q['exact_current_revision_revisit_requirements'][0]['current_registration']['sha256'],'e275 registration identity')
need(j('QUEUE-2703.json')['contract_id']=='REVIEW-L3-PREPARED-BUILDING-PUBLICATION-ADMISSION','q2703 identity');need(j('QUEUE-2357.json')['contract_id']=='L2-EXT-PREPARED-NATIVE-MESH-OWNER-FIT','q2357 identity')
for k,v in j('QUEUE-2357-VIEW.json').items():need(j('QUEUE-2357.json')[k]==v,'q2357 projection')
prior=j('PRIOR-Q2357-PRIVATE-MANIFEST.json')['files']
for old,new in [('REPORT.json','PRIOR-Q2357-REPORT.json'),('REPORT.md','PRIOR-Q2357-REPORT.md'),('LEDGER-ROW-266.json','PRIOR-Q2357-LEDGER-ROW-266.json')]:need(sha(read(new))==prior[old]['sha256'],'prior manifest '+old)
need(j('PRIOR-Q2357-OBS.json')==j('PRIOR-Q2357-REPORT.json')['observations'][0],'prior observation')
limits=j('READ-LIMITS.json');assigned=[x for x in limits['sources'] if x['role']=='assigned Lens 8'];need(len(assigned)==3 and all(x['display_ranges']==[[1,54]] and x['unread_ranges']==[] for x in assigned),'assigned coverage')
repo_bound=False
if args.repo:
 def git(*a):return subprocess.check_output(['git','-C',args.repo,*a])
 for x in g['proofs']:need(git('rev-parse',x['revision']+'^{tree}').decode().strip()==x['root_tree'],'commit tree '+x['revision'])
 old=git('show','56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e:baseline-059/queue.json');pub=git('show','e709f11f0aa6bf736bd20819a628e4790cd03999:baseline-059/queue.json');co=j('QUEUE-COMPARE.json');need(sha(old)==co['original_sha256'] and sha(pub)==co['public_sha256'],'queue whole identities');o=json.loads(old)['entries'];u=json.loads(pub)['entries']
 for i,name in [(2357,'QUEUE-2357.json'),(2703,'QUEUE-2703.json'),(2760,'QUEUE-RAW.json')]:need(o[i]==u[i]==j(name),'selected queue '+str(i))
 repo_bound=True
print(json.dumps({'result':'PASS','archive_members':len(files),'assigned_revision_displays':3,'support_displays':len(rs),'tree_proofs':len(g['proofs']),'tree_objects':len(g['tree_objects']),'git_commit_and_queue_binding':repo_bound,'qualification':'Evidence integrity only: no product execution, visual validation, whole-tree behavior claim, foreign read credit, 3821 promotion, alias or queue/lens closure. Without --repo commit-to-root and 059 publication binding are not checked.'},indent=2))
