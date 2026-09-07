// Independent audit geometry. All tolerances are distances in model metres;
// these are tolerance-bounded numerical predicates, not exact arithmetic.
export const sub = (a, b) => a.map((v, i) => v - b[i]);
export const add = (a, b) => a.map((v, i) => v + b[i]);
export const mul = (a, s) => a.map((v) => v * s);
export const dot = (a, b) => a.reduce((s, v, i) => s + v * b[i], 0);
export const cross = (a, b) => [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]];
export const norm = (a) => Math.hypot(...a);
export const distance = (a, b) => norm(sub(a, b));
export const unit = (a) => mul(a, 1 / norm(a));
export function bounds(points) {
  const min = [Infinity, Infinity, Infinity], max = [-Infinity, -Infinity, -Infinity];
  for (const p of points) for (let i = 0; i < 3; i++) { min[i] = Math.min(min[i], p[i]); max[i] = Math.max(max[i], p[i]); }
  return { min, max };
}
export const overlaps = (a, b, e = 0) => a.min.every((v, i) => v <= b.max[i] + e && b.min[i] <= a.max[i] + e);
export function triangle(points, id, vertices = []) {
  const raw = cross(sub(points[1], points[0]), sub(points[2], points[0])), area2 = norm(raw);
  return { points, id, vertices, area2, normal: area2 ? mul(raw, 1/area2) : [0,0,0], ...bounds(points) };
}
export function bvh(triangles) {
  if (!triangles.length) return null;
  const box = bounds(triangles.flatMap((t) => [t.min, t.max]));
  if (triangles.length <= 12) return { ...box, triangles };
  const spans = sub(box.max, box.min), axis = spans.indexOf(Math.max(...spans));
  const ordered = [...triangles].sort((a,b) => a.min[axis]+a.max[axis]-b.min[axis]-b.max[axis] || a.id-b.id);
  const mid = ordered.length >> 1;
  return { ...box, left: bvh(ordered.slice(0,mid)), right: bvh(ordered.slice(mid)) };
}
export function query(tree, box, epsilon, visit) {
  if (!tree || !overlaps(tree, box, epsilon)) return;
  if (tree.triangles) { for (const t of tree.triangles) if (overlaps(t, box, epsilon)) visit(t); }
  else { query(tree.left, box, epsilon, visit); query(tree.right, box, epsilon, visit); }
}
export function pointSegmentDistance(p, a, b) {
  const ab = sub(b,a), denominator = dot(ab,ab);
  const t = denominator ? Math.max(0,Math.min(1,dot(sub(p,a),ab)/denominator)) : 0;
  return distance(p, add(a,mul(ab,t)));
}
export function insideTriangle(p, t, e) {
  return t.points.every((a,i) => {
    const edge = sub(t.points[(i+1)%3],a);
    return dot(cross(edge,sub(p,a)),t.normal) >= -e*norm(edge);
  });
}
export function pointTriangleDistance(p, t) {
  const d = dot(sub(p,t.points[0]),t.normal), projected = sub(p,mul(t.normal,d));
  if (t.area2 && insideTriangle(projected,t,0)) return Math.abs(d);
  return Math.min(...t.points.map((a,i) => pointSegmentDistance(p,a,t.points[(i+1)%3])));
}
function segmentDistance(p1,q1,p2,q2) {
  const d1=sub(q1,p1),d2=sub(q2,p2),r=sub(p1,p2),a=dot(d1,d1),e=dot(d2,d2),f=dot(d2,r);
  let s=0,t=0;
  if (!a) return pointSegmentDistance(p1,p2,q2);
  if (!e) return pointSegmentDistance(p2,p1,q1);
  const c=dot(d1,r),b=dot(d1,d2),den=a*e-b*b;
  if (den>Number.EPSILON*a*e) s=Math.max(0,Math.min(1,(b*f-c*e)/den));
  t=(b*s+f)/e;
  if(t<0){t=0;s=Math.max(0,Math.min(1,-c/a));}
  else if(t>1){t=1;s=Math.max(0,Math.min(1,(b-c)/a));}
  return distance(add(p1,mul(d1,s)),add(p2,mul(d2,t)));
}
export function triangleDistance(a,b,e=1e-10) {
  if (triangleContact(a,b,e)) return 0;
  let best=Math.min(...a.points.map(p=>pointTriangleDistance(p,b)),...b.points.map(p=>pointTriangleDistance(p,a)));
  for(let i=0;i<3;i++)for(let j=0;j<3;j++)best=Math.min(best,segmentDistance(a.points[i],a.points[(i+1)%3],b.points[j],b.points[(j+1)%3]));
  return best;
}
// Return the intersection polygon/segment/point, including coplanar cases.
// Callers decide whether that contact is an expected shared topological feature.
export function triangleContact(a,b,e) {
  if (!a.area2 || !b.area2 || !overlaps(a,b,e)) return null;
  // A welded shared vertex lies on both planes within the distance budget.
  // This includes periodic seams whose sin(2*pi) coordinate is not exactly 0.
  // Recomputing that known zero
  // with a rounded normal can create a false tiny segment when another vertex
  // is nearly coplanar (division by a small plane-distance difference).
  const side=(p,t)=>t.points.some(q=>distance(p,q)<=e)?0:dot(sub(p,t.points[0]),t.normal);
  const da=a.points.map(p=>side(p,b)), db=b.points.map(p=>side(p,a));
  if ([da,db].some(ds=>ds.every(d=>d>e)||ds.every(d=>d<-e))) return null;
  const points=[];
  const push=p=>{if(!points.some(q=>distance(p,q)<=e))points.push(p);};
  const coplanar=da.every(d=>Math.abs(d)<=e)&&db.every(d=>Math.abs(d)<=e);
  if(coplanar){
    // Clip A against the three inward half planes of B, in their common plane.
    let polygon=a.points;
    for(let i=0;i<3&&polygon.length;i++){
      const origin=b.points[i],edge=sub(b.points[(i+1)%3],origin),edgeLength=norm(edge);
      const side=p=>dot(cross(edge,sub(p,origin)),b.normal)/edgeLength;
      const output=[];
      for(let j=0;j<polygon.length;j++){
        const p=polygon[j],q=polygon[(j+1)%polygon.length],dp=side(p),dq=side(q),ip=dp>=-e,iq=dq>=-e;
        if(ip)output.push(p);
        if(ip!==iq&&Math.abs(dp-dq)>Number.EPSILON*edgeLength){
          const t=Math.max(0,Math.min(1,dp/(dp-dq)));output.push(add(p,mul(sub(q,p),t)));
        }
      }
      polygon=output;
    }
    polygon.forEach(push);
  } else {
    for(const [source,target,ds] of [[a,b,da],[b,a,db]])for(let i=0;i<3;i++){
      const p=source.points[i],q=source.points[(i+1)%3],dp=ds[i],dq=ds[(i+1)%3];
      if(Math.abs(dp)<=e&&insideTriangle(p,target,e))push(p);
      if((dp>0&&dq<0)||(dp<0&&dq>0)){
        const x=add(p,mul(sub(q,p),dp/(dp-dq)));if(insideTriangle(x,target,e))push(x);
      }
    }
  }
  if(!points.length)return null;
  let area=0;
  if(coplanar&&points.length>=3)for(let i=1;i<points.length-1;i++)area+=norm(cross(sub(points[i],points[0]),sub(points[i+1],points[0])))/2;
  return {points,coplanar,area};
}
export function unexpectedContact(a,b,contact,e) {
  const common=a.points.filter(p=>b.points.some(q=>distance(p,q)<=e));
  if(common.length===3)return true;
  if(!common.length)return true;
  if(common.length===1)return contact.points.some(p=>distance(p,common[0])>e*4);
  return contact.points.some(p=>pointSegmentDistance(p,common[0],common[1])>e*4);
}
// Generalized winding number avoids a ray direction accidentally hitting an edge.
// Only use for closed, non-intersecting shells and points off their surface.
export function containsPoint(triangles,p) {
  let angle=0;
  for(const t of triangles){
    const [a,b,c]=t.points.map(q=>sub(q,p)),la=norm(a),lb=norm(b),lc=norm(c);
    angle+=2*Math.atan2(dot(a,cross(b,c)),la*lb*lc+dot(a,b)*lc+dot(b,c)*la+dot(c,a)*lb);
  }
  return Math.abs(angle)>2*Math.PI;
}
export function rayDistance(origin,direction,t,e) {
  const denominator=dot(t.normal,direction);
  if(Math.abs(denominator)<1e-12)return Infinity;
  const d=dot(sub(t.points[0],origin),t.normal)/denominator;
  return d>e&&insideTriangle(add(origin,mul(direction,d)),t,e)?d:Infinity;
}
// Exact triangle-distance minimization up to the supplied predicate tolerance;
// bounding-box lower bounds prune pairs that cannot improve the current witness.
export function closestSurfaces(a,b,e=1e-9){
  let best={metres:Infinity,triangles:null};
  const lower=(x,y)=>Math.hypot(...x.min.map((v,i)=>Math.max(0,v-y.max[i],y.min[i]-x.max[i])));
  const visit=(x,y)=>{
    if(!x||!y||lower(x,y)>=best.metres)return;
    if(x.triangles&&y.triangles){
      for(const t of x.triangles)for(const u of y.triangles)if(lower(t,u)<best.metres){const d=triangleDistance(t,u,e);if(d<best.metres)best={metres:d,triangles:[t.id,u.id]};}
    }else{
      const pairs=x.triangles?[[x,y.left],[x,y.right]]:y.triangles?[[x.left,y],[x.right,y]]:norm(sub(x.max,x.min))>=norm(sub(y.max,y.min))?[[x.left,y],[x.right,y]]:[[x,y.left],[x,y.right]];
      pairs.sort(([x1,y1],[x2,y2])=>lower(x1,y1)-lower(x2,y2));for(const [p,q]of pairs)visit(p,q);
    }
  };
  visit(a,b);return best;
}
