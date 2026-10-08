import init, { wasm_generate_job } from "/tactical/wasm/adventuresim-tactical-client.js";

function protocolError(message) {
  const error = new Error(message);
  error.name = "worker-protocol";
  return error;
}

self.onmessage = async event => {
  const dispatch = event.data?.dispatch;
  try {
    if (!Number.isSafeInteger(dispatch) || dispatch < 1) throw protocolError("Invalid generation dispatch identity");
    const { kind } = event.data;
    if (kind === "initialize") {
      await init({ module_or_path: event.data.module });
      self.postMessage({ kind: "ready", dispatch });
      return;
    }
    if (kind !== "generate" || typeof event.data.job !== "string") throw protocolError("Invalid generation request");
    const started = performance.now();
    const bytes = wasm_generate_job(event.data.job, event.data.dependencies);
    self.postMessage({ kind: "generated", dispatch,
      bytes, milliseconds: performance.now() - started }, [bytes.buffer]);
  } catch (error) {
    self.postMessage({ kind: "failed", dispatch, error: {
      code: error?.name || "worker-execution", message: error?.message || String(error),
    } });
  }
};
