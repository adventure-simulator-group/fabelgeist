#!/usr/bin/env python3
"""Read-only evidence integrity verifier; optional Git commit binding; no product execution."""
import argparse,datetime,gzip,hashlib,io,json,pathlib,subprocess,tarfile
P=pathlib.Path(__file__).resolve().parent

def require(value,message):
    if not value: raise SystemExit('FAIL: '+message)
def sha(b):return hashlib.sha256(b).hexdigest()
def gh(kind,b):return hashlib.sha1((kind+' '+str(len(b))+'\0').encode()+b).hexdigest()
def tim(s):return datetime.datetime.fromisoformat(s.replace('Z','+00:00'))
A=argparse.ArgumentParser();A.add_argument('--repo',help='read-only Git repository with source and publication commits');args=A.parse_args()
outer=json.loads((P/'PUBLIC-MANIFEST.json').read_text())
for x in outer['files']:
 b=(P/x['path']).read_bytes();require(len(b)==x['bytes'] and sha(b)==x['sha256'],'outer '+x['path'])
raw=(P/'q2708-evidence.tar.gz').read_bytes();data=gzip.decompress(raw)
files={}
with tarfile.open(fileobj=io.BytesIO(data),mode='r:') as tar:
 for m in tar.getmembers():
  require(m.isfile() and not m.name.startswith('/') and '..' not in pathlib.PurePosixPath(m.name).parts and m.name not in files,'archive member safety')
  files[m.name]=tar.extractfile(m).read()
def read(name):
 require(name in files,'missing '+name);return files[name]
def obj(name):return json.loads(read(name))
manifest=obj('PRIVATE-MANIFEST.json'); expected={r['path'] for r in manifest['files']};require(set(files)==expected|{'PRIVATE-MANIFEST.json'},'complete archived member inventory')
for r in manifest['files']:
 b=read(r['path']);require(len(b)==r['bytes'] and sha(b)==r['sha256'],'member '+r['path'])
proof=obj('GIT-TREE-PROOFS.json')
for oid,item in proof['tree_objects'].items():
 b=read(item['file']);require(len(b)==item['bytes'] and sha(b)==item['sha256'] and gh('tree',b)==oid,'tree object '+oid)
def tree_entries(b):
 i=0;out={}
 while i<len(b):
  j=b.find(b'\0',i);require(j>=0 and j+21<=len(b),'tree encoding');prefix=b[i:j];require(b' ' in prefix,'tree mode/name')
  mode,name=prefix.split(b' ',1);require(name not in out,'duplicate tree name');out[name]=(mode.decode(),b[j+1:j+21].hex());i=j+21
 return out
for r in proof['proofs']:
 b=read(r['file']);require(sha(b)==r['sha256'] and gh('blob',b)==r['blob_oid'],'source blob '+r['file']);current=r['root_tree']
 require([x['component'] for x in r['chain']]==r['path'].split('/'),'path chain '+r['path'])
 for c in r['chain']:
  require(c['tree_oid']==current,'chain continuity');entry=tree_entries(read(proof['tree_objects'][current]['file'])).get(c['component'].encode());require(entry is not None and int(entry[0],8)==int(c['entry_mode'],8) and entry[1]==c['child_oid'],'tree entry '+c['component']);current=c['child_oid']
 require(current==r['blob_oid'],'chain terminates')
receipt_names=[n for n in files if n.startswith('DISPLAY-V2-') and n.endswith('.receipt.json')]
require(len(receipt_names)==34,'registered source display count')
for n in receipt_names:
 r=obj(n);plan=obj(r['registration']);items=[v for v in plan['items'] if v['file']==r['source'] and r['range'] in v['display_ranges']];require(len(items)==1,'window registration '+n)
 src=read(r['source']);require(sha(src)==items[0]['sha256'],'registered source '+n); lines=src.decode().splitlines();lo,hi=r['range'];require(1<=lo<=hi<=len(lines),'range '+n)
 out=''.join(f'{i:6}\t{lines[i-1]}\n' for i in range(lo,hi+1)).encode();require(read(r['display'])==out and sha(out)==r['view_sha256'] and not r['truncated'],'exact display '+n);require(tim(plan['registered_at_utc'])<=tim(r['displayed_at_utc']),'prospective source registration '+n)
limits=obj('READ-LIMITS.json'); item=next(x for x in limits['sources'] if x['file']=='ERRANTRY-CURRENT.rs');covered=set()
for lo,hi in item['display_ranges']:covered.update(range(lo,hi+1))
require(covered==set(range(1,828)) and item['unread_ranges']==[],'full current errantry scope'); require(all(next(x for x in limits['sources'] if x['file']==n)['display_ranges']==[] for n in ['ERRANTRY-OLD.rs','ERRANTRY-ADOPTED.rs']),'historical display qualification')
for n in ['HISTORY-CAPTURE.json','Q2697-HISTORY-CAPTURE.json']:
 for row in obj(n)['rows']:
  require(sha(read(row['file']))==row['sha256']==row['expected_sha256'],'historical capture '+row['file'])
