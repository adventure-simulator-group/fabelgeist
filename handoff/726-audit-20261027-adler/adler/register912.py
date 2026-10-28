import bookkeeping as b,subprocess,hashlib
original=b.PUBLISHED/'audit05/original-scope-072.json'
later=b.PUBLISHED/'audit05/later-material/registry-196.json'
for regnum,path,blob in [
(912,'crates/adventuresim-tactical-server/src/mission/enrollment.rs','f1ee59791c70c50a3eb0f26dd6f1fbb4d82fab72'),
(913,'crates/adventuresim-tactical-server/src/mission/receipt.rs','dbd49f0b04b844d012a2edf1ffebdd618a3bf0e6')]:
 locator='6533e444fa37552927070cc8d8190907fd3b413e:'+path
 data=subprocess.check_output(['git','show',locator])
 digest=hashlib.sha256(data).hexdigest()
 obj=b.OUT/'source-objects'/digest
 if obj.exists():assert obj.read_bytes()==data
 else:obj.write_bytes(data)
 sid='pr-working:'+path
 absence=[]
 for p,key in [(original,'files'),(later,'entries')]:
  assert not any(x['source_id']==sid for x in b.read(p)[key])
  absence.append(dict(path=str(p),sha256=b.sha(p),source_id=sid,present=False))
 n=len(data.decode('utf-8-sig').splitlines())
 g=dict(lens=11,population='connected-material',population_key=digest,source_id=sid,sha256=digest,object_path=str(obj),ranges=[[1,n]],prospective=True,provenance=locator,git_blob=blob,source_identity_scope='New receiving registration, absent original2853/later196; global identity across other inherited populations unresolved, no new global count credited.',new_global_source='unreconciled across inherited populations',population_absence_evidence=absence,mandatory_lenses=list(range(1,14)),other_lens_coverage_transferred=False,purpose='Necessary actual terminal/enrollment child owner reached byoriginal2063/2064: complete capturedroster/refusal andterminalreceipt semantics foracceptance/ownership. Exact fullsmallowner includingconnectedsource fixtures; independentall13 mandatory, no original2853/later196 frontiercredit, source-only.')
 b.save(b.OUT/f'REGISTRATION-ADLER-{regnum:03}.json',g)
 print(regnum,sid,digest,n)
