#!/usr/bin/env python3
"""Classify bounded pattern hits without saving matched text or addresses."""
import argparse,ast,collections,gzip,hashlib,io,json,pathlib,re,subprocess,tarfile
p=argparse.ArgumentParser();p.add_argument('--stage',required=True,type=pathlib.Path);p.add_argument('--publication',required=True,type=pathlib.Path);p.add_argument('--out',required=True,type=pathlib.Path);a=p.parse_args();H=lambda b:hashlib.sha256(b).hexdigest();root=a.stage;expected='5dfe0bfe427cf9089f636ac8e52ae1a7e479e5ce0372e2fa8d884cd3f6c07e15';assert H((root/'STAGE-MANIFEST.json').read_bytes())==expected
path=r'/('+'?:home|users|tmp|root|mnt|workspace|workspaces|private|var/(?:tmp|folders)' +r')/[^\s"\x27<>\x00\\]+';win=r'[A-Za-z]:[\\/]+(?:Users|Documents and Settings)[\\/]+[^\s"\x27<>\x00]+';rx=re.compile((r'(?:(?<![A-Za-z0-9_])|\\[nrt])('+path+'|'+win+')').encode(),re.I)
with tarfile.open(root/'archives/hair-gpu.tar.gz') as t:records=[('hair-gpu/evidence/privacy-scan.py.txt',t.extractfile('evidence/privacy-scan.py.txt').read()),('hair-gpu/privacy-check.py',t.extractfile('privacy-check.py').read()),('scan-private-digests.py',(root/'scan-private-digests.py').read_bytes())]
found=[];tokens=[]
for name,b in records:
 norm=re.sub(rb'(?i)\\u002f',b'/',b.replace(b'\\/',b'/'));tree=ast.parse(b.decode());matches=list(rx.finditer(norm));assert len(matches)==1;m=matches[0];v=m.group(1);line=norm.count(b'\n',0,m.start())+1;constants=[]
 for n in ast.walk(tree):
  if isinstance(n,ast.Constant) and isinstance(n.value,(str,bytes)) and n.lineno<=line<=getattr(n,'end_lineno',n.lineno):
   literal=n.value.encode() if isinstance(n.value,str) else n.value
   if v in literal or v in literal.replace(b'\\/',b'/'):constants.append(n)
 assert constants;found.append({'file':name,'line':line,'classification':'pattern bytes literal in source code'});tokens.append(v)
assert tokens[1]==tokens[2]
# The additional email-pattern views are inside compressed binary, with the
# tar-stream view duplicating the member. Matched bytes remain in memory.
email=re.compile(rb'[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}');garment=(root/'archives/garment-pattern.tar.gz').read_bytes();stream=gzip.decompress(garment);outer=list(email.finditer(garment));tarhit=list(email.finditer(stream))
with tarfile.open(fileobj=io.BytesIO(stream),mode='r:') as t:
 z=t.getmember('bundle/reference/ledger.json.gz');inner=t.extractfile(z).read();inside=list(email.finditer(inner));assert garment.startswith(b'\x1f\x8b') and inner.startswith(b'\x1f\x8b') and len(outer)==len(tarhit)==len(inside)==1 and z.offset_data<=tarhit[0].start()<z.offset_data+z.size and tarhit[0].group()==inside[0].group()
# Check all disclosed 15 commit-object copies against locally retained d7.
d7='d7b14a15ef132422eca86e7d898b47423af92779';b=subprocess.check_output(['git','-C',str(a.publication),'show',d7+':delegated/gpu-reflection-evidence.tar.gz']);prior={}
with tarfile.open(fileobj=io.BytesIO(b),mode='r:gz') as t:
 for x in t:
  if x.isfile() and x.name.endswith('.commit.gz'):prior[pathlib.PurePosixPath(x.name).name]=t.extractfile(x).read()
assert len(prior)==3
copy_counts=collections.Counter();copies=0
for name in ['garment-pattern','cloth','gpu-reflection','hair-head','hair-style']:
 with tarfile.open(root/('archives/'+name+'.tar.gz'),'r:gz') as t:
  files=[x for x in t if x.isfile() and x.name.endswith('.commit.gz')];assert len(files)==3
  for x in files:
   basename=pathlib.PurePosixPath(x.name).name;stored=t.extractfile(x).read();assert stored==prior[basename];decoded=gzip.decompress(stored);blob=hashlib.sha1(b'commit '+str(len(decoded)).encode()+b'\0'+decoded).hexdigest();assert blob==basename.split('.')[0];copy_counts[blob]+=1;copies+=1
assert copies==15 and all(x==5 for x in copy_counts.values());assert H((root/'STAGE-MANIFEST.json').read_bytes())==expected
out={'schema':'726-successor-pattern-classification-v1','stage_manifest_sha256':expected,'raw_pattern_scan_candidates':5,'source_code_regex_literal_occurrences':3,'tar_stream_duplicate_occurrences':2,'source_locations':found,'binary_email_pattern_views':3,'binary_email_unique_streams':2,'binary_email_qualification':'One pattern occurs in the outer gzip stored bytes; another occurs in an inner gzip stored member and is repeated in its enclosing tar view. Binary matches are not established email addresses.','disclosed_commit_objects':{'copies':15,'distinct_objects':3,'archive_count':5,'project_commit_objects_match_locally_retained_d7':True,'local_prior_revision':d7,'qualification':'Exact Git commit bytes include author/committer metadata. The local d7 archive was inspected, with no live-remote check or claim of full personal-data clearance.'},'qualification':'Pattern classification only; no matched value, address or individual-value digest is output. Unknown patterns or encodings remain outside scope.'};assert not a.out.exists();a.out.write_text(json.dumps(out,indent=2)+'\n');print(json.dumps({'pattern_literals':3,'binary_email_streams':2,'commit_copies':15,'distinct_commits':3}))
