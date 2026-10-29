@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if(id.x>=params.count || seating_domain[id.x]==0u){return;}
    let start=adjacency[id.x*2u];let count=adjacency[id.x*2u+1u];
    var shift=vec3<f32>(0.0);var samples=0.0;
    for(var i=0u;i<count;i++) {
        let first=incidents[start+i]*4u;
        shift+=vec3<f32>(face_shifts[first],face_shifts[first+1u],face_shifts[first+2u]);samples+=face_shifts[first+3u];
    }
    if(samples==0.0){return;}
    let frame=frame_at(0u);let first=FIRST_POINT+id.x*POINT_WORDS;
    let point=host_point(frame,vec3<f32>(fit[first],fit[first+1u],fit[first+2u]));
    let seated=host_local(frame,point+shift/samples);
    fit[first]=seated.x;fit[first+1u]=seated.y;fit[first+2u]=seated.z;
}
