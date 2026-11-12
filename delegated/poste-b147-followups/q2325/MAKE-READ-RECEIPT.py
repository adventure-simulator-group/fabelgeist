#!/usr/bin/env python3
"""Record displayed source fragments only; captured files are not read credit."""
import datetime as dt,hashlib,json,pathlib,re
p=pathlib.Path(__file__).resolve().parent
load=lambda n:json.loads((p/n).read_text())
rows=load('SOURCE-CAPTURES.json')+load('SOURCE-SUPPORT-CAPTURES.json')+[load('SOURCE-FITTING-CAPTURE.json')]
captures={r['path']:r for r in rows}; assert len(captures)==len(rows)
labels={'animation':'crates/adventuresim-character-creator/src/animation_preview.rs','binding':'crates/adventuresim-character-creator/src/animation_preview/binding.rs','preview':'crates/adventuresim-character-creator/src/preview.rs','rig_skin':'crates/fabelgeist-rig/src/skin.rs','mhr_skin':'crates/fabelgeist-mhr/src/character/skin.rs','clothing':'crates/adventuresim-character-creator/src/clothing.rs','fitting':'crates/adventuresim-character-creator/src/clothing/fitting.rs','export_skin':'crates/adventuresim-character-creator/src/export/skinning.rs','colliders':'crates/adventuresim-character-creator/src/animation_preview/colliders.rs','drape':'crates/adventuresim-character-creator/src/drape_preview.rs'}
regs=[]
for f in p.glob('SOURCE*REGISTRATION.json'):
 d=json.loads(f.read_text());
 if 'sources' in d: regs.append((f.name,d))
def spans(ns):
 out=[]
 for i in sorted(ns):
  if not out or out[-1][1]+1!=i:out.append([i,i])
  else:out[-1][1]=i
 return out
seen={s:set() for s in captures};chunks=[]
for f in sorted((p/'displays').glob('*.txt')):
 m=re.fullmatch(r'(.+)-(\d+)-(\d+)\.txt',f.name); assert m; label,lo,hi=m.group(1),int(m.group(2)),int(m.group(3));src=labels[label];saved=dt.datetime.fromtimestamp(f.stat().st_mtime,dt.timezone.utc).isoformat(); matching=[(n,r) for n,r in regs if r['registered_utc']<=saved and any(x['path']==src and x['range'][0]<=lo<=hi<=x['range'][1] for x in r['sources'])];assert matching,(f,saved)
 n,r=sorted(matching,key=lambda x:x[1]['registered_utc'])[0];b=f.read_bytes();expected=''.join(f'{i:4d} {line}\n' for i,line in enumerate((p/captures[src]['capture']).read_text().splitlines(),1) if lo<=i<=hi).encode();assert b==expected;seen[src].update(range(lo,hi+1));chunks.append({'source_path':src,'start':lo,'end':hi,'display':str(f.relative_to(p)),'registration':n,'saved_utc':saved,'sha256':hashlib.sha256(b).hexdigest()})
coverage=[]
for src,x in sorted(captures.items()):coverage.append({'source_path':src,'eof':x['eof'],'personally_displayed':spans(seen[src]),'unread':spans(set(range(1,x['eof']+1))-seen[src])})
(p/'SOURCE-READ-RECEIPT.json').write_text(json.dumps({'source_revision':'95e63628f9d70b8dd6e6fede64334696c6995009','reader':'/root/verifier_scope','chunks':chunks,'coverage':coverage,'unique_lines_personally_displayed':sum(map(len,seen.values())),'qualification':'Personal display credit only for listed intervals. Timestamp records process chronology; capture, locator, and historical reads grant no additional personal display credit.'},indent=2)+'\n')
print('source chunks',len(chunks),'unique personally displayed lines',sum(map(len,seen.values())),'captured files',len(captures))
