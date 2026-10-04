//! Seat the trim rim against its actual facets without reshaping flute valleys.
use super::{
    kernels::{Params, dispatch},
    plates::Plate,
};
use crate::{GenerateError, gpu::ArmorGpu};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::Buffer;

impl Plate {
    pub(super) fn record_miter(
        &self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
        directions: &Buffer,
        status: &Buffer,
    ) -> Result<Buffer, GenerateError> {
        let faces = &self.topology.faces;
        let mut rim = vec![false; self.count() as usize];
        for &i in &self.rim_vertices {
            rim[i as usize] = true;
        }
        let mut incident = vec![Vec::new(); self.count() as usize];
        for (id, face) in faces.iter().enumerate() {
            for &vertex in face {
                if rim[vertex as usize] {
                    incident[vertex as usize].push(id as u32);
                }
            }
        }
        let mut ids = Vec::new();
        let ranges = incident
            .iter()
            .map(|faces| {
                let range = [ids.len() as u32, faces.len() as u32];
                ids.extend_from_slice(faces);
                range
            })
            .collect::<Vec<_>>();
        let normals = gpu.scratch(faces.len() as u64 * 12, "clipped facet normals")?;
        dispatch(
            gpu,
            batch,
            include_str!("miter_normals.wgsl"),
            &[("positions", false), ("normals", true)],
            Params::counted(faces.len() as u32),
            &[
                ("positions", &self.positions),
                ("faces", &gpu.upload(faces)?),
                ("normals", &normals),
                ("status", status),
            ],
            faces.len() as u32,
        )?;
        let offsets = gpu.scratch(self.count() as u64 * 12, "clipped rim offsets")?;
        dispatch(
            gpu,
            batch,
            include_str!("miter_offsets.wgsl"),
            &[("normals", false), ("initial", false), ("offsets", true)],
            Params::counted(self.count()),
            &[
                ("normals", &normals),
                ("initial", directions),
                ("ranges", &gpu.upload(&ranges)?),
                ("incident", &gpu.upload(&ids)?),
                ("offsets", &offsets),
                ("status", status),
            ],
            self.count(),
        )?;
        Ok(offsets)
    }
}
