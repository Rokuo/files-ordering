#![allow(dead_code)]

use chrono::Local;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::models::conflict_policy::ConflictPolicy;
use crate::models::error::move_error::MoveError;
use crate::models::move_mode::MoveMode;
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
    Replace,
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

fn create_dir_all_and_check_path(path: &Path) -> io::Result<bool> {
    if let Some(parent_path) = path.parent() {
        fs::create_dir_all(parent_path)?;
    }
    path.try_exists()
}

fn copy_then_delete_with(
    from: &Path,
    to: &Path,
    delete: impl Fn(&Path) -> io::Result<()>,
) -> io::Result<MoveOutcome> {
    copy_file_with(from, to, |org, dest| {
        fs::copy(org, dest)?;
        Ok(MoveOutcome::Copied)
    })?;
    match delete(from) {
        Ok(()) => Ok(MoveOutcome::Moved),
        Err(err) => Ok(MoveOutcome::CopiedButDeleteFailed(MoveError::from(err))),
    }
}

fn copy_then_delete(from: &Path, to: &Path) -> io::Result<MoveOutcome> {
    copy_then_delete_with(from, to, |path| fs::remove_file(path))
}

fn move_file_with(
    from: &Path,
    to: &Path,
    rename: impl Fn(&Path, &Path) -> io::Result<MoveOutcome>,
) -> io::Result<MoveOutcome> {
    match rename(from, to) {
        Err(err) if err.kind() == io::ErrorKind::CrossesDevices => copy_then_delete(from, to),
        other => other,
    }
}

fn copy_file_with(
    from: &Path,
    to: &Path,
    copy: impl Fn(&Path, &Path) -> io::Result<MoveOutcome>,
) -> io::Result<MoveOutcome> {
    match copy(from, to) {
        Err(err) => {
            let _ = fs::remove_file(to);
            Err(err)
        }
        other => other,
    }
}

fn move_file(from: &Path, to: &Path, move_mode: MoveMode) -> io::Result<MoveOutcome> {
    if create_dir_all_and_check_path(to)? {
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "File already exists",
        ))
    } else {
        match move_mode {
            MoveMode::Move => move_file_with(from, to, |from, to| {
                fs::rename(from, to)?;
                Ok(MoveOutcome::Moved)
            }),
            MoveMode::Copy => copy_file_with(from, to, |from, to| {
                fs::copy(from, to)?;
                Ok(MoveOutcome::Copied)
            }),
        }
    }
}