for n in ['QUEUE-VIEW-RECEIPT.json','HISTORY-VIEW-RECEIPT.json','HISTORY-CURRENT-VIEW-RECEIPT.json','HISTORIC-Q2697-VIEW-RECEIPT.json','Q2697-IDENTITY-RECEIPT.json','ASSESSMENT-VIEW-RECEIPT.json','PREDECESSOR-ROW-RECEIPT.json']:
 r=obj(n);plan=obj(r['registration']); require(tim(plan['registered_at_utc'])<=tim(r['display_completed_before_utc']),'historical view receipt chronology '+n)
 for it in r.get('items',[{'file':r.get('file'),'sha256':r.get('sha256'),'truncated':r.get('truncated')}]):
  require(it['file'] is not None and sha(read(it['file']))==it['sha256'] and not it.get('truncated',False),'view receipt '+n)
require(obj('HISTORY-0-ROW-0.json')==obj('HISTORY-0.json')['rows'][0],'original registration row')
require(obj('HISTORY-1-ROW-0.json')==obj('HISTORY-1.json')['rows'][0],'original reviewer receipt row')
require(obj('HISTORY-2-ROW-0.json')==obj('HISTORY-2.json')['rows'][0],'current registration row')
require(obj('HISTORY-CURRENT-ROW-0.json')==obj('Q2697-HISTORY-3.json')['rows'][0],'current reviewer receipt row')
q=obj('QUEUE-RAW.json');require(q['contract_id']=='OBLIGATION-ROOT-L3-EXT-0001-LENS-04','q2708 id');q2697=obj('HISTORIC-Q2697-RAW.json');require(q2697['contract_id']=='REVIEW-L3-ORDERED-CLUE-POSITION-ADMISSION','q2697 id')
gz=read('PREDECESSOR-BASELINE.json.gz'); nav=obj('PREDECESSOR-NAV.json');require(sha(gz)==nav['stored_sha256'] and sha(gzip.decompress(gz))==nav['decoded_sha256'],'predecessor stored/decoded');require(json.loads(gzip.decompress(gz))['repair_ledger']['entries'][37]==obj('PREDECESSOR-ROW.json'),'predecessor selected row')
root_binding=False
if args.repo:
 def gitshow(commit,path):return subprocess.check_output(['git','-C',args.repo,'show',commit+':'+path])
 def git(*a):return subprocess.check_output(['git','-C',args.repo,*a]).decode().strip()
 for r in proof['proofs']: require(git('rev-parse',r['revision']+'^{tree}')==r['root_tree'],'revision root '+r['revision'])
 old=gitshow('56f24f6457f6cec8ffcdc46f03f5ff26a8abb50e','baseline-059/queue.json');pub=gitshow('e709f11f0aa6bf736bd20819a628e4790cd03999','baseline-059/queue.json');capture=obj('QUEUE-CAPTURE.json')
 require(sha(old)==capture['original_queue_sha256'] and sha(pub)==capture['public_queue_sha256'],'distinct queue full identity');oldj=json.loads(old)['entries'];pubj=json.loads(pub)['entries'];require(oldj[2708]==pubj[2708]==q and oldj[2697]==pubj[2697]==q2697,'selected queue equality')
 require(gitshow('31e130bfa8f8914f759c7d5cddc3db92962704c7','ASSESSMENT-L1-SIGIL-POSITIONS.json')==read('ASSESSMENT-0.json'),'prior assessment committed binding')
 require(gitshow('31e130bfa8f8914f759c7d5cddc3db92962704c7','main-advance-95e/predecessor-baseline.json.gz')==gz,'prior baseline committed binding')
 prior=obj('PRIOR-C100-COMPARISON.json');require(git('rev-parse',prior['revision']+':'+prior['path'])==prior['blob_oid'],'c100 errantry Git object')
 root_binding=True
print(json.dumps({'result':'PASS','archive_members':len(files),'source_display_receipts':len(receipt_names),'tree_proofs':len(proof['proofs']),'tree_objects':len(proof['tree_objects']),'revision_root_and_published_queue_binding':root_binding,'qualification':'Source and evidence integrity only; no proof of product behavior, full unregistered reads, global absence, historical cross-lens credit or queue closure. Without --repo, raw tree chains are internally verified but commit root and published evidence anchors are not.'},indent=2))
