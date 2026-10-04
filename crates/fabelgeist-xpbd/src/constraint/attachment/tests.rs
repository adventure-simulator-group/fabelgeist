use super::*;
use std::error::Error;

#[tokio::test]
async fn record_admission_retains_context_and_uploads_empty_and_wide_records() {
    let context = WgpuContext::new().await.unwrap();
    let name = ConstraintName::from("test set");
    let error = ConstraintAttachment::from_records("weights".into(), &[[1.0f32; 8]])
        .upload(&context, &name, ConstraintCount::from(2))
        .unwrap_err();
    assert!(
        matches!(&error, ConstraintAttachmentError::RecordCount {set,parameter,actual,expected}
        if *set == name && *parameter == PassParameterName::from("weights")
            && *actual == ConstraintCount::from(1) && *expected == ConstraintCount::from(2))
    );
    assert!(error.source().is_none());
    assert_eq!(
        error.to_string(),
        "ConstraintSet `test set`: attachment `weights` has 1 values for 2 constraints"
    );
    let empty: [[f32; 8]; 0] = [];
    let (parameter, buffer) = ConstraintAttachment::from_records("weights".into(), &empty)
        .upload(&context, &name, ConstraintCount::from(0))
        .unwrap();
    assert_eq!(parameter, PassParameterName::from("weights"));
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), [0]);
    let words = [[-0.0f32, 1.0, 2.0, 3.0, 4.0, 0.0, 0.0, 0.0], [5.0; 8]];
    let (_, buffer) = ConstraintAttachment::from_records("weights".into(), &words)
        .upload(&context, &name, ConstraintCount::from(2))
        .unwrap();
    assert_eq!(
        buffer.read::<u32>(&context).await.unwrap(),
        words
            .as_flattened()
            .iter()
            .copied()
            .map(f32::to_bits)
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn replacing_attachment_moves_it_to_the_end_and_keeps_parameter_identity() {
    let context = WgpuContext::new().await.unwrap();
    let name = ConstraintName::from("set");
    let mut attachments = ConstraintAttachments::default();
    for parameter in ["weights", "rest_lengths", "weights"] {
        let (parameter, buffer) = ConstraintAttachment::from_records(parameter.into(), &[7u32])
            .upload(&context, &name, ConstraintCount::from(1))
            .unwrap();
        attachments.insert(parameter, buffer);
    }
    let mut parameters = PassParameters::new();
    attachments.bind(&mut parameters);
    let keys: Vec<_> = parameters.iter().map(|(name, _): (&PassParameterName, &fabelgeist_gpu::prelude::PassParameter)| -> PassParameterName { name.clone() }).collect();
    assert_eq!(
        keys,
        [
            PassParameterName::from("rest_lengths"),
            PassParameterName::from("weights")
        ]
    );
    assert_eq!(
        attachments
            .get(&PassParameterName::from("weights"))
            .unwrap()
            .read::<u32>(&context)
            .await
            .unwrap(),
        [7]
    );
}

#[tokio::test]
async fn nonempty_zero_sized_records_keep_allocation_failure_and_concrete_source() {
    let context = WgpuContext::new().await.unwrap();
    let name = ConstraintName::from("zero-sized set");
    let error = ConstraintAttachment::from_records("weights".into(), &[()])
        .upload(&context, &name, ConstraintCount::from(1))
        .unwrap_err();
    assert!(
        matches!(&error, ConstraintAttachmentError::Allocation {set,parameter,source}
        if *set == name && *parameter == PassParameterName::from("weights")
            && *source == BufferCreationError::Empty)
    );
    assert!(error.source().unwrap().is::<BufferCreationError>());
    assert_eq!(error.to_string(), "Buffer size must be greater than 0");
}
