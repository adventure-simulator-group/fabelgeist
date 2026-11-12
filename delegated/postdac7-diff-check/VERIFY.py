#!/usr/bin/env python3
import gzip,hashlib,json,re,subprocess,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
p=Path(__file__).resolve().parent;root=p.parent.parent;H=lambda b:hashlib.sha256(b).hexdigest();r=json.loads((p/"RECORD.json").read_bytes());assert r["schema"]=="726-postdac7-diff-check-v1" and r["strict_exit"]==2 and r["remainder_exit"]==0 and r["strict_stderr_bytes"]==r["remainder_stdout_bytes"]==r["remainder_stderr_bytes"]==0
c=(p/"STRICT-STDOUT.txt.gz").read_bytes();b=gzip.decompress(c);assert H(c)==r["strict_archive_sha256"] and len(b)==r["strict_stdout_bytes"] and H(b)==r["strict_stdout_sha256"]
violations=[]
for line in b.decode().splitlines():
 m=re.match(r"^(.*?):(\d+): (?:trailing whitespace|space before tab in indent|new blank line at EOF)\.",line)
 if m:violations.append(m.group(1))
assert len(violations)==r["classified_violations"]==333 and set(violations)=={x["path"] for x in r["excluded_exact_files"]} and len(r["excluded_exact_files"])==25
cat=json.loads((root/"delegated/postdac7-followups/CATALOG.json").read_bytes());by={x["path"]:x for x in cat["bundle_files"]}
for row in r["excluded_exact_files"]:
 n=row["path"];assert n.startswith("delegated/postdac7-followups/");rel=n.removeprefix("delegated/postdac7-followups/");raw=(root/n).read_bytes();assert rel in by and H(raw)==row["sha256"]==by[rel]["sha256"] and len(raw)==row["bytes"] and subprocess.check_output(["git","-C",str(root),"rev-parse",":"+n],text=True).strip()==row["git_blob"] and subprocess.check_output(["git","-C",str(root),"show",":"+n])==raw
print("Verified recorded strict whitespace failure and passing remainder; exact evidence bytes remain unchanged.")
