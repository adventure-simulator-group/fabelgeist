const CITY_INSTALLATION_DEADLINE_MS = 120_000;

// Only the small ticket is decoded in JavaScript. The canonical document stays
// opaque text; parsing its physical seeds here would lose integer precision.
export async function installRegionalCity(runtime, {document, preparation, signal}) {
  signal.throwIfAborted();
  const ticket = JSON.parse(preparation);
  runtime.wasm_command(JSON.stringify({type:"regional-map", command:{
    type:"install-city", document_json:document, preparation:ticket,
  }}));
  const deadline = performance.now() + CITY_INSTALLATION_DEADLINE_MS;
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
