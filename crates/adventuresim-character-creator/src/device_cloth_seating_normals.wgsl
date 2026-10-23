fn carrier_point(vertex:u32)->vec3<f32> {
    let first=FIRST_POINT+vertex*POINT_WORDS;
    return vec3<f32>(fit[first],fit[first+1u],fit[first+2u]);
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if(id.x>=params.count || seating_domain[id.x]==0u){return;}
    let start=adjacency[id.x*2u];let count=adjacency[id.x*2u+1u];
    if(count==0u){return;}
    var normal=vec3<f32>(0.0);
    for(var i=0u;i<count;i++) {
        let first=(incidents[start+i]/3u)*3u;
        let a=carrier_point(carrier_faces[first]);let b=carrier_point(carrier_faces[first+1u]);let c=carrier_point(carrier_faces[first+2u]);
        normal+=host_cross(b-a,c-a);
    }
    normal=host_normalized(normal);
    let first=FIRST_POINT+id.x*POINT_WORDS;
    fit[first+3u]=normal.x;fit[first+4u]=normal.y;fit[first+5u]=normal.z;
}
