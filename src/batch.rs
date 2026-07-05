//! Batch settlement request/response types for the Worldline NAM Batch API.

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use std::{error, fmt};

fn deserialize_flex_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>
{
    struct FlexStringVisitor;

    impl<'de> Visitor<'de> for FlexStringVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or numeric value")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(value.to_string())
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(value)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(value.to_string())
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error
        {
            Ok(value.to_string())
        }
    }

    deserializer.deserialize_any(FlexStringVisitor)
}

pub type BatchResult = Result<BatchSuccessResponse, BatchErrorResult>;

#[derive(Serialize, Debug, Clone)]
pub struct BatchCriteria {
    pub process_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_merchant_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_now: Option<u8>
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BatchSuccessResponse {
    pub code: u64,
    pub message: String,
    #[serde(deserialize_with = "deserialize_flex_string")]
    pub batch_id: String,
    #[serde(deserialize_with = "deserialize_flex_string")]
    pub process_date: String,
    pub process_time_zone: Option<String>,
    pub batch_mode: Option<String>
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct BatchErrorResponse {
    pub code: u64,
    pub category: u64,
    pub message: String,
    pub reference: Option<String>
}

#[derive(thiserror::Error, Debug)]
pub enum BatchErrorResult {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error(transparent)]
    WorldlineError(#[from] BatchErrorResponse)
}

impl BatchErrorResult {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            BatchErrorResult::UnexpectedError(v) => json!({"error": v.to_string()}),
            BatchErrorResult::WorldlineError(v) => json!({"error": v})
        }
    }
}

impl fmt::Display for BatchErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "code: {}, category: {}, message: {}", self.code, self.category, self.message)
    }
}

impl error::Error for BatchErrorResponse {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_batch_success_with_numeric_batch_id() {
        let raw = r#"{
            "code": 1,
            "message": "Batch accepted",
            "batch_id": 10000005,
            "process_date": 20260703
        }"#;

        let parsed: BatchSuccessResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.batch_id, "10000005");
        assert_eq!(parsed.process_date, "20260703");
    }

    #[test]
    fn batch_criteria_serializes_optional_fields() {
        let criteria = BatchCriteria {
            process_date: "20260703".to_string(),
            sub_merchant_id: Some(42),
            process_now: Some(1)
        };

        let value = serde_json::to_value(&criteria).unwrap();
        assert_eq!(value["process_date"], "20260703");
        assert_eq!(value["sub_merchant_id"], 42);
        assert_eq!(value["process_now"], 1);
    }

    #[test]
    fn batch_success_response_roundtrips_string_fields() {
        let raw = r#"{
            "code": 1,
            "message": "Batch accepted",
            "batch_id": "10000005",
            "process_date": "20260703",
            "process_time_zone": "America/Vancouver",
            "batch_mode": "auto"
        }"#;

        let parsed: BatchSuccessResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(parsed.batch_id, "10000005");
        assert_eq!(parsed.process_time_zone.as_deref(), Some("America/Vancouver"));

        let again = serde_json::to_string(&parsed).unwrap();
        let reparsed: BatchSuccessResponse = serde_json::from_str(&again).unwrap();
        assert_eq!(reparsed.batch_id, parsed.batch_id);
        assert_eq!(reparsed.process_date, parsed.process_date);
    }

    #[test]
    fn batch_error_response_roundtrips() {
        let err = BatchErrorResponse {
            code: 400,
            category: 3,
            message: "Invalid date".to_string(),
            reference: Some("ref-1".to_string())
        };

        let value = serde_json::to_value(&err).unwrap();
        let reparsed: BatchErrorResponse = serde_json::from_value(value).unwrap();
        assert_eq!(reparsed.code, err.code);
        assert_eq!(reparsed.message, err.message);
        assert_eq!(reparsed.reference.as_deref(), Some("ref-1"));
    }

    #[test]
    fn batch_error_result_to_json_variants() {
        let worldline = BatchErrorResult::WorldlineError(BatchErrorResponse {
            code: 1,
            category: 2,
            message: "failed".to_string(),
            reference: None
        });
        assert!(worldline.to_json().get("error").is_some());

        let unexpected = BatchErrorResult::UnexpectedError(anyhow::anyhow!("boom"));
        assert_eq!(unexpected.to_json()["error"], "boom");
    }

    #[test]
    fn batch_error_response_display_formats_message() {
        let err = BatchErrorResponse {
            code: 9,
            category: 1,
            message: "oops".to_string(),
            reference: None
        };
        assert_eq!(err.to_string(), "code: 9, category: 1, message: oops");
    }
}
