#!/usr/bin/env python3
"""Independent CP39 evidence inspection. Reads inputs only; emits non-sensitive summary JSON."""
import argparse, collections, gzip, hashlib, io, json, os, pathlib, pwd, re, stat, tarfile, tomllib
PARSER=argparse.ArgumentParser()
for n in ('public','local','source2623','source2625','source2626','prior'): PARSER.add_argument('--'+n, required=True, type=pathlib.Path)
a=PARSER.parse_args(); p=a.public; loc=a.local
H=lambda b:hashlib.sha256(b).hexdigest()
def read(f):
 assert f.is_file() and not f.is_symlink(), 'missing/nonregular/symlink: '+str(f)
 return f.read_bytes()
def dec(raw):return gzip.decompress(raw) if raw.startswith(b'\x1f\x8b') else raw
def sha_file(f):
 h=hashlib.sha256()
 with f.open('rb') as q:
  for v in iter(lambda:q.read(1048576),b''):h.update(v)
 return h.hexdigest()
def load(f):return json.loads(read(f))
def verify_manifest(root,filename,expected):
 raw=read(root/filename);assert H(raw)==expected; obj=json.loads(raw)
 actual={str(x.relative_to(root)) for x in root.rglob('*') if x.is_file() or x.is_symlink()}-{filename}
 assert actual==set(obj['files']), (len(actual),len(obj['files']),actual-set(obj['files']),set(obj['files'])-actual)
 for name,row in obj['files'].items():
  f=root/name; assert not f.is_symlink() and f.is_file(); assert (f.stat().st_size,sha_file(f))==(row['bytes'],row['sha256']),name
 return len(actual)
public_n=verify_manifest(p,'MANIFEST.json','f8f2bb8d08a3f3fcc404bc941ace2fde02bc103ca31c22fd6135b91e9b5abf08')
local_n=verify_manifest(loc,'LOCAL-MANIFEST.json','110e7d3c557bc8b2d784119637bb3b4021829a3db13e9a2d5361cb569dc42e18')
index=load(p/'EVIDENCE-IDENTITIES.json'); supp=load(p/'SUPPLEMENTAL-PRIOR-MEMBERS.json'); prior_cmp=load(p/'PRIOR-ARCHIVE-MANIFEST-COMPARISON.json'); boundary=load(p/'HISTORICAL-BOUNDARIES.json'); source_roots={'2623-2624':a.source2623,'2625':a.source2625,'2626-2628':a.source2626}; members=index['members']; assert len(members)==333 and len(supp['members'])==5
for src in index['source_packets']:
 root=source_roots[src['packet']]; mf=read(root/src['source_manifest']); assert H(mf)==src['source_manifest_sha256']; record=json.loads(mf)['files']; declared={x['path']:(x['sha256'],x['bytes']) for x in record} if isinstance(record,list) else {k:(v['sha256'],v['bytes']) for k,v in record.items()}; assert len(declared)==src['source_manifest_member_count']
 subset={x['path']:x for x in members if x['packet']==src['packet']}; assert len(subset)==src['source_file_count_including_manifest_and_extras']
 assert set(subset)==set(declared)|{src['source_manifest']}|set(src['supplemental_original_members'])
 for k,(sha,n) in declared.items(): assert (subset[k]['original_stored_sha256'],subset[k]['original_stored_bytes'])==(sha,n) and subset[k]['source_manifest_bound']

