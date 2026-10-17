// A two-harmonic seat follows neck curvature without copying body facets.
alias SeatEquation=array<f32,SEAT_TERMS+1u>;
fn seat_basis(angle:f32)->array<f32,SEAT_TERMS> {
    return array<f32,SEAT_TERMS>(1.0,cos(angle),sin(angle),cos(2.0*angle),sin(2.0*angle));
}
@compute @workgroup_size(1)
fn main() {
    var equations:array<SeatEquation,SEAT_TERMS>;
    for(var column=0u;column<params.columns;column+=1u) {
        let angle=bitcast<f32>(samples[column*params.sample_words+3u]);
        var basis=seat_basis(angle);
        for(var row=0u;row<SEAT_TERMS;row+=1u) {
            for(var term=0u;term<SEAT_TERMS;term+=1u) {
                equations[row][term]+=basis[row]*basis[term];
            }
            equations[row][SEAT_TERMS]+=basis[row]*seating[column];
        }
    }
    // Partial-pivot elimination of this fixed five-coefficient fit.
    for(var pivot=0u;pivot<SEAT_TERMS;pivot+=1u) {
        var best=pivot;
        for(var row=pivot+1u;row<SEAT_TERMS;row+=1u) {
            if (abs(equations[row][pivot])>abs(equations[best][pivot])) {best=row;}
        }
        if (abs(equations[best][pivot])<FLT_EPSILON) {fit[FIT_FAILED]=1.0;return;}
        let swapped=equations[pivot];equations[pivot]=equations[best];equations[best]=swapped;
        let divisor=equations[pivot][pivot];
        for(var term=pivot;term<=SEAT_TERMS;term+=1u) {equations[pivot][term]/=divisor;}
        for(var row=0u;row<SEAT_TERMS;row+=1u) {
            if (row==pivot) {continue;}
            let factor=equations[row][pivot];
            for(var term=pivot;term<=SEAT_TERMS;term+=1u) {equations[row][term]-=factor*equations[pivot][term];}
        }
    }
    var lift=0.0;
    for(var column=0u;column<params.columns;column+=1u) {
        let angle=bitcast<f32>(samples[column*params.sample_words+3u]);
        var basis=seat_basis(angle);var height=0.0;
        for(var term=0u;term<SEAT_TERMS;term+=1u) {height+=basis[term]*equations[term][SEAT_TERMS];}
        lift=max(lift,seating[column]-height);
    }
    for(var term=0u;term<SEAT_TERMS;term+=1u) {
        seating[params.columns+term]=equations[term][SEAT_TERMS]+select(0.0,lift,term==0u);
    }
}
