fn measured_radius(section: u32,direction: vec2<f32>) -> f32 {
    let coordinate=rem_tau(atan2(direction.y,direction.x))/TAU*f32(RADII);
    let index=u32(coordinate);let t=fract(coordinate);let t2=t*t;let t3=t2*t;
    let weights=vec4<f32>(pow(1.0-t,3.0),3.0*t3-6.0*t2+4.0,-3.0*t3+3.0*t2+3.0*t+1.0,t3)/6.0;
    var radius=0.0;
    for (var i=0u;i<4u;i++) { radius+=weights[i]*measured[section*MEASURED_WORDS+2u+(index+i+RADII-1u)%RADII]; }
    return radius;
}
fn ellipse_radius(radii: vec2<f32>,direction: vec2<f32>) -> f32 { return 1.0/length(direction/radii); }
@compute @workgroup_size(1)
fn main() {
    if (atomicLoad(&status[0])!=0u) { return; }
    let nodes=array<u32,3>(0u,12u,25u);
    for (var i=0u;i<3u;i++) {
        let node=nodes[i];
        var radii=vec2<f32>(max(measured_radius(node,vec2<f32>(1.0,0.0)),measured_radius(node,vec2<f32>(-1.0,0.0))),
            max(measured_radius(node,vec2<f32>(0.0,1.0)),measured_radius(node,vec2<f32>(0.0,-1.0))));
        var factor=1.0;
        for (var j=0u;j<RADII;j++) {
            let angle=TAU*f32(j)/f32(RADII);let direction=vec2<f32>(sin(angle),cos(angle));
            factor=max(factor,measured_radius(node,direction)/ellipse_radius(radii,direction));
        }
        radii*=factor;
        fit[32u+i*4u]=measured[node*MEASURED_WORDS];fit[33u+i*4u]=measured[node*MEASURED_WORDS+1u];
        fit[34u+i*4u]=radii.x;fit[35u+i*4u]=radii.y;
    }
    for (var k=0u;k<4u;k++) { fit[36u+k]=2.0*fit[36u+k]-(fit[32u+k]+fit[40u+k])*0.5; }
    var allowance=0.0;
    for (var node=0u;node<25u;node++) {
        let axial=f32(node)/24.0;
        let weights=vec3<f32>((1.0-axial)*(1.0-axial),2.0*axial*(1.0-axial),axial*axial);
        var section=vec4<f32>(0.0);
        for (var i=0u;i<3u;i++) {
            let at=32u+i*4u;section+=weights[i]*vec4<f32>(fit[at],fit[at+1u],fit[at+2u],fit[at+3u]);
        }
        if (any(section.zw<=vec2<f32>(0.0))) { atomicOr(&status[0],1u);return; }
        let center=vec2<f32>(measured[node*MEASURED_WORDS],measured[node*MEASURED_WORDS+1u]);
        for (var j=0u;j<RADII;j++) {
            let angle=TAU*f32(j)/f32(RADII);let direction=vec2<f32>(sin(angle),cos(angle));
            let delta=center+direction*measured_radius(node,direction)-section.xy;
            let distance=length(delta);
            if (distance>FLT_EPSILON) { allowance=max(allowance,distance-ellipse_radius(section.zw,delta/distance)); }
        }
    }
    fit[44u]=allowance;
}
