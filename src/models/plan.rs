#![allow(dead_code)]

use std::path::PathBuf;

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

impl Plan {
    pub fn check_disk(&mut self) -> std::io::Result<()> {
        for entry in &mut self.entries {
            if entry.status == PlanStatus::New && entry.destination.try_exists()? {
                entry.status = PlanStatus::Conflict(ConflictKind::OnDisk);
            }
        }
        Ok(())
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
        // An interior NUL cannot appear in a real path, so `try_exists`
        // fails rather than answering "no such file".
        let mut plan = Plan {
            entries: vec![planned(PathBuf::from("inva\0lid"), PlanStatus::New)],
        };

        assert!(plan.check_disk().is_err());
        assert_eq!(plan.entries[0].status, PlanStatus::New);
    }
}