# Verify each original, public, and original-to-public transition independently.
reconstructed=0; substitutions=0; labels={}; categories=collections.Counter(); original_objects=set(); git_oids=[]; git_commit_oids=[]
absolute=re.compile(rb'^(?:/(?:home|users|tmp|root|mnt|workspace|workspaces|private|var/(?:tmp|folders))/|[A-Za-z]:[\\/]+(?:Users|Documents and Settings)[\\/]+)',re.I)
def check_reconstruction(x,original):
 global reconstructed,substitutions
 raw=read(p/x['public_path']); assert (H(raw),len(raw))==(x['public_stored_sha256'],x['public_stored_bytes']); plain=dec(raw)
 assert (H(plain),len(plain))==(x['public_decoded_sha256'],x['public_decoded_bytes'])
 assert (H(original),len(original))==(x['original_stored_sha256'],x['original_stored_bytes']); orig=dec(original)
 assert (H(orig),len(orig))==(x['original_decoded_sha256'],x['original_decoded_bytes'])
 red=x['redactions']
 if not red:
  assert x['status']=='exact-safe-copy' and raw==original
 else:
  assert x['status']=='privacy-reconstructed'; reconstructed+=1; original_objects.add(H(original))
  retained=read(loc/'changed-originals'/H(original)); assert retained==original
  pieces=[];last=0
  for z in red:
   start=z['original_decoded_offset_0based'];end=start+z['original_decoded_bytes'];assert start>=last and end<=len(orig)
   token=orig[start:end]; norm=re.sub(rb'(?i)\\u002f',b'/',token.replace(b'\\/',b'/'))
   category='absolute-root' if absolute.match(norm) else 'other';categories[category]+=1
   assert not any(c in token for c in (b'\n',b'\r',b'"',b"'",b'<',b'>',b' ')) and not token.rstrip().endswith((b'.',b',',b';',b':',b'!',b'?'))
   key=z['public_placeholder'];assert key not in labels or labels[key]==norm;labels[key]=norm
   pieces.extend([orig[last:start],key.encode()]);last=end; substitutions+=1
  pieces.append(orig[last:]); assert b''.join(pieces)==plain, x['public_path']
 if x.get('git_object'):
  g=x['git_object'];oid=hashlib.sha1(g['type'].encode()+b' '+str(len(plain)).encode()+b'\x00'+plain).hexdigest();assert oid==g['git_sha1'] and raw==original
  git_oids.append(oid)
  if g['type']=='commit':
   assert re.search(rb'^author [^\n]+ <[^\n<>]+> [^\n]+$',plain,re.M) and re.search(rb'^committer [^\n]+ <[^\n<>]+> [^\n]+$',plain,re.M)
   git_commit_oids.append(oid)
for x in members:check_reconstruction(x,read(source_roots[x['packet']]/x['path']))
for x in supp['members']:check_reconstruction(x,read(a.prior/x['prior_member']))
assert (reconstructed,substitutions,len(labels),len(original_objects))==(38,125,56,36)
assert len(git_oids)==87 and len(git_commit_oids)==10
local=load(loc/'CHANGED-ORIGINALS-LOCAL.json'); assert {x['sha256'] for x in local['objects']}==original_objects and len(local['objects'])==36
for x in local['objects']:
 raw=read(loc/x['local_content_addressed_relative_path']);assert (H(raw),len(raw))==(x['sha256'],x['bytes'])

# Recalculate every prior-manifest binding against the prior original and tar bytes.
bind=load(p/'OMITTED-MANIFEST-BINDING.json'); prior_mf=read(a.prior/'MANIFEST.json'); assert H(prior_mf)==bind['omitted_source_member_sha256']==prior_cmp['prior_manifest_sha256'];assert prior_mf==read(p/bind['retained_public_member'])
prior=json.loads(prior_mf); orig={x['path']:x for x in prior['files']};assert len(orig)==83
for name,row in orig.items():
 raw=read(a.prior/name);assert (H(raw),len(raw))==(row['sha256'],row['bytes'])
archive=read(p/bind['prior_published_archive']);assert H(archive)==bind['prior_published_archive_sha256']==prior_cmp['prior_published_archive_sha256'];same=0;different=[]
with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as t:
 assert t.extractfile(bind['inner_member']).read()==prior_mf
 for row in prior_cmp['members']:
  name=row['prior_member']; item=orig[name]; pub=t.extractfile(name).read(); original=read(a.prior/name); equal=pub==original
  assert (item['sha256'],item['bytes'])==(row['prior_manifest_original_stored_sha256'],row['prior_manifest_original_stored_bytes'])
  assert (H(pub),len(pub),H(dec(pub)),equal)==(row['prior_published_stored_sha256'],row['prior_published_stored_bytes'],row['prior_published_decoded_sha256'],row['original_equals_prior_published'])
  if equal:same+=1
  else:
   different.append(name); s=next(z for z in supp['members'] if z['prior_member']==name);assert s['original_stored_sha256']==H(original) and s['public_path']==row['supplemental_public_path'] and s['public_stored_sha256']==row['supplemental_public_stored_sha256']
