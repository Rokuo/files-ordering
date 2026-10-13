#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::io::{self, ErrorKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveError {
    NotFound,
    PermissionDenied,
    Other(String),
}

impl From<io::Error> for MoveError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            ErrorKind::NotFound => MoveError::NotFound,
            ErrorKind::PermissionDenied => MoveError::PermissionDenied,
            _ => MoveError::Other(value.to_string()),
        }
    }
}
