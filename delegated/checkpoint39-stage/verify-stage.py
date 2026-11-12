#!/usr/bin/env python3
"""Read-only checkpoint-39 evidence verifier; never runs product code."""
import argparse,collections,gzip,hashlib,io,json,pwd,re,sys,tarfile,tomllib
from pathlib import Path
P=Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest();slash=b'/'
def filehash(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
ap=argparse.ArgumentParser(description='Verify public evidence and optionally retained originals and source packets; this program performs no writes.')
ap.add_argument('--local-originals-root',type=Path,help='Local-only root containing changed-originals/ content-addressed objects')
for n in ('2623','2625','2626'):ap.add_argument('--source-'+n,type=Path,help='Separately retained frozen source packet root')
ap.add_argument('--prior-original-root',type=Path,help='Separately retained exact 83-member prior source root')
args=ap.parse_args();srcs=[args.source_2623,args.source_2625,args.source_2626]
if any(srcs) and not all(srcs):ap.error('supply all three source roots together')
m=json.loads((P/'MANIFEST.json').read_bytes());actual={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file()}-{'MANIFEST.json'};assert actual==set(m['files'])
for name,x in m['files'].items():
 f=P/name;assert f.is_file() and not f.is_symlink() and f.stat().st_size==x['bytes'] and filehash(f)==x['sha256']
idx=json.loads((P/'EVIDENCE-IDENTITIES.json').read_bytes());report=json.loads((P/'REPORT.json').read_bytes());supp=json.loads((P/'SUPPLEMENTAL-PRIOR-MEMBERS.json').read_bytes());cmp=json.loads((P/'PRIOR-ARCHIVE-MANIFEST-COMPARISON.json').read_bytes());policy=json.loads((P/'PRIVACY-POLICY.json').read_bytes())
assert len(idx['source_packets'])==3 and len(idx['members'])==333 and len(supp['members'])==5
assert report['counts']['total_source_and_supplemental_files']==338 and report['counts']['total_private_path_replacements']==125
by_packet=collections.defaultdict(dict);git_count=0
for x in idx['members']:
 f=P/x['public_path'];raw=f.read_bytes();assert (H(raw),len(raw))==(x['public_stored_sha256'],x['public_stored_bytes']);assert x['path'] not in by_packet[x['packet']];by_packet[x['packet']][x['path']]=x
 dec=gzip.decompress(raw) if x['encoding'] in ('gzip','gzip-tar') else raw;assert (H(dec),len(dec))==(x['public_decoded_sha256'],x['public_decoded_bytes'])
 if x['status']=='exact-safe-copy':assert not x['redactions'] and x['original_stored_sha256']==x['public_stored_sha256'] and x['original_decoded_sha256']==x['public_decoded_sha256']
 else:assert x['status']=='privacy-reconstructed' and x['redactions']
 if x['git_object']:
  z=x['git_object'];assert hashlib.sha1(z['type'].encode()+b' '+str(len(dec)).encode()+b'\0'+dec).hexdigest()==z['git_sha1'] and x['status']=='exact-safe-copy';git_count+=1
 if x['path'].endswith('.json') and x['encoding']=='identity':json.loads(raw)
assert git_count==87
for source in idx['source_packets']:
 files=by_packet[source['packet']];mr=files[source['source_manifest']];assert mr['status']=='exact-safe-copy' and mr['original_stored_sha256']==source['source_manifest_sha256'];doc=json.loads((P/mr['public_path']).read_bytes());z=doc['files'];expected={x['path']:(x['sha256'],x['bytes']) for x in z} if isinstance(z,list) else {k:(v['sha256'],v['bytes']) for k,v in z.items()};assert len(expected)==source['source_manifest_member_count']
 for name,(sha,size) in expected.items():assert name in files and (files[name]['original_stored_sha256'],files[name]['original_stored_bytes'])==(sha,size) and files[name]['source_manifest_bound']
 extras=set(files)-set(expected)-{source['source_manifest']};assert extras==set(source['supplemental_original_members'])
for x in supp['members']:
 raw=(P/x['public_path']).read_bytes();assert (H(raw),len(raw))==(x['public_stored_sha256'],x['public_stored_bytes']);dec=gzip.decompress(raw) if x['encoding']=='gzip' else raw;assert H(dec)==x['public_decoded_sha256'] and len(dec)==x['public_decoded_bytes'];assert (x['status']=='privacy-reconstructed')==bool(x['redactions'])
assert sum(x['status']=='privacy-reconstructed' for x in idx['members'])==34 and sum(x['status']=='privacy-reconstructed' for x in supp['members'])==4
assert sum(len(x['redactions']) for x in idx['members'])==118 and sum(len(x['redactions']) for x in supp['members'])==7
archive_map={(a['packet'],a['source_path']):a for a in idx['archives']};entry_count=0;nested_gzip=0
for key,a in archive_map.items():
 x=by_packet[key[0]][key[1]];blob=(P/x['public_path']).read_bytes();assert H(blob)==a['original_archive_sha256']==a['public_archive_sha256']
 with tarfile.open(fileobj=io.BytesIO(blob),mode='r:gz') as t:
  entries=list(t);assert len(entries)==len(a['entries'])
  for z,row in zip(entries,a['entries']):
   assert z.name==row['member'] and not z.name.startswith('/') and '..' not in Path(z.name).parts and not z.uname and not z.gname;assert z.isdir() or z.isfile();assert row['kind']==('directory' if z.isdir() else 'regular')
   if z.isfile():
    b=t.extractfile(z).read();assert (H(b),len(b))==(row['original_stored_sha256'],row['original_bytes']) and H(b)==row['public_stored_sha256'];plain=gzip.decompress(b) if row['encoding']=='gzip' else b;assert (H(plain),len(plain))==(row['original_decoded_sha256'],row['decoded_bytes']) and H(plain)==row['public_decoded_sha256'];nested_gzip+=row['encoding']=='gzip'
   entry_count+=1
assert len(archive_map)==3 and entry_count==184 and nested_gzip==33
# The prior manifest is a historical original record, not a manifest of its
# earlier reconstructed archive. Verify all original/public distinctions.
bind=json.loads((P/'OMITTED-MANIFEST-BINDING.json').read_bytes());pm=(P/bind['retained_public_member']).read_bytes();assert H(pm)==bind['omitted_source_member_sha256']==cmp['prior_manifest_sha256'];prior=json.loads(pm);items={x['path']:x for x in prior['files']};assert len(items)==83 and len(cmp['members'])==83
with tarfile.open(P/bind['prior_published_archive'],'r:gz') as t:
 assert H(t.extractfile(t.getmember(bind['inner_member'])).read())==bind['inner_member_sha256']
 for row in cmp['members']:
  x=items[row['prior_member']];raw=t.extractfile(t.getmember(row['prior_member'])).read();assert (x['sha256'],x['bytes'])==(row['prior_manifest_original_stored_sha256'],row['prior_manifest_original_stored_bytes']) and (H(raw),len(raw))==(row['prior_published_stored_sha256'],row['prior_published_stored_bytes']);assert (H(raw)==x['sha256'])==row['original_equals_prior_published'];plain=gzip.decompress(raw) if raw.startswith(b'\x1f\x8b') else raw;assert H(plain)==row['prior_published_decoded_sha256']
  if not row['original_equals_prior_published']:
   y=next(x for x in supp['members'] if x['prior_member']==row['prior_member']);assert (y['original_stored_sha256'],y['public_path'],y['public_stored_sha256'])==(row['prior_manifest_original_stored_sha256'],row['supplemental_public_path'],row['supplemental_public_stored_sha256'])
assert sum(x['original_equals_prior_published'] for x in cmp['members'])==78
native=json.loads((P/'NATIVE-PACKAGE-PROVENANCE.json').read_bytes())
for z in native['package_archives']:assert filehash(P/z['path'])==z['sha256']
for view in native['lock_views']:
 raw=(P/'bundles'/'2626-2628'/view['view']).read_bytes();plain=gzip.decompress(raw);assert (H(raw),H(plain))==(view['stored_sha256'],view['decoded_sha256']);lock=tomllib.loads(plain.decode())
 for pkg in view['matched_packages']:
  found=[x for x in lock['package'] if x['name']==pkg['name'] and x['version']==pkg['version']];assert len(found)==1 and found[0]['checksum']==pkg['checksum'] and found[0]['source']==pkg['registry_source']
historical=json.loads((P/'HISTORICAL-BOUNDARIES.json').read_bytes());assert filehash(P/historical['p16_patch']['source'])==historical['p16_patch']['sha256']
falsepos=json.loads((P/'FALSE-POSITIVE-PRESERVATION.json').read_bytes())
for row in falsepos['checks']:
 raw=(P/row['public_path']).read_bytes();plain=gzip.decompress(raw) if raw.startswith(b'\x1f\x8b') else raw;assert H(raw)==row['public_stored_sha256'] and H(plain)==row['public_decoded_sha256'] and plain.count(row['known_non_path_phrase'].encode())==row['public_phrase_occurrences']
for records in historical['original_059_identities'].values():
 for x in records:
  y=by_packet[x['packet']][x['path']]
  for k in ('encoding','original_stored_sha256','original_decoded_sha256','public_stored_sha256','public_decoded_sha256','status'):assert x[k]==y[k]
# Contextual scan of public decoded bytes. Only known, reviewed semantic tokens
# and canonical agent IDs are exempt; private values are never printed.
agents={slash+b'root/'+x.encode() for x in policy['agent_identifiers']};semantic=set(policy['semantic_fragment_sha256']);regexsafe=set(policy['known_literal_regex_sha256'])
root=slash+rb'(?:home|users|tmp|root|mnt|workspace|workspaces|private|var/(?:tmp|folders))/[^\s"\x27<>\x00\\]+'
win=rb'[A-Za-z]:[\\/]+(?:Users|Documents and Settings)[\\/]+[^\s"\x27<>\x00]+'
path_rx=re.compile(rb'(?:(?<![A-Za-z0-9_])|\\[nrt])('+root+rb'|'+win+rb')',re.I)
home=Path('/'+'home');users={u.pw_name.lower() for u in pwd.getpwall()}|({x.name.lower() for x in home.iterdir() if x.is_dir()} if home.is_dir() else set());u_rx=re.compile(slash+rb'(?:home|users)/(?:'+b'|'.join(re.escape(u.encode()) for u in users if len(u)>=2)+rb')(?:/|\\/|["\x27\s]|$)',re.I)
secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |DSA |OPENSSH |EC |ENCRYPTED )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{32,}|AIza[0-9A-Za-z_-]{35}|glpat-[0-9A-Za-z_-]{20,}|npm_[0-9A-Za-z]{30,}|eyJ[A-Za-z0-9_-]{18,}\.[A-Za-z0-9_-]{15,}\.[A-Za-z0-9_-]{15,}|(?:Bearer\s+)[A-Za-z0-9._~+/-]{24,}',re.I)
issues=[];stats=collections.Counter()
def scan(where,b):
 a=re.sub(rb'(?i)\\u002f',slash,b.replace(b'\\/',slash));stats['decoded_payloads']+=1;stats['decoded_bytes']+=len(a)
 for z in path_rx.finditer(a):
  token=z.group(1);bare=token.rstrip(b'.,;:!?`');pref=a[max(0,z.start()-180):z.start()].lower()
  if bare in agents:stats['agent_ids']+=1;continue
  if H(token) in semantic or H(bare) in semantic:assert not u_rx.search(token);stats['known_nonpath']+=1;continue
  if H(token) in regexsafe:stats['regex_literals']+=1;continue
  if bare==slash+b'tmp/726' and b'paths' in a[z.end():z.end()+25].lower():continue
  if bare.startswith(slash+b'root/') and pref.endswith(b'..') and b'"path"' in pref:continue
  issues.append((where,'path',H(token)))
 for rx,kind in ((u_rx,'account'),(secret,'secret')):
  for z in rx.finditer(a):issues.append((where,kind,H(z.group())))