fn with_counter(file_path: &Path, n: u32) -> PathBuf {
    let mut path_buf: PathBuf = PathBuf::from(file_path);
    let file_name: String = file_path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
    let file_name_with_counter : String = format!("{file_name} ({n})");
    path_buf.set_file_name(file_name_with_counter);
    path_buf.set_extension(file_path.extension().and_then(|ext| ext.to_str()).unwrap_or(""));
    path_buf
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

    pub fn execute(&self, rule_name: String, move_mode: MoveMode) -> RunReport {
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
                        match move_file(&planned_move.source, &planned_move.destination, move_mode)
                        {
                            Ok(move_outcome) => move_outcome,
                            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                                MoveOutcome::Conflict
                            }
                            Err(err) => MoveOutcome::Failed(MoveError::from(err)),
                        }
                    }
                    PlanStatus::Skipped => MoveOutcome::Skipped,
                    PlanStatus::Conflict(_) => MoveOutcome::Conflict,
                    PlanStatus::Replace => todo!(),
                },
            };
            run_report.reports.push(file_report)
        }
        run_report
    }
    
    fn resolve_conflicts(&self, _policy: ConflictPolicy) -> io::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::models::conflict_policy::ConflictPolicy;

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

        let report = plan.execute("rule".to_string(), MoveMode::Move);

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

        let report = plan.execute("rule".to_string(), MoveMode::Move);

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

        let report = plan.execute("rule".to_string(), MoveMode::Move);

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

        let report = plan.execute("rule".to_string(), MoveMode::Move);

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
        let report = plan.execute("Camera dump".to_string(), MoveMode::Move);
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

        let run_report = plan.execute("rule name".to_string(), MoveMode::Move);

        assert_eq!(run_report.reports.len(), 1);
        assert_eq!(run_report.reports[0].outcome, MoveOutcome::Moved);
        assert!(!source.exists());
        assert_eq!(std::fs::read(&destination).unwrap(), b"coucou");
    }

    #[test]
    fn test_copy_then_delete_moves_the_bytes_and_removes_the_source() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("photo.jpg");
        let destination = dir.path().join("photo_copy.jpg");
        std::fs::write(&source, b"pixels").unwrap();

        copy_then_delete(&source, &destination).unwrap();

        assert!(!source.exists());
        assert_eq!(std::fs::read(&destination).unwrap(), b"pixels");
    }
    #[test]
    fn test_move_file_falls_back_to_a_copy_when_rename_crosses_devices() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("photo.jpg");
        let destination = dir.path().join("photo_moved.jpg");
        std::fs::write(&source, b"pixels").unwrap();

        move_file_with(&source, &destination, |_, _| {
            Err(io::Error::from(io::ErrorKind::CrossesDevices))
        })
        .unwrap();

        assert!(!source.exists());
        assert_eq!(std::fs::read(&destination).unwrap(), b"pixels");
    }

    #[test]
    fn test_move_file_does_not_copy_when_rename_fails_for_another_reason() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("photo.jpg");
        let destination = dir.path().join("photo_moved.jpg");
        std::fs::write(&source, b"pixels").unwrap();

        let result = move_file_with(&source, &destination, |_, _| {
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        });

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        assert!(source.exists());
        assert!(!destination.exists());
    }

    #[test]
    fn test_move_file_in_copy_mode_never_overwrites_an_existing_destination() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("photo.jpg");
        let destination = dir.path().join("photo_copy.jpg");
        std::fs::write(&source, b"pixels").unwrap();
        std::fs::write(&destination, b"already there").unwrap();

        let result = move_file(&source, &destination, MoveMode::Copy);

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&source).unwrap(), b"pixels");
        assert_eq!(std::fs::read(&destination).unwrap(), b"already there");
    }

    #[test]
    fn test_move_file_keeps_the_message_of_a_failed_rename() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("photo.jpg");
        let destination = dir.path().join("photo_moved.jpg");
        std::fs::write(&source, b"pixels").unwrap();

        let result = move_file_with(&source, &destination, |_, _| {
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "the drive stopped answering",
            ))
        });

        assert_eq!(
            result.unwrap_err().to_string(),
            "the drive stopped answering"
        );
    }

    #[test]
    fn test_copy_then_delete_reports_a_source_it_could_not_delete() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("photo.jpg");
        let destination = dir.path().join("photo_moved.jpg");
        std::fs::write(&source, b"pixels").unwrap();

        let result = copy_then_delete_with(&source, &destination, |_| {
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        });

        assert_eq!(
            result.unwrap(),
            MoveOutcome::CopiedButDeleteFailed(MoveError::PermissionDenied)
        );
        assert!(source.exists());
        assert_eq!(std::fs::read(&destination).unwrap(), b"pixels");
    }

    #[test]
    fn test_copy_file_removes_a_half_written_destination_when_the_copy_fails() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("photo.jpg");
        let destination = dir.path().join("photo_copy.jpg");
        std::fs::write(&source, b"pixels").unwrap();

        let result = copy_file_with(&source, &destination, |_, to| {
            std::fs::write(to, b"pix")?;
            Err(io::Error::from(io::ErrorKind::StorageFull))
        });

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::StorageFull);
        assert!(!destination.exists());
        assert_eq!(std::fs::read(&source).unwrap(), b"pixels");
    }

    #[test]
    fn test_execute_in_copy_mode_leaves_the_source_in_place() {
        let dir = TempDir::new().unwrap();
        let plan = Plan {
            entries: vec![create_file_and_plan(&dir, "file.txt")],
        };

        let report = plan.execute("rule".to_string(), MoveMode::Copy);

        assert!(plan.entries[0].source.exists());
        assert_eq!(report.reports[0].outcome, MoveOutcome::Copied);
        assert_eq!(
            std::fs::read(&plan.entries[0].destination).unwrap(),
            b"some content"
        );
    }

    #[test]
    fn test_copy_file_keeps_the_copy_error_when_nothing_was_written() {
        let dir = TempDir::new().unwrap();
        let source = dir.path().join("photo.jpg");
        std::fs::write(&source, b"pixels").unwrap();

        let result = copy_file_with(&source, &dir.path().join("photo_copy.jpg"), |_, _| {
            Err(io::Error::from(io::ErrorKind::StorageFull))
        });

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::StorageFull);
    }

    #[test]
    #[ignore = "needs two filesystems; run by hand with: cargo test -- --ignored"]
    fn test_execute_moves_a_file_to_another_filesystem() {
        let here = TempDir::new().unwrap();
        let elsewhere = TempDir::new_in("target").unwrap();
        let probe = here.path().join("probe");
        std::fs::write(&probe, b"").unwrap();
        let crossing = std::fs::rename(&probe, elsewhere.path().join("probe"));
        assert!(
            crossing.is_err_and(|e| e.kind() == io::ErrorKind::CrossesDevices),
            "the temp folder and target/ are on the same filesystem on this machine"
        );
        let source = here.path().join("photo.jpg");
        let destination = elsewhere.path().join("photo.jpg");
        std::fs::write(&source, b"pixels").unwrap();
        let plan = Plan {
            entries: vec![PlannedMove {
                source: source.clone(),
                destination: destination.clone(),
                status: PlanStatus::New,
            }],
        };

        let report = plan.execute("rule".to_string(), MoveMode::Move);

        assert_eq!(report.reports[0].outcome, MoveOutcome::Moved);
        assert!(!source.exists());
        assert_eq!(std::fs::read(&destination).unwrap(), b"pixels");
    }

    #[test]
    fn test_with_counter_adds_the_number_before_extension() {
        let photo = PathBuf::from("photos").join("IMG_4471.heic");

        assert_eq!(
            with_counter(&photo, 1),
            PathBuf::from("photos").join("IMG_4471 (1).heic")
        );
        assert_eq!(
            with_counter(&photo, 2),
            PathBuf::from("photos").join("IMG_4471 (2).heic")
        );
    }

    #[test]
    fn test_with_counter_handles_names_without_a_plain_extension() {
        assert_eq!(
            with_counter(Path::new("README"), 1),
            PathBuf::from("README (1)")
        );
        assert_eq!(
            with_counter(Path::new(".gitignore"), 1),
            PathBuf::from(".gitignore (1)")
        );
        assert_eq!(
            with_counter(Path::new("archive.tar.gz"), 1),
            PathBuf::from("archive.tar (1).gz")
        );
    }

    #[test]
    fn test_resolve_conflicts_keep_both_finds_the_first_free_number() {
        let dir = TempDir::new().unwrap();
        let taken = dir.path().join("photo.jpg");
        let free = dir.path().join("free.jpg");
        std::fs::write(&taken, b"already there").unwrap();
        std::fs::write(with_counter(&taken, 1), b"also there").unwrap();
        let plan = Plan {
            entries: vec![
                planned(taken.clone(), PlanStatus::Conflict(ConflictKind::OnDisk)),
                planned(free.clone(), PlanStatus::New),
            ],
        };

        plan.resolve_conflicts(ConflictPolicy::KeepBoth).unwrap();

        assert_eq!(plan.entries[0].status, PlanStatus::New);
        assert_eq!(plan.entries[0].destination, with_counter(&taken, 2));
        assert_eq!(plan.entries[1].status, PlanStatus::New);
        assert_eq!(plan.entries[1].destination, free);
    }

    #[test]
    fn test_resolve_conflicts_keep_both_lets_the_first_claimant_keep_the_name() {
        let dir = TempDir::new().unwrap();
        let shared = dir.path().join("photo.jpg");
        let plan = Plan {
            entries: vec![
                planned(shared.clone(), PlanStatus::Conflict(ConflictKind::InPlan)),
                planned(shared.clone(), PlanStatus::Conflict(ConflictKind::InPlan)),
            ],
        };

        plan.resolve_conflicts(ConflictPolicy::KeepBoth).unwrap();

        assert_eq!(plan.entries[0].destination, shared);
        assert_eq!(plan.entries[1].destination, with_counter(&shared, 1));
        assert!(plan.entries.iter().all(|e| e.status == PlanStatus::New));
    }

    #[test]
    fn test_resolve_conflicts_replace_never_targets_a_file_from_the_same_run() {
        let dir = TempDir::new().unwrap();
        let on_disk = dir.path().join("old.jpg");
        let shared = dir.path().join("shared.jpg");
        std::fs::write(&on_disk, b"old").unwrap();
        let plan = Plan {
            entries: vec![
                planned(on_disk, PlanStatus::Conflict(ConflictKind::OnDisk)),
                planned(shared.clone(), PlanStatus::Conflict(ConflictKind::InPlan)),
                planned(shared, PlanStatus::Conflict(ConflictKind::InPlan)),
            ],
        };

        plan.resolve_conflicts(ConflictPolicy::Replace).unwrap();

        assert_eq!(plan.entries[0].status, PlanStatus::Replace);
        assert_eq!(plan.entries[1].status, PlanStatus::New);
        assert_eq!(
            plan.entries[2].status,
            PlanStatus::Conflict(ConflictKind::InPlan)
        );
    }

    #[test]
    fn test_resolve_conflicts_skip_leaves_real_conflicts_alone() {
        let dir = TempDir::new().unwrap();
        let on_disk = dir.path().join("old.jpg");
        let shared = dir.path().join("shared.jpg");
        std::fs::write(&on_disk, b"old").unwrap();
        let plan = Plan {
            entries: vec![
                planned(on_disk.clone(), PlanStatus::Conflict(ConflictKind::OnDisk)),
                planned(shared.clone(), PlanStatus::Conflict(ConflictKind::InPlan)),
                planned(shared.clone(), PlanStatus::Conflict(ConflictKind::InPlan)),
            ],
        };

        plan.resolve_conflicts(ConflictPolicy::Skip).unwrap();

        assert_eq!(
            plan.entries[0].status,
            PlanStatus::Conflict(ConflictKind::OnDisk)
        );
        assert_eq!(plan.entries[1].status, PlanStatus::New);
        assert_eq!(
            plan.entries[2].status,
            PlanStatus::Conflict(ConflictKind::InPlan)
        );
        assert_eq!(plan.entries[0].destination, on_disk);
        assert_eq!(plan.entries[2].destination, shared);
    }

    #[test]
    fn test_execute_replace_overwrites_and_leaves_no_backup_behind() {
        let dir = TempDir::new().unwrap();
        let entry = create_file_and_plan(&dir, "photo.jpg");
        std::fs::write(&entry.destination, b"old content").unwrap();
        let plan = Plan {
            entries: vec![PlannedMove {
                status: PlanStatus::Replace,
                ..entry
            }],
        };

        let report = plan.execute("rule".to_string(), MoveMode::Move);

        assert_eq!(report.reports[0].outcome, MoveOutcome::Moved);
        assert!(!plan.entries[0].source.exists());
        assert_eq!(
            std::fs::read(&plan.entries[0].destination).unwrap(),
            b"some content"
        );
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn test_execute_replace_keeps_the_old_file_when_the_move_fails() {
        let dir = TempDir::new().unwrap();
        let destination = dir.path().join("photo.jpg");
        std::fs::write(&destination, b"old content").unwrap();
        let plan = Plan {
            entries: vec![PlannedMove {
                source: dir.path().join("ghost.jpg"),
                destination: destination.clone(),
                status: PlanStatus::Replace,
            }],
        };

        let report = plan.execute("rule".to_string(), MoveMode::Move);

        assert_eq!(
            report.reports[0].outcome,
            MoveOutcome::Failed(MoveError::NotFound)
        );
        assert_eq!(std::fs::read(&destination).unwrap(), b"old content");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn test_keep_both_moves_the_file_under_its_new_name() {
        let dir = TempDir::new().unwrap();
        let entry = create_file_and_plan(&dir, "photo.jpg");
        let original = entry.destination.clone();
        std::fs::write(&original, b"old content").unwrap();
        let mut plan = Plan {
            entries: vec![entry],
        };
        plan.check_disk().unwrap();

        plan.resolve_conflicts(ConflictPolicy::KeepBoth).unwrap();
        let report = plan.execute("rule".to_string(), MoveMode::Move);

        let renamed = with_counter(&original, 1);
        assert_eq!(report.reports[0].outcome, MoveOutcome::Moved);
        assert_eq!(report.reports[0].destination, renamed);
        assert_eq!(std::fs::read(&renamed).unwrap(), b"some content");
        assert_eq!(std::fs::read(&original).unwrap(), b"old content");
    }
}
