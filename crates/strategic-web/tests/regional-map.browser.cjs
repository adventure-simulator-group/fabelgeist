const assert = require("node:assert/strict");
const fs = require("node:fs");
const http = require("node:http");
const path = require("node:path");
const test = require("node:test");
const { chromium } = require("playwright");

const root = path.resolve(__dirname, "../../..");
const wasm = process.env.REGIONAL_MAP_WASM_DIR;
const density = Number(process.env.REGIONAL_MAP_BROWSER_DENSITY || 1);
assert([1,2].includes(density), "The renderer fixture supports real 1× and 2× display density");
const output = path.resolve(root, process.env.REGIONAL_MAP_REVIEW_DIR || "target/regional-map-browser");
const assetRoots = [path.join(root, "assets"),
  process.env.REGIONAL_MAP_ASSET_DIR || path.join(root, "crates/adventuresim-stdb-module/static/assets")];
// These authored motion sources are currently absent from the repository. This
// fixture has no actors and verifies map readiness independently of equipment.
// It does not substitute another motion; every other missing asset still fails.
const unusedMissingMotions = new Set(["airborne_center", "airborne_travel", "quickstep_back", "quickstep_left"]
  .map(name => `/tactical/assets/animations/biped/unarmed/${name}.glb`));

test("regional terrain reuses one real renderer across camera changes and hiding", {
  skip: !wasm && "Set REGIONAL_MAP_WASM_DIR to the freshly built Wasm bindings",
  timeout: 360_000,
}, async () => {
  fs.mkdirSync(output, { recursive: true });
  fs.writeFileSync(path.join(output, "console.log"), "");
  const missing = [], errors = [];
  const checkpoint = stage => fs.writeFileSync(path.join(output,"stage.json"), JSON.stringify({stage,time:Date.now()}));
  const server = http.createServer((request, response) => {
    const url = new URL(request.url, "http://localhost");
    const relative = decodeURIComponent(url.pathname);
    let roots, suffix;
    if (relative.startsWith("/tactical/wasm/")) {
      roots = [path.resolve(root, wasm)]; suffix = relative.slice(15);
    } else if (relative.startsWith("/tactical/assets/")) {
      roots = assetRoots; suffix = relative.slice(17);
    } else if (relative.startsWith("/static/")) {
      roots = [path.join(root, "crates/strategic-web/static")]; suffix = relative.slice(8);
    } else if (relative === "/input.json") {
      roots = [root]; suffix = "assets/tactical-scenes/sparse-woodland.json";
    }
    if (roots) {
      const file = roots.map(directory => path.resolve(directory, suffix))
        .find((candidate, index) => candidate.startsWith(path.resolve(roots[index]) + path.sep)
          && fs.existsSync(candidate) && fs.statSync(candidate).isFile());
      if (!file) { missing.push(relative); response.writeHead(404); response.end(); return; }
      response.setHeader("Content-Type", {
        ".js": "text/javascript", ".css": "text/css", ".wasm": "application/wasm", ".json": "application/json",
      }[path.extname(file)] || "application/octet-stream");
      fs.createReadStream(file).pipe(response); return;
    }
    response.setHeader("Content-Type", "text/html");
    response.end(`<!doctype html><html><body style="margin:0;background:#201815">
      <div style="width:100vw;height:100vh"><canvas id="game-canvas"></canvas></div>
      <script type="module">
        import init, * as runtime from "/tactical/wasm/adventuresim-tactical-client.js";
        import {prepareGeneratedScene} from "/static/strategic-generation.js";
        try {
          const module = await WebAssembly.compileStreaming(fetch("/tactical/wasm/adventuresim-tactical-client_bg.wasm"));
          await init({module_or_path:module});
          const [graphics, audio, input] = await Promise.all([
            fetch("/tactical/assets/config/tactical-graphics.yaml").then(r=>r.text()),
            fetch("/tactical/assets/config/tactical-audio.yaml").then(r=>r.text()),
            fetch("/input.json").then(r=>r.text())]);
          runtime.wasm_boot(graphics, audio);
          const preparationRuntime={...runtime,generationModule:module,
            generationRevision:"regional-map-browser",generationGraphicsConfig:graphics};
          const [preparation,preview]=await Promise.all([
            prepareGeneratedScene(preparationRuntime,input,{places:[],people:[]}),
            prepareGeneratedScene(preparationRuntime,input,{places:[],people:[]},{owner:"regional-map"}),
          ]);
          if(JSON.parse(preparation).sequence===JSON.parse(preview).sequence)
            throw new Error("Concurrent presentation owners reused a preparation identity");
          const replacement=runtime.wasm_begin_generation(JSON.stringify("scene"));
          runtime.wasm_cancel_generation(replacement);
          runtime.wasm_cancel_generation(preview);
          try {runtime.wasm_generation_jobs(preparation,input);throw new Error("Completed preparation accepted more worker jobs");}
          catch(error){if(error.name!=="generation/stale-preparation")throw error;}
          window.preparationFixture={scene:JSON.parse(preparation),preview:JSON.parse(preview)};
          runtime.wasm_command(JSON.stringify({type:"prepare-strategic-scene", location:"fixture", input_json:input,
            preparation:JSON.parse(preparation)}));
          runtime.wasm_command(JSON.stringify({type:"sync-strategic-view", view:{
            revision:1,location:"fixture",places:[],people:[],active_place:null,selected:null,
            street:null,stage:null,forge:null,portraits:[]}}));
          window.runtime=runtime;
        } catch(error) { console.error(error); window.bootFailure=String(error); }
      </script></body></html>`);
  });
  await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
  const browser = await chromium.launch({ headless: true,
    channel: process.platform === "win32" ? "msedge" : undefined,
    args: ["--enable-unsafe-webgpu", `--force-device-scale-factor=${density}`] });
  let page;
  try {
    page = await browser.newPage({viewport: {width: 1000, height: 700},deviceScaleFactor:density});
    page.setDefaultTimeout(90_000);
    await page.addInitScript({path: path.join(__dirname, "webgpu-probe.js")});
    page.on("pageerror", error => errors.push({text:error.message}));
    page.on("console", message => {
      if (message.type() === "error") errors.push({text:message.text().slice(0,4000),url:message.location().url});
      fs.appendFileSync(path.join(output, "console.log"), message.text().slice(0, 4000) + "\n");
    });
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    checkpoint("booting");
    await page.waitForFunction(() => window.runtime || window.bootFailure, null, {timeout: 180_000});
    assert.equal(await page.evaluate(() => window.bootFailure), undefined);
    assert.equal(await page.evaluate(()=>preparationFixture.scene.owner),"scene");
    assert.equal(await page.evaluate(()=>preparationFixture.preview.owner),"regional-map");
    checkpoint("runtime-prepared");
    const initial = await page.evaluate(() => {
      const source = "a".repeat(64), origin = {latitude: 50_500_000, longitude: 10_500_000};
      const rect = Object.fromEntries(Object.entries({x:80,y:60,width:840,height:560,
        full_width:840,full_height:560,offset_x:0,offset_y:0}).map(([key,value])=>[key,value*devicePixelRatio]));
      window.overlayRevision = 0;
      window.mapCommand = command => runtime.wasm_command(JSON.stringify({type:"regional-map",command}));
      window.mapStatus = () => JSON.parse(runtime.wasm_regional_map_status());
      window.mapOpen = {type:"open",source,origin,span:1200,rect};
      const vertices = Array.from({length:65*65}, (_, index) => {
        const x=index%65, y=Math.floor(index/65);
        if((x<4 && y<4) || (x===32 && y===33)) return null;
        return {elevation:{meters:Math.round(300+60*Math.sin(x/8)*Math.cos(y/9))},environment:{
          canopy_bps:0,wetland_bps:0,cultivation_bps:x>32?8000:0,water_bps:0,hilly_bps:6000,
          crossing_bps:0,surface:"open"}};
      });
      window.mapTerrain = {source,request:{origin,scale:"neighborhood"},vertices};
      window.mapConnections=["land","river","coast","canal","ferry","winter","inferred_walking_link"].map((kind,index)=>({kind,points:[
        {latitude:505_000_000+(index-3)*20_000,longitude:104_850_000},
        {latitude:505_000_000+(index-3)*20_000,longitude:105_150_000},
      ]}));
      window.mapEnvironment=()=>({terrain:mapTerrain,connections:mapConnections});
      window.mapOverlay = {source,markers:[
        {place:"place:v1:settlement:66697874757265",origin,rank:"town",emphasis:"current"},
        {place:"place:v1:case-site:666978747572652d73697465",origin:{latitude:50_502_000,longitude:10_503_000},rank:"case-site",emphasis:"selected"},
        {place:"place:v1:settlement:686f6c65",origin:{latitude:50_492_000,longitude:10_487_000},rank:"village",emphasis:"ordinary"},
        {place:"place:v1:settlement:64697374616e74",origin:{latitude:51_000_000,longitude:11_000_000},rank:"capital",emphasis:"ordinary"},
      ],route:{kind:"computed",points:[
        {latitude:50_496_000,longitude:10_494_000},origin,
        {latitude:50_504_000,longitude:10_506_000},
      ]}};
      mapCommand(mapOpen); mapCommand({type:"install-environment",environment:mapEnvironment()});
      mapCommand({type:"install-overlay",revision:++window.overlayRevision,overlay:mapOverlay});
      return mapOpen;
    });
    await page.waitForFunction(() => mapStatus().ready, null, {timeout: 90_000});
    checkpoint("map-ready");
    assert.equal(await page.evaluate(()=>mapStatus().connection_meshes),7,"All connection classes share the terrain window");
    const acknowledgement = await page.evaluate(async () => {
      const revision = mapStatus().overlay_revision;
      mapCommand({type:"install-overlay",revision,overlay:{...mapOverlay,markers:[],route:null}});
      for(let frame=0;frame<5;frame++)await new Promise(requestAnimationFrame);
      return mapStatus();
    });
    assert.equal(acknowledgement.overlay_revision,1);
    assert.equal(acknowledgement.markers.length,2,"An obsolete overlay cannot remove current pins");
    const projected = await page.evaluate(() => mapStatus().markers);
    assert.equal(projected.length,2,"Covered markers project; source holes and distant pins stay hidden");
    const homeMarker = projected.find(marker => marker.place === "place:v1:settlement:66697874757265");
    assert(Math.abs(homeMarker.x - 500) < 2 && Math.abs(homeMarker.y - 340) < 2,
      "The home pin uses the real clipped camera centre and a covered adjacent triangle");
    await page.screenshot({path:path.join(output,"terrain.png")});
    const capture = await page.evaluate(() => renderProbe.capture("gpu", 3));
    checkpoint("terrain-captured");
    assert.deepEqual(capture.failures, []);
    assert(capture.frames.some(frame => frame.passes.some(pass => pass.draws?.some(draw =>
      draw.pipeline?.label === "pbr_opaque_mesh_pipeline" && draw.triangles > 4000))),
      "The real WebGPU renderer must draw admitted terrain");
    assert(capture.frames.some(frame => frame.passes.some(pass => pass.draws?.some(draw =>
      draw.pipeline?.label === "pbr_opaque_mesh_pipeline" && draw.triangles > 0 && draw.triangles < 1000))),
      "The selected route must submit its own covered geometry");
    const changed = await page.evaluate(async () => {
      mapCommand({type:"rotate",angle:0.7}); mapCommand({type:"pan",delta:[80*devicePixelRatio,40*devicePixelRatio]});
      mapCommand({type:"zoom",ratio:0.5});
      await new Promise(requestAnimationFrame); await new Promise(requestAnimationFrame);
      return mapStatus();
    });
    assert.equal(changed.span, initial.span * 0.5);
    assert(Math.abs(changed.yaw - 0.7) < 0.00001);
    assert.notDeepEqual(changed.origin, initial.origin);
    assert.notDeepEqual(changed.markers,projected,"Marker projection follows the real camera transform");
    await page.screenshot({path:path.join(output,"rotated.png")});
    checkpoint("rotated");
    await page.evaluate(async () => {
      mapCommand({type:"hide"}); await new Promise(requestAnimationFrame);
      await new Promise(requestAnimationFrame);
    });
    assert.equal(await page.evaluate(() => mapStatus().visible), false);
    assert.deepEqual(await page.evaluate(() => mapStatus().markers), []);
    const warm = await page.evaluate(async () => {
      const started = performance.now(); mapCommand(mapOpen);
      while (!mapStatus().ready) await new Promise(requestAnimationFrame);
      return {milliseconds:performance.now()-started,status:mapStatus(),canvases:document.querySelectorAll("canvas").length};
    });
    assert.equal(warm.canvases, 1);
    checkpoint("warm-reopened");
    assert.deepEqual(warm.status.origin, changed.origin, "Reopening retains the geographic pose");
    assert.equal(warm.status.span, changed.span);
    assert.deepEqual(warm.status.presented, {origin:initial.origin,scale:"neighborhood"});
    const rejected = await page.evaluate(() => {
      const invalid = [{...mapOpen,span:0}, {...mapOpen,rect:{...mapOpen.rect,full_height:0}},
        {type:"zoom",ratio:100}, {type:"install-environment",environment:{terrain:{...mapTerrain,vertices:[]},connections:[]}},
        {type:"install-overlay",revision:0,overlay:mapOverlay},
        {type:"install-overlay",revision:2**53,overlay:mapOverlay},
        {type:"install-overlay",revision:++window.overlayRevision,overlay:{...mapOverlay,markers:[mapOverlay.markers[0],mapOverlay.markers[0]]}},
        {type:"install-overlay",revision:++window.overlayRevision,overlay:{...mapOverlay,markers:[{...mapOverlay.markers[0],rank:"case-site"}]}},
        {type:"install-overlay",revision:++window.overlayRevision,overlay:{...mapOverlay,route:{kind:"computed",points:[]}}},
        {type:"install-overlay",revision:++window.overlayRevision,overlay:{...mapOverlay,markers:mapOverlay.markers.map((marker,index)=>
          index===3?{...marker,emphasis:"current"}:marker)}},
        {type:"install-overlay",revision:++window.overlayRevision,overlay:{...mapOverlay,markers:mapOverlay.markers.map((marker,index)=>
          index===3?{...marker,emphasis:"selected"}:marker)}}];
      return invalid.map(command => {try { mapCommand(command); return false; } catch {return true;}});
    });
    assert.deepEqual(rejected, Array(11).fill(true));
    await page.evaluate(() => mapCommand({type:"install-overlay",revision:++window.overlayRevision,overlay:{...mapOverlay,route:{
      kind:"computed",points:Array.from({length:600},(_,index)=>index%2
        ? {latitude:50_508_000,longitude:10_512_000}
        : {latitude:50_492_000,longitude:10_488_000})}}}));
    await page.waitForFunction(() => mapStatus().error === "route-capacity", null, {timeout:10_000});
    await page.evaluate(async () => { for(let frame=0;frame<4;frame++) await new Promise(requestAnimationFrame); });
    const capacity = await page.evaluate(() => mapStatus());
    assert.equal(capacity.error,"route-capacity"); assert.equal(capacity.ready,false);
    assert.equal(capacity.covered,true,"A route capacity failure leaves terrain available");
    await page.evaluate(() => mapCommand({type:"install-overlay",revision:++window.overlayRevision,overlay:mapOverlay}));
    await page.waitForFunction(() => mapStatus().ready && !mapStatus().error, null, {timeout:10_000});
    await page.evaluate(()=>mapCommand({type:"install-environment",environment:{
      terrain:{...mapTerrain,request:{...mapTerrain.request,origin:{...mapTerrain.request.origin,latitude:mapTerrain.request.origin.latitude+1}}},
      connections:Array.from({length:2048},()=>({...mapConnections[0],points:mapConnections[3].points})),
    }}));
    await page.waitForFunction(()=>mapStatus().error==="connection-capacity",null,{timeout:10_000});
    const connectionCapacity=await page.evaluate(()=>mapStatus());
    assert.equal(connectionCapacity.ready,false);
    assert.equal(connectionCapacity.covered,true,"Connection capacity keeps the terrain available");
    assert.equal(connectionCapacity.connection_meshes,0,"Capacity does not publish a partial network");
    await page.evaluate(()=>mapCommand({type:"install-environment",environment:mapEnvironment()}));
    await page.waitForFunction(()=>mapStatus().ready&&!mapStatus().error,null,{timeout:10_000});
    const cameraControls = await page.evaluate(async () => {
      const settle = async () => { await new Promise(requestAnimationFrame); await new Promise(requestAnimationFrame); };
      const endpoints = [mapOverlay.route.points[0],mapOverlay.route.points[2]].map((origin,index)=>({
        place:index ? "place:v1:settlement:656e64" : "place:v1:settlement:7374617274",
        origin,rank:"town",emphasis:"connected"}));
      mapCommand({type:"install-overlay",revision:++window.overlayRevision,overlay:{...mapOverlay,markers:[...mapOverlay.markers,...endpoints]}});
      mapCommand({type:"frame-route"}); await settle();
      const framed = mapStatus();
      mapCommand({type:"reset"});
      mapCommand({type:"resize",rect:{...mapOpen.rect,width:560*devicePixelRatio,full_width:560*devicePixelRatio}});
      mapCommand({type:"zoom",ratio:800/1200}); await settle();
      const square = mapStatus();
      mapCommand({type:"rotate",angle:Math.PI/4}); await settle();
      const rotatedSquare = mapStatus();
      mapCommand({type:"resize",rect:mapOpen.rect}); mapCommand({type:"reset"});
      mapCommand({type:"zoom",ratio:0.05}); mapCommand({type:"zoom",ratio:0.5}); await settle();
      const before = mapStatus();
      for(let index=0;index<20;index++) mapCommand({type:"pan",delta:[0.01*devicePixelRatio,0]});
      await settle(); const after = mapStatus();
      return {framed,square,rotatedSquare,before,after};
    });
    const {framed,square,rotatedSquare,before,after} = cameraControls;
    for(const place of ["place:v1:settlement:7374617274","place:v1:settlement:656e64"]) {
      const marker = framed.markers.find(marker=>marker.place===place);
      assert(marker,"Framing a rotated route keeps both covered endpoints in view");
      assert(marker.x>90 && marker.x<910 && marker.y>70 && marker.y<610);
    }
    assert.equal(square.requested.scale,"neighborhood");
    assert.equal(rotatedSquare.requested.scale,"district","Rotated ground footprint selects sufficient terrain");
    assert.equal(after.span,40);
    assert.deepEqual(after.origin,before.origin,"Tiny drags stay below the rounded geographic wire resolution");
    const home = state=>state.markers.find(marker=>marker.place==="place:v1:settlement:66697874757265");
    assert(Math.abs(home(after).x-home(before).x-0.2)<0.02,
      "Repeated sub-centimetre drags accumulate in the continuous camera pose");
    checkpoint("camera-controls");
    await page.evaluate(() => mapCommand({type:"install-environment",environment:{terrain:{
      ...mapTerrain,request:{...mapTerrain.request,origin:{latitude:50_501_000,longitude:10_500_000}},
      vertices:Array(65*65).fill(null)},connections:mapConnections}}));
    await page.waitForFunction(() => mapStatus().ready && !mapStatus().covered, null, {timeout: 10_000});
    assert.deepEqual(await page.evaluate(() => mapStatus().markers), []);
    await page.screenshot({path:path.join(output,"uncovered.png")});
    const interfaceResult = await require("./regional-map-interface-check.cjs")(page,output);
    const realSource=await require("./regional-map-real-source-check.cjs")(page,output,process.env.REGIONAL_MAP_ENVIRONMENT_DIR);
    fs.writeFileSync(path.join(output,"result.json"), JSON.stringify({warm,changed,capacity,connectionCapacity,cameraControls,interfaceResult,realSource,capture,missing,errors}, null, 2));
    assert.deepEqual(missing.filter(url => !unusedMissingMotions.has(url)), []);
    assert.deepEqual(errors.filter(error => {
      try { return !unusedMissingMotions.has(new URL(error.url).pathname); } catch { return true; }
    }), []);
    checkpoint("complete");
  } catch (error) {
    const state = page && await page.evaluate(() => ({
      scene:window.runtime && JSON.parse(runtime.wasm_strategic_status()),
      map:window.runtime && JSON.parse(runtime.wasm_regional_map_status()),
      interface:window.mapBridgeInstalled && {rect:window.mapRect(),metrics:window.strategicRendererMetrics,
        status:document.querySelector("[data-map-status]")?.textContent},
      bootFailure:window.bootFailure,
    })).catch(() => ({}));
    fs.writeFileSync(path.join(output,"failure.json"), JSON.stringify({error:String(error),state,missing,errors},null,2));
    throw error;
  } finally { await browser.close(); server.close(); }
});
