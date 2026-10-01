// Independent CPU workers share compiled code, never a renderer or GPU device.
const MAX_GENERATION_WORKERS = 4;
const JOB_TIMEOUT_MS = 180_000;

export function createGenerationPool(module, options = {}) {
  const createWorker = options.createWorker || (() => new Worker(
    new URL("./strategic-generation-worker.js", import.meta.url), { type: "module" }));
  const concurrency = Math.min(MAX_GENERATION_WORKERS,
    Math.max(1, (options.hardwareConcurrency ?? navigator.hardwareConcurrency ?? 2) - 1));
  const workers = [];
  const pending = new Set();
  let closed = false, running = false;
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
  async function run(jobs, receive, jobOptions = {}) {
    if (closed || running) throw new Error("Scene generation pool is not available");
    running = true;
    let cursor = 0, failed = false, created = 0;
    try {
      await Promise.all(Array.from({ length: Math.min(jobs.length, concurrency) }, async (_, index) => {
        let worker = workers[index];
        while (!failed && cursor < jobs.length) {
          const candidate = jobs[cursor++];
          const job = jobOptions.resolveJob ? await jobOptions.resolveJob(candidate) : candidate;
          if (failed || closed || job == null) continue;
          if (!worker) {
            worker = createWorker(); workers[index] = worker; created++;
            await exchange(worker, { module });
          }
          const dependencies = jobOptions.dependencies?.(job);
          const result = await exchange(worker, { job, dependencies });
          await receive(job, result.bytes, result.milliseconds);
        }
      }).map(promise => promise.catch(error => { failed = true; throw error; })));
    } catch (error) {
      close(); throw error;
    } finally {
      running = false;
    }
    return created;
  }
  function close() {
    closed = true;
    for (const cancel of pending) cancel();
    for (const worker of workers.filter(Boolean)) worker.terminate();
    workers.length = 0;
  }
  return { run, close };
}
