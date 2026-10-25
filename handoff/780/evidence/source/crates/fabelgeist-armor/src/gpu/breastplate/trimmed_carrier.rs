//! A neckline clips section samples; it does not warp their heights.
use super::topology::{MidTopology, V_SAMPLES};

impl MidTopology {
    /// Construction rows use retained outer-wall vertices at the requested
    /// fraction of each rail. Carrier samples beyond the trim are not probes.
    pub(super) fn construction_grid(&mut self, points: &[[f32; 3]]) {
        let width = self.width();
        let mut used = vec![false; points.len()];
        for &vertex in self.faces.iter().flatten() {
            used[vertex as usize] = true;
        }
        let mut rails = vec![Vec::new(); width];
        for (vertex, &active) in used.iter().enumerate() {
            if !active {
                continue;
            }
            let (column, _, blend) = self.surface_column(vertex).bracket(width as u32).unwrap();
            if blend == 0.0 && points[vertex][1] >= points[column as usize][1] {
                rails[column as usize].push(vertex);
            }
        }
        let mut grid = Vec::with_capacity(self.vertex_count());
        for row in (0..V_SAMPLES).rev() {
            for column in 0..width {
                let bottom = points[column][1];
                let top = rails[column]
                    .iter()
                    .map(|&i| points[i][1])
                    .fold(bottom, f32::max);
                let height = bottom + (top - bottom) * row as f32 / (V_SAMPLES - 1) as f32;
                let vertex = rails[column]
                    .iter()
                    .copied()
                    .min_by(|&a, &b| {
                        (points[a][1] - height)
                            .abs()
                            .total_cmp(&(points[b][1] - height).abs())
                    })
                    .expect("each trimmed carrier rail retains its boundary vertices");
                grid.push(vertex as u32);
            }
        }
        grid.extend(
            (V_SAMPLES..self.rows)
                .flat_map(|row| (0..width).map(move |column| (row * width + column) as u32)),
        );
        self.grid_vertices = Some(grid);
    }
}
