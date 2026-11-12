#!/usr/bin/env python3
"""Bounded read-only scan of staged stored/gzip/tar evidence; no private values printed."""
import argparse,collections,gzip,hashlib,io,json,pathlib,re,tarfile
ap=argparse.ArgumentParser();ap.add_argument('--stage',required=True,type=pathlib.Path);a=ap.parse_args();R=a.stage;H=lambda b:hashlib.sha256(b).hexdigest();slash=b'/'
pol=json.loads((pathlib.Path(__file__).resolve().parent/'PRIVACY-POLICY.json').read_bytes());agents={slash+b'root/'+x.encode() for x in pol['agent_identifiers']};semantic=set(pol['semantic_fragment_sha256']);regexsafe=set(pol['known_literal_regex_sha256']);root=slash+rb'(?:home|users|tmp|root|mnt|workspace|workspaces|private|var/(?:tmp|folders))/[^\s"\x27<>\x00\\]+';win=rb'[A-Za-z]:[\\/]+(?:Users|Documents and Settings)[\\/]+[^\s"\x27<>\x00]+';pathrx=re.compile(rb'(?:(?<![A-Za-z0-9_])|\\[nrt])('+root+rb'|'+win+rb')',re.I);secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |DSA |OPENSSH |EC |ENCRYPTED )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{32,}|AIza[0-9A-Za-z_-]{35}|glpat-[0-9A-Za-z_-]{20,}|npm_[0-9A-Za-z]{30,}|eyJ[A-Za-z0-9_-]{18,}\.[A-Za-z0-9_-]{15,}\.[A-Za-z0-9_-]{15,}|(?:Bearer\s+)[A-Za-z0-9._~+/-]{24,}|https?://[^\s/@:]+:[^\s/@]+@',re.I);email=re.compile(rb'[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}');home=pathlib.Path('/'+'home');users=[x.name.encode() for x in home.iterdir() if x.is_dir() and len(x.name)>=6] if home.exists() else [];stats=collections.Counter();unknown=[];email_locations=[];unsupported=[]
def scan(name,data):
 norm=re.sub(rb'(?i)\\u002f',slash,data.replace(b'\\/',slash));stats['views']+=1;stats['bytes']+=len(norm)
 for z in pathrx.finditer(norm):
  token=z.group(1);bare=token.rstrip(b'.,;:!?`');pre=norm[max(0,z.start()-180):z.start()].lower()
  if bare in agents:stats['agent-identifier']+=1;continue
  if H(token) in semantic or H(bare) in semantic:stats['semantic-exemption']+=1;continue
  if H(token) in regexsafe:stats['regex-exemption']+=1;continue
  if bare==slash+b'tmp/726' and b'paths' in norm[z.end():z.end()+25].lower():stats['literal-example']+=1;continue
  if bare.startswith(slash+b'root/') and pre.endswith(b'..') and b'"path"' in pre:stats['root-relative']+=1;continue
  unknown.append({'file':name,'kind':'path','length':len(token)})
 for z in secret.finditer(norm):unknown.append({'file':name,'kind':'secret-pattern','length':len(z.group())})
 for user in users:
  for z in re.finditer(re.escape(user),norm,re.I):unknown.append({'file':name,'kind':'local-account-name','length':len(z.group())})
 matches=email.findall(norm)
 if matches:email_locations.append({'file':name,'count':len(matches),'is_git_commit':('.commit' in name)})
def visit(name,raw,depth):
 if depth>5:unsupported.append({'file':name,'reason':'depth limit'});return
 scan(name+'#stored',raw)
 if raw.startswith(b'\x1f\x8b'):
  plain=gzip.decompress(raw);stats['gzip_streams']+=1
  if len(plain)>128*1024*1024:unsupported.append({'file':name,'reason':'decoded size limit'});return
  try:
   with tarfile.open(fileobj=io.BytesIO(plain),mode='r:') as t:
    ms=t.getmembers()
    if not ms:raise tarfile.ReadError('empty')
    stats['tar_archives']+=1;scan(name+'#tar-decoded',plain)
    for x in ms:
     stats['tar_entries']+=1;scan(name+'!member-name',x.name.encode())
     if x.isfile():visit(name+'!'+x.name,t.extractfile(x).read(),depth+1)
    return
  except tarfile.ReadError:pass
  scan(name+'#decoded',plain)
 elif raw.startswith((b'PK\x03\x04',b'\xfd7zXZ\x00',b'\x28\xb5\x2f\xfd')):unsupported.append({'file':name,'reason':'other archive encoding'})
for f in sorted(R.rglob('*')):
 if f.is_file():visit(str(f.relative_to(R)),f.read_bytes(),0)
print(json.dumps({'schema':'726-followup-independent-privacy-scan-v1','counts':dict(stats),'candidate_count':len(unknown),'candidates':unknown,'email_locations':email_locations,'unsupported_count':len(unsupported),'unsupported':unsupported,'limits':'Pattern- and host-bounded; stored and gzip/tar views, depth at most five and 128 MiB decoded payload; unknown encodings, alternative local roots, credential formats and semantic privacy are not ruled out. Exact Git author/committer fields remain part of preserved objects.'},indent=2))
