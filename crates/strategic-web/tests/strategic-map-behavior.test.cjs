const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");
const {parseHTML} = require("linkedom");

const read = name => fs.readFileSync(path.join(__dirname,"../static",name),"utf8");
const turn = () => new Promise(resolve=>setImmediate(resolve));
const deferred = () => {let resolve;const promise=new Promise(yes=>resolve=yes);return {promise,resolve};};
const origin = {latitude:50_500_000,longitude:10_500_000};
const source = "a".repeat(64);
const input = (home=origin,selected=null) => ({origin:home,span:60000,selected,
  overlay:{source,markers:[],route:selected?{kind:"estimate",points:[home,{...home,latitude:home.latitude+100}]}:null}});
const rectangle = {x:10,y:20,width:600,height:400,full_width:600,full_height:400,offset_x:0,offset_y:0};

async function fixture({fetchTerrain,runtimePromise}={}) {
  const {document} = parseHTML("<html><body><main></main></body></html>");
  const commands=[],errors=[],metrics={},requests=[];
  let state={source,home:origin,requested:{origin,scale:"district"},ready:true,
    rect:rectangle,
    presented:{origin,scale:"district"},
    covered:true,presentation_ready:true,overlay_revision:0,markers:[]};
  const runtime={wasm_command(json){commands.push(JSON.parse(json).command);},
    wasm_regional_map_status(){return JSON.stringify(state);}};
  const context={document,window:{strategicFetch:async(url,options)=>{
    requests.push({url,options});
    if(fetchTerrain)return fetchTerrain(url,options);
    const query=new URL(url,"https://fixture.invalid").searchParams;
    return {ok:true,json:async()=>({source,request:{origin:{
      latitude:Number(query.get("latitude")),longitude:Number(query.get("longitude"))},
      scale:query.get("scale")},vertices:Array(65*65).fill(null)})};
  }},AbortController,URLSearchParams,performance,location:{pathname:"/map"},
    console:{error:(...args)=>errors.push(args)},
    installMapGestures(){return ()=>{};}};
  vm.createContext(context);
  vm.runInContext(read("regional-terrain-request.js").replaceAll("export ",""),context);
  vm.runInContext(read("regional-map-view.js").replace(/^import .*;\r?\n/gm,"").replaceAll("export ","")+
    "\nglobalThis.createView=createRegionalMapView;",context);
  const view=context.createView({runtimePromise:runtimePromise||Promise.resolve(runtime),changed(){},metrics});
  const page=document.querySelector("main");
  const mount=value=>{
    page.innerHTML='<section data-regional-map><script data-regional-map-input type="application/json"></script>'+
      '<div data-map-window></div><a data-map-place="current" href="?destination=current" hidden>Current</a>'+
      '<p data-map-status></p><button data-map-action="retry" hidden>Retry</button></section>';
    const host=page.querySelector("section");
    host.querySelector("script").textContent=JSON.stringify(value);
    host.getBoundingClientRect=()=>({left:10,top:20});
    host.querySelector("[data-map-window]").getBoundingClientRect=()=>({left:10,right:610});
    host.querySelector("a").getBoundingClientRect=()=>({left:50,top:50,right:90,bottom:70});
    return host;
  };
  const sync=(rect=rectangle)=>view.sync(page,()=>rect);
  const acknowledge=()=>{state.overlay_revision=commands.filter(c=>c.type==="install-overlay").at(-1).revision;};
  mount(input());await turn();
  return {view,page,document,commands,errors,metrics,requests,runtime,mount,sync,acknowledge,
    get state(){return state;},set state(value){state=value;}};
}

test("the current overlay must settle before terrain, pins and readiness are exposed",async()=>{
  const f=await fixture();
  assert.equal(f.sync(),null);await turn();
  assert.equal(f.requests.length,1);
  assert.equal(f.sync(),null,"Matching source and home cannot acknowledge an old overlay");
  assert.equal(f.document.body.hasAttribute("data-regional-map-ready"),false);
  f.acknowledge();f.state.presentation_ready=false;
  assert.equal(f.sync(),null);
  f.state.presentation_ready=true;f.state.presented={origin,scale:"neighborhood"};
  assert.equal(f.sync(),null,"Queued installation is insufficient until the renderer presents that product");
  f.state.presented=f.state.requested;
  f.state.presentation_ready=true;f.state.markers=[{place:"current",x:80,y:60}];
  assert(f.sync());
  assert.equal(f.page.querySelector("a").hidden,false);
  assert.equal(f.document.body.hasAttribute("data-regional-map-ready"),true);
  assert.equal(f.metrics.maps.length,1);
  f.sync();assert.equal(f.metrics.maps.length,1);
});

