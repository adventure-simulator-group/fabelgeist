//! Evaluate backing on the actual angle-weighted inner cloth surface.
use crate::cloth_patch_plan::ClothPatchPlan;
use crate::device_cloth_ribbon::ClothFitLayout;
use crate::device_frames::DeviceWearer;
use crate::device_garment_kernel::{Grid, Word, atomic, dispatch, read, read_u32, write};
use anyhow::Result;
use fabelgeist_compute::{KernelBatch, VertexNormals};
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};

impl DeviceWearer<'_> {
    pub(crate) fn record_cloth_patch(
        &self,
        batch: &mut KernelBatch,
        fit: &Buffer,
        patch: &ClothPatchPlan,
        layout: ClothFitLayout,
        status: &Buffer,
    ) -> Result<()> {
        let points = self
            .gpu
            .scratch(u64::from(patch.carrier_points) * 12, "backed cloth surface")?;
        let triangles = self
            .gpu
            .upload(BufferUpload::from_elements(&patch.carrier_indices))?;
        let table = self.gpu.upload(BufferUpload::from_elements(&patch.table))?;
        let source = format!(
            "const FIRST_POINT:u32={}u;\nconst POINT_WORDS:u32={}u;\n{}",
            layout.first_point,
            layout.point_words,
            include_str!("device_cloth_patch_points.wgsl")
        );
        dispatch(
            self,
            batch,
            &source,
            &[read("fit", fit), write("cloth_points", &points)],
            &[Word::U("count", patch.carrier_points)],
            Grid::Items(patch.carrier_points),
        )?;
        let triangle_count = patch.carrier_indices.len() as u32 / 3;
        let mut normals =
            VertexNormals::new(self.gpu.context(), patch.carrier_points, triangle_count)?;
        self.gpu.normals(patch.weighting).record(
            batch,
            &points,
            &triangles,
            patch.carrier_points,
            triangle_count,
            &mut normals,
        )?;
        let source = format!(
            "const FIRST_POINT:u32={}u;\nconst POINT_WORDS:u32={}u;\nconst SOURCE_WORDS:u32={}u;\n{}",
            layout.first_point,
            layout.point_words,
            crate::surface_cut::POINT_WORDS,
            include_str!("device_cloth_patch.wgsl")
        );
        dispatch(
            self,
            batch,
            &source,
            &[
                write("fit", fit),
                read("cloth_points", &points),
                read("cloth_normals", &normals.normals),
                read_u32("table", &table),
                read_u32("normal_status", &normals.status),
                atomic("cloth_status", status),
            ],
            &[
                Word::U("first", patch.first),
                Word::U("count", patch.point_count()),
                Word::F("support", patch.support.metres()),
            ],
            Grid::Items(patch.point_count()),
        )
    }
}
