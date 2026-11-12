#!/usr/bin/env python3
import hashlib,json,pathlib,sys
p=pathlib.Path(__file__).resolve().parent;m=json.loads((p/"MANIFEST.json").read_bytes())
for x in m["files"]:
 b=(p/x["path"]).read_bytes();assert (hashlib.sha256(b).hexdigest(),len(b))==(x["sha256"],x["bytes"])
q=json.loads((p/"QUALIFICATION.json").read_bytes());assert len(q["correct_sha256"])==64 and q["correct_sha256"]!=q["handoff_typo"]
if len(sys.argv)>1:assert hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest()==q["correct_sha256"]
print("PASS: qualification record; supply frozen receipt manifest to recheck its identity")
