#!/usr/bin/env python3
"""Read-only verification of public review reconstructions and optional inputs."""
import argparse,collections,gzip,hashlib,io,json,pwd,re,sys,tarfile
from pathlib import Path
P=Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest();slash=b'/'
def filehash(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
ap=argparse.ArgumentParser(description='Verify the public packet. Optional arguments independently verify local originals and/or the external CP37 stage snapshot; no files are written.')
ap.add_argument('--local-originals-root',type=Path,help='Root holding the five exact original review directories')
ap.add_argument('--checked-stage',type=Path,help='Separately retained CP37 checked-stage directory')
ap.add_argument('--checked-stage-snapshot',type=Path,help='Separately retained CP37 snapshot manifest')
args=ap.parse_args()
if bool(args.checked_stage)!=bool(args.checked_stage_snapshot):ap.error('supply both checked-stage options together')
m=json.loads((P/'MANIFEST.json').read_bytes());actual={str(f.relative_to(P)) for f in P.rglob('*') if f.is_file()}-{'MANIFEST.json'};assert actual==set(m['files'])
for rel,z in m['files'].items():
 p=P/rel;assert p.is_file() and not p.is_symlink() and p.stat().st_size==z['bytes'] and filehash(p)==z['sha256']
idx=json.loads((P/'REVIEW-IDENTITIES.json').read_bytes());report=json.loads((P/'REPORT.json').read_bytes());deps=json.loads((P/'DEPENDENCIES.json').read_bytes());ex=json.loads((P/'PRIVACY-EXCEPTIONS.json').read_bytes())
assert len(idx['reviews'])==5 and len(idx['files'])==71 and len(idx['nested_archives'])==2
by_review=collections.defaultdict(dict)
for f in idx['files']:
 p=P/f['public_relative_path'];raw=p.read_bytes();assert (H(raw),len(raw))==(f['public_sha256'],f['public_bytes']);assert f['original_relative_path'] not in by_review[f['review']];by_review[f['review']][f['original_relative_path']]=f
 assert f['status'] in ('exact-safe-copy','privacy-reconstructed')
 if f['status']=='exact-safe-copy':assert not f['redactions'] and f['original_sha256']==f['public_sha256']
 else:assert f['redactions'] and f['original_sha256']!=f['public_sha256']
 if f['original_relative_path'].endswith('.json'):json.loads(raw)
assert sum(f['status']=='privacy-reconstructed' for f in idx['files'])==6
assert sum(len(f['redactions']) for f in idx['files'])==27
for review in idx['reviews']:
 files=by_review[review['review']];mf=P/files['MANIFEST.json']['public_relative_path'];assert H(mf.read_bytes())==review['original_manifest_sha256'];old=json.loads(mf.read_bytes());expect={x['path']:(x['sha256'],x['bytes']) for x in old['files']};assert set(expect)==set(files)-{'MANIFEST.json'}
 for name,(sha,size) in expect.items():assert (files[name]['original_sha256'],files[name]['original_bytes'])==(sha,size)
# Recursive identities for already-reconstructed CP7 archives; gzip payloads
# are decoded for content checking, never executed or extracted to the host.
nested_count=0;gzip_count=0;archives={x['review']:x for x in idx['nested_archives']}
assert set(archives)=={'cp7-initial','cp7-successor'}
for review,a in archives.items():
 rec=by_review[review][a['member_path']];blob=(P/rec['public_relative_path']).read_bytes();assert H(blob)==a['source_archive_sha256']==a['public_archive_sha256'];assert H(blob)!=deps['cp7_initial_and_successor']['separate_private_original_archive_sha256']
 with tarfile.open(fileobj=io.BytesIO(blob),mode='r:gz') as t:
  entries=list(t);assert len(entries)==a['entry_count']==186
  for z,meta in zip(entries,a['entries']):
   assert z.name==meta['member'] and not z.name.startswith('/') and '..' not in Path(z.name).parts and not z.uname and not z.gname
   assert z.isdir() or z.isfile();assert meta['kind']==('directory' if z.isdir() else 'regular')
   if z.isfile():
    raw=t.extractfile(z).read();assert (len(raw),H(raw),H(raw))==(meta['original_bytes'],meta['original_sha256'],meta['public_sha256'])
    plain=gzip.decompress(raw) if meta['content_encoding']=='gzip' else raw
    assert H(plain)==meta['original_decoded_sha256']==meta['public_decoded_sha256'] and len(plain)==meta['decoded_bytes']
    if meta['content_encoding']=='gzip':gzip_count+=1
   nested_count+=1
assert nested_count==372 and gzip_count==104
assert archives['cp7-initial']['public_archive_sha256']==deps['cp7_initial_and_successor']['initial_reconstruction_archive_sha256']
assert archives['cp7-successor']['public_archive_sha256']==deps['cp7_initial_and_successor']['successor_archive_sha256']
# Context-aware privacy scan. Original path tokens are never emitted. Known
# false-positive phrases and regex literals are fingerprinted and attributed.
agents={slash+b'root/'+x.encode() for x in ('hair_ops','rig_identity','mhr_character','camera_targeting','rig_garments','native_reports','lens12_frontier','segovb')}
root=slash+rb'(?:home|users|tmp|root|mnt|workspace|workspaces|private|var/(?:tmp|folders))/[^\s"\x27<>]+'
win=rb'[A-Za-z]:[\\/]+(?:Users|Documents and Settings)[\\/]+[^\s"\x27<>]+'
path_rx=re.compile(rb'(?:(?<![A-Za-z0-9_])|\\[nrt])('+root+rb'|'+win+rb')',re.I)
home=Path('/'+'home');users={u.pw_name.lower() for u in pwd.getpwall()}|({x.name.lower() for x in home.iterdir() if x.is_dir()} if home.is_dir() else set())
userpat=re.compile(slash+rb'(?:home|users)/(?:'+b'|'.join(re.escape(u.encode()) for u in users if len(u)>=2)+rb')(?:/|\\/|["\x27\s]|$)',re.I)
secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |DSA |OPENSSH |EC |ENCRYPTED )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{32,}|AIza[0-9A-Za-z_-]{35}|glpat-[0-9A-Za-z_-]{20,}|npm_[0-9A-Za-z]{30,}|eyJ[A-Za-z0-9_-]{18,}\.[A-Za-z0-9_-]{15,}\.[A-Za-z0-9_-]{15,}|(?:Bearer\s+)[A-Za-z0-9._~+/-]{24,}',re.I)
semantic=set(ex['semantic_fragment_fingerprints']);regexsafe=set(ex['fixed_regex_literal_fingerprints']);observed=[];privacy=collections.Counter();issues=[]
def scan(where,b,record):
 b=re.sub(rb'(?i)\\u002f',slash,b.replace(b'\\/',slash));privacy['decoded_bytes']+=len(b);privacy['leaf_payloads']+=1
 for z in path_rx.finditer(b):
  token=z.group(1);bare=token.rstrip(b'.,;:!?');pre=b[max(0,z.start()-180):z.start()].lower();kind=None
  if bare in agents:privacy['canonical_agent_id']+=1;continue
  if H(token) in semantic or H(bare) in semantic:kind='prior-reviewed-nonpath-fragment'
  elif H(token) in regexsafe:kind='fixed-source-regex-literal'
  elif bare==slash+b'tmp/726' and b'paths' in b[z.end():z.end()+25].lower():kind='generic-temp-prefix-in-prose'
  elif bare.startswith(slash+b'root/') and pre.endswith(b'..') and b'"path"' in pre:kind='relative-source-reference'
  else:issues.append((where,'path',H(token)))
  if kind and record:observed.append({'public_member':where,'normalized_offset_0based':z.start(1),'matched_sha256':H(token),'classification':kind})
 for rx,kind in ((userpat,'account'),(secret,'secret')):
  for z in rx.finditer(b):issues.append((where,kind,H(z.group())))
