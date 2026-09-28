import { generateJobs } from "./strategic-generation-pool.js";

export async function prepareGeneratedScene(runtime, input) {
  const started = performance.now();
  // Rust owns numeric parsing: JSON.parse would truncate 64-bit scene seeds.
  const jobs = JSON.parse(runtime.wasm_generation_jobs(input));
  const metrics = { jobs: jobs.length, workers: 0, bytes: 0, workerMilliseconds: 0,
    receiveMilliseconds: 0, milliseconds: 0 };
  window.strategicGenerationMetrics = metrics;
  metrics.workers = await generateJobs(runtime.generationModule, jobs, (job, bytes, milliseconds) => {
    metrics.bytes += bytes.byteLength;
    metrics.workerMilliseconds += milliseconds;
    const receiveStarted = performance.now();
    runtime.wasm_receive_job(job, bytes);
    metrics.receiveMilliseconds += performance.now() - receiveStarted;
  });
  metrics.milliseconds = performance.now() - started;
}
