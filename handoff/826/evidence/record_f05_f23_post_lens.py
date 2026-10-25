import json,sys,hashlib,subprocess
from pathlib import Path
p=Path('target/pr826-audit/ledger.json');d=json.loads(p.read_text());w=d['f05_f23_post_repair']
if subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()!=w['revision']:
 raise SystemExit('revision changed; discovery cannot be recorded')
for f in w['files']:
 if hashlib.sha256(Path(f['path']).read_bytes()).hexdigest()!=f['sha256']:
  raise SystemExit('source changed: '+f['path'])
name=sys.argv[1];prior=next(r for r in d['f05_f23_pre_repair']['lenses'] if r['lens']==name);row=next(r for r in w['lenses'] if r['lens']==name);row.update(json.load(sys.stdin));row['discovery']='complete with unresolved actionable contracts; not clear';row['repair']='prior checkpoint repairs revalidated; successor findings/dependencies pending';row['validation']='source evidence only; native/Wasm/runtime/lint/quality/required hosted checks unrun';row['prior_contract_evidence']='f05_f23_pre_repair/'+name+' and shared_graph_pre_repair_extension; current traces supersede previous repaired gaps';oldfiles={f['path']:f['sha256'] for f in d['f05_f23_pre_repair']['files']};row['file_accounting']=[{'path':f['path'],'sha256':f['sha256'],'evidence':'Byte-identical to prior audited source; current producer/recipient relationships revalidated under this lens.' if oldfiles.get(f['path'])==f['sha256'] else 'Successor source read and lens contract re-evaluated; see source_traces and checkpoint_repairs receipts.','disposition':'Retain unchanged implementation contract or named unresolved owner dependency; no implicit deferral.'} for f in w['files']];p.write_text(json.dumps(d,indent=2)+'\n');print(name,': discovery recorded, no clearance')
