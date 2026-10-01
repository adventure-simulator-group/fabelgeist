const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const http = require("node:http");
const test = require("node:test");
const { chromium } = require("playwright");

const root = path.resolve(__dirname, "../../..");
const reviewRoot = path.resolve(root, process.env.STRATEGIC_SCENE_REVIEW_DIR || "target/strategic-scene-review");
const realRenderer = process.env.STRATEGIC_RENDER_BENCHMARK === "1";
const sceneFixture = process.env.STRATEGIC_SCENE_INPUT || "assets/tactical-scenes/massive-city.json";
const peoplePerPlace = Number(process.env.STRATEGIC_SCENE_PEOPLE_PER_PLACE || 2);
const residents = Array.from({length: peoplePerPlace}, (_, index) => index);
const services = ["public-square", "residences", "keep", "merchants", "weapons", "armor", "clothing", "herbalist", "books", "inn", "religion"];
const town = "/locations/settlement/scene-review";
const personId = (place, offset = 0) => String(100 + services.indexOf(place) * peoplePerPlace + offset);
const styles = ["base", "reset", "layout", "components", "strategic", "architecture", "utilities", "workspace", "portraits", "chat-dock", "readability", "strategic-scene"];

function fixture(url) {
  const settlement = url.pathname.split("/settlement/")[1]?.split("/")[0] || "scene-review";
  const town = `/locations/settlement/${settlement}`;
  if (url.pathname === "/journal") return `<!doctype html><html><head><title>Journal</title></head><body>
    <div id="strategic-render-surface"><canvas id="game-canvas"></canvas></div>
    <!-- strategic-page-start --><div id="strategic-page" data-strategic-workspace data-script-profile="strategic" data-page-title="Journal">
      <main id="journal-fixture">Journal remains HTML</main><a href="${town}/places/inn">Return</a>
    </div><!-- strategic-page-end --><script src="/static/strategic-navigation.js"></script>
    <script type="module" src="/static/strategic-renderer.js"></script></body></html>`;
  const selected = url.pathname.split("/places/")[1] || "map";
  const nav = ["map", ...services].map(id => `<a href="${id === "map" ? town : `${town}/places/${id}`}" class="nav-tab ${selected === id ? "active" : ""}" data-building-id="${id}" data-service-id="${id}" aria-label="${id}"><span class="service-tab-label">${id}</span></a>`).join("");
  const portraits = residents.map(offset => `<button data-person="${personId(selected, offset)}" aria-label="Talk to resident ${offset}" class="party-portrait-select"><span class="party-portrait-initial"><span data-bevy-character="${personId(selected, offset)}"></span></span><span>Resident ${offset}</span></button>`).join("");
  return `<!doctype html><html><head>${styles.map(style => `<link rel="stylesheet" href="/static/css/${style}.css">`).join("")}<title>Scene review</title></head><body>
    <div id="strategic-render-surface"><canvas id="game-canvas"></canvas></div>
    <!-- strategic-page-start --><div id="strategic-page" class="app" data-strategic-workspace data-script-profile="strategic" data-page-title="Scene review" data-path="${url.pathname}">
    <header class="top-bar settlement-top-bar" data-environment="settlement"><nav class="settlement-services" data-settlement-id="${settlement}">${nav}</nav></header>
    <div class="main-grid"><aside class="left-sidebar">Character details</aside>
    <main class="center-content settlement-main ${selected === "map" ? "settlement-map-main" : ""}">
    ${selected === "map" ? '<div id="map-fixture">Map remains HTML</div>' : `<div class="party-portrait-overlay">${portraits}</div><section class="visual-stage npc-description-stage"><h2>${selected}</h2><p>Resident's description</p></section>`}
    </main><aside class="right-sidebar">Available actions</aside></div>
    <aside data-chat-dock><section class="settlement-chat">Conversation</section></aside></div><!-- strategic-page-end -->
    <script>document.addEventListener('click',event=>{const target=event.target.closest('[data-person]');if(target)document.dispatchEvent(new CustomEvent('strategic-character-selected',{detail:{id:target.dataset.person}}));});</script>
    <script src="/static/location-urls.js"></script><script src="/static/strategic-navigation.js"></script>
    <script type="module" src="/static/strategic-renderer.js"></script></body></html>`;
}

async function serve() {
  const requests = [], missing = [];
  const carry = realRenderer ? JSON.parse(fs.readFileSync(path.join(reviewRoot, "fixtures/carry.json"), "utf8")) : null;
  const server = http.createServer((request, response) => {
    const url = new URL(request.url, "http://localhost"); requests.push(url.pathname);
    if (url.pathname === "/api/scene-assets") {
      response.setHeader("Content-Type", "application/json");
      let input = fs.readFileSync(path.resolve(root, sceneFixture), "utf8");
      if (url.searchParams.get("settlement") === "travel-destination") {
        // A distinct terrain/scene with validated occupied building layouts.
        // Preserve full-width seeds; do not parse this document through JS numbers.
        input = input.replace(/("seed"\s*:\s*)(\d+)/,
          (_, prefix, seed) => prefix + BigInt.asUintN(64, BigInt(seed) + 1n))
          .replace(/("scene_key"\s*:\s*)"[^"]+"/, '$1"travel-destination"');
      }
      response.end(input); return;
    }
    if (url.pathname === "/api/scene-equipment") {
      response.setHeader("Content-Type", "application/json");
      const outfit = (id) => {
        const equipment = [
        ["linen_tunic", "chest", "base_clothing"],
        ["linen_breeches", "left_leg", "base_clothing"],
        ["leather_belt", "front_belt", "accessory"],
        ...(Number(id) % 2 ? [["barbute", "head", "rigid_armor"], ["breastplate", "chest", "rigid_armor"]] : []),
      ].map(([item, location, channel], index) => ({id: `${id}${index}`, item, placement: "worn",
        occupancies: [{anchor: {Character: location}, channel, order: 0, requirement_index: 0, capacity_index: 0}], weapon: null, holder: null}));
        if (carry) {
          equipment.push({id: `${id}5`, item: "scabbard", placement: "belt",
            occupancies: [{anchor: {Attachment: {parent: `${id}2`, point: "left"}}, channel: "mount", order: 0, requirement_index: 0, capacity_index: 0}],
            weapon: null, holder: carry.holder});
          equipment.push({id: `${id}6`, item: "longsword", placement: "held",
            occupancies: [{anchor: {Character: "right_hand"}, channel: "held", order: 0, requirement_index: 0, capacity_index: 0}],
            weapon: carry.weapon, holder: null});
        }
        return equipment;
      };
      response.end(JSON.stringify(url.searchParams.get("characters").split(",").map(id => ({id, equipment: outfit(id)})))); return;
    }
    if (url.pathname === "/renderer-capabilities") { response.end("<!doctype html><title>WebGPU probe</title>"); return; }
    if (url.pathname.startsWith("/api/locations/")) {
      const place = url.pathname.split("/places/")[1]?.split("/")[0];
      response.setHeader("Content-Type", "application/json");
      response.end(JSON.stringify(residents.map(offset => ({ id: personId(place, offset), name: `Resident ${offset}` })))); return;
    }
    if (!realRenderer && url.pathname === "/tactical/wasm/adventuresim-tactical-client.js") {
      response.setHeader("Content-Type", "text/javascript");
      const street = {height: 30, width: services.length * 20, bays: services.map((id, index) => ({id, width: index % 2 ? 18 : 24}))};
      response.end(`export default async function(){}; export function wasm_begin_generation(){} export function wasm_generation_jobs(){return "[]";} export function wasm_venue_jobs(){return "[]";} export function wasm_boot(){window.boots=(window.boots||0)+1;} export function wasm_command(json){(window.commands||=[]).push(JSON.parse(json));} export function wasm_strategic_status(){return JSON.stringify({ready:true,street:${JSON.stringify(street)},revision:window.commands?.filter(command=>command.type==="sync-strategic-view").at(-1)?.view.revision})}`); return;
    }
    if (!realRenderer && url.pathname === "/tactical/wasm/adventuresim-tactical-client_bg.wasm") {
      response.setHeader("Content-Type", "application/wasm");
      response.end(Buffer.from([0, 97, 115, 109, 1, 0, 0, 0])); return;
    }
    let file;
    if (url.pathname.startsWith("/static/")) file = path.join(root, "crates/strategic-web/static", url.pathname.slice(8));
    if (url.pathname.startsWith("/tactical/")) file = path.join(root, "crates/adventuresim-stdb-module/static", url.pathname.slice(10));
    if (process.env.STRATEGIC_WASM_DIR && url.pathname.startsWith("/tactical/wasm/"))
      file = path.join(root, process.env.STRATEGIC_WASM_DIR, path.basename(url.pathname));
    if (file) {
      const type = { ".js": "text/javascript", ".css": "text/css", ".wasm": "application/wasm", ".json": "application/json" }[path.extname(file)] || "application/octet-stream";
      response.setHeader("Content-Type", type); response.setHeader("Cache-Control", "public, max-age=3600");
      if (!fs.existsSync(file)) {
        missing.push(url.pathname);
        fs.writeFileSync(path.join(reviewRoot, "missing-assets.json"), JSON.stringify(missing, null, 2));
        response.statusCode = 404; response.end(); return;
      }
      if (process.env.STRATEGIC_STARTUP_PROFILE === "1" && url.pathname.startsWith("/static/") && file.endsWith(".js")) {
        response.end(require("./strategic-startup-profile.cjs").rewrite(path.basename(file), fs.readFileSync(file, "utf8")));
      } else fs.createReadStream(file).pipe(response);
      return;
    }
    response.setHeader("Content-Type", "text/html"); response.setHeader("X-Strategic-Response", "root");
    response.setHeader("X-Strategic-Script-Profile", "strategic"); response.setHeader("X-Strategic-Canonical-Url", url.pathname);
    response.end(fixture(url));
  });
  await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
  return { server, requests, missing, origin: `http://127.0.0.1:${server.address().port}` };
}

