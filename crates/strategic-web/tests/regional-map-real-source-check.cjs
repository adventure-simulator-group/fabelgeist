const assert=require("node:assert/strict");
const fs=require("node:fs");
const path=require("node:path");

// Render products emitted by the owning native capture implementation, using
// the production HTML bridge and one already-running Bevy instance.
module.exports=async function checkRealSource(page,output,directory) {
  if(!directory)return null;
  const products=new Map(["neighborhood","district","region","country","continent"].map(scale=>
    [scale,JSON.parse(fs.readFileSync(path.join(directory,scale+".json"),"utf8"))]));
  const environment=products.get("region"),requests=[];
  await page.route("**/api/map/environment/"+environment.terrain.source+"?**",async route=>{
    const query=new URL(route.request().url()).searchParams;
    const product=products.get(query.get("scale"));
    assert(product,"The real capture owns every admitted scale");
    assert.equal(Number(query.get("latitude")),product.terrain.request.origin.latitude);
    assert.equal(Number(query.get("longitude")),product.terrain.request.origin.longitude);
    requests.push(route.request().url());
    await route.fulfill({contentType:"application/json",body:JSON.stringify(product)});
  });
  await page.setViewportSize({width:1000,height:700});
  await page.evaluate(environment=>{
    document.dispatchEvent(new Event("strategic-page-unmounting"));
    mapOpen.source=environment.terrain.source;mapOpen.origin=environment.terrain.request.origin;
    mapOpen.span=30000;
    const origin=mapOpen.origin;
    mapOverlay={source:mapOpen.source,markers:[
      {place:"place:v1:settlement:66697874757265",origin,rank:"town",emphasis:"current"},
      {place:"place:v1:settlement:6d61702d726576696577",origin:{latitude:origin.latitude+1000,longitude:origin.longitude+1000},rank:"village",emphasis:"selected"},
    ],route:null};
    document.querySelector("#strategic-page").innerHTML=mapMarkup();
    document.dispatchEvent(new Event("strategic-page-mounted"));
  },environment);
  await page.waitForFunction(source=>document.body.hasAttribute("data-regional-map-ready")
    && mapStatus().source===source,environment.terrain.source,{timeout:30000});
  const cold=await page.evaluate(()=>mapStatus());
  assert.equal(cold.presented.scale,"region");assert(cold.connection_meshes>0);
  assert.equal(requests.length,1,"Terrain and roads share one real window response");
  await page.locator(".strategic-map-key summary").click();
  await page.waitForFunction(()=>document.querySelector(".strategic-map-key").open);
  await page.evaluate(async()=>{for(let frame=0;frame<4;frame++)await new Promise(requestAnimationFrame);});
  await page.screenshot({path:path.join(output,"real-source-desktop.png")});
  await page.evaluate(async()=>{
    document.dispatchEvent(new Event("strategic-page-unmounting"));
    document.querySelector("#strategic-page").innerHTML="<main>Journal</main>";
    document.dispatchEvent(new Event("strategic-page-mounted"));
    for(let frame=0;frame<6;frame++)await new Promise(requestAnimationFrame);
    document.querySelector("#strategic-page").innerHTML=mapMarkup();
    document.dispatchEvent(new Event("strategic-page-mounted"));
  });
  await page.waitForFunction(()=>document.body.hasAttribute("data-regional-map-ready"));
  const warm=await page.evaluate(()=>({milliseconds:strategicRendererMetrics.maps.at(-1).milliseconds,
    state:mapStatus(),canvases:document.querySelectorAll("canvas").length}));
  assert.equal(requests.length,1);assert.equal(warm.canvases,1);
  assert.equal(warm.state.connection_meshes,cold.connection_meshes);
  return {cold,warm,requests};
};
