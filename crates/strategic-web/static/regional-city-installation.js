const CITY_INSTALLATION_DEADLINE_MS = 120_000;

// Only the small ticket is decoded in JavaScript. The canonical document stays
// opaque text; parsing its physical seeds here would lose integer precision.
export async function installRegionalCity(runtime, {request, document, preparation, signal}) {
  signal.throwIfAborted();
  const ticket = JSON.parse(preparation);
  const deadline = performance.now() + CITY_INSTALLATION_DEADLINE_MS;
  // Preparation and terrain publication run independently. Native city
  // admission needs an actual presented window for its geographic frame.
  while (true) {
    signal.throwIfAborted();
    const state = JSON.parse(runtime.wasm_regional_map_status());
    if (state.source === request.source && state.city_requested === request.place
      && state.environment_ready && state.presented && state.requested
      && state.presented.scale === state.requested.scale
      && state.presented.origin.latitude === state.requested.origin.latitude
      && state.presented.origin.longitude === state.requested.origin.longitude) break;
    if (performance.now() >= deadline) throw new Error("City detail terrain presentation timed out");
    await new Promise(resolve => requestAnimationFrame(resolve));
  }
  runtime.wasm_command(JSON.stringify({type:"regional-map", command:{
    type:"install-city", document_json:document, preparation:ticket,
  }}));
  while (true) {
    signal.throwIfAborted();
    const status = JSON.parse(runtime.wasm_regional_map_status()).city_installation;
    if (status?.preparation.owner === ticket.owner
      && status.preparation.sequence === ticket.sequence) {
      if (status.phase === "ready") return;
      if (status.phase === "failed") throw new Error(`City detail unavailable (${status.error})`);
    }
    if (performance.now() >= deadline) throw new Error("City detail installation timed out");
    await new Promise(resolve => requestAnimationFrame(resolve));
  }
}
