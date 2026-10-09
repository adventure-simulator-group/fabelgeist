// Document-local geographic products. GPU residency and camera readiness belong
// to the persistent renderer; this controller does not prepare settlement jobs.
const WINDOW_VERTICES = 65 * 65;
const RECENT_WINDOW_LIMIT = 4;
const MAX_CONNECTIONS = 8192;
const MAX_CONNECTION_POINTS = 65536;
const connectionKinds = new Set(["land","river","coast","canal","ferry","winter","inferred_walking_link"]);
const scales = new Set(["neighborhood", "district", "region", "country", "continent"]);

export class RegionalEnvironmentLoadError extends Error {
  constructor(code, message, cause) {
    super(message, { cause }); this.name = "RegionalEnvironmentLoadError"; this.code = code;
  }
}

function admitRequest({ source, origin, scale }) {
  if (typeof source !== "string" || !/^[a-f0-9]{64}$/.test(source)
    || !scales.has(scale) || !Number.isInteger(origin?.latitude)
    || !Number.isInteger(origin?.longitude) || Math.abs(origin.latitude) > 90_000_000
    || Math.abs(origin.longitude) > 180_000_000) {
    throw new RegionalEnvironmentLoadError("map/environment-request", "Invalid regional environment request");
  }
  return Object.freeze({ source, scale, origin: Object.freeze({
    latitude: origin.latitude, longitude: origin.longitude,
  }) });
}

function keyFor(request) {
  return `${request.source}/${request.scale}/${request.origin.latitude}/${request.origin.longitude}`;
}

function admitWindow(environment, request) {
  const terrain=environment?.terrain;
  if (terrain?.source !== request.source || terrain?.request?.scale !== request.scale
    || terrain?.request?.origin?.latitude !== request.origin.latitude
    || terrain?.request?.origin?.longitude !== request.origin.longitude
    || !Array.isArray(terrain?.vertices) || terrain.vertices.length !== WINDOW_VERTICES
    || !Array.isArray(environment.connections) || environment.connections.length>MAX_CONNECTIONS) {
    throw new RegionalEnvironmentLoadError("map/environment-response", "Environment response does not match its window");
  }
  let count=0;
  for(const line of environment.connections) {
    if(!connectionKinds.has(line?.kind) || !Array.isArray(line.points) || line.points.length<2
      || (count+=line.points.length)>MAX_CONNECTION_POINTS || line.points.some(point=>
        !Number.isInteger(point?.latitude) || Math.abs(point.latitude)>900_000_000
        || !Number.isInteger(point?.longitude) || Math.abs(point.longitude)>1_800_000_000)) {
      throw new RegionalEnvironmentLoadError("map/environment-response", "Invalid regional connections");
    }
  }
  return environment;
}

export function createRegionalEnvironmentRequests({ runtimePromise, install, changed = () => {},
  fetchEnvironment = (...args) => (window.strategicFetch || fetch)(...args) }) {
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
        const response = await fetchEnvironment(`/api/map/environment/${request.source}?${query}`,
          { headers: { Accept: "application/json" }, cache: "no-store", signal });
        signal.throwIfAborted();
        if (!response.ok) throw new RegionalEnvironmentLoadError("map/environment-http",
          `Could not load map environment (${response.status})`);
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
