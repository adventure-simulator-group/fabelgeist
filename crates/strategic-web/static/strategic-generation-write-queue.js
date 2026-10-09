// Optional persistence begins after generated assets are ready. Buffers move
// into this queue after Wasm has consumed them; accepted bytes cannot mutate.
export function createDeferredWrites(write, limits) {
  const pending = new Map();
  let retainedBytes = 0, closed = false, closing;
  return {
    put(job, bytes) {
      const previous = pending.get(job);
      const charge = bytes.buffer.byteLength + job.length * 2;
      if (closed || bytes.byteLength > limits.maximumProductBytes
        || bytes.buffer.byteLength > limits.maximumProductBytes
        || (!previous && pending.size >= limits.maximumProducts)
        || retainedBytes - (previous?.charge ?? 0) + charge > limits.maximumBytes) return false;
      let owned;
      try { owned = structuredClone(bytes, { transfer: [bytes.buffer] }); }
      catch { return false; }
      pending.set(job, { bytes: owned, charge });
      retainedBytes += charge - (previous?.charge ?? 0);
      return true;
    },
    remove(job) {
      const previous = pending.get(job);
      if (previous) { pending.delete(job); retainedBytes -= previous.charge; }
    },
    close() {
      if (closing) return closing;
      closed = true;
      closing = Promise.all(Array.from({ length: Math.min(limits.concurrency, pending.size) }, async () => {
        while (pending.size) {
          const [job, product] = pending.entries().next().value;
          pending.delete(job);
          try { await write(job, product.bytes); }
          finally { retainedBytes -= product.charge; }
        }
      })).finally(() => pending.clear());
      return closing;
    },
  };
}
