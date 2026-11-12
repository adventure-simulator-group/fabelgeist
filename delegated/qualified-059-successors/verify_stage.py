#!/usr/bin/env python3
"""Read-only verification of the bounded 059 successor stage."""
from pathlib import Path
import argparse,ast,gzip,hashlib,json,os,stat
P=Path(__file__).absolute().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def blob(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def no_symlink(path):
 p=Path(os.path.abspath(os.fspath(path))); cur=Path(p.anchor)
 for x in p.parts[1:]:
  cur=cur/x
  if stat.S_ISLNK(os.lstat(cur).st_mode):raise ValueError('symlink component not permitted')
 return p
def tree_no_symlink(root,skip_git=True):
 for here,dirs,files in os.walk(root,topdown=True,followlinks=False):
  if skip_git:dirs[:]=[x for x in dirs if x!='.git']
  for x in dirs+files:
   if stat.S_ISLNK(os.lstat(Path(here)/x).st_mode):raise ValueError('symlink in evidence tree not permitted')
def ptr(o,path):
 for x in path.lstrip('/').split('/'):
  x=x.replace('~1','/').replace('~0','~');o=o[int(x)] if isinstance(o,list) else o[x]
 return o
def diffs(a,b,p):
 if isinstance(a,dict) and isinstance(b,dict):
  assert a.keys()==b.keys()
  for k in a:yield from diffs(a[k],b[k],p+'/'+str(k).replace('~','~0').replace('/','~1'))
 elif isinstance(a,list) and isinstance(b,list):
  assert len(a)==len(b)
  for i,(x,y) in enumerate(zip(a,b)):yield from diffs(x,y,p+'/'+str(i))
 elif a!=b:yield p,a,b
canon=lambda o:json.dumps(o,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path);ap.add_argument('--public-dir',type=Path);ap.add_argument('--private-dir',type=Path);args=ap.parse_args()
if args.private_dir and not args.public_dir:ap.error('--private-dir requires --public-dir')
no_symlink(P);tree_no_symlink(P,skip_git=False)
m=json.loads((P/'MANIFEST.json').read_bytes());assert m['schema']=='726-059-successor-stage-manifest-v1';files={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file()}-{'MANIFEST.json'};assert files==set(m['files'])
for n,x in m['files'].items():
 b=(P/n).read_bytes();assert sha(b)==x['sha256'] and len(b)==x['bytes'],n
rows=json.loads((P/'SOURCE-MAP.json').read_bytes())['rows'];assert len(rows)==15 and [x['checkpoint'] for x in rows]==list(range(15,30));assert (P/'SOURCE-MAP.json').read_bytes()==(P/'verification/REBUILT-SOURCE-MAP.json').read_bytes()
header=("import sys as _cw_sys\n_cw_sys.dont_write_bytecode = True\nfrom pathlib import Path as _CW_Path\nfrom qualified_crosswalk import configure as _cw_configure\n_CW = _cw_configure(_CW_Path(__file__).parent / 'CROSSWALK-059.json')\n");ending='\n_CW.finish()\n';known_old='known={sha((R/p).read_bytes()):len((R/p).read_bytes()) for p in paths}';known_new='known=_CW.known(paths)'
rep15=[(" if raw is None:raw=p.read_bytes()\n assert sha(raw)==ref['sha256'] and len(raw)==ref['bytes']"," if raw is None:raw=_CW.read(ref['path'])\n assert _CW.matches(ref['path'],raw,ref['sha256'],ref['bytes'])",1), (" raw=(R/a[key+'_path']).read_bytes();assert sha(raw)==a[key+'_sha256']"," raw=_CW.read(a[key+'_path']);assert _CW.matches(a[key+'_path'],raw,a[key+'_sha256'])",1),(" if raw is None:raw=(R/path).read_bytes()\n assert sha(raw)==sources[key]['sha256'] and len(raw)==sources[key]['bytes']"," if raw is None:raw=_CW.read(path)\n assert _CW.matches(path,raw,sources[key]['sha256'],sources[key]['bytes'])",1)]
rep16=[("raw=p.read_bytes();assert sha(raw)==x['sha256'] and len(raw)==x['bytes']","raw=_CW.read(x['path']);assert _CW.matches(x['path'],raw,x['sha256'],x['bytes'])",2),(known_old,known_new,2)]
rep20=[("raw=p.read_bytes();assert sha(raw)==x['sha256'] and len(raw)==x['bytes']","raw=_CW.read(x['path']);assert _CW.matches(x['path'],raw,x['sha256'],x['bytes'])",1)]
rep21=[("b=p.read_bytes();assert sha(b)==x['sha256'] and len(b)==x['bytes']","b=_CW.read(x['path']);assert _CW.matches(x['path'],b,x['sha256'],x['bytes'])",1)]
rep22=[("raw=p.read_bytes();assert (sha(raw),len(raw))==(x['sha256'],x['bytes'])","raw=_CW.read(x['path']);assert _CW.matches(x['path'],raw,x['sha256'],x['bytes'])",1)]
for row in rows:
 n=row['checkpoint'];old=(P/'originals'/row['original']).read_bytes();new=(P/'successors'/row['successor']).read_bytes();assert sha(old)==row['original_sha256'] and len(old)==row['original_bytes'] and sha(new)==row['successor_sha256'] and len(new)==row['successor_bytes'];assert new==(P/'verification/rebuilt'/row['successor']).read_bytes()
 s=new.decode();assert s.endswith(ending);s=s[:-len(ending)];edits=rep15 if n==15 else rep16 if n==16 else [(known_old,known_new,1)] if n in (17,18,19) else rep20 if n==20 else rep21 if n==21 else rep22
 for orig,repl,count in reversed(edits):assert s.count(repl)==count;s=s.replace(repl,orig)
 assert s.count('R=_CW.root')==1;s=s.replace('R=_CW.root','R=Path(__file__).resolve().parent',1);assert s.count(header)==1;s=s.replace(header,'',1);assert s.encode()==old
 assert sum(isinstance(z,ast.Assert) for z in ast.walk(ast.parse(old)))==sum(isinstance(z,ast.Assert) for z in ast.walk(ast.parse(new)))
config=(P/'successors/CROSSWALK-059.json').read_bytes();cfg=json.loads(config);helper=(P/'successors/qualified_crosswalk.py').read_bytes();assert sha(config).encode() in helper and len(cfg['records'])==2
receipt=json.loads((P/'verification/RUN-RECEIPT.json').read_bytes());assert len(receipt['results'])==15
for row,run in zip(rows,receipt['results']):
 assert run['checkpoint']==row['checkpoint'] and run['successor_sha256']==row['successor_sha256'] and run['helper_sha256']==sha(helper) and run['exit_code']==0 and not run['stderr']
 assert sha(run['stdout'].encode())==run['stdout_sha256'] and sha(run['stderr'].encode())==run['stderr_sha256'] and 'QUALIFIED_059_CROSSWALK_RESULT' in run['stdout'] and run['stdout'].count('QUALIFIED_ORIGINAL_TO_PUBLIC_CROSSWALK')==2
neg=json.loads((P/'verification/NEGATIVE-ORIGINAL-REJECTION.json').read_bytes());assert neg['exit_code']!=0 and neg['helper_sha256']==sha(helper)
sem=json.loads((P/'SEMANTIC-POINTERS.json').read_bytes());assert len(sem['records'])==7 and [x['changed_leaf_count'] for x in sem['records']]==[8,4,8,4,8,4,8]
if args.publication:
 r=no_symlink(args.publication);tree_no_symlink(r)
 plan=json.loads((P/'DRIVER-INTEGRATION.json').read_bytes());b=(r/plan['original_driver']['path']).read_bytes();assert sha(b)==plan['original_driver']['sha256'] and b==(P/plan['original_driver']['captured_copy']).read_bytes()
 for row in rows:assert sha((r/row['original']).read_bytes())==row['original_sha256']
 t=cfg['transport'];z=(r/t['path']).read_bytes();raw=gzip.decompress(z);assert sha(z)==t['stored_sha256'] and sha(raw)==t['uncompressed_sha256'];entries=json.loads(raw)['entries']
 for x in cfg['records']:
  y=[a for a in entries if a['path']==x['historical_transport_path']];assert len(y)==1 and (y[0]['sha256'],y[0]['bytes'],y[0]['git_blob'])==(x['original_sha256'],x['original_bytes'],x['original_git_blob'])
 for x in sem['records']:assert sha((r/x['referencing_file']).read_bytes())==x['referencing_file_sha256']
 print('Checked publication driver, 15 original scripts, transport tuples and seven referring-file hashes; no symlinks followed.')
else:print('Publication not supplied: external driver, original script, transport and referring-file identities were not rechecked.')
if args.public_dir:
 public=no_symlink(args.public_dir);tree_no_symlink(public,skip_git=False);public_docs={}
 for x in cfg['records']:
  b=(public/Path(x['public_path']).name).read_bytes();assert (sha(b),len(b),blob(b))==(x['public_sha256'],x['public_bytes'],x['public_git_blob']);public_docs[x['name']]=json.loads(b)
 if args.private_dir:
  private=no_symlink(args.private_dir);tree_no_symlink(private,skip_git=False);private_docs={}
  for x in cfg['records']:
   b=gzip.decompress((private/(x['name']+'.original.json.gz')).read_bytes());assert (sha(b),len(b),blob(b))==(x['original_sha256'],x['original_bytes'],x['original_git_blob']);private_docs[x['name']]=json.loads(b)
  for x in sem['records']:
   a=ptr(private_docs[x['document']],x['referenced_pointer']);b=ptr(public_docs[x['document']],x['referenced_pointer']);d=list(diffs(a,b,x['referenced_pointer']));assert [z[0] for z in d]==x['changed_descendant_pointers'] and len(d)==x['changed_leaf_count'];assert sha(canon(a))==x['original_subtree_canonical_json_sha256'] and sha(canon(b))==x['public_subtree_canonical_json_sha256']
  print('Compared all seven historical/public subtrees with locally retained originals; only recorded path values differ.')
 else:print('Private originals not supplied: public blobs checked but semantic transformation was not independently recomputed.')
else:print('Public blobs not supplied: external public SHA/length/Git-blob checks were not repeated.')
print('Verified 15 deterministic source adaptations, exact original assertion preservation, qualified run receipts and semantic inventory; no old verifier or product code ran.')