assert same==78 and len(different)==5

# Verify each archive member and nested stored/decoded identities, without extracting.
by_member={(x['packet'],x['path']):x for x in members}; archive_entries=0; archive_gzip=0
for z in index['archives']:
 m=by_member[(z['packet'],z['source_path'])];raw=read(p/m['public_path']);assert H(raw)==z['original_archive_sha256']==z['public_archive_sha256']
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
  ms=t.getmembers();assert len(ms)==len(z['entries'])
  for member,row in zip(ms,z['entries']):
   assert member.name==row['member'] and not member.name.startswith('/') and '..' not in pathlib.PurePosixPath(member.name).parts and not member.uname and not member.gname
   assert (member.isdir() or member.isfile()) and row['kind']==('directory' if member.isdir() else 'regular')
   if member.isfile():
    b=t.extractfile(member).read();u=dec(b);assert (H(b),len(b),H(u),len(u))==(row['original_stored_sha256'],row['original_bytes'],row['original_decoded_sha256'],row['decoded_bytes']);assert H(b)==row['public_stored_sha256'] and H(u)==row['public_decoded_sha256'];archive_gzip+=row['encoding']=='gzip'
   archive_entries+=1
assert len(index['archives'])==3 and archive_entries==184 and archive_gzip==33

# Compare the locked native package archive identities to each separately pinned lock view.
native=load(p/'NATIVE-PACKAGE-PROVENANCE.json');packages={(x['name'],x['version']):x for x in native['package_archives']}
for x in packages.values():assert sha_file(p/x['path'])==x['sha256']
for view in native['lock_views']:
 raw=read(p/'bundles/2626-2628'/view['view']); plain=dec(raw);assert (H(raw),H(plain))==(view['stored_sha256'],view['decoded_sha256'])
 lock=tomllib.loads(plain.decode())
 for package in view['matched_packages']:
  key=(package['name'],package['version']); rows=[z for z in lock['package'] if (z['name'],z['version'])==key];assert len(rows)==1
  assert rows[0]['checksum']==package['checksum']==packages[key]['sha256'] and rows[0]['source']==package['registry_source']

# Bind 059 original decoded identities and Git blobs to the supplied transport manifest.
transport=gzip.decompress(read(p/'bundles/2623-2624/history/original-transport-manifest.json.gz')); trans=json.loads(transport); asserts={'queue':('workflow-20261030/CLOSURE-QUEUE-100.json','2e1f4430edb375adff24b6b46d434ac58bf5fd18c0d9dda4cf92fe215e5cfe08'),'ledger':('evidence/continuation-segovb/ledger.json','3933e8654ea22a7273e3218d5bc329d052968f20c1512a803de9b002038a6aac'),'registry':('evidence/audit05/original-scope-072.json','d1c798df997434ae8e89e433237f798de5c87a38d634f3a7d2f1b0f8af9b1f0f')}; seen059={}
for x in boundary['original_059_identities']['transport_manifest']:
 y=by_member[(x['packet'],x['path'])];assert all(x[k]==y[k] for k in ('encoding','original_stored_sha256','original_decoded_sha256','public_stored_sha256','public_decoded_sha256','status'));assert H(transport)==x['original_decoded_sha256']==x['public_decoded_sha256']
