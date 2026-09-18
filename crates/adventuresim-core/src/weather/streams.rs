//! Named random purposes owned by this generator. Names are part of its replay contract.
use fabelgeist_determinism::StreamId;

pub(super) const CONVECTION: StreamId = StreamId::new("weather.convection");
pub(super) const MIDDLE_MOISTURE: StreamId = StreamId::new("weather.middle-moisture");
pub(super) const MOISTURE: StreamId = StreamId::new("weather.moisture");
pub(super) const PRECIPITATION: StreamId = StreamId::new("weather.precipitation");
pub(super) const PRESSURE: StreamId = StreamId::new("weather.pressure");
pub(super) const SHEAR: StreamId = StreamId::new("weather.shear");
pub(super) const TEMPERATURE: StreamId = StreamId::new("weather.temperature");
pub(super) const UPPER_MOISTURE: StreamId = StreamId::new("weather.upper-moisture");
