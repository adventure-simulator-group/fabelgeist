const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const { chromium } = require('playwright');

const checkout = process.env.STRATEGIC_INIT_CHECKOUT || path.resolve(__dirname, "../../..");
const output = process.env.STRATEGIC_INIT_OUTPUT;
const cityFile = process.env.STRATEGIC_INIT_CITY;
const environmentFile = process.env.STRATEGIC_INIT_ENVIRONMENT;
const city = fs.readFileSync(cityFile, 'utf8');
const metadata = JSON.parse(city);
const environments = JSON.parse(fs.readFileSync(environmentFile, 'utf8'));
const places = ['public-square', 'residences', 'keep', 'merchants', 'weapons', 'armor', 'clothing', 'herbalist', 'books', 'inn', 'religion'];
const actor = fs.readFileSync(process.env.STRATEGIC_INIT_ACTOR, 'utf8');
const secondCity = fs.readFileSync(process.env.STRATEGIC_INIT_SECOND_CITY, 'utf8');
const secondMetadata = JSON.parse(secondCity);
const secondActor = fs.readFileSync(process.env.STRATEGIC_INIT_SECOND_ACTOR, 'utf8');
assert.equal(secondMetadata.source, metadata.source, 'Both captures must use the terrain source');
assert.notEqual(secondMetadata.place, metadata.place, 'The second capture must name another settlement');
const identities = ['a', 'b', 'a'];
function cityFor(id) {
  return id === 'a' ? city : secondCity;
}
function markup(id) {
  const place = id === 'a' ? metadata.place : secondMetadata.place;
  const input = { origin: environments[0].terrain.request.origin, span: 1200, selected: null,
    overlay: {source: metadata.source, markers: [{place,origin:environments[0].terrain.request.origin,rank:'town',emphasis:'current'}],route:null} };
  return `<!doctype html><html><head><title>Initialization benchmark</title>
  ${['base','reset','layout','components','strategic','utilities','strategic-scene'].map(name=>`<link rel="stylesheet" href="/static/css/${name}.css">`).join('')}
  <style>#strategic-page{position:relative;color:#f4ead5}.settlement-map-main{height:650px}.strategic-map{height:600px;min-height:0}</style></head><body>
  <div id="strategic-render-surface"><canvas id="game-canvas"></canvas></div>
  <!-- strategic-page-start --><div id="strategic-page" data-strategic-workspace data-script-profile="strategic" data-path="/locations/settlement/${id}" data-page-title="Benchmark">
  <header class="top-bar settlement-top-bar"><nav class="settlement-services" data-settlement-id="${id}"><a href="/locations/settlement/${id}" class="active" data-building-id="map">Map</a>${places.map(p=>`<a href="/locations/settlement/${id}/places/${p}" data-building-id="${p}" data-service-id="${p}">${p}</a>`).join('')}</nav></header>
  <main class="settlement-map-main"><section data-regional-map class="strategic-map" tabindex="0" role="region" aria-label="Settlement map">
  <script data-regional-map-input type="application/json">${JSON.stringify(input)}</script>
  <div class="strategic-map-window" data-map-window aria-hidden="true"></div>
  <div class="strategic-map-markers"><a data-map-place="${place}" data-map-foreground hidden href="#place">Settlement</a></div>
  <p data-map-status data-map-foreground role="status">Loading map…</p><button data-map-action="retry" hidden>Retry</button></section></main>
  </div><!-- strategic-page-end --><script src="/static/location-urls.js"></script><script src="/static/strategic-navigation.js"></script>
  <script type="module" src="/static/strategic-renderer.js"></script></body></html>`;
}