for label,(path,sha) in asserts.items():
 entry=next(x for x in trans['entries'] if x['path']==path); assert entry['sha256']==sha; refs=boundary['original_059_identities'][label]; assert len(refs)==2
 for x in refs:
  y=by_member[(x['packet'],x['path'])]; assert all(x[k]==y[k] for k in ('encoding','original_stored_sha256','original_decoded_sha256','public_stored_sha256','public_decoded_sha256','status'))
  original=read(source_roots[x['packet']]/x['path']); decoded=dec(original);assert H(decoded)==entry['sha256'] and len(decoded)==entry['bytes'] and hashlib.sha1(b'blob '+str(len(decoded)).encode()+b'\0'+decoded).hexdigest()==entry['git_blob']
 seen059[label]={'transport_path':path,'original_decoded_sha256':sha,'original_git_blob':entry['git_blob'],'original_stored_sha256':refs[0]['original_stored_sha256'],'public_stored_sha256':refs[0]['public_stored_sha256'],'public_decoded_sha256':refs[0]['public_decoded_sha256'],'status':refs[0]['status'],'occurrences':len(refs)}
 all_top=[x for x in members if sha in (x['original_stored_sha256'],x['original_decoded_sha256'])]
 all_inner=[x for ar in index['archives'] for x in ar['entries'] if sha in (x.get('original_stored_sha256'),x.get('original_decoded_sha256'))]
 assert len(all_top)==2 and not all_inner and all(x['original_decoded_sha256']==sha and x['original_stored_sha256']!=sha for x in all_top)

# Structural comparison of the original and publicly reconstructed 059 JSON.
json_diff={}
for label,expected in [('queue',12),('ledger',14)]:
 ref=boundary['original_059_identities'][label][0]; original=dec(read(source_roots[ref['packet']]/ref['path']));public=dec(read(p/by_member[(ref['packet'],ref['path'])]['public_path']))
 stack=[(json.loads(original),json.loads(public))];changes=collections.Counter()
 while stack:
  x,y=stack.pop()
  if type(x)!=type(y):changes['type']+=1
  elif isinstance(x,dict):
   if set(x)!=set(y):changes['key-set']+=1
   stack.extend((x[k],y[k]) for k in set(x)&set(y))
  elif isinstance(x,list):
   if len(x)!=len(y):changes['list-length']+=1
   stack.extend(zip(x,y))
  elif x!=y:changes['value-'+type(x).__name__]+=1
 assert changes=={'value-str':expected};json_diff[label]=dict(changes)

# Independently scan stored and decoded public bytes, including nested tar/crate members.
policy=load(p/'PRIVACY-POLICY.json'); semantic=set(policy['semantic_fragment_sha256']); regexsafe=set(policy['known_literal_regex_sha256']); agents={b'/root/'+x.encode() for x in policy['agent_identifiers']}
slash=b'/'; root=slash+rb'(?:home|users|tmp|root|mnt|workspace|workspaces|private|var/(?:tmp|folders))/[^\s"\x27<>\x00\\]+';win=rb'[A-Za-z]:[\\/]+(?:Users|Documents and Settings)[\\/]+[^\s"\x27<>\x00]+'
pathrx=re.compile(rb'(?:(?<![A-Za-z0-9_])|\\[nrt])('+root+rb'|'+win+rb')',re.I)
users={u.pw_name.lower() for u in pwd.getpwall()}|{x.name.lower() for x in pathlib.Path('/home').iterdir() if x.is_dir()};urx=re.compile(slash+rb'(?:home|users)/(?:'+b'|'.join(re.escape(u.encode()) for u in users if len(u)>=2)+rb')(?:/|\\/|["\x27\s]|$)',re.I)
secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |DSA |OPENSSH |EC |ENCRYPTED )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{32,}|AIza[0-9A-Za-z_-]{35}|glpat-[0-9A-Za-z_-]{20,}|npm_[0-9A-Za-z]{30,}|eyJ[A-Za-z0-9_-]{18,}\.[A-Za-z0-9_-]{15,}\.[A-Za-z0-9_-]{15,}|(?:Bearer\s+)[A-Za-z0-9._~+/-]{24,}',re.I)
scanned=collections.Counter(); issues=[]; commit_nested=[]; account_locations=collections.Counter()
local_names=[x.name.encode() for x in pathlib.Path('/home').iterdir() if x.is_dir() and len(x.name)>=6]
name_rx=re.compile(rb'(?i)(?<![A-Za-z0-9_])(?:'+b'|'.join(re.escape(x) for x in local_names)+rb')(?![A-Za-z0-9_])') if local_names else None
def scan(name,b):
 norm=re.sub(rb'(?i)\\u002f',slash,b.replace(b'\\/',slash));scanned['payloads']+=1;scanned['bytes']+=len(norm)
 for z in pathrx.finditer(norm):
  tok=z.group(1);bare=tok.rstrip(b'.,;:!?`');prefix=norm[max(0,z.start()-180):z.start()].lower()
  if bare in agents:scanned['agent-identifiers']+=1;continue
  if H(tok) in semantic or H(bare) in semantic:scanned['policy-semantic']+=1;continue
  if H(tok) in regexsafe:scanned['policy-regex']+=1;continue
  if bare==b'/tmp/726' and b'paths' in norm[z.end():z.end()+25].lower():scanned['literal-example']+=1;continue
  if bare.startswith(b'/root/') and prefix.endswith(b'..') and b'"path"' in prefix:scanned['root-relative-reference']+=1;continue
  issues.append({'where':name,'kind':'path-pattern','sha256':H(tok)})
 for rx,kind in ((urx,'account'),(secret,'secret')):
  for z in rx.finditer(norm):issues.append({'where':name,'kind':kind,'sha256':H(z.group())})
 if name_rx:
  count=len(name_rx.findall(norm))
  if count:account_locations[name]+=count
