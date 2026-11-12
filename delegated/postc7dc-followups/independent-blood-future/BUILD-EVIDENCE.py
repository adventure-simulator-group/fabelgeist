#!/usr/bin/env python3
"""Construct source and historical evidence receipts, and raw Git-tree chains."""
import datetime,hashlib,json,pathlib,subprocess,sys
P=pathlib.Path(__file__).resolve().parent; REPO=pathlib.Path(sys.argv[1]); REV='95e63628f9d70b8dd6e6fede64334696c6995009'; OLD='6533e444fa37552927070cc8d8190907fd3b413e'
def git(*a):return subprocess.check_output(['git','-C',str(REPO),*a])
def sha(d):return hashlib.sha256(d).hexdigest()
def read(n):return json.loads((P/n).read_text())
def write(n,v):(P/n).write_text(json.dumps(v,indent=2,ensure_ascii=False)+'\n')
regnames=['SOURCE-REGISTRATION.json','SURGERY-EXT-REGISTRATION.json','TIME-REGISTRATION.json','CALENDAR-EXT-REGISTRATION.json']; rows=[]; display=[]
for name in regnames:
 reg=read(name)
 for e in reg['sources']:
  d=(P/e['capture']).read_bytes(); assert sha(d)==e['sha256']; lines=d.decode().splitlines(); assert len(lines)==e['eof']; actual=e['planned_ranges']
  for a,b in actual:
   assert 1<=a<=b<=len(lines); display.append(f'--- {name} {e["path"]} {e["revision"]} {a}-{b} ---\n'); display.extend(f'{i:6d}\t{lines[i-1]}\n' for i in range(a,b+1))
  rows.append({'registration':name,'registration_sha256':sha((P/name).read_bytes()),'path':e['path'],'revision':e['revision'],'sha256':e['sha256'],'git_blob':e['git_blob'],'capture':e['capture'],'eof':e['eof'],'planned_ranges':e['planned_ranges'],'actual_display_chunks':actual,'status':'personally displayed, untruncated as observed'})
(P/'DISPLAY-RECONSTRUCTED.txt').write_text(''.join(display))
def ranges(indices):
 seq=sorted(indices); out=[]
 if not seq:return out
 a=b=seq[0]
 for n in seq[1:]:
  if n==b+1:b=n
  else:out.append([a,b]); a=b=n
 out.append([a,b]);return out
coverage={}
for r in rows:
 key=(r['path'],r['revision'],r['eof']); coverage.setdefault(key,set())
 for a,b in r['actual_display_chunks']:coverage[key].update(range(a,b+1))
union=[{'path':path,'revision':rev,'eof':eof,'actually_displayed':ranges(got),'unread_complement':ranges(set(range(1,eof+1))-got)} for (path,rev,eof),got in sorted(coverage.items())]
write('READ-RECEIPTS.json',{'reader':'/root/mhr_character','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'qualification':'Reconstructed exact numbered source lines after registered personal displays; transcripts are not independent proof of perception. Full-file storage is identity evidence, not whole-body read credit. Prior q2297 readers remain separate.','rows':rows,'same_reader_union':union})
write('HISTORY-READ-RECEIPTS.json',{'reader':'/root/mhr_character','registration':'HISTORY-REGISTRATION.json','registration_sha256':sha((P/'HISTORY-REGISTRATION.json').read_bytes()),'selected_values':'HISTORY-SELECTED-RECONSTRUCTED.json','selected_sha256':sha((P/'HISTORY-SELECTED-RECONSTRUCTED.json').read_bytes()),'actual_display':'All registered selected historic values personally displayed in two untruncated chunks, 1–130 and 131–255. Four original documents retained exactly but not claimed fully personally read.','qualification':'Original 6533-qualified pr-working Lens13 reviewer and later root adoption without reread retain their attribution; no 3821 credit.'})
write('OTHER-READ-RECEIPTS.json',{'reader':'/root/mhr_character','entries':[{'registration':'INITIAL-REGISTRATION.json','capture':'QUEUE-SELECTED-RECONSTRUCTED.json','display':'whole selected historical q2297 and q894 values, no original body or 3821 credit'}, {'registration':'PRIOR-REGISTRATION.json','display':'complete earlier q2297 report and staged working 13V assessment; q2297 manifest identity only, no full manifest display credit'}], 'qualification':'Staged working 13V and frozen prior packet are distinct from committed product-source revision. Metadata-only locator truncation is recorded separately.'})
treed=P/'git-trees';treed.mkdir(exist_ok=True)
def tree_entries(raw):
 i=0;out={}
 while i<len(raw):
  sp=raw.index(b' ',i);z=raw.index(b'\0',sp+1);out[raw[sp+1:z].decode()]=(raw[i:sp].decode(),raw[z+1:z+21].hex());i=z+21
 return out
def proof(rev,path):
 root=git('rev-parse',rev+'^{tree}').decode().strip();t=root;chain=[];parts=path.split('/')
 for i,part in enumerate(parts):
  raw=git('cat-file','tree',t);assert hashlib.sha1(b'tree '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==t;(treed/(t+'.tree')).write_bytes(raw);mode,oid=tree_entries(raw)[part];chain.append({'tree':t,'name':part,'mode':mode,'object':oid})
  if i<len(parts)-1:assert mode=='40000';t=oid
 d=git('show',rev+':'+path);assert hashlib.sha1(b'blob '+str(len(d)).encode()+b'\0'+d).hexdigest()==chain[-1]['object'];return {'revision':rev,'root_tree':root,'path':path,'chain':chain,'blob':chain[-1]['object'],'source_sha256':sha(d),'source_bytes':len(d)}
proofs=[proof(REV,path) for path in sorted({r['path'] for r in rows})];proofs.append(proof(OLD,'crates/adventuresim-stdb-module/src/filth/blood_attempts.rs'))
write('GIT-TREE-PROOFS.json',{'qualification':'Raw per-path tree membership; optional repository check binds roots to commits. Commit bodies and author metadata omitted. Same blood_attempts bytes are per-file evidence, not whole-tree equivalence or historical read-credit transfer.','proofs':proofs})
print(json.dumps({'receipt_rows':len(rows),'tree_proofs':len(proofs),'tree_objects':len(list(treed.glob('*.tree'))),'coverage':union},indent=2))
