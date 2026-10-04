use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ActiveInterval {
    from: Option<CalendarYear>,
    to: Option<CalendarYear>,
}

impl ActiveInterval {
    pub(super) fn parse(
        path: &Path,
        from_field: &'static str,
        from: &str,
        to_field: &'static str,
        to: &str,
    ) -> Result<Self> {
        let interval = Self {
            from: optional_calendar_year(path, from_field, from)?,
            to: optional_calendar_year(path, to_field, to)?,
        };
        if interval
            .from
            .zip(interval.to)
            .is_some_and(|(from, to)| from > to)
        {
            return Err(Error::InvalidField {
                path: path.into(),
                field: from_field,
                value: format!("{:?}..{:?}", interval.from, interval.to),
                message: format!("{from_field} must not be later than {to_field}"),
            });
        }
        Ok(interval)
    }

    pub(super) fn contains(self, year: CalendarYear) -> bool {
        self.from.is_none_or(|from| from <= year) && self.to.is_none_or(|to| year < to)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DatedFeature {
    Absent,
    ContradictoryEmptyRange {
        year: CalendarYear,
    },
    Present {
        from: Option<CalendarYear>,
        to: Option<CalendarYear>,
    },
}

impl DatedFeature {
    pub(super) fn parse(
        path: &Path,
        field: &'static str,
        present: SourceFlag,
        from: &str,
        to: &str,
    ) -> Result<Self> {
        let from_year = optional_calendar_year(path, field, from)?;
        let to_year = optional_calendar_year(path, field, to)?;
        if !present.is_set() {
            if from_year.is_some() || to_year.is_some() {
                return Err(Error::InvalidField {
                    path: path.into(),
                    field,
                    value: format!("{from:?}..{to:?}"),
                    message: "feature dates are present while its source flag is unset".into(),
                });
            }
            return Ok(Self::Absent);
        }
        if let Some((from, to)) = from_year.zip(to_year)
            && from == to
        {
            return Ok(Self::ContradictoryEmptyRange { year: from });
        }
        if from_year.zip(to_year).is_some_and(|(from, to)| from > to) {
            return Err(Error::InvalidField {
                path: path.into(),
                field,
                value: format!("{from:?}..{to:?}"),
                message: "feature end year must be later than its start year".into(),
            });
        }
        Ok(Self::Present {
            from: from_year,
            to: to_year,
        })
    }

    pub(super) fn active_in(self, year: CalendarYear) -> bool {
        match self {
            Self::Absent => false,
            Self::ContradictoryEmptyRange { .. } => false,
            Self::Present { from, to } => ActiveInterval { from, to }.contains(year),
        }
    }

    pub(super) const fn is_contradictory(self) -> bool {
        matches!(self, Self::ContradictoryEmptyRange { .. })
    }
}

pub(super) fn optional_calendar_year(
    path: &Path,
    field: &'static str,
    value: &str,
) -> Result<Option<CalendarYear>> {
    optional_number::<i32>(path, field, value)?
        .map(|raw| {
            CalendarYear::try_from(raw).map_err(|message| Error::InvalidField {
                path: path.into(),
                field,
                value: value.into(),
                message: message.to_string(),
            })
        })
        .transpose()
}
