#!/usr/bin/env python3
"""Restore the original ciphertext packet into an absent destination.

Leaves any partial output in place on failure. Does not decrypt or authenticate.
"""
import argparse,hashlib,os,pathlib,stat,sys
if sys.flags.optimize:raise RuntimeError("unoptimized interpreter required")
a=argparse.ArgumentParser(description="Restore a fresh original ciphertext packet after no-key stream verification.");a.add_argument("--strict-modes",action="store_true");a.add_argument("destination");z=a.parse_args()
root=pathlib.Path(os.path.abspath(__file__)).parent;cur=pathlib.Path(root.anchor)
for part in root.parts[1:]:
 cur=cur/part;st=os.lstat(cur)
 if not stat.S_ISDIR(st.st_mode) or stat.S_ISLNK(st.st_mode):raise ValueError("transport path contains symlink or non-directory")
q=root/"VERIFY-SPLIT.py";fd=os.open(q,os.O_RDONLY|os.O_NOFOLLOW)
try:
 st=os.fstat(fd)
 if not stat.S_ISREG(st.st_mode) or st.st_nlink!=1:raise ValueError("verifier must be a single-link regular file")
 h=hashlib.sha256()
 while True:
  b=os.read(fd,1048576)
  if not b:break
  h.update(b)
 if h.hexdigest()!="65deb4852a0f6b412a3f34c6999787b5eb732485cc80784b173b777c106f8d61":raise ValueError("verifier identity mismatch")
finally:os.close(fd)
os.execv(sys.executable,[sys.executable,"-B",str(q)]+(["--strict-modes"] if z.strict_modes else [])+["--destination",z.destination])
