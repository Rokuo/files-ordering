#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    io::{self, ErrorKind},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveError {
    NotFound,
    PermissionDenied,
    Other(String),
    StorageFull,
}

impl Display for MoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MoveError::NotFound => write!(f, "File not found"),
            MoveError::PermissionDenied => write!(f, "Permission denied"),
            MoveError::Other(msg) => write!(f, "{msg}"),
            MoveError::StorageFull => write!(f, "Not enough space on the destination drive"),
        }
    }
}

impl From<io::Error> for MoveError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            ErrorKind::NotFound => MoveError::NotFound,
            ErrorKind::PermissionDenied => MoveError::PermissionDenied,
            ErrorKind::StorageFull => MoveError::StorageFull,
            _ => MoveError::Other(value.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_io_error_maps_the_kinds_it_knows() {
        assert_eq!(
            MoveError::from(io::Error::from(ErrorKind::NotFound)),
            MoveError::NotFound
        );
        assert_eq!(
            MoveError::from(io::Error::from(ErrorKind::PermissionDenied)),
            MoveError::PermissionDenied
        );
        assert_eq!(
            MoveError::from(io::Error::from(ErrorKind::StorageFull)),
            MoveError::StorageFull
        );
        assert_eq!(
            MoveError::NotFound.to_string(),
            "File not found".to_string()
        );
        assert_eq!(
            MoveError::PermissionDenied.to_string(),
            "Permission denied".to_string()
        );
        assert_eq!(
            MoveError::Other("coucou".to_string()).to_string(),
            "coucou".to_string()
        );
        assert_eq!(
            MoveError::StorageFull.to_string(),
            "Not enough space on the destination drive".to_string()
        );
    }

    #[test]
    fn test_from_io_error_keeps_the_message_of_any_other_kind() {
        let err = io::Error::new(ErrorKind::TimedOut, "the drive stopped answering");

        assert_eq!(
            MoveError::from(err),
            MoveError::Other("the drive stopped answering".to_string())
        );
    }
}
