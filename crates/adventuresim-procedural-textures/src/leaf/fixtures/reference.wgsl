struct LeafParameters {
    profile: vec4f,       // half length, half width, widest-at, base exponent
    shape: vec4f,         // tip exponent, base width, tip width, asymmetry
    axis: vec4f,          // bend, petiole length, petiole width, midrib width
    notches: vec4f,       // basal depth/width, apical depth/width
    lobes: vec4f,         // count, depth, apex sharpness, side stagger
    teeth: vec4f,         // count, depth, sharpness, lean
    veins: vec4f,         // count, first, last, reach
    vein_style: vec4f,    // sweep, curve, width, terminal taper
    lobe_grade: vec4f,    // basal, middle, apical scale, progressive sweep
    margin_style: vec4f,  // secondary teeth, hierarchy, side asymmetry, bristle
    venation: vec4f,      // alternation, size gradation, branching, loops
    topology: vec4f,      // palmate blend, compound separation, fan blend, parallel
    organs: vec4f,        // ray/leaflet count, spread, length, width
    organ_style: vec4f,   // attachment, terminal scale, opposite/alternate, dichotomy
    hierarchy: vec4f,     // bipinnate amount, pinna count, leaflet count, pinna length
    landmarks: vec4f,     // basal obliquity, landmark lobe amount, side bias, apex truncate
    planar: vec4f,        // frond amount, fan segmentation, peltate depth, longitudinal veins
    blade_color: vec4f,
    vein_color: vec4f,
    render: vec4f,
}

@group(0) @binding(0) var<uniform> leaf: LeafParameters;

struct VertexOutput { @builtin(position) position: vec4f, @location(0) uv: vec2f }

@vertex fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var positions = array<vec2f, 3>(vec2f(-1.0,-1.0), vec2f(3.0,-1.0), vec2f(-1.0,3.0));
    let p = positions[i];
    var o: VertexOutput; o.position = vec4f(p,0.0,1.0); o.uv = p*vec2f(0.5,-0.5)+0.5; return o;
}

fn axis_x(t:f32)->f32 { return leaf.axis.x*sin(3.14159265*t); }
fn axis_point(t:f32)->vec2f { return vec2f(axis_x(t), leaf.profile.x*(1.0-2.0*t)); }
fn end_fade(t:f32)->f32 { return smoothstep(0.025,0.13,t)*smoothstep(0.025,0.13,1.0-t); }

fn profile_width(t:f32)->f32 {
    let w=clamp(leaf.profile.z,0.03,0.97);
    if t<w { let u=clamp(t/w,0.0,1.0); return mix(leaf.shape.y,1.0,pow(sin(1.5707963*u),leaf.profile.w)); }
    let u=clamp((1.0-t)/(1.0-w),0.0,1.0);
    return mix(leaf.shape.z,1.0,pow(sin(1.5707963*u),leaf.shape.x));
}

fn graded(t:f32)->f32 {
    if t<0.5 { return mix(leaf.lobe_grade.x,leaf.lobe_grade.y,smoothstep(0.0,0.5,t)); }
    return mix(leaf.lobe_grade.y,leaf.lobe_grade.z,smoothstep(0.5,1.0,t));
}

// Integer crossings are continuous: established organs slide continuously and the
// entering organ fades from zero instead of changing a ceil-based denominator.
fn organ_position(index:f32,count:f32,lo:f32,hi:f32)->f32 {
    let organic_offset=0.065*sin(index*12.9898+leaf.render.z*0.071)/max(count,1.0);
    return mix(lo,hi,clamp((index+0.5)/max(count,1.0)+organic_offset,0.0,1.0));
}

fn lobe_signal(t:f32,side:f32)->f32 {
    let count=clamp(leaf.lobes.x,0.0,16.0);
    var signal=0.0;
    for(var i=0u;i<16u;i++) {
        let fi=f32(i); let presence=clamp(count-fi,0.0,1.0);
        let c=organ_position(fi,count,0.10,0.91)+side*leaf.lobes.w/max(count,1.0);
        let spacing=0.81/max(count,1.0);
        let d=abs(t-c)/max(spacing*0.62,0.001);
        let rounded=pow(max(0.0,1.0-d*d),max(0.08,leaf.lobes.z));
        signal=max(signal,presence*rounded*graded(c));
    }
    return signal;
}

fn skew_triangle(x:f32,peak:f32)->f32 {
    let p=clamp(peak,0.05,0.95); return select((1.0-x)/(1.0-p),x/p,x<p);
}

