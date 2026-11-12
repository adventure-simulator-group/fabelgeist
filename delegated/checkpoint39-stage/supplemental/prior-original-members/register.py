import pathlib,subprocess,hashlib,gzip,json,datetime
p=pathlib.Path(__file__).parent;repo='<PRIVATE_PATH_0001>';rf='<PRIVATE_PATH_0056>';raw=gzip.decompress(pathlib.Path(rf).read_bytes());registry=json.loads(raw);rows=[]
for idx in (2623,2624):
 ent=registry['files'][idx];path=ent['source_id'].split(':',1)[1];objects=[]
 for label,rev in [('historical','6533e444fa37552927070cc8d8190907fd3b413e'),('current','95e63628f9d70b8dd6e6fede64334696c6995009')]:
  b=subprocess.check_output(['git','-C',repo,'show',f'{rev}:{path}']);blob=subprocess.check_output(['git','-C',repo,'rev-parse',f'{rev}:{path}'],text=True).strip();sha=hashlib.sha256(b).hexdigest();rel=f'source/{idx}-{label}.rs';(p/rel).write_bytes(b)
  objects.append(dict(revision=rev,label=label,git_path=path,sha256=sha,git_blob=blob,bytes=len(b),eof=len(b.splitlines()),prospective_ranges=[[1,len(b.splitlines())]],local_file=rel))
 assert objects[0]['sha256']==ent['sha256'] and objects[0]['bytes']==ent['bytes']; rows.append(dict(original_index=idx,registry_entry=ent,objects=objects))
reg=dict(author='/root/mhr_character',registered_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),registry_uncompressed_sha256=hashlib.sha256(raw).hexdigest(),rows=rows,qualification='Distinct historical/current full personal bounded displays to follow; file identity does not establish clean pr-working snapshot, historical reviewer transfer, P16 or 3821 credit.')
(p/'registration.json').write_text(json.dumps(reg,indent=2)+'\n');print(json.dumps([dict(index=r['original_index'],objects=r['objects']) for r in rows],indent=2))
