#![allow(dead_code)]

use crate::models::destination::Destination;
use crate::models::error::pattern_error::PatternError;
use crate::models::file_item::FileItem;
use crate::models::history::History;
use crate::models::matches::{Condition, MatchMode};
use crate::models::plan::{ConflictKind, Plan, PlanStatus, PlannedMove};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    pub name: String,
    pub destination: Destination,
    pub match_mode: MatchMode,
    pub conditions: Vec<Condition>,
    pub last_run: Option<History>,
}

impl Rule {
    pub fn matches(&self, file_item: &FileItem) -> bool {
        if self.conditions.is_empty() {
            return true;
        }

        match self.match_mode {
            MatchMode::Any => self
                .conditions
                .iter()
                .any(|condition| condition.matches(file_item)),
            MatchMode::All => self
                .conditions
                .iter()
                .all(|condition| condition.matches(file_item)),
        }
    }

    pub fn plan(&self, files: &[FileItem]) -> Result<Plan, PatternError> {
        let mut plan = Plan {
            entries: Vec::new(),
        };
        let mut claimed: HashSet<PathBuf> = HashSet::new();
        let mut contested: HashSet<PathBuf> = HashSet::new();

        for file_item in files {
            if !self.matches(file_item) {
                continue;
            }

            let destination = self
                .destination
                .folder
                .join(self.destination.render_folder(file_item)?)
                .join(file_item.file_name());

            if !claimed.insert(destination.clone()) {
                contested.insert(destination.clone());
            }

            let status = if destination == file_item.path {
                PlanStatus::Skipped
            } else {
                PlanStatus::New
            };

            plan.entries.push(PlannedMove {
                source: file_item.path.clone(),
                destination,
                status,
            });
        }

        plan.entries
            .iter_mut()
            .filter(|entry| entry.status == PlanStatus::New)
            .filter(|entry| contested.contains(&entry.destination))
            .for_each(|entry| entry.status = PlanStatus::Conflict(ConflictKind::InPlan));

        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::destination::Rename;
    use crate::models::file_item::FileItem;
    use crate::models::matches::{DateTest, ExtensionTest, NameTest, SizeTest};
    use chrono::NaiveDate;
    use std::path::PathBuf;

    fn sample_file_item() -> FileItem {
        crate::models::file_item::FileItem {
            path: std::path::PathBuf::new(),
            name: "test_file".to_string(),
            extension: Some("png".to_string()),
            size: 600_000,
            destination: None,
            created_at: NaiveDate::from_ymd_opt(2023, 6, 15),
            modified_at: NaiveDate::from_ymd_opt(2023, 6, 20),
        }
    }

    fn sample_rule(match_mode: MatchMode) -> Rule {
        Rule {
            name: "Sample Rule".to_string(),
            destination: Destination {
                folder: PathBuf::from("/path/to/destination"),
                sub_folder_pattern: "{year}/{month}".to_string(),
                rename: Some(Rename {
                    pattern: r"^IMG_(\d+)".to_string(),
                    replacement: "holiday-$1".to_string(),
                }),
            },
            match_mode,
            conditions: vec![
                Condition::Stem(NameTest::StartsWith("test".to_string())),
                Condition::Stem(NameTest::Contains("invoice".to_string())),
                Condition::Size(SizeTest::LargerThan(1024)),
                Condition::Created(DateTest::After(
                    NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                )),
                Condition::Modified(DateTest::Before(
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                )),
                Condition::Extension(ExtensionTest::IsOneOf(vec![
                    "jpg".to_string(),
                    "png".to_string(),
                ])),
            ],
            last_run: None,
        }
    }

    #[test]
    fn test_rule_serialization() {
        // arrange
        let rule: Rule = sample_rule(MatchMode::All);

        // act
        let json: String = serde_json::to_string(&rule).unwrap();
        println!("Serialized Rule: {}", json);
        let back: Rule = serde_json::from_str(&json).unwrap();
        // Assert
        assert_eq!(rule, back);
    }

    #[test]
    fn test_rule_serialization_as_nested_json() {
        // arrange
        let rule: Rule = sample_rule(MatchMode::All);

        // act
        let json: String = serde_json::to_string(&rule).unwrap();
        // Assert
        assert_eq!(
            json,
            r#"{"name":"Sample Rule","destination":{"folder":"/path/to/destination","sub_folder_pattern":"{year}/{month}","rename":{"pattern":"^IMG_(\\d+)","replacement":"holiday-$1"}},"match_mode":"All","conditions":[{"Stem":{"StartsWith":"test"}},{"Stem":{"Contains":"invoice"}},{"Size":{"LargerThan":1024}},{"Created":{"After":"2023-01-01"}},{"Modified":{"Before":"2024-01-01"}},{"Extension":{"IsOneOf":["jpg","png"]}}],"last_run":null}"#
        );
    }

    /// The sample file satisfies some conditions but not all of them, so Any
    /// accepts it and All rejects it. One fixture, two modes, opposite answers.
    #[test]
    fn test_any_condition_matches() {
        // arrange
        let rule = sample_rule(MatchMode::Any);
        let file_item = sample_file_item();

        // act
        let matched = rule.matches(&file_item);

        // assert
        assert!(matched);
    }

    #[test]
    fn test_all_conditions_must_match() {
        // arrange
        let rule = sample_rule(MatchMode::All);
        let file_item = sample_file_item();

        // act
        let matched = rule.matches(&file_item);

        // assert
        assert!(!matched);
    }

    #[test]
    fn test_rule_with_no_conditions_matches_everything() {
        // arrange
        let file_item = sample_file_item();
        let mut all_mode = sample_rule(MatchMode::All);
        all_mode.conditions.clear();
        let mut any_mode = sample_rule(MatchMode::Any);
        any_mode.conditions.clear();

        // act / assert
        assert!(all_mode.matches(&file_item));
        assert!(any_mode.matches(&file_item));
    }

    #[test]
    fn test_plan_builds_the_full_destination_path() {
        // arrange
        let rule = sample_rule(MatchMode::Any);
        let file_item = sample_file_item();
        let files = vec![file_item];

        // act
        let plan = rule.plan(&files).unwrap();

        // assert
        assert_eq!(plan.entries.len(), 1);
        assert_eq!(
            plan.entries[0].destination,
            PathBuf::from("/path/to/destination")
                .join("2023")
                .join("06")
                .join("test_file.png")
        );
        assert_eq!(plan.entries[0].status, PlanStatus::New);
    }

    #[test]
    fn test_plan_leaves_distinct_destinations_as_new() {
        // arrange
        let rule = sample_rule(MatchMode::Any);
        let files = vec![
            sample_file_item(),
            FileItem {
                name: "another_file".to_string(),
                ..sample_file_item()
            },
        ];

        // act
        let plan = rule.plan(&files).unwrap();

        // assert
        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.entries[0].status, PlanStatus::New);
        assert_eq!(plan.entries[1].status, PlanStatus::New);
    }

