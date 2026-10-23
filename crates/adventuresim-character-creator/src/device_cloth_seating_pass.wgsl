@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if(id.x>=params.count || seating_domain[id.x]==0u){return;}
    let first=FIRST_POINT+id.x*POINT_WORDS;
    let frame=frame_at(0u);
    let local=vec3<f32>(fit[first],fit[first+1u],fit[first+2u]);
    let point=host_point(frame,local);
    let seated=host_local(frame,cloth_fitted_point(point,params.clearance));
    fit[first]=seated.x;fit[first+1u]=seated.y;fit[first+2u]=seated.z;
}
