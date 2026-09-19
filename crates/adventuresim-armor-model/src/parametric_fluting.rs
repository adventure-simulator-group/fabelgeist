//! Runtime normal-map charts retained through parametric fitting and assembly.
use super::{FlutingChart, FlutingPattern, PartMesh, ShellLayout, dot, subtract};
use crate::GenerateError;

impl PartMesh {
    pub(crate) fn with_fluting_chart(
        mut self,
        pattern: Option<&crate::PlateFluting>,
        coordinates: Vec<[f32; 2]>,
    ) -> Result<Self, GenerateError> {
        let shell = self
            .shells
            .last_mut()
            .ok_or(GenerateError::InvalidSurface)?;
        if coordinates.len() != shell.vertex_count {
            return Err(GenerateError::InvalidSurface);
        }
        shell.fluting = pattern.map(|pattern| FlutingChart {
            pattern: FlutingPattern::Plate(*pattern),
            coordinates,
        });
        Ok(self)
    }

    pub(crate) fn with_radial_fluting_chart(
        mut self,
        pattern: Option<crate::RadialFluting>,
        radius: crate::Millimeters,
        coordinates: Vec<[f32; 2]>,
    ) -> Result<Self, GenerateError> {
        let shell = self
            .shells
            .last_mut()
            .ok_or(GenerateError::InvalidSurface)?;
        if coordinates.len() != shell.vertex_count {
            return Err(GenerateError::InvalidSurface);
        }
        shell.fluting = pattern.map(|pattern| FlutingChart {
            pattern: FlutingPattern::Radial { pattern, radius },
            coordinates,
        });
        Ok(self)
    }

    /// Build one normal atlas for every distinct runtime flute pattern and
    /// replace body-atlas UVs only where that atlas applies.
    pub fn runtime_fluting(
        &self,
        fallback_texcoords: &[[f32; 2]],
    ) -> Result<Option<super::RuntimeFluting>, GenerateError> {
        if fallback_texcoords.len() != self.positions.len() {
            return Err(GenerateError::InvalidSurface);
        }
        let mut pattern_extents = Vec::<(FlutingPattern, Vec<[f32; 2]>)>::new();
        let mut shell_tiles = Vec::with_capacity(self.shells.len());
        for shell in &self.shells {
            let Some(chart) = &shell.fluting else {
                shell_tiles.push(None);
                continue;
            };
            let extent = match chart.pattern {
                FlutingPattern::Plate(_) => self.chart_extents(shell, chart),
                FlutingPattern::Radial { radius, .. } => [radius.metres(), radius.metres()],
            };
            let index = pattern_extents
                .iter()
                .position(|(pattern, _)| *pattern == chart.pattern)
                .unwrap_or_else(|| {
                    pattern_extents.push((chart.pattern, Vec::new()));
                    pattern_extents.len() - 1
                });
            pattern_extents[index].1.push(extent);
            shell_tiles.push(Some(index));
        }
        if pattern_extents.is_empty() {
            return Ok(None);
        }
        let median = |mut values: Vec<f32>, fallback| {
            values.retain(|value| value.is_finite() && *value > 0.0);
            values.sort_by(f32::total_cmp);
            values.get(values.len() / 2).copied().unwrap_or(fallback)
        };
        let tiles = pattern_extents
            .into_iter()
            .map(|(pattern, extents)| match pattern {
                FlutingPattern::Plate(pattern) => crate::fluting_texture::FlutingTile::new(
                    pattern,
                    median(extents.iter().map(|extent| extent[0]).collect(), 0.3),
                    median(extents.iter().map(|extent| extent[1]).collect(), 0.4),
                ),
                FlutingPattern::Radial { pattern, radius } => {
                    crate::fluting_texture::FlutingTile::radial(pattern, radius.metres())
                }
            })
            .collect::<Vec<_>>();
        let mut samples = vec![(0, [0.0, 0.0]); self.positions.len()];
        for (shell, tile) in self.shells.iter().zip(shell_tiles) {
            let Some(chart) = &shell.fluting else {
                continue;
            };
            for (target, coordinate) in samples
                [shell.first_vertex..shell.first_vertex + shell.vertex_count]
                .iter_mut()
                .zip(&chart.coordinates)
            {
                *target = (tile.expect("fluted shell has an atlas tile"), *coordinate);
            }
        }
        let (normal_map, texcoords) = crate::fluting_texture::atlas(&tiles, &samples);
        Ok(Some((normal_map, texcoords)))
    }

    fn chart_extents(&self, shell: &ShellLayout, chart: &FlutingChart) -> [f32; 2] {
        let mut widths = Vec::new();
        let mut heights = Vec::new();
        for triangle in self.indices[shell.first_index..shell.first_index + shell.index_count]
            .as_chunks::<3>()
            .0
        {
            let local = triangle.map(|index| index as usize - shell.first_vertex);
            let [a, b, c] = local.map(|index| self.positions[shell.first_vertex + index]);
            let [uv_a, uv_b, uv_c] = local.map(|index| chart.coordinates[index]);
            let edge_ab = subtract(b, a);
            let edge_ac = subtract(c, a);
            let duv_ab = [uv_b[0] - uv_a[0], uv_b[1] - uv_a[1]];
            let duv_ac = [uv_c[0] - uv_a[0], uv_c[1] - uv_a[1]];
            let determinant = duv_ab[0] * duv_ac[1] - duv_ab[1] * duv_ac[0];
            if determinant.abs() <= 1e-6 {
                continue;
            }
            let inverse = determinant.recip();
            let tangent = std::array::from_fn(|axis| {
                (edge_ab[axis] * duv_ac[1] - edge_ac[axis] * duv_ab[1]) * inverse
            });
            let bitangent = std::array::from_fn(|axis| {
                (edge_ac[axis] * duv_ab[0] - edge_ab[axis] * duv_ac[0]) * inverse
            });
            widths.push(dot(tangent, tangent).sqrt());
            heights.push(dot(bitangent, bitangent).sqrt());
        }
        [median(widths, 0.3), median(heights, 0.4)]
    }
}

fn median(mut values: Vec<f32>, fallback: f32) -> f32 {
    values.retain(|value| value.is_finite() && *value > 0.0);
    values.sort_by(f32::total_cmp);
    values.get(values.len() / 2).copied().unwrap_or(fallback)
}
