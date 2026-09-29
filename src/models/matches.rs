#![allow(dead_code)]
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::models::file_item::FileItem;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchMode {
    Any,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NameTest {
    StartsWith(String),
    EndsWith(String),
    Contains(String),
}

impl NameTest {
    fn matches(&self, arg: &str) -> bool {
        match self {
            NameTest::StartsWith(prefix) => arg.starts_with(prefix),
            NameTest::EndsWith(suffix) => arg.ends_with(suffix),
            NameTest::Contains(substring) => arg.contains(substring),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DateTest {
    After(NaiveDate),
    Before(NaiveDate),
    On(NaiveDate),
}

impl DateTest {
    fn matches(&self, arg: &NaiveDate) -> bool {
        match self {
            DateTest::After(date) => arg > date,
            DateTest::Before(date) => arg < date,
            DateTest::On(date) => arg == date,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SizeTest {
    LargerThan(u64),
    SmallerThan(u64),
}
impl SizeTest {
    fn matches(&self, arg: &u64) -> bool {
        match self {
            SizeTest::LargerThan(size) => arg > size,
            SizeTest::SmallerThan(size) => arg < size,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtensionTest {
    IsOneOf(Vec<String>),
    IsNotOneOf(Vec<String>),
}
impl ExtensionTest {
    fn matches(&self, arg: &str) -> bool {
        match self {
            ExtensionTest::IsOneOf(extensions) => {
                extensions.iter().any(|ext| ext.eq_ignore_ascii_case(arg))
            }
            ExtensionTest::IsNotOneOf(extensions) => {
                !extensions.iter().any(|ext| ext.eq_ignore_ascii_case(arg))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Condition {
    Stem(NameTest),
    Extension(ExtensionTest),
    Created(DateTest),
    Modified(DateTest),
    Size(SizeTest),
}
impl Condition {
    pub fn matches(&self, file: &FileItem) -> bool {
        match self {
            Condition::Stem(name_test) => name_test.matches(&file.name),
            Condition::Extension(extension_test) => {
                extension_test.matches(file.extension.as_deref().unwrap_or_default())
            }
            Condition::Created(date_test) => {
                file.created_at.is_some_and(|date| date_test.matches(&date))
            }
            Condition::Modified(date_test) => file
                .modified_at
                .is_some_and(|date| date_test.matches(&date)),
            Condition::Size(size_test) => size_test.matches(&file.size),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_condition_round_trips_through_json() {
        // arrange
        let condition = Condition::Size(SizeTest::LargerThan(512_000));

        // act
        let json = serde_json::to_string(&condition).unwrap();
        let back: Condition = serde_json::from_str(&json).unwrap();

        // assert
        assert_eq!(condition, back);
    }
    #[test]
    fn test_size_condition_serializes_as_nested_objects() {
        // arrange
        let condition = Condition::Size(SizeTest::LargerThan(512_000));

        // act
        let json = serde_json::to_string(&condition).unwrap();

        // assert
        assert_eq!(json, r#"{"Size":{"LargerThan":512000}}"#);
    }

    #[test]
    fn test_name_condition_matches() {
        // arrange
        let namestart = NameTest::StartsWith("test".to_string());
        let nameend = NameTest::EndsWith("test".to_string());
        let namecontains = NameTest::Contains("test".to_string());

        // act
        let teststart: bool = namestart.matches("test_file");
        let testend: bool = nameend.matches("file_test");
        let testcontains: bool = namecontains.matches("file_test");
        let testnotcontains: bool = namecontains.matches("file");
        let testnotstart: bool = namestart.matches("file_test");
        let testnotend: bool = nameend.matches("test_file");

        // assert
        assert!(teststart);
        assert!(testend);
        assert!(testcontains);
        assert!(!testnotcontains);
        assert!(!testnotstart);
        assert!(!testnotend);
    }

    #[test]
    fn test_date_condition_matches() {
        // arrange
        let dateafter = DateTest::After(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
        let datebefore = DateTest::Before(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());

        // act
        let testafter: bool = dateafter.matches(&NaiveDate::from_ymd_opt(2023, 1, 2).unwrap());
        let testnotafter: bool = dateafter.matches(&NaiveDate::from_ymd_opt(2022, 12, 31).unwrap());
        let testbefore: bool = datebefore.matches(&NaiveDate::from_ymd_opt(2022, 12, 31).unwrap());
        let testnotbefore: bool = datebefore.matches(&NaiveDate::from_ymd_opt(2023, 1, 2).unwrap());

        // assert
        assert!(testafter);
        assert!(!testnotafter);
        assert!(testbefore);
        assert!(!testnotbefore);
    }

    #[test]
    fn test_extension_condition_matches() {
        // arrange
        let extension_is_one_of = ExtensionTest::IsOneOf(vec!["txt".to_string(), "md".to_string()]);
        let extension_is_not_one_of =
            ExtensionTest::IsNotOneOf(vec!["jpg".to_string(), "png".to_string()]);

        // act
        let test_is_one_of: bool = ExtensionTest::matches(&extension_is_one_of, "txt");
        let test_is_not_one_of: bool = ExtensionTest::matches(&extension_is_not_one_of, "txt");

        // assert
        assert!(test_is_one_of);
        assert!(test_is_not_one_of);
    }

    #[test]
    fn test_size_condition_matches() {
        // arrange
        let size_larger_than = SizeTest::LargerThan(512_000);
        let size_smaller_than = SizeTest::SmallerThan(512_000);

        // act
        let test_larger_than: bool = SizeTest::matches(&size_larger_than, &600_000);
        let test_not_larger_than: bool = SizeTest::matches(&size_larger_than, &400_000);
        let test_smaller_than: bool = SizeTest::matches(&size_smaller_than, &400_000);
        let test_not_smaller_than: bool = SizeTest::matches(&size_smaller_than, &600_000);

        // assert
        assert!(test_larger_than);
        assert!(!test_not_larger_than);
        assert!(test_smaller_than);
        assert!(!test_not_smaller_than);
    }

    #[test]
    fn test_condition_condition_matches() {
        // arrange
        let condition_name = Condition::Stem(NameTest::StartsWith("test".to_string()));
        let condition_extension =
            Condition::Extension(ExtensionTest::IsOneOf(vec!["txt".to_string()]));
        let condition_size = Condition::Size(SizeTest::LargerThan(512_000));

        let file_item = FileItem {
            path: std::path::PathBuf::new(),
            name: "test_file".to_string(),
            extension: Some("txt".to_string()),
            size: 600_000,
            destination: None,
            created_at: None,
            modified_at: None,
        };

        // act
        let test_name: bool = condition_name.matches(&file_item);
        let test_extension: bool = condition_extension.matches(&file_item);
        let test_size: bool = condition_size.matches(&file_item);

        // assert
        assert!(test_name);
        assert!(test_extension);
        assert!(test_size);
    }

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn file_with_dates(created_at: Option<NaiveDate>, modified_at: Option<NaiveDate>) -> FileItem {
        FileItem {
            path: std::path::PathBuf::new(),
            name: "test_file".to_string(),
            extension: Some("txt".to_string()),
            size: 600_000,
            destination: None,
            created_at,
            modified_at,
        }
    }

    #[test]
    fn test_date_boundary_is_strict_on_both_sides() {
        let limit = day(2023, 1, 1);

        // the boundary day itself
        assert!(!DateTest::After(limit).matches(&limit));
        assert!(!DateTest::Before(limit).matches(&limit));
        assert!(DateTest::On(limit).matches(&limit));

        // and a day either side is unaffected
        assert!(DateTest::After(limit).matches(&day(2023, 1, 2)));
        assert!(DateTest::Before(limit).matches(&day(2022, 12, 31)));
        assert!(!DateTest::On(limit).matches(&day(2023, 1, 2)));
    }

    #[test]
    fn test_absent_date_never_matches() {
        let file = file_with_dates(None, None);

        assert!(!Condition::Created(DateTest::After(day(2020, 1, 1))).matches(&file));
        assert!(!Condition::Created(DateTest::Before(day(2020, 1, 1))).matches(&file));
        assert!(!Condition::Modified(DateTest::After(day(2020, 1, 1))).matches(&file));
        assert!(!Condition::Modified(DateTest::Before(day(2020, 1, 1))).matches(&file));

        let dated = file_with_dates(Some(day(2023, 6, 15)), None);
        assert!(Condition::Created(DateTest::After(day(2020, 1, 1))).matches(&dated));
    }

    #[test]
    fn test_extension_condition_does_not_match() {
        let is_one_of = ExtensionTest::IsOneOf(vec!["txt".to_string(), "md".to_string()]);
        let is_not_one_of = ExtensionTest::IsNotOneOf(vec!["jpg".to_string(), "png".to_string()]);

        assert!(!is_one_of.matches("jpg"));
        assert!(!is_not_one_of.matches("jpg"));

        assert!(is_one_of.matches("md"));
        assert!(is_not_one_of.matches("md"));
    }
}
