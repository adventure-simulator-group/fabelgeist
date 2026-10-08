/** Checked byte, record and write-lane budgets for one deferred flush. */
export class DeferredWriteLimits {
  constructor({ maximumBytes, maximumProductBytes, maximumProducts, concurrency }) {
    const values = { maximumBytes, maximumProductBytes, maximumProducts, concurrency };
    for (const [field, value] of Object.entries(values)) {
      if (!Number.isSafeInteger(value) || value <= 0) {
        throw new RangeError(`Deferred write ${field} must be a positive safe integer`);
      }
    }
    Object.assign(this, values);
    Object.freeze(this);
  }
}

/**
 * @typedef {{status: "accepted", disposition: "inserted" | "replaced"} |
 *   {status: "rejected", reason: "closed" | "invalid-job" | "invalid-buffer" |
 *     "product-limit" | "record-limit" | "byte-limit" | "transfer-failed",
 *     cause?: unknown}} DeferredWriteAdmission
 * @typedef {{job: string, cause: unknown}} DeferredWriteFailure
 * @typedef {{status: "flushed" | "failed", writtenProducts: number,
 *   failures: DeferredWriteFailure[]}} DeferredWriteFlush
 */

// Optional persistence begins after generated assets are ready. The exact Rust
// job string is an opaque address, never parsed or normalized by this queue.
// Buffers move here only after Wasm receipt; rejection preserves caller ownership.
export function createDeferredWrites(write, suppliedLimits) {
  const limits = new DeferredWriteLimits(suppliedLimits);
  const pending = new Map();
  let retainedBytes = 0, inFlightProducts = 0, phase = "accepting", closing;
  return {
    /** @returns {DeferredWriteAdmission} */
    put(job, bytes) {
      if (phase !== "accepting") return { status: "rejected", reason: "closed" };
      if (typeof job !== "string") return { status: "rejected", reason: "invalid-job" };
      if (!(bytes instanceof Uint8Array) || !(bytes.buffer instanceof ArrayBuffer)) {
        return { status: "rejected", reason: "invalid-buffer" };
      }
      const previous = pending.get(job);
      // Charge the full transferred allocation, including unused view bytes,
      // and the UTF-16 job address retained by the Map.
      const charge = bytes.buffer.byteLength + job.length * 2;
      if (bytes.buffer.byteLength > limits.maximumProductBytes) {
        return { status: "rejected", reason: "product-limit" };
      }
      if (!previous && pending.size >= limits.maximumProducts) {
        return { status: "rejected", reason: "record-limit" };
      }
      const nextRetainedBytes = retainedBytes - (previous?.charge ?? 0) + charge;
      if (!Number.isSafeInteger(nextRetainedBytes) || nextRetainedBytes > limits.maximumBytes) {
        return { status: "rejected", reason: "byte-limit" };
      }
      let owned;
      try { owned = structuredClone(bytes, { transfer: [bytes.buffer] }); }
      catch (cause) { return { status: "rejected", reason: "transfer-failed", cause }; }
      pending.set(job, { bytes: owned, charge });
      retainedBytes = nextRetainedBytes;
      return { status: "accepted", disposition: previous ? "replaced" : "inserted" };
    },
    remove(job) {
      const previous = pending.get(job);
      if (!previous) return { status: "absent" };
      pending.delete(job);
      retainedBytes -= previous.charge;
      return { status: "removed" };
    },
    snapshot() {
      return { phase, pendingProducts: pending.size, inFlightProducts, retainedBytes };
    },
    /** @returns {Promise<DeferredWriteFlush>} */
    close() {
      if (closing) return closing;
      phase = "draining";
      let writtenProducts = 0;
      const failures = [];
      const lanes = Array.from({ length: Math.min(limits.concurrency, pending.size) }, async () => {
        while (pending.size) {
          const [job, product] = pending.entries().next().value;
          pending.delete(job);
          inFlightProducts++;
          try { await write(job, product.bytes); writtenProducts++; }
          catch (cause) { failures.push({ job, cause }); }
          finally { retainedBytes -= product.charge; inFlightProducts--; }
        }
      });
      // All lanes settle before closure, including after a failed write. Every
      // accepted allocation is released exactly once; failure is optional-cache
      // diagnostics, not an unhandled rejection that races asset readiness.
      closing = Promise.all(lanes).then(() => {
        phase = "closed";
        return { status: failures.length ? "failed" : "flushed", writtenProducts, failures };
      });
      return closing;
    },
  };
}
