const assert=require("node:assert/strict");
const path=require("node:path");
const fs=require("node:fs");

// Extend the real Bevy fixture with the production HTML bridge and compositor.
// Actor scene loading deliberately stays pending: map readiness is independent.
module.exports=async function checkInterface(page,output) {
  const requests=[];
  await page.route("**/api/scene-assets**",()=>{});
  await page.route("**/api/map/environment/**",async route=>{
    const url=new URL(route.request().url());
    const terrain=await page.evaluate(query=>({...mapTerrain,vertices:mapTerrain.vertices.map(vertex=>vertex||{
      elevation:{meters:300},environment:{canopy_bps:0,wetland_bps:0,cultivation_bps:0,water_bps:0,hilly_bps:6000,
        crossing_bps:0,surface:"open"}}),request:{
      scale:query.scale,origin:{latitude:Number(query.latitude),longitude:Number(query.longitude)}}}),
      Object.fromEntries(url.searchParams));
    requests.push(url.pathname+url.search);
    await route.fulfill({contentType:"application/json",body:JSON.stringify({terrain,connections:await page.evaluate(()=>mapConnections)})});
  });
  await page.evaluate(async()=>{
    mapOpen.origin={...mapOpen.origin,latitude:mapOpen.origin.latitude+100};
    const canvas=document.querySelector("canvas");
    canvas.parentElement.id="strategic-render-surface";
    for(const name of ["base","reset","layout","components","strategic","utilities","strategic-scene"]) {
      const style=document.createElement("link");style.rel="stylesheet";style.href="/static/css/"+name+".css";
      document.head.append(style);
    }
    // Fixture layout positions the real component in a scrollable HTML panel.
    const style=document.createElement("style");
    style.textContent="#strategic-render-surface{position:fixed;inset:0;width:100%;height:100%;pointer-events:none}"+
      "#strategic-page{position:relative;margin:50px 70px;color:#f4ead5}"+
      ".settlement-map-main{height:550px;overflow:auto}"+
      ".strategic-map{height:520px;min-height:0}"+
      "@media(max-width:600px){#strategic-page{margin:40px 10px}.settlement-map-main{height:400px}.strategic-map{height:520px}}";
    document.head.append(style);
    window.mapMarkup=()=>'<main class="settlement-map-main"><section data-regional-map class="strategic-map" tabindex="0" role="region" aria-label="Map around Fixture">'+
      '<script data-regional-map-input type="application/json">'+JSON.stringify({
        origin:mapOpen.origin,span:mapOpen.span,selected:mapOverlay.markers[1].place,overlay:mapOverlay})+'</script>'+
      '<div class="strategic-map-window" data-map-window aria-hidden="true"></div>'+
      '<div class="strategic-map-controls" data-map-foreground>'+
      '<button class="strategic-map-control" data-map-action="zoom-in" aria-label="Zoom in">+</button>'+
      '<button class="strategic-map-control" data-map-action="zoom-out" aria-label="Zoom out">−</button>'+
      '<button class="strategic-map-control" data-map-action="rotate-right" aria-label="Rotate right">↻</button>'+
      '<button class="strategic-map-control" data-map-action="reset" aria-label="Reset view">⌂</button>'+
      '<button class="strategic-map-control" data-map-action="frame-route" aria-label="Frame route">↔</button>'+
      '<button class="strategic-map-control" data-map-action="retry" aria-label="Retry map" hidden>↻</button></div>'+
      '<div class="strategic-map-markers">'+mapOverlay.markers.slice(0,2).map((marker,index)=>
      '<a class="map-place-link '+(index?"selected":"current")+'" data-map-foreground hidden data-map-place="'+marker.place+
        '" href="#destination-'+index+'"><span class="map-place-symbol" aria-hidden="true">⌂</span><span>'+(index?"Known site":"Fixture")+'</span><small class="map-place-state">'+(index?"Selected":"Origin")+'</small></a>').join("")+'</div>'+
      '<p class="strategic-map-status" role="status" data-map-status data-map-foreground>Loading map…</p>'+
      '<details class="strategic-map-key" data-map-foreground><summary>Map key</summary><ul><li><span class="map-key-road"></span>Road</li><li><span class="map-key-water"></span>Shipping route or ferry</li><li><span class="map-key-winter"></span>Winter route</li><li><span class="map-key-inferred"></span>Inferred walking link</li><li><span class="map-key-selected"></span>Computed route</li><li><span class="map-key-estimated"></span>Estimated route</li></ul></details>'+
      '<a class="strategic-map-license-link" data-map-foreground href="#licence">Map data licence</a></section></main>';
    const host=document.createElement("div");host.id="strategic-page";host.innerHTML=mapMarkup();document.body.append(host);
    const {installStrategicScene,canvasRect}=await import("/static/strategic-scene.js");
    window.mapRect=()=>canvasRect(document.querySelector("[data-map-window]"));
    installStrategicScene(command=>runtime.wasm_command(JSON.stringify(command)),Promise.resolve(runtime));
    window.mapBridgeInstalled=true;
  });
  await page.waitForFunction(()=>document.body.hasAttribute("data-regional-map-ready"),null,{timeout:30_000});
  const desktop=await page.evaluate(()=>({
    status:mapStatus(),clip:document.querySelector("#strategic-render-surface").style.clipPath,
    links:[...document.querySelectorAll("[data-map-place]")].filter(link=>!link.hidden).length,
    canvases:document.querySelectorAll("canvas").length}));
  assert.equal(desktop.canvases,1);assert.equal(desktop.links,2);assert.notEqual(desktop.clip,"inset(100%)");
  await page.screenshot({path:path.join(output,"interface-desktop.png")});
  await page.locator("[data-regional-map]").focus();
  await page.keyboard.press("e");
  await page.waitForFunction(()=>mapStatus().yaw>0.1);
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("Home");
  await page.waitForFunction(()=>mapStatus().yaw===0);
  const map=page.locator("[data-regional-map]");
  const bounds=await map.boundingBox();
  const drag={x:bounds.x+bounds.width*0.3,y:bounds.y+bounds.height*0.7};
  await page.mouse.move(drag.x,drag.y);
  await page.mouse.down({button:"right"});
  await page.mouse.move(drag.x+50,drag.y);
  await page.mouse.up({button:"right"});
  await page.waitForFunction(()=>mapStatus().yaw>0.3);
  await page.mouse.wheel(0,120);
  await page.waitForFunction(()=>mapStatus().span>1200);
  await page.getByRole("button",{name:"Reset view"}).click();
  await page.waitForFunction(()=>mapStatus().yaw===0);
  await page.getByRole("button",{name:"Zoom in",exact:true}).click();
  await page.waitForFunction(()=>mapStatus().span===960);
  await page.getByRole("button",{name:"Reset view"}).click();
  await page.waitForFunction(()=>mapStatus().span===1200);
  const pinch=await page.evaluate(()=>{
    const host=document.querySelector("[data-regional-map]");
    const capture=host.setPointerCapture;
    // Synthetic touch intent still exercises real DOM event listeners; native
    // pointer capture requires a browser-owned active pointer, so omit that port.
    host.setPointerCapture=()=>{};
    const emit=(type,id,x,y)=>host.dispatchEvent(new PointerEvent(type,{
      bubbles:true,pointerId:id,pointerType:"touch",button:0,clientX:x,clientY:y}));
    emit("pointerdown",101,200,200);emit("pointerdown",102,400,200);
    emit("pointermove",102,450,250);
    emit("pointerup",101,200,200);emit("pointerup",102,450,250);
    host.setPointerCapture=capture;
    return mapStatus().span;
  });
  await page.waitForFunction(()=>mapStatus().yaw>0.1 && mapStatus().span<1200);
  await page.getByRole("button",{name:"Reset view"}).click();
  await page.waitForFunction(()=>mapStatus().yaw===0 && mapStatus().span===1200);
  const beforeReopen=requests.length;
  const warmStarted=await page.evaluate(async()=>{
    document.dispatchEvent(new Event("strategic-page-unmounting"));
    for(let frame=0;frame<6;frame++)await new Promise(requestAnimationFrame);
    if(mapStatus().visible)throw new Error("The unmounted page reopened while navigation was pending");
    const host=document.querySelector("#strategic-page");
    host.innerHTML="<main>Journal</main>";
    document.dispatchEvent(new Event("strategic-page-mounted"));
    return performance.now();
  });
  await page.waitForFunction(()=>!mapStatus().visible);
  await page.evaluate(()=>{
    document.querySelector("#strategic-page").innerHTML=mapMarkup();
    document.dispatchEvent(new Event("strategic-page-mounted"));
  });
  await page.waitForFunction(()=>document.body.hasAttribute("data-regional-map-ready"));
  const warm=await page.evaluate(start=>({milliseconds:strategicRendererMetrics.maps.at(-1).milliseconds,
    roundTripMilliseconds:performance.now()-start,
    sample:strategicRendererMetrics.maps.at(-1),canvases:document.querySelectorAll("canvas").length}),warmStarted);
  assert.equal(warm.canvases,1);assert.equal(requests.length,beforeReopen,"Warm reopening cannot fetch terrain again");
  await page.setViewportSize({width:390,height:700});
  await page.waitForFunction(()=>document.body.hasAttribute("data-regional-map-ready")
    && mapRect().width/devicePixelRatio<390 && mapStatus().ready
    && Object.entries(mapRect()).every(([key,value])=>Math.abs(mapStatus().rect[key]-value)<0.01));
  await page.screenshot({path:path.join(output,"interface-narrow.png")});
  const narrow=await page.evaluate(()=>{
    const host=document.querySelector("[data-regional-map]"),controls=host.querySelector("[data-map-foreground]");
    return {host:host.getBoundingClientRect().toJSON(),controls:controls.getBoundingClientRect().toJSON(),
      window:mapRect(),markers:mapStatus().markers,clip:document.querySelector("#strategic-render-surface").style.clipPath,
      labels:[...host.querySelectorAll("[data-map-place]:not([hidden])")].map(link=>link.getBoundingClientRect().toJSON())};
  });
  assert(narrow.controls.right<=390 && narrow.controls.left>=0);
  for(const label of narrow.labels)assert(label.right<=narrow.host.right && label.left>=narrow.host.left,
    "Narrow viewport names remain inside the map after choosing their label side");
  await page.locator(".settlement-map-main").evaluate(element=>element.scrollTop=120);
  await page.waitForFunction(()=>mapRect().offset_y>20 && document.body.hasAttribute("data-regional-map-ready")
    && mapStatus().ready && Object.entries(mapRect()).every(([key,value])=>Math.abs(mapStatus().rect[key]-value)<0.01));
  await page.screenshot({path:path.join(output,"interface-scrolled.png")});
  const scrolled=await page.evaluate(()=>mapRect());
  await page.locator(".settlement-map-main").evaluate(element=>element.scrollTop=0);
  await page.waitForFunction(()=>mapRect().offset_y<5
    && document.body.hasAttribute("data-regional-map-ready") && mapStatus().ready
    && Object.entries(mapRect()).every(([key,value])=>Math.abs(mapStatus().rect[key]-value)<0.01));
  // A map link remains ordinary navigation; keyboard activation never pans.
  const visibleLink=page.locator("[data-map-place]:not([hidden])").first();
  await visibleLink.focus();await page.keyboard.press("Enter");
  await page.waitForURL(/#destination-/);
  assert.match(page.url(),/#destination-/);
  assert.equal(await page.locator("canvas").count(),1);
  // Launch the browser at the requested physical density. CDP density overrides
  // change devicePixelRatio without changing ResizeObserver's physical pixel box.
  const densityPort=await page.evaluate(async()=>{
    const canvas=document.querySelector("canvas");
    const pixelBox=await new Promise(resolve=>{
      const observer=new ResizeObserver(entries=>{
        const box=entries[0].devicePixelContentBoxSize[0];
        observer.disconnect();resolve({width:box.inlineSize,height:box.blockSize});
      });observer.observe(canvas,{box:"device-pixel-content-box"});
    });
    return {ratio:devicePixelRatio,canvas:{width:canvas.width,height:canvas.height,
      rectangle:canvas.getBoundingClientRect().toJSON()},pixelBox,map:mapStatus()};
  });
  fs.writeFileSync(path.join(output,"density-port.json"),JSON.stringify(densityPort,null,2));
  assert(Math.abs(densityPort.pixelBox.width-densityPort.canvas.rectangle.width*densityPort.ratio)<1,
    "The browser must expose real physical pixels to the native resize observer");
  assert.equal(densityPort.canvas.width,densityPort.pixelBox.width);
  await page.waitForFunction(()=>document.body.hasAttribute("data-regional-map-ready"),null,{timeout:10000});
  await page.waitForFunction(()=>mapStatus().markers.length>0);
  await page.waitForFunction(()=>mapStatus().markers.every(marker=>{
    const link=[...document.querySelectorAll("[data-map-place]:not([hidden])")].find(link=>link.dataset.mapPlace===marker.place);
    if(!link)return false;
    const symbol=link.querySelector(".map-place-symbol").getBoundingClientRect();
    return Math.abs(symbol.y+symbol.height/2-marker.y)<2;
  }));
  const dense=await page.evaluate(()=>({ratio:devicePixelRatio,window:mapRect(),markers:mapStatus().markers,
    links:[...document.querySelectorAll("[data-map-place]:not([hidden])")].map(link=>({
      place:link.dataset.mapPlace,rectangle:link.getBoundingClientRect().toJSON(),
      symbol:link.querySelector(".map-place-symbol").getBoundingClientRect().toJSON()}))}));
  assert(Math.abs(dense.window.full_width-narrow.window.full_width)<1,
    "Canvas commands retain the actual physical viewport after scrolling");
  for(const link of dense.links) {
    const marker=dense.markers.find(marker=>marker.place===link.place);
    assert(Math.abs(link.symbol.x+link.symbol.width/2-marker.x)<4,"Native logical projection still anchors HTML links");
    assert(Math.abs(link.symbol.y+link.symbol.height/2-marker.y)<2,"Vertical HTML projection follows the settled camera");
    const previous=narrow.markers.find(point=>point.place===link.place);
    assert(Math.abs(previous.y-marker.y)<1,"Returning from scroll preserves logical geographic projection");
  }
  await page.screenshot({path:path.join(output,"interface-density.png")});
  return {desktop,warm,narrow,scrolled,dense,requests,pinch};
};