test("one canvas retains street, portraits and character views across warm navigation", { timeout: realRenderer ? 1_800_000 : 60_000 }, async () => {
  const { server, requests, missing, origin } = await serve();
  const browser = await chromium.launch({ headless: true,
    channel: process.platform === "win32" ? "msedge" : undefined,
    args: realRenderer ? ["--enable-unsafe-webgpu"] : [] });
  const errors = [], samples = [];
  const warnings = [];
  const output = path.join(reviewRoot, peoplePerPlace > 2 ? "stress" : ""); fs.mkdirSync(output, { recursive: true });
  fs.writeFileSync(path.join(output, "equipment-diagnostics.log"), "");
  let telemetry, initialLoad;
  let rejectFatal;
  const fatal = new Promise((_, reject) => { rejectFatal = reject; });
  fatal.catch(() => {});
  try {
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
    if (process.env.STRATEGIC_GPU_PROFILE === '1')
      await page.addInitScript({path: path.join(__dirname, 'webgpu-probe.js')});
    const recordError = text => {
      errors.push(text);
      fs.writeFileSync(path.join(output, "live-errors.json"), JSON.stringify(errors.slice(-10), null, 2));
      if (/panicked|unreachable|strategic scene failed|validation error|failed to process shader/i.test(text)) rejectFatal(new Error(text));
    };
    page.on("pageerror", error => recordError(error.message));
    page.on("console", message => {
      if (/equipment|armor body|bracer design|breastplate design|strategic snapshot|GPU city|City fixed boundary/i.test(message.text())) {
        fs.appendFileSync(path.join(output, "equipment-diagnostics.log"), `${message.text().slice(0, 1500)}\n`);
      }
      if (message.type() === "error") recordError(message.text().slice(0, 12000));
      if (message.type() === "warning") {
        warnings.push(message.text().slice(0, 500));
        fs.writeFileSync(path.join(output, "warnings.json"), JSON.stringify(warnings.slice(-20), null, 2));
      }
    });
    await page.addInitScript(() => {
      window.rendererFrames = [];
      let previous;
      function frame(now) { if (previous) window.rendererFrames.push(now - previous); previous = now;
        if (window.rendererFrames.length > 300) window.rendererFrames.shift(); requestAnimationFrame(frame); }
      requestAnimationFrame(frame);
    });
    if (realRenderer) {
      await page.goto(`${origin}/renderer-capabilities`);
      const adapter = await page.evaluate(async () => {
        const adapter = await navigator.gpu?.requestAdapter();
        return adapter ? { vendor: adapter.info.vendor, architecture: adapter.info.architecture, description: adapter.info.description } : null;
      });
      fs.writeFileSync(path.join(output, "adapter.json"), JSON.stringify(adapter, null, 2));
      assert(adapter, "The benchmark requires an actual WebGPU adapter");
      fs.writeFileSync(path.join(output, "adapter.json"), JSON.stringify({...adapter, browser: browser.version(), captured: new Date().toISOString()}, null, 2));
    }
    const startup = process.env.STRATEGIC_STARTUP_PROFILE === "1"
      ? await require("./strategic-startup-profile.cjs").attach(page, output) : null;
    await startup?.start();
    await page.goto(`${origin}${town}/places/${process.env.STRATEGIC_PROFILE_PLACE || 'inn'}`);
    await page.evaluate(() => { window.originalCanvas = document.querySelector("#game-canvas"); });
    telemetry = setInterval(async () => {
      const state = await page.evaluate(() => ({ metrics: window.strategicRendererMetrics, generation: window.strategicGenerationMetrics, probe: window.renderProbe?.status?.(),
        status: document.querySelector('#strategic-scene-status')?.textContent })).catch(error => ({ error: error.message }));
      fs.writeFileSync(path.join(output, "live-state.json"), JSON.stringify(state, null, 2));
    }, 5000);
    const ready = () => Promise.race([fatal, page.waitForFunction(() => document.body.hasAttribute("data-strategic-scene-ready"), null, { timeout: realRenderer ? 1_500_000 : 15_000 })]);
    await ready();
    await startup?.stop("cold-startup");
    if (process.env.STRATEGIC_TRAVEL_BENCHMARK === "1") {
      await require("./strategic-travel-benchmark.cjs").run(page, output, ready, startup);
      assert.deepEqual(errors, []);
      return;
    }
    if (process.env.STRATEGIC_RELOAD_BENCHMARK === "1") {
      await page.evaluate(() => window.strategicGenerationCacheSettled);
      initialLoad = await page.evaluate(() => ({ metrics: window.strategicRendererMetrics,
        generation: window.strategicGenerationMetrics }));
      initialLoad.storage = await page.evaluate(() => new Promise((resolve, reject) => {
        const request = indexedDB.open("fabelgeist-generated-assets", 1);
        request.onerror = () => reject(request.error);
        request.onsuccess = () => {
          const db = request.result, tx = db.transaction("products", "readonly");
          const totals = { records: 0, bytes: 0, decodedBytes: 0 };
          tx.oncomplete = () => { db.close(); resolve(totals); };
          tx.onerror = () => reject(tx.error);
          const cursor = tx.objectStore("products").openCursor();
          cursor.onsuccess = () => {
            if (!cursor.result) return;
            const record = cursor.result.value;
            totals.records++; totals.bytes += record.size;
            totals.decodedBytes += record.decodedSize;
            cursor.result.continue();
          };
        };
      }));
      fs.writeFileSync(path.join(output, "initial-load.json"), JSON.stringify(initialLoad, null, 2));
      if (process.env.STRATEGIC_RELOAD_CLEAR_CACHE === "1") {
        await page.evaluate(() => new Promise((resolve, reject) => {
          const request = indexedDB.deleteDatabase("fabelgeist-generated-assets");
          request.onsuccess = resolve; request.onerror = () => reject(request.error);
          request.onblocked = () => reject(new Error("Generated cache is still open"));
        }));
      }
      await startup?.start();
      await page.reload(); await ready();
      await startup?.stop("reload-startup");
      await page.evaluate(() => { window.originalCanvas = document.querySelector("#game-canvas"); });
    }
    const generationAtReady = await page.evaluate(() => window.strategicGenerationMetrics);
    assert.equal(await page.locator("canvas").count(), 1, "all views share one DOM canvas");
    if (!realRenderer) {
      // Forge teardown must not discard the retained strategic view on ordinary
      // page navigation. Exercise the actual mount/unmount handlers.
      await page.evaluate(() => {
        const host = document.createElement("section"); host.dataset.bevyScene = "forge";
        document.querySelector("#strategic-page").append(host);
        document.dispatchEvent(new Event("strategic-page-mounted"));
      });
      await page.locator('[data-bevy-scene="forge"][data-renderer-ready]').waitFor();
      await page.evaluate(() => {
        document.dispatchEvent(new Event("strategic-page-unmounting"));
        document.querySelector('[data-bevy-scene="forge"]').remove();
        document.dispatchEvent(new Event("strategic-page-mounted"));
      });
      await ready();
      assert.equal(await page.evaluate(() => window.commands.filter(command => command.type === "hide-forge-preview").length), 1);
      assert.equal(await page.evaluate(() => window.commands.some(command => command.type === "hide-strategic-scene")), false);
      await page.waitForFunction(() => {
        const links = [...document.querySelectorAll('.settlement-services [data-building-id]:not([data-building-id="map"])')];
        const view = window.commands.filter(command => command.type === "sync-strategic-view").at(-1).view;
        const width = links.at(-1).getBoundingClientRect().right - links[0].getBoundingClientRect().left;
        return !view.frontages && Math.abs(view.street.full_width - width * devicePixelRatio) <= 1
          && links[0].getBoundingClientRect().width > links[1].getBoundingClientRect().width;
      });
    }
    if (process.env.STRATEGIC_GPU_PROFILE === '1') {
      await require('./strategic-gpu-profile.cjs').profile(page, output, ready);
      fs.writeFileSync(path.join(output, 'profile-errors.json'), JSON.stringify({missing, errors}, null, 2));
      if (process.env.STRATEGIC_PROFILE_AND_NAVIGATION !== '1') {
        assert.deepEqual(errors, []);
        return;
      }
    }
    if (!realRenderer) {
      await page.evaluate(() => {
        const scene = document.querySelector(".bevy-scene-window").getBoundingClientRect();
        const chat = document.querySelector("[data-chat-dock]");
        Object.assign(chat.style, {left: `${scene.left + 20}px`, top: `${scene.top + 10}px`, bottom: "auto", width: "200px", height: "50px"});
        dispatchEvent(new Event("resize"));
      });
      await page.waitForFunction(() => {
        const chat = document.querySelector("[data-chat-dock]").getBoundingClientRect();
        const x = chat.left + 10, y = chat.top + 10;
        const path = document.querySelector("#strategic-render-surface").style.clipPath;
        const rectangles = [...path.matchAll(/M\s*([\d.]+)[ ,]+([\d.]+)\s*H\s*([\d.]+)\s*V\s*([\d.]+)/g)];
        return rectangles.length > 0 && !rectangles.some(([, left, top, right, bottom]) => x >= +left && x < +right && y >= +top && y < +bottom);
      });
      await page.evaluate(() => { document.querySelector("[data-chat-dock]").removeAttribute("style"); dispatchEvent(new Event("resize")); });
    }
    fs.writeFileSync(path.join(output, "cold-requests.json"), JSON.stringify(requests, null, 2));
    const steadyFrames = await page.evaluate(() => new Promise(resolve => {
      const samples = []; let previous;
      function sample(now) { if (previous) samples.push(now - previous); previous = now;
        if (samples.length === 120) resolve(samples); else requestAnimationFrame(sample); }
      requestAnimationFrame(sample);
    }));
    const screenshot = await page.screenshot({ path: path.join(output, "desktop.png") });
    if (realRenderer) {
      const pixels = await page.evaluate(async encoded => {
        const bytes = Uint8Array.from(atob(encoded), character => character.charCodeAt(0));
        const image = await createImageBitmap(new Blob([bytes], {type: "image/png"}));
        const analysis = new OffscreenCanvas(image.width, image.height);
        const context = analysis.getContext("2d", {willReadFrequently: true}); context.drawImage(image, 0, 0);
        return ["[data-building-id='residences']", ".bevy-scene-window", "[data-bevy-character]"].map(selector => {
          const rect = document.querySelector(selector).getBoundingClientRect();
          const height = rect.height - (selector.includes("building-id") ? 34 : 0);
          const data = context.getImageData(Math.ceil(rect.x + 3), Math.ceil(rect.y + 3),
            Math.floor(rect.width - 6), Math.floor(height - 6)).data;
          const colors = new Set();
          let lit = 0;
          for (let offset = 0; offset < data.length; offset += 4) {
            colors.add((data[offset] << 16) | (data[offset + 1] << 8) | data[offset + 2]);
            if (Math.max(data[offset], data[offset + 1], data[offset + 2]) > 32) lit++;
          }
          return {selector, colors: colors.size, litFraction: lit / (data.length / 4)};
        });
      }, screenshot.toString("base64"));
      fs.writeFileSync(path.join(output, "rendered-pixels.json"), JSON.stringify(pixels, null, 2));
      assert(pixels.every(region => region.colors > 8), "building, central scene and portrait must contain rendered detail");
      assert(pixels.every(region => region.litFraction > 0.1), "daytime fixture views must be readable, not merely nonblank");
    }
    fs.writeFileSync(path.join(output, "layout.json"), JSON.stringify(await page.evaluate(() =>
      [".settlement-top-bar", ".settlement-services", "[data-building-id='public-square']", "#game-canvas"].map(selector => {
        const el = document.querySelector(selector), style = getComputedStyle(el);
        return {selector, rect: el.getBoundingClientRect().toJSON(), height: style.height,
          maxHeight: style.maxHeight, transform: style.transform, clip: document.querySelector('#strategic-render-surface').style.clipPath};
      })), null, 2));
    const coldRequests = requests.length;
    if (process.env.STRATEGIC_NAVIGATION_PROFILE === "1") await startup?.beginNavigation();
    for (const place of [...services, "inn"]) {
      const started = performance.now();
      await page.locator(`[data-building-id="${place}"]`).click();
      await page.waitForFunction(expected => document.querySelector("#strategic-page").dataset.path.endsWith(`/places/${expected}`), place);
      await ready();
      await page.waitForFunction(() => window.strategicRendererMetrics.navigations.at(-1)?.path === location.pathname);
      samples.push({ place, milliseconds: performance.now() - started });
      assert.equal(await page.evaluate(() => window.originalCanvas === document.querySelector("#game-canvas")), true);
      const previousSamples = await page.evaluate(() => window.strategicRendererMetrics.navigations.length);
      const portraitStarted = performance.now();
      await page.locator("[data-person]").last().click();
      await page.waitForFunction(count => window.strategicRendererMetrics.navigations.length > count, previousSamples);
      samples.push({ portrait: personId(place, peoplePerPlace - 1), milliseconds: performance.now() - portraitStarted });
      if (realRenderer) {
        fs.mkdirSync(path.join(output, "venues"), {recursive: true});
        await page.screenshot({path: path.join(output, "venues", `${place}.png`)});
        fs.writeFileSync(path.join(output, "venues", `${place}.json`), JSON.stringify(
          await page.evaluate(() => window.strategicRendererMetrics.state), null, 2));
      }
    }
    if (process.env.STRATEGIC_NAVIGATION_PROFILE === "1") await startup?.endNavigation();
    assert.equal(requests.slice(coldRequests).filter(url => url.startsWith("/tactical/")).length, 0, "warm navigation loads no renderer assets");
    assert.equal(requests.slice(coldRequests).filter(url => url.startsWith("/api/scene-equipment")).length, 0, "warm navigation reuses equipment appearances");
    assert.equal(requests.slice(coldRequests).filter(url => url === "/api/scene-assets").length, 0, "warm navigation reuses tactical scene document");
    await page.locator('[data-building-id="map"]').click();
    await page.locator("#map-fixture").waitFor();
    await page.goBack();
    await page.locator("[data-person]").first().waitFor();
    const beforeJournal = requests.length;
    await page.evaluate(() => {
      const link = document.createElement("a"); link.href = "/journal";
      document.body.append(link); link.click(); link.remove();
    });
    await page.locator("#journal-fixture").waitFor();
    await page.waitForFunction(() => document.querySelector("#strategic-render-surface").style.clipPath === "inset(100%)");
    await page.goBack();
    await page.locator("[data-person]").first().waitFor();
    await ready();
    assert.equal(requests.slice(beforeJournal).filter(url => url.startsWith("/api/scene-") || url.startsWith("/api/locations/")).length, 0,
      "HTML-only pages retain the city's document, roster and equipment");
    assert.equal(await page.evaluate(() => window.originalCanvas === document.querySelector("#game-canvas")), true);
    const beforeResize = realRenderer ? await page.evaluate(() => window.strategicRendererMetrics.state.revision) : 0;
    await page.setViewportSize({ width: 390, height: 844 });
    if (realRenderer) {
      await page.waitForFunction(previous => {
        const state = window.strategicRendererMetrics.state;
        return state.revision > previous && state.ready && document.querySelector('#game-canvas').width === 390;
      }, beforeResize, {timeout: 120_000});
      await ready();
    }
    if (!realRenderer) await page.waitForFunction(() => {
      const view = window.commands.filter(command => command.type === "sync-strategic-view").at(-1)?.view;
      return view?.street && view.street.full_width > view.street.width;
    });
    await page.screenshot({ path: path.join(output, "narrow.png"), fullPage: true });
    if (!realRenderer) {
      const state = await page.evaluate(() => ({ boots: window.boots, commands: window.commands }));
      assert.equal(state.boots, 1);
      const views = state.commands.filter(command => command.type === "sync-strategic-view").map(command => command.view);
      assert(views.some(view => view.selected === personId("inn", peoplePerPlace - 1)));
      assert(views.some(view => view.active_place === null && view.stage === null));
      assert(views.at(-1).street.full_width > views.at(-1).street.width);
      const before = views.length;
      await page.evaluate(() => {
        const link = document.createElement("a"); link.dataset.persistentTactical = "";
        link.dataset.serverAddr = "ws://127.0.0.1:1"; link.dataset.characterId = "100";
        document.body.append(link); link.click(); link.remove();
      });
      await page.waitForFunction(() => document.body.hasAttribute("data-tactical-active"));
      await page.keyboard.press("Escape");
      await page.waitForFunction(count => window.commands.filter(command => command.type === "sync-strategic-view").length > count, before);
      assert.equal(await page.evaluate(() => window.originalCanvas === document.querySelector("#game-canvas")), true);
    }
    const generation = await page.evaluate(() => window.strategicGenerationMetrics);
    assert.deepEqual(generation, generationAtReady, "warm navigation schedules no new generation");
    fs.writeFileSync(path.join(output, realRenderer ? "benchmark.json" : "bridge.json"), JSON.stringify({ sceneFixture, samples, steadyFrames, missing, errors, initialLoad, metrics: await page.evaluate(() => window.strategicRendererMetrics), generation }, null, 2));
    const production = path.join(output, "fixtures/inventory.html");
    if (realRenderer && fs.existsSync(production)) {
      await page.setViewportSize({width: 1440, height: 1000});
      const replacementStarted = performance.now();
      const replacementRequests = requests.length;
      await page.evaluate(html => {
        document.body.removeAttribute("data-strategic-scene-ready");
        const documentFixture = new DOMParser().parseFromString(html, "text/html");
        document.querySelector("#strategic-page").replaceWith(documentFixture.querySelector("#strategic-page"));
        document.dispatchEvent(new CustomEvent("strategic-page-mounted"));
      }, fs.readFileSync(production, "utf8"));
      await page.addScriptTag({url: `${origin}/static/chat-dock.js`});
      await ready();
      fs.writeFileSync(path.join(output, "location-replacement.json"), JSON.stringify({
        milliseconds: performance.now() - replacementStarted,
        requests: requests.slice(replacementRequests),
      }, null, 2));
      await page.screenshot({path: path.join(output, "production-desktop.png")});
      await page.setViewportSize({width: 390, height: 844});
      await page.waitForTimeout(250);
      fs.writeFileSync(path.join(output, "production-layout.json"), JSON.stringify(await page.evaluate(() => ({
        scroll: scrollY,
        elements: [".settlement-top-bar", ".settlement-services", "[data-building-id='public-square']", ".bevy-scene-window", "[data-chat-dock]"].map(selector => {
          const el = document.querySelector(selector); return {selector, rect: el?.getBoundingClientRect().toJSON(), style: el ? {position: getComputedStyle(el).position, top: getComputedStyle(el).top, height: getComputedStyle(el).height} : null};
        })
      })), null, 2));
      await page.screenshot({path: path.join(output, "production-narrow.png"), fullPage: true});
    }
    assert.deepEqual(errors, []);
  } finally { clearInterval(telemetry); await browser.close(); server.close(); }
});
