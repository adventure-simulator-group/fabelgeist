import pathlib,hashlib,subprocess,json,gzip,tarfile,shutil
p=pathlib.Path(__file__).parent; repo=pathlib.Path('<PRIVATE_PATH_0001>');old=pathlib.Path('<PRIVATE_PATH_0038>');f82='f82adcabd9d89c4fa2dd7f99253a45297a0681ee';sha=lambda b:hashlib.sha256(b).hexdigest();blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
arc=repo/'delegated/sdk-noiz-sats-hair-evidence.tar.gz'
assert sha(arc.read_bytes())=='421ed4b801935c1995a630147c88af97cebd478fb7ea05a33cda143dcc99998a'
with tarfile.open(arc,'r:gz') as t:
 manifest=json.loads(gzip.decompress(t.extractfile('reference/original-transport-manifest.json.gz').read())); entries={x['path']:x for x in manifest['entries']}
rels=['segovb/READ-L11-428.json','segovb/REGISTRATION-SEGOVB-1538.json','segovb/REGISTRATION-SEGOVB-1539.json','segovb/READ-L12-897.json','parallel-l12-20261029/PROPOSAL-L12-897.json','parallel-l12-20261029/REGISTRATION-PAR-L12-689.json','parallel-l12-20261029/REGISTRATION-PAR-L12-690.json','parallel-l12-20261029/QUALIFICATION-PAR-L12-689-ORIGINAL2623.json','parallel-l12-20261029/QUALIFICATION-PAR-L12-690-ORIGINAL2624.json']
(p/'history-059').mkdir(exist_ok=True);(p/'history-774').mkdir(exist_ok=True);records=[]
for rel in rels:
 b=(old/rel).read_bytes();e=entries[rel];assert (len(b),sha(b),blob(b))==(e['bytes'],e['sha256'],e['git_blob']);dest='history-059/'+pathlib.Path(rel).name;(p/dest).write_bytes(b);records.append(dict(kind='original-059',transport_path=rel,file=dest,sha256=sha(b),bytes=len(b),git_blob=blob(b)))
for rel in ['rig/admission03/source-range-registration.jsonl','rig/admission03/additional-contracts.jsonl','rig/admission03/contract-groups.jsonl','rig/admission03/findings.jsonl','rig/initial06-type-ownership/contracts-022.json']:
 path='handoff/774/adler-original/audit/continuation/'+rel;b=subprocess.check_output(['git','-C',str(repo),'show',f82+':'+path]);dest='history-774/'+pathlib.Path(rel).name+'.gz';(p/dest).write_bytes(gzip.compress(b,mtime=0));records.append(dict(kind='historical-774',transport_revision=f82,transport_path=path,file=dest,encoding='gzip',sha256=sha(b),bytes=len(b),git_blob=blob(b)))
(p/'historical-index.json').write_text(json.dumps(dict(records=records,qualification='059 exact bytes verified against original transport manifest; 774 exact bytes from f82 transport, not a project revision; gzip is lossless storage. Historical reads retain their original reviewer, source revision and scope.'),indent=2)+'\n')
(p/'rules').mkdir(exist_ok=True); rules=[];root=pathlib.Path('<PRIVATE_PATH_0055>')
for index,rel in enumerate(['AGENTS.md','crates/AGENTS.md','crates/fabelgeist-armor/AGENTS.md','crates/adventuresim-character-creator/AGENTS.md']):
 b=(root/rel).read_bytes();dst='rules/'+str(index)+'-AGENTS.md';(p/dst).write_bytes(b)
 try:gb=subprocess.check_output(['git','-C',str(repo),'show','95e63628f9d70b8dd6e6fede64334696c6995009:'+rel],stderr=subprocess.DEVNULL);current_equal=gb==b
 except subprocess.CalledProcessError:current_equal=None
 rules.append(dict(path=rel,copy=dst,sha256=sha(b),current_revision_equal=current_equal,qualification='Guidance read, without product execution; user ban on implementation and product validation controls.'))
(p/'rules.json').write_text(json.dumps(dict(records=rules),indent=2)+'\n')
print('recovered',len(rels),'059 and 5 774, rules',len(rules))
