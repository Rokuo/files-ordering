#![allow(dead_code)]
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DateTest {
    After(NaiveDate),
    Before(NaiveDate),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SizeTest {
    LargerThan(u64),
    SmallerThan(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtensionTest {
    IsOneOf(Vec<String>),
    IsNotOneOf(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Condition {
    Stem(NameTest),
    Extension(ExtensionTest),
    Created(DateTest),
    Modified(DateTest),
    Size(SizeTest),
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
}
