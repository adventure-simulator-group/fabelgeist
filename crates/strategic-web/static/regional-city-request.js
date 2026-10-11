import { prepareRegionalCity } from "./strategic-generation.js";

// One focused settlement per document owner. The admitted Rust document and
// ticket stay opaque: JSON rewriting would lose precision in physical seeds.
export function createRegionalCityRequests({ runtimePromise, install, changed = () => {},
  fetchCity = (...args) => (window.strategicFetch || fetch)(...args),
  prepareCity = prepareRegionalCity }) {
  let state = Object.freeze({ phase: "idle" });
  let current, resident, preparing;
  const publish = value => { state = Object.freeze(value); changed(state); };
  const keyFor = request => `${request.source}/${request.place}`;

  async function load(owned) {
    const { request, key, controller } = owned;
    const { signal } = controller;
    try {
      signal.throwIfAborted();
      const query = new URLSearchParams({ place: request.place });
      const response = await fetchCity(`/api/map/city/${request.source}?${query}`,
        { headers: { Accept: "application/json" }, cache: "no-store", signal });
      signal.throwIfAborted();
      if (!response.ok) throw new Error(`Could not prepare city (${response.status})`);
      const document = await response.text();
      signal.throwIfAborted();
      const runtime = await runtimePromise;
      signal.throwIfAborted();
      // Release the previous map owner's preparation before beginning another.
      // Actor scene preparation has independent ownership and can run alongside.
      while (preparing) {
        await preparing.catch(() => {});
        signal.throwIfAborted();
      }
      const ownedPreparation = prepareCity(runtime, document, { signal });
      preparing = ownedPreparation;
      let preparation;
      try { preparation = await ownedPreparation; }
      finally { if (preparing === ownedPreparation) preparing = undefined; }
      try {
        signal.throwIfAborted();
        // Native installation may fail after accepting the command. Residency
        // requires its acknowledgement, not merely successful command sending.
        await install({ document, preparation, signal });
        signal.throwIfAborted();
      } catch (cause) {
        runtime.wasm_cancel_generation(preparation);
        throw cause;
      }
      resident = { request, key };
      publish({ phase: "prepared", request });
      return { status: "prepared", request };
    } catch (cause) {
      if (signal.aborted || current !== owned) return { status: "superseded" };
      publish({ phase: "failed", request, cause });
      return { status: "failed", request, cause };
    } finally {
      if (current === owned) current = undefined;
    }
  }

  const controller = {
    get state() { return state; },
    request({ source, place }, { retry = false } = {}) {
      if (typeof source !== "string" || !/^[a-f0-9]{64}$/.test(source)
        || typeof place !== "string" || !/^place:v1:settlement:(?:[a-f0-9]{2})+$/.test(place)) {
        throw new Error("Invalid focused settlement request");
      }
      const request = Object.freeze({ source, place }), key = keyFor(request);
      if (resident?.key === key) {
        current?.controller.abort(); current = undefined;
        if (state.phase !== "prepared" || keyFor(state.request) !== key) {
          publish({ phase: "prepared", request });
        }
        return Promise.resolve({ status: "reused", request });
      }
      if (current?.key === key) return current.promise;
      if (!retry && state.phase === "failed" && keyFor(state.request) === key) {
        return Promise.resolve({ status: "failed", request, cause: state.cause });
      }
      current?.controller.abort();
      const owned = { request, key, controller: new AbortController() };
      current = owned;
      owned.promise = Promise.resolve().then(() => load(owned));
      publish({ phase: "loading", request });
      return owned.promise;
    },
    retry() {
      return state.phase === "failed" ? controller.request(state.request, { retry: true })
        : Promise.resolve({ status: "unchanged" });
    },
    cancel() {
      current?.controller.abort(); current = undefined;
      publish(resident ? { phase: "prepared", request: resident.request } : { phase: "idle" });
    },
  };
  return controller;
}
