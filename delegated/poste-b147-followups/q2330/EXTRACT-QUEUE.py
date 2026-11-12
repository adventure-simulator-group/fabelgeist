import argparse,datetime,hashlib,json,pathlib,re,subprocess
p=pathlib.Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--repo',required=True);args=a.parse_args();reg=json.loads((p/'QUEUE-REGISTRATION.json').read_text());src=reg['source'];cmp=reg['comparison'];
def git(rev,path):return subprocess.check_output(['git','-C',args.repo,'show',rev+':'+path])
def sha(x):return hashlib.sha256(x).hexdigest()
raw=git(src['publication_commit'],src['path']);other=git(cmp['publication_commit'],cmp['path'])
if sha(raw)!=src['sha256'] or len(raw)!=src['bytes'] or sha(other)!=cmp['sha256']:raise ValueError('queue source mismatch')
t=raw.decode('utf8');m=re.search(r'"entries"\s*:\s*\[',t)
if not m:raise ValueError('entries not found')
dec=json.JSONDecoder();i=m.end();start=end=0
for n in range(2331):
 while t[i] in ' \t\n\r,':i+=1
 start=i;value,end=dec.raw_decode(t,i);i=end
lo=len(t[:start].encode());hi=len(t[:end].encode());fragment=raw[lo:hi]
allq=json.loads(other);public=allq['entries'][2330]
projection=(json.dumps(value,indent=2,ensure_ascii=True)+'\n').encode();comparison=(json.dumps(public,indent=2,ensure_ascii=True)+'\n').encode()
for name,b in [('QUEUE-2330-RAW.json',fragment),('QUEUE-2330-PROJECTION.json',projection),('QUEUE-2330-PUBLIC-PROJECTION.json',comparison)]: (p/name).write_bytes(b)
x={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_queue_sha256':sha(raw),'pointer':'/entries/2330','byte_range_half_open':[lo,hi],'raw_sha256':sha(fragment),'raw_bytes':len(fragment),'projection_sha256':sha(projection),'projection_bytes':len(projection),'public_queue_sha256':sha(other),'public_revision':cmp['publication_commit'],'public_projection_sha256':sha(comparison),'original_and_public_selected_objects_equal':value==public,'qualification':'Raw exact selected source bytes; projections reconstructed. Selected object equality does not imply equal complete queue files.'}
(p/'QUEUE-IDENTITY.json').write_text(json.dumps(x,indent=2)+'\n'); print(json.dumps({'contract_id':value.get('contract_id'),'disposition':value.get('disposition'),'record_bytes':len(fragment),'selected_objects_equal':value==public,'raw_sha256':sha(fragment)}))
