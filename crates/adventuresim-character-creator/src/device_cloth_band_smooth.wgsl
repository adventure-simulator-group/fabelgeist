// Smooth radial band profiles retain the open rear seam and span small gathers.
const BAND_FILTER_ARC_DIVISIONS:u32=24u;
fn band_point(index:u32)->vec3<f32> {
    let word=FIRST_POINT+index*POINT_WORDS;
    return vec3<f32>(fit[word],fit[word+1u],fit[word+2u]);
}
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x>=params.rows) {return;}
    let first=params.first+id.x*params.columns;
    var center=vec2<f32>(0.0);
    for(var column=0u;column+1u<params.columns;column+=1u) {
        center+=band_point(first+column).xz;
    }
    center/=f32(params.columns-1u);
    var radii:array<f32,MAX_COLLAR_COLUMNS>;
    var filtered:array<f32,MAX_COLLAR_COLUMNS>;
    for(var column=0u;column<params.columns;column+=1u) {
        radii[column]=length(band_point(first+column).xz-center);
    }
    // A short construction-scale neighborhood, independent of mesh resolution.
    let window=max(2u,(params.columns-1u)/BAND_FILTER_ARC_DIVISIONS);
    var expansion=0.0;
    for(var column=0u;column<params.columns;column+=1u) {
        let low=u32(max(0,i32(column)-i32(window)));
        let high=min(params.columns-1u,column+window);
        var total=0.0;var weights=0.0;
        for(var sample=low;sample<=high;sample+=1u) {
            let weight=f32(window+1u)-abs(f32(sample)-f32(column));
            total+=radii[sample]*weight;weights+=weight;
        }
        filtered[column]=total/weights;
        expansion=max(expansion,radii[column]-filtered[column]);
    }
    for(var column=0u;column<params.columns;column+=1u) {
        let point=band_point(first+column);
        let delta=point.xz-center;
        let direction=delta/max(length(delta),FLT_EPSILON);
        let envelope=center+direction*(filtered[column]+expansion);
        let word=FIRST_POINT+(first+column)*POINT_WORDS;
        fit[word]=envelope.x;fit[word+2u]=envelope.y;
        fit[word+3u]=direction.x;fit[word+4u]=0.0;fit[word+5u]=direction.y;
    }
}