    #[test]
    fn test_plan_with_invalid_pattern() {
        let mut rule = sample_rule(MatchMode::Any);
        rule.destination.sub_folder_pattern = "{invalid_token}".to_string();
        let file_one = sample_file_item();
        let file_two = FileItem {
            name: "another_file".to_string(),
            ..sample_file_item()
        };
        let files = vec![file_one, file_two];
        let plan_result = rule.plan(&files);

        assert!(plan_result.is_err());
        assert_eq!(
            plan_result,
            Err(PatternError::UnsupportedToken("invalid_token".to_string()))
        );
    }

    #[test]
    fn test_plan_with_multiple_matching_name_raise_conflict() {
        let mut rule = sample_rule(MatchMode::Any);
        rule.conditions
            .push(Condition::Stem(NameTest::Contains("test".to_string())));
        let file_one = sample_file_item();
        let file_two = sample_file_item();
        let files = vec![file_one, file_two];
        let plan_result = rule.plan(&files);

        assert!(plan_result.is_ok());
        assert!(
            plan_result
                .unwrap()
                .entries
                .iter()
                .find(|entry| entry.status == PlanStatus::Conflict(ConflictKind::InPlan))
                .is_some()
        );
    }

    #[test]
    fn test_plan_with_files_not_matching_conditions() {
        let rule = sample_rule(MatchMode::All);
        let file_item = FileItem {
            name: "non_matching_file".to_string(),
            ..sample_file_item()
        };
        let files = vec![file_item];
        let plan_result = rule.plan(&files);

        assert!(plan_result.is_ok());
        assert_eq!(plan_result.unwrap().entries.len(), 0);
    }

    #[test]
    fn test_plan_with_files_matching_conditions_skipped() {
        let rule = sample_rule(MatchMode::Any);
        let file_item = FileItem {
            name: "test_file".to_string(),
            extension: Some("png".to_string()),
            path: PathBuf::from("/path/to/destination/2023/06/test_file.png"),
            ..sample_file_item()
        };
        let files = vec![file_item];
        let plan_result = rule.plan(&files);

        assert!(plan_result.is_ok());
        let plan = plan_result.unwrap();
        assert_eq!(plan.entries[0].status, PlanStatus::Skipped);
    }
}
