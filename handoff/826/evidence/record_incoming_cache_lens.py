import json,sys,hashlib,subprocess
from pathlib import Path
p=Path('target/pr826-audit/ledger.json');d=json.loads(p.read_text());w=d['incoming_cache_pre_integration']
if subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()!=w['receiving_head']:raise SystemExit('receiving revision changed')
root=Path(w['export_manifest']).parent
for f in w['immutable_export_files']:
 if hashlib.sha256((root/'source'/f['path']).read_bytes()).hexdigest()!=f['sha256']:raise SystemExit('export changed '+f['path'])
for f in w['files']:
 if hashlib.sha256(Path(f['path']).read_bytes()).hexdigest()!=f['sha256']:raise SystemExit('current source changed '+f['path'])
r=next(x for x in w['lenses'] if x['lens']==sys.argv[1]);r.update(json.load(sys.stdin));r['discovery']='complete for pinned incoming cache integration frontier; required whole-family owner migrations unresolved';r['repair']='not integrated; no owner source body independently repaired';r['validation']='source only; no new tests/cases/helpers/assertions, no project execution/push/CI';r['file_accounting']=[{'path':f['path'],'sha256':f['sha256'],'role':f['integration'],'disposition':'Reference-only Reduce dependency; not integrated or accepted.' if '/reduce/' in f['path'] else 'Integrate only exact cache/constant delta preserving current core; remaining owner count/layout/parser/mode families unresolved.'} for f in w['immutable_export_files']];r['prior_scope_accounting']='All73 receiving paths retained at f05_f23_post_repair exact3ff; cache delta and unchanged consumers source read; same no-final-clearance dependencies remain.'
p.write_text(json.dumps(d,indent=2)+'\n');print(r['lens'],'recorded; no clearance')
