import sys,json,pathlib,subprocess
p=pathlib.Path('target/pr826-audit/ledger.json');d=json.loads(p.read_text());w=d['checkpoint_wave'];assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()==w['revision']
name=sys.argv[1]; row=next(r for r in w['lenses'] if r['lens']==name)
row.update(json.loads(sys.stdin.read()));row['discovery']='complete';row['repair']='not started after checkpoint';row['validation']='not run; source-only findings, no CI and no clearance'
row['file_accounting']=[{'path':f['path'],'receipt':'exact checkpoint manifest; '+('Authored fixture/test oracle and existing API adaptation; no added cases/helpers/assertions.' if '/tests/' in f['path'] or f['path'].endswith('/tests.rs') else 'Hosted validation invocation/evidence, no execution yet.' if '/.github/' in '/'+f['path'] else 'Durable native API/validation contract.' if f['path'].endswith('.md') else 'Native texture/shader/compute producer or receiving adapter; see this lens source traces and shared_graph_pre_repair_extension.'),'disposition':'Source accounted; unresolved connected findings/dependencies remain explicit.'} for f in w['files']]
if all(r['discovery']=='complete' for r in w['lenses']):w['discovery']='complete with actionable findings; not clear'
p.write_text(json.dumps(d,indent=2)+'\n');print(name,'discovery complete; no clearance')
