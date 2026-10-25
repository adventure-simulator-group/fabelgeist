fn source_point(index: u32) -> vec3<f32> {
    return vec3<f32>(points[index*3u],points[index*3u+1u],points[index*3u+2u]);
}
fn collect_points(first: u32, start: u32, end: u32, height: f32) -> u32 {
    var count=0u;
    for (var i=start;i<end;i++) {
        let p=source_point(i);
        if (abs(p.y-height)<0.025) { sample_set(first+count,p.xz);count++; }
    }
    if (count>=16u || count==end-start) { return count; }
    count=0u;
    var last_distance=-1.0; var last=0u;
    for (var k=0u;k<min(16u,end-start);k++) {
        var best_distance=INFINITY; var best=0xffffffffu;
        for (var i=start;i<end;i++) {
            let distance=abs(source_point(i).y-height);
            let after=distance>last_distance || (distance==last_distance && i>last);
            if (after && distance<best_distance) { best_distance=distance;best=i; }
        }
        if (best==0xffffffffu) { break; }
        sample_set(first+count,source_point(best).xz);count++;
        last=best;last_distance=best_distance;
    }
    return count;
}
fn triangle_point(index: u32) -> vec3<f32> {
    return vec3<f32>(triangles[index*3u],triangles[index*3u+1u],triangles[index*3u+2u]);
}
fn collect_triangles(first: u32,height: f32,half_width: f32) -> u32 {
    var count=0u;
    let planes=vec2<f32>(height-half_width,height+half_width);
    for (var i=0u;i<params.faces;i++) {
        for (var corner=0u;corner<3u;corner++) {
            let a=triangle_point(i*3u+corner);let b=triangle_point(i*3u+(corner+1u)%3u);
            if (a.y>=planes.x && a.y<=planes.y) { sample_set(first+count,a.xz);count++; }
            for (var j=0u;j<2u;j++) {
                let y=planes[j];
                if ((a.y<y && b.y>y)||(b.y<y && a.y>y)) {
                    sample_set(first+count,mix(a.xz,b.xz,(y-a.y)/(b.y-a.y)));count++;
                }
            }
        }
    }
    return count;
}
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let section=id.x;let first=section*params.capacity;
    var count=0u;
    if (section<MEASURED_SECTIONS) {
        let waist=section==MEASURED_SECTIONS-1u;
        let height=select(mix(fit[16u],fit[17u],f32(section)/24.0),fit[17u],waist);
        count=collect_points(first,select(0u,params.thigh,waist),select(params.thigh,params.waist,waist),height);
    } else if (section<MEASURED_SECTIONS+params.nodes) {
        let step=(fit[19u]-fit[18u])/f32(params.nodes-1u);
        count=collect_triangles(first,fit[18u]+step*f32(section-MEASURED_SECTIONS),step);
    } else {
        let index=section-MEASURED_SECTIONS-params.nodes;
        count=collect_points(first,params.waist,params.point_count,mix(fit[23u],fit[24u],f32(index)/f32(LAYER_STATIONS-1u)));
    }
    let vertices=build_hull(first,count);
    if (vertices<3u || vertices>HULL_LIMIT) { atomicOr(&status[0],1u);return; }
    if (section>=MEASURED_SECTIONS) {
        let output=HULL_START+(section-MEASURED_SECTIONS)*HULL_WORDS;
        fit[output]=f32(vertices);
        for (var i=0u;i<vertices;i++) {
            let p=hull_at(first+i);fit[output+1u+i*2u]=p.x;fit[output+2u+i*2u]=p.y;
        }
        return;
    }
    var center=vec2<f32>(0.0);
    for (var i=0u;i<vertices;i++) { center+=hull_at(first+i)/f32(vertices); }
    let output=section*MEASURED_WORDS;
    measured[output]=center.x;measured[output+1u]=center.y;
    for (var i=0u;i<RADII;i++) {
        let angle=TAU*f32(i)/f32(RADII);
        measured[output+2u+i]=radius_from(first,vertices,center,vec2<f32>(cos(angle),sin(angle)));
    }
}
