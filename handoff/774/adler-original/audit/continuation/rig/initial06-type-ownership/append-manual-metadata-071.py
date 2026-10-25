"""Ignored receipt bookkeeping only. Caller supplies every manual semantic comparison/range."""
import json,pathlib,hashlib,datetime
b=pathlib.Path('target/774-continuation-20261021/continuation');o=b/'rig/initial06-type-ownership'
def append(number,groups,entries,support=(),limits='Current06 source-role analysis only. No execution, other-lens or called-helper body credit.',extra=None):
    scope='cumulative-source-scope-parallel-071.json';s=json.loads((b/scope).read_text(encoding='utf-8'));now=datetime.datetime.now(datetime.timezone.utc).isoformat();name=f'contracts-{number:03}.json';out=o/name;assert not out.exists()
    rows=[]
    for i,key,ranges,mode in entries:
        x=s['files'][i];snap={'physical-p16':'p16','report-p17':'p17'}.get(x['snapshot'],x['snapshot']);p=pathlib.Path(x['path']);p=p if p.is_absolute() else pathlib.Path('target/774-snapshots')/snap/p;raw=p.read_bytes();assert hashlib.sha256(raw).hexdigest()==x['sha256'];eof=len(raw.decode('utf-8').splitlines());assert all(1<=a<=z<=eof for a,z in ranges)
        rows.append({'id':f'RIG-INITIAL06-SOURCE-{i}','source_id':x['source_id'],'sha256':x['sha256'],'actual_eof':eof,'scope_record':scope,'scope_pointer':f'/files/{i}','actual_fresh_or_compared_ranges':ranges,'read_mode':mode,'contract_refs':[name+'#/groups/'+key],'initial06_disposition':groups[key]['disposition'],'limits':limits})
    packet={'id':f'RIG-INITIAL06-GROUP-{number:03}','at':now,'lens':'audit-type-ownership','scope':scope,'groups':groups,'supporting_existing_findings':list(support),'limits':limits}
    if extra:packet.update(extra)
    out.write_text(json.dumps(packet,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    with (o/'coverage.jsonl').open('a',encoding='utf-8') as f:
        for r in rows:f.write(json.dumps(r,ensure_ascii=False)+'\n')
    allrows=[json.loads(r) for r in (o/'coverage.jsonl').read_text(encoding='utf-8').splitlines()];progress={'at':now,'completed_global_source_variants':len({r['source_id'] for r in allrows}),'coverage_observations':len(allrows),'current_scope_variants':len(s['files']),'mandatory_overlays':245,'packet':name,'whole_lens_status':'incomplete','next':'Continue exhaustive06;09/12 unstarted.'}
    with (o/'progress.jsonl').open('a',encoding='utf-8') as f:f.write(json.dumps(progress)+'\n')
    print(json.dumps(progress))
