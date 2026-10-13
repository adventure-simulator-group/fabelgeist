const assert=require("node:assert/strict");
const fs=require("node:fs");
const path=require("node:path");
const {PNG}=require("playwright-core/lib/utilsBundle");

// Native capture supplies both the complete city and its matching terrain.
// Only fixture metadata is inspected here; the city response preserves the
// original document text, including all physical integer seeds.
module.exports=async function checkCity(page,output,cityFile,environmentFile) {
  if(!environmentFile)return null;
  const checkpoint=stage=>fs.writeFileSync(path.join(output,"city-stage.json"),
    JSON.stringify({stage,at:new Date().toISOString()}));
  checkpoint("opening");
  const document=fs.readFileSync(cityFile,"utf8"),metadata=JSON.parse(document);
  const products=JSON.parse(fs.readFileSync(environmentFile,"utf8"));
  const environments=new Map(products.map(product=>[product.terrain.request.scale,product]));
  const environment=environments.get("district"),cityRequests=[],terrainRequests=[];
  assert.equal(environment.terrain.source,metadata.source);
  await page.route("**/api/map/city/"+metadata.source+"?**",async route=>{
    const query=new URL(route.request().url()).searchParams;
    assert.equal(query.get("place"),metadata.place);
    cityRequests.push(route.request().url());
    await route.fulfill({contentType:"application/json",body:document});
  });
  await page.route("**/api/map/environment/"+metadata.source+"?**",async route=>{
    const query=new URL(route.request().url()).searchParams;
    const product=environments.get(query.get("scale"));
    assert(product,"The native fixture owns the requested scale");
    assert.equal(Number(query.get("latitude")),product.terrain.request.origin.latitude);
    assert.equal(Number(query.get("longitude")),product.terrain.request.origin.longitude);
    terrainRequests.push(route.request().url());
    await route.fulfill({contentType:"application/json",body:JSON.stringify(product)});
  });
  await page.setViewportSize({width:1000,height:700});
  await page.evaluate(({environment,place})=>{
    document.dispatchEvent(new Event("strategic-page-unmounting"));
    mapOpen={...mapOpen,source:environment.terrain.source,origin:environment.terrain.request.origin,span:1200};
    const origin=mapOpen.origin;
    mapOverlay={source:mapOpen.source,markers:[
      {place,origin,rank:"town",emphasis:"current"},
      {place:"place:v1:case-site:6f627365727665642d73697465",
        origin:{latitude:origin.latitude+100,longitude:origin.longitude+100},rank:"case-site",emphasis:"selected"},
    ],route:null};
    document.querySelector("#strategic-page").innerHTML=mapMarkup();
    document.dispatchEvent(new Event("strategic-page-mounted"));
  },{environment,place:metadata.place});
  await page.waitForFunction(source=>mapStatus().source===source
    && mapStatus().city_installation?.phase==="ready" && mapStatus().city_visible
    && document.body.hasAttribute("data-regional-map-ready"),metadata.source,{timeout:120_000});
  const cold=await page.evaluate(()=>mapStatus());
  checkpoint("installed");
  assert.equal(cold.city_requested,metadata.place);
  assert.equal(cold.markers.length,2,"Both admitted pins have complete city ground support");
  // Browser screenshots capture the presented WebGPU texture. Copying its
  // canvas into a 2D context after presentation reads a cleared drawing buffer.
  // Hide only foreground labels during capture so they cannot mask a hole.
  const ground=PNG.sync.read(await page.screenshot({
    path:path.join(output,"map-city-ground.png"),
    style:"[data-map-foreground],.strategic-map-markers{visibility:hidden!important}",
  }));
  const pixelRatio=await page.evaluate(()=>devicePixelRatio);
  const groundPixels=cold.markers.map(marker=>{
    const offset=(Math.round(marker.y*pixelRatio)*ground.width
      +Math.round(marker.x*pixelRatio))*4;
    return {place:marker.place,rgba:[...ground.data.subarray(offset,offset+4)]};
  });
  for(const pixel of groundPixels) {
    assert.equal(pixel.rgba[3],255,"Admitted pin ground is rendered opaque");
    // The map camera's authored clear color is sRGB (0.13, 0.16, 0.19).
    assert.notDeepEqual(pixel.rgba,[33,41,48,255],
      "The canonical landform renders ground beneath each admitted pin");
  }
  assert.equal(cityRequests.length,1,"Only the focused settlement is fetched");
  const capture=await page.evaluate(()=>renderProbe.capture("gpu",3));
  assert.deepEqual(capture.failures,[]);
  assert(capture.frames.some(frame=>frame.passes.some(pass=>pass.draws?.some(draw=>
    draw.pipeline?.label==="gpu_city" && draw.indirect && draw.triangles>0))),
    "The focused map city draws through the reflected geographic frame");
  fs.writeFileSync(path.join(output,"map-city-capture.json"),JSON.stringify(capture,null,2));
  await page.screenshot({path:path.join(output,"map-city-overhead.png")});
  checkpoint("drawn");
  await page.evaluate(()=>{
    mapCommand({type:"zoom",ratio:0.15});
  });
  // Use the production markup update so the controller owns the new overlay.
  await page.evaluate(()=>{
    document.dispatchEvent(new Event("strategic-page-unmounting"));
    mapOverlay={...mapOverlay,route:{kind:"computed",points:[
      {...mapOpen.origin,latitude:mapOpen.origin.latitude-200},mapOpen.origin,
      {...mapOpen.origin,latitude:mapOpen.origin.latitude+200}]}};
    document.querySelector("#strategic-page").innerHTML=mapMarkup();
    document.dispatchEvent(new Event("strategic-page-mounted"));
  });
  await page.waitForFunction(()=>mapStatus().city_visible && mapStatus().ready
    && document.body.hasAttribute("data-regional-map-ready"));
  await page.getByRole("button",{name:"Rotate right",exact:true}).click();
  await page.waitForFunction(()=>mapStatus().yaw>0.1 && mapStatus().ready);
  const beforePan=await page.evaluate(()=>mapStatus());
  await page.locator("[data-regional-map]").focus();
  await page.keyboard.press("ArrowRight");
  await page.waitForFunction(origin=>mapStatus().origin.longitude!==origin.longitude
    && mapStatus().ready && mapStatus().city_visible,beforePan.origin);
  await page.evaluate(()=>mapCommand({type:"zoom",ratio:40/mapStatus().span}));
  await page.waitForFunction(()=>mapStatus().span===40 && mapStatus().ready
    && mapStatus().city_visible);
  await page.screenshot({path:path.join(output,"map-city-streets.png")});
  const detail=await page.evaluate(()=>mapStatus());
  checkpoint("street-controls");
  assert.equal(detail.city_installation.preparation.sequence,cold.city_installation.preparation.sequence);
  assert.equal(cityRequests.length,1,"Zoom, rotation and nearby panning reuse the resident city");
  await page.evaluate(async()=>{
    document.dispatchEvent(new Event("strategic-page-unmounting"));
    document.querySelector("#strategic-page").innerHTML="<main>Journal</main>";
    document.dispatchEvent(new Event("strategic-page-mounted"));
    for(let frame=0;frame<6;frame++)await new Promise(requestAnimationFrame);
    document.querySelector("#strategic-page").innerHTML=mapMarkup();
    document.dispatchEvent(new Event("strategic-page-mounted"));
  });
  await page.waitForFunction(()=>document.body.hasAttribute("data-regional-map-ready")
    && mapStatus().city_visible);
  const warm=await page.evaluate(()=>({milliseconds:strategicRendererMetrics.maps.at(-1).milliseconds,
    state:mapStatus(),canvases:document.querySelectorAll("canvas").length}));
  assert.equal(warm.canvases,1);assert.equal(cityRequests.length,1);
  assert.equal(warm.state.city_installation.preparation.sequence,cold.city_installation.preparation.sequence);
  checkpoint("warm-return");
  await page.evaluate(async()=>{
    const {prepareGeneratedScene}=await import("/static/strategic-generation.js");
    const input=await fetch("/tactical/assets/tactical-scenes/sparse-woodland.json").then(response=>response.text());
    const preparation=await prepareGeneratedScene(runtime,input,{places:[],people:[]});
    runtime.wasm_command(JSON.stringify({type:"prepare-strategic-scene",location:"fixture",
      input_json:input,preparation:JSON.parse(preparation)}));
  });
  await page.waitForFunction(()=>JSON.parse(runtime.wasm_strategic_status()).ready
    && mapStatus().city_visible && mapStatus().ready,null,{timeout:120_000});
  const replacedActor=await page.evaluate(()=>mapStatus());
  assert.equal(replacedActor.city_installation.preparation.sequence,cold.city_installation.preparation.sequence,
    "Replacing actor scenery leaves the focused map city installed");
  assert.equal(cityRequests.length,1);
  checkpoint("actor-replaced");
  return {cold,groundPixels,detail,warm,replacedActor,cityRequests,terrainRequests};
};