for f in sorted(P.rglob('*')):
 if not f.is_file():continue
 rel=str(f.relative_to(P));raw=f.read_bytes();record=rel.startswith('reviews/')
 if f.name.endswith('.tar.gz'):
  with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
   for z in t:
    if z.isfile():
     a=t.extractfile(z).read();scan(rel+'!'+z.name,gzip.decompress(a) if a.startswith(b'\x1f\x8b') else a,record)
 else:scan(rel,raw,record)
assert not issues,('privacy candidates',issues[:10]);assert observed==ex['classified_occurrences'] and privacy['canonical_agent_id']>=ex['canonical_agent_id_occurrences']
assert deps['cp37']['snapshot_manifest_sha256']=='4a0ac2ca629ddc23900ca3e643e83bdd2054afc3addbb5aae047f9a62023e744'
print('Verified public manifest, five original-manifest bindings, 71 file identities, both CP7 archives, 372 nested entries, 104 decoded gzip members and contextual privacy scan.')
if args.local_originals_root:
 rootdir=args.local_originals_root;checked=0;global_labels={}
 for f in idx['files']:
  src=rootdir/f['review']/f['original_relative_path'];raw=src.read_bytes();assert (H(raw),len(raw))==(f['original_sha256'],f['original_bytes']);last=0;parts=[]
  for z in f['redactions']:
   start=z['original_offset_0based'];end=start+z['original_bytes'];assert start>=last;token=raw[start:end];mark=z['public_placeholder'];assert token and mark.startswith('<PRIVATE_')
   if mark in global_labels:assert global_labels[mark]==token
   global_labels[mark]=token;parts.extend((raw[last:start],mark.encode()));last=end
  parts.append(raw[last:]);assert b''.join(parts)==(P/f['public_relative_path']).read_bytes();checked+=1
 print('Verified',checked,'exact local original files and all redaction operations without printing path values.')
if args.checked_stage:
 raw=args.checked_stage_snapshot.read_bytes();assert H(raw)==deps['cp37']['snapshot_manifest_sha256'];rows=json.loads(raw)['files'];assert len(rows)==deps['cp37']['independently_verified_stage_regular_file_count'];wanted={x['path'] for x in rows};found={str(x.relative_to(args.checked_stage)) for x in args.checked_stage.rglob('*') if x.is_file()};assert wanted==found
 total=0
 for x in rows:
  path=args.checked_stage/x['path'];assert not path.is_symlink() and path.stat().st_size==x['bytes'] and filehash(path)==x['sha256'];total+=x['bytes']
 assert total==deps['cp37']['independently_verified_stage_total_bytes'];print('Verified the external CP37 checked-stage snapshot:',len(rows),'files and',total,'bytes.')
if not args.local_originals_root or not args.checked_stage:print('Optional independently retained originals and CP37 stage are not included in the public packet.')