fn tooth_signal(t:f32,side:f32)->f32 {
    let count=leaf.teeth.x;
    if count<=0.0 { return 0.0; }
    let hierarchy=max(1.0,leaf.margin_style.x);
    let base_phase=fract(count*t+select(0.0,0.5,side>0.0));
    let primary=pow(max(0.0,skew_triangle(base_phase,leaf.teeth.w)),max(0.05,leaf.teeth.z));
    let fine_phase=fract(count*hierarchy*t+0.37+select(0.0,0.5,side<0.0));
    let fine=pow(max(0.0,skew_triangle(fine_phase,leaf.teeth.w)),max(0.05,leaf.teeth.z));
    return mix(primary,max(primary,0.48*fine),clamp(leaf.margin_style.y,0.0,1.0));
}

fn half_width(t:f32,side:f32)->f32 {
    let base=leaf.profile.y*profile_width(t);
    let asym=leaf.shape.w*sin(3.14159265*t)+leaf.margin_style.z*(2.0*t-1.0)*0.15;
    let lobe=(1.0-leaf.lobes.y*(1.0-clamp(lobe_signal(t,side),0.0,1.5)))*end_fade(t)+(1.0-end_fade(t));
    let teeth=1.0+leaf.teeth.y*tooth_signal(t,side)*end_fade(t);
    let bristle=leaf.margin_style.w*pow(lobe_signal(t,side),8.0);
    let basal_oblique=leaf.landmarks.x*side*pow(max(0.0,1.0-t),3.0);
    let landmark_a=pow(max(0.0,1.0-abs(t-0.43)/0.13),1.4);
    let landmark_b=pow(max(0.0,1.0-abs(t-0.68)/0.12),1.4);
    let side_weight=clamp(1.0+side*leaf.landmarks.z,0.0,2.0);
    let use_two_landmarks=select(0.0,1.0,leaf.landmarks.w>0.0);
    let landmark=base*leaf.landmarks.y*side_weight*mix(landmark_a,max(landmark_a,landmark_b),use_two_landmarks);
    return max(0.0,base*(1.0-side*asym)*lobe*teeth+bristle+landmark+basal_oblique);
}

fn simple_field(p:vec2f)->f32 {
    let t=(leaf.profile.x-p.y)/(2.0*leaf.profile.x);
    if t<0.0 || t>1.0 { return -1.0; }
    let x=p.x-axis_x(t); let side=select(-1.0,1.0,x>=0.0);
    var field=half_width(t,side)-abs(x);
    if leaf.landmarks.w>0.0 && t>1.0-leaf.landmarks.w {
        let shoulder=half_width(1.0-leaf.landmarks.w,side);
        let distal_u=(t-(1.0-leaf.landmarks.w))/leaf.landmarks.w;
        let plateau=shoulder*mix(1.0,0.86,smoothstep(0.0,1.0,distal_u));
        field=plateau-abs(x);
    }
    if leaf.notches.x>0.0 && t<leaf.notches.x {
        let curved_opening=leaf.notches.y*pow(max(0.0,1.0-t/leaf.notches.x),0.58);
        field=min(field,abs(x)-curved_opening);
    }
    if leaf.notches.z>0.0 && t>1.0-leaf.notches.z {
        field=min(field,abs(x)-leaf.notches.w*(1.0-(1.0-t)/leaf.notches.z));
    }
    return field;
}

fn ellipse_field(p:vec2f,origin:vec2f,dir:vec2f,length_:f32,width_:f32)->f32 {
    let q=p-origin; let along=dot(q,dir); let across=dot(q,vec2f(-dir.y,dir.x));
    let center=length_*0.52; let a=max(length_*0.52,0.001); let b=max(width_,0.001);
    return 1.0-((along-center)*(along-center)/(a*a)+across*across/(b*b));
}

fn tapered_organ_field(p:vec2f,origin:vec2f,dir:vec2f,length_:f32,width_:f32)->f32 {
    let q=p-origin; let along=dot(q,dir); let across=abs(dot(q,vec2f(-dir.y,dir.x)));
    let u=along/max(length_,0.001);
    if u<0.0 || u>1.0 { return -1.0; }
    let taper=pow(max(0.0,sin(3.14159265*u)),0.58);
    let basal_width=mix(0.10,0.0,clamp(leaf.topology.y,0.0,1.0));
    let separated_base=smoothstep(0.035,0.28,u);
    let local_width=width_*mix(basal_width,1.0,taper)*mix(1.0,separated_base,leaf.topology.y);
    return local_width-across;
}

