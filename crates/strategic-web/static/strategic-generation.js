import { createGenerationPool } from "./strategic-generation-pool.js";
import { openGeneratedCache } from "./strategic-generation-cache.js";

// Rust owns these exception names. Only invalid persisted products warrant
// eviction and regeneration; unavailable residency or dependencies fail readiness.
const invalidCachedProductCodes = new Set([
  "generation/product-decode", "generation/product-mismatch",
  "generation/scene-bindings-mismatch",
]);

export async function prepareGeneratedScene(runtime, input, venues, { signal, owner = "scene" } = {}) {
  signal?.throwIfAborted();
  const started = performance.now();
  const preparation = runtime.wasm_begin_generation(JSON.stringify(owner));
  let completed = false, cache, pool;
  const cancel = () => pool?.close();
  signal?.addEventListener("abort", cancel, { once: true });
  try {
    // Rust owns numeric parsing: JSON.parse would truncate 64-bit scene seeds.
    const jobs = JSON.parse(runtime.wasm_generation_jobs(preparation, input));
    const metrics = { jobs: jobs.length, workers: 0, bytes: 0, workerMilliseconds: 0,
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
    const run = async jobs => pool.run(jobs,
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
    const venueJobs = JSON.parse(runtime.wasm_venue_jobs(preparation, input, JSON.stringify(venues)));
    metrics.jobs += venueJobs.length;
    // Occupied plans/interiors/meshes and shared exterior programs are independent.
    // Terrain then consumes their prepared plans and compact frontage records.
    // JSON only identifies the enum variant; numeric seeds stay in Rust strings.
    const scenes = jobs.filter(job => Object.hasOwn(JSON.parse(job), "Scene"));
    metrics.workers = (await run([...venueJobs, ...jobs.filter(job => !scenes.includes(job))])).createdWorkers;
    metrics.workers += (await run(scenes)).createdWorkers;
    const landscapeJobs = JSON.parse(runtime.wasm_landscape_jobs(preparation, input, runtime.generationGraphicsConfig));
    metrics.jobs += landscapeJobs.length;
    metrics.workers += (await run(landscapeJobs)).createdWorkers;
    metrics.milliseconds = performance.now() - started;
    signal?.throwIfAborted();
    runtime.wasm_finish_generation(preparation, input);
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
