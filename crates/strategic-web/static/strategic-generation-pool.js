// Independent CPU workers share compiled code, never a renderer or GPU device.
const MAX_GENERATION_WORKERS = 4;
const JOB_TIMEOUT_MS = 180_000;

/**
 * @typedef {{status: "generate", job: string} | {status: "reused"}} GenerationDecision
 * @typedef {{job: string, bytes: Uint8Array, workerMilliseconds: number}} GeneratedDelivery
 * @typedef {{status: "completed", createdWorkers: number}} GenerationPhaseResult
 */

export class GenerationPoolError extends Error {
  constructor(code, message, cause) {
    super(message, { cause });
    this.name = "GenerationPoolError";
    this.code = code;
  }
}

function workerCapacity(hardwareConcurrency) {
  if (!Number.isSafeInteger(hardwareConcurrency) || hardwareConcurrency < 1) {
    throw new GenerationPoolError("invalid-capacity", "Worker capacity requires a positive processor count");
  }
  return Math.min(MAX_GENERATION_WORKERS, Math.max(1, hardwareConcurrency - 1));
}

export function createGenerationPool(module, options = {}) {
  const createWorker = options.createWorker || (() => new Worker(
    new URL("./strategic-generation-worker.js", import.meta.url), { type: "module" }));
  const concurrency = workerCapacity(options.hardwareConcurrency ?? navigator.hardwareConcurrency ?? 2);
  const workers = [];
  const pending = new Set();
  let phase = "idle", nextDispatch = 1;
  function exchange(worker, message) {
    return new Promise((resolve, reject) => {
      // Dispatch ordinals belong to this pool lifetime, not product/cache identity.
      const dispatch = nextDispatch++;
      if (!Number.isSafeInteger(dispatch)) {
        reject(new GenerationPoolError("dispatch-limit", "Generation dispatch capacity exhausted"));
        return;
      }
      let settled = false;
      const finish = (error, result) => {
        if (settled) return;
        settled = true;
        clearTimeout(timer); pending.delete(cancel);
        worker.onmessage = worker.onerror = worker.onmessageerror = null;
        if (error) reject(error); else resolve(result);
      };
      const cancel = () => finish(new GenerationPoolError("cancelled", "Scene generation cancelled"));
      const timer = setTimeout(() => finish(new GenerationPoolError(
        "worker-timeout", "Scene generation worker timed out")), JOB_TIMEOUT_MS);
      pending.add(cancel);
      worker.onmessage = event => {
        const result = event.data;
        if (!Number.isSafeInteger(result?.dispatch) || result.dispatch < 1) {
          finish(new GenerationPoolError("worker-protocol", "Generation worker omitted its dispatch identity"));
          return;
        }
        if (result.dispatch !== dispatch) return;
        if (result.kind === "failed" && typeof result.error?.message === "string"
          && typeof result.error.code === "string") {
          finish(new GenerationPoolError(result.error.code, result.error.message, result.error));
        } else if (message.kind === "initialize" && result.kind === "ready") {
          finish(null, result);
        } else if (message.kind === "generate" && result.kind === "generated"
          && result.bytes instanceof Uint8Array && result.bytes.buffer instanceof ArrayBuffer
          && Number.isFinite(result.milliseconds)
          && result.milliseconds >= 0) {
          finish(null, result);
        } else {
          finish(new GenerationPoolError("worker-protocol", "Generation worker returned an invalid result"));
        }
      };
      worker.onerror = event => {
        finish(new GenerationPoolError("worker-failed", event.message || "Scene generation worker failed"));
      };
      worker.onmessageerror = () => {
        finish(new GenerationPoolError("transfer-failed", "Scene generation transfer failed"));
      };
      try { worker.postMessage({ ...message, dispatch }, message.dependencies ? [message.dependencies.buffer] : []); }
      catch (cause) { finish(new GenerationPoolError("dispatch-failed", "Scene generation dispatch failed", cause)); }
    });
  }
  async function run(jobs, receive, jobOptions = {}) {
    if (phase !== "idle") throw new GenerationPoolError(phase === "closed" ? "closed" : "busy",
      "Scene generation pool is not available");
    if (!Array.isArray(jobs)) throw new GenerationPoolError("invalid-jobs", "Generation requires a job list");
    phase = "running";
    let cursor = 0, failed = false, created = 0;
    try {
      await Promise.all(Array.from({ length: Math.min(jobs.length, concurrency) }, async (_, index) => {
        let worker = workers[index];
        while (!failed && cursor < jobs.length) {
          const candidate = jobs[cursor++];
          if (typeof candidate !== "string") {
            throw new GenerationPoolError("invalid-job", "Generation jobs require their exact serialized Rust address");
          }
          const decision = jobOptions.resolveJob ? await jobOptions.resolveJob(candidate)
            : { status: "generate", job: candidate };
          if (phase === "closed") throw new GenerationPoolError("cancelled", "Scene generation cancelled");
          if (failed) continue;
          if (decision?.status === "reused") continue;
          if (decision?.status !== "generate") {
            throw new GenerationPoolError("invalid-resolution", "Generation requires an explicit reuse or generate decision");
          }
          const job = decision.job;
          if (typeof job !== "string") {
            throw new GenerationPoolError("invalid-job", "Generation jobs require their exact serialized Rust address");
          }
          if (job !== candidate) throw new GenerationPoolError("job-address-mismatch",
            "Generation resolution changed its requested product address");
          if (!worker) {
            try { worker = createWorker(); }
            catch (cause) { throw new GenerationPoolError("worker-start-failed", "Scene generation worker could not start", cause); }
            workers[index] = worker; created++;
            await exchange(worker, { kind: "initialize", module });
          }
          if (phase === "closed") throw new GenerationPoolError("cancelled", "Scene generation cancelled");
          if (failed) break;
          const dependencies = jobOptions.dependencies?.(job);
          if (dependencies !== undefined && (!(dependencies instanceof Uint8Array)
            || !(dependencies.buffer instanceof ArrayBuffer))) {
            throw new GenerationPoolError("invalid-dependencies", "Generation dependencies require transferable bytes");
          }
          const result = await exchange(worker, { kind: "generate", job, dependencies });
          if (phase === "closed") throw new GenerationPoolError("cancelled", "Scene generation cancelled");
          await receive({ job, bytes: result.bytes, workerMilliseconds: result.milliseconds });
          if (phase === "closed") throw new GenerationPoolError("cancelled", "Scene generation cancelled");
        }
      }).map(promise => promise.catch(error => { failed = true; throw error; })));
    } catch (error) {
      close(); throw error;
    } finally {
      if (phase !== "closed") phase = "idle";
    }
    return { status: "completed", createdWorkers: created };
  }
  function close() {
    phase = "closed";
    for (const cancel of pending) cancel();
    for (const worker of workers.filter(Boolean)) worker.terminate();
    workers.length = 0;
  }
  return { run, close };
}