fn toothed_organ_field(p:vec2f,origin:vec2f,dir:vec2f,length_:f32,width_:f32,phase:f32)->f32 {
    let q=p-origin; let along=dot(q,dir); let across=abs(dot(q,vec2f(-dir.y,dir.x)));
    let u=along/max(length_,0.001);
    if u<0.0 || u>1.0 { return -1.0; }
    let taper=pow(max(0.0,sin(3.14159265*u)),0.58);
    let margin_tooth=pow(max(0.0,skew_triangle(fract(leaf.teeth.x*u+phase),leaf.teeth.w)),max(0.05,leaf.teeth.z));
    let basal_width=mix(0.10,0.0,clamp(leaf.topology.y,0.0,1.0));
    let separated_base=smoothstep(0.035,0.28,u);
    let local_width=width_*mix(basal_width,1.0,taper)*mix(1.0,separated_base,leaf.topology.y)*(1.0+leaf.teeth.y*margin_tooth);
    return local_width-across;
}

fn radial_field(p:vec2f,separate:f32)->f32 {
    let count=clamp(leaf.organs.x,1.0,11.0); let base=axis_point(clamp(leaf.organ_style.x+leaf.planar.z,0.0,0.55));
    var field=-10.0;
    for(var i=0u;i<11u;i++) {
        let fi=f32(i); let presence=clamp(count-fi,0.0,1.0);
        let u=select(0.5,clamp((fi+0.5)/count,0.0,1.0),count>1.0);
        let angle=mix(mix(-leaf.organs.y,leaf.organs.y,u),mix(-3.14159265,3.14159265,u),clamp(leaf.planar.z*2.0,0.0,1.0));
        let dir=normalize(vec2f(sin(angle),-cos(angle)));
        let center_weight=1.0-abs(2.0*u-1.0);
        let ordinal_wave=1.0+0.07*sin(fi*4.17+leaf.render.z*0.017);
        let len=leaf.organs.z*mix(0.70,leaf.organ_style.y,center_weight)*ordinal_wave;
        let blade_origin=base+dir*len*0.075*separate;
        let blade_length=len*(1.0-0.075*separate);
        let segment_width=leaf.organs.w*mix(1.8,1.0,separate)*mix(1.0,0.42,leaf.planar.y);
        // Fractional counts grow the entering organ from the hub. Scaling an
        // implicit field value instead leaves a full-size rib with a detached
        // remnant of blade near count boundaries.
        if segment_width*presence*leaf.render.y>=2.5 {
            let growing_origin=mix(base,blade_origin,presence);
            let ef=toothed_organ_field(p,growing_origin,dir,blade_length*presence,segment_width*presence,fract(fi*0.37));
            field=max(field,ef);
        }
    }
    // General simple-leaf web plus a palm-specific proximal sector. The latter
    // guarantees one connected blade through roughly the first third of each ray.
    let web= simple_field(p)-separate*mix(0.0,0.62,leaf.organs.y/1.5);
    let q=p-base; let fan_radius=leaf.organs.z*0.34;
    let fan_web=select(-1.0,fan_radius-length(q),leaf.planar.y>0.0 && q.y<=0.02);
    return max(max(field,web),fan_web);
}

fn pinnate_compound_field(p:vec2f)->f32 {
    let count=clamp(leaf.organs.x,1.0,12.0); let separation=clamp(leaf.topology.y,0.0,1.0); var field=-10.0;
    for(var i=0u;i<12u;i++) {
        let fi=f32(i); let presence=clamp(count-fi,0.0,1.0);
        let t=organ_position(fi,count,0.13,0.78); let origin=axis_point(t);
        for(var s=0u;s<2u;s++) {
            let side=select(-1.0,1.0,s==1u);
            let enabled=select(1.0,clamp(count-fi-0.5,0.0,1.0),leaf.organ_style.z>0.5 && s==1u);
            let dir=normalize(mix(vec2f(side*0.72,-0.69-leaf.lobe_grade.w),vec2f(side*0.96,-0.28-leaf.lobe_grade.w),leaf.planar.x));
            let scale=graded(t)*(1.0+0.055*sin(fi*5.13+side*1.7+leaf.render.z*0.013));
            let growth=presence*enabled*separation;
            if leaf.organs.w*scale*growth*leaf.render.y>=2.5 {
                let ef=toothed_organ_field(p,origin,dir,leaf.organs.z*scale*growth,leaf.organs.w*scale*growth,fract(fi*0.29+side*0.21));
                field=max(field,ef);
            }
        }
    }
    let terminal_growth=separation;
    if leaf.organs.w*leaf.organ_style.y*terminal_growth*leaf.render.y>=2.5 {
        let terminal=tapered_organ_field(p,axis_point(0.80),vec2f(0.0,-1.0),leaf.organs.z*leaf.organ_style.y*terminal_growth,leaf.organs.w*leaf.organ_style.y*terminal_growth);
        field=max(field,terminal);
    }
    return field;
}

