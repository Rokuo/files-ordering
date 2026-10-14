#![allow(dead_code)]

use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::{fs, io};

use chrono::Local;

use crate::models::error::move_error::MoveError;
use crate::models::run_report::{MoveOutcome, ReportFile, RunReport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictKind {
    InPlan,
    OnDisk,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanStatus {
    New,
    Conflict(ConflictKind),
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedMove {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub status: PlanStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub entries: Vec<PlannedMove>,
}

fn move_file(from: &Path, to: &Path) -> Result<(), io::Error> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    if !to.try_exists()? {
        fs::rename(from, to)
    } else {
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "File already exists",
        ))
    }
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

    pub fn execute(&self, rule_name: String) -> RunReport {
        let mut run_report = RunReport {
            date: Local::now().date_naive(),
            rule_name,
            reports: vec![],
        };
        for planned_move in &self.entries {
            let file_report = ReportFile {
                source: planned_move.source.clone(),
                destination: planned_move.destination.clone(),
                outcome: match planned_move.status {
                    PlanStatus::New => {
                        match move_file(&planned_move.source, &planned_move.destination) {
                            Ok(_) => MoveOutcome::Moved,
                            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                                MoveOutcome::Conflict
                            }
                            Err(err) => MoveOutcome::Failed(MoveError::from(err)),
                        }
                    }
                    PlanStatus::Skipped => MoveOutcome::Skipped,
                    PlanStatus::Conflict(_) => MoveOutcome::Conflict,
                },
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
    fn test_execute_moves_new_entries() {
        let dir = TempDir::new().unwrap();
        let plan = Plan {
            entries: vec![
                create_file_and_plan(&dir, "file1.txt"),
                create_file_and_plan(&dir, "file2.txt"),
            ],
        };

        let report = plan.execute("rule".to_string());

        assert_eq!(report.reports.len(), 2);
        for (entry, planned) in report.reports.iter().zip(&plan.entries) {
            assert_eq!(entry.outcome, MoveOutcome::Moved);
            assert_eq!(entry.source, planned.source);
            assert_eq!(entry.destination, planned.destination);
            assert!(!planned.source.exists());
            assert_eq!(
                std::fs::read(&planned.destination).unwrap(),
                b"some content"
            );
        }
    }

    #[test]
    fn test_execute_leaves_skipped_and_conflicting_entries_in_place() {
        let dir = TempDir::new().unwrap();
        let plan = Plan {
            entries: vec![
                PlannedMove {
                    status: PlanStatus::Skipped,
                    ..create_file_and_plan(&dir, "skipped.txt")
                },
                PlannedMove {
                    status: PlanStatus::Conflict(ConflictKind::InPlan),
                    ..create_file_and_plan(&dir, "in_plan.txt")
                },
                PlannedMove {
                    status: PlanStatus::Conflict(ConflictKind::OnDisk),
                    ..create_file_and_plan(&dir, "on_disk.txt")
                },
            ],
        };

        let report = plan.execute("rule".to_string());

        let outcomes: Vec<MoveOutcome> = report.reports.iter().map(|r| r.outcome.clone()).collect();
        assert_eq!(
            outcomes,
            vec![
                MoveOutcome::Skipped,
                MoveOutcome::Conflict,
                MoveOutcome::Conflict
            ]
        );
        for planned in &plan.entries {
            assert!(planned.source.exists());
            assert!(!planned.destination.exists());
        }
    }

    #[test]
    fn test_execute_keeps_going_after_a_failed_move() {
        let dir = TempDir::new().unwrap();
        let missing = PlannedMove {
            source: dir.path().join("ghost.txt"),
            destination: dir.path().join("dest_ghost.txt"),
            status: PlanStatus::New,
        };
        let plan = Plan {
            entries: vec![missing, create_file_and_plan(&dir, "file.txt")],
        };

        let report = plan.execute("rule".to_string());

        assert_eq!(
            report.reports[0].outcome,
            MoveOutcome::Failed(MoveError::NotFound)
        );
        assert_eq!(report.reports[1].outcome, MoveOutcome::Moved);
        assert!(plan.entries[1].destination.exists());
    }

    #[test]
    fn test_execute_never_overwrites_an_existing_destination() {
        let dir = TempDir::new().unwrap();
        let entry = create_file_and_plan(&dir, "file.txt");
        std::fs::write(&entry.destination, b"already there").unwrap();
        let plan = Plan {
            entries: vec![entry],
        };

        let report = plan.execute("rule".to_string());

        assert_eq!(report.reports[0].outcome, MoveOutcome::Conflict);
        assert!(plan.entries[0].source.exists());
        assert_eq!(
            std::fs::read(&plan.entries[0].destination).unwrap(),
            b"already there"
        );
    }

    #[test]
    fn test_execute_records_the_rule_name_and_the_day_of_the_run() {
        let plan = Plan { entries: vec![] };

        // bracketed by two reads of the clock so a run across midnight cannot fail it
        let before = Local::now().date_naive();
        let report = plan.execute("Camera dump".to_string());
        let after = Local::now().date_naive();

        assert_eq!(report.rule_name, "Camera dump");
        assert!(before <= report.date && report.date <= after);
        assert!(report.reports.is_empty());
    }

    #[test]
    fn test_execute_plan_with_sub_folder_expects_moved() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("photo.txt");
        let destination = dir.path().join("2026").join("09").join("photo.txt");
        std::fs::write(&source, b"coucou").unwrap();
        let plan = Plan {
            entries: vec![PlannedMove {
                source: source.clone(),
                destination: destination.clone(),
                status: PlanStatus::New,
            }],
        };

        let run_report = plan.execute("rule name".to_string());

        assert_eq!(run_report.reports.len(), 1);
        assert_eq!(run_report.reports[0].outcome, MoveOutcome::Moved);
        assert!(!source.exists());
        assert_eq!(std::fs::read(&destination).unwrap(), b"coucou");
    }
}
