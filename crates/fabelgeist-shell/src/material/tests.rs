use super::*;

fn material() -> ShellMaterial {
    ShellMaterial {
        stretch_compliance: 0.0,
        bend_compliance: 0.0,
        seam_compliance: 0.0,
        thickness: 0.003,
        friction: 0.0,
        damping: 0.0,
    }
}

#[test]
fn invalid_values_retain_their_material_parameter() {
    use ShellMaterialParameter::*;
    for parameter in [
        StretchCompliance,
        BendCompliance,
        SeamCompliance,
        Thickness,
        Friction,
        Damping,
    ] {
        for invalid in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut sample = material();
            match parameter {
                StretchCompliance => sample.stretch_compliance = invalid,
                BendCompliance => sample.bend_compliance = invalid,
                SeamCompliance => sample.seam_compliance = invalid,
                Thickness => sample.thickness = invalid,
                Friction => sample.friction = invalid,
                Damping => sample.damping = invalid,
            }
            let error = sample.validate().unwrap_err();
            assert_eq!(error, ShellMaterialError::InvalidParameter { parameter });
            assert_eq!(error.to_string(), "invalid shell material");
        }
    }
}

#[test]
fn all_parameter_admission_precedes_positive_thickness() {
    let mut sample = material();
    sample.thickness = 0.0;
    sample.damping = -1.0;
    assert_eq!(
        sample.validate(),
        Err(ShellMaterialError::InvalidParameter {
            parameter: ShellMaterialParameter::Damping
        })
    );
    sample.stretch_compliance = -1.0;
    assert_eq!(
        sample.validate(),
        Err(ShellMaterialError::InvalidParameter {
            parameter: ShellMaterialParameter::StretchCompliance
        })
    );
    sample.stretch_compliance = 0.0;
    sample.damping = 0.0;
    assert_eq!(
        sample.validate(),
        Err(ShellMaterialError::NonPositiveThickness)
    );
    assert_eq!(
        sample.validate().unwrap_err().to_string(),
        "shell thickness must be positive"
    );
}

#[test]
fn zero_and_negative_zero_are_admitted_except_for_thickness() {
    let mut sample = material();
    sample.stretch_compliance = -0.0;
    sample.bend_compliance = -0.0;
    sample.seam_compliance = -0.0;
    sample.friction = -0.0;
    sample.damping = -0.0;
    assert!(sample.validate().is_ok());
    sample.thickness = -0.0;
    assert_eq!(
        sample.validate(),
        Err(ShellMaterialError::NonPositiveThickness)
    );
}