fn bipinnate_field(p:vec2f)->f32 {
    let pinnae=clamp(leaf.hierarchy.y,1.0,6.0); let leaflets=clamp(leaf.hierarchy.z,1.0,10.0); var field=-10.0;
    let small_asset_scale=clamp(128.0/leaf.render.y,1.0,2.0);
    for(var i=0u;i<6u;i++) {
        let fi=f32(i); let pinna_present=clamp(pinnae-fi,0.0,1.0); let t=organ_position(fi,pinnae,0.16,0.76); let rachis=axis_point(t);
        if pinna_present<=0.0 { continue; }
        for(var ps=0u;ps<2u;ps++) {
            let side=select(-1.0,1.0,ps==1u); let pdir=normalize(vec2f(side*0.88,-0.48)); let plen=leaf.hierarchy.w*graded(t);
            // At zero, a pinna is a single narrow blade. Increasing bipinnation
            // retracts that web while paired second-order leaflets emerge on the
            // exact same axis; no unrelated simple blade is cross-faded in.
            let web_width=leaf.organs.w*2.2*(1.0-leaf.hierarchy.x)*pinna_present;
            if web_width*leaf.render.y>=3.0 {
                let pinna_web=tapered_organ_field(p,rachis,pdir,plen*pinna_present,web_width);
                field=max(field,pinna_web);
            }
            for(var j=0u;j<10u;j++) {
                let fj=f32(j); let leaflet_present=clamp(leaflets-fj,0.0,1.0); let u=(fj+0.45)/max(leaflets,1.0); let origin=mix(rachis,rachis+pdir*plen*u,pinna_present);
                if leaflet_present<=0.0 { continue; }
                let normal=vec2f(-pdir.y,pdir.x);
                for(var ls=0u;ls<2u;ls++) {
                    let lside=select(-1.0,1.0,ls==1u); let ldir=normalize(pdir*0.35+normal*lside);
                    let emergence=smoothstep(0.08,0.82,leaf.hierarchy.x);
                    let presence=pinna_present*leaflet_present;
                    if leaf.organs.w*presence*emergence*small_asset_scale*leaf.render.y>=3.0 {
                        let ef=tapered_organ_field(p,origin,ldir,leaf.organs.z*presence*emergence,leaf.organs.w*presence*emergence*small_asset_scale);
                        field=max(field,ef);
                    }
                }
            }
        }
    }
    return field;
}

fn blade_field(p:vec2f)->f32 {
    let simple=simple_field(p);
    let radial_blade_influence=leaf.topology.x*(1.0-leaf.organ_style.z);
    // Erode the shared web only as radial organization becomes established.
    // Interpolating two implicit fields directly can cancel their connecting
    // necks and create islands even though both endpoint shapes are valid.
    let radial_separation=leaf.topology.y*pow(smoothstep(0.65,1.0,radial_blade_influence),3.0);
    let radial=radial_field(p,radial_separation);
    let palmate=select(simple,radial,radial_blade_influence>0.0001);
    let fan=mix(palmate,radial_field(p,0.0),clamp(leaf.topology.z,0.0,1.0));
    let pinnate_compound=pinnate_compound_field(p);
    let pinnate_separation=clamp(leaf.topology.y*(1.0-leaf.topology.x),0.0,1.0);
    let pinnate_web=fan-pinnate_separation*1.25;
    let pinnate=max(pinnate_web,pinnate_compound);
    // Select the already-conditioned radial construction. A second implicit
    // interpolation here used to erase connecting tissue and leave isolated
    // remnants of the simple envelope.
    let primary=select(pinnate,palmate,radial_blade_influence>0.0001);
    return select(primary,bipinnate_field(p),leaf.hierarchy.y>0.0);
}

fn distance_segment(p:vec2f,a:vec2f,b:vec2f)->f32 {
    let d=b-a; return length(p-(a+d*clamp(dot(p-a,d)/max(dot(d,d),1e-7),0.0,1.0)));
}
fn bezier(a:vec2f,b:vec2f,c:vec2f,t:f32)->vec2f { return mix(mix(a,b,t),mix(b,c,t),t); }
fn line_hit(p:vec2f,a:vec2f,b:vec2f,width_:f32)->bool { return distance_segment(p,a,b)<width_; }
fn axis_hit(p:vec2f,start_t:f32,end_t:f32,width_:f32)->bool {
    for(var segment=0u;segment<16u;segment++) {
        let u0=f32(segment)/16.0; let u1=f32(segment+1u)/16.0;
        if line_hit(p,axis_point(mix(start_t,end_t,u0)),axis_point(mix(start_t,end_t,u1)),width_) { return true; }
    }
    return false;
}

