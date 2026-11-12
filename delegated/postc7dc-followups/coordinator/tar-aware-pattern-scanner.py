import gzip,hashlib,io,os,pwd,re,subprocess,tarfile
from pathlib import Path
import sys
R=Path(sys.argv[1]).resolve()
files=sorted(p.relative_to(R).as_posix() for p in R.rglob('*') if p.is_file() and not p.is_symlink())
home=pwd.getpwuid(os.getuid()).pw_dir.encode(); exact=re.compile(re.escape(home)+rb'(?:/|["\'\s]|$)'); tmp=re.compile(rb'/t' + rb'mp/726-[A-Za-z0-9_-]{3,}');windows=re.compile(rb'[A-Za-z]:[\\/]+Users[\\/]+[A-Za-z0-9._-]+',re.I)
secret=re.compile(rb'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |DSA |OPENSSH |EC |ENCRYPTED )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{32,}|AIza[0-9A-Za-z_-]{35}|glpat-[0-9A-Za-z_-]{20,}|npm_[0-9A-Za-z]{30,}|eyJ[A-Za-z0-9_-]{18,}\.[A-Za-z0-9_-]{15,}\.[A-Za-z0-9_-]{15,}|Bearer\s+[A-Za-z0-9._~+/-]{24,}',re.I)
stats={'files':0,'streams':0,'tar_entries':0,'gzip_streams':0};issues=[]
def scan(body,where,depth):
 if depth>9:issues.append((where,'depth'));return
 stats['streams']+=1
 if body.startswith(b'\x1f\x8b'):
  try:body=gzip.decompress(body);stats['gzip_streams']+=1
  except Exception:issues.append((where,'gzip decode'));return
 if len(body)>262 and body[257:262]==b'ustar':
  try:
   with tarfile.open(fileobj=io.BytesIO(body),mode='r:') as t:
    for m in t.getmembers():
     stats['tar_entries']+=1
     if not (m.isfile() or m.isdir()) or m.name.startswith('/') or '..' in Path(m.name).parts:issues.append((where,'unsafe tar entry'));continue
     meta=str((m.name,m.uname,m.gname,m.pax_headers)).encode()
     for pat,kind in ((exact,'local-home-in-tar-metadata'),(tmp,'tmp-in-tar-metadata'),(windows,'windows-in-tar-metadata'),(secret,'credential-in-tar-metadata')):
      if pat.search(meta):issues.append((where,kind))
     if m.isfile():scan(t.extractfile(m).read(),where+'!'+m.name,depth+1)
  except Exception as e:issues.append((where,'tar parse '+type(e).__name__))
  return
 norm=re.sub(rb'(?i)\\u002f',b'/',body.replace(b'\\/',b'/'))
 for pat,kind in ((exact,'local-home'),(tmp,'local-temp'),(windows,'windows-path'),(secret,'credential')):
  m=pat.search(norm)
  if m:issues.append((where,kind,hashlib.sha256(m.group()).hexdigest()))
for f in files:
 p=R/f
 if p.is_file():stats['files']+=1;scan(p.read_bytes(),f,0)
print('Checked',stats,'issues',len(issues))
for x in issues[:30]:print(x)
if issues:raise SystemExit(1)
