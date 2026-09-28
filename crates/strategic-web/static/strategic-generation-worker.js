import init, { wasm_generate_job } from "/tactical/wasm/adventuresim-tactical-client.js";

self.onmessage = async event => {
  try {
    if (event.data.module) {
      await init({ module_or_path: event.data.module });
      self.postMessage({ ready: true });
      return;
    }
    const started = performance.now();
    const bytes = wasm_generate_job(event.data.job);
    self.postMessage({ bytes, milliseconds: performance.now() - started }, [bytes.buffer]);
  } catch (error) {
    self.postMessage({ error: String(error) });
  }
};