for f in sorted(P.rglob('*')):
 if not f.is_file():continue
 rel=str(f.relative_to(P));raw=f.read_bytes()
 if raw.startswith(b'\x1f\x8b'):
  plain=gzip.decompress(raw)
  try:
   with tarfile.open(fileobj=io.BytesIO(plain),mode='r:') as t:
    members=list(t)
    if not members:raise tarfile.TarError()
    for z in members:
     if z.isfile():
      b=t.extractfile(z).read();scan(rel+'!'+z.name,gzip.decompress(b) if b.startswith(b'\x1f\x8b') else b)
    continue
  except tarfile.TarError:pass
  scan(rel,plain)
 else:scan(rel,raw)
assert not issues,('unclassified privacy candidates',issues[:8])
print('Verified public manifest, 338 evidence files, source manifests, 87 Git objects, three archives/184 entries, native locks, prior 83-member comparison and contextual privacy scan.')
# Optional exact-original replay. Original values are never printed.
if args.local_originals_root:
 changed=[x for x in idx['members'] if x['redactions']]+[x for x in supp['members'] if x['redactions']];labels={};seen=set()
 for x in changed:
  sha=x['original_stored_sha256'];raw=(args.local_originals_root/'changed-originals'/sha).read_bytes();assert (H(raw),len(raw))==(sha,x['original_stored_bytes']);seen.add(sha);plain=gzip.decompress(raw) if x['encoding']=='gzip' else raw;parts=[];last=0
  for z in x['redactions']:
   start=z['original_decoded_offset_0based'];end=start+z['original_decoded_bytes'];assert start>=last;token=plain[start:end];norm=re.sub(rb'(?i)\\u002f',slash,token.replace(b'\\/',slash));key=z['public_placeholder']
   if key in labels:assert labels[key]==norm
   labels[key]=norm;parts.extend((plain[last:start],key.encode()));last=end
  parts.append(plain[last:]);out=b''.join(parts);pub=(P/x['public_path']).read_bytes();assert out==(gzip.decompress(pub) if x['encoding']=='gzip' else pub)
 assert len(changed)==38 and len(seen)==36 and len(labels)==56
 print('Verified 36 retained original objects and replayed all 125 path-redaction operations without printing private values.')
