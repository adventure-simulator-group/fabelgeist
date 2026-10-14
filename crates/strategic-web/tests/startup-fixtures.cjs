const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const directory = process.env.STRATEGIC_STARTUP_FIXTURE_DIR;
const fixtures = new Map();
exports.load = settlement => {
  if (!directory) return null;
  const name = settlement === "travel-destination" ? "b" : "a";
  if (!fixtures.has(name)) {
    // Only metadata is parsed. Preserve the original scene/city text and seeds.
    const actor = fs.readFileSync(path.join(directory, `actor-${name}.json`), "utf8");
    const city = fs.readFileSync(path.join(directory, `city-${name}.json`), "utf8");
    const metadata = JSON.parse(city);
    const environments = JSON.parse(fs.readFileSync(path.join(directory, `environment-${name}.json`)));
    assert(environments.every(product => product.terrain.source === metadata.source));
    fixtures.set(name, { actor, city, metadata, environments,
      origin: environments[0].terrain.request.origin });
  }
  return fixtures.get(name);
};

exports.respond = (url, response) => {
  if (!directory || !url.pathname.startsWith("/api/map/")) return false;
  const products = [exports.load("scene-review"), exports.load("travel-destination")];
  const selected = products.find(product => product.metadata.place === url.searchParams.get("place"));
  let body;
  if (url.pathname.startsWith("/api/map/city/")) {
    assert(selected, "The requested settlement has a canonical fixture");
    assert(url.pathname.endsWith(selected.metadata.source));
    body = selected.city;
  } else {
    const product = products.flatMap(product => product.environments).find(product =>
      product.terrain.source === url.pathname.split("/").at(-1)
      && product.terrain.request.scale === url.searchParams.get("scale")
      && product.terrain.request.origin.latitude === Number(url.searchParams.get("latitude"))
      && product.terrain.request.origin.longitude === Number(url.searchParams.get("longitude")));
    assert(product, "The requested source, origin and terrain scale match the fixture");
    body = JSON.stringify(product);
  }
  response.setHeader("Content-Type", "application/json"); response.end(body);
  return true;
};

exports.markup = settlement => {
  const { origin, metadata } = exports.load(settlement);
  const input = { origin, span: 1200, selected: null, overlay: {
    source: metadata.source, markers: [{ place: metadata.place, origin,
      rank: "town", emphasis: "current" }], route: null,
  } };
  return `<section id="map-fixture" data-regional-map class="strategic-map" tabindex="0" role="region" aria-label="Settlement map">
    <script data-regional-map-input type="application/json">${JSON.stringify(input)}</script>
    <div class="strategic-map-window" data-map-window aria-hidden="true"></div>
    <div class="strategic-map-controls" data-map-foreground>
      <button data-map-action="zoom-in" aria-label="Zoom in">+</button>
      <button data-map-action="zoom-out" aria-label="Zoom out">−</button>
      <button data-map-action="reset" aria-label="Reset view">⌂</button>
      <button data-map-action="retry" aria-label="Retry map" hidden>↻</button>
    </div><div class="strategic-map-markers"><a data-map-place="${metadata.place}" data-map-foreground hidden href="#">Settlement</a></div>
    <p data-map-status data-map-foreground role="status">Loading map…</p></section>`;
};

exports.ready = async page => {
  const settlement = new URL(page.url()).pathname.split("/settlement/")[1]?.split("/")[0];
  const fixture = exports.load(settlement);
  assert(fixture, "Full map timing requires canonical settlement captures");
  await page.waitForFunction(({ source, place, origin }) => {
    const metrics = window.strategicRendererMetrics;
    const state = metrics?.mapState;
    return document.body.hasAttribute("data-strategic-scene-ready")
      && document.body.hasAttribute("data-regional-map-ready")
      && state?.source === source && state.home?.latitude === origin.latitude
      && state.home?.longitude === origin.longitude && state.ready && state.covered
      && state.presentation_ready && state.environment_ready && state.city_requested === place
      && state.presented?.origin.latitude === state.requested?.origin.latitude
      && state.presented?.origin.longitude === state.requested?.origin.longitude
      && state.presented?.scale === state.requested?.scale
      && metrics.cityRequest?.phase === "prepared"
      && state.city_installation?.phase === "ready" && state.city_visible
      && !state.error && !state.city_installation.error
      && metrics.navigations.at(-1)?.path === location.pathname;
  }, { source: fixture.metadata.source, place: fixture.metadata.place, origin: fixture.origin },
  { timeout: 300_000 });
};
