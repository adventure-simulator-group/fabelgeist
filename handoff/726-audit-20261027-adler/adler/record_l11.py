"""Store already-read source dispositions, without running project code."""
import bookkeeping as b
import continuation as c


def record(batch, predecessor, sources, dispositions, contexts, pending, next_index,
           findings=None, connected=None, extensions=None):
    receipts = []
    for index, registration_number, fresh in sources:
        path = b.OUT / f'REGISTRATION-ADLER-{registration_number:03}.json'
        reg = b.read(path)
        assert reg['original_index'] == index
        if reg.get('deleted_original'):
            receipts.append(dict(original_index=index,registration=str(path),
                registration_sha256=b.sha(path),source_id=reg['source_id'],
                sha256=None,scope=[],fresh_reads=[],deleted_original=True,
                absence_evidence=reg['absence_evidence'],base_source=reg['base_source'],
                base_fresh_reads=fresh,owner_proof_qualification=reg['owner_proof_qualification']))
        else:
            receipts.append(dict(original_index=index, registration=str(path),
                                 registration_sha256=b.sha(path),
                                 source_id=reg['source_id'], sha256=reg['sha256'],
                                 scope=reg['ranges'], fresh_reads=fresh))
            if 'inherited_scope_extent_qualification' in reg:
                receipts[-1]['inherited_scope_extent_qualification'] = reg['inherited_scope_extent_qualification']
    rp = b.OUT / f'READ-L11-{batch:03}.json'
    b.save(rp, dict(lens=11, batch=batch,
                   status='Exact registered source ranges accounted WITH findings',
                   sources=receipts, role_dispositions=dispositions,
                   connected_reinspection=connected or [],
                   same_lens_context=[dict(receipt=str(b.OUT/f'READ-L11-{n:03}.json'),
                        sha256=b.sha(b.OUT/f'READ-L11-{n:03}.json')) for n in contexts],
                   pending_contracts=pending,
                   accepted_unique_count_change=0, current_reconciled=False,
                   validated=False, implementation_gate_complete=False,
                   next_original=next_index))
    prev = b.OUT / f'STATE-L11-{predecessor:03}.json'
    sp = b.OUT / f'STATE-L11-{batch:03}.json'
    assert b.read(prev)['next_original'] == sources[0][0]
    assert [x[0] for x in sources] == list(range(sources[0][0], next_index))
    b.save(sp, dict(lens=11, original_accounted=next_index, original_total=2853,
                   next_original=next_index, newly_accounted=[x[0] for x in sources],
                   predecessor=str(prev), predecessor_sha256=b.sha(prev),
                   receipt=str(rp), receipt_sha256=b.sha(rp),
                   all_inherited_qualifications_retained=True,
                   implementation_gate_complete=False, accepted_unique_count_change=0))
    c.advance(11, next_index, sp, rp,
              f'{next_index}/2853 original source-accounted; all other populations and qualifications retained',
              f'Register and read original{next_index}; all13 populations/current/all20/aliases/connected acceptance remain incomplete; no project execution or repairs.',
              findings=findings, extensions=extensions)
