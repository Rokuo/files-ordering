#![allow(dead_code)]
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum MoveError {
    NotFound,
    PermissionDenied,
    Other(String),
}
