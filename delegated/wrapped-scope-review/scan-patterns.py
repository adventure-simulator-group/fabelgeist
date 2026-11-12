#!/usr/bin/env python3
"""Bounded flat-file privacy candidate scan; reports counts, not values."""
import argparse, json, os, pwd, re
from pathlib import Path
ap=argparse.ArgumentParser(); ap.add_argument("target",type=Path); a=ap.parse_args()
p=a.target
patterns={
 "local_home":re.compile(re.escape(pwd.getpwuid(os.getuid()).pw_dir.encode())+rb"(?:/|[\"\x27\s]|$)"),
 "temporary_path":re.compile(rb"/t"+rb"mp/726-[A-Za-z0-9_-]{3,}"),
 "windows_user":re.compile(rb"[A-Za-z]:[\\/]+Users[\\/]+[A-Za-z0-9._-]+",re.I),
 "email":re.compile(rb"(?<![A-Za-z0-9_.+%-])[A-Za-z0-9_.+%-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}(?![A-Za-z0-9_.-])"),
 "credential":re.compile(rb"gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |DSA |OPENSSH |EC |ENCRYPTED )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{32,}|AIza[0-9A-Za-z_-]{35}|glpat-[0-9A-Za-z_-]{20,}|npm_[A-Za-z0-9]{30,}|eyJ[A-Za-z0-9_-]{18,}\.[A-Za-z0-9_-]{15,}\.[A-Za-z0-9_-]{15,}|Bearer\s+[A-Za-z0-9._~+/-]{24,}",re.I),
}
files=[x for x in p.rglob("*") if x.is_file() and not x.is_symlink()]; counts={k:0 for k in patterns}
for f in files:
 data=re.sub(rb"(?i)\\u002f",b"/",f.read_bytes().replace(b"\\/",b"/"))
 for k,pat in patterns.items():counts[k]+=len(pat.findall(data))
print(json.dumps({"files":len(files),"views":len(files),"counts":counts,"qualification":"Flat regular-file bytes with escaped slashes normalized; not a complete privacy guarantee. Symlinks, Git history and nested archive contents are not scanned."},indent=2))
