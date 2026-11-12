#!/usr/bin/env python3
"""Read-only verification of an additive, sanitized evidence packet."""
import argparse,collections,gzip,hashlib,io,json,pwd,re,sys,tarfile
from pathlib import Path
P=Path(__file__).resolve().parent
H=lambda b:hashlib.sha256(b).hexdigest()
def hashfile(path):
 h=hashlib.sha256()
 with path.open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
def child(raw,chain):
 for name in chain:
  with tarfile.open(fileobj=io.BytesIO(raw),mode='r:*') as t:
   z=t.getmember(name);assert z.isfile();raw=t.extractfile(z).read()
 return raw
ap=argparse.ArgumentParser(description='Verify packet alone, then optionally verify observed indexed archives and privately retained original identities. This program performs no writes.')
ap.add_argument('--publication-root',type=Path,help='Observed publication directory for checkpoints 1 through 35')
for n in (36,37,38):ap.add_argument('--stage'+str(n)+'-root',type=Path,help='Observed staged publication directory for this checkpoint')
ap.add_argument('--private-inventory',type=Path,help='Optional local-only exact original mapping; requires all four publication roots')
args=ap.parse_args()
roots_supplied=[args.publication_root,args.stage36_root,args.stage37_root,args.stage38_root]
if any(roots_supplied) and not all(roots_supplied):ap.error('supply all four publication roots together')
if args.private_inventory and not all(roots_supplied):ap.error('original verification also requires all four publication roots')
m=json.loads((P/'MANIFEST.json').read_bytes());files={str(x.relative_to(P)) for x in P.rglob('*') if x.is_file()}-{'MANIFEST.json'}
assert files==set(m['files'])
for name,z in m['files'].items():
 path=P/name;assert path.is_file() and not path.is_symlink();assert path.stat().st_size==z['bytes'] and hashfile(path)==z['sha256']
r=json.loads((P/'REPORT.json').read_bytes());d=json.loads((P/'RECONSTRUCTION.json').read_bytes());qbytes=(P/'PRIOR-SYSTEMIC-OCCURRENCES.json').read_bytes();q=json.loads(qbytes)
assert H(qbytes)==r['input_inventory_sha256']
assert H((P/'PRIOR-SYSTEMIC-PROPOSAL.json').read_bytes())=='a8552be3b611c1499f3e5c9750590b7909a64b6b252f9b692b903d07587b92c9'
assert H((P/'original-cp36-review/MANIFEST.json').read_bytes())=='c07f3e935a58ac9894969ef6b6a0891c05727c1edeeea378a2801e2943173794'
assert H((P/'original-cp36-review-correction/MANIFEST.json').read_bytes())=='35ce2e165819c2f5ebb42a4b864ad0caf68b300650f825d706ddc8a500d3a1e8'
for packet in ('original-cp36-review','original-cp36-review-correction'):
 sub=P/packet;submanifest=json.loads((sub/'MANIFEST.json').read_bytes());subfiles={str(x.relative_to(sub)) for x in sub.rglob('*') if x.is_file()}-{'MANIFEST.json'};assert subfiles==set(submanifest['files'])
 for name,x in submanifest['files'].items():
  raw=(sub/name).read_bytes();assert (len(raw),H(raw))==(x['bytes'],x['sha256'])
old_review=json.loads((P/'original-cp36-review/verification.json').read_bytes());corrected_review=json.loads((P/'original-cp36-review-correction/verification-corrected.json').read_bytes())
field='user_extraction_original_proposal_sha256';assert old_review[field]!=corrected_review[field];old_review[field]=corrected_review[field];assert old_review==corrected_review
assert len(d['identities'])==27 and len(d['locations'])==310 and len(q['occurrences'])==390
pairs={(x['source_original_sha256'],x['old_public_sha256']):x for x in d['identities']};assert len(pairs)==27
loc={(x['checkpoint'],x['archive'],tuple(x['member_chain'])):x for x in d['locations']};assert len(loc)==310
source=collections.defaultdict(list)
for x in q['occurrences']:
 k=(x['checkpoint'],x['archive'],tuple(x['member_chain']));source[k].append(x)
assert set(source)==set(loc)
for k,rows in source.items():
 p=loc[k];x=rows[0];assert p['source_original_sha256']==x['source_original_sha256'] and p['old_public_sha256']==x['public_reconstructed_sha256'] and p['archive_sha256']==x['archive_sha256']
 assert p['original_occurrence_offsets_0based']==[x['original_byte_offset_0based'] for x in rows]
 rec=pairs[(p['source_original_sha256'],p['old_public_sha256'])];assert p['status']==rec['status'] and p['corrected_sha256']==rec.get('corrected_sha256') and p['corrected_member']==rec.get('corrected_member')
 if rec['status']=='corrected':
  edits={v['original_offset_0based']:v for v in rec['corrections']}
  for row in rows:
   c=edits[row['original_byte_offset_0based']];fragment=c['non_path_context_including_replacement'].encode()[c['leading_context_bytes']:]
   assert (len(fragment),c['public_placeholder'],c['original_line_1based'],c['recognized_non_path_excerpt'])==(row['redacted_byte_length'],row['public_placeholder'],row['original_line_1based'],row['non_path_excerpt'])
