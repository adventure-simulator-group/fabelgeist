fn ribbon_vector(point:u32, offset:u32)->vec3<f32> {
    let first=FIRST_POINT+point*POINT_WORDS+offset;
    return vec3<f32>(fit[first],fit[first+1u],fit[first+2u]);
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x>=params.count) {return;}
    let word=id.x*4u;
    let previous=ribbon_vector(tape[word],0u);
    let center=ribbon_vector(tape[word+1u],0u);
    let next=ribbon_vector(tape[word+2u],0u);
    let normal=ribbon_vector(tape[word+1u],3u);
    var incoming=host_normalized(center-previous);
    var outgoing=host_normalized(next-center);
    if (length(center-previous)<FLT_EPSILON) {incoming=outgoing;}
    if (length(next-center)<FLT_EPSILON) {outgoing=incoming;}
    let inward=host_normalized(host_cross(normal,incoming+outgoing));
    // Bounded miter: acute cutter corners must not create long tape spikes.
    let scale=1.0/max(0.5,host_dot(inward,host_cross(normal,outgoing)));
    let point=center+inward*(bitcast<f32>(tape[word+3u])*scale);
    let first=FIRST_POINT+(params.first+id.x)*POINT_WORDS;
    fit[first]=point.x;fit[first+1u]=point.y;fit[first+2u]=point.z;
    fit[first+3u]=normal.x;fit[first+4u]=normal.y;fit[first+5u]=normal.z;
}
