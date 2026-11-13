# Evidence-guard review, initial record

This source-only review covers the pending post-76fe guard and capture and
installer templates. It does not establish that the successor checkpoint was
installed or published, nor does it assess product behavior.

The preserved v2 guard (`062b7c078c4e8428ed64a7c837dc6b50ae9fde5c3e612a8d2f73c0baac788224`)
checked seven unique candidate root paths without naming them. The actual
prospective capture contains the intended seven names. Guard v3
(`b699d18c1afd8cf7b44ec0cefb5e2fc1485c67befbb8395c754030b48cfe4953`)
changes only that assertion to the exact names; the v2 bytes and correction
remain separately preserved. The installer already checks the exact set.

The initial application custody receipt wrapper passed its no-key identity
check. Its optional original prior-inventory replay failed: the wrapper read
`nonce_hex`, while the registered inventory uses `nonce`. A correction must
be separately preserved and checked. Its two replay labels should also be
bound, not just the count and success codes.

The guard checks the captured regular-file set and Git executable bits; the
installer additionally compares the captured directory set and local modes.
The guard checks four closing files for all-or-none presence; the root
entrypoint invokes their separate verifier. Predecessor replay is optional;
the parent Git tree check alone does not prove the current checkout HEAD or
replay a clean predecessor. Final PINS values were not available during this
review. Later closing capture and trusted commit anchoring, independently
held keys and earlier historical objects remain separate dependencies.

The installer checks the expected parent, clean checkout and before-images,
then copies evidence and installs root files sequentially. Its status and log
explicitly qualify partial failure and do not claim an atomic transaction.
No product execution, repair, build, test, runtime probe, historical 3821
recovery, unique defect count or semantic audit closure was performed or
established.
