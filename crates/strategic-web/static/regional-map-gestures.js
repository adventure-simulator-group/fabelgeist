// Pointer and keyboard intent for Bevy's geographic camera. Coordinates here
// are browser pixels; all geographic projection and fitting stay in Bevy.
export function installMapGestures(host, send, changed) {
  const listeners = new AbortController(), pointers = new Map();
  const options = {signal:listeners.signal};
  const foreground = "[data-map-foreground],a,button,input,select,textarea,summary";
  let travelled = 0;
  const emit = command => { send(command); changed(); };
  const pan = (x,y) => emit({type:"pan",delta:[x*devicePixelRatio,y*devicePixelRatio]});
  const zoom = ratio => emit({type:"zoom",ratio:Math.max(0.05,Math.min(20,ratio))});
  const rotate = angle => emit({type:"rotate",angle});
  const pair = () => {
    const [first,second] = [...pointers.values()];
    if (!second) return null;
    return {x:(first.x+second.x)/2,y:(first.y+second.y)/2,
      distance:Math.hypot(second.x-first.x,second.y-first.y),
      angle:Math.atan2(second.y-first.y,second.x-first.x)};
  };
  host.addEventListener("pointerdown",event=>{
    if (event.target.closest(foreground)) {travelled=0;return;}
    if (pointers.size>=2) return;
    if (event.pointerType!=="touch" && ![0,1,2].includes(event.button)) return;
    event.preventDefault(); host.focus({preventScroll:true});
    if (!pointers.size) travelled=0;
    pointers.set(event.pointerId,{x:event.clientX,y:event.clientY,
      rotate:event.button!==0 || event.shiftKey});
    host.setPointerCapture(event.pointerId);
  },options);
  host.addEventListener("pointermove",event=>{
    const previous=pointers.get(event.pointerId);
    if (!previous) return;
    const before=pair(), dx=event.clientX-previous.x, dy=event.clientY-previous.y;
    travelled+=Math.hypot(dx,dy);
    pointers.set(event.pointerId,{...previous,x:event.clientX,y:event.clientY});
    const after=pair();
    if (before && after) {
      pan(after.x-before.x,after.y-before.y);
      if (before.distance>0 && after.distance>0) zoom(before.distance/after.distance);
      const angle=after.angle-before.angle;
      rotate(Math.atan2(Math.sin(angle),Math.cos(angle)));
    } else if (previous.rotate) rotate(dx*0.008);
    else pan(dx,dy);
  },options);
  const release=event=>pointers.delete(event.pointerId);
  for (const name of ["pointerup","pointercancel","lostpointercapture"]) {
    host.addEventListener(name,release,options);
  }
  host.addEventListener("click",event=>{
    if (event.detail && travelled>3 && !event.target.closest("button")) {
      event.preventDefault(); event.stopPropagation(); travelled=0;
    }
  },{...options,capture:true});
  host.addEventListener("contextmenu",event=>{
    if (!event.target.closest(foreground)) event.preventDefault();
  },options);
  host.addEventListener("wheel",event=>{
    if (event.target.closest(foreground)) return;
    event.preventDefault();
    const pixels=event.deltaY*(event.deltaMode===1 ? 16 : event.deltaMode===2 ? host.clientHeight : 1);
    zoom(Math.exp(Math.max(-2,Math.min(2,pixels*0.002))));
  },{...options,passive:false});
  host.addEventListener("keydown",event=>{
    if (event.target!==host) return;
    const commands={ArrowLeft:()=>pan(64,0),ArrowRight:()=>pan(-64,0),
      ArrowUp:()=>pan(0,64),ArrowDown:()=>pan(0,-64),
      "+":()=>zoom(0.8),"=":()=>zoom(0.8),"-":()=>zoom(1.25),
      q:()=>rotate(-0.15),e:()=>rotate(0.15),Home:()=>emit({type:"reset"}),
      f:()=>emit({type:"frame-route"})};
    const action=commands[event.key];
    if (action) {event.preventDefault();action();}
  },options);
  host.addEventListener("click",event=>{
    const button=event.target.closest("[data-map-action]");
    if (!button || !host.contains(button)) return;
    const actions={"zoom-in":()=>zoom(0.8),"zoom-out":()=>zoom(1.25),
      "rotate-left":()=>rotate(-0.15),"rotate-right":()=>rotate(0.15),
      reset:()=>emit({type:"reset"}),"frame-route":()=>emit({type:"frame-route"})};
    actions[button.dataset.mapAction]?.();
  },options);
  return ()=>{listeners.abort();pointers.clear();};
}
