fn cloth_point(index:u32)->vec3<f32> {
    return vec3<f32>(cloth_points[index*3u],cloth_points[index*3u+1u],cloth_points[index*3u+2u]);
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x>=params.count) {return;}
    let face=nearest[id.x]*3u;
    let a=cloth_point(triangles[face]);let b=cloth_point(triangles[face+1u]);let c=cloth_point(triangles[face+2u]);
    let word=id.x*3u;
    let normal=host_normalized(host_cross(b-a,c-a));
    // Keep the carrier's offset basis: shell evaluation adds it back together
    // with the tape gauge. This seats tape on the cloth rather than bare skin.
    let point=a*weights[word]+b*weights[word+1u]+c*weights[word+2u]-normal*params.carrier_offset;
    let first=FIRST_POINT+(params.first+id.x)*POINT_WORDS;
    fit[first]=point.x;fit[first+1u]=point.y;fit[first+2u]=point.z;
    fit[first+3u]=normal.x;fit[first+4u]=normal.y;fit[first+5u]=normal.z;
}
