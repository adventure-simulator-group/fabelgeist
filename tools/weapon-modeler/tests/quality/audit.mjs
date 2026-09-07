import { add, sub, mul, dot, cross, norm, distance, bounds, overlaps, triangle, bvh, query, triangleContact, unexpectedContact, containsPoint, triangleDistance, rayDistance } from "./geometry.mjs";

export const POLICY=Object.freeze({weldMetres:1e-9,relativeTolerance:1e-10,maxAspect:100,thinWallMetres:0.00005,contactMetres:0.000002,samplesPerCode:8});
function collector(){
  const findings=new Map();
  return {add(code,severity,example){const item=findings.get(code)??{code,severity,count:0,examples:[]};item.count++;if(item.examples.length<POLICY.samplesPerCode)item.examples.push(example);findings.set(code,item);},values(){return [...findings.values()];}};
}
// Spatial welding, including neighboring hash cells; never weld across parts.
function weld(points,e){
  const unique=[],cells=new Map();
  const ids=points.map(p=>{
    const cell=p.map(v=>Math.floor(v/e));let match;
    for(let x=-1;x<=1;x++)for(let y=-1;y<=1;y++)for(let z=-1;z<=1;z++){
      for(const id of cells.get([cell[0]+x,cell[1]+y,cell[2]+z].join(","))??[])if(distance(p,unique[id])<=e&&(match===undefined||id<match))match=id;
    }
    if(match!==undefined)return match;
    const id=unique.length;unique.push(p);const key=cell.join(","),bucket=cells.get(key)??[];bucket.push(id);cells.set(key,bucket);return id;
  });
  return {ids,unique};
}
function components(adjacency){
  const seen=new Set(),groups=[];
  for(let start=0;start<adjacency.length;start++)if(!seen.has(start)){
    const group=[],stack=[start];seen.add(start);
    while(stack.length){const next=stack.pop();group.push(next);for(const id of adjacency[next])if(!seen.has(id)){seen.add(id);stack.push(id);}}
    groups.push(group);
  }
  return groups;
}
export function auditPart(mesh,{intersections=true,thickness=false,epsilon,maxAspect=POLICY.maxAspect}={}){
  const out=collector(),positions=Array.from(mesh.positions??[]),indices=Array.from(mesh.indices??[]),normals=mesh.normals?Array.from(mesh.normals):null;
  if(!positions.length||positions.length%3||!positions.every(Number.isFinite)||!indices.length||indices.length%3||indices.some(i=>!Number.isInteger(i)||i<0||i>=positions.length/3)){
    out.add("invalid-buffer","error",{vertices:positions.length/3,indices:indices.length});return {findings:out.values(),triangles:[],shells:[],tree:null};
  }
  const points=Array.from({length:positions.length/3},(_,i)=>positions.slice(i*3,i*3+3)),box=bounds(points);
  const e=epsilon??Math.max(POLICY.weldMetres,norm(sub(box.max,box.min))*POLICY.relativeTolerance);
  const {ids,unique}=weld(points,e),triangles=[],edges=new Map(),faces=new Map(),links=Array.from({length:unique.length},()=>[]),used=new Set();
  let worstAspect={ratio:0},minArea=Infinity;
  for(let offset=0;offset<indices.length;offset+=3){
    const raw=indices.slice(offset,offset+3),v=raw.map(i=>ids[i]),id=offset/3,t=triangle(raw.map(i=>points[i]),id,v);triangles.push(t);raw.forEach(i=>used.add(i));
    const lengths=t.points.map((p,i)=>distance(p,t.points[(i+1)%3])),longest=Math.max(...lengths),aspect=t.area2?longest*longest/t.area2:Infinity;
    if(aspect>worstAspect.ratio)worstAspect={triangle:id,ratio:aspect,points:t.points};minArea=Math.min(minArea,t.area2/2);
    if(new Set(v).size<3||t.area2<=e*longest)out.add("degenerate-triangle","error",{triangle:id,area:t.area2/2,altitude:longest?t.area2/longest:0,points:t.points});
    else if(aspect>maxAspect)out.add("high-aspect-triangle","review",{triangle:id,ratio:aspect});
    const faceKey=[...v].sort((a,b)=>a-b).join(",");
    if(faces.has(faceKey))out.add("duplicate-triangle","error",{triangles:[faces.get(faceKey),id]});else faces.set(faceKey,id);
    if(mesh.normals){
      for(const vertex of raw){const normal=normals.slice(vertex*3,vertex*3+3),n=norm(normal);
        if(normal.length!==3||!normal.every(Number.isFinite)||Math.abs(n-1)>1e-4){out.add("invalid-normal","error",{triangle:id,vertex,length:n});break;}
        if(t.area2&&dot(t.normal,normal)<=0){out.add("normal-winding","error",{triangle:id,vertex});break;}
      }
    }
    for(let i=0;i<3;i++){
      const a=v[i],b=v[(i+1)%3],key=a<b?`${a},${b}`:`${b},${a}`,list=edges.get(key)??[];list.push({face:id,forward:a<b});edges.set(key,list);
      links[a].push([v[(i+1)%3],v[(i+2)%3]]);
    }
  }
  if(used.size<points.length)out.add("unused-vertices","review",{count:points.length-used.size});
  const adjacency=triangles.map(()=>new Set());
  for(const [edge,list]of edges){
    if(list.length!==2)out.add("edge-manifold","error",{edge,triangles:list.map(x=>x.face)});
    else if(list[0].forward===list[1].forward)out.add("edge-winding","error",{edge,triangles:list.map(x=>x.face)});
    for(const a of list)for(const b of list)if(a!==b)adjacency[a.face].add(b.face);
  }
  for(let vertex=0;vertex<links.length;vertex++){
    if(!links[vertex].length)continue;
    const neighbors=new Map();
    for(const [a,b]of links[vertex]){for(const [x,y]of [[a,b],[b,a]]){const list=neighbors.get(x)??[];list.push(y);neighbors.set(x,list);}}
    const reached=new Set(),stack=[neighbors.keys().next().value];
    while(stack.length){const n=stack.pop();if(reached.has(n))continue;reached.add(n);stack.push(...neighbors.get(n));}
    if(reached.size!==neighbors.size||[...neighbors.values()].some(ns=>ns.length!==2))out.add("vertex-manifold","error",{vertex,point:unique[vertex],disconnected:reached.size!==neighbors.size});
  }
  const groups=components(adjacency),shellOf=new Map();
  const shells=groups.map((faces,id)=>{
    faces.forEach(face=>shellOf.set(face,id));const ts=faces.map(i=>triangles[i]),origin=ts[0].points[0];
    const volume=ts.reduce((s,t)=>s+dot(sub(t.points[0],origin),cross(sub(t.points[1],origin),sub(t.points[2],origin)))/6,0);
    const vs=new Set(ts.flatMap(t=>t.vertices)),es=new Set(ts.flatMap(t=>t.vertices.map((v,i)=>[v,t.vertices[(i+1)%3]].sort((a,b)=>a-b).join(","))));
    return {id,faces,volume,euler:vs.size-es.size+faces.length,...bounds(ts.flatMap(t=>t.points))};
  });
  const tree=bvh(triangles.filter(t=>t.area2>0)),crossingShells=new Set();
  if(intersections)for(const a of triangles)query(tree,a,e,b=>{
    if(b.id<=a.id)return;
    const common=a.vertices.filter(v=>b.vertices.includes(v));
    // Noncoplanar triangles sharing an edge can intersect only at that edge.
    if(common.length===2&&norm(cross(a.normal,b.normal))>1e-7)return;
    const contact=triangleContact(a,b,e);if(!contact||!unexpectedContact(a,b,contact,e))return;
    const same=shellOf.get(a.id)===shellOf.get(b.id);
    if(!same){crossingShells.add(shellOf.get(a.id));crossingShells.add(shellOf.get(b.id));}
    out.add(same?"self-intersection":"inter-shell-contact",same?"error":"review",{triangles:[a.id,b.id],coplanar:contact.coplanar,area:contact.area,point:contact.points[0]});
  });
  const topologyInvalid=out.values().some(f=>["edge-manifold","edge-winding","vertex-manifold","degenerate-triangle","self-intersection"].includes(f.code));
  if(!topologyInvalid)for(const shell of shells){
    if(Math.abs(shell.volume)<=e**3){out.add("zero-volume-shell","error",{shell:shell.id,volume:shell.volume});continue;}
    if(crossingShells.has(shell.id))continue;
    const p=triangles[shell.faces[0]].points[0];
    const depth=shells.filter(other=>other!==shell&&!crossingShells.has(other.id)&&other.min.every((v,i)=>v<=shell.min[i]+e&&other.max[i]>=shell.max[i]-e)&&containsPoint(other.faces.map(i=>triangles[i]),p)).length;
    if((shell.volume>0)!==(depth%2===0))out.add("shell-orientation","error",{shell:shell.id,volume:shell.volume,nestingDepth:depth});
  }
  if(shells.length>1)out.add("multiple-shells","review",{count:shells.length});
  let wall=null;
  if(thickness&&!topologyInvalid){
    // A deterministic diagnostic, not a global minimum-thickness proof. Tips and
    // intentional cutting edges can legitimately be thinner than this budget.
    const stride=Math.max(1,Math.ceil(triangles.length/32));
    for(let i=0;i<triangles.length;i+=stride){
      const a=triangles[i],p=mul(a.points.reduce(add,[0,0,0]),1/3),direction=mul(a.normal,-1);
      let nearest=Infinity,target;
      for(const b of triangles)if(a!==b){const d=rayDistance(p,direction,b,e);if(d<nearest){nearest=d;target=b;}}
      if(target&&dot(a.normal,target.normal)<-0.5&&(!wall||nearest<wall.metres))wall={metres:nearest,triangles:[a.id,target.id]};
    }
    if(wall&&wall.metres<POLICY.thinWallMetres)out.add("thin-wall-sample","review",wall);
  }
  return {findings:out.values(),triangles,tree,shells,epsilon:e,bounds:box,worstAspect,minArea,wall};
}
export function publicPartAudit(a){return {findings:a.findings,shells:a.shells.map(({faces,...rest})=>({...rest,triangles:faces.length})),epsilon:a.epsilon,worstAspect:a.worstAspect,minArea:a.minArea,wall:a.wall};}