fn secondary_hit(p:vec2f)->bool {
    let count=clamp(leaf.veins.x,0.0,16.0); let px=1.05/leaf.render.y;
    for(var i=0u;i<16u;i++) {
        let fi=f32(i); let present=clamp(count-fi,0.0,1.0);
        let base_t=organ_position(fi,count,leaf.veins.y,leaf.veins.z);
        for(var s=0u;s<2u;s++) {
            let side=select(-1.0,1.0,s==1u);
            let alternating=leaf.venation.x*0.45/max(count,1.0)*select(-1.0,1.0,(i+s)%2u==0u);
            let t=clamp(base_t+alternating,0.02,0.97); let origin=mix(axis_point(t),axis_point(0.04),leaf.topology.x);
            let coupled=abs(leaf.teeth.x-count)<0.75 && leaf.teeth.y>0.0;
            let tooth_target=(fi+leaf.teeth.w)/max(leaf.teeth.x,1.0);
            let target_t=select(clamp(t+leaf.vein_style.x+leaf.lobe_grade.w*(t-0.5),0.02,0.98),clamp(tooth_target,0.02,0.98),coupled);
            let scale=1.0+leaf.venation.y*(graded(t)-1.0);
            let endpoint=axis_point(target_t)+vec2f(side*half_width(target_t,side)*leaf.veins.w*scale,0.0);
            let control=mix(origin,endpoint,0.5)+vec2f(0.0,-leaf.vein_style.y*leaf.profile.x);
            for(var j=0u;j<9u;j++) {
                let u0=f32(j)/9.0; let u1=f32(j+1u)/9.0;
                let a=bezier(origin,control,endpoint,u0); let b=bezier(origin,control,endpoint,u1);
                let secondary_presence=1.0-clamp(leaf.topology.x,0.0,1.0);
                let width=max(px,leaf.vein_style.z*mix(1.0,leaf.vein_style.w,(u0+u1)*0.5)*secondary_presence)*present;
                if line_hit(p,a,b,width) { return true; }
                if leaf.venation.z>0.0 && j>4u {
                    let branch_end=b+vec2f(side*0.055,-0.035)*leaf.venation.z;
                    if line_hit(p,b,branch_end,width*0.62) { return true; }
                }
                if leaf.venation.w>0.0 && j==8u {
                    let loop_end=endpoint+vec2f(0.0,-0.09)*leaf.venation.w;
                    if line_hit(p,endpoint,loop_end,width*0.7) { return true; }
                }
            }
        }
    }
    return false;
}

fn radial_vein_hit(p:vec2f)->bool {
    let count=clamp(leaf.organs.x,1.0,11.0); let base=axis_point(clamp(leaf.organ_style.x+leaf.planar.z,0.0,0.55)); let px=1.05/leaf.render.y;
    if line_hit(p,axis_point(0.0),base,max(px,leaf.axis.w)) { return true; }
    for(var i=0u;i<11u;i++) {
        let fi=f32(i); let present=clamp(count-fi,0.0,1.0); let u=select(0.5,clamp((fi+0.5)/count,0.0,1.0),count>1.0);
        let angle=mix(mix(-leaf.organs.y,leaf.organs.y,u),mix(-3.14159265,3.14159265,u),clamp(leaf.planar.z*2.0,0.0,1.0)); let dir=normalize(vec2f(sin(angle),-cos(angle)));
        let center_weight=1.0-abs(2.0*u-1.0); let len=leaf.organs.z*mix(0.70,leaf.organ_style.y,center_weight)*(1.0+0.07*sin(fi*4.17+leaf.render.z*0.017));
        let owner_width=leaf.organs.w*mix(1.8,1.0,leaf.topology.y)*mix(1.0,0.42,leaf.planar.y)*present;
        if owner_width*leaf.render.y<2.5 { continue; }
        let dichotomy=leaf.organ_style.w;
        let first_fork=mix(1.0,0.36+0.16*fract(fi*0.618+0.21),dichotomy);
        let trunk_end=base+dir*len*first_fork*present;
        let radial_presence=clamp(leaf.topology.x,0.0,1.0);
        let peltate_scale=select(1.0,0.28,leaf.planar.z>0.0 || leaf.organ_style.z>0.5);
        if radial_presence*leaf.vein_style.z*peltate_scale*leaf.render.y>=0.50 && line_hit(p,base,trunk_end,max(px*0.72,leaf.vein_style.z*radial_presence*peltate_scale)*present) { return true; }
        if leaf.organ_style.w>0.0 {
            let normal=vec2f(-dir.y,dir.x);
            for(var branch=0u;branch<2u;branch++) {
                let bs=select(-1.0,1.0,branch==1u);
                let first_end=base+(dir*len*0.72+normal*bs*len*0.11*dichotomy)*present;
                if line_hit(p,trunk_end,first_end,max(px,leaf.vein_style.z*0.72)*present) { return true; }
                let second_fraction=0.67+0.12*fract(fi*0.414+f32(branch)*0.33);
                let second_origin=mix(trunk_end,first_end,second_fraction);
                for(var twig=0u;twig<2u;twig++) {
                    let ts=select(-1.0,1.0,twig==1u);
                    let second_end=first_end+(dir*len*0.20+normal*(bs*0.055+ts*0.045)*len*dichotomy)*present;
                    if line_hit(p,second_origin,second_end,max(px,leaf.vein_style.z*0.46)*present) { return true; }
                }
            }
        } else {
            if line_hit(p,trunk_end,base+dir*len*present,max(px,leaf.vein_style.z)*present) { return true; }
        }
    }
    return false;
}

