// Vertical surface seating chooses the nearest upward-facing shoulder/chest
// crossing to the authored seam. Undersides of the jaw are excluded. This is
// an unposed construction query, not collision-free posed cloth simulation.
fn cloth_body_height(point:vec3<f32>, near_height:f32)->f32 {
    var best=INFINITY;
    var height=near_height;
    for(var triangle=0u;triangle<params.face_count;triangle+=1u) {
        let a=positions_at(faces[triangle*3u]);
        let b=positions_at(faces[triangle*3u+1u]);
        let c=positions_at(faces[triangle*3u+2u]);
        let ab=b-a;let ac=c-a;let delta=point-a;
        let determinant=ab.x*ac.z-ab.z*ac.x;
        if (-determinant<=FLT_EPSILON*FLT_EPSILON) {continue;}
        let u=(delta.x*ac.z-delta.z*ac.x)/determinant;
        let v=(ab.x*delta.z-ab.z*delta.x)/determinant;
        if (u<0.0||v<0.0||u+v>1.0) {continue;}
        let candidate=a.y+u*ab.y+v*ac.y;
        let distance=abs(candidate-near_height);
        // Torso construction region: exclude feet and the skull. A missing
        // torso crossing is a fit failure, not permission to move onto the head.
        if (candidate<joints[params.seat_bottom*8u+1u]||candidate>joints[params.seat_top*8u+1u]) {continue;}
        if (distance<best) {best=distance;height=candidate;}
    }
    if (best==INFINITY) {atomicStore(&collar_status[0],1u);}
    return height;
}
