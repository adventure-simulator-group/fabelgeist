// Fitted attachment points retain their construction field. Free skirt and
// transition geometry obtains its extrusion from the actual geometric normals.
fn skirt_authored_extrusion(coord: vec4<f32>) -> ShellExtrusion {
    if ((coord.w>0.5 && coord.w<1.5) || coord.w==6.0) {
        return ShellExtrusion(FITTED_SHELL_EXTRUSION, attachment_vector(u32(coord.x),3u));
    }
    return ShellExtrusion(GEOMETRIC_SHELL_EXTRUSION, vec3<f32>(0.0));
}
fn mixed_extrusion(a: ShellExtrusion, b: ShellExtrusion, blend: f32) -> ShellExtrusion {
    if (a.kind==FITTED_SHELL_EXTRUSION && b.kind==FITTED_SHELL_EXTRUSION) {
        return ShellExtrusion(FITTED_SHELL_EXTRUSION,mix(a.direction,b.direction,blend));
    }
    return ShellExtrusion(GEOMETRIC_SHELL_EXTRUSION,vec3<f32>(0.0));
}
fn skirt_edge_extrusion(coord: vec4<f32>) -> ShellExtrusion {
    if (coord.w==7.0) {
        let a=skirt_authored_extrusion(skirt_restored_coord(coords[u32(coord.x)]));
        let b=skirt_authored_extrusion(skirt_restored_coord(coords[u32(coord.y)]));
        return mixed_extrusion(a,b,coord.z);
    }
    return skirt_authored_extrusion(coord);
}
fn skirt_source_extrusion(coord: vec4<f32>) -> ShellExtrusion {
    if (coord.w==8.0) {
        let a=skirt_edge_extrusion(skirt_restored_coord(coords[u32(coord.x)]));
        let b=skirt_edge_extrusion(skirt_restored_coord(coords[u32(coord.y)]));
        return mixed_extrusion(a,b,coord.z);
    }
    return skirt_edge_extrusion(coord);
}
fn shell_extrusion(index: u32, coord: vec4<f32>) -> ShellExtrusion {
    let restored=skirt_restored_coord(coord);
    var field=skirt_source_extrusion(restored);
    if (field.kind==FITTED_SHELL_EXTRUSION && coord.w>=SLIT_COORD_VARIANTS) {
        // A dressing lip rotates its fitted point around the upright axis.
        // Its construction field undergoes the same rotation exactly once.
        let source=skirt_source_vertex(restored).point;
        let placed=shell_vertex(index,coord).point;
        let turn=atan2(placed.x,placed.z)-atan2(source.x,source.z);
        let c=cos(turn);let s=sin(turn);let n=field.direction;
        field.direction=vec3<f32>(c*n.x+s*n.z,n.y,-s*n.x+c*n.z);
    }
    return field;
}
