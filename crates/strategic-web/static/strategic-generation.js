import { generateJobs } from "./strategic-generation-pool.js";
import { openGeneratedCache } from "./strategic-generation-cache.js";

const CACHE_READ_CONCURRENCY = 4;

export async function prepareGeneratedScene(runtime, input) {
  const started = performance.now();
  // Rust owns numeric parsing: JSON.parse would truncate 64-bit scene seeds.
  const jobs = JSON.parse(runtime.wasm_generation_jobs(input));
  const metrics = { jobs: jobs.length, workers: 0, bytes: 0, workerMilliseconds: 0,
    receiveMilliseconds: 0, milliseconds: 0, cacheHits: 0, cacheMisses: 0, cacheReadMilliseconds: 0 };
  window.strategicGenerationMetrics = metrics;
  const receive = (job, bytes) => {
    metrics.bytes += bytes.byteLength;
    const receiveStarted = performance.now();
    runtime.wasm_receive_job(job, bytes);
    metrics.receiveMilliseconds += performance.now() - receiveStarted;
  };
  const cache = await openGeneratedCache(runtime.generationRevision);
  try {
    let cursor = 0;
    const missing = new Set(), readStarted = performance.now();
    await Promise.all(Array.from({ length: CACHE_READ_CONCURRENCY }, async () => {
      while (cursor < jobs.length) {
        const index = cursor++, job = jobs[index];
        const bytes = await cache.get(job);
        if (bytes) {
          try { receive(job, bytes); metrics.cacheHits++; continue; }
          catch { await cache.remove(job); }
        }
        missing.add(index);
      }
    }));
    metrics.cacheReadMilliseconds = performance.now() - readStarted;
    metrics.cacheMisses = missing.size;
    metrics.workers = await generateJobs(runtime.generationModule,
      jobs.filter((_, index) => missing.has(index)), (job, bytes, milliseconds) => {
        receive(job, bytes);
        metrics.workerMilliseconds += milliseconds;
        cache.put(job, bytes);
      });
    metrics.milliseconds = performance.now() - started;
  } finally {
    // Storage is optional and does not delay asset readiness.
    window.strategicGenerationCacheSettled = cache.close();
  }
}
