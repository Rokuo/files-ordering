#![allow(dead_code)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::models::error::move_error::MoveError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveOutcome {
    Moved,
    Skipped,
    Conflict,
    Failed(MoveError),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportFile {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub outcome: MoveOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunReport {
    pub date: NaiveDate,
    pub rule_name: String,
    pub reports: Vec<ReportFile>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, outcome: MoveOutcome) -> ReportFile {
        ReportFile {
            source: PathBuf::from("inbox").join(name),
            destination: PathBuf::from("sorted").join(name),
            outcome,
        }
    }

    #[test]
    fn test_run_report_round_trips_through_json() {
        let report = RunReport {
            date: NaiveDate::from_ymd_opt(2026, 10, 14).unwrap(),
            rule_name: "Camera dump".to_string(),
            reports: vec![
                entry("moved.jpg", MoveOutcome::Moved),
                entry("skipped.jpg", MoveOutcome::Skipped),
                entry("conflict.jpg", MoveOutcome::Conflict),
                entry(
                    "failed.jpg",
                    MoveOutcome::Failed(MoveError::Other("disk full".to_string())),
                ),
            ],
        };

        let json = serde_json::to_string(&report).unwrap();
        let back: RunReport = serde_json::from_str(&json).unwrap();

        assert_eq!(back, report);
    }
}