fn pinnate_organ_vein_hit(p:vec2f)->bool {
    let count=clamp(leaf.organs.x,1.0,12.0); let px=1.05/leaf.render.y;
    for(var i=0u;i<12u;i++) {
        let fi=f32(i); let present=clamp(count-fi,0.0,1.0); let t=organ_position(fi,count,0.13,0.78); let origin=axis_point(t);
        for(var s=0u;s<2u;s++) {
            let side=select(-1.0,1.0,s==1u); let enabled=select(1.0,clamp(count-fi-0.5,0.0,1.0),leaf.organ_style.z>0.5 && s==1u);
            let dir=normalize(mix(vec2f(side*0.72,-0.69-leaf.lobe_grade.w),vec2f(side*0.96,-0.28-leaf.lobe_grade.w),leaf.planar.x));
            let scale=graded(t)*(1.0+0.055*sin(fi*5.13+side*1.7+leaf.render.z*0.013));
            let organ_endpoint=origin+dir*leaf.organs.z*scale;
            let simple_t=clamp(t+leaf.vein_style.x,0.02,0.98);
            let simple_endpoint=axis_point(simple_t)+vec2f(side*half_width(simple_t,side)*leaf.veins.w,0.0);
            let mature_endpoint=mix(simple_endpoint,organ_endpoint,leaf.topology.y);
            let growth=present*enabled;
            let endpoint=mix(origin,mature_endpoint,growth);
            if leaf.organs.w*scale*growth*leaf.render.y<2.5 { continue; }
            if line_hit(p,origin,endpoint,max(px,leaf.vein_style.z*growth)) { return true; }
            if leaf.planar.w>0.0 {
                let owner=toothed_organ_field(p,origin,dir,leaf.organs.z*scale*growth,leaf.organs.w*scale*growth,fract(fi*0.29+side*0.21));
                let normal=vec2f(-dir.y,dir.x); let offset=leaf.organs.w*scale*0.28;
                if owner>=px*1.5 && line_hit(p,origin+normal*offset*growth,endpoint+normal*offset*growth,max(px,leaf.vein_style.z*0.45*growth)*leaf.planar.w) { return true; }
                if owner>=px*1.5 && line_hit(p,origin-normal*offset*growth,endpoint-normal*offset*growth,max(px,leaf.vein_style.z*0.45*growth)*leaf.planar.w) { return true; }
            }
        }
    }
    let terminal_origin=axis_point(0.80); let terminal_end=terminal_origin+vec2f(0.0,-leaf.organs.z*leaf.organ_style.y*leaf.topology.y);
    return line_hit(p,terminal_origin,terminal_end,max(px,leaf.vein_style.z));
}

