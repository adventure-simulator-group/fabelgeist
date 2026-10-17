fn enclosure_source(index:u32)->PatchPoint {
    let start=index*SOURCE_WORDS;
    return cloth_source(vec3<u32>(enclosure_table[start],enclosure_table[start+1u],enclosure_table[start+2u]),
        vec3<f32>(bitcast<f32>(enclosure_table[start+6u]),bitcast<f32>(enclosure_table[start+7u]),bitcast<f32>(enclosure_table[start+8u])));
}
// One loose hanging enclosure spans the torso as well as the lower body.
// Shoulder points above the lowest bound neckline stay in the suspended cap.
fn loose_axes(frame:Frame)->vec2<f32> {
    var lowest_neck=3.402823e38;
    for (var source=0u;source<params.sources;source++) {
        if (neckline[source*NECK_WORDS]==source && neckline[source*NECK_WORDS+1u]==source) {
            lowest_neck=min(lowest_neck,host_local(frame,body_source(source).point).y);
        }
    }
    var axes=vec2<f32>(fit[16u+RADII/4u],fit[16u])-vec2<f32>(params.ease);
    for (var row=0u;row<=ROWS;row++) {
        axes=max(axes,vec2<f32>(fit[16u+row*RADII+RADII/4u],fit[16u+row*RADII])-vec2<f32>(params.ease));
    }
    for (var source=0u;source<params.enclosure_sources;source++) {
        let local=host_local(frame,enclosure_source(source).point);
        if (local.y<=lowest_neck) {axes=max(axes,abs(local.xz));}
    }
    var enclosure=1.0;
    for (var source=0u;source<params.enclosure_sources;source++) {
        let local=host_local(frame,enclosure_source(source).point);
        if (local.y<=lowest_neck) {enclosure=max(enclosure,length(local.xz/axes));}
    }
    return axes*enclosure;
}

@compute @workgroup_size(1)
fn main() {
    let axes=loose_axes(frame_at(0u));
    enclosure[0u]=axes.x;enclosure[1u]=axes.y;
}
