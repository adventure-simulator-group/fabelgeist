#!/usr/bin/env python3
"""Verify exact public packet transports; optionally restore only to a new directory."""
import argparse,hashlib,io,json,os,stat,sys,tarfile
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError("Unoptimized interpreter required")
P=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument("--restore-root",type=Path);args=ap.parse_args();m=json.loads((P/"MANIFEST.json").read_bytes());assert m["schema"]=="726-postdac7-public-review-transports-v1"
if args.restore_root:
 root=args.restore_root;assert not root.exists() and not root.is_symlink();root.mkdir(parents=False,mode=0o700)
count=0
for row in m["objects"]:
 data=(P/row["archive"]).read_bytes();assert len(data)==row["bytes"] and hashlib.sha256(data).hexdigest()==row["sha256"];expected={x["path"]:x for x in row["members"]};assert len(expected)==len(row["members"]);seen=set();dirs=[]
 if args.restore_root:(root/row["label"]).mkdir(mode=0o700)
 with tarfile.open(fileobj=io.BytesIO(data),mode="r:gz") as tar:
  for item in tar:
   name=PurePosixPath(item.name);assert not name.is_absolute() and ".." not in name.parts and str(name) in expected and str(name) not in seen;seen.add(str(name));exp=expected[str(name)];assert not item.issym() and not item.islnk() and stat.S_IMODE(item.mode)==int(exp["mode"],8)
   if item.isdir():
    assert exp["kind"]=="directory"
    if args.restore_root:
     dest=root/row["label"]/str(name);dest.mkdir(parents=True,exist_ok=True);dirs.append((dest,item.mode))
   else:
    assert item.isfile() and exp["kind"]=="file";raw=tar.extractfile(item).read();assert len(raw)==exp["bytes"] and hashlib.sha256(raw).hexdigest()==exp["sha256"];count+=1
    if args.restore_root:
     dest=root/row["label"]/str(name);dest.parent.mkdir(parents=True,exist_ok=True)
     with dest.open("xb") as f:f.write(raw)
     os.chmod(dest,item.mode)
  assert seen==set(expected)
 if args.restore_root:
  for dest,mode in reversed(dirs):os.chmod(dest,mode)
print("Verified",len(m["objects"]),"public review archives and",count,"file members; restricted originals and commit bindings remain separate.")
