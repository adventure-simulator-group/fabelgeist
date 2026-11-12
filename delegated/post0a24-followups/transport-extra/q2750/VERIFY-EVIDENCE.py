#!/usr/bin/env python3
"""Read-only q2750 evidence verification; no product/dependency execution."""
import argparse,datetime,difflib,gzip,hashlib,io,json,pathlib,subprocess,tarfile,tomllib
P=pathlib.Path(__file__).resolve().parent
def need(x,m):
 if not x:raise SystemExit('FAIL: '+m)
def sha(b):return hashlib.sha256(b).hexdigest()
def gh(kind,b):return hashlib.sha1((kind+' '+str(len(b))+'\0').encode()+b).hexdigest()
def tm(s):return datetime.datetime.fromisoformat(s.replace('Z','+00:00'))
a=argparse.ArgumentParser();a.add_argument('--repo',help='read-only Git repo exposing source and audit commits');args=a.parse_args()
o=json.loads((P/'PUBLIC-MANIFEST.json').read_text())
for x in o['files']:
 b=(P/x['path']).read_bytes();need(len(b)==x['bytes'] and sha(b)==x['sha256'],'outer '+x['path'])
files={}
with tarfile.open(fileobj=io.BytesIO(gzip.decompress((P/'q2750-evidence.tar.gz').read_bytes())),mode='r:') as tar:
 for m in tar.getmembers():
  need(m.isfile() and not m.name.startswith('/') and '..' not in pathlib.PurePosixPath(m.name).parts and m.name not in files,'archive safety');files[m.name]=tar.extractfile(m).read()
def read(n):need(n in files,'missing '+n);return files[n]
def j(n):return json.loads(read(n))
def ptr(d,path):
 for s in path.split('/')[1:]:d=d[int(s)] if isinstance(d,list) else d[s.replace('~1','/').replace('~0','~')]
 return d
m=j('PRIVATE-MANIFEST.json');expect={x['path'] for x in m['files']};need(set(files)==expect|{'PRIVATE-MANIFEST.json'},'complete member inventory')
for x in m['files']:
 b=read(x['path']);need(len(b)==x['bytes'] and sha(b)==x['sha256'],'member '+x['path'])
g=j('GIT-TREE-PROOFS.json')
for oid,x in g['tree_objects'].items():
 b=read(x['file']);need(len(b)==x['bytes'] and sha(b)==x['sha256'] and gh('tree',b)==oid,'tree '+oid)
def entries(b):
 i=0;out={}
 while i<len(b):
  z=b.find(b'\0',i);need(z>=0 and z+21<=len(b),'tree encoding');head=b[i:z];need(b' ' in head,'tree entry');mode,name=head.split(b' ',1);need(name not in out,'duplicate name');out[name]=(mode.decode(),b[z+1:z+21].hex());i=z+21
 return out
for x in g['proofs']:
 cur=x['root_tree'];parts=x['path'].split('/');need([z['component'] for z in x['chain']]==parts[:len(x['chain'])],'path chain')
 for z in x['chain']:
  need(z['tree_oid']==cur,'continuity');e=entries(read(g['tree_objects'][cur]['file'])).get(z['component'].encode());need(e is not None and int(e[0],8)==int(z['entry_mode'],8) and e[1]==z['child_oid'],'tree entry');cur=z['child_oid']
 if 'missing' in x:
  z=x['missing'];need(cur==z['tree_oid'] and len(x['chain'])<len(parts) and z['component']==parts[len(x['chain'])] and z['component'].encode() not in entries(read(g['tree_objects'][cur]['file'])),'path absence')
 else:
  b=read(x['file']);need(cur==x['blob_oid'] and sha(b)==x['sha256'] and gh('blob',b)==cur and len(x['chain'])==len(parts),'source blob')
