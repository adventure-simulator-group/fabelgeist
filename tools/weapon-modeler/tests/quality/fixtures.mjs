export function tetra(offset=[0,0,0],scale=1,inward=false){
  const positions=[[0,0,0],[1,0,0],[0,1,0],[0,0,1]].flatMap(p=>p.map((v,i)=>v*scale+offset[i]));
  const indices=[0,2,1,0,1,3,0,3,2,1,2,3];
  if(inward)for(let i=0;i<indices.length;i+=3)[indices[i+1],indices[i+2]]=[indices[i+2],indices[i+1]];
  return {positions,indices};
}
export function combine(...meshes){
  const positions=[],indices=[];
  for(const mesh of meshes){const offset=positions.length/3;positions.push(...mesh.positions);indices.push(...mesh.indices.map(i=>i+offset));}
  return {positions,indices};
}
export function soup(points){return {positions:points.flat(2),indices:Array.from({length:points.length*3},(_,i)=>i)};}