for rec in d['identities']:
 if rec['status']!='corrected':assert rec['status']=='withheld-opaque-tar';continue
 raw=(P/rec['corrected_member']).read_bytes();assert (H(raw),len(raw))==(rec['corrected_sha256'],rec['corrected_bytes']);plain=gzip.decompress(raw) if rec['encoding']=='gzip' else raw;assert H(plain)==rec['corrected_uncompressed_sha256']
 assert rec['corrected_raw_equals_original']==(rec['corrected_sha256']==rec['source_original_sha256'])
 assert rec['corrected_decoded_equals_original']==(rec['corrected_uncompressed_sha256']==rec['original_uncompressed_sha256'])
assert sum(x['status']=='corrected' for x in d['identities'])==26
assert sum(x.get('corrected_raw_equals_original',False) for x in d['identities'])==13
assert sum(x.get('corrected_decoded_equals_original',False) for x in d['identities'])==19
# Contextual privacy scan. Agent IDs and fixed regex examples are classified
# separately from actual local paths; matches are never printed as values.
slash=b'/'
roots=slash+rb'(?:home|users|tmp|root|mnt|workspace|workspaces|private|var/(?:tmp|folders))/[^\s"\x27<>]+'
windows=rb'[A-Za-z]:[\\/]+(?:Users|Documents and Settings)[\\/]+[^\s"\x27<>]+'
path_rx=re.compile(rb'(?:(?<![A-Za-z0-9_])|\\[nrt])('+roots+rb'|'+windows+rb')',re.I)
secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |DSA |OPENSSH |EC |ENCRYPTED )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{32,}|AIza[0-9A-Za-z_-]{35}|glpat-[0-9A-Za-z_-]{20,}|npm_[0-9A-Za-z]{30,}|eyJ[A-Za-z0-9_-]{18,}\.[A-Za-z0-9_-]{15,}\.[A-Za-z0-9_-]{15,}|(?:Bearer\s+)[A-Za-z0-9._~+/-]{24,}',re.I)
agents={slash+b'root/'+x.encode() for x in ('hair_ops','rig_identity','mhr_character','camera_targeting','rig_garments','native_reports','lens12_frontier')}
users={u.pw_name.lower() for u in pwd.getpwall()}|{p.name.lower() for p in Path('/'+'home').iterdir() if p.is_dir()}
userpat=re.compile(slash+rb'(?:home|users)/(?:'+b'|'.join(re.escape(u.encode()) for u in users if len(u)>=2)+rb')(?:/|\\/|["\x27\s]|$)',re.I)
privacy_counts=collections.Counter();issues=[]
for f in sorted(P.rglob('*')):
 if not f.is_file():continue
 raw=f.read_bytes()
 if raw.startswith(b'\x1f\x8b'):raw=gzip.decompress(raw);privacy_counts['gzip_decoded']+=1
 raw=re.sub(rb'(?i)\\u002f',slash,raw.replace(b'\\/',slash));privacy_counts['files']+=1;relative=str(f.relative_to(P))
 for z in path_rx.finditer(raw):
  token=z.group(1);prefix=raw[max(0,z.start()-130):z.start()].lower()
  if token.rstrip(b'.,;:!?') in agents and any(w in prefix for w in (b'owner',b'agent',b'author',b'reviewer',b'attribution',b'subagent',b'separate',b'checked by')):privacy_counts['agent_identity']+=1
  elif token.rstrip(b'.,;:!?')==slash+b'users/dependencies' and any(w in prefix for w in (b'pattern',b'capture == b',b'definitions')):privacy_counts['semantic_example']+=1
  elif (token==slash+b'tmp/726-' and b'capture.startswith' in prefix) or (token.startswith(slash+b'tmp/726-[^') and relative in ('original-cp36-review/verify-observed.py','original-cp36-review-correction/verify-observed-corrected.py')):privacy_counts['regex_literal']+=1
  else:issues.append((relative,'path',H(token)))
 for rx,kind in ((secret,'secret'),(userpat,'local_account')):
  for z in rx.finditer(raw):issues.append((relative,kind,H(z.group())))
assert not issues,('privacy candidates',issues[:10])
print('Packet manifest, 27 identity records, 310 locations, preserved prior review/correction and contextual privacy scan verified; private values were not printed.')
if not all(roots_supplied):
 print('Pass all four observed publication roots for archive/old-public checks; add the retained local mapping for original-byte checks.')
 sys.exit(0)
