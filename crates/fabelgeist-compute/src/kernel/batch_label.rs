//! Borrowed diagnostic spelling for a compute batch's command encoder.

/// A batch label, admitted exactly as authored or supplied by a native caller.
///
/// Every string is admitted, including empty strings and control characters.
/// This bespoke type borrows without allocating or normalizing. It identifies
/// an encoder in native diagnostics; it does not select a kernel or cache entry.
/// Its borrow lasts through encoder creation, independently of the returned
/// batch's context borrow.
///
/// Raw strings require explicit admission at a literal or native boundary:
///
/// ```compile_fail
/// use fabelgeist_compute::KernelBatch;
/// use fabelgeist_gpu::prelude::WgpuContext;
/// fn raw_label(context: &WgpuContext) {
///     let _ = KernelBatch::labelled(context, "unadmitted label");
/// }
/// ```
///
/// Buffer allocation labels belong to a separate resource role:
///
/// ```compile_fail
/// use fabelgeist_compute::KernelBatch;
/// use fabelgeist_gpu::prelude::{BufferLabel, WgpuContext};
/// fn buffer_label(context: &WgpuContext, label: BufferLabel) {
///     let _ = KernelBatch::labelled(context, label);
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct KernelBatchLabel<'label>(&'label str);

impl<'label> From<&'label str> for KernelBatchLabel<'label> {
    fn from(label: &'label str) -> Self {
        Self(label)
    }
}

/// Native command-encoder descriptors require a borrowed string. Keep this
/// representation conversion at that descriptor boundary.
impl<'label> From<KernelBatchLabel<'label>> for &'label str {
    fn from(label: KernelBatchLabel<'label>) -> Self {
        label.0
    }
}
