// Document-local geographic products. GPU residency and camera readiness belong
// to the persistent renderer; this controller does not prepare settlement jobs.
const WINDOW_VERTICES = 65 * 65;
const RECENT_WINDOW_LIMIT = 4;
const scales = new Set(["neighborhood", "district", "region", "country", "continent"]);

export class RegionalTerrainLoadError extends Error {
  constructor(code, message, cause) {
    super(message, { cause }); this.name = "RegionalTerrainLoadError"; this.code = code;
  }
}

function admitRequest({ source, origin, scale }) {
  if (typeof source !== "string" || !/^[a-f0-9]{64}$/.test(source)
    || !scales.has(scale) || !Number.isInteger(origin?.latitude)
    || !Number.isInteger(origin?.longitude) || Math.abs(origin.latitude) > 90_000_000
    || Math.abs(origin.longitude) > 180_000_000) {
    throw new RegionalTerrainLoadError("map/terrain-request", "Invalid regional terrain request");
  }
  return Object.freeze({ source, scale, origin: Object.freeze({
    latitude: origin.latitude, longitude: origin.longitude,
  }) });
}

function keyFor(request) {
  return `${request.source}/${request.scale}/${request.origin.latitude}/${request.origin.longitude}`;
}

function admitWindow(window, request) {
  if (window?.source !== request.source || window?.request?.scale !== request.scale
    || window?.request?.origin?.latitude !== request.origin.latitude
    || window?.request?.origin?.longitude !== request.origin.longitude
    || !Array.isArray(window.vertices) || window.vertices.length !== WINDOW_VERTICES) {
    throw new RegionalTerrainLoadError("map/terrain-response", "Terrain response does not match its window");
  }
  return window;
}

export function createRegionalTerrainRequests({ runtimePromise, install, changed = () => {},
  fetchTerrain = (...args) => (window.strategicFetch || fetch)(...args) }) {
  const recent = new Map();
  let state = Object.freeze({ phase: "idle" });
  let current, resident;
  const publish = value => { state = Object.freeze(value); changed(state); };

  async function load(owned) {
    const { request, key, controller } = owned;
    const { signal } = controller;
    try {
      signal.throwIfAborted();
      let product = recent.get(key);
      const reused = product !== undefined;
      if (!reused) {
        const query = new URLSearchParams({ latitude: request.origin.latitude,
          longitude: request.origin.longitude, scale: request.scale });
        const response = await fetchTerrain(`/api/map/terrain/${request.source}?${query}`,
          { headers: { Accept: "application/json" }, cache: "no-store", signal });
        signal.throwIfAborted();
        if (!response.ok) throw new RegionalTerrainLoadError("map/terrain-http",
          `Could not load terrain (${response.status})`);
        product = admitWindow(await response.json(), request);
        signal.throwIfAborted();
      }
      await runtimePromise;
      signal.throwIfAborted();
      if (resident?.key !== key) install(product);
      resident = { key, request };
      recent.delete(key); recent.set(key, product);
      while (recent.size > RECENT_WINDOW_LIMIT) recent.delete(recent.keys().next().value);
      publish({ phase: "prepared", request });
      return { status: reused ? "reused" : "prepared", request };
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
    request(input, { retry = false } = {}) {
      const request = admitRequest(input);
      const key = keyFor(request);
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
