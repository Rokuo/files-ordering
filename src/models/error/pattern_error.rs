#![allow(dead_code)]
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum PatternError {
    InvalidPattern(String),
    UnsupportedToken(String),
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPattern(pattern) => write!(f, "invalid pattern: '{pattern}'"),
            Self::UnsupportedToken(token) => write!(f, "unsupported pattern token: '{token}'"),
        }
    }
}

impl Error for PatternError {}
