fn patch_point(vertex:u32)->vec3<f32> {
    return vec3<f32>(cloth_points[vertex*3u],cloth_points[vertex*3u+1u],cloth_points[vertex*3u+2u]);
}
fn patch_normal(vertex:u32)->vec3<f32> {
    return vec3<f32>(cloth_normals[vertex*3u],cloth_normals[vertex*3u+1u],cloth_normals[vertex*3u+2u]);
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x>=params.count) {return;}
    if (normal_status[0]!=0u) {atomicOr(&cloth_status[0],1u);return;}
    let word=id.x*SOURCE_WORDS;
    var point=vec3<f32>(0.0);var normal=vec3<f32>(0.0);
    for (var corner=0u;corner<3u;corner++) {
        let vertex=table[word+corner];let weight=bitcast<f32>(table[word+6u+corner]);
        let n=patch_normal(vertex);
        point+=(patch_point(vertex)-n*params.support)*weight;
        normal+=n*weight;
    }
    normal=host_normalized(normal);
    let first=FIRST_POINT+(params.first+id.x)*POINT_WORDS;
    fit[first]=point.x;fit[first+1u]=point.y;fit[first+2u]=point.z;
    fit[first+3u]=normal.x;fit[first+4u]=normal.y;fit[first+5u]=normal.z;
}
