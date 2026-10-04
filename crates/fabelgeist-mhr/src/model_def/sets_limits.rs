//! Named subsets and the supported minmax limit grammar.
use super::lexical::{DefinitionLines, Directive, TokenSeparator};
use super::terms::ExpressionWeight;
use super::{
    DefinitionNumberRole, ModelDefinitionError, ParameterBounds, ParameterBoundsError,
    ParameterLimit, ParameterTransform, SolverLimitWeight,
};
impl ParameterTransform {
    pub(super) fn read_sets(&mut self, lines: &DefinitionLines) {
        for line in &lines.0 {
            let tokens = line.token().tokens(TokenSeparator::Words);
            if tokens.0.len() < 2 || !matches!(tokens.0[0].directive(), Directive::ParameterSet) {
                continue;
            }
            let mut set = vec![false; usize::from(self.num_parameters())];
            for name in &tokens.0[2..] {
                if let Some(index) = self.parameter_index(&name.parameter_name()) {
                    set[index.0] = true;
                }
            }
            self.parameter_sets.insert(tokens.0[1].set_name(), set);
        }
    }
    pub(super) fn read_limits(
        &mut self,
        lines: &DefinitionLines,
    ) -> Result<(), ModelDefinitionError> {
        for line in &lines.0 {
            let tokens = line.token().tokens(TokenSeparator::Words);
            // Other Momentum limit kinds remain outside the MHR grammar subset.
            if tokens.0.len() < 3
                || !matches!(tokens.0[0].directive(), Directive::Limit)
                || !matches!(tokens.0[2].directive(), Directive::MinMax)
            {
                continue;
            }
            let Some(parameter) = self.parameter_index(&tokens.0[1].parameter_name()) else {
                continue;
            };
            let Some((bounds, weight)) = line.bounds() else {
                return Err(ModelDefinitionError::LimitSyntax(line.clone()));
            };
            let bounds = bounds.tokens(TokenSeparator::Bounds);
            let [minimum, maximum] = bounds.0.as_slice() else {
                return Err(ModelDefinitionError::LimitSyntax(line.clone()));
            };
            let min = ExpressionWeight::parse(*minimum, line, DefinitionNumberRole::Minimum)?.0;
            let max = ExpressionWeight::parse(*maximum, line, DefinitionNumberRole::Maximum)?.0;
            let bounds = ParameterBounds::try_from((min, max)).map_err(
                |source: ParameterBoundsError| -> ModelDefinitionError {
                    ModelDefinitionError::LimitBounds {
                        line: line.clone(),
                        source,
                    }
                },
            )?;
            let weight = if weight.0.is_empty() {
                1.0
            } else {
                ExpressionWeight::parse(weight, line, DefinitionNumberRole::SolverWeight)?.0
            };
            self.limits.push(ParameterLimit {
                parameter,
                bounds,
                weight: SolverLimitWeight::from(weight),
            });
        }
        Ok(())
    }
}
