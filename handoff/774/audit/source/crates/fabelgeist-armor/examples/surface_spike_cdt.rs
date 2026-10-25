use spade::{ConstrainedDelaunayTriangulation, Point2, Triangulation};
use std::{env, fs, path::PathBuf};

fn inside(point: [f64; 2], polygon: &[[f64; 2]]) -> bool {
    let mut result = false;
    for (a, b) in polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
    {
        if (a[1] > point[1]) != (b[1] > point[1])
            && point[0] < (b[0] - a[0]) * (point[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            result = !result;
        }
    }
    result
}

fn main() {
    let root = PathBuf::from(env::args().nth(1).expect("workspace root"));
    let text = fs::read_to_string(root.join("target/surface-first-domain.txt")).unwrap();
    let mut lines = text.lines();
    let boundary_count: usize = lines.next().unwrap().parse().unwrap();
    let positions = lines
        .map(|line| {
            let mut fields = line.split_whitespace().map(|v| v.parse::<f64>().unwrap());
            [fields.next().unwrap(), fields.next().unwrap()]
        })
        .collect::<Vec<_>>();
    let constraints = (0..boundary_count)
        .map(|i| [i, (i + 1) % boundary_count])
        .collect();
    let cdt = ConstrainedDelaunayTriangulation::<Point2<f64>>::bulk_load_cdt(
        positions.iter().map(|p| Point2::new(p[0], p[1])).collect(),
        constraints,
    )
    .unwrap();
    let vertices = cdt
        .vertices()
        .map(|v| [v.position().x, v.position().y])
        .collect::<Vec<_>>();
    let polygon = &vertices[..boundary_count];
    let faces = cdt
        .inner_faces()
        .filter_map(|face| {
            let ids = face.vertices().map(|v| v.fix().index());
            let center = [
                ids.iter().map(|i| vertices[*i][0]).sum::<f64>() / 3.0,
                ids.iter().map(|i| vertices[*i][1]).sum::<f64>() / 3.0,
            ];
            inside(center, polygon).then_some(ids)
        })
        .collect::<Vec<_>>();
    let mut output = format!("{} {}\n", vertices.len(), faces.len());
    for p in vertices {
        output.push_str(&format!("v {:.12} {:.12}\n", p[0], p[1]));
    }
    for f in faces {
        output.push_str(&format!("f {} {} {}\n", f[0], f[1], f[2]));
    }
    fs::write(root.join("target/surface-first-cdt.txt"), output).unwrap();
}
