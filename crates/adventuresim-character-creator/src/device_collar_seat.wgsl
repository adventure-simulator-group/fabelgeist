@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x>=params.columns) {return;}
    let frame=collar_frame(id.x);
    let base=collar_sample(id.x);
    seating[id.x]=max(base.y,cloth_body_height(frame.center+frame.oval,base.y));
}