fn compound_local_secondary_hit(p:vec2f)->bool {
    let separation=leaf.topology.y;
    if separation<0.30 { return false; }
    let px=1.05/leaf.render.y;
    if leaf.topology.x>0.5 {
        let count=clamp(leaf.organs.x,1.0,11.0); let base=axis_point(clamp(leaf.organ_style.x,0.0,0.35));
        for(var i=0u;i<11u;i++) {
            let fi=f32(i); let present=clamp(count-fi,0.0,1.0); let u=select(0.5,(fi+0.5)/count,count>1.0);
            let angle=mix(-leaf.organs.y,leaf.organs.y,u); let dir=normalize(vec2f(sin(angle),-cos(angle)));
            let center_weight=1.0-abs(2.0*u-1.0); let len=leaf.organs.z*mix(0.70,leaf.organ_style.y,center_weight)*(1.0+0.07*sin(fi*4.17+leaf.render.z*0.017));
            let width=leaf.organs.w; let blade_origin=base+dir*len*0.075*separation;
            let blade_length=len*(1.0-0.075*separation);
            let organ_field=toothed_organ_field(p,blade_origin,dir,blade_length,width,fract(fi*0.37));
            if organ_field>=0.0 && present>0.0 {
                let normal=vec2f(-dir.y,dir.x);
                for(var j=0u;j<3u;j++) {
                    let along=(0.30+f32(j)*0.18)*len; let origin=base+dir*along;
                    let local_half=width*pow(max(0.0,sin(3.14159265*along/len)),0.58)*0.78;
                    for(var s=0u;s<2u;s++) {
                        let side=select(-1.0,1.0,s==1u); let endpoint=origin+dir*len*0.07+normal*side*local_half;
                        if line_hit(p,origin,endpoint,max(px,leaf.vein_style.z*0.50*separation)*present) { return true; }
                    }
                }
            }
        }
    } else {
        let count=clamp(leaf.organs.x,1.0,12.0);
        for(var i=0u;i<12u;i++) {
            let fi=f32(i); let present=clamp(count-fi,0.0,1.0); let t=organ_position(fi,count,0.13,0.78); let base=axis_point(t);
            for(var side_index=0u;side_index<2u;side_index++) {
                let side=select(-1.0,1.0,side_index==1u); let dir=normalize(vec2f(side*0.72,-0.69-leaf.lobe_grade.w)); let normal=vec2f(-dir.y,dir.x);
                let scale=graded(t); let len=leaf.organs.z*scale; let width=leaf.organs.w*scale;
                let organ_field=toothed_organ_field(p,base,dir,len,width,fract(fi*0.29+side*0.21));
                if organ_field>=0.0 && present>0.0 {
                    for(var j=0u;j<2u;j++) {
                        let along=(0.36+f32(j)*0.23)*len; let origin=base+dir*along;
                        let local_half=width*pow(max(0.0,sin(3.14159265*along/len)),0.58)*0.76;
                        for(var s=0u;s<2u;s++) {
                            let branch_side=select(-1.0,1.0,s==1u); let endpoint=origin+dir*len*0.065+normal*branch_side*local_half;
                            if line_hit(p,origin,endpoint,max(px,leaf.vein_style.z*0.48*separation)*present) { return true; }
                        }
                    }
                }
            }
        }
    }
    return false;
}

fn basal_primary_hit(p:vec2f)->bool {
    if leaf.notches.x<=0.0 && leaf.landmarks.y<=0.0 { return false; }
    let base=axis_point(0.03); let px=1.05/leaf.render.y;
    for(var level=0u;level<2u;level++) {
        let landmark_level=select(0.0,1.0,level==1u && leaf.landmarks.w>0.0);
        if level==1u && landmark_level<=0.0 { continue; }
        for(var s=0u;s<2u;s++) {
            let side=select(-1.0,1.0,s==1u);
            let lobe_presence=clamp(leaf.landmarks.y*(1.0+side*leaf.landmarks.z),0.0,1.0);
            let cordate_presence=select(0.0,1.0,leaf.notches.x>0.0 && level==0u);
            let presence=max(cordate_presence,lobe_presence);
            let landmark_t=mix(0.43,0.68,landmark_level);
            let target_t=mix(0.22,landmark_t,select(0.0,1.0,leaf.landmarks.y>0.0));
            let endpoint=axis_point(target_t)+vec2f(side*half_width(target_t,side)*0.90,0.0);
            let growth=smoothstep(0.0,0.38,presence);
            if presence*leaf.vein_style.z*leaf.render.y>=0.75 && line_hit(p,base,mix(base,endpoint,growth),max(px,leaf.vein_style.z*presence)) { return true; }
        }
    }
    return false;
}

fn parallel_hit(p:vec2f)->bool {
    let amount=leaf.topology.w; if amount<=0.0 { return false; }
    let count=clamp(leaf.veins.x,1.0,16.0); let px=1.05/leaf.render.y;
    for(var i=0u;i<16u;i++) {
        let fi=f32(i); let present=clamp(count-fi,0.0,1.0); let u=(fi+0.5)/count; let offset=(u*2.0-1.0)*leaf.profile.y*0.72;
        let a=axis_point(0.08)+vec2f(offset*0.25,0.0); let b=axis_point(0.93)+vec2f(offset,0.0);
        if line_hit(p,a,b,max(px,leaf.vein_style.z*0.7)*present*amount) { return true; }
    }
    return false;
}

