#![allow(dead_code)]

use crate::models::destination::Destination;
use crate::models::history::History;
use crate::models::matches::{Condition, MatchMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    pub name: String,
    pub destination: Destination,
    pub match_mode: MatchMode,
    pub conditions: Vec<Condition>,
    pub last_run: Option<History>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::destination::Rename;
    use crate::models::matches::{DateTest, ExtensionTest, NameTest, SizeTest};
    use chrono::NaiveDate;
    use std::path::PathBuf;

    fn sample_rule() -> Rule {
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
            match_mode: MatchMode::All,
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
        let rule: Rule = sample_rule();

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
        let rule: Rule = sample_rule();

        // act
        let json: String = serde_json::to_string(&rule).unwrap();
        // Assert
        assert_eq!(
            json,
            r#"{"name":"Sample Rule","destination":{"folder":"/path/to/destination","sub_folder_pattern":"{year}/{month}","rename":{"pattern":"^IMG_(\\d+)","replacement":"holiday-$1"}},"match_mode":"All","conditions":[{"Stem":{"StartsWith":"test"}},{"Stem":{"Contains":"invoice"}},{"Size":{"LargerThan":1024}},{"Created":{"After":"2023-01-01"}},{"Modified":{"Before":"2024-01-01"}},{"Extension":{"IsOneOf":["jpg","png"]}}],"last_run":null}"#
        );
    }
}
