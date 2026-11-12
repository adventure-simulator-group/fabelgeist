#!/usr/bin/env python3
"""Read-only bounded decoded-text privacy check, without printing matches."""
import json,pathlib,re,sys,hashlib
from collections import Counter
D=pathlib.Path(sys.argv[1]); c=Counter(); issues=[]
# Any slash path must start at a token boundary. Embedded slash-separated
# English fragments remain visible, and are not classified as absolute paths.
path=re.compile(r'(?<![A-Za-z0-9_])/(?:home|users|tmp|root|mnt|workspace|workspaces|private|var/(?:tmp|folders))/[^\s"\'<>]+',re.I)
windows=re.compile(r'(?<![A-Za-z0-9_])[A-Za-z]:[\\/]+(?:Users|Documents and Settings)[\\/]+[^\s"\'<>]+',re.I)
secrets=re.compile(r'gh[pousr]_[A-Za-z0-9_]{25,}|github_pat_[A-Za-z0-9_]{30,}|xox[baprs]-[A-Za-z0-9-]{20,}|-----BEGIN (?:RSA |DSA |OPENSSH |EC |ENCRYPTED )?PRIVATE KEY-----|AKIA[0-9A-Z]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{32,}|AIza[0-9A-Za-z_-]{35}|glpat-[0-9A-Za-z_-]{20,}|npm_[0-9A-Za-z]{30,}|eyJ[A-Za-z0-9_-]{18,}\.[A-Za-z0-9_-]{15,}\.[A-Za-z0-9_-]{15,}|(?:Bearer\s+)[A-Za-z0-9._~+/-]{24,}',re.I)
agents={'/'+'root/'+x for x in ('hair_ops','rig_identity','lens12_frontier')}
def walk(o,ptr=''):
 if isinstance(o,dict):
  for k,v in o.items():
   yield ptr+'/<key>',k
   yield from walk(v,ptr+'/'+k.replace('~','~0').replace('/','~1'))
 elif isinstance(o,list):
  for k,v in enumerate(o):yield from walk(v,ptr+'/'+str(k))
 elif isinstance(o,str):yield ptr,o
for f in sorted(D.rglob('*')):
 if not f.is_file():continue
 c['files']+=1; raw=f.read_bytes(); name=f.relative_to(D).as_posix()
 try: text=raw.decode('utf-8'); data=json.loads(text) if f.suffix=='.json' else {'text':text}
 except Exception: issues.append((name,'', 'non-text-or-invalid-json')); continue
 for ptr,text in walk(data):
  c['string_values_and_keys']+=1
  norm=text.replace('\\n','\n').replace('\\r','\r').replace('\\/','/')
  norm=re.sub(r'\\u002[fF]','/',norm)
  norm=re.sub(r'\\u005[cC]',lambda _:chr(92),norm)
  for m in path.finditer(norm):
   tok=m.group().rstrip('.,;:!?)]}'); context=(ptr+' '+norm[max(0,m.start()-130):m.start()]).lower()
   if tok in agents and any(z in context for z in ('owner','agent','author','reviewer','attribution','subagent','checked by')):
    c['historical_agent_identifiers']+=1
   else: issues.append((name,ptr,'path-candidate'))
  if windows.search(norm):issues.append((name,ptr,'windows-path-candidate'))
  if secrets.search(norm):issues.append((name,ptr,'secret-pattern-candidate'))
print(json.dumps({'schema':'726-expanded-privacy-review-v1','stage_manifest_sha256':hashlib.sha256((D/'MANIFEST.json').read_bytes()).hexdigest(),'files':c['files'],'string_values_and_keys':c['string_values_and_keys'],'qualified_agent_identifier_occurrences':c['historical_agent_identifiers'],'unclassified_candidates':len(issues),'candidate_locations_and_kinds':[{'file':f,'json_pointer':p,'kind':k} for f,p,k in issues[:50]],'qualification':'Decoded JSON string values and keys and plain UTF-8 text were scanned after explicit escaped-newline, escaped-slash and unicode slash normalization. Pattern scanning is bounded and cannot establish absence of every possible secret or encoded path; historical agent IDs are retained only in attribution context. No private candidate values are printed.'},indent=2))
sys.exit(bool(issues))
