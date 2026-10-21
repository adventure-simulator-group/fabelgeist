use super::*;

#[test]
fn identity_rows_distinguish_exact_batches_from_single_row_broadcast() {
    for batch in [
        ModelBatchSize::from(0),
        ModelBatchSize::from(1),
        ModelBatchSize::from(3),
    ] {
        assert!(matches!(
            IdentityLayout::from([usize::from(batch), 45]).admit(batch),
            Ok(IdentityRows::Exact)
        ));
    }
    for batch in [ModelBatchSize::from(0), ModelBatchSize::from(3)] {
        assert!(matches!(
            IdentityLayout::from([1, 45]).admit(batch),
            Ok(IdentityRows::Broadcast)
        ));
    }
}

#[test]
fn rejected_identity_width_precedes_rejected_batch_and_keeps_existing_text() {
    let failure = IdentityLayout::from([2, 44])
        .admit(ModelBatchSize::from(3))
        .err()
        .unwrap();
    assert_eq!(
        failure,
        MhrEvaluationError::IdentityColumns(IdentityCoefficientCount::from(44))
    );
    assert_eq!(
        failure.to_string(),
        "identity coefficients have 44 columns, expected 45"
    );
}

#[test]
fn identity_row_errors_keep_actual_and_evaluation_batch() {
    for rows in [ModelBatchSize::from(0), ModelBatchSize::from(2)] {
        let failure = IdentityLayout::from([usize::from(rows), 45])
            .admit(ModelBatchSize::from(3))
            .err()
            .unwrap();
        assert_eq!(
            failure,
            MhrEvaluationError::IdentityRows {
                actual: rows,
                batch: ModelBatchSize::from(3)
            }
        );
        assert_eq!(
            failure.to_string(),
            format!("identity coefficients have {rows} rows, expected 3 or 1")
        );
    }
}

#[test]
fn pose_columns_use_the_loaded_definition_count_including_empty_layouts() {
    for columns in [
        PoseParameterCount::from(0),
        PoseParameterCount::from(17),
        PoseParameterCount::from(204),
    ] {
        assert!(
            PoseLayout::from([0, usize::from(columns)])
                .admit(columns)
                .is_ok()
        );
    }
    let failure = PoseLayout::from([3, 17])
        .admit(PoseParameterCount::from(204))
        .unwrap_err();
    assert_eq!(
        failure,
        MhrEvaluationError::PoseColumns {
            actual: PoseParameterCount::from(17),
            expected: PoseParameterCount::from(204)
        }
    );
    assert_eq!(
        failure.to_string(),
        "model parameters have 17 columns, expected 204"
    );
}

#[test]
fn expression_width_precedes_rows_and_expression_rows_are_never_broadcast() {
    let failure = ExpressionLayout::from([1, 71])
        .admit(ModelBatchSize::from(3))
        .unwrap_err();
    assert_eq!(
        failure,
        MhrEvaluationError::ExpressionColumns(ExpressionCoefficientCount::from(71))
    );
    assert_eq!(
        failure.to_string(),
        "expression coefficients have 71 columns, expected 72"
    );
    let failure = ExpressionLayout::from([1, 72])
        .admit(ModelBatchSize::from(3))
        .unwrap_err();
    assert_eq!(
        failure,
        MhrEvaluationError::ExpressionRows {
            actual: ModelBatchSize::from(1),
            batch: ModelBatchSize::from(3)
        }
    );
    assert_eq!(
        failure.to_string(),
        "expression coefficients have 1 rows, expected 3"
    );
}

#[test]
fn expression_admission_preserves_exact_and_empty_batches() {
    for batch in [
        ModelBatchSize::from(0),
        ModelBatchSize::from(1),
        ModelBatchSize::from(3),
    ] {
        assert!(
            ExpressionLayout::from([usize::from(batch), 72])
                .admit(batch)
                .is_ok()
        );
    }
}
