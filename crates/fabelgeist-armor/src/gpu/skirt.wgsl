fn attachment_vector(point: u32, offset: u32) -> vec3<f32> {
    let first = ATTACHMENT + point * POINT_WORDS + offset;
    return vec3<f32>(frames[first], frames[first + 1u], frames[first + 2u]);
}

fn skirt_authored_vertex(coord: vec4<f32>) -> ShellVertex {
    if (coord.w>5.5) {
        let point=u32(coord.x);
        return ShellVertex(attachment_vector(point,0u)+attachment_vector(point,3u)*coord.y,0.0);
    }
    if (coord.w>4.5) {
        let sewn=attachment_vector(u32(coord.x),0u);
        return ShellVertex(vec3<f32>(sewn.x,mix(sewn.y,-fit.half_extents.y,coord.y),sewn.z),0.0);
    }
    if (coord.w > 2.5) {
        var column = coord.x;
        var row = coord.y;
        if (coord.w < 3.5) {
            let center = skirt_surface(column, row, 0.0);
            column += coord.z * params.value0 * 0.5 / length(center.xz) * f32(COLUMNS) / TAU;
        } else {
            // Reserve the complete band inside the ordinary skirt span. For a
            // tiny span, cap nominal width to each member's share and map the
            // whole pattern into the remaining center interval.
            let available = f32(ROWS) - design[FIRST_SKIRT_ROW];
            let half_rows = min(params.value0 * 0.5 / (2.0 * fit.half_extents.y) * f32(ROWS),
                available / (2.0 * params.value2));
            let descent = (row - design[FIRST_SKIRT_ROW]) / available;
            let center = design[FIRST_SKIRT_ROW] + half_rows + descent * (available - 2.0 * half_rows);
            row = center + coord.z * half_rows;
        }
        var point = skirt_surface(column, row, 0.0);
        point -= vec3<f32>(normalize(point.xz).x, 0.0, normalize(point.xz).y) * params.value1;
        return ShellVertex(point, 0.0);
    }
    if (coord.w < 0.5) { return ShellVertex(skirt_point(coord.x, coord.y), 0.0); }
    let body = attachment_vector(u32(coord.x), 0u);
    if (coord.w < 1.5) { return ShellVertex(body, 0.0); }
    let column = coord.y;
    let row = design[FIRST_SKIRT_ROW];
    let lower = skirt_point(column, row);
    let normal = attachment_vector(u32(coord.x), 3u);
    let angle = TAU * f32(column) / f32(COLUMNS);
    let radial = vec2<f32>(sin(angle), cos(angle));
    let height = body.y - lower.y;
    let slope = normal.y / max(dot(normal.xz, radial), MINIMUM_JOIN_RADIAL_NORMAL);
    let top_tangent = vec3<f32>(radial.x * slope * height, -height, radial.y * slope * height);
    let bottom_tangent = (skirt_point(column, row + 1.0) - lower)
        * (height / (2.0 * fit.half_extents.y / f32(ROWS)));
    let t = coord.z;
    let t2 = t * t;
    let t3 = t2 * t;
    let point = body * (2.0 * t3 - 3.0 * t2 + 1.0)
        + top_tangent * (t3 - 2.0 * t2 + t)
        + lower * (-2.0 * t3 + 3.0 * t2)
        + bottom_tangent * (t3 - t2);
    // Hermite tangents express the sewn transition, but can bow inside worn
    // layers between clear endpoints. Reapply the fitted radial enclosure at
    // the resulting physical height and angle, rather than the nominal row.
    let radial_length=length(point.xz);
    let actual_column=atan2(point.x,point.z)*f32(COLUMNS)/TAU;
    let actual_row=(fit.half_extents.y-point.y)/(2.0*fit.half_extents.y)*f32(ROWS);
    let required=radius(actual_row,actual_column);
    let supported=point.xz*max(1.0,required/radial_length);
    return ShellVertex(vec3<f32>(supported.x,point.y,supported.y), 0.0);
}

// Restore an original edge endpoint even if that endpoint also owns a lip.
fn skirt_restored_coord(coord:vec4<f32>)->vec4<f32> {
    var restored=coord;
    if (coord.w>=2.0*SLIT_COORD_VARIANTS) {restored.w-=2.0*SLIT_COORD_VARIANTS;}
    else if (coord.w>=SLIT_COORD_VARIANTS) {restored.w-=SLIT_COORD_VARIANTS;}
    return restored;
}
fn skirt_edge_vertex(coord:vec4<f32>)->ShellVertex {
    if (coord.w==7.0) {
        let a=skirt_authored_vertex(skirt_restored_coord(coords[u32(coord.x)]));
        let b=skirt_authored_vertex(skirt_restored_coord(coords[u32(coord.y)]));
        return ShellVertex(mix(a.point,b.point,coord.z),mix(a.height,b.height,coord.z));
    }
    return skirt_authored_vertex(coord);
}
fn skirt_source_vertex(coord:vec4<f32>)->ShellVertex {
    if (coord.w==8.0) {
        let a=skirt_edge_vertex(skirt_restored_coord(coords[u32(coord.x)]));
        let b=skirt_edge_vertex(skirt_restored_coord(coords[u32(coord.y)]));
        return ShellVertex(mix(a.point,b.point,coord.z),mix(a.height,b.height,coord.z));
    }
    return skirt_edge_vertex(coord);
}
// Native cut tags preserve original evaluation payload and split lip ownership.
const SLIT_AXIS:u32=19u;
const SLIT_GAP:u32=20u;
const SLIT_END_ROW:u32=21u;
const SLIT_TOP_ROW:u32=22u;
const SLIT_OVERLAP:u32=23u;
const SLIT_CLOTH_GAUGE:u32=15u;
fn shell_vertex(index:u32,coord:vec4<f32>)->ShellVertex {
    if (coord.w<SLIT_COORD_VARIANTS) {return skirt_source_vertex(coord);}
    let right=coord.w>=2.0*SLIT_COORD_VARIANTS;
    var restored=coord;
    restored.w-=select(SLIT_COORD_VARIANTS,2.0*SLIT_COORD_VARIANTS,right);
    let evaluated=skirt_source_vertex(restored);
    var point=evaluated.point;
    let row=(fit.half_extents.y-point.y)/(2.0*fit.half_extents.y)*f32(ROWS);
    var taper=clamp((design[SLIT_END_ROW]-row)/(design[SLIT_END_ROW]-design[SLIT_TOP_ROW]),0.0,1.0);
    if (restored.w==8.0 || (restored.w<0.5&&restored.y>=design[SLIT_END_ROW])) {taper=0.0;}
    var radius=length(point.xz);
    var angle=atan2(point.x,point.z)+select(-0.5,0.5,right)*design[SLIT_GAP]*taper/radius;
    let overlap=design[SLIT_OVERLAP];
    let cover_lower=overlap>0.0;
    if (overlap!=0.0 && right!=cover_lower) {
        angle+=(overlap+sign(overlap)*design[SLIT_GAP])*taper/radius;
        // Construction-scale nesting of the broad overlapping lip; not a
        // measured interlayer clearance or posed collision guarantee.
        radius+=design[SLIT_CLOTH_GAUGE]*taper;
    }
    point=vec3<f32>(sin(angle)*radius,point.y,cos(angle)*radius);
    return ShellVertex(point,evaluated.height);
}