if all(srcs):
 roots={'2623-2624':args.source_2623,'2625':args.source_2625,'2626-2628':args.source_2626}
 for x in idx['members']:
  f=roots[x['packet']]/x['path'];assert f.stat().st_size==x['original_stored_bytes'] and filehash(f)==x['original_stored_sha256']
 for row in falsepos['checks']:
  raw=(roots[row['packet']]/row['source_path']).read_bytes();plain=gzip.decompress(raw) if raw.startswith(b'\x1f\x8b') else raw;assert H(raw)==row['original_stored_sha256'] and H(plain)==row['original_decoded_sha256'] and plain.count(row['known_non_path_phrase'].encode())==row['original_phrase_occurrences']
 print('Verified all 333 frozen source-packet member identities and the known non-path examples against separately retained inputs.')
if args.prior_original_root:
 raw=(args.prior_original_root/'MANIFEST.json').read_bytes();assert H(raw)==cmp['prior_manifest_sha256'];total=0
 for x in prior['files']:
  f=args.prior_original_root/x['path'];assert f.stat().st_size==x['bytes'] and filehash(f)==x['sha256'];total+=1
 assert total==83;print('Verified all 83 separately retained prior-original members against their original manifest.')
if not args.local_originals_root:print('Local exact changed originals and external source packets are not included in this public stage.')
