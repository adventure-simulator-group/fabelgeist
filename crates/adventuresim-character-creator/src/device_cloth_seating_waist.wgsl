@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if(id.x>=params.count){return;}
    let a=FIRST_POINT+samples[id.x*3u]*POINT_WORDS;let b=FIRST_POINT+samples[id.x*3u+1u]*POINT_WORDS;
    let blend=bitcast<f32>(samples[id.x*3u+2u]);
    let first=FIRST_POINT+(params.points+id.x)*POINT_WORDS;
    for(var axis=0u;axis<3u;axis++) {fit[first+axis]=mix(fit[a+axis],fit[b+axis],blend);}
    let normal=host_normalized(mix3(vec3<f32>(fit[a+3u],fit[a+4u],fit[a+5u]),vec3<f32>(fit[b+3u],fit[b+4u],fit[b+5u]),blend));
    fit[first+3u]=normal.x;fit[first+4u]=normal.y;fit[first+5u]=normal.z;
}
