#![allow(dead_code)]

use chrono::Datelike;

pub const NO_EXTENSION: &str = "no-extension";
pub const UNKNOWN_TYPE: &str = "unknown";
pub const UNKNOWN_DATE: &str = "unknown-date";
pub const IMG_TYPES: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "heic", "heif",
];
pub const VID_TYPES: &[&str] = &["mp4", "avi", "mov", "mkv", "flv", "wmv", "webm"];
pub const DOC_TYPES: &[&str] = &["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt"];
pub const RAW_TYPES: &[&str] = &["cr2", "nef", "arw", "dng", "rw2", "orf", "pef"];
const MB: u64 = 1024 * 1024;
const TEN_MB: u64 = 10 * MB;
const HUNDRED_MB: u64 = 100 * MB;
const TYPES: &[(&str, &[&str])] = &[
    ("photos", IMG_TYPES),
    ("raw", RAW_TYPES),
    ("video", VID_TYPES),
    ("documents", DOC_TYPES),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Year,
    Month,
    Day,
    Name,
    Extension,
    Type,
    SizeBand,
}

impl Token {
    pub fn parse(tag: &str) -> Option<Self> {
        match tag {
            "year" => Some(Token::Year),
            "month" => Some(Token::Month),
            "day" => Some(Token::Day),
            "name" => Some(Token::Name),
            "ext" => Some(Token::Extension),
            "type" => Some(Token::Type),
            "size-band" => Some(Token::SizeBand),
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
            Token::Type => {
                if let Some(ext) = &file_item.extension {
                    let ext_lower = ext.to_lowercase();
                    match TYPES
                        .iter()
                        .find(|tuple| tuple.1.contains(&ext_lower.as_str()))
                    {
                        Some(tuple) => tuple.0.to_string(),
                        None => UNKNOWN_TYPE.to_string(),
                    }
                } else {
                    UNKNOWN_TYPE.to_string()
                }
            }
            Token::SizeBand => {
                let size = file_item.size;
                match size {
                    0..MB => "tiny".to_string(),
                    MB..TEN_MB => "small".to_string(),
                    TEN_MB..HUNDRED_MB => "medium".to_string(),
                    _ => "large".to_string(),
                }
            }
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
            size: 5_000_000, // 5 MB
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
        assert_eq!(Token::parse("type"), Some(Token::Type));
        assert_eq!(Token::parse("size-band"), Some(Token::SizeBand));
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
        assert_eq!(Token::Type.value(&file), "documents");
        assert_eq!(Token::SizeBand.value(&file), "small");
    }

    #[test]
    fn test_value_falls_back_when_the_extension_is_missing() {
        let file = FileItem {
            extension: None,
            ..sample_file()
        };

        assert_eq!(Token::Extension.value(&file), NO_EXTENSION);
        assert_eq!(Token::Type.value(&file), UNKNOWN_TYPE);
        assert_eq!(Token::SizeBand.value(&file), "small");
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
        assert_eq!(Token::Type.value(&file), "documents");
        assert_eq!(Token::SizeBand.value(&file), "small");
    }

    #[test]
    fn test_values_of_sizes() {
        let tiny_files = FileItem {
            size: 0,
            ..sample_file()
        };
        let near_small_files = FileItem {
            size: MB - 1,
            ..sample_file()
        };
        let small_files = FileItem {
            size: MB,
            ..sample_file()
        };
        let near_medium_files = FileItem {
            size: TEN_MB - 1,
            ..sample_file()
        };
        let medium_files = FileItem {
            size: TEN_MB,
            ..sample_file()
        };
        let near_large_files = FileItem {
            size: HUNDRED_MB - 1,
            ..sample_file()
        };
        let large_files = FileItem {
            size: HUNDRED_MB,
            ..sample_file()
        };

        assert_eq!(Token::SizeBand.value(&tiny_files), "tiny");
        assert_eq!(Token::SizeBand.value(&near_small_files), "tiny");
        assert_eq!(Token::SizeBand.value(&small_files), "small");
        assert_eq!(Token::SizeBand.value(&near_medium_files), "small");
        assert_eq!(Token::SizeBand.value(&medium_files), "medium");
        assert_eq!(Token::SizeBand.value(&near_large_files), "medium");
        assert_eq!(Token::SizeBand.value(&large_files), "large");
    }

    #[test]
    fn test_value_all_types() {
        let img_file = FileItem {
            extension: Some("jpg".to_string()),
            ..sample_file()
        };
        let raw_file = FileItem {
            extension: Some("cr2".to_string()),
            ..sample_file()
        };
        let vid_file = FileItem {
            extension: Some("mp4".to_string()),
            ..sample_file()
        };
        let doc_file = FileItem {
            extension: Some("pdf".to_string()),
            ..sample_file()
        };
        let ukn_file = FileItem {
            extension: Some("aaaaaaaa".to_string()),
            ..sample_file()
        };

        assert_eq!(Token::Type.value(&img_file), "photos");
        assert_eq!(Token::Type.value(&raw_file), "raw");
        assert_eq!(Token::Type.value(&vid_file), "video");
        assert_eq!(Token::Type.value(&doc_file), "documents");
        assert_eq!(Token::Type.value(&ukn_file), UNKNOWN_TYPE);
    }
}
