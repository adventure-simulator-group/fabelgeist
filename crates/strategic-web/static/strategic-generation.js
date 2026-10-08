import { createGenerationPool } from "./strategic-generation-pool.js";
import { openGeneratedCache } from "./strategic-generation-cache.js";

export async function prepareGeneratedScene(runtime, input, venues) {
  const started = performance.now();
  runtime.wasm_begin_generation();
  // Rust owns numeric parsing: JSON.parse would truncate 64-bit scene seeds.
  const jobs = JSON.parse(runtime.wasm_generation_jobs(input));
  const metrics = { jobs: jobs.length, workers: 0, bytes: 0, workerMilliseconds: 0,
    receiveMilliseconds: 0, dependencyMilliseconds: 0, dependencyBytes: 0,
    milliseconds: 0, cacheHits: 0, cacheMisses: 0, cacheLookupMilliseconds: 0,
    cacheWritesAccepted: 0, cacheWritesReplaced: 0, cacheWriteRejections: {} };
  window.strategicGenerationMetrics = metrics;
  const receive = (job, bytes) => {
    metrics.bytes += bytes.byteLength;
    const receiveStarted = performance.now();
    runtime.wasm_receive_job(job, bytes);
    metrics.receiveMilliseconds += performance.now() - receiveStarted;
  };
  const cache = await openGeneratedCache(runtime.generationRevision);
  const pool = createGenerationPool(runtime.generationModule);
  try {
    // A cache miss starts generation immediately; unrelated cache reads must
    // not postpone the destination's critical scene job. Workers are lazy, so
    // a completely cached destination never instantiates another Wasm runtime.
    const run = async jobs => pool.run(jobs,
      (job, bytes, milliseconds) => {
        receive(job, bytes);
        metrics.workerMilliseconds += milliseconds;
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
        const bytes = runtime.wasm_generation_dependencies(job);
        metrics.dependencyMilliseconds += performance.now() - started;
        metrics.dependencyBytes += bytes.byteLength;
        return bytes;
      }, async resolveJob(job) {
        const readStarted = performance.now();
        const bytes = await cache.get(job);
        metrics.cacheLookupMilliseconds += performance.now() - readStarted;
        if (bytes) {
          try { receive(job, bytes); metrics.cacheHits++; return null; }
          catch { await cache.remove(job); }
        }
        metrics.cacheMisses++;
        return job;
      } });
    const venueJobs = JSON.parse(runtime.wasm_venue_jobs(input, JSON.stringify(venues)));
    metrics.jobs += venueJobs.length;
    // Occupied plans/interiors/meshes and shared exterior programs are independent.
    // Terrain then consumes their prepared plans and compact frontage records.
    // JSON only identifies the enum variant; numeric seeds stay in Rust strings.
    const scenes = jobs.filter(job => Object.hasOwn(JSON.parse(job), "Scene"));
    metrics.workers = await run([...venueJobs, ...jobs.filter(job => !scenes.includes(job))]);
    metrics.workers += await run(scenes);
    const landscapeJobs = JSON.parse(runtime.wasm_landscape_jobs(input, runtime.generationGraphicsConfig));
    metrics.jobs += landscapeJobs.length;
    metrics.workers += await run(landscapeJobs);
    metrics.milliseconds = performance.now() - started;
  } finally {
    pool.close();
    // Storage is optional and does not delay asset readiness.
    window.strategicGenerationCacheSettled = cache.close();
  }
}
