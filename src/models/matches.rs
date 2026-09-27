#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchMode {
    Any,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NameTest {
    StartWith(String),
    EndWith(String),
    Contain(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DateTest {
    After(String),
    Before(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SizeTest {
    LargerThan(u64),
    SmallerThan(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionTest {
    IsOneOf(Vec<String>),
    IsNotOneOf(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    Filename(NameTest),
    Extension(ExtensionTest),
    Created(DateTest),
    Modified(DateTest),
    Size(SizeTest)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub condition_type: ConditionType,
    pub value: String,
}