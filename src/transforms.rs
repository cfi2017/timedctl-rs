use chrono::{Duration, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

/// Transform trait for converting between Rust types and API representation
pub trait Transform<T, U> {
    /// Serialize a Rust value to an API representation
    fn serialize(&self, value: T) -> U;

    /// Deserialize an API representation to a Rust value
    fn deserialize(&self, value: U) -> T;
}

/// Date transform for handling API date formats
pub struct DateTransform;

impl Transform<Option<NaiveDate>, Option<String>> for DateTransform {
    fn serialize(&self, value: Option<NaiveDate>) -> Option<String> {
        value.map(|date| date.format("%Y-%m-%d").to_string())
    }

    fn deserialize(&self, value: Option<String>) -> Option<NaiveDate> {
        value.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok())
    }
}

/// Time transform for handling API time formats
pub struct TimeTransform;

impl Transform<Option<NaiveTime>, Option<String>> for TimeTransform {
    fn serialize(&self, value: Option<NaiveTime>) -> Option<String> {
        value.map(|time| time.format("%H:%M:%S").to_string())
    }

    fn deserialize(&self, value: Option<String>) -> Option<NaiveTime> {
        value.and_then(|s| NaiveTime::parse_from_str(&s, "%H:%M:%S").ok())
    }
}

/// Duration transform for handling API duration formats
pub struct DurationTransform;

impl Transform<Option<Duration>, Option<String>> for DurationTransform {
    fn serialize(&self, value: Option<Duration>) -> Option<String> {
        value.map(|duration| {
            let total_seconds = duration.num_seconds();
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;
            format!("{hours:02}:{minutes:02}:00")
        })
    }

    fn deserialize(&self, value: Option<String>) -> Option<Duration> {
        value.and_then(|s| {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() >= 2 {
                if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>())
                {
                    return Some(Duration::hours(hours) + Duration::minutes(minutes));
                }
            }
            None
        })
    }
}

/// Boolean transform for handling API boolean formats
pub struct BooleanTransform;

impl Transform<bool, i32> for BooleanTransform {
    fn serialize(&self, value: bool) -> i32 {
        if value {
            1
        } else {
            0
        }
    }

    fn deserialize(&self, value: i32) -> bool {
        value != 0
    }
}

/// ID transform for handling API ID formats
pub struct IdTransform;

impl Transform<Option<String>, Option<String>> for IdTransform {
    fn serialize(&self, value: Option<String>) -> Option<String> {
        value
    }

    fn deserialize(&self, value: Option<String>) -> Option<String> {
        value
    }
}

/// A relationship reference that can be serialized to/from API format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    #[serde(rename = "type")]
    pub type_name: String,
    pub id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_date_transform() {
        let transform = DateTransform;
        let date = NaiveDate::from_ymd_opt(2023, 7, 15).unwrap();

        let serialized = transform.serialize(Some(date));
        assert_eq!(serialized, Some("2023-07-15".to_string()));

        let deserialized = transform.deserialize(serialized);
        assert_eq!(deserialized, Some(date));
    }

    #[test]
    fn test_time_transform() {
        let transform = TimeTransform;
        let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();

        let serialized = transform.serialize(Some(time));
        assert_eq!(serialized, Some("14:30:00".to_string()));

        let deserialized = transform.deserialize(serialized);
        assert_eq!(deserialized, Some(time));
    }

    #[test]
    fn test_duration_transform() {
        let transform = DurationTransform;
        let duration = Duration::hours(2) + Duration::minutes(45);

        let serialized = transform.serialize(Some(duration));
        assert_eq!(serialized, Some("02:45:00".to_string()));

        let deserialized = transform.deserialize(serialized);
        assert_eq!(deserialized, Some(duration));
    }

    #[test]
    fn test_boolean_transform() {
        let transform = BooleanTransform;

        assert_eq!(transform.serialize(true), 1);
        assert_eq!(transform.serialize(false), 0);

        assert!(transform.deserialize(1));
        assert!(!transform.deserialize(0));
    }
}
