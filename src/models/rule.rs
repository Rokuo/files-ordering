#![allow(dead_code)]

use crate::models::destination::Destination;
use crate::models::history::History;
use crate::models::matches::Condition;
use crate::models::matches::MatchMode;
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
    #[test]
    fn test_() {
        // arrange

        // act

        // Assert
    }
}
