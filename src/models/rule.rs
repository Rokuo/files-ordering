#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use crate::models::matches::MatchMode;
use crate::models::matches::Condition;
use crate::models::destination::Destination;
use crate::models::history::History;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub destination: Destination,
    pub match_mode: MatchMode,
    pub conditions: Vec<Condition>,
    pub last_run: Option<History>,
}