// Rules use explicit part indices and optionally triangle regions and a joint
// envelope. No implicit whole-component exemption for an intentional joint.
export function auditRelation(a,b,{mode="separate",regionA=()=>true,regionB=()=>true,joint,clearance=0}={}){
  const out=collector(),e=Math.max(a.epsilon,b.epsilon),threshold=Math.max(clearance,POLICY.contactMetres);
  let contact=false,minimum=Infinity;
  for(const x of a.triangles.filter(regionA))query(b.tree,x,threshold,y=>{
    if(!regionB(y))return;
    const hit=triangleContact(x,y,e),d=hit?0:triangleDistance(x,y,e);minimum=Math.min(minimum,d);
    if(d<=POLICY.contactMetres)contact=true;
    if(hit&&mode==="separate")out.add("forbidden-contact","error",{triangles:[x.id,y.id],point:hit.points[0]});
    if(hit&&mode==="joint"&&(!joint||hit.points.some(p=>!joint(p))))out.add("outside-joint","error",{triangles:[x.id,y.id],point:hit.points.find(p=>!joint?.(p))});
    if(!hit&&clearance>0&&d<clearance)out.add("insufficient-clearance","error",{triangles:[x.id,y.id],metres:d,required:clearance});
  });
  // Surface intersection alone cannot detect a fully buried shell. Use every
  // connected shell, not merely the first vertex of the combined part.
  if(mode!=="surface-only")for(const [source,target,region]of [[a,b,regionA],[b,a,regionB]])for(const shell of source.shells){
    const t=source.triangles[shell.faces[0]],p=t.points[0];
    if(!region(t)||!overlaps(shell,target.bounds,e))continue;
    let boundary=false;query(target.tree,{min:p,max:p},e,q=>{if(triangleDistance(triangle([p,p,p],-1),q,e)<=e)boundary=true;});
    if(!boundary&&containsPoint(target.triangles,p)){
      contact=true;
      if(mode==="separate"||(mode==="joint"&&!joint?.(p)))out.add("forbidden-containment","error",{shell:shell.id,point:p});
    }
  }
  // A fully buried piece can extend beyond its allowed joint without crossing
  // the other piece's surface. Its first shell vertex alone cannot certify this.
  if(mode==="joint"&&joint)for(const [source,target,region]of [[a,b,regionA],[b,a,regionB]]){
    const seen=new Set();
    for(const t of source.triangles.filter(region))for(const p of t.points){
      const key=p.join(",");if(seen.has(key)||joint(p))continue;seen.add(key);
      if(!target.bounds.min.every((v,i)=>p[i]>=v-e&&p[i]<=target.bounds.max[i]+e))continue;
      if(containsPoint(target.triangles,p))out.add("outside-joint","error",{point:p,contained:true});
    }
  }
  if(mode==="contact"&&!contact)out.add("missing-contact","error",{});
  return {findings:out.values(),contact,minimum:Number.isFinite(minimum)?minimum:null};
}
export function lodDifferences(reference,candidate){
  const errors=[];
  if(reference.length!==candidate.length)return [{code:"lod-part-count",severity:"error",count:1,examples:[{reference:reference.length,candidate:candidate.length}]}];
  for(let i=0;i<reference.length;i++){
    const signature=a=>a.shells.map(s=>s.euler).sort((a,b)=>a-b).join(",");
    if(signature(reference[i])!==signature(candidate[i]))errors.push({code:"lod-topology",severity:"error",count:1,examples:[{part:i,reference:signature(reference[i]),candidate:signature(candidate[i])}]});
  }
  return errors;
}
