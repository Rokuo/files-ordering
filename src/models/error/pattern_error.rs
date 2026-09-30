#![allow(dead_code)]
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum PatternError {
    EmptyPattern,
    InvalidPattern(String),
    UnsupportedToken(String),
    InvalidRange(String),
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPattern => write!(f, "pattern cannot be empty"),
            Self::InvalidPattern(pattern) => write!(f, "invalid pattern: '{pattern}'"),
            Self::UnsupportedToken(token) => write!(f, "unsupported pattern token: '{token}'"),
            Self::InvalidRange(range) => write!(f, "invalid range in pattern: '{range}'"),
        }
    }
}

impl Error for PatternError {}
