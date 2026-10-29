fn carrier_point(vertex:u32)->vec3<f32> {
    let first=FIRST_POINT+vertex*POINT_WORDS;
    return host_point(frame_at(0u),vec3<f32>(fit[first],fit[first+1u],fit[first+2u]));
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if(id.x>=params.face_count){return;}
    let vertices=vec3<u32>(carrier_faces[id.x*3u],carrier_faces[id.x*3u+1u],carrier_faces[id.x*3u+2u]);
    if(seating_domain[vertices.x]==0u&&seating_domain[vertices.y]==0u&&seating_domain[vertices.z]==0u){
        for(var word=0u;word<12u;word++){face_shifts[id.x*12u+word]=0.0;}
        return;
    }
    let movable=vec3<f32>(select(0.0,1.0,seating_domain[vertices.x]!=0u),
        select(0.0,1.0,seating_domain[vertices.y]!=0u),select(0.0,1.0,seating_domain[vertices.z]!=0u));
    let points=array<vec3<f32>,3>(carrier_point(vertices.x),carrier_point(vertices.y),carrier_point(vertices.z));
    var shifts=array<vec4<f32>,3>(vec4<f32>(0.0),vec4<f32>(0.0),vec4<f32>(0.0));
    // Corners, centroid and three edge midpoints constrain actual carrier faces.
    for(var sample=0u;sample<7u;sample++) {
        var weights=vec3<f32>(0.0);
        if(sample<3u){weights[sample]=1.0;}
        else if(sample==3u){weights=vec3<f32>(1.0/3.0);}
        else {let edge=sample-4u;weights[edge]=0.5;weights[(edge+1u)%3u]=0.5;}
        // Fixed seam corners stay anchored; free corners carry the sampled
        // correction in proportion to their contribution to this point.
        let movable_weights=weights*movable;let squared=host_dot(movable_weights,movable_weights);
        if(squared==0.0){continue;}
        let point=points[0]*weights.x+points[1]*weights.y+points[2]*weights.z;
        let delta=cloth_fitted_point(point,params.clearance)-point;
        if(all(delta==vec3<f32>(0.0))){continue;}
        for(var corner=0u;corner<3u;corner++) {
            if(movable_weights[corner]>0.0){shifts[corner]+=vec4<f32>(delta*(movable_weights[corner]/squared),1.0);}
        }
    }
    for(var corner=0u;corner<3u;corner++) {
        let first=(id.x*3u+corner)*4u;
        face_shifts[first]=shifts[corner].x;face_shifts[first+1u]=shifts[corner].y;
        face_shifts[first+2u]=shifts[corner].z;face_shifts[first+3u]=shifts[corner].w;
    }
}