plan=j('DISPLAY-PLAN.json');rs=j('DISPLAY-RECEIPTS.json');need(len(rs['items'])==7,'assigned/history receipt count')
for x in rs['items']:
 b=read(x['file']);need(sha(b)==x['sha256'] and not x['truncated'] and tm(plan['registered_at_utc'])<=tm(x['displayed_at_utc']),'assigned receipt')
 if 'range' in x:
  opt=[v for v in plan['sources'] if v['file']==x['source'] and v['range']==x['range']];need(len(opt)==1 and sha(read(x['source']))==opt[0]['sha256'],'assigned registration');lo,hi=x['range'];lines=read(x['source']).decode().splitlines();v=''.join(f'{i:5}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode();need(b==v,'assigned display')
 else:need(j(x['file'])==ptr(j(x['source']),x['pointer']),'historical projection')
rs=[n for n in files if n.startswith('VIEW-') and n.endswith('.receipt.json')];need(len(rs)==14,'support receipt count')
for n in rs:
 x=j(n);r=j(x['registration']);items=[v for v in r['items'] if v['file']==x['source'] and x['range'] in v['display_ranges']];need(len(items)==1 and sha(read(x['source']))==items[0]['sha256'],'support registration');lo,hi=x['range'];lines=read(x['source']).decode().splitlines();need(1<=lo<=hi<=len(lines),'support bounds');b=''.join(f'{i:5}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode();need(read(x['view'])==b and sha(b)==x['sha256'] and not x['truncated'] and tm(r['registered_at_utc'])<=tm(x['displayed_at_utc']),'support view '+n)
old=read('MATERIALS-6533e4.rs').decode().splitlines(keepends=True);new=read('MATERIALS-95e636.rs').decode().splitlines(keepends=True);diff=''.join(difflib.unified_diff(old,new,fromfile='6533/materials.rs',tofile='95e/materials.rs')).encode();need(read('SOURCE-DIFF.patch')==diff,'source delta')
q=j('QUEUE-RAW.json');need(q['contract_id']=='OBLIGATION-ROOT-L3-EXT-0004-LENS-10','q2750 id');need(sha(read('HISTORY-0.json'))==q['dependencies'][0]['sha256'],'old registration');need(sha(read('HISTORY-1.json'))==q['exact_source_identities_and_assessed_ranges'][0]['discovering_lens_receipt']['sha256'],'old receipt');need(sha(read('HISTORY-2.json'))==q['exact_current_revision_revisit_requirements'][0]['current_registration']['sha256'],'e275 registration')
for key,v in j('QUEUE-49-VIEW.json').items():need(j('QUEUE-49.json')[key]==v,'q49 selected projection')
need(j('QUEUE-2701.json')['contract_id']=='REVIEW-L3-TEXTURE-RECIPE-CHANNEL-INSTALL-ADMISSION','q2701');need(j('LEDGER-201.json')==j('WORKING-0.json')['newly_assessed_ledger']['entries'][201],'ledger 201')
for n in ['WORKING-2.json','HEAD-2.json','INDEX-2.json']:
 x=j(n)['entries'][2750];need(x['contract_id']=='OBLIGATION-ROOT-L3-EXT-0004-LENS-10' and x['current_reconciliation']=='unassessed','captured candidate status')
for i in range(4):need(sha(read('WORKING-'+str(i)+'.json'))==sha(read('HEAD-'+str(i)+'.json'))==sha(read('INDEX-'+str(i)+'.json')),'captured metadata equality')
locks=[tomllib.loads(read(n).decode()) for n in ['CARGO-LOCK-6533','CARGO-LOCK-95E']];native=j('NATIVE-CAPTURE.json')
for x in native['rows']:
 b=read('NATIVE-'+x['name']+'.crate');need(sha(b)==x['archive_sha256'] and x['checksum_match'],'archive identity')
 for lock in locks:
  pkgs=[p for p in lock['package'] if p['name']==x['name'] and p['version']==x['version']];need(len(pkgs)==1 and pkgs[0]['checksum']==sha(b),'lock checksum')
 with tarfile.open(fileobj=io.BytesIO(b),mode='r:gz') as tar:
  need(tar.extractfile(x['member']).read()==read(x['file']),'package member')
  if x['name']=='bevy_app':need(tar.extractfile('bevy_app-0.19.1/src/app.rs').read()==read('NATIVE-bevy_app-main.rs'),'native app member')
bound=False
if args.repo:
 def git(*v):return subprocess.check_output(['git','-C',args.repo,*v])
 for x in g['proofs']:need(git('rev-parse',x['revision']+'^{tree}').decode().strip()==x['root_tree'],'commit tree')
 old=git('show','56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e:baseline-059/queue.json');pub=git('show','e709f11f0aa6bf736bd20819a628e4790cd03999:baseline-059/queue.json');c=j('QUEUE-COMPARE.json');need(sha(old)==c['original_sha256'] and sha(pub)==c['public_sha256'],'queue whole identity');a1=json.loads(old)['entries'];b1=json.loads(pub)['entries']
 for i,n in [(49,'QUEUE-49.json'),(2701,'QUEUE-2701.json'),(2750,'QUEUE-RAW.json')]:need(a1[i]==b1[i]==j(n),'queue selected')
 meta=j('HEAD-INDEX-RESULT.json');head=meta['head_commit']
 for x in meta['rows']:
  if x['representation']=='HEAD' and x['exit_code']==0:need(git('show',head+':'+x['file'])==read(x['capture']),'captured HEAD metadata')
 bound=True
print(json.dumps({'result':'PASS','archive_members':len(files),'assigned_revision_displays':3,'support_display_receipts':len(rs),'git_path_proofs':len(g['proofs']),'absent_paths':sum('missing' in x for x in g['proofs']),'raw_tree_objects':len(g['tree_objects']),'bevy_archive_lock_checks':4,'git_commit_queue_and_ledger_binding':bound,'qualification':'Evidence integrity only; no product behavior, runtime, full unregistered reads, feature selection, cross-lens credit, 3821 promotion, duplicate count or queue/lens closure. Without --repo, Git commit bindings are not checked.'},indent=2))
