import {createRegionalEnvironmentRequests} from "./regional-environment-request.js";
import {installMapGestures} from "./regional-map-gestures.js";
import {createRegionalCityRequests} from "./regional-city-request.js";
import {installRegionalCity} from "./regional-city-installation.js";

// Canvas offsets enter Bevy as f32; tolerate only its subpixel wire rounding.
const VIEWPORT_OFFSET_TOLERANCE=1/1024;
const sameViewport=(presented,requested)=>presented
  && ["x","y","width","height","full_width","full_height"].every(field=>presented[field]===requested[field])
  && ["offset_x","offset_y"].every(field=>Math.abs(presented[field]-requested[field])<VIEWPORT_OFFSET_TOLERANCE);

// One document owner for geographic HTTP products and semantic camera intent.
// The existing strategic scene bridge owns the canvas and compositor mask.
export function createRegionalMapView({runtimePromise,changed,metrics}) {
  let runtime,host,input,scope,requests,cities,disposeGestures,lastRect,lastSelection;
  let error,runtimeError,openedAt,retryListener,revision=0,payload,installedWindow;
  const send=command=>runtime.wasm_command(JSON.stringify({type:"regional-map",command}));
  const fail=cause=>{
    error=cause;console.error("regional map unavailable",cause);
    const status=host?.querySelector("[data-map-status]");
    if(status){status.textContent="Map unavailable";status.hidden=false;}
    if(host)hideMarkers();
    document.body.removeAttribute("data-regional-map-ready");changed();
  };
  runtimePromise.then(value=>{runtime=value;changed();}).catch(cause=>{runtimeError=cause;fail(cause);});
  function suspend() {
    if(!disposeGestures)return;
    requests?.cancel();cities?.cancel();disposeGestures();disposeGestures=undefined;
    send({type:"hide"});lastRect=undefined;
    hideMarkers();
    document.body.removeAttribute("data-regional-map-ready");
  }
  function hide() {
    suspend();retryListener?.abort();retryListener=undefined;
    host=undefined;input=undefined;lastRect=undefined;
    document.body.removeAttribute("data-regional-map-ready");
  }
  function mount(element) {
    hide();host=element;hideMarkers();error=runtimeError;openedAt=performance.now();
    input=JSON.parse(host.querySelector("[data-regional-map-input]").textContent);
    payload=host.querySelector("[data-regional-map-input]").textContent;
    const nextScope=JSON.stringify([input.overlay.source,input.origin]);
    if (nextScope!==scope) {
      scope=nextScope;lastSelection=undefined;installedWindow=undefined;
      requests=createRegionalEnvironmentRequests({runtimePromise,
        install:environment=>send({type:"install-environment",environment}),
        changed:state=>{if(state.phase==="prepared")installedWindow=state.request;
          if(state.phase==="failed")console.error("regional environment unavailable",state.cause);changed();}});
      cities=createRegionalCityRequests({runtimePromise,
        install:installation=>installRegionalCity(runtime,installation),changed});
    }
    retryListener=new AbortController();
    host.querySelector('[data-map-action="retry"]')?.addEventListener("click",()=>{
      requests.retry();cities.retry();
    },
      {signal:retryListener.signal});
  }
  function hideMarkers() {
    for(const link of host.querySelectorAll("[data-map-place]"))link.hidden=true;
  }
  function updateMarkers(state) {
    const projected=new Map(state.markers.map(marker=>[marker.place,marker]));
    const bounds=host.getBoundingClientRect(),occupied=[];
    const edge=host.querySelector("[data-map-window]").getBoundingClientRect();
    for(const link of host.querySelectorAll("[data-map-place]")) {
      const point=projected.get(link.dataset.mapPlace);
      let hidden=!point;
      if(point) {
        link.style.left=`${point.x-bounds.left}px`;
        link.style.top=`${point.y-bounds.top}px`;
        link.dataset.mapLabelSide="right";
        if(link.hidden) link.hidden=false;
        let rect=link.getBoundingClientRect();
        if(rect.right>edge.right) {
          link.dataset.mapLabelSide="left";
          const alternate=link.getBoundingClientRect();
          if(alternate.left>=edge.left)rect=alternate;
          else link.dataset.mapLabelSide="right";
        }
        hidden=occupied.some(other=>rect.left<other.right+4 && rect.right>other.left-4
          && rect.top<other.bottom+4 && rect.bottom>other.top-4);
        if(!hidden)occupied.push(rect);
      }
      if(link.hidden!==hidden)link.hidden=hidden;
    }
  }
  function updateStatus(state,acknowledged) {
    const prepared=requests.state.phase==="prepared";
    const failed=requests.state.phase==="failed";
    const cityFocused=state.city_requested && cities.state.request?.place===state.city_requested;
    const cityFailed=cityFocused && cities.state.phase==="failed";
    const ready=!error && !failed && prepared && acknowledged && state.ready;
    document.body.toggleAttribute("data-regional-map-ready",ready);
    const status=host.querySelector("[data-map-status]");
    const message=error || failed ? "Map unavailable" : !prepared ? "Loading map…"
      : !acknowledged ? "Loading map…" : state.error==="connection-capacity" ? "Connections unavailable"
      : state.error==="route-capacity" ? "Route unavailable" : state.error ? "Map unavailable"
      : !state.covered ? "Terrain unavailable here" : cityFailed ? "City detail unavailable"
      : cityFocused && cities.state.phase==="loading" ? "Loading city detail…" : "";
    if(status.textContent!==message)status.textContent=message;
    if(status.hidden===Boolean(message))status.hidden=!message;
    const retry=host.querySelector('[data-map-action="retry"]');
    if(retry && retry.hidden===(failed || cityFailed))retry.hidden=!(failed || cityFailed);
    metrics.mapState=state;
    if(ready && openedAt!==undefined) {
      (metrics.maps ||= []).push({path:location.pathname,milliseconds:performance.now()-openedAt});
      if(metrics.maps.length>100)metrics.maps.shift();
      openedAt=undefined;
    }
  }
  return {
    hide,
    sync(page,rectFor) {
      const element=page.querySelector("[data-regional-map]");
      if(!element) {if(host)hide();return null;}
      try {
        if(element!==host || element.querySelector("[data-regional-map-input]").textContent!==payload)mount(element);
        const viewport=host.querySelector("[data-map-window]");
        const rect=rectFor(viewport);
        if(!rect){suspend();return null;}
        if(error || !runtime)return null;
        const key=JSON.stringify(rect);
        if(!disposeGestures) {
          send({type:"open",source:input.overlay.source,origin:input.origin,span:input.span,rect});
          revision++;
          if(!Number.isSafeInteger(revision))throw new Error("Map overlay revision exhausted");
          send({type:"install-overlay",revision,overlay:input.overlay});
          if(input.selected!==lastSelection && input.overlay.route)send({type:"frame-route"});
          lastSelection=input.selected;
          disposeGestures=installMapGestures(host,send,changed);
          lastRect=key;
        } else if(key!==lastRect) {send({type:"resize",rect});lastRect=key;}
        const state=JSON.parse(runtime.wasm_regional_map_status());
        const matched=state.source===input.overlay.source
          && state.home?.latitude===input.origin.latitude && state.home?.longitude===input.origin.longitude;
        if(matched && state.requested) {
          requests.request({source:input.overlay.source,...state.requested});
          if(state.city_requested)cities.request({source:input.overlay.source,place:state.city_requested});
          else if(cities.state.phase==="loading")cities.cancel();
          const acknowledged=terrainPresented(state) && sameViewport(state.rect,rect)
            && state.overlay_revision===revision && state.presentation_ready;
          if(acknowledged)updateMarkers(state);else hideMarkers();
          updateStatus(state,acknowledged);
        } else {
          hideMarkers();document.body.removeAttribute("data-regional-map-ready");
        }
        changed();
        return matched && terrainPresented(state) && sameViewport(state.rect,rect)
          && state.overlay_revision===revision && state.presentation_ready
          ? {element:viewport,rect} : null;
      } catch(cause) {fail(cause);return null;}
    },
  };
  function terrainPresented(state) {
    return installedWindow && state.presented?.scale===installedWindow.scale
      && state.presented?.origin.latitude===installedWindow.origin.latitude
      && state.presented?.origin.longitude===installedWindow.origin.longitude;
  }
}
