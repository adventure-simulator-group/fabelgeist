// Independent CPU workers share compiled code, never a renderer or GPU device.
const MAX_GENERATION_WORKERS = 4;
const JOB_TIMEOUT_MS = 180_000;

export async function generateJobs(module, jobs, receive, options = {}) {
  const createWorker = options.createWorker || (() => new Worker(
    new URL("./strategic-generation-worker.js", import.meta.url), { type: "module" }));
  const concurrency = Math.min(jobs.length, MAX_GENERATION_WORKERS,
    Math.max(1, (options.hardwareConcurrency ?? navigator.hardwareConcurrency ?? 2) - 1));
  const workers = [];
  const pending = new Set();
  let cursor = 0, failed = false;
  function exchange(worker, message) {
    return new Promise((resolve, reject) => {
      const finish = (error, result) => {
        clearTimeout(timer); pending.delete(cancel);
        if (error) reject(error); else resolve(result);
      };
      const cancel = () => finish(new Error("Scene generation cancelled"));
      const timer = setTimeout(() => finish(new Error("Scene generation worker timed out")), JOB_TIMEOUT_MS);
      pending.add(cancel);
      worker.onmessage = event => {
        if (event.data.error) finish(new Error(event.data.error));
        else finish(null, event.data);
      };
      worker.onerror = event => {
        finish(new Error(event.message || "Scene generation worker failed"));
      };
      worker.onmessageerror = () => {
        finish(new Error("Scene generation transfer failed"));
      };
      try { worker.postMessage(message, message.dependencies ? [message.dependencies.buffer] : []); }
      catch (error) { finish(error); }
    });
  }
  try {
    await Promise.all(Array.from({ length: concurrency }, async () => {
      let worker;
      while (!failed && cursor < jobs.length) {
        const candidate = jobs[cursor++];
        const job = options.resolveJob ? await options.resolveJob(candidate) : candidate;
        if (failed || job == null) continue;
        if (!worker) {
          worker = createWorker(); workers.push(worker);
          await exchange(worker, { module });
        }
        const dependencies = options.dependencies?.(job);
        const result = await exchange(worker, { job, dependencies });
        await receive(job, result.bytes, result.milliseconds);
      }
    }).map(promise => promise.catch(error => { failed = true; throw error; })));
  } finally {
    for (const cancel of pending) cancel();
    for (const worker of workers) worker.terminate();
  }
  return workers.length;
}
