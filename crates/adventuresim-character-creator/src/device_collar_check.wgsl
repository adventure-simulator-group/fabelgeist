// Check the shortest realized fold, including every supplied morph endpoint.
// This conservative sinusoidal radius predicate is not a collision proof.
fn collar_base(column:u32)->vec3<f32> {
    let a=FIRST_POINT+samples[column*3u]*POINT_WORDS;
    let b=FIRST_POINT+samples[column*3u+1u]*POINT_WORDS;
    let weight=bitcast<f32>(samples[column*3u+2u]);
    return mix3(vec3<f32>(fit[a],fit[a+1u],fit[a+2u]),
        vec3<f32>(fit[b],fit[b+1u],fit[b+2u]),weight);
}
@compute @workgroup_size(1)
fn main() {
    let intervals=(params.columns-1u)/params.pleats;
    var shortest=INFINITY;
    for(var fold=0u;fold<params.pleats;fold+=1u) {
        var pitch=0.0;
        for(var step=0u;step<intervals;step+=1u) {
            let column=fold*intervals+step;
            pitch+=length(collar_base(column+1u)-collar_base(column));
        }
        shortest=min(shortest,pitch);
    }
    let pitch=shortest*min(1.0,params.top_scale);
    let radius=pitch*pitch/(0.5*TAU*TAU*params.depth);
    if (!(params.thickness<radius)) {fit[FIT_FAILED]=1.0;}
}
