use super::*;
use crate::Coloring;
use std::error::Error;

#[test]
fn rejects_zero_arity_and_incomplete_records_before_upload() {
    let error = ConstraintArity::try_from(0).unwrap_err();
    assert_eq!(error, ConstraintLayoutError::ZeroArity);
    assert_eq!(
        error.to_string(),
        "ConstraintSet: arity must be at least one"
    );
    assert!(error.source().is_none());
    let error = ConstraintIncidence::from_native_records::<0>(&[[]]).unwrap_err();
    assert_eq!(error, ConstraintLayoutError::ZeroArity);
    let arity = ConstraintArity::try_from(2).unwrap();
    let error = ConstraintIncidence::from_native_flat(&[0, 1, 2], arity).unwrap_err();
    assert_eq!(
        error,
        ConstraintLayoutError::Incomplete {
            particles: ConstraintParticleCount(3),
            arity
        }
    );
    assert_eq!(
        error.to_string(),
        "ConstraintSet: 3 indices is not a whole number of arity-2 constraints"
    );
    assert!(error.source().is_none());
}

#[tokio::test]
async fn uploads_the_original_color_order_and_native_empty_sentinel() {
    let context = WgpuContext::new().await.unwrap();
    let incidence = ConstraintIncidence::from_native_records(&[
        [0u32, 1],
        [1, 2],
        [2, 3],
        [3, 0],
        [u32::MAX, 7],
    ])
    .unwrap();
    let coloring = Coloring::for_incidence(&incidence);
    let buffer = incidence.upload_ordered(&context, &coloring).unwrap();
    assert_eq!(
        buffer.read::<u32>(&context).await.unwrap(),
        [0, 1, 2, 3, u32::MAX, 7, 1, 2, 3, 0]
    );
    let empty = ConstraintIncidence::from_native_records::<4>(&[]).unwrap();
    let buffer = empty
        .upload_ordered(&context, &Coloring::for_incidence(&empty))
        .unwrap();
    assert_eq!(buffer.read::<u32>(&context).await.unwrap(), [0]);
}