test("warm remount retains the installed window and frames a selection only once",async()=>{
  const f=await fixture();f.mount(input(origin,"selected"));f.sync();await turn();f.acknowledge();f.sync();
  f.view.hide();f.mount(input(origin,"selected"));f.sync();await turn();f.acknowledge();f.sync();
  assert.equal(f.requests.length,1);
  assert.equal(f.commands.filter(c=>c.type==="install-terrain").length,1);
  assert.equal(f.commands.filter(c=>c.type==="frame-route").length,1);
  f.mount(input(origin,"second"));f.sync();await turn();
  assert.equal(f.commands.filter(c=>c.type==="frame-route").length,2);
});

test("viewport changes stay masked until the resized camera is acknowledged and settled",async()=>{
  const f=await fixture();f.sync();await turn();f.acknowledge();assert(f.sync());
  const dense={...rectangle,x:20,y:40,width:1200,height:800,full_width:1200,full_height:800};
  assert.equal(f.sync(dense),null);
  assert.equal(f.document.body.hasAttribute("data-regional-map-ready"),false);
  f.state.rect=dense;f.state.presentation_ready=false;
  assert.equal(f.sync(dense),null);
  f.state.presentation_ready=true;assert(f.sync(dense));
  assert.equal(f.requests.length,1,"Changing pixel density reuses geographic terrain");
});

test("same-host knowledge updates conceal old pins until the replacement overlay settles",async()=>{
  const f=await fixture();f.sync();await turn();f.acknowledge();
  f.state.markers=[{place:"current",x:80,y:60}];f.sync();
  f.page.querySelector("script").textContent=JSON.stringify(input(origin,"changed"));
  assert.equal(f.sync(),null);
  assert.equal(f.page.querySelector("a").hidden,true);
  assert.equal(f.document.body.hasAttribute("data-regional-map-ready"),false);
  f.acknowledge();assert(f.sync());
  assert.equal(f.requests.length,1);
});

test("a new home owns a fresh terrain request even when its window matches the previous one",async()=>{
  const f=await fixture();f.sync();await turn();f.acknowledge();f.sync();
  const next={...origin,latitude:origin.latitude+10};
  f.mount(input(next));f.state.home=next;f.sync();await turn();f.acknowledge();f.sync();
  assert.equal(f.requests.length,2);
  assert.equal(f.commands.filter(c=>c.type==="install-terrain").length,2);
});

test("hiding cancels an obsolete HTTP response before it can install terrain",async()=>{
  const pending=deferred();
  const f=await fixture({fetchTerrain:()=>pending.promise});
  f.sync();await turn();f.acknowledge();
  assert.equal(f.sync(),null,"A settled old surface cannot expose a fresh owner before its terrain arrives");
  f.view.hide();
  assert.equal(f.requests[0].options.signal.aborted,true);
  pending.resolve({ok:true,json:async()=>({source,request:f.state.requested,vertices:Array(65*65).fill(null)})});
  await turn();
  assert.equal(f.commands.some(c=>c.type==="install-terrain"),false);
});

test("a clipped map suspends once and reopens without refetching resident terrain",async()=>{
  const f=await fixture();f.sync();await turn();f.acknowledge();f.sync();
  f.sync(null);f.sync(null);
  assert.equal(f.commands.filter(c=>c.type==="hide").length,1);
  assert.equal(f.document.body.hasAttribute("data-regional-map-ready"),false);
  f.sync();await turn();f.acknowledge();assert(f.sync());
  assert.equal(f.requests.length,1);
});

test("busy terrain waits for an explicit retry instead of sending a request every frame",async()=>{
  let fetches=0;
  const f=await fixture({fetchTerrain:async()=>++fetches===1?{ok:false,status:503}:
    {ok:true,json:async()=>({source,request:{origin,scale:"district"},vertices:Array(65*65).fill(null)})}});
  f.sync();await turn();f.acknowledge();f.sync();f.sync();
  assert.equal(fetches,1);
  assert.equal(f.page.querySelector("[data-map-status]").textContent,"Map unavailable");
  assert.equal(f.page.querySelector("button").hidden,false);
  f.page.querySelector("button").click();await turn();f.sync();
  assert.equal(fetches,2);
  assert.equal(f.document.body.hasAttribute("data-regional-map-ready"),true);
});

test("a route capacity failure preserves settled terrain and reports its local failure",async()=>{
  const f=await fixture();f.sync();await turn();f.acknowledge();
  f.state.ready=false;f.state.error="route-capacity";
  assert(f.sync(),"Settled terrain remains a compositor window");
  assert.equal(f.page.querySelector("[data-map-status]").textContent,"Route unavailable");
  assert.equal(f.document.body.hasAttribute("data-regional-map-ready"),false);
});

test("a pending renderer cannot start requests or expose stale links",async()=>{
  const pending=deferred();const f=await fixture({runtimePromise:pending.promise});
  assert.equal(f.sync(),null);assert.equal(f.requests.length,0);
  pending.resolve(f.runtime);await turn();f.sync();await turn();
  assert.equal(f.requests.length,1);
});
