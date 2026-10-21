//! Native component measurements; these do not establish visible arrival.
use std::time::Duration;

#[derive(Default)]
pub(super) struct CacheAdmissionMeasurements {
    pub(super) encoded_file_read: Duration,
    pub(super) container_guards: Duration,
    pub(super) decompression: Duration,
    pub(super) decoded_digest: Duration,
    pub(super) payload_decode: Duration,
    pub(super) payload_validation: Duration,
}

impl CacheAdmissionMeasurements {
    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({
            "encoded_file_read": self.encoded_file_read.as_micros(),
            "container_guards": self.container_guards.as_micros(),
            "decompression": self.decompression.as_micros(),
            "decoded_digest": self.decoded_digest.as_micros(),
            "payload_decode": self.payload_decode.as_micros(),
            "payload_validation": self.payload_validation.as_micros(),
            "unit": "microseconds",
            "scope": "nonoverlapping CPU subphases; excludes decoder cleanup and outer process/scene setup; no GPU/arrival claim",
        })
    }
}
