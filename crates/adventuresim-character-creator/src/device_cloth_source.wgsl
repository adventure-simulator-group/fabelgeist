struct PatchPoint { point: vec3<f32>, normal: vec3<f32> };

fn cloth_source(vertices:vec3<u32>,weights:vec3<f32>) -> PatchPoint {
    var point = vec3<f32>(0.0);
    var normal = vec3<f32>(0.0);
    for (var corner = 0u; corner < 3u; corner++) {
        let vertex = vertices[corner];
        let weight = weights[corner];
        point += positions_at(vertex) * weight;
        normal += normals_at(vertex) * weight;
    }
    normal = host_normalized(normal);
    return PatchPoint(point+normal*params.allowance,normal);
}


fn body_source(index:u32)->PatchPoint {
    let start=index*SOURCE_WORDS;
    return cloth_source(vec3<u32>(table[start],table[start+1u],table[start+2u]),
        vec3<f32>(bitcast<f32>(table[start+6u]),bitcast<f32>(table[start+7u]),bitcast<f32>(table[start+8u])));
}
