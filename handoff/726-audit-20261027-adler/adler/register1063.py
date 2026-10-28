import subprocess
from pathlib import Path
import bookkeeping as b
rawpath=b.OUT/'REGISTRATION-ADLER-1055.json';old=b.read(rawpath)
registry=b.PUBLISHED/'audit05/original-scope-072.json'
meta=b.read(registry)['files'][2197]
obj=Path(old['object_path']);blob=obj.read_bytes()
loc='6533e444fa37552927070cc8d8190907fd3b413e:'+old['source_id'].split(':',1)[1]
assert b.sha(obj)==old['sha256']==meta['sha256'] and subprocess.check_output(['git','show',loc])==blob
assert len(blob.decode().splitlines())==362 and old['ranges']==[[1,369]]
g=dict(old);g['ranges']=[[1,362]]
g['purpose']='Prospective exact EOF362 qualification: metadata union requests1..369 but exact frozen source ends362. RawREG1055 retained, no rawbodyread attempted. Effective1..362 registered beforebody. All13 mandatoryqualification, no newidentity or scopedextension.'
g['inherited_scope_extent_qualification']=dict(kind='Frozen Git exact-byte EOF qualification',original_index=2197,registry=str(registry),registry_sha256=b.sha(registry),source_sha256=g['sha256'],actual_EOF=362,raw_requested_range=[1,369],effective_range=[1,362],excluded_nonexistent_ranges=[[363,369]],frozen_source_locator=loc,inherited_metadata=meta,superseded_registration=str(rawpath),superseded_registration_sha256=b.sha(rawpath),failed_read='No rawbodyreadattempted: register1053 metadataEOF output362 detectedovershoot beforebody. RawREG1055full369 retained; prospectiveREG1063full362 beforefirstbody.',limits='All13 independentlenses mustqualifyexacthashEOF362, no363..369credit/newidentity/scopedextension or otherlenscoverage.',mandatory_lenses=list(range(1,14)),new_global_source=False,other_lens_coverage_transferred=False)
b.save(b.OUT/'REGISTRATION-ADLER-1063.json',g)
v=b.OUT/'verify_l11_evidence.py';code=v.read_text(encoding='utf-8')
b.save(b.OUT/'VERIFIER-BEFORE-347.json',dict(sha256=b.sha(v),scope='Previous evidence verifier before sixth strict exact EOF2197 case; no product verification.'))
needle="            else:\n                raise AssertionError('Unrecognized exact EOF qualification')"
assert code.count(needle)==1
replacement="""            elif x['original_index'] == 2197:
                assert extent['kind'] == 'Frozen Git exact-byte EOF qualification'
                assert n == 362 and extent['raw_requested_range'] == [1,369]
                assert extent['effective_range'] == [1,362]
                assert extent['excluded_nonexistent_ranges'] == [[363,369]]
                assert extent['inherited_metadata'] == meta
                assert extent['frozen_source_locator'] == '6533e444fa37552927070cc8d8190907fd3b413e:'+meta['source_id'].split(':',1)[1]
                assert subprocess.check_output(['git','show',extent['frozen_source_locator']]) == obj.read_bytes()
                old_path = Path(extent['superseded_registration'])
                assert b.sha(old_path) == extent['superseded_registration_sha256']
                old = b.read(old_path)
                assert old['ranges'] == [[1,369]] and old['sha256'] == g['sha256']
                assert extent['mandatory_lenses'] == list(range(1,14))
                assert extent['new_global_source'] is False and extent['other_lens_coverage_transferred'] is False
            else:
                raise AssertionError('Unrecognized exact EOF qualification')"""
v.write_text(code.replace(needle,replacement),encoding='utf-8')
print('REG1063 original2197 exactEOF362 vsraw369; 363..369nonexistent all13mandatory. Strictignoredverifiercaseadded beforebody, rawREG1055retained.')
