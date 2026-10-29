fn ribbon_vector(point:u32, offset:u32)->vec3<f32> {
    let first=FIRST_POINT+point*POINT_WORDS+offset;
    return vec3<f32>(fit[first],fit[first+1u],fit[first+2u]);
}
fn ribbon_carrier(point:u32)->vec3<f32> {
    return ribbon_vector(point,0u)+ribbon_vector(point,3u)*params.carrier_offset;
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x<params.carrier_count) {
        let p=ribbon_carrier(id.x);cloth_points[id.x*3u]=p.x;cloth_points[id.x*3u+1u]=p.y;cloth_points[id.x*3u+2u]=p.z;
    }
    if (id.x>=params.count) {return;}
    let word=id.x*TAPE_WORDS;
    let previous=ribbon_carrier(tape[word]);
    let center=ribbon_carrier(tape[word+1u]);
    let next=ribbon_carrier(tape[word+2u]);
    let previous_inside=ribbon_carrier(tape[word+4u]);
    let next_inside=ribbon_carrier(tape[word+5u]);
    // The source skin normal is not the normal of released or supported cloth.
    // Derive tape seating from its two actual fitted incident cloth faces.
    let normal=host_normalized(host_cross(center-previous,previous_inside-previous)+host_cross(next-center,next_inside-center));
    var incoming=host_normalized(center-previous);
    var outgoing=host_normalized(next-center);
    if (all(center==previous)) {incoming=outgoing;}
    if (all(next==center)) {outgoing=incoming;}
    let inward=host_normalized(host_cross(normal,incoming+outgoing));
    // Bounded miter: acute cutter corners must not create long tape spikes.
    let scale=1.0/max(0.5,host_dot(inward,host_cross(normal,outgoing)));
    let point=center+inward*(bitcast<f32>(tape[word+3u])*scale);
    queries[id.x*3u]=point.x;queries[id.x*3u+1u]=point.y;queries[id.x*3u+2u]=point.z;
}