def visit(name,raw,depth=0):
 assert depth<=3
 scan(name+'#stored',raw)
 if name.endswith(('.commit','.commit.gz')) and re.search(rb'^author [^\n]+ <[^\n<>]+>',dec(raw),re.M):commit_nested.append(name)
 if raw.startswith(b'\x1f\x8b'):
  plain=gzip.decompress(raw)
  try:
   with tarfile.open(fileobj=io.BytesIO(plain),mode='r:') as t:
    ms=t.getmembers()
    if not ms:raise tarfile.ReadError('empty')
    scanned['archives']+=1
    scan(name+'#tar-decoded',plain)
    for z in ms:
     if z.isfile():visit(name+'!'+z.name,t.extractfile(z).read(),depth+1)
    return
  except tarfile.ReadError:pass
  scan(name+'#decoded',plain)
for f in sorted(p.rglob('*')):
 if f.is_file(): visit(str(f.relative_to(p)),read(f))
# Check selected false-positive examples in original and public bytes.
for x in load(p/'FALSE-POSITIVE-PRESERVATION.json')['checks']:
 original=dec(read(source_roots[x['packet']]/x['source_path']));public=dec(read(p/x['public_path']));needle=x['known_non_path_phrase'].encode();assert original.count(needle)==x['original_phrase_occurrences']==public.count(needle)==x['public_phrase_occurrences']
result={'schema':'independent-checkpoint39-input-review-v1','public_manifest_members':public_n,'local_manifest_members':local_n,'source_members_checked_against_original_packets':len(members),'supplemental_prior_members_checked':len(supp['members']),'reconstructed_occurrences':reconstructed,'substitutions_replayed':substitutions,'substitution_prefix_categories':dict(categories),'distinct_placeholders':len(labels),'distinct_retained_originals':len(original_objects),'git_objects':len(git_oids),'top_level_git_commit_copies':len(git_commit_oids),'distinct_commit_objects':sorted(set(git_commit_oids)),'commit_object_locations_including_archive':commit_nested,'archive_count':len(index['archives']),'archive_entries':archive_entries,'nested_gzip_members':archive_gzip,'native_package_archives':len(packages),'independent_lock_views':len(native['lock_views']),'prior_manifest_members':len(orig),'prior_exact_earlier_tar':same,'prior_different_earlier_tar':len(different),'prior_different_members':different,'original_059':seen059,'original_vs_public_059_json_differences':json_diff,'bounded_privacy_scan':{'counts':dict(scanned),'unclassified_count':len(issues),'unclassified':issues,'local_account_name_occurrence_locations':dict(account_locations)},'local_original_permissions':{'root_octal':oct(stat.S_IMODE(loc.stat().st_mode)),'objects_dir_octal':oct(stat.S_IMODE((loc/'changed-originals').stat().st_mode)),'object_modes_octal':sorted({oct(stat.S_IMODE(x.stat().st_mode)) for x in (loc/'changed-originals').iterdir()})},'scope':'Evidence only; account and secret patterns and policy fingerprint exemptions are bounded, not a proof of privacy. No product code or old verifier was run.'}
print(json.dumps(result,indent=2))
