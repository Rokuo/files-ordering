#![allow(dead_code)]

use std::fmt::Debug;
use std::io::ErrorKind;
use std::path::PathBuf;

use serde::Serialize;

use crate::files::move_file;
use crate::models::error::move_error::MoveError;
use crate::models::run_report::{MoveOutcome, ReportFile, RunReport};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ConflictKind {
    InPlan,
    OnDisk,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PlanStatus {
    New,
    Conflict(ConflictKind),
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlannedMove {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub status: PlanStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Plan {
    pub entries: Vec<PlannedMove>,
}

impl Plan {
    pub fn check_disk(&mut self) -> std::io::Result<()> {
        for entry in &mut self.entries {
            if entry.status == PlanStatus::New && entry.destination.try_exists()? {
                entry.status = PlanStatus::Conflict(ConflictKind::OnDisk);
            }
        }
        Ok(())
    }

    pub fn execute(&self) -> RunReport {
        let mut run_report = RunReport { reports: vec![] };
        for planned_move in &self.entries {
            let outcome: MoveOutcome = match planned_move.status {
                PlanStatus::New => {
                    if let Err(err) = move_file(&planned_move.source, &planned_move.destination) {
                        match err.kind() {
                            ErrorKind::NotFound => {
                                MoveOutcome::Failed(MoveError::NotFound)
                            }
                            ErrorKind::PermissionDenied => {
                                MoveOutcome::Failed(MoveError::PermissionDenied)
                            }
                            _ => {
                                MoveOutcome::Failed(MoveError::Other(err.to_string()))
                            }
                        }
                    } else {
                        MoveOutcome::Moved
                    }
                },
                PlanStatus::Skipped => MoveOutcome::Skipped,
                PlanStatus::Conflict(_) => MoveOutcome::Conflict,
            };

            let file_report = ReportFile {
                original_path: planned_move
                    .source
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap(),
                destination_path: planned_move
                    .destination
                    .clone()
                    .into_os_string()
                    .into_string()
                    .unwrap(),
                outcome: outcome,
            };

            run_report.reports.push(file_report)
        }
        run_report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn planned(destination: PathBuf, status: PlanStatus) -> PlannedMove {
        PlannedMove {
            source: PathBuf::from("/source").join(destination.file_name().unwrap()),
            destination,
            status,
        }
    }

    fn create_file_and_plan(dir: &TempDir, file_name: &str) -> PlannedMove {
        let src = dir.path().join(file_name);
        let dest = dir.path().join("dest_".to_owned() + file_name);
        std::fs::write(&src, b"some content").unwrap();
        PlannedMove {
            source: src,
            destination: dest,
            status: PlanStatus::New,
        }
    }

    #[test]
    fn test_check_disk_flags_only_destinations_that_exist() {
        let dir = TempDir::new().unwrap();
        let taken = dir.path().join("already-there.txt");
        let free = dir.path().join("nothing-here.txt");
        std::fs::write(&taken, b"an existing file").unwrap();

        let mut plan = Plan {
            entries: vec![
                planned(taken, PlanStatus::New),
                planned(free, PlanStatus::New),
            ],
        };

        // act
        plan.check_disk().unwrap();

        // assert
        assert_eq!(
            plan.entries[0].status,
            PlanStatus::Conflict(ConflictKind::OnDisk)
        );
        assert_eq!(plan.entries[1].status, PlanStatus::New);
    }

    #[test]
    fn test_check_disk_leaves_other_statuses_alone() {
        // arrange
        let dir = TempDir::new().unwrap();
        let taken = dir.path().join("already-there.txt");
        std::fs::write(&taken, b"an existing file").unwrap();

        let mut plan = Plan {
            entries: vec![
                planned(taken.clone(), PlanStatus::Skipped),
                planned(taken, PlanStatus::Conflict(ConflictKind::InPlan)),
            ],
        };

        // act
        plan.check_disk().unwrap();

        // assert
        assert_eq!(plan.entries[0].status, PlanStatus::Skipped);
        assert_eq!(
            plan.entries[1].status,
            PlanStatus::Conflict(ConflictKind::InPlan)
        );
    }

    #[test]
    fn test_check_disk_reports_a_path_it_cannot_read() {
        let mut plan = Plan {
            entries: vec![planned(PathBuf::from("inva\0lid"), PlanStatus::New)],
        };

        assert!(plan.check_disk().is_err());
        assert_eq!(plan.entries[0].status, PlanStatus::New);
    }

    #[test]
    fn test_execute_sample_plan_no_errors() {
        let tmp_dir = TempDir::new().unwrap();
        let skipped = tmp_dir.path().join("conflict-planned.txt");
        let plan = Plan {
            entries: vec![
                create_file_and_plan(&tmp_dir, "file1.txt"),
                create_file_and_plan(&tmp_dir, "file2.txt"),
                planned(skipped, PlanStatus::Skipped),
            ],
        };

        let runreport = plan.execute();

        assert!(runreport.reports.len() == 3);
        assert_eq!(runreport.reports[0].outcome, MoveOutcome::Moved);
        assert_ne!(runreport.reports[0].destination_path, String::new());
        assert_eq!(runreport.reports[1].outcome, MoveOutcome::Moved);
        assert_ne!(runreport.reports[1].destination_path, String::new());
        assert_eq!(runreport.reports[2].outcome, MoveOutcome::Skipped);
        assert_ne!(runreport.reports[2].destination_path, String::new());
    }

    #[test]
    fn test_execute_sample_plan_with_errors() {
        let dir = TempDir::new().unwrap();
        let err_not_found = dir.path().join("nothing-here.txt");
        let conflict = dir.path().join("conflict-planned.txt");
        let plan = Plan {
            entries: vec![
                create_file_and_plan(&dir, "already-here.txt"),
                planned(err_not_found, PlanStatus::New),
                planned(conflict, PlanStatus::Conflict(ConflictKind::OnDisk)),
            ],
        };

        let run_report = plan.execute();

        assert_eq!(run_report.reports.len(), 3);
        assert_eq!(
            run_report.reports[1].outcome,
            MoveOutcome::Failed(MoveError::NotFound)
        );
        assert_ne!(run_report.reports[1].destination_path, String::new());
        assert_eq!(run_report.reports[2].outcome, MoveOutcome::Conflict);
    }

    #[test]
    fn test_execute_plan_with_sub_folder_expects_moved() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("photo.txt");
        let destination = dir.path().join("2026").join("09").join("photo.txt");
        std::fs::write(&source, b"coucou").unwrap();
        let plan = Plan {
            entries: vec![ PlannedMove {
                    source: source.clone(),
                    destination: destination.clone(),
                    status: PlanStatus::New
                }
            ],
        };

        let run_report = plan.execute();

        assert_eq!(run_report.reports.len(), 1);
        assert_eq!(run_report.reports[0].outcome, MoveOutcome::Moved);
        assert!(!source.exists());
        assert_eq!(std::fs::read(&destination).unwrap(), b"coucou");
    }
}
