"""Ignored evidence integrity bookkeeping only, not a product scanner/test."""
import sys, subprocess, hashlib, tarfile, tomllib
from pathlib import Path
import bookkeeping as b

def all_ranges(value):
    if isinstance(value, dict):
        for key, item in value.items():
            if key in ('ranges', 'read_ranges', 'read_ranges_as_recorded', 'material_ranges', 'effective_material_ranges', 'material_contract_ranges_as_recorded') and isinstance(item, list):
                for pair in item:
                    if isinstance(pair, list) and len(pair) == 2 and all(isinstance(n, int) for n in pair):
                        yield pair
            yield from all_ranges(item)
    elif isinstance(value, list):
        for item in value:
            yield from all_ranges(item)

def verify(batch, predecessor):
    cur = b.read(b.ROOT/'CURRENT-ADLER.json')
    assert b.sha(b.ROOT/cur['ledger']) == cur['ledger_sha256']
    assert cur['implementation_gate_complete'] is False
    assert cur['accepted_unique_count_change'] == 0
    rp = b.OUT/f'READ-L11-{batch:03}.json'
    sp = b.OUT/f'STATE-L11-{batch:03}.json'
    j, s = b.read(rp), b.read(sp)
    assert b.sha(Path(s['receipt'])) == s['receipt_sha256']
    assert b.sha(Path(s['predecessor'])) == s['predecessor_sha256']
    previous = b.read(Path(s['predecessor']))
    assert [x['original_index'] for x in j['sources']] == list(range(previous['next_original'], j['next_original']))
    assert s['newly_accounted'] == [x['original_index'] for x in j['sources']]
    assert s['next_original'] == j['next_original']
    registry = b.read(b.PUBLISHED/'audit05/original-scope-072.json')['files']
    checks = []
    for x in j['sources'] + j['connected_reinspection']:
        regpath = Path(x['registration'])
        assert b.sha(regpath) == x['registration_sha256']
        g = b.read(regpath)
        if g.get('deleted_original'):
            meta = registry[x['original_index']]
            assert meta['state']=='deleted' and meta['sha256'] is None
            assert g['source_id']==x['source_id']==meta['source_id']
            assert g['sha256'] is None and x['sha256'] is None
            assert g['ranges']==x['scope']==[] and x['fresh_reads']==[]
            assert g['prospective'] and x['deleted_original']
            assert g['mandatory_lenses']==list(range(1,14)) and g['other_lens_coverage_transferred'] is False
            absence=g['absence_evidence']
            assert x['absence_evidence']==absence and absence['commit']=='6533e444fa37552927070cc8d8190907fd3b413e'
            assert absence['path']==meta['path'] and absence['git_ls_tree_stdout']=='' and absence['git_ls_tree_exit_code']==0
            assert not subprocess.check_output(['git','ls-tree',absence['commit'],'--',absence['path']])
            base=g['base_source'];assert x['base_source']==base and base['sha256']==meta['base_sha256']
            obj=Path(base['object_path']);assert b.sha(obj)==base['sha256'] and base['prospective']
            n=len(obj.read_text(encoding='utf-8-sig').splitlines())
            assert base['ranges']==[[1,n]]
            covered=set(i for a,z in x['base_fresh_reads'] for i in range(a,z+1))
            assert set(range(1,n+1))<=covered
            for a,z in all_ranges(meta):assert set(range(a,z+1))<=covered
            assert x['owner_proof_qualification']==g['owner_proof_qualification']
            if base['provenance'].startswith('3a50b708381ad00c116445c2c999a49c3943088b:'):
                assert subprocess.check_output(['git','show',base['provenance']])==obj.read_bytes()
            checks.append(dict(original_index=x['original_index'],source_id=x['source_id'],current_deleted=True,current_body_reads=[],historical_base_sha256=base['sha256'],historical_base_ranges=base['ranges'],absence_commit=absence['commit'],owner_current_range_proof_qualified=True,proof_kind='Frozen deletion plus exact historical base body, no live body credit'))
            continue
        obj = Path(g['object_path'])
        assert b.sha(obj) == g['sha256']
        n = len(obj.read_text(encoding='utf-8-sig').splitlines())
        assert g['prospective'] and all(1 <= a <= z <= n for a,z in g['ranges'])
        if 'original_index' in x:
            meta = registry[x['original_index']]
        else:
            assert x['kind'] in ('Existing later-material full same-lens source read, no original credit','Existing later-material scoped same-lens source read, no original credit')
            assert g['population'] == x['population'] == 'later-material'
            assert g['population_index'] == x['population_index']
            population_path = Path(g['population_registry'])
            assert b.sha(population_path) == g['population_registry_sha256']
            meta = b.read(population_path)['entries'][g['population_index']]
            assert g['new_global_source'] is False and x['new_global_source'] is False
            assert g['mandatory_lenses'] == list(range(1,14))
            assert g['other_lens_coverage_transferred'] is False
        assert meta['sha256'] == g['sha256'] and meta['source_id'] == g['source_id']
        assert x.get('source_sha256',x.get('sha256')) == g['sha256'] and x['source_id'] == g['source_id']
        assert x['scope'] == g['ranges']
        ranges = x['fresh_reads']
        if not ranges:
            role = next(v for v in j['role_dispositions'] if x['original_index'] in v['source_indices'])
            proof = role['reuse_proof']
            prior = Path(proof['receipt'])
            assert b.sha(prior) == proof['receipt_sha256']
            old = next(v for v in b.read(prior)['connected_reinspection'] if v['original_index'] == x['original_index'])
            assert old['registration'] == x['registration']
            assert old['registration_sha256'] == x['registration_sha256']
            assert old['source_sha256'] == g['sha256']
            assert old['additional_original_frontier_credit'] == 0
            ranges = old['fresh_reads']
            assert proof['read_ranges'] == ranges
        covered = set(i for a,z in ranges for i in range(a,z+1))
        assert set(i for a,z in g['ranges'] for i in range(a,z+1)) <= covered
        if meta.get('scope_kind') == 'frozen_requested_change':
            assert set(range(1,n+1)) <= covered
        # Exact complementary prospective scopes may share one receipt.
        # Each is checked independently above; metadata completeness is their union.
        companions = []
        for other in j['sources'] + j['connected_reinspection']:
            if other is x or other.get('original_index') != x.get('original_index'):
                continue
            if other['source_id'] != x['source_id'] or other.get('source_sha256',other.get('sha256')) != g['sha256']:
                continue
            if not other['fresh_reads']:
                continue
            op = Path(other['registration']); og = b.read(op)
            assert b.sha(op) == other['registration_sha256']
            assert og['prospective'] and og['source_id'] == g['source_id'] and og['sha256'] == g['sha256']
            assert b.sha(Path(og['object_path'])) == g['sha256']
            assert other['scope'] == og['ranges']
            oc = set(i for a,z in other['fresh_reads'] for i in range(a,z+1))
            assert all(1 <= a <= z <= n for a,z in other['fresh_reads'])
            assert set(i for a,z in og['ranges'] for i in range(a,z+1)) <= oc
            covered |= oc
            companions.append(dict(registration=str(op),sha256=b.sha(op),read_ranges=other['fresh_reads']))
        raw_metadata_ranges = list(all_ranges(meta))
        extent = g.get('inherited_scope_extent_qualification')
        if extent:
            assert x['inherited_scope_extent_qualification'] == extent
            assert extent['original_index'] == x['original_index']
            registry_path = b.PUBLISHED/'audit05/original-scope-072.json'
            assert extent['registry'] == str(registry_path)
            assert b.sha(registry_path) == extent['registry_sha256']
            assert extent['source_sha256'] == g['sha256'] == meta['sha256']
            assert extent['actual_EOF'] == n
            if x['original_index'] == 1622:
                assert n == 231 and extent['raw_requested_range'] == [155,240]
                assert extent['effective_range'] == [155,231]
                assert extent['excluded_nonexistent_ranges'] == [[232,240]]
                assert subprocess.check_output(['git','show',extent['frozen_source_locator']]) == obj.read_bytes()
                assert extent['frozen_source_locator'] == '6533e444fa37552927070cc8d8190907fd3b413e:'+meta['path']
                producer = meta['continuation_actual_producer_source_proofs'][0]
                rig = meta['admission03_increment077_role_observations'][0]
                assert extent['prior_producer_qualification'] == producer
                assert extent['prior_rig_qualification'] == rig
                assert producer['sha256'] == g['sha256'] and producer['read_ranges'] == [[155,231]]
                assert producer['proof']['actual_EOF'] == rig['actual_EOF'] == rig['original_record']['actual_eof'] == n
                old_path = Path(extent['superseded_registration'])
                assert b.sha(old_path) == extent['superseded_registration_sha256']
                old = b.read(old_path)
                assert old['ranges'] == [[16,17],[155,240]] and old['sha256'] == g['sha256']
            elif x['original_index'] == 4:
                assert extent['kind'] == 'Native package member exact-byte EOF qualification'
                assert n == 129 and extent['raw_requested_range'] == [1,159]
                assert extent['effective_range'] == [1,129]
                assert extent['excluded_nonexistent_ranges'] == [[130,159]]
                assert extent['inherited_owner_proofs'] == meta['owner_contract_proofs']
                archive = Path(extent['native_archive'])
                assert b.sha(archive) == extent['native_archive_sha256']
                assert extent['native_member'] == 'bevy_asset-0.19.1/src/event.rs'
                with tarfile.open(archive, 'r:gz') as package:
                    assert package.extractfile(extent['native_member']).read() == obj.read_bytes()
                assert extent['frozen_lock_locator'] == '6533e444fa37552927070cc8d8190907fd3b413e:Cargo.lock'
                lock = subprocess.check_output(['git','show',extent['frozen_lock_locator']])
                assert hashlib.sha256(lock).hexdigest() == extent['frozen_lock_sha256']
                dependency = next(v for v in tomllib.loads(lock.decode())['package']
                                  if v['name']=='bevy_asset' and v['version']=='0.19.1')
                assert {k:dependency[k] for k in ('name','version','source','checksum')} == extent['native_package']
                assert dependency['checksum'] == extent['native_archive_sha256']
                assert extent['mandatory_lenses'] == list(range(1,14))
                assert extent['new_global_source'] is False and extent['other_lens_coverage_transferred'] is False
            elif x['original_index'] == 1800:
                assert extent['kind'] == 'Frozen Git exact-byte EOF qualification'
                assert n == 97 and extent['raw_requested_range'] == [1,120]
                assert extent['effective_range'] == [1,97]
                assert extent['excluded_nonexistent_ranges'] == [[98,120]]
                assert extent['inherited_metadata'] == meta
                assert extent['frozen_source_locator'] == '6533e444fa37552927070cc8d8190907fd3b413e:'+meta['source_id'].split(':',1)[1]
                assert subprocess.check_output(['git','show',extent['frozen_source_locator']]) == obj.read_bytes()
                old_path = Path(extent['superseded_registration'])
                assert b.sha(old_path) == extent['superseded_registration_sha256']
                old = b.read(old_path)
                assert old['ranges'] == [[1,120]] and old['sha256'] == g['sha256']
            elif x['original_index'] == 1907:
                assert extent['kind'] == 'Frozen Git exact-byte EOF qualification'
                assert n == 122 and extent['raw_requested_range'] == [1,155]
                assert extent['effective_range'] == [1,122]
                assert extent['excluded_nonexistent_ranges'] == [[123,155]]
                assert extent['inherited_metadata'] == meta
                assert extent['frozen_source_locator'] == '6533e444fa37552927070cc8d8190907fd3b413e:'+meta['source_id'].split(':',1)[1]
                assert subprocess.check_output(['git','show',extent['frozen_source_locator']]) == obj.read_bytes()
                old_path = Path(extent['superseded_registration'])
                assert b.sha(old_path) == extent['superseded_registration_sha256']
                old = b.read(old_path)
                assert old['ranges'] == [[1,155]] and old['sha256'] == g['sha256']
                assert extent['mandatory_lenses'] == list(range(1,14))
                assert extent['new_global_source'] is False and extent['other_lens_coverage_transferred'] is False
            elif x['original_index'] == 2104:
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
            elif x['original_index'] == 2197:
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
            elif x['original_index'] == 2370:
                assert extent['kind'] == 'Frozen Git exact-byte EOF qualification'
                assert n == 930 and extent['raw_requested_range'] == [1,935]
                assert extent['effective_range'] == [1,930]
                assert extent['excluded_nonexistent_ranges'] == [[931,935]]
                assert extent['inherited_metadata'] == meta
                assert extent['frozen_source_locator'] == '6533e444fa37552927070cc8d8190907fd3b413e:'+meta['source_id'].split(':',1)[1]
                assert subprocess.check_output(['git','show',extent['frozen_source_locator']]) == obj.read_bytes()
                old_path = Path(extent['superseded_registration'])
                assert b.sha(old_path) == extent['superseded_registration_sha256']
                old = b.read(old_path)
                assert old['ranges'] == [[1,935]] and old['sha256'] == g['sha256']
                assert extent['mandatory_lenses'] == list(range(1,14))
                assert extent['new_global_source'] is False and extent['other_lens_coverage_transferred'] is False
            elif x['original_index'] == 2376:
                assert extent['kind'] == 'Frozen Git exact-byte EOF qualification'
                assert n == 567 and extent['raw_requested_range'] == [1,568]
                assert extent['effective_range'] == [1,567]
                assert extent['excluded_nonexistent_ranges'] == [[568,568]]
                assert extent['inherited_metadata'] == meta
                assert extent['frozen_source_locator'] == '6533e444fa37552927070cc8d8190907fd3b413e:'+meta['source_id'].split(':',1)[1]
                assert subprocess.check_output(['git','show',extent['frozen_source_locator']]) == obj.read_bytes()
                old_path = Path(extent['superseded_registration'])
                assert b.sha(old_path) == extent['superseded_registration_sha256']
                old = b.read(old_path)
                assert old['ranges'] == [[1,568]] and old['sha256'] == g['sha256']
                assert extent['mandatory_lenses'] == list(range(1,14))
                assert extent['new_global_source'] is False and extent['other_lens_coverage_transferred'] is False
            else:
                raise AssertionError('Unrecognized exact EOF qualification')
            beyond = set(i for a,z in raw_metadata_ranges for i in range(a,z+1) if i > n)
            assert beyond == set(i for a,z in extent['excluded_nonexistent_ranges'] for i in range(a,z+1))
            assert all(1 <= a <= z for a,z in raw_metadata_ranges)
            assert all(1 <= a <= z <= n for a,z in ranges)
            for a,z in raw_metadata_ranges:
                assert set(range(a,min(z,n)+1)) <= covered
        else:
            for a,z in raw_metadata_ranges:
                assert set(range(a,z+1)) <= covered
        if x in j['connected_reinspection']:
            assert x['additional_original_frontier_credit'] == 0
            if x['kind'] in ('Existing later-material full same-lens source read, no original credit','Existing later-material scoped same-lens source read, no original credit'):
                assert 'original_index' not in x and x['population'] == 'later-material'
            elif x['kind'] == 'Earlier original full fresh connected reinspection, no original credit':
                assert x['original_index'] < previous['next_original']
                assert x['fresh_reads'] and x['additional_original_frontier_credit'] == 0
            elif x['kind'] == 'Within-existing original scoped extension':
                assert any(v['original_index']==x['original_index'] for v in j['sources'])
                ext = next(v for v in b.read(b.ROOT/cur['ledger'])['adler_continuation']['prospective_extensions'] if Path(v['registration']).name == regpath.name)
                assert ext['registration_sha256']==b.sha(regpath) and ext['sha256']==g['sha256']
                assert ext['ranges']==g['ranges'] and ext['mandatory_lenses']==list(range(1,14))
                assert ext['new_global_source'] is False and ext['other_lens_coverage_transferred'] is False
                assert set(i for a,z in ext['full_connected_registration_ranges'] for i in range(a,z+1)) <= covered
            elif x['kind'].startswith('Future original'):
                assert x['original_index'] >= j['next_original']
            else:
                assert x['kind'] == 'Already accounted original same-lens connected reinspection'
                assert x['original_index'] < previous['next_original']
                prior = Path(x['prior_full_source_receipt'])
                assert b.sha(prior) == x['prior_full_source_receipt_sha256']
                old = next(v for v in b.read(prior)['sources'] if v['original_index'] == x['original_index'])
                assert old['sha256'] == g['sha256'] and old['scope'] == [[1,n]]
        checks.append(dict(original_index=x.get('original_index'),population=x.get('population','original'),population_index=x.get('population_index'),sha256=g['sha256'],lines=n,registered_ranges=g['ranges'],complementary_same_receipt_scopes=companions,full_file_covered=set(range(1,n+1))<=covered,all_recorded_metadata_ranges_covered=not bool(extent),effective_metadata_ranges_covered=True,excluded_nonexistent_metadata_ranges=extent['excluded_nonexistent_ranges'] if extent else [],proof_kind='qualified same-lens full reuse' if not x['fresh_reads'] else 'fresh exact registered source ranges with inherited EOF qualification' if extent else 'fresh exact registered source ranges'))
    for v in j['same_lens_context']:
        assert b.sha(Path(v['receipt'])) == v['sha256']
    contextual_checks = []
    for role in j['role_dispositions']:
        for context in role.get('context_reinspections',[]):
            rp2 = Path(context['receipt'])
            assert b.sha(rp2) == context['receipt_sha256']
            regpath = Path(context['registration'])
            assert b.sha(regpath) == context['registration_sha256']
            reg = b.read(regpath)
            old = next(v for v in b.read(rp2)['sources'] if v['original_index']==context['original_index'])
            assert old['sha256'] == reg['sha256'] == context['source_sha256']
            assert b.sha(Path(reg['object_path'])) == reg['sha256']
            assert old['registration'] == str(regpath) and old['scope'] == reg['ranges']
            cover = set(i for a,z in old['fresh_reads'] for i in range(a,z+1))
            assert set(i for a,z in reg['ranges'] for i in range(a,z+1)) <= cover
            assert set(i for a,z in context['fresh_reinspection_ranges'] for i in range(a,z+1)) <= cover
            assert context['additional_original_frontier_credit'] == 0
            contextual_checks.append(context)
    extension_checks = []
    ledger = b.read(b.ROOT/cur['ledger'])
    for e in ledger['adler_continuation'].get('prospective_extensions',[]):
        regpath = (b.ROOT/'evidence/continuation-segovb'/e['registration']).resolve()
        registration = b.read(regpath)
        g = next(v for v in registration['sources'] if v['source_id']==e['source_id']) if 'sources' in registration else registration
        assert e['source_id'] == g['source_id'] and e['sha256'] == g['sha256']
        if 'registration_sha256' in e:
            assert b.sha(regpath) == e['registration_sha256']
        if e.get('new_global_source') is False and 'original_index' in e:
            ep = (b.ROOT/'evidence/continuation-segovb'/e['receipt']).resolve()
            extension_receipt = b.read(ep)
            missing = set(i for a,z in e['ranges'] for i in range(a,z+1))
            candidates = [v for v in extension_receipt['sources'] + extension_receipt['connected_reinspection']
                          if v.get('original_index')==e['original_index']
                          and v.get('source_sha256',v.get('sha256'))==e['sha256']
                          and missing <= set(i for a,z in v['fresh_reads'] for i in range(a,z+1))]
            assert candidates, 'No exact fresh source proof covers mandatory extension'
            source = candidates[0]
            if source in extension_receipt['connected_reinspection']:
                assert source['additional_original_frontier_credit'] == 0
                assert source['registration_sha256'] == b.sha(regpath)
                assert source['scope'] == g['ranges']
                if source['kind'].startswith('Future original'):
                    assert e['original_index'] >= extension_receipt['next_original']
                else:
                    assert source['kind']=='Within-existing original scoped extension'
                    assert any(v['original_index']==e['original_index'] for v in extension_receipt['sources'])
            assert source.get('source_sha256',source.get('sha256')) == e['sha256']
            assert set(i for a,z in e['ranges'] for i in range(a,z+1)) <= set(i for a,z in source['fresh_reads'] for i in range(a,z+1))
        if e.get('population') == 'later-material':
            assert g['population'] == e['population'] and g['population_index'] == e['population_index']
            ep = (b.ROOT/'evidence/continuation-segovb'/e['receipt']).resolve()
            source = next(v for v in b.read(ep)['connected_reinspection']
                          if v.get('population')==e['population'] and v.get('population_index')==e['population_index'])
            assert source['source_sha256'] == e['sha256'] and source['scope'] == g['ranges']
            assert g['newly_reached_ranges'] == e['ranges']
            assert set(i for a,z in e['ranges'] for i in range(a,z+1)) <= set(i for a,z in source['fresh_reads'] for i in range(a,z+1))
            population_path = Path(g['population_registry'])
            assert b.sha(population_path) == g['population_registry_sha256']
            assert g['inherited_ranges'] == b.read(population_path)['entries'][e['population_index']]['ranges']
        assert b.sha(Path(g['object_path'])) == g['sha256']
        assert e['mandatory_lenses'] == list(range(1,14))
        assert e['other_lens_coverage_transferred'] is False
        assert set(i for a,z in e['ranges'] for i in range(a,z+1)) <= set(i for a,z in g['ranges'] for i in range(a,z+1))
        extension_checks.append(dict(source_id=e['source_id'],ranges=e['ranges'],registration_sha256=b.sha(regpath),mandatory_lenses=e['mandatory_lenses'],new_global_source=e.get('new_global_source','unreconciled across inherited populations')))
    frontiers = []
    for v in cur['frontiers']:
        assert b.sha(b.ROOT/v['state']) == v['state_sha256']
        frontiers.append(dict(lens=v['lens'],original_accounted=v['original_accounted'],state_sha256=v['state_sha256']))
    assert next(v for v in cur['frontiers'] if v['lens']==11)['original_accounted'] == j['next_original']
    archive = b.ROOT/cur['previous_checkpoint_archive']
    assert b.sha(archive) == archive.stem
    head = subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
    branch = subprocess.check_output(['git','branch','--show-current'],text=True).strip()
    status = subprocess.check_output(['git','status','--porcelain'],text=True)
    assert head == 'b97bcedc262f91de07635c69cd1a4fc4f9a7d13f' and not status
    assert branch == 'handoff/726-audit-ledger-20261027-adler'
    b.save(b.OUT/f'VERIFY-L11-{batch:03}.json',dict(kind='Ignored evidence integrity only; not product test/runtime verification',receipt=str(rp),receipt_sha256=b.sha(rp),state=str(sp),state_sha256=b.sha(sp),previous_verification_sha256=b.sha(b.OUT/f'VERIFY-L11-{predecessor:03}.json'),ledger_sha256=cur['ledger_sha256'],checks=checks,contextual_checks=contextual_checks,extension_checks=extension_checks,frontiers=frontiers,head=head,branch=branch,worktree_status=status,next_original_metadata=[dict(original_index=i,registry_record=registry[i]) for i in range(j['next_original'],min(j['next_original']+6,len(registry)))],implementation_gate_complete=False,accepted_unique_count_change=0,current_reconciled=False,validated=False,qualifications=j['pending_contracts']))
    print(dict(verified=batch,next_original=j['next_original'],source_checks=len(checks),ledger_sha256=cur['ledger_sha256']))

if __name__ == '__main__':
    verify(int(sys.argv[1]),int(sys.argv[2]))
