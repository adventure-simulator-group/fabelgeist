pub use super::super::*;
pub use crate::test_utils::*;

pub async fn test_generalized<IN, OUT, S>(
    definition_code: &str,
    input_data: &[IN],
    expected_output: &[OUT],
) -> Result<()>
where
    IN: bytemuck::NoUninit + bytemuck::AnyBitPattern + PartialEq + std::fmt::Debug + Default + Copy,
    OUT:
        bytemuck::NoUninit + bytemuck::AnyBitPattern + PartialEq + std::fmt::Debug + Default + Copy,
    S: bytemuck::Pod + std::fmt::Debug + Default + Copy + PartialEq,
{
    let definition = MapDefinition::new(definition_code)?;
    let sig = MapSignature::parse(definition_code)?;

    let input_element_type = sig
        .user_parameters()
        .next()
        .map(|parameter| parameter.value_type().clone())
        .unwrap_or(ResourceBaseType::F32); // Default for generator tests without input data

    run_compute_test::<IN, OUT, S, _>(
        input_data,
        expected_output,
        input_element_type,
        sig.output_element_type().clone(),
        |context, in_res, out_res| Map::execute(context, &definition, Some(&in_res), &out_res),
    )
    .await
}

#[macro_export]
macro_rules! test_map {
    ($name:ident, $definition:expr, $input:expr, $output:expr, $s:ty) => {
        #[tokio::test]
        async fn $name() -> Result<()> {
            test_generalized::<_, _, $s>($definition, &$input, &$output).await
        }
    };
}
