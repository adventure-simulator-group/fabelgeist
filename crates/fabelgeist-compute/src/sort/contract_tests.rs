use super::*;
use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};
use serde::Deserialize;

// Native fixtures pin the original sorter's serialized words and metadata.
#[derive(Deserialize)]
struct NativeFixture {
    cases: Vec<NativeCase>,
    layout: Vec<NativeLayout>,
    widths: Vec<NativeWidth>,
    shaders: Vec<String>,
}
#[derive(Deserialize)]
struct NativeCase {
    count: u32,
    bits: u32,
    input: Vec<u32>,
    keys: Vec<u32>,
    values: Vec<u32>,
}
#[derive(Deserialize)]
struct NativeLayout {
    count: u32,
    bytes: u64,
    tiles: u32,
    capacity: u32,
    histogram_bytes: u64,
}
#[derive(Deserialize)]
struct NativeWidth {
    bits: u32,
    passes: u32,
}
#[derive(Deserialize)]
struct NativeAdmission {
    growth: Vec<NativeGrowth>,
    scratch_error: String,
    byte_error: String,
    dispatches: usize,
}
#[derive(Deserialize)]
struct NativeGrowth {
    requested: u32,
    grew: bool,
    capacity: u32,
}

#[test]
fn preserves_native_layout_width_and_shader_fixtures() {
    let fixture: NativeFixture = serde_json::from_str(include_str!("native-cases.json")).unwrap();
    for row in fixture.layout {
        let count = SortItemCount::from(row.count);
        assert_eq!(u64::from(count.word_bytes()), row.bytes);
        assert_eq!(<[u32; 3]>::from(count.tiles()), [row.tiles, 1, 1]);
        let capacity = count.with_sentinel();
        assert_eq!(capacity, row.capacity.into());
        assert_eq!(
            u64::from(capacity.tiles().histogram_bytes()),
            row.histogram_bytes
        );
    }
    for row in fixture.widths {
        let width = SortKeyWidth::from(row.bits);
        assert_eq!(width.pass_count().to_string(), row.passes.to_string());
    }
    assert_eq!(
        vec![histogram_code(), scan_code(), scatter_code()],
        fixture.shaders
    );
}

#[tokio::test]
async fn preserves_original_gpu_sort_outputs() -> Result<()> {
    let fixture: NativeFixture = serde_json::from_str(include_str!("native-cases.json"))?;
    let context = WgpuContext::new().await?;
    let sort = RadixSort::new(&context)?;
    for row in fixture.cases {
        let values: Vec<u32> = (0..row.count.max(1)).collect();
        let definition = BufferDefinition::storage().with_usage(BufferUse::CopySource);
        let keys = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&row.input),
            definition.clone(),
        )?;
        let payload =
            Buffer::from_upload(&context, BufferUpload::from_elements(&values), definition)?;
        let mut scratch = SortScratch::new(&context, row.count.into())?;
        sort.run(
            &context,
            &keys,
            &payload,
            &mut scratch,
            row.count.into(),
            row.bits.into(),
        )?;
        assert_eq!(
            keys.read::<u32>(&context).await?,
            row.keys,
            "count {}, width {}",
            row.count,
            row.bits
        );
        assert_eq!(
            payload.read::<u32>(&context).await?,
            row.values,
            "count {}, width {}",
            row.count,
            row.bits
        );
    }
    Ok(())
}

#[tokio::test]
async fn preserves_scratch_growth_admission_order_and_noops() -> Result<()> {
    let fixture: NativeAdmission = serde_json::from_str(include_str!("native-admission.json"))?;
    let context = WgpuContext::new().await?;
    let sort = RadixSort::new(&context)?;
    let tiny = Buffer::from_upload(
        &context,
        BufferUpload::from_elements(&[0u32]),
        BufferDefinition::storage().with_usage(BufferUse::CopySource),
    )?;
    let mut scratch = SortScratch::new(&context, 0.into())?;
    for row in fixture.growth {
        let growth = scratch.ensure(&context, row.requested.into())?;
        let expected = if row.grew {
            ScratchGrowth::Grown
        } else {
            ScratchGrowth::Unchanged
        };
        assert_eq!(growth, expected);
        assert_eq!(scratch.capacity(), row.capacity.into());
    }
    let mut small = SortScratch::new(&context, 2.into())?;
    let mut batch = KernelBatch::new(&context);
    let error = sort
        .record(&mut batch, &tiny, &tiny, &mut small, 4.into(), 32.into())
        .unwrap_err();
    assert_eq!(error.to_string(), fixture.scratch_error);
    assert_eq!(small.capacity(), 2.into());
    let error = sort
        .record(&mut batch, &tiny, &tiny, &mut scratch, 4.into(), 32.into())
        .unwrap_err();
    assert_eq!(error.to_string(), fixture.byte_error);
    for count in [0u32, 1] {
        sort.record(
            &mut batch,
            &tiny,
            &tiny,
            &mut small,
            count.into(),
            32.into(),
        )?;
    }
    assert_eq!(batch.dispatch_count(), fixture.dispatches);
    Ok(())
}
