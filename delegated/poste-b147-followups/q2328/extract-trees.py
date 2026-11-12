import pathlib,json,hashlib,sys
s=pathlib.Path(__file__).parent
prefix=sys.argv[1]; allowed=set(sys.argv[2:]); out=[]
for p in sorted(s.glob('TREE-CONNECTOR-'+prefix+'-*.json')):
 oid=p.stem.split('-')[-1]; j=json.loads(p.read_text()); assert not j.get('isError'); outer=j['structuredContent']; obj=outer.get('structuredContent')
 if not isinstance(obj,dict) or 'tree' not in obj: obj=json.loads(outer['content'])
 assert obj['sha']==oid and not obj.get('truncated',False)
 raw=b''.join((e['mode'].lstrip('0')+' '+e['path']).encode()+b'\0'+bytes.fromhex(e['sha']) for e in obj['tree'])
 assert hashlib.sha1(b'tree '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==oid
 (s/'git-trees'/(oid+'.tree')).write_bytes(raw)
 out.append({'tree':oid,'tree_sha256':hashlib.sha256(raw).hexdigest(),'entries':[{'path':e['path'],'mode':e['mode'],'sha':e['sha']} for e in obj['tree'] if e['path'] in allowed]})
(s/('TREE-CONNECTOR-VIEW-'+prefix+'.json')).write_text(json.dumps(out,indent=2)+'\n'); print(json.dumps(out,indent=2))