roots={n:args.publication_root for n in range(1,36)};roots.update({36:args.stage36_root,37:args.stage37_root,38:args.stage38_root})
indexes={}
for ref in q['indexes']:
 raw=(roots[ref['checkpoint']]/ref['index_file']).read_bytes();assert (H(raw),len(raw))==(ref['sha256'],ref['bytes']);indexes[ref['checkpoint']]=json.loads(raw)
archives=collections.defaultdict(dict)
for key,rr in source.items():
 cp,archive,chain=key;x=rr[0];bundle=next(z for z in indexes[cp]['bundles'] if z['archive']==archive);assert bundle['archive_sha256']==x['archive_sha256']
 node=next(z for z in bundle['members'] if z['path']==chain[0])
 for name in chain[1:]:node=next(z for z in node['members'] if z['path']==name)
 assert (node['original_sha256'],node['published_sha256'])==(x['source_original_sha256'],x['public_reconstructed_sha256'])
 archives[(cp,archive,x['archive_sha256'])][chain]=(rr,node)
oldby={};actual=0
for (cp,archive,digest),targets in sorted(archives.items()):
 p=roots[cp]/archive;assert hashfile(p)==digest;top=collections.defaultdict(dict)
 for chain,val in targets.items():top[chain[0]][chain[1:]]=val
 with tarfile.open(p,'r:gz') as t:
  for first,tails in top.items():
   z=t.getmember(first);assert z.isfile();first_raw=t.extractfile(z).read()
   for tail,(rr,node) in tails.items():
    raw=child(first_raw,tail);x=rr[0];assert H(raw)==x['public_reconstructed_sha256']
    if 'published_bytes' in node:assert len(raw)==node['published_bytes']
    pair=(x['source_original_sha256'],x['public_reconstructed_sha256']);oldby.setdefault(pair,raw);actual+=1
assert len(archives)==180 and actual==310 and len(oldby)==27
for pair,raw in oldby.items():
 rec=pairs[pair]
 if rec['status']!='corrected':continue
 b=gzip.decompress(raw) if rec['encoding']=='gzip' else raw;assert H(b)==rec['old_public_uncompressed_sha256'];edits=[]
 for c in rec['corrections']:
  pos=c['public_offset_0based'];marker=c['public_placeholder'].encode();context=c['non_path_context_including_replacement'].encode();n=c['leading_context_bytes'];assert b[pos:pos+len(marker)]==marker and b[pos-n:pos]==context[:n];edits.append((pos,pos+len(marker),context[n:]))
 for start,end,value in sorted(edits,reverse=True):b=b[:start]+value+b[end:]
 assert H(b)==rec['corrected_uncompressed_sha256'];blob=(P/rec['corrected_member']).read_bytes();assert b==(gzip.decompress(blob) if rec['encoding']=='gzip' else blob)
print('Verified 38 observed indexes, 180 archive hashes, 310 old-public member locations and all 26 byte-level reconstruction recipes.')
if not args.private_inventory:
 print('Original member identities require the optional retained local-only inventory.')
 sys.exit(0)
inv=json.loads(args.private_inventory.read_bytes());oldmap=inv['exact_original_paths_by_sha256'];recs={(v['checkpoint'],v['archive'],v['member']):v for v in inv['changed_top_level_records']};tested={}
for (cp,archive,chain),rr in source.items():
 reg=recs[(cp,archive,chain[0])];key=(reg['original_sha256'],chain[1:])
 if key not in tested:
  raw=Path(oldmap[reg['original_sha256']]).read_bytes();assert (H(raw),len(raw))==(reg['original_sha256'],reg['original_bytes']);original=child(raw,chain[1:]);tested[key]=(H(original),len(original))
 else:original=None
 x=rr[0];assert tested[key][0]==x['source_original_sha256']
 if original is None:
  raw=Path(oldmap[reg['original_sha256']]).read_bytes();assert H(raw)==reg['original_sha256'];original=child(raw,chain[1:])
 rec=pairs[(x['source_original_sha256'],x['public_reconstructed_sha256'])];plain=gzip.decompress(original) if rec['encoding']=='gzip' else original;assert H(plain)==rec['original_uncompressed_sha256']
 if rec['status']=='corrected':
  edits={v['original_offset_0based']:v for v in rec['corrections']}
  for v in rr:
   c=edits[v['original_byte_offset_0based']];off=c['original_offset_0based'];context=c['non_path_context_including_replacement'].encode();lead=c['leading_context_bytes'];fragment=context[lead:]
   assert plain[off-lead:off]==context[:lead] and plain[off:off+len(fragment)]==fragment and plain.count(b'\n',0,off)+1==c['original_line_1based']
print('Verified retained original top-member hashes and each indexed original leaf, offset and non-path replacement without exposing local path values.')