fn bipinnate_vein_hit(p:vec2f)->bool {
    if leaf.hierarchy.y<=0.0 { return false; }
    let pinnae=clamp(leaf.hierarchy.y,1.0,6.0); let leaflets=clamp(leaf.hierarchy.z,1.0,10.0); let px=1.05/leaf.render.y;
    if axis_hit(p,0.0,0.84,max(px*1.5,leaf.axis.w)) { return true; }
    for(var i=0u;i<6u;i++) {
        let fi=f32(i); let pp=clamp(pinnae-fi,0.0,1.0); let t=organ_position(fi,pinnae,0.16,0.76); let base=axis_point(t);
        if pp<=0.0 { continue; }
        for(var ps=0u;ps<2u;ps++) {
            let side=select(-1.0,1.0,ps==1u); let dir=normalize(vec2f(side*0.88,-0.48)); let plen=leaf.hierarchy.w*graded(t);
            let emergence=smoothstep(0.08,0.82,leaf.hierarchy.x);
            let owner_width=max(leaf.organs.w*2.2*(1.0-leaf.hierarchy.x),leaf.organs.w*emergence)*pp;
            if owner_width*leaf.render.y<3.0 { continue; }
            let occupied_fraction=clamp((leaflets-0.45)/max(leaflets,1.0),0.25,0.96);
            let end=base+dir*plen*occupied_fraction;
            if line_hit(p,base,mix(base,end,pp),max(px*1.35,leaf.vein_style.z*pp)) { return true; }
            let normal=vec2f(-dir.y,dir.x);
            for(var j=0u;j<10u;j++) {
                let fj=f32(j); let lp=clamp(leaflets-fj,0.0,1.0); let origin=mix(base,base+dir*plen*((fj+0.45)/max(leaflets,1.0)),pp);
                if lp<=0.0 { continue; }
                let presence=pp*lp*emergence;
                if leaf.organs.w*presence*leaf.render.y<3.0 { continue; }
                for(var ls=0u;ls<2u;ls++) { let lside=select(-1.0,1.0,ls==1u); let ldir=normalize(dir*0.35+normal*lside); if line_hit(p,origin,origin+ldir*leaf.organs.z*0.92*presence,max(px,leaf.vein_style.z*0.6*presence)) { return true; } }
            }
        }
    }
    return false;
}

@fragment fn fs_main(input:VertexOutput)->@location(0) vec4f {
    let aspect=leaf.render.x/leaf.render.y; let p=(input.uv-vec2f(0.5))*vec2f(2.0*aspect,2.0);
    let field=blade_field(p); let inside=field>=0.0;
    let t=clamp((leaf.profile.x-p.y)/(2.0*leaf.profile.x),0.0,1.0); let px=1.05/leaf.render.y;
    let pinnate_compound=leaf.topology.y>0.01 && leaf.topology.x<0.5;
    let ordinary_simple=leaf.topology.x<0.01 && leaf.topology.y<0.01;
    let basal_attachment=leaf.notches.x>0.0 && t<leaf.notches.x;
    let terminal_origin=axis_point(0.80);
    let rachis_end=terminal_origin+vec2f(0.0,-leaf.organs.z*leaf.organ_style.y*0.86*leaf.topology.y);
    let simple_midrib=ordinary_simple && (inside || basal_attachment) && axis_hit(p,0.0,0.98,max(px,leaf.axis.w));
    let compound_rachis=pinnate_compound && (axis_hit(p,0.0,0.80,max(px,leaf.axis.w)) || line_hit(p,terminal_origin,rachis_end,max(px,leaf.axis.w)));
    let midrib=simple_midrib || compound_rachis;
    let insertion=axis_point(clamp(leaf.planar.z,0.0,0.55));
    let petiole=line_hit(p,insertion,insertion+vec2f(0.0,leaf.axis.y+2.0*leaf.profile.x*leaf.planar.z),max(px,leaf.axis.z));
    let petiole_bridge=axis_hit(p,clamp(leaf.planar.z,0.0,0.55),clamp(leaf.planar.z+0.08,0.0,0.63),max(px*1.5,leaf.axis.z));
    let attachment_hub=length(p-axis_point(0.0))<max(px*4.0,leaf.axis.z*2.2);
    let insertion_hub=leaf.planar.z>0.0 && length(p-insertion)<max(px*1.8,leaf.axis.z*1.8);
    let radial=leaf.topology.x>0.01 && radial_vein_hit(p) && (inside || leaf.topology.y>0.01);
    let radial_base=axis_point(clamp(leaf.organ_style.x+leaf.planar.z,0.0,0.55));
    let radial_attachment=leaf.topology.x>0.01 && axis_hit(p,0.0,clamp(leaf.organ_style.x+leaf.planar.z,0.0,0.55),max(px,leaf.axis.w));
    let landmark_suppression=clamp(leaf.landmarks.y*3.0,0.0,1.0);
    let secondary=leaf.topology.y<0.01 && leaf.hierarchy.y<=0.0 && leaf.topology.x<0.98 && landmark_suppression<0.5 && secondary_hit(p); let parallel=parallel_hit(p);
    let pinnate_local=pinnate_compound && pinnate_organ_vein_hit(p);
    let compound_secondary=field>=px*1.5 && compound_local_secondary_hit(p);
    let basal_primary=basal_primary_hit(p);
    let bipinnate=bipinnate_vein_hit(p);
    if petiole || petiole_bridge || attachment_hub || insertion_hub || radial_attachment || midrib || radial || bipinnate || pinnate_local || (inside && (basal_primary || secondary || parallel || compound_secondary)) { return leaf.vein_color; }
    if inside { return leaf.blade_color; }
    return vec4f(0.0);
}