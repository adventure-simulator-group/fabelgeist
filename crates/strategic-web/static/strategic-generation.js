import { createGenerationPool } from "./strategic-generation-pool.js";
import { openGeneratedCache } from "./strategic-generation-cache.js";

// Rust owns these exception names. Only invalid persisted products warrant
// eviction and regeneration; unavailable residency or dependencies fail readiness.
const invalidCachedProductCodes = new Set([
  "generation/product-decode", "generation/product-mismatch",
  "generation/scene-bindings-mismatch",
]);

export async function prepareGeneratedScene(runtime, input, venues, { signal } = {}) {
  return prepareProducts(runtime, "scene", signal, async (preparation, run) => {
    // The strings remain opaque; JSON parsing here only classifies job roles.
    const jobs = JSON.parse(runtime.wasm_generation_jobs(preparation, input));
    const venueJobs = JSON.parse(runtime.wasm_venue_jobs(preparation, input, JSON.stringify(venues)));
    const scenes = jobs.filter(job => Object.hasOwn(JSON.parse(job), "Scene"));
    // Terrain consumes occupied plans and compact frontage records.
    await run([...venueJobs, ...jobs.filter(job => !scenes.includes(job))]);
    await run(scenes);
    await run(JSON.parse(runtime.wasm_landscape_jobs(preparation, input, runtime.generationGraphicsConfig)));
  }, preparation => runtime.wasm_finish_generation(preparation, input));
}

export async function prepareRegionalCity(runtime, document, { signal } = {}) {
  return prepareProducts(runtime, "regional-map", signal, async (preparation, run) => {
    const jobs = JSON.parse(runtime.wasm_regional_city_jobs(
      preparation, document, runtime.generationGraphicsConfig,
    ));
    // Supported terrain and facade programmes are independent. Neither phase
    // requests occupied interiors, landscape scatter or interactive furniture.
    await run(jobs);
  }, preparation => runtime.wasm_finish_regional_city(preparation, document));
}

async function prepareProducts(runtime, owner, signal, prepare, finish) {
  signal?.throwIfAborted();
  const started = performance.now();
  const preparation = runtime.wasm_begin_generation(JSON.stringify(owner));
  let completed = false, cache, pool;
  const cancel = () => pool?.close();
  signal?.addEventListener("abort", cancel, { once: true });
  try {
    const metrics = { jobs: 0, workers: 0, bytes: 0, workerMilliseconds: 0,
      receiveMilliseconds: 0, dependencyMilliseconds: 0, dependencyBytes: 0,
      milliseconds: 0, cacheHits: 0, cacheMisses: 0, cacheLookupMilliseconds: 0,
      cacheWritesAccepted: 0, cacheWritesReplaced: 0, cacheWriteRejections: {} };
    (window.strategicGenerationMetrics ||= {})[owner] = metrics;
    const receive = (job, bytes) => {
      metrics.bytes += bytes.byteLength;
      const receiveStarted = performance.now();
      runtime.wasm_receive_job(preparation, job, bytes);
      metrics.receiveMilliseconds += performance.now() - receiveStarted;
    };
    cache = await openGeneratedCache(runtime.generationRevision);
    pool = createGenerationPool(runtime.generationModule);
    signal?.throwIfAborted();
    // A cache miss starts generation immediately; unrelated cache reads must
    // not postpone the destination's critical scene job. Workers are lazy, so
    // a completely cached destination never instantiates another Wasm runtime.
    const run = async jobs => {
      metrics.jobs += jobs.length;
      const result = await pool.run(jobs,
      ({ job, bytes, workerMilliseconds }) => {
        receive(job, bytes);
        metrics.workerMilliseconds += workerMilliseconds;
        const admission = cache.put(job, bytes);
        if (admission.status === "accepted") {
          metrics.cacheWritesAccepted++;
          if (admission.disposition === "replaced") metrics.cacheWritesReplaced++;
        } else {
          metrics.cacheWriteRejections[admission.reason] =
            (metrics.cacheWriteRejections[admission.reason] ?? 0) + 1;
        }
      }, { dependencies(job) {
        const started = performance.now();
        const bytes = runtime.wasm_generation_dependencies(preparation, job);
        metrics.dependencyMilliseconds += performance.now() - started;
        metrics.dependencyBytes += bytes.byteLength;
        return bytes;
      }, async resolveJob(job, signal) {
        const readStarted = performance.now();
        const bytes = await cache.get(job);
        signal.throwIfAborted();
        metrics.cacheLookupMilliseconds += performance.now() - readStarted;
        if (bytes) {
          try { receive(job, bytes); metrics.cacheHits++; return { status: "reused" }; }
          catch (error) {
            signal.throwIfAborted();
            if (!invalidCachedProductCodes.has(error?.name)) throw error;
            await cache.remove(job);
          }
        }
        signal.throwIfAborted();
        metrics.cacheMisses++;
        return { status: "generate", job };
      } });
      metrics.workers += result.createdWorkers;
    };
    await prepare(preparation, run);
    metrics.milliseconds = performance.now() - started;
    signal?.throwIfAborted();
    finish(preparation);
    completed = true;
    return preparation;
  } finally {
    signal?.removeEventListener("abort", cancel);
    pool?.close();
    if (!completed) runtime.wasm_cancel_generation(preparation);
    // Storage is optional and does not delay asset readiness.
    if (cache) (window.strategicGenerationCacheSettled ||= {})[owner] = cache.close();
  }
}