async function run() {
  fs.mkdirSync(output, { recursive:true });
  const errors = [], missing = [], requests = [], results = [];
  let rejectFatal;
  const fatal = new Promise((_, reject) => { rejectFatal = reject; });
  fatal.catch(() => {});
  const server = http.createServer((request,response)=>{
    const url = new URL(request.url,'http://localhost');
    requests.push({path:url.pathname,at:Date.now()});
    if (url.pathname.startsWith('/api/map/environment/')) {
      const product = environments.find(product=>product.terrain.request.scale===url.searchParams.get('scale'));
      if(!product){response.writeHead(404);response.end();return;}
      response.setHeader('Content-Type','application/json');response.end(JSON.stringify(product));return;
    }
    if (url.pathname.startsWith('/api/map/city/')) {
      response.setHeader('Content-Type','application/json');response.end(cityFor(url.searchParams.get('place')===metadata.place?'a':'b'));return;
    }
    if (url.pathname==='/api/scene-assets') {
      const id=url.searchParams.get('settlement');
      response.setHeader('Content-Type','application/json');
      response.end(id==='b'?secondActor:actor);return;
    }
    if(url.pathname.startsWith('/api/locations/')){response.setHeader('Content-Type','application/json');response.end('[]');return;}
    let file;
    if(url.pathname.startsWith('/static/')) file=path.join(checkout,'crates/strategic-web/static',url.pathname.slice(8));
    if(url.pathname.startsWith('/tactical/wasm/'))file=path.join(process.env.STRATEGIC_INIT_WASM,path.basename(url.pathname));
    if(url.pathname.startsWith('/tactical/assets/'))file=path.join(process.env.STRATEGIC_INIT_ASSETS,url.pathname.slice(17));
    if(file){
      if(!fs.existsSync(file)){missing.push(url.pathname);response.writeHead(404);response.end();return;}
      response.setHeader('Content-Type', {'.js':'text/javascript','.css':'text/css','.wasm':'application/wasm','.json':'application/json'}[path.extname(file)]||'application/octet-stream');
      response.setHeader('Cache-Control','public, max-age=3600');fs.createReadStream(file).pipe(response);return;
    }
    response.setHeader('Content-Type','text/html');response.setHeader('X-Strategic-Response','root');
    response.setHeader('X-Strategic-Script-Profile','strategic');response.setHeader('X-Strategic-Canonical-Url',url.pathname);
    response.end(markup(url.pathname.split('/')[3]||'a'));
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  const browser = await chromium.launch({headless:true,channel:process.platform==='win32'?'msedge':undefined,args:['--enable-unsafe-webgpu']});
  const page = await browser.newPage({viewport:{width:1000,height:700}});
  await page.addInitScript(()=>{
    window.initTrace=[];let previous;
    function sample(){const metrics=window.strategicRendererMetrics;
      if(metrics){const state=metrics.mapState;
        if(metrics.cityRequest?.phase==='failed')window.initFailure=metrics.cityRequest;
        const key=JSON.stringify([state?.ready,state?.city_installation,state?.city_visible,window.strategicGenerationMetrics]);
        if(key!==previous)window.initTrace.push({at:performance.now(),map:state,generation:window.strategicGenerationMetrics});previous=key;}
      requestAnimationFrame(sample);}
    requestAnimationFrame(sample);
  });
  page.on('pageerror',error=>{errors.push(error.message);rejectFatal(error);fs.writeFileSync(path.join(output,'errors.json'),JSON.stringify(errors,null,2));});
  page.on('console',message=>{
    fs.appendFileSync(path.join(output,'console.log'),message.text().slice(0,4000)+'\n');
    if(message.type()==='error')errors.push(message.text());
    if (/panicked|unreachable|regional map unavailable|strategic scene failed|persistent Bevy renderer unavailable/i.test(message.text())) rejectFatal(new Error(message.text()));
  });
  const telemetry=setInterval(async()=>{
    const state=await page.evaluate(()=>({map:window.strategicRendererMetrics?.mapState,generation:window.strategicGenerationMetrics,scene:window.strategicRendererMetrics?.state})).catch(e=>({error:e.message}));
    fs.writeFileSync(path.join(output,'live-state.json'),JSON.stringify(state,null,2));
  },3000);
  try {
    let originalCanvas;
    for(let index=0;index<identities.length;index++){
      const id=identities[index],destination='/locations/settlement/'+id;
      const started=performance.now();
      if(index===0){await page.goto(`http://127.0.0.1:${server.address().port}${destination}`);originalCanvas=await page.locator('canvas').elementHandle();}
      else {await page.evaluate(href=>{const a=document.createElement('a');a.href=href;document.body.append(a);a.click();a.remove();},destination);}
      await Promise.race([fatal, page.waitForFunction(expected=>window.initFailure||(document.querySelector('#strategic-page')?.dataset.path===expected&&document.body.hasAttribute('data-regional-map-ready')),destination,{timeout:300000})]);
      assert.equal(await page.evaluate(()=>window.initFailure),undefined,'City request must remain valid');
      const terrainMilliseconds=performance.now()-started;
      await Promise.race([fatal, page.waitForFunction(expected=>window.initFailure||(window.strategicRendererMetrics?.mapState?.city_requested===expected&&window.strategicRendererMetrics?.mapState?.city_installation?.phase==='ready'&&window.strategicRendererMetrics?.mapState?.city_visible),id==='a'?metadata.place:secondMetadata.place,{timeout:300000})]);
      assert.equal(await page.evaluate(()=>window.initFailure),undefined,'City request must finish successfully');
      results.push({id,kind:index===0?'cold':index===1?'warm-new':'warm-return',milliseconds:performance.now()-started,terrainMilliseconds,
        ...await page.evaluate(()=>({generation:window.strategicGenerationMetrics,metrics:window.strategicRendererMetrics,trace:window.initTrace}))});
      fs.writeFileSync(path.join(output,'results.json'),JSON.stringify({results,errors,missing,requests},null,2));
      assert.equal(await page.evaluate(canvas=>canvas===document.querySelector('canvas'),originalCanvas),true);
      await page.screenshot({path:path.join(output,`${index}-${id}.png`)});
      await page.evaluate(()=>Promise.all(Object.values(window.strategicGenerationCacheSettled||{})));
      console.log(JSON.stringify({kind:results.at(-1).kind,milliseconds:results.at(-1).milliseconds,terrainMilliseconds}));
    }
  } finally {clearInterval(telemetry);fs.writeFileSync(path.join(output,'requests.json'),JSON.stringify({errors,missing,requests},null,2));await browser.close();server.close();}
}
run().catch(error=>{console.error(error);process.exitCode=1;});
