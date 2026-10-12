#![allow(dead_code)]

use crate::models::error::move_error::MoveError;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum MoveOutcome {
    Moved,
    Skipped,
    Conflict,
    Failed(MoveError),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReportFile {
    pub original_path: String,
    pub destination_path: String,
    pub outcome: MoveOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunReport {
    pub reports: Vec<ReportFile>,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::models::plan::{Plan, PlanStatus, PlannedMove};
    use tempfile::TempDir;

    fn planned(destination: PathBuf, status: PlanStatus) -> PlannedMove {
        PlannedMove {
            source: PathBuf::from("/source").join(destination.file_name().unwrap()),
            destination,
            status,
        }
    }

    #[test]
    fn test_execute_moves_a_new_entry() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("already-there.txt");
        let _plan = Plan {
            entries: vec![planned(path, PlanStatus::New)],
        };
        let _run_report: RunReport;
    }
}
