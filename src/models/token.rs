#![allow(dead_code)]

use chrono::Datelike;

pub const NO_EXTENSION: &str = "no-extension";
pub const UNKNOWN_DATE: &str = "unknown-date";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Year,
    Month,
    Day,
    Name,
    Extension,
}

impl Token {
    pub fn parse(tag: &str) -> Option<Self> {
        match tag {
            "year" => Some(Token::Year),
            "month" => Some(Token::Month),
            "day" => Some(Token::Day),
            "name" => Some(Token::Name),
            "ext" => Some(Token::Extension),
            _ => None,
        }
    }

    pub fn value(&self, file_item: &crate::models::file_item::FileItem) -> String {
        match self {
            Token::Year => file_item
                .modified_at
                .map_or_else(|| UNKNOWN_DATE.to_string(), |d| format!("{:04}", d.year())),
            Token::Month => file_item
                .modified_at
                .map_or_else(|| UNKNOWN_DATE.to_string(), |d| format!("{:02}", d.month())),
            Token::Day => file_item
                .modified_at
                .map_or_else(|| UNKNOWN_DATE.to_string(), |d| format!("{:02}", d.day())),
            Token::Name => file_item.name.clone(),
            Token::Extension => file_item
                .extension
                .clone()
                .unwrap_or_else(|| NO_EXTENSION.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::file_item::FileItem;
    use chrono::NaiveDate;

    fn sample_file() -> FileItem {
        FileItem {
            created_at: NaiveDate::from_ymd_opt(2023, 10, 5),
            modified_at: NaiveDate::from_ymd_opt(2024, 3, 14),
            name: "sample".to_string(),
            extension: Some("txt".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_parse() {
        assert_eq!(Token::parse("year"), Some(Token::Year));
        assert_eq!(Token::parse("month"), Some(Token::Month));
        assert_eq!(Token::parse("day"), Some(Token::Day));
        assert_eq!(Token::parse("name"), Some(Token::Name));
        assert_eq!(Token::parse("ext"), Some(Token::Extension));
        assert_eq!(Token::parse("extension"), None);
        assert_eq!(Token::parse("invalid"), None);
    }

    #[test]
    fn test_value() {
        let file = sample_file();
        assert_eq!(Token::Year.value(&file), "2024");
        assert_eq!(Token::Month.value(&file), "03");
        assert_eq!(Token::Day.value(&file), "14");
        assert_eq!(Token::Name.value(&file), "sample");
        assert_eq!(Token::Extension.value(&file), "txt");
    }

    #[test]
    fn test_value_falls_back_when_the_extension_is_missing() {
        let file = FileItem {
            extension: None,
            ..sample_file()
        };

        assert_eq!(Token::Extension.value(&file), NO_EXTENSION);
    }

    #[test]
    fn test_value_falls_back_when_there_is_no_timestamp() {
        let file = FileItem {
            modified_at: None,
            ..sample_file()
        };

        assert_eq!(Token::Year.value(&file), UNKNOWN_DATE);
        assert_eq!(Token::Month.value(&file), UNKNOWN_DATE);
        assert_eq!(Token::Day.value(&file), UNKNOWN_DATE);

        assert_eq!(Token::Name.value(&file), "sample");
    }
}
