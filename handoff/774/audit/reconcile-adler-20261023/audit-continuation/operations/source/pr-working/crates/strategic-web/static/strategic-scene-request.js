import { prepareGeneratedScene } from "./strategic-generation.js";

// A document becomes prepared only after the complete CPU product set arrives.
// GPU readiness belongs to the renderer. Generation shares one residency registry,
// so a replacement waits for the cancelled preparation to release its ownership.
export function createSceneRequests({ runtimePromise, install, changed,
  fetchScene = (...args) => (window.strategicFetch || fetch)(...args),
  prepareScene = prepareGeneratedScene }) {
  let state = Object.freeze({ phase: "idle" });
  let current, residentLocation, preparing;
  const publish = value => { state = Object.freeze(value); changed(state); };

  async function load(request) {
    const { location, settlement, venues, controller } = request;
    const { signal } = controller;
    try {
      signal.throwIfAborted();
      const response = await fetchScene(`/api/scene-assets${settlement
        ? `?settlement=${encodeURIComponent(settlement)}` : ""}`,
      { headers: { Accept: "application/json" }, signal });
      signal.throwIfAborted();
      if (!response.ok) throw new Error(`Could not prepare tactical scene (${response.status})`);
      const input = await response.text();
      signal.throwIfAborted();
      const runtime = await runtimePromise;
      signal.throwIfAborted();
      while (preparing) {
        await preparing.catch(() => {});
        signal.throwIfAborted();
      }
      const owned = prepareScene(runtime, input, venues, { signal });
      preparing = owned;
      let preparation;
      try { preparation = await owned; }
      finally { if (preparing === owned) preparing = undefined; }
      try {
        signal.throwIfAborted();
        install({ location, input, preparation });
      } catch (cause) {
        runtime.wasm_cancel_generation(preparation);
        throw cause;
      }
      residentLocation = location;
      publish({ phase: "prepared", location });
      return { status: "prepared", location };
    } catch (cause) {
      if (signal.aborted || current !== request) return { status: "superseded" };
      publish({ phase: "failed", location, cause });
      return { status: "failed", location, cause };
    } finally {
      if (current === request) current = undefined;
    }
  }

  return {
    get state() { return state; },
    request({ location, settlement, venues }) {
      if (location === residentLocation) {
        current?.controller.abort(); current = undefined;
        publish({ phase: "prepared", location });
        return Promise.resolve({ status: "reused", location });
      }
      if (current?.location === location) return current.promise;
      if (state.phase === "failed" && state.location === location) {
        return Promise.resolve({ status: "failed", location, cause: state.cause });
      }
      current?.controller.abort();
      const request = { location, settlement, venues, controller: new AbortController() };
      current = request;
      request.promise = Promise.resolve().then(() => load(request));
      publish({ phase: "loading", location });
      return request.promise;
    },
    cancel() {
      current?.controller.abort(); current = undefined;
      publish(residentLocation === undefined ? { phase: "idle" }
        : { phase: "prepared", location: residentLocation });
    },
  };
}
