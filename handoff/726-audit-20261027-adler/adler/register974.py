import subprocess
from pathlib import Path
import bookkeeping as b
rawpath=b.OUT/'REGISTRATION-ADLER-953.json';old=b.read(rawpath)
meta=b.read(b.PUBLISHED/'audit05/original-scope-072.json')['files'][2104]
obj=Path(old['object_path']);blob=obj.read_bytes();loc='6533e444fa37552927070cc8d8190907fd3b413e:'+old['source_id'].split(':',1)[1]
assert b.sha(obj)==old['sha256']==meta['sha256'] and subprocess.check_output(['git','show',loc])==blob
assert len(blob.decode().splitlines())==145 and old['ranges']==[[1,150]]
g=dict(old);g['ranges']=[[1,145]];g['purpose']='Prospective exact EOF145 qualification: inherited metadata requests1..150 but frozen exact object ends145. RawREG953 retained with no bodyread attempted; effective1..145 registered before any body. All13 independently mandatory qualification, no new identity or extension.'
g['inherited_scope_extent_qualification']=dict(kind='Frozen Git exact-byte EOF qualification',original_index=2104,registry=str(b.PUBLISHED/'audit05/original-scope-072.json'),registry_sha256=b.sha(b.PUBLISHED/'audit05/original-scope-072.json'),source_sha256=g['sha256'],actual_EOF=145,raw_requested_range=[1,150],effective_range=[1,145],excluded_nonexistent_ranges=[[146,150]],frozen_source_locator=loc,inherited_metadata=meta,superseded_registration=str(rawpath),superseded_registration_sha256=b.sha(rawpath),failed_read='No rawbodyread attempted. Registration949 metadataEOF display revealed145 before body. REG953raw1..150 retained; effective974full145 prospectively saved before firstbody.',limits='All13 independent lenses must qualify exact same hash EOF145; no146..150 body credit/newglobalidentity/extension/otherlens verdict transfer.',mandatory_lenses=list(range(1,14)),new_global_source=False,other_lens_coverage_transferred=False)
b.save(b.OUT/'REGISTRATION-ADLER-974.json',g)
print('REG974 original2104 exactEOF145 rawmetadata150; no bodyread attempted yet, five nonexistent lines excluded for all13.')
v=b.OUT/'verify_l11_evidence.py';code=v.read_text(encoding='utf-8');oldver=b.sha(v);b.save(b.OUT/'VERIFIER-BEFORE-328.json',dict(sha256=oldver,scope='Preserve previous evidence verifier hash before explicit strict original2104 EOF case, no product code verification.'))
needle="            else:\n                raise AssertionError('Unrecognized exact EOF qualification')"
assert code.count(needle)==1
replacement="""            elif x['original_index'] == 2104:
                assert n == 145 and extent['raw_requested_range'] == [1,150]
                assert extent['effective_range'] == [1,145]
                assert extent['excluded_nonexistent_ranges'] == [[146,150]]
                assert extent['inherited_metadata'] == meta
                assert extent['frozen_source_locator'] == '6533e444fa37552927070cc8d8190907fd3b413e:'+meta['source_id'].split(':',1)[1]
                assert subprocess.check_output(['git','show',extent['frozen_source_locator']]) == obj.read_bytes()
                old_path = Path(extent['superseded_registration'])
                assert b.sha(old_path) == extent['superseded_registration_sha256']
                old = b.read(old_path)
                assert old['ranges'] == [[1,150]] and old['sha256'] == g['sha256']
                assert extent['mandatory_lenses'] == list(range(1,14))
                assert extent['new_global_source'] is False and extent['other_lens_coverage_transferred'] is False
            else:
                raise AssertionError('Unrecognized exact EOF qualification')"""
v.write_text(code.replace(needle,replacement),encoding='utf-8')
print('Strict ignored verifier added2104case, previous hash preserved; raw registration953 notmodified.')
