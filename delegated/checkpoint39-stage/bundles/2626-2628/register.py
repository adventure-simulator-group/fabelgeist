import pathlib,hashlib,json,tarfile,subprocess,gzip,datetime,shutil,re
p=pathlib.Path(__file__).parent; repo='<PRIVATE_PATH_0001>'; prior=pathlib.Path('<PRIVATE_PATH_0039>'); source=p/'sources'; source.mkdir(exist_ok=True); sha=lambda b:hashlib.sha256(b).hexdigest()
current='95e63628f9d70b8dd6e6fede64334696c6995009'; old='6533e444fa37552927070cc8d8190907fd3b413e'; entries=[]
archives=[('num_enum_derive-0.7.6.crate','680998035259dcfcafe653688bf2aa6d3e2dc05e98be6ab46afb089dc84f1df8'),('num_enum-0.7.6.crate','5d0bca838442ec211fa11de3a8b0e0e8f3a4522575b5c4c06ed722e005036f26')]
for n,h in archives:
 b=(prior/'metadata'/n).read_bytes();assert sha(b)==h;(p/n).write_bytes(b)
def put(id,b,kind,path,rev,ranges,**extra):
 filename='sources/'+id; (p/filename).write_bytes(b); lines=len(b.splitlines()); assert all(1<=a<=z<=lines for a,z in ranges);entries.append(dict(id=id,kind=kind,path=path,revision=rev,archive=filename,sha256=sha(b),bytes=len(b),eof=lines,registered_ranges=ranges,**extra))
roles=[('2626-lib.rs','num_enum_derive-0.7.6/src/lib.rs',[[190,260]],'8dd8a3259e3f016b189332efb66b735c1473c70ce7e1c152da33a3e99250741a'),('2627-parsing.rs','num_enum_derive-0.7.6/src/parsing.rs',[[1,20],[62,160],[263,281],[361,447],[506,560]],'abe7e01b2939a26e273d91fd975056216b87c87cf494a1869248762145469fd1'),('2628-enum_attributes.rs','num_enum_derive-0.7.6/src/enum_attributes.rs',[[1,30],[112,156],[181,212]],'169a5b973235cf21f0d2c1c088acf3b6526d62758c83e14a0b1e624afcd648ed')]
support=[('derive-Cargo.toml','num_enum_derive-0.7.6/Cargo.toml',[[30,55]],'c2b63f5feb0277a2d0e0cdd8c4620e144515daa0f53d30f892e1803ed4c835e2'),('runtime-Cargo.toml','num_enum-0.7.6/Cargo.toml',[[47,51],[85,88]],'1451ba4c3948ab057e45c8e30674938dc1c2d025206557f72543d8e3ec81579d'),('runtime-lib.rs','num_enum-0.7.6/src/lib.rs',[[1,53]],'c4890107b756ad546bddacb8bf5a2d735bdf6ebe12fa0aed8052710e985cd79b')]
for id,member,ranges,expected in roles+support:
 archive='num_enum_derive-0.7.6.crate' if member.startswith('num_enum_derive') else 'num_enum-0.7.6.crate'
 with tarfile.open(p/archive,'r:gz') as t:b=t.extractfile(member).read()
 assert sha(b)==expected;put(id,b,'native-package',member,None,ranges,package_archive=archive,expected_sha256=expected,role=int(id[:4]) if id[:4].isdigit() else None)
for id,path,ranges in [('fbx-manifest','crates/fabelgeist-fbx/Cargo.toml',[[1,18]]),('fbx-array-encoding','crates/fabelgeist-fbx/src/array_encoding.rs',[[1,24]]),('fbx-reader','crates/fabelgeist-fbx/src/lib.rs',[[222,243],[420,480]])]:
 b=subprocess.check_output(['git','-C',repo,'show',current+':'+path]);put(id,b,'current-project',path,current,ranges)
for label,rev in [('old',old),('current',current)]:
 b=subprocess.check_output(['git','-C',repo,'show',rev+':Cargo.lock']);lines=b.decode().splitlines();ranges=[]
 for i,l in enumerate(lines):
  if l=='[[package]]' and i+1<len(lines) and lines[i+1] in ('name = "num_enum"','name = "num_enum_derive"'):
   j=i+1
   while j<len(lines) and lines[j]!='[[package]]':j+=1
   while j>i+1 and not lines[j-1]:j-=1
   ranges.append([i+1,j])
 assert len(ranges)==2
 filename='sources/'+label+'-Cargo.lock.gz';(p/filename).write_bytes(gzip.compress(b,mtime=0)); entries.append(dict(id=label+'-lock',kind='project-lock',path='Cargo.lock',revision=rev,archive=filename,encoding='gzip',sha256=sha(b),bytes=len(b),eof=len(lines),registered_ranges=ranges,qualification='Structural package scan locates sections only; no claim of full-body read or Cargo feature resolution.'))
registration={'schema':'726-num-enum-reconciliation-prospective-v1','registered_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'author':'/root/mhr_character','entries':entries,'archives':[{'path':n,'sha256':h} for n,h in archives],'qualification':'These are prospective personal display windows only; archived complements are not personally read. Native crate roles are not current Git source. f82 transports recovered evidence only. Source inspection only; no macro execution, builds, tests or runtime.'}
(p/'registration.json').write_text(json.dumps(registration,indent=2)+'\n');print('registered',len(entries),'entries');print(json.dumps([{'id':e['id'],'ranges':e['registered_ranges'],'eof':e['eof']} for e in entries],indent=2))
