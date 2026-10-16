// Horizontal ray/triangle crossings enclose the actual body at each final row.
// Kept separate from the style taper: fitted support cannot shrink the garment.
fn cloth_body_support(origin:vec3<f32>, direction:vec3<f32>)->f32 {
    var radius=0.0;
    for(var triangle=0u;triangle<params.face_count;triangle+=1u) {
        let a=positions_at(faces[triangle*3u]);
        let b=positions_at(faces[triangle*3u+1u]);
        let c=positions_at(faces[triangle*3u+2u]);
        let edge1=b-a;let edge2=c-a;
        let cross=host_cross(direction,edge2);
        let determinant=host_dot(edge1,cross);
        if (abs(determinant)<FLT_EPSILON*FLT_EPSILON) {continue;}
        let delta=origin-a;
        let u=host_dot(delta,cross)/determinant;
        let q=host_cross(delta,edge1);
        let v=host_dot(direction,q)/determinant;
        if (u<0.0||v<0.0||u+v>1.0) {continue;}
        radius=max(radius,host_dot(edge2,q)/determinant);
    }
    return radius;
}
