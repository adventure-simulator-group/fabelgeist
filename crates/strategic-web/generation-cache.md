# Generated product persistence

The strategic renderer receives generated products into WebAssembly before
offering their buffers to optional browser persistence. Cache writes begin when
preparation closes the deferred queue; compression and storage never gate scene
readiness. The cache contains disposable geometry, not simulation state.

`strategic-generation-write-queue.js` owns the deferred-write contracts. Its
`DeferredWriteLimits` admits positive safe-integer byte, product-count and
concurrency limits and freezes them for the queue lifetime. Production retains
the 512 MiB pending-byte budget, 512 pending records, two write lanes and 128 MiB
individual backing-buffer limit configured in `strategic-generation-cache.js`.

The queue treats the exact Rust job string as an opaque product address. It does
not parse seeds, normalize JSON or introduce a second cache identity. Persisted
keys remain SHA-256 of the existing format, generator revision and job string;
gzip-compressed product bytes and their checksums retain their existing format.

`put` returns an `accepted` result with `inserted` or `replaced` disposition, or a
`rejected` result with an explicit cause. Queue causes distinguish closure,
invalid addresses or buffers, product/record/byte limits and transfer failure.
The cache additionally reports unavailable storage. A rejected write leaves its
buffer with the caller and preserves any previously queued product. Acceptance
transfers the entire backing buffer, detaching every caller view, while
persistence writes only the accepted view's bytes. Accounting includes the full
backing allocation and the UTF-16 job address. Successful replacement releases
the previous allocation without changing its queue order.

`remove` reports whether a pending product was removed or absent. `snapshot`
reports the accepting, draining or closed phase, pending and in-flight product
counts, and retained bytes. Draining products remain charged until their writes
settle. After closure, all three resource counters are zero.

`close` is idempotent and returns the same promise to every caller. It drains all
lanes even if a writer throws and resolves with a `flushed` or `failed` result,
the number of successful writes, and per-job failure causes. Cache write errors
carry a stable code and retain the underlying storage error. A storage timeout
does not disable subsequent attempts; other storage failures disable the cache.
The cache closes its database after flushing and pruning. These outcomes are
diagnostics for optional persistence and do not reject scene readiness.

`strategic-generation.js` exposes accepted and replaced write counts and
rejections grouped by cause in `window.strategicGenerationMetrics`. The flush
result is available through `window.strategicGenerationCacheSettled`.

Run the queue and worker tests with:

```sh
node --test crates/strategic-web/tests/strategic-generation-write-queue.test.cjs crates/strategic-web/tests/strategic-generation.test.cjs
```

Run the cache and preparation browser checks from `crates/strategic-web` after
installing its Node dependencies:

```sh
node --test tests/strategic-generation-cache.browser.cjs
```

The browser checks cover persisted byte/identity round trips, corruption,
replacement, removal, storage failures and timeouts, and cold versus cached
preparation. A fully cached preparation starts no workers. Unavailable storage
regenerates the same products without delaying readiness for persistence.
