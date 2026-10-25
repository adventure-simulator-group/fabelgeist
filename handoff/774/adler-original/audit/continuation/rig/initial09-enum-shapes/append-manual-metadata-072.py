"""Ignored receipt bookkeeping. Every group/entry/range supplied from manual lens analysis."""
import pathlib,json,hashlib,datetime
b=pathlib.Path('target/774-continuation-20261021/continuation');o=b/'rig/initial09-enum-shapes'
def append(number,groups,entries,findings=(),extra=None):
 scope='cumulative-source-scope-parallel-072.json';s=json.loads((b/scope).read_text(encoding='utf-8'));now=datetime.datetime.now(datetime.timezone.utc).isoformat();name=f'contracts-{number:03}.json';p=o/name;assert not p.exists();rows=[]
 for i,key,rs,mode in entries:
  x=s['files'][i];q=pathlib.Path(x['path']);q=q if q.is_absolute() else pathlib.Path('target/774-snapshots')/{'physical-p16':'p16','report-p17':'p17'}.get(x['snapshot'],x['snapshot'])/q;raw=q.read_bytes();assert hashlib.sha256(raw).hexdigest()==x['sha256'];eof=len(raw.decode('utf-8').splitlines());assert all(1<=a<=z<=eof for a,z in rs);rows.append({'source_id':x['source_id'],'sha256':x['sha256'],'scope':scope,'scope_pointer':f'/files/{i}','actual_eof':eof,'actual_fresh_or_compared_ranges':rs,'read_mode':mode,'contract_ref':name+'#/groups/'+key,'disposition':groups[key]['disposition'],'limits':'Initial09 source-specific comparison only. No project execution/source edits/other-lens verdict carry.'})
 r={'id':f'RIG-INITIAL09-GROUP-{number:03}','at':now,'lens':'audit-enum-shapes','scope':scope,'groups':groups,'finding_refs':list(findings),'limits':'All original scope/context exclusions retained; no whole caller/nativepackage credit.'}
 if extra:r.update(extra)
 p.write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 with (o/'coverage.jsonl').open('a',encoding='utf-8') as f:
  for r in rows:f.write(json.dumps(r,ensure_ascii=False)+'\n')
 c=[json.loads(l) for l in (o/'coverage.jsonl').read_text(encoding='utf-8').splitlines()];v={'at':now,'completed_distinct_sources':len({r['source_id'] for r in c}),'observations':len(c),'scope_variants':len(s['files']),'packet':name,'status':'initial09_incomplete','next':'Continue all scoped carried boolean/tag/payload/transition/codec comparisons;12 unstarted.'}
 with (o/'progress.jsonl').open('a',encoding='utf-8') as f:f.write(json.dumps(v)+'\n')
 print(json.dumps(v))